import 'package:connectivity_plus/connectivity_plus.dart';
import 'package:firebase_core/firebase_core.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:hive_flutter/hive_flutter.dart';

import 'core/auth/firebase_config_validator.dart';
import 'core/network/api_base.dart';
import 'core/offline/sync_service.dart';
import 'core/router/app_router.dart';
import 'core/theme/app_theme.dart';
import 'core/theme/theme_controller.dart';
import 'firebase_options.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();

  // Pick a reachable API origin early (tunnel / adb reverse / LAN).
  try {
    await ApiBase.resolve().timeout(const Duration(seconds: 12));
  } catch (_) {}

  // Firebase is optional until CONFIGURATION_REQUIRED.md values are provided.
  if (DefaultFirebaseOptions.isConfigured) {
    try {
      if (Firebase.apps.isEmpty) {
        await Firebase.initializeApp(
          options: DefaultFirebaseOptions.currentPlatform,
        );
      }
      if (kDebugMode) {
        debugPrint(
          'Firebase.initializeApp OK project=${DefaultFirebaseOptions.projectId} '
          'appId=${DefaultFirebaseOptions.appId}',
        );
        // Live probe: surfaces configuration-not-found before the user taps Google.
        final report = await FirebaseConfigValidator.validate();
        debugPrint('Firebase Auth ready=${report.readyForGoogleSignIn}');
        if (!report.readyForGoogleSignIn) {
          debugPrint(report.userFacingSummary);
        }
      }
    } catch (e, st) {
      debugPrint('Firebase.initializeApp failed: $e\n$st');
    }
  } else if (kDebugMode) {
    debugPrint(DefaultFirebaseOptions.configurationHelp);
    debugPrint(
      'Compile-time presence: ${FirebaseConfigValidator.compileTimePresence()}',
    );
  }

  await Hive.initFlutter();
  await Hive.openBox('cache');
  await SystemChrome.setPreferredOrientations([
    DeviceOrientation.portraitUp,
    DeviceOrientation.portraitDown,
  ]);

  FlutterError.onError = (details) {
    FlutterError.presentError(details);
  };

  runApp(const ProviderScope(child: InvestIqApp()));
}

class InvestIqApp extends ConsumerStatefulWidget {
  const InvestIqApp({super.key});

  @override
  ConsumerState<InvestIqApp> createState() => _InvestIqAppState();
}

class _InvestIqAppState extends ConsumerState<InvestIqApp> {
  @override
  void initState() {
    super.initState();
    Connectivity().onConnectivityChanged.listen((results) async {
      final online = results.any((r) => r != ConnectivityResult.none);
      if (online) {
        try {
          await ref.read(syncServiceProvider).flush();
        } catch (_) {}
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    final themeMode = ref.watch(themeModeProvider);
    final router = ref.watch(appRouterProvider);

    return MaterialApp.router(
      title: 'InvestIQ AI',
      debugShowCheckedModeBanner: false,
      theme: AppTheme.light,
      darkTheme: AppTheme.dark,
      themeMode: themeMode,
      routerConfig: router,
      builder: (context, child) {
        ErrorWidget.builder = (details) {
          return Material(
            child: Center(
              child: Padding(
                padding: const EdgeInsets.all(24),
                child: Text(
                  'Something went wrong.\n${details.exceptionAsString()}',
                  textAlign: TextAlign.center,
                ),
              ),
            ),
          );
        };
        return child ?? const SizedBox.shrink();
      },
    );
  }
}
