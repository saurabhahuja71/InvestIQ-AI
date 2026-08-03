// Firebase options for InvestIQ AI.
//
// Values come from compile-time environment (dart-define / dart-define-from-file).
// Do NOT hardcode production secrets in git.
//
// flutter run -d chrome \
//   --dart-define=API_BASE_URL=http://127.0.0.1:8080 \
//   --dart-define-from-file=config/firebase.dart-define.json
//
// See CONFIGURATION_REQUIRED.md

import 'package:firebase_core/firebase_core.dart' show FirebaseOptions;
import 'package:flutter/foundation.dart'
    show defaultTargetPlatform, kIsWeb, TargetPlatform;

/// Compile-time Firebase + Google OAuth configuration (FlutterFire-compatible).
class DefaultFirebaseOptions {
  static const apiKey = String.fromEnvironment('FIREBASE_API_KEY');
  static const appId = String.fromEnvironment('FIREBASE_APP_ID');
  static const messagingSenderId =
      String.fromEnvironment('FIREBASE_MESSAGING_SENDER_ID');
  static const projectId = String.fromEnvironment('FIREBASE_PROJECT_ID');
  static const authDomain = String.fromEnvironment('FIREBASE_AUTH_DOMAIN');
  static const storageBucket = String.fromEnvironment('FIREBASE_STORAGE_BUCKET');
  static const measurementId = String.fromEnvironment('FIREBASE_MEASUREMENT_ID');
  static const googleWebClientId =
      String.fromEnvironment('GOOGLE_WEB_CLIENT_ID');

  static const androidApiKey =
      String.fromEnvironment('FIREBASE_ANDROID_API_KEY');
  static const androidAppId = String.fromEnvironment('FIREBASE_ANDROID_APP_ID');
  static const androidMessagingSenderId =
      String.fromEnvironment('FIREBASE_ANDROID_MESSAGING_SENDER_ID');

  /// Required for Firebase.initializeApp + Auth on web/android.
  static bool get isConfigured =>
      projectId.isNotEmpty &&
      apiKey.isNotEmpty &&
      appId.isNotEmpty &&
      messagingSenderId.isNotEmpty &&
      authDomain.isNotEmpty;

  static bool get hasGoogleWebClientId => googleWebClientId.isNotEmpty;

  static String get effectiveStorageBucket => storageBucket.isNotEmpty
      ? storageBucket
      : (projectId.isNotEmpty ? '$projectId.appspot.com' : '');

  static List<String> missingKeys() {
    final missing = <String>[];
    if (projectId.isEmpty) missing.add('FIREBASE_PROJECT_ID');
    if (apiKey.isEmpty) missing.add('FIREBASE_API_KEY');
    if (appId.isEmpty) missing.add('FIREBASE_APP_ID');
    if (messagingSenderId.isEmpty) missing.add('FIREBASE_MESSAGING_SENDER_ID');
    if (authDomain.isEmpty) missing.add('FIREBASE_AUTH_DOMAIN');
    if (googleWebClientId.isEmpty) missing.add('GOOGLE_WEB_CLIENT_ID (optional on web)');
    return missing;
  }

  static String get configurationHelp =>
      'Google Sign-In client config incomplete.\n\n'
      'Missing: ${missingKeys().join(", ")}.\n\n'
      'Copy mobile/config/firebase.dart-define.json.example → '
      'firebase.dart-define.json, fill values from Firebase Console, run with '
      '--dart-define-from-file=config/firebase.dart-define.json\n\n'
      'If keys are present but you still see configuration-not-found: '
      'enable Authentication + Google provider in Firebase Console.\n'
      'See CONFIGURATION_REQUIRED.md';

  static FirebaseOptions get currentPlatform {
    if (!isConfigured) {
      throw StateError(configurationHelp);
    }
    if (kIsWeb) return web;
    switch (defaultTargetPlatform) {
      case TargetPlatform.android:
        return android;
      case TargetPlatform.iOS:
        return ios;
      default:
        return web;
    }
  }

  static FirebaseOptions get web => FirebaseOptions(
        apiKey: apiKey,
        appId: appId,
        messagingSenderId: messagingSenderId,
        projectId: projectId,
        authDomain: authDomain,
        storageBucket: effectiveStorageBucket,
        measurementId: measurementId.isEmpty ? null : measurementId,
      );

  static FirebaseOptions get android {
    final key = androidApiKey.isNotEmpty ? androidApiKey : apiKey;
    final id = androidAppId.isNotEmpty ? androidAppId : appId;
    final sender = androidMessagingSenderId.isNotEmpty
        ? androidMessagingSenderId
        : messagingSenderId;
    return FirebaseOptions(
      apiKey: key,
      appId: id,
      messagingSenderId: sender,
      projectId: projectId,
      authDomain: authDomain,
      storageBucket: effectiveStorageBucket,
    );
  }

  static FirebaseOptions get ios => FirebaseOptions(
        apiKey: apiKey,
        appId: appId,
        messagingSenderId: messagingSenderId,
        projectId: projectId,
        authDomain: authDomain,
        storageBucket: effectiveStorageBucket,
        iosBundleId: 'com.investiq.ai',
      );
}
