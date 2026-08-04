import 'dart:async';

import 'package:firebase_auth/firebase_auth.dart';
import 'package:firebase_core/firebase_core.dart';
import 'package:flutter/foundation.dart';
import 'package:google_sign_in/google_sign_in.dart';

import '../../../core/auth/firebase_config_validator.dart';
import '../../../firebase_options.dart';

void _log(String message) {
  // Always print so `adb logcat | grep InvestIQ-Auth` shows steps.
  // ignore: avoid_print
  print('InvestIQ-Auth: $message');
  if (kDebugMode) {
    debugPrint('InvestIQ-Auth: $message');
  }
}

/// Google Sign-In for InvestIQ.
///
/// **Android/iOS:** uses the native `google_sign_in` Credential Manager sheet
/// (no Chrome / Custom Tabs). That avoids Firebase `signInWithProvider`
/// failures like "missing initial state in chrome".
///
/// **Web:** Firebase popup.
///
/// Returns a Firebase ID token when possible, otherwise the raw Google ID
/// token for `POST /api/v1/auth/google`.
class GoogleAuthService {
  bool _firebaseReady = false;
  bool _googleReady = false;
  FirebaseConfigReport? lastReport;

  bool get isConfigured => DefaultFirebaseOptions.isConfigured;

  Future<void> ensureInitialized() async {
    if (!DefaultFirebaseOptions.isConfigured) {
      throw StateError(DefaultFirebaseOptions.configurationHelp);
    }
    if (!_firebaseReady) {
      if (Firebase.apps.isEmpty) {
        await Firebase.initializeApp(
          options: DefaultFirebaseOptions.currentPlatform,
        );
      }
      _firebaseReady = true;
      _log(
        'Firebase ready project=${DefaultFirebaseOptions.projectId} '
        'androidAppId=${DefaultFirebaseOptions.androidAppId}',
      );
    }
    if (!_googleReady && !kIsWeb) {
      final webClientId = DefaultFirebaseOptions.googleWebClientId;
      // Android/iOS: only serverClientId (Web OAuth client) for ID tokens.
      // Do NOT pass the web client as clientId — that can force a bad flow.
      await GoogleSignIn.instance.initialize(
        serverClientId: webClientId.isEmpty ? null : webClientId,
      );
      _googleReady = true;
      _log(
        'GoogleSignIn initialized serverClientId='
        '${webClientId.isEmpty ? "(empty)" : "${webClientId.substring(0, 24)}…"}',
      );
    }
  }

  Future<FirebaseConfigReport> validateConfiguration() async {
    try {
      await ensureInitialized();
    } catch (_) {}
    final report = await FirebaseConfigValidator.validate(probeNetwork: true);
    lastReport = report;
    return report;
  }

  Future<String> signInAndGetIdToken() async {
    await ensureInitialized();
    _log('signInAndGetIdToken start (native google_sign_in on mobile)');

    unawaited(() async {
      try {
        final report = await FirebaseConfigValidator.validate(probeNetwork: true)
            .timeout(const Duration(seconds: 5));
        lastReport = report;
        _log(
          'config probe authConfigured=${report.authProjectConfigured} '
          'ready=${report.readyForGoogleSignIn}',
        );
      } catch (e) {
        _log('config probe skipped/failed: $e');
      }
    }());

    try {
      if (kIsWeb) {
        return await _signInWebFirebasePopup();
      }
      // Mobile: never use Firebase signInWithProvider — it opens Chrome and
      // often fails with "missing initial state".
      return await _signInWithGoogleSignInPackage().timeout(
        const Duration(seconds: 120),
        onTimeout: () {
          throw StateError(
            'Google sign-in timed out after 2 minutes.\n'
            'Check network, Google Play Services, and try again.',
          );
        },
      );
    } on GoogleSignInException catch (e) {
      _log('GoogleSignInException code=${e.code} desc=${e.description}');
      throw StateError(FirebaseConfigValidator.mapGoogleSignInException(e));
    } on FirebaseAuthException catch (e) {
      _log('FirebaseAuthException code=${e.code} message=${e.message}');
      throw StateError(FirebaseConfigValidator.mapAuthException(e));
    } catch (e, st) {
      _log('signIn error: $e\n$st');
      final mapped = FirebaseConfigValidator.mapAuthException(e);
      if (mapped != e.toString()) {
        throw StateError(mapped);
      }
      rethrow;
    }
  }

