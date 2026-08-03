import 'package:fl_chart/fl_chart.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/constants/app_constants.dart';
import '../../../core/network/api_client.dart';
import '../../../core/widgets/glass_card.dart';
import 'portfolio_providers.dart';

class PortfolioScreen extends ConsumerWidget {
  const PortfolioScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final dash = ref.watch(portfolioDashboardProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('Portfolio'),
        actions: [
          IconButton(
            icon: const Icon(Icons.add),
            onPressed: () => _showAddHolding(context, ref),
          ),
        ],
      ),
      body: dash.when(
        data: (data) {
          if (data == null) {
            return Center(
              child: FilledButton(
                onPressed: () => _showAddHolding(context, ref),
                child: const Text('Create first holding'),
              ),
            );
          }
          final analytics = Map<String, dynamic>.from(data['analytics'] as Map);
          final holdings = (data['holdings'] as List? ?? [])
              .map((e) => Map<String, dynamic>.from(e as Map))
              .toList();
          final allocation =
              (analytics['allocation_by_class'] as List? ?? [])
                  .map((e) => Map<String, dynamic>.from(e as Map))
                  .toList();

          return RefreshIndicator(
            onRefresh: () async => ref.invalidate(portfolioDashboardProvider),
            child: ListView(
              padding: const EdgeInsets.all(16),
              children: [
                GlassCard(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      const Text('Total value'),
                      Text(
                        '₹${analytics['total_value']}',
                        style: Theme.of(context).textTheme.headlineMedium?.copyWith(
                              fontWeight: FontWeight.w700,
                            ),
                      ),
                      const SizedBox(height: 8),
                      Wrap(
                        spacing: 12,
                        runSpacing: 8,
                        children: [
                          _metric('Return',
                              '${(analytics['overall_return_pct'] as num?)?.toStringAsFixed(2) ?? '0'}%'),
                          _metric(
                            'XIRR',
                            analytics['xirr'] == null
                                ? '—'
                                : '${((analytics['xirr'] as num) * 100).toStringAsFixed(1)}%',
                          ),
                          _metric(
                            'CAGR',
                            analytics['cagr'] == null
                                ? '—'
                                : '${((analytics['cagr'] as num) * 100).toStringAsFixed(1)}%',
                          ),
                          _metric(
                            'Today',
                            '₹${analytics['today_pnl'] ?? 0} (${(analytics['today_pnl_pct'] as num?)?.toStringAsFixed(2) ?? '0'}%)',
                          ),
                          _metric(
                            'Unrealized',
                            '₹${analytics['unrealized_pnl'] ?? 0}',
                          ),
                        ],
                      ),
                    ],
                  ),
                ),
                const SizedBox(height: 16),
                if (allocation.isNotEmpty)
                  GlassCard(
                    child: SizedBox(
                      height: 180,
                      child: PieChart(
                        PieChartData(
                          sectionsSpace: 2,
                          centerSpaceRadius: 40,
                          sections: allocation.asMap().entries.map((e) {
                            final colors = [
                              Colors.teal,
                              Colors.indigo,
                              Colors.orange,
                              Colors.purple,
                              Colors.blueGrey,
                            ];
                            final pct = (e.value['pct'] as num?)?.toDouble() ?? 0;
                            return PieChartSectionData(
                              value: pct <= 0 ? 0.01 : pct,
                              title: '${e.value['key']}\n${pct.toStringAsFixed(0)}%',
                              radius: 48,
                              titleStyle: const TextStyle(
                                fontSize: 10,
                                color: Colors.white,
                                fontWeight: FontWeight.w600,
                              ),
                              color: colors[e.key % colors.length],
                            );
                          }).toList(),
                        ),
                      ),
                    ),
                  ),
                const SizedBox(height: 16),
                Text('Holdings', style: Theme.of(context).textTheme.titleMedium),
                const SizedBox(height: 8),
                ...holdings.map(
                  (h) => Padding(
                    padding: const EdgeInsets.only(bottom: 8),
                    child: GlassCard(
                      child: ListTile(
                        contentPadding: EdgeInsets.zero,
                        title: Text(h['name']?.toString() ?? ''),
                        subtitle: Text(
                          '${h['asset_class']} · Qty ${h['quantity']} @ ${h['avg_cost']}',
                        ),
                        trailing: Text(
                          h['symbol']?.toString() ?? '',
                          style: const TextStyle(fontWeight: FontWeight.w600),
                        ),
                      ),
                    ),
                  ),
                ),
                const SizedBox(height: 8),
                FilledButton.tonalIcon(
                  onPressed: () async {
                    final portfolios = await ref.read(portfoliosProvider.future);
                    if (portfolios.isEmpty) return;
                    final id = portfolios.first['id'];
                    final dio = ref.read(dioProvider);
                    final res = await dio.post('/portfolios/$id/ai-review');
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
                                Text('AI Portfolio Review',
                                    style: Theme.of(context).textTheme.titleLarge),
                                const SizedBox(height: 12),
                                Text(body['review']?.toString() ?? ''),
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
                  icon: const Icon(Icons.auto_awesome),
                  label: const Text('AI portfolio review'),
                ),
              ],
            ),
          );
        },
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (e, _) => Center(child: Text('$e')),
      ),
    );
  }

  Widget _metric(String label, String value) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(label, style: const TextStyle(fontSize: 12)),
        Text(value, style: const TextStyle(fontWeight: FontWeight.w700)),
      ],
    );
  }

  Future<void> _showAddHolding(BuildContext context, WidgetRef ref) async {
    final name = TextEditingController();
    final symbol = TextEditingController();
    final qty = TextEditingController(text: '1');
    final price = TextEditingController();
    String assetClass = 'stock';

    await showModalBottomSheet(
      context: context,
      isScrollControlled: true,
      builder: (ctx) {
        return Padding(
          padding: EdgeInsets.only(
            left: 16,
            right: 16,
            top: 16,
            bottom: MediaQuery.of(ctx).viewInsets.bottom + 16,
          ),
          child: StatefulBuilder(
            builder: (context, setModal) {
              return Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Text('Add holding',
                      style: Theme.of(context).textTheme.titleLarge),
                  const SizedBox(height: 12),
                  DropdownButtonFormField<String>(
                    initialValue: assetClass,
                    items: const [
                      DropdownMenuItem(value: 'stock', child: Text('Stock')),
                      DropdownMenuItem(value: 'etf', child: Text('ETF')),
                      DropdownMenuItem(
                          value: 'mutual_fund', child: Text('Mutual Fund')),
                      DropdownMenuItem(value: 'gold', child: Text('Gold')),
                      DropdownMenuItem(value: 'bond', child: Text('Bond')),
                      DropdownMenuItem(value: 'cash', child: Text('Cash')),
                    ],
                    onChanged: (v) => setModal(() => assetClass = v ?? 'stock'),
                    decoration: const InputDecoration(labelText: 'Asset class'),
                  ),
                  TextField(
                      controller: name,
                      decoration: const InputDecoration(labelText: 'Name')),
                  TextField(
                      controller: symbol,
                      decoration: const InputDecoration(labelText: 'Symbol')),
                  TextField(
                    controller: qty,
                    keyboardType: TextInputType.number,
                    decoration: const InputDecoration(labelText: 'Quantity'),
                  ),
                  TextField(
                    controller: price,
                    keyboardType: TextInputType.number,
                    decoration: const InputDecoration(labelText: 'Avg cost'),
                  ),
                  const SizedBox(height: 12),
                  FilledButton(
                    onPressed: () async {
                      final portfolios =
                          await ref.read(portfoliosProvider.future);
                      if (portfolios.isEmpty) return;
                      final id = portfolios.first['id'];
                      final dio = ref.read(dioProvider);
                      await dio.post('/portfolios/$id/holdings', data: {
                        'asset_class': assetClass,
                        'name': name.text.trim(),
                        'symbol': symbol.text.trim().isEmpty
                            ? null
                            : symbol.text.trim(),
                        'quantity': qty.text,
                        'avg_cost': price.text,
                      });
                      ref.invalidate(portfolioDashboardProvider);
                      ref.invalidate(portfoliosProvider);
                      if (ctx.mounted) Navigator.pop(ctx);
                    },
                    child: const Text('Save'),
                  ),
                ],
              );
            },
          ),
        );
      },
    );
  }
}
