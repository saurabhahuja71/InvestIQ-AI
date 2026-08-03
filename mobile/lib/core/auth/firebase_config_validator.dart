import 'dart:convert';

import 'package:firebase_core/firebase_core.dart';
import 'package:flutter/foundation.dart';
import 'package:http/http.dart' as http;

import '../../firebase_options.dart';

/// Result of compile-time + live Firebase Auth project checks.
class FirebaseConfigReport {
  const FirebaseConfigReport({
    required this.compileTimeOk,
    required this.missingCompileKeys,
    required this.presentCompileKeys,
    required this.firebaseAppInitialized,
    required this.authProjectConfigured,
    required this.googleProviderLikelyEnabled,
    required this.messages,
  });

  final bool compileTimeOk;
  final List<String> missingCompileKeys;
  final List<String> presentCompileKeys;
  final bool firebaseAppInitialized;
  final bool? authProjectConfigured;
  final bool? googleProviderLikelyEnabled;
  final List<String> messages;

  bool get readyForGoogleSignIn =>
      compileTimeOk &&
      firebaseAppInitialized &&
      authProjectConfigured == true;

  String get userFacingSummary {
    if (readyForGoogleSignIn) {
      return 'Firebase client config and Auth project look ready.';
    }
    return messages.join('\n\n');
  }
}

/// Validates FlutterFire options and probes Firebase Auth project config.
class FirebaseConfigValidator {
  /// Non-secret summary of which dart-define keys are set.
  static Map<String, bool> compileTimePresence() {
    return {
      'FIREBASE_API_KEY': DefaultFirebaseOptions.apiKey.isNotEmpty,
      'FIREBASE_APP_ID': DefaultFirebaseOptions.appId.isNotEmpty,
      'FIREBASE_MESSAGING_SENDER_ID':
          DefaultFirebaseOptions.messagingSenderId.isNotEmpty,
      'FIREBASE_PROJECT_ID': DefaultFirebaseOptions.projectId.isNotEmpty,
      'FIREBASE_AUTH_DOMAIN': DefaultFirebaseOptions.authDomain.isNotEmpty,
      'FIREBASE_STORAGE_BUCKET':
          DefaultFirebaseOptions.storageBucket.isNotEmpty,
      'FIREBASE_MEASUREMENT_ID':
          DefaultFirebaseOptions.measurementId.isNotEmpty,
      'GOOGLE_WEB_CLIENT_ID':
          DefaultFirebaseOptions.googleWebClientId.isNotEmpty,
    };
  }

  static List<String> requiredMissingKeys() {
    final missing = <String>[];
    if (DefaultFirebaseOptions.projectId.isEmpty) {
      missing.add('FIREBASE_PROJECT_ID');
    }
    if (DefaultFirebaseOptions.apiKey.isEmpty) missing.add('FIREBASE_API_KEY');
    if (DefaultFirebaseOptions.appId.isEmpty) missing.add('FIREBASE_APP_ID');
    if (DefaultFirebaseOptions.messagingSenderId.isEmpty) {
      missing.add('FIREBASE_MESSAGING_SENDER_ID');
    }
    if (DefaultFirebaseOptions.authDomain.isEmpty) {
      missing.add('FIREBASE_AUTH_DOMAIN');
    }
    return missing;
  }

