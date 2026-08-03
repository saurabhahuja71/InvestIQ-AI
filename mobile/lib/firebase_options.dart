// File generated for InvestIQ AI Firebase options.
// Populate via --dart-define or replace values from Firebase Console.
//
// flutter run -d chrome \
//   --dart-define=FIREBASE_API_KEY=... \
//   --dart-define=FIREBASE_APP_ID=... \
//   --dart-define=FIREBASE_MESSAGING_SENDER_ID=... \
//   --dart-define=FIREBASE_PROJECT_ID=... \
//   --dart-define=FIREBASE_AUTH_DOMAIN=... \
//   --dart-define=GOOGLE_WEB_CLIENT_ID=...

import 'package:firebase_core/firebase_core.dart' show FirebaseOptions;
import 'package:flutter/foundation.dart'
    show defaultTargetPlatform, kIsWeb, TargetPlatform;

class DefaultFirebaseOptions {
  static const apiKey = String.fromEnvironment('FIREBASE_API_KEY');
  static const appId = String.fromEnvironment('FIREBASE_APP_ID');
  static const messagingSenderId =
      String.fromEnvironment('FIREBASE_MESSAGING_SENDER_ID');
  static const projectId = String.fromEnvironment('FIREBASE_PROJECT_ID');
  static const authDomain = String.fromEnvironment('FIREBASE_AUTH_DOMAIN');
  static const googleWebClientId =
      String.fromEnvironment('GOOGLE_WEB_CLIENT_ID');

  static bool get isConfigured =>
      projectId.isNotEmpty && apiKey.isNotEmpty && appId.isNotEmpty;

  static FirebaseOptions get currentPlatform {
    if (!isConfigured) {
      throw StateError(
        'Firebase is not configured. Pass FIREBASE_* and GOOGLE_WEB_CLIENT_ID '
        'dart-defines from your Firebase / Google Cloud project.',
      );
    }
    if (kIsWeb) {
      return web;
    }
    switch (defaultTargetPlatform) {
      case TargetPlatform.android:
        return android;
      case TargetPlatform.iOS:
        return ios;
      default:
        return web;
    }
  }

  static const FirebaseOptions web = FirebaseOptions(
    apiKey: apiKey,
    appId: appId,
    messagingSenderId: messagingSenderId,
    projectId: projectId,
    authDomain: authDomain,
  );

  // Replace with Android app config when shipping Play Store build.
  static const FirebaseOptions android = FirebaseOptions(
    apiKey: apiKey,
    appId: appId,
    messagingSenderId: messagingSenderId,
    projectId: projectId,
    authDomain: authDomain,
  );

  static const FirebaseOptions ios = FirebaseOptions(
    apiKey: apiKey,
    appId: appId,
    messagingSenderId: messagingSenderId,
    projectId: projectId,
    authDomain: authDomain,
    iosBundleId: 'com.investiq.ai',
  );
}
