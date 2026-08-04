import 'dart:convert';

import 'package:firebase_core/firebase_core.dart';
import 'package:flutter/foundation.dart';
import 'package:google_sign_in/google_sign_in.dart';
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

  /// Android Credential Manager often reports config errors as "canceled"
  /// right after the user picks an account.
  static const String androidSignInHelp = '''
Google Sign-In failed after account selection.

Common fixes (do all of these):

1) Firebase Android SHA fingerprints
   Project settings → Android app (ai.investiq.investiq_ai)
   SHA-1:  AC:4E:85:AB:10:45:54:96:A7:26:D2:69:D7:70:1A:25:FB:2D:9E:F4
   SHA-256: BE:5C:92:47:BF:64:20:49:F2:F3:D5:96:87:7E:D5:31:30:12:69:98:4D:17:30:AF:E9:A3:F8:B1:95:88:B2:DC
   Re-download google-services.json after adding.

2) Google Cloud OAuth consent screen (most common after SHA is fixed)
   https://console.cloud.google.com/apis/credentials/consent?project=investiq-ai-a514e
   • If User type is Testing: add YOUR Google email under Test users
   • Or publish the app to Production
   Without this, the account picker closes with no login.

3) Support email set on the consent screen / Firebase Google provider.

4) Rebuild APK after google-services.json change:
   ./scripts/build-android-debug-apk.sh && adb install -r dist/android/investiq-ai-debug.apk

Details: docs/12-android-run.md
''';

  /// Maps [GoogleSignInException] to actionable text.
  static String mapGoogleSignInException(GoogleSignInException e) {
    final detail = [
      if (e.description != null && e.description!.isNotEmpty) e.description,
      if (e.details != null) e.details.toString(),
    ].whereType<String>().join(' | ');

    switch (e.code) {
      case GoogleSignInExceptionCode.canceled:
        // True user cancel OR (very common) misconfigured SHA / consent screen.
        if (!kIsWeb && defaultTargetPlatform == TargetPlatform.android) {
          return '${androidSignInHelp.trim()}'
              '${detail.isEmpty ? '' : '\n\nSDK detail: $detail'}';
        }
        return 'Google sign-in was canceled.';
      case GoogleSignInExceptionCode.clientConfigurationError:
      case GoogleSignInExceptionCode.providerConfigurationError:
        if (!kIsWeb && defaultTargetPlatform == TargetPlatform.android) {
          return '${androidSignInHelp.trim()}'
              '${detail.isEmpty ? '' : '\n\nSDK detail: $detail'}';
        }
        return 'Google Sign-In client is misconfigured.\n'
            'Set GOOGLE_WEB_CLIENT_ID (Web OAuth client) and verify '
            'Firebase Android package + SHA fingerprints.\n'
            'See CONFIGURATION_REQUIRED.md'
            '${detail.isEmpty ? '' : '\n\nSDK detail: $detail'}';
      case GoogleSignInExceptionCode.interrupted:
        return 'Google sign-in was interrupted. Try again.';
      case GoogleSignInExceptionCode.uiUnavailable:
        return 'Google sign-in UI is unavailable on this device.';
      default:
        return 'Google sign-in failed (${e.code.name}): '
            '${detail.isEmpty ? e.toString() : detail}';
    }
  }

  /// Maps FirebaseAuthException codes to actionable text.
  static String mapAuthException(Object error) {
    if (error is GoogleSignInException) {
      return mapGoogleSignInException(error);
    }
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
    // Firebase signInWithProvider → Chrome Custom Tabs OAuth state loss
    if (s.contains('missing initial state') ||
        s.contains('initial_state') ||
        s.contains('INVALID_STATE')) {
      return 'Google browser sign-in lost its session (missing initial state).\n'
          'This build uses native Google Sign-In instead of Chrome.\n'
          'Force-stop the app, reopen, and try Continue with Google again.\n'
          'If Chrome still opens, reinstall the latest APK.';
    }
    // ApiException: 10 / DEVELOPER_ERROR (legacy Play Services path)
    if (s.contains('ApiException: 10') ||
        s.contains('DEVELOPER_ERROR') ||
        s.contains('CommonStatusCodes.DEVELOPER_ERROR')) {
      return androidSignInHelp.trim();
    }
    if (s.contains('network') ||
        s.contains('SocketException') ||
        s.contains('Failed host lookup') ||
        s.contains('Connection refused')) {
      return 'Cannot reach the InvestIQ API after Google sign-in.\n'
          'Phone and PC must be on the same Wi‑Fi. '
          'API default: ${DefaultFirebaseOptions.projectId.isEmpty ? "see API_BASE_URL" : "http://<PC-LAN-IP>:8080"}.\n'
          'Start backend: ./scripts/dev.sh\n'
          'Or: adb reverse tcp:8080 tcp:8080 with API_BASE_URL=http://127.0.0.1:8080\n\n'
          'Detail: $s';
    }
    return s;
  }
}
