import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../core/constants/app_constants.dart';
import '../../../core/network/api_client.dart';
import '../../../core/widgets/glass_card.dart';

final journalTradesProvider =
    FutureProvider<List<Map<String, dynamic>>>((ref) async {
  final dio = ref.watch(dioProvider);
  final res = await dio.get('/journal/trades');
  final data = unwrapData(res, (d) => d);
  if (data is List) {
    return data.map((e) => Map<String, dynamic>.from(e as Map)).toList();
  }
  return [];
});

final journalAnalyticsProvider =
    FutureProvider<Map<String, dynamic>>((ref) async {
  final dio = ref.watch(dioProvider);
  final res = await dio.get('/journal/analytics');
  return unwrapData(res, (d) => Map<String, dynamic>.from(d as Map));
});

class JournalScreen extends ConsumerWidget {
  const JournalScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final trades = ref.watch(journalTradesProvider);
    final analytics = ref.watch(journalAnalyticsProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('Trading Journal'),
        actions: [
          IconButton(
            tooltip: 'AI mistake detection',
            icon: const Icon(Icons.psychology_outlined),
            onPressed: () async {
              final dio = ref.read(dioProvider);
              final res = await dio.post('/journal/ai/mistakes');
              final body = unwrapData(
                res,
                (d) => Map<String, dynamic>.from(d as Map),
              );
              if (context.mounted) {
                showModalBottomSheet(
                  context: context,
                  isScrollControlled: true,
                  builder: (_) => Padding(
                    padding: const EdgeInsets.all(20),
                    child: SingleChildScrollView(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text('AI insights',
                              style: Theme.of(context).textTheme.titleLarge),
                          const SizedBox(height: 12),
                          Text(body['insights']?.toString() ?? ''),
                          const SizedBox(height: 12),
                          Text(
                            body['disclaimer']?.toString() ??
                                AppConstants.investmentDisclaimer,
                            style: Theme.of(context).textTheme.labelSmall,
                          ),
                        ],
                      ),
                    ),
                  ),
                );
              }
            },
          ),
        ],
      ),
      floatingActionButton: FloatingActionButton.extended(
        onPressed: () => context.push('/journal/new'),
        icon: const Icon(Icons.add),
        label: const Text('New trade'),
      ),
      body: RefreshIndicator(
        onRefresh: () async {
          ref.invalidate(journalTradesProvider);
          ref.invalidate(journalAnalyticsProvider);
        },
        child: ListView(
          padding: const EdgeInsets.all(16),
          children: [
            analytics.when(
              data: (a) => GlassCard(
                child: Wrap(
                  spacing: 16,
                  runSpacing: 12,
                  children: [
                    _stat('Win rate',
                        '${(a['win_rate'] as num?)?.toStringAsFixed(1) ?? 0}%'),
                    _stat('Total P&L', '₹${a['total_pnl'] ?? 0}'),
                    _stat('Avg win', '₹${a['average_profit'] ?? 0}'),
                    _stat('Avg loss', '₹${a['average_loss'] ?? 0}'),
                    _stat('Best', '₹${a['largest_winner'] ?? 0}'),
                    _stat('Worst', '₹${a['largest_loser'] ?? 0}'),
                  ],
                ),
              ),
              loading: () => const LinearProgressIndicator(),
              error: (_, __) => const SizedBox.shrink(),
            ),
            const SizedBox(height: 16),
            trades.when(
              data: (list) {
                if (list.isEmpty) {
                  return const GlassCard(
                    child: Text('No trades yet. Log your first trade.'),
                  );
                }
                return Column(
                  children: list.map((t) {
                    final pnl = t['pnl'];
                    final profit = pnl is num && pnl >= 0;
                    return Padding(
                      padding: const EdgeInsets.only(bottom: 10),
                      child: GlassCard(
                        child: ListTile(
                          contentPadding: EdgeInsets.zero,
                          title: Text(
                            '${t['symbol']} · ${t['side']}',
                            style: const TextStyle(fontWeight: FontWeight.w700),
                          ),
                          subtitle: Text(
                            [
                              if (t['strategy_name'] != null) t['strategy_name'],
                              if (t['emotion_before'] != null)
                                'Before: ${t['emotion_before']}',
                              if ((t['tags'] as List?)?.isNotEmpty == true)
                                (t['tags'] as List).join(', '),
                            ].whereType<String>().join(' · '),
                          ),
                          trailing: Text(
                            pnl == null ? 'Open' : '₹$pnl',
                            style: TextStyle(
                              fontWeight: FontWeight.w700,
                              color: pnl == null
                                  ? null
                                  : profit
                                      ? Colors.green
                                      : Colors.redAccent,
                            ),
                          ),
                        ),
                      ),
                    );
                  }).toList(),
                );
              },
              loading: () => const Center(child: CircularProgressIndicator()),
              error: (e, _) => Text('$e'),
            ),
            const SizedBox(height: 80),
          ],
        ),
      ),
    );
  }

  Widget _stat(String k, String v) {
    return SizedBox(
      width: 100,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(k, style: const TextStyle(fontSize: 11)),
          Text(v, style: const TextStyle(fontWeight: FontWeight.w700)),
        ],
      ),
    );
  }
}
