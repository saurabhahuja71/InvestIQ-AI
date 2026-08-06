import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../features/ai_assistant/presentation/ai_chat_screen.dart';
import '../../features/auth/presentation/auth_controller.dart';
import '../../features/auth/presentation/login_screen.dart';
import '../../features/auth/presentation/register_screen.dart';
import '../../features/home/presentation/home_screen.dart';
import '../../features/ipo/presentation/ipo_detail_screen.dart';
import '../../features/ipo/presentation/ipo_list_screen.dart';
import '../../features/journal/presentation/journal_screen.dart';
import '../../features/journal/presentation/trade_entry_screen.dart';
import '../../features/notifications/presentation/notifications_screen.dart';
import '../../features/portfolio/presentation/portfolio_screen.dart';
import '../../features/settings/presentation/settings_screen.dart';
import '../../features/watchlist/presentation/watchlist_screen.dart';
import '../widgets/main_shell.dart';

final appRouterProvider = Provider<GoRouter>((ref) {
  final auth = ref.watch(authControllerProvider);

  return GoRouter(
    initialLocation: '/',
    refreshListenable: _AuthListenable(ref),
    redirect: (context, state) {
      final loggingIn =
          state.matchedLocation == '/login' || state.matchedLocation == '/register';
      if (auth.status == AuthStatus.unknown) return null;
      if (auth.status == AuthStatus.unauthenticated) {
        return loggingIn ? null : '/login';
      }
      if (loggingIn) return '/';
      return null;
    },
    routes: [
      GoRoute(path: '/login', builder: (_, __) => const LoginScreen()),
      GoRoute(path: '/register', builder: (_, __) => const RegisterScreen()),
      StatefulShellRoute.indexedStack(
        builder: (context, state, navigationShell) {
          return MainShell(navigationShell: navigationShell);
        },
        branches: [
          StatefulShellBranch(routes: [
            GoRoute(path: '/', builder: (_, __) => const HomeScreen()),
          ]),
          StatefulShellBranch(routes: [
            GoRoute(
              path: '/ipos',
              builder: (_, __) => const IpoListScreen(),
              routes: [
                GoRoute(
                  path: ':id',
                  builder: (_, state) =>
                      IpoDetailScreen(id: state.pathParameters['id']!),
                ),
              ],
            ),
          ]),
          StatefulShellBranch(routes: [
            GoRoute(path: '/portfolio', builder: (_, __) => const PortfolioScreen()),
          ]),
          StatefulShellBranch(routes: [
            GoRoute(
              path: '/journal',
              builder: (_, __) => const JournalScreen(),
              routes: [
                GoRoute(
                  path: 'new',
                  builder: (_, __) => const TradeEntryScreen(),
                ),
              ],
            ),
          ]),
          StatefulShellBranch(routes: [
            GoRoute(path: '/ai', builder: (_, __) => const AiChatScreen()),
          ]),
        ],
      ),
      GoRoute(path: '/settings', builder: (_, __) => const SettingsScreen()),
      GoRoute(
        path: '/watchlist',
        builder: (_, __) => const WatchlistScreen(),
      ),
      GoRoute(
        path: '/notifications',
        builder: (_, __) => const NotificationsScreen(),
      ),
      GoRoute(
        path: '/notifications/prefs',
        builder: (_, __) => const NotificationPrefsScreen(),
      ),
    ],
  );
});

class _AuthListenable extends ChangeNotifier {
  _AuthListenable(this.ref) {
    ref.listen(authControllerProvider, (_, __) => notifyListeners());
  }
  final Ref ref;
}
