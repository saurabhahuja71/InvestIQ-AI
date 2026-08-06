import 'package:flutter/material.dart';
import 'package:flutter_animate/flutter_animate.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../core/network/api_client.dart';
import '../../../core/network/api_exception.dart';
import '../../../core/widgets/glass_card.dart';
import 'watchlist_providers.dart';

class WatchlistScreen extends ConsumerWidget {
  const WatchlistScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final async = ref.watch(watchlistProvider);
    final scheme = Theme.of(context).colorScheme;

    return Scaffold(
      appBar: AppBar(
        title: const Text('Watchlist'),
        actions: [
          IconButton(
            tooltip: 'IPO alert settings',
            onPressed: () => context.push('/notifications/prefs'),
            icon: const Icon(Icons.notifications_active_outlined),
          ),
          IconButton(
            tooltip: 'Refresh & sync alerts',
            onPressed: () async {
              final dio = ref.read(dioProvider);
              await syncWatchlistAlerts(dio);
              ref.invalidate(watchlistProvider);
            },
            icon: const Icon(Icons.sync),
          ),
        ],
      ),
      body: async.when(
        data: (result) {
          if (result.items.isEmpty) {
            return Center(
              child: Padding(
                padding: const EdgeInsets.all(24),
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Icon(Icons.star_border_rounded, size: 56, color: scheme.outline),
                    const SizedBox(height: 12),
                    Text(
                      'No watched IPOs yet',
                      style: Theme.of(context).textTheme.titleMedium,
                    ),
                    const SizedBox(height: 8),
                    Text(
                      'Tap the star on any IPO card to add it. Your list syncs across devices.',
                      textAlign: TextAlign.center,
                      style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                            color: scheme.onSurfaceVariant,
                          ),
                    ),
                    const SizedBox(height: 16),
                    FilledButton.tonal(
                      onPressed: () => context.go('/ipos'),
                      child: const Text('Browse IPOs'),
                    ),
                  ],
                ),
              ),
            );
          }

          return Column(
            children: [
              if (result.fromCache)
                Container(
                  width: double.infinity,
                  color: scheme.secondaryContainer.withValues(alpha: 0.55),
                  padding:
                      const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
                  child: const Row(
                    children: [
                      Icon(Icons.cloud_off_outlined, size: 18),
                      SizedBox(width: 8),
                      Expanded(
                        child: Text('Showing offline watchlist cache'),
                      ),
                    ],
                  ),
                ),
              Expanded(
                child: RefreshIndicator(
                  onRefresh: () async {
                    final dio = ref.read(dioProvider);
                    await syncWatchlistAlerts(dio);
                    ref.invalidate(watchlistProvider);
                    await ref.read(watchlistProvider.future);
                  },
                  child: ListView.separated(
                    padding: const EdgeInsets.all(16),
                    itemCount: result.items.length,
                    separatorBuilder: (_, __) => const SizedBox(height: 10),
                    itemBuilder: (context, i) {
                      final ipo = result.items[i];
                      return _WatchlistCard(ipo: ipo)
                          .animate()
                          .fadeIn(duration: 200.ms, delay: (20 * (i % 8)).ms)
                          .slideY(begin: 0.03, end: 0);
                    },
                  ),
                ),
              ),
            ],
          );
        },
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (e, _) => Center(
          child: Padding(
            padding: const EdgeInsets.all(24),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                Text('$e', textAlign: TextAlign.center),
                const SizedBox(height: 12),
                FilledButton(
                  onPressed: () => ref.invalidate(watchlistProvider),
                  child: const Text('Retry'),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class _WatchlistCard extends ConsumerWidget {
  const _WatchlistCard({required this.ipo});
  final Map<String, dynamic> ipo;

  static String _na(dynamic v) {
    if (v == null) return 'Not Available';
    final s = v.toString().trim();
    if (s.isEmpty || s == 'null') return 'Not Available';
    return s;
  }

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final scheme = Theme.of(context).colorScheme;
    final id = ipo['id']?.toString() ?? '';
    final name = _na(ipo['company_name']);
    final status = _na(ipo['status']);
    final open = _na(ipo['open_date']);
    final close = _na(ipo['close_date']);
    final listing = _na(ipo['listing_date']);
    final sub = ipo['subscription_total'];
    final subLabel = sub == null ? 'Not Available' : '${sub}x';

    return GlassCard(
      onTap: id.isEmpty ? null : () => context.push('/ipos/$id'),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Expanded(
                child: Text(
                  name,
                  style: const TextStyle(
                    fontWeight: FontWeight.w700,
                    fontSize: 16,
                  ),
                ),
              ),
              IconButton(
                tooltip: 'Remove from watchlist',
                onPressed: id.isEmpty
                    ? null
                    : () async {
                        try {
                          await removeFromWatchlist(ref, id);
                          if (context.mounted) {
                            ScaffoldMessenger.of(context).showSnackBar(
                              const SnackBar(
                                content: Text('Removed from watchlist'),
                              ),
                            );
                          }
                        } catch (e) {
                          if (context.mounted) {
                            ScaffoldMessenger.of(context).showSnackBar(
                              SnackBar(
                                content: Text(
                                  AppException.fromDio(e).message,
                                ),
                              ),
                            );
                          }
                        }
                      },
                icon: Icon(Icons.star_rounded, color: scheme.primary),
              ),
            ],
          ),
          Chip(
            label: Text(status),
            visualDensity: VisualDensity.compact,
            side: BorderSide.none,
            backgroundColor: scheme.secondaryContainer,
          ),
          const SizedBox(height: 8),
          _row(context, 'Open', open),
          _row(context, 'Close', close),
          _row(context, 'Listing', listing),
          _row(context, 'Subscription', subLabel),
        ],
      ),
    );
  }

  Widget _row(BuildContext context, String label, String value) {
    final scheme = Theme.of(context).colorScheme;
    return Padding(
      padding: const EdgeInsets.only(bottom: 4),
      child: Row(
        children: [
          SizedBox(
            width: 100,
            child: Text(
              label,
              style: Theme.of(context).textTheme.labelMedium?.copyWith(
                    color: scheme.onSurfaceVariant,
                  ),
            ),
          ),
          Expanded(
            child: Text(
              value,
              style: Theme.of(context).textTheme.bodyMedium,
            ),
          ),
        ],
      ),
    );
  }
}
