// Firebase options for InvestIQ AI.
//
// Values come from compile-time environment (dart-define / dart-define-from-file).
// Do NOT hardcode production secrets here.
//
// See CONFIGURATION_REQUIRED.md and:
//   mobile/config/firebase.dart-define.json.example
//
// flutter run -d chrome \
//   --dart-define=API_BASE_URL=http://127.0.0.1:8080 \
//   --dart-define-from-file=config/firebase.dart-define.json

import 'package:firebase_core/firebase_core.dart' show FirebaseOptions;
import 'package:flutter/foundation.dart'
    show defaultTargetPlatform, kIsWeb, TargetPlatform;

/// Compile-time Firebase + Google OAuth configuration.
class DefaultFirebaseOptions {
  static const apiKey = String.fromEnvironment('FIREBASE_API_KEY');
  static const appId = String.fromEnvironment('FIREBASE_APP_ID');
  static const messagingSenderId =
      String.fromEnvironment('FIREBASE_MESSAGING_SENDER_ID');
  static const projectId = String.fromEnvironment('FIREBASE_PROJECT_ID');
  static const authDomain = String.fromEnvironment('FIREBASE_AUTH_DOMAIN');
  static const googleWebClientId =
      String.fromEnvironment('GOOGLE_WEB_CLIENT_ID');

  /// Optional Android-specific overrides (use when Android app id ≠ Web app id).
  static const androidApiKey = String.fromEnvironment('FIREBASE_ANDROID_API_KEY');
  static const androidAppId = String.fromEnvironment('FIREBASE_ANDROID_APP_ID');
  static const androidMessagingSenderId =
      String.fromEnvironment('FIREBASE_ANDROID_MESSAGING_SENDER_ID');

  static bool get isConfigured =>
      projectId.isNotEmpty &&
      apiKey.isNotEmpty &&
      appId.isNotEmpty &&
      messagingSenderId.isNotEmpty;

  static bool get hasGoogleWebClientId => googleWebClientId.isNotEmpty;

  /// Human-readable checklist of missing compile-time keys.
  static List<String> missingKeys() {
    final missing = <String>[];
    if (projectId.isEmpty) missing.add('FIREBASE_PROJECT_ID');
    if (apiKey.isEmpty) missing.add('FIREBASE_API_KEY');
    if (appId.isEmpty) missing.add('FIREBASE_APP_ID');
    if (messagingSenderId.isEmpty) missing.add('FIREBASE_MESSAGING_SENDER_ID');
    if (authDomain.isEmpty) missing.add('FIREBASE_AUTH_DOMAIN');
    if (googleWebClientId.isEmpty) missing.add('GOOGLE_WEB_CLIENT_ID');
    return missing;
  }

  static String get configurationHelp =>
      'Google Sign-In is not configured.\n\n'
      'Missing: ${missingKeys().join(", ")}.\n\n'
      'Create a Firebase project, enable Google provider, then either:\n'
      '  • copy mobile/config/firebase.dart-define.json.example → '
      'firebase.dart-define.json and run with '
      '--dart-define-from-file=config/firebase.dart-define.json\n'
      '  • or pass FIREBASE_* + GOOGLE_WEB_CLIENT_ID dart-defines.\n\n'
      'Full steps: CONFIGURATION_REQUIRED.md in the repo root.';

  static FirebaseOptions get currentPlatform {
    if (!isConfigured) {
      throw StateError(configurationHelp);
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
      authDomain: authDomain.isNotEmpty ? authDomain : '$projectId.firebaseapp.com',
      storageBucket: '$projectId.appspot.com',
    );
  }

  static const FirebaseOptions ios = FirebaseOptions(
    apiKey: apiKey,
    appId: appId,
    messagingSenderId: messagingSenderId,
    projectId: projectId,
    authDomain: authDomain,
    iosBundleId: 'com.investiq.ai',
  );
}