  static Future<FirebaseConfigReport> validate({
    bool probeNetwork = true,
  }) async {
    final presence = compileTimePresence();
    final present = presence.entries
        .where((e) => e.value)
        .map((e) => e.key)
        .toList();
    final missingRequired = requiredMissingKeys();
    final messages = <String>[];

    if (missingRequired.isNotEmpty) {
      messages.add(
        'Missing compile-time Firebase keys: ${missingRequired.join(", ")}.\n'
        'Fill mobile/config/firebase.dart-define.json and run with '
        '--dart-define-from-file=config/firebase.dart-define.json\n'
        'See CONFIGURATION_REQUIRED.md',
      );
    }

    if (DefaultFirebaseOptions.googleWebClientId.isEmpty) {
      messages.add(
        'Optional: GOOGLE_WEB_CLIENT_ID is empty. '
        'Required for Android google_sign_in idToken; '
        'Web Firebase popup can work without it once Auth is enabled.',
      );
    }

    final appInitialized = Firebase.apps.isNotEmpty;
    if (!appInitialized && missingRequired.isEmpty) {
      messages.add(
        'Firebase.initializeApp() has not run successfully. '
        'Check main.dart logs.',
      );
    }

    bool? authConfigured;
    bool? googleLikely;

    if (probeNetwork && DefaultFirebaseOptions.apiKey.isNotEmpty) {
      try {
        final projectProbe = await _getProjectConfig(
          DefaultFirebaseOptions.apiKey,
        );
        if (projectProbe == 'CONFIGURATION_NOT_FOUND') {
          authConfigured = false;
          messages.add(
            'Firebase Authentication is NOT enabled for this project '
            '(Identity Toolkit returned CONFIGURATION_NOT_FOUND).\n\n'
            'This is a Firebase Console setup issue — not a missing apiKey/appId.\n\n'
            'Fix:\n'
            '1. Open https://console.firebase.google.com/project/'
            '${DefaultFirebaseOptions.projectId}/authentication\n'
            '2. Click Get started (enable Authentication)\n'
            '3. Sign-in method → Google → Enable → Save\n'
            '4. Set a support email\n'
            '5. Hot restart the app and try again\n\n'
            'Details: CONFIGURATION_REQUIRED.md § "configuration-not-found"',
          );
        } else if (projectProbe == 'OK') {
          authConfigured = true;
        } else if (projectProbe.startsWith('API_KEY')) {
          authConfigured = false;
          messages.add(
            'Firebase API key rejected ($projectProbe). '
            'Copy apiKey again from Project settings → Your apps → Web.',
          );
        } else {
          messages.add('Auth project probe returned: $projectProbe');
        }

        if (authConfigured == true) {
          final googleProbe = await _probeGoogleProvider(
            DefaultFirebaseOptions.apiKey,
          );
          googleLikely = googleProbe;
          if (googleProbe == false) {
            messages.add(
              'Google provider does not appear enabled.\n'
              'Firebase Console → Authentication → Sign-in method → '
              'Google → Enable → Save.',
            );
          }
        }
      } catch (e) {
        messages.add('Could not probe Firebase Auth project: $e');
      }
    }

    if (kDebugMode) {
      debugPrint('Firebase config presence: $presence');
      debugPrint('Auth project configured: $authConfigured');
      for (final m in messages) {
        debugPrint('FirebaseConfig: $m');
      }
    }

    return FirebaseConfigReport(
      compileTimeOk: missingRequired.isEmpty,
      missingCompileKeys: missingRequired,
      presentCompileKeys: present,
      firebaseAppInitialized: appInitialized,
      authProjectConfigured: authConfigured,
      googleProviderLikelyEnabled: googleLikely,
      messages: messages,
    );
  }

  static Future<String> _getProjectConfig(String apiKey) async {
    final uri = Uri.parse(
      'https://www.googleapis.com/identitytoolkit/v3/relyingparty/'
      'getProjectConfig?key=$apiKey',
    );
    final res = await http.get(uri).timeout(const Duration(seconds: 12));
    if (res.statusCode == 200) return 'OK';
    try {
      final body = jsonDecode(res.body) as Map<String, dynamic>;
      final err = body['error'];
      if (err is Map && err['message'] != null) {
        return err['message'].toString();
      }
    } catch (_) {}
    return 'HTTP_${res.statusCode}';
  }

  /// Best-effort: createAuthUri for google.com.
  static Future<bool?> _probeGoogleProvider(String apiKey) async {
    final uri = Uri.parse(
      'https://identitytoolkit.googleapis.com/v1/accounts:createAuthUri?key=$apiKey',
    );
    final res = await http
        .post(
          uri,
          headers: {'Content-Type': 'application/json'},
          body: jsonEncode({
            'continueUri': 'http://localhost',
            'providerId': 'google.com',
          }),
        )
        .timeout(const Duration(seconds: 12));
    if (res.statusCode == 200) return true;
    try {
      final body = jsonDecode(res.body) as Map<String, dynamic>;
      final msg = (body['error'] is Map)
          ? body['error']['message']?.toString() ?? ''
          : '';
      if (msg.contains('OPERATION_NOT_ALLOWED') ||
          msg.contains('CONFIGURATION_NOT_FOUND')) {
        return false;
      }
    } catch (_) {}
    return null;
  }

  /// Maps FirebaseAuthException codes to actionable text.
  static String mapAuthException(Object error) {
    final s = error.toString();
    if (s.contains('configuration-not-found') ||
        s.contains('CONFIGURATION_NOT_FOUND')) {
      return 'Firebase Authentication is not enabled for project '
          '"${DefaultFirebaseOptions.projectId}" (configuration-not-found).\n\n'
          'Open Firebase Console → Authentication → Get started, then enable '
          'Google under Sign-in method. Full steps in CONFIGURATION_REQUIRED.md.';
    }
    if (s.contains('operation-not-allowed') ||
        s.contains('OPERATION_NOT_ALLOWED')) {
      return 'Google sign-in provider is disabled for this Firebase project.\n'
          'Authentication → Sign-in method → Google → Enable → Save.';
    }
    if (s.contains('unauthorized-domain') ||
        s.contains('UNAUTHORIZED_DOMAIN')) {
      return 'This web origin is not authorized.\n'
          'Authentication → Settings → Authorized domains → add localhost '
          'and your Flutter web host/port.';
    }
    if (s.contains('popup-closed-by-user') ||
        s.contains('popup-blocked')) {
      return 'Google sign-in popup was closed or blocked. Allow popups and try again.';
    }
    return s;
  }
}
