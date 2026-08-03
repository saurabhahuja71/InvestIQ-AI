import 'package:firebase_auth/firebase_auth.dart';
import 'package:firebase_core/firebase_core.dart';
import 'package:flutter/foundation.dart';
import 'package:google_sign_in/google_sign_in.dart';

import '../../../firebase_options.dart';

/// Google Sign-In via Firebase Auth. Returns a Firebase ID token for the API.
class GoogleAuthService {
  bool _firebaseReady = false;
  bool _googleReady = false;

  Future<void> ensureInitialized() async {
    if (!DefaultFirebaseOptions.isConfigured) {
      throw StateError(
        'Google Sign-In is not configured. Create a Firebase project, enable '
        'Google provider, and pass FIREBASE_* + GOOGLE_WEB_CLIENT_ID dart-defines.',
      );
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
      await GoogleSignIn.instance.initialize(
        clientId: DefaultFirebaseOptions.googleWebClientId.isEmpty
            ? null
            : DefaultFirebaseOptions.googleWebClientId,
        serverClientId: DefaultFirebaseOptions.googleWebClientId.isEmpty
            ? null
            : DefaultFirebaseOptions.googleWebClientId,
      );
      _googleReady = true;
    }
  }

  Future<String> signInAndGetIdToken() async {
    await ensureInitialized();

    if (kIsWeb) {
      final provider = GoogleAuthProvider();
      provider.addScope('email');
      provider.addScope('profile');
      final userCred =
          await FirebaseAuth.instance.signInWithPopup(provider);
      final user = userCred.user;
      if (user == null) {
        throw StateError('Firebase did not return a user');
      }
      final token = await user.getIdToken(true);
      if (token == null || token.isEmpty) {
        throw StateError('Firebase did not return an ID token');
      }
      return token;
    }

    if (!GoogleSignIn.instance.supportsAuthenticate()) {
      throw StateError(
        'Google interactive sign-in is not supported on this platform.',
      );
    }

    final GoogleSignInAccount account =
        await GoogleSignIn.instance.authenticate(
      scopeHint: const ['email', 'profile'],
    );

    final String? idToken = account.authentication.idToken;
    if (idToken == null || idToken.isEmpty) {
      throw StateError('Google did not return an ID token');
    }

    final credential = GoogleAuthProvider.credential(idToken: idToken);
    final userCred =
        await FirebaseAuth.instance.signInWithCredential(credential);
    final user = userCred.user;
    if (user == null) {
      throw StateError('Firebase did not return a user');
    }

    final token = await user.getIdToken(true);
    if (token == null || token.isEmpty) {
      throw StateError('Firebase did not return an ID token');
    }
    return token;
  }

  Future<void> signOut() async {
    try {
      if (DefaultFirebaseOptions.isConfigured) {
        await ensureInitialized();
        await FirebaseAuth.instance.signOut();
      }
    } catch (_) {}
    try {
      if (!kIsWeb) {
        await GoogleSignIn.instance.signOut();
      }
    } catch (_) {}
  }
}

final googleAuthService = GoogleAuthService();