  Future<String> _signInWebFirebasePopup() async {
    final provider = GoogleAuthProvider();
    provider.addScope('email');
    provider.addScope('profile');
    provider.setCustomParameters({'prompt': 'select_account'});

    final UserCredential userCred =
        await FirebaseAuth.instance.signInWithPopup(provider);
    return _firebaseIdToken(userCred.user);
  }

  Future<String> _signInWithGoogleSignInPackage() async {
    _log('native google_sign_in.authenticate (no browser)');

    if (!GoogleSignIn.instance.supportsAuthenticate()) {
      throw StateError(
        'Google interactive sign-in is not supported on this platform. '
        'Use Android/iOS device or Chrome web.',
      );
    }

    if (DefaultFirebaseOptions.googleWebClientId.isEmpty) {
      throw StateError(
        'GOOGLE_WEB_CLIENT_ID is required on Android/iOS.\n'
        'Google Cloud Console → APIs & Services → Credentials → '
        'OAuth 2.0 Web client → Client ID.\n'
        'Place in firebase.dart-define.json as GOOGLE_WEB_CLIENT_ID.\n'
        'See CONFIGURATION_REQUIRED.md',
      );
    }

    // Clear stale sessions that can cause silent cancel after account pick.
    try {
      await GoogleSignIn.instance.signOut();
      _log('signed out previous Google session');
    } catch (e) {
      _log('signOut before auth ignored: $e');
    }

    final GoogleSignInAccount account;
    try {
      // No scopeHint: keep auth separate from authorization (recommended).
      account = await GoogleSignIn.instance.authenticate();
    } on GoogleSignInException catch (e) {
      _log(
        'authenticate failed: code=${e.code.name} desc=${e.description} '
        'details=${e.details}',
      );
      rethrow;
    }

    _log('Google account=${account.email} id=${account.id}');

    final String? googleIdToken = account.authentication.idToken;
    if (googleIdToken == null || googleIdToken.isEmpty) {
      throw StateError(
        'Google did not return an ID token.\n'
        'Set GOOGLE_WEB_CLIENT_ID to your Web OAuth client ID '
        '(serverClientId). See CONFIGURATION_REQUIRED.md.',
      );
    }
    _log('Got Google idToken length=${googleIdToken.length}');

    // Exchange for Firebase ID token when possible (backend FIREBASE_PROJECT_ID).
    try {
      final credential = GoogleAuthProvider.credential(idToken: googleIdToken);
      final userCred =
          await FirebaseAuth.instance.signInWithCredential(credential);
      final token = await _firebaseIdToken(userCred.user);
      _log('Firebase signInWithCredential success');
      return token;
    } on FirebaseAuthException catch (e) {
      _log(
        'Firebase signInWithCredential failed: ${e.code} ${e.message} — '
        'using raw Google idToken for backend (GOOGLE_CLIENT_IDS)',
      );
      return googleIdToken;
    }
  }

  Future<String> _firebaseIdToken(User? user) async {
    if (user == null) {
      throw StateError('Firebase did not return a user after Google sign-in');
    }
    final token = await user.getIdToken(true);
    if (token == null || token.isEmpty) {
      throw StateError('Firebase did not return an ID token');
    }
    return token;
  }

  Future<void> signOut() async {
    try {
      if (DefaultFirebaseOptions.isConfigured && Firebase.apps.isNotEmpty) {
        await FirebaseAuth.instance.signOut();
      }
    } catch (_) {}
    try {
      if (!kIsWeb && _googleReady) {
        await GoogleSignIn.instance.signOut();
      }
    } catch (_) {}
  }
}

final googleAuthService = GoogleAuthService();
