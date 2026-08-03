import 'package:firebase_auth/firebase_auth.dart';
import 'package:firebase_core/firebase_core.dart';
import 'package:flutter/foundation.dart';
import 'package:google_sign_in/google_sign_in.dart';

import '../../../core/auth/firebase_config_validator.dart';
import '../../../firebase_options.dart';

/// Google Sign-In via Firebase Auth.
///
/// Returns a **Firebase ID token** for `POST /api/v1/auth/google`.
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
    }
    if (!_googleReady && !kIsWeb) {
      final webClientId = DefaultFirebaseOptions.googleWebClientId;
      await GoogleSignIn.instance.initialize(
        clientId: webClientId.isEmpty ? null : webClientId,
        serverClientId: webClientId.isEmpty ? null : webClientId,
      );
      _googleReady = true;
    }
  }

  /// Runs compile-time + live Auth project checks (safe to call often).
  Future<FirebaseConfigReport> validateConfiguration() async {
    try {
      await ensureInitialized();
    } catch (_) {
      // Still probe network with apiKey if present.
    }
    final report = await FirebaseConfigValidator.validate(probeNetwork: true);
    lastReport = report;
    return report;
  }

  Future<String> signInAndGetIdToken() async {
    await ensureInitialized();

    final report = await FirebaseConfigValidator.validate(probeNetwork: true);
    lastReport = report;
    if (report.authProjectConfigured == false) {
      throw StateError(report.userFacingSummary);
    }

    try {
      if (kIsWeb) {
        return await _signInWebFirebasePopup();
      }
      return await _signInMobileGoogleThenFirebase();
    } on FirebaseAuthException catch (e) {
      throw StateError(
        FirebaseConfigValidator.mapAuthException(e),
      );
    } catch (e) {
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

  Future<String> _signInMobileGoogleThenFirebase() async {
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

    final GoogleSignInAccount account =
        await GoogleSignIn.instance.authenticate(
      scopeHint: const ['email', 'profile'],
    );

    final String? googleIdToken = account.authentication.idToken;
    if (googleIdToken == null || googleIdToken.isEmpty) {
      throw StateError(
        'Google did not return an ID token. '
        'Set GOOGLE_WEB_CLIENT_ID to your Web OAuth client ID '
        '(serverClientId). See CONFIGURATION_REQUIRED.md.',
      );
    }

    final credential = GoogleAuthProvider.credential(idToken: googleIdToken);
    final userCred =
        await FirebaseAuth.instance.signInWithCredential(credential);
    return _firebaseIdToken(userCred.user);
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
