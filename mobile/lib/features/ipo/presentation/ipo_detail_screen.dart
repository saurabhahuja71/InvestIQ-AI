import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/constants/app_constants.dart';
import '../../../core/network/api_client.dart';
import '../../../core/widgets/glass_card.dart';
import 'ipo_providers.dart';

class IpoDetailScreen extends ConsumerWidget {
  const IpoDetailScreen({super.key, required this.id});
  final String id;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final async = ref.watch(ipoDetailProvider(id));
    final scheme = Theme.of(context).colorScheme;

    return Scaffold(
      appBar: AppBar(title: const Text('IPO Details')),
      body: async.when(
        data: (ipo) {
          final gmp = ipo['gmp'] as Map<String, dynamic>?;
          return ListView(
            padding: const EdgeInsets.all(16),
            children: [
              Text(
                ipo['company_name']?.toString() ?? '',
                style: Theme.of(context).textTheme.headlineSmall?.copyWith(
                      fontWeight: FontWeight.w700,
                    ),
              ),
              const SizedBox(height: 8),
              Wrap(
                spacing: 8,
                children: [
                  Chip(label: Text(ipo['board']?.toString() ?? '')),
                  Chip(label: Text(ipo['status']?.toString() ?? '')),
                  if (ipo['sector'] != null) Chip(label: Text('${ipo['sector']}')),
                ],
              ),
              const SizedBox(height: 16),
              GlassCard(
                child: Column(
                  children: [
                    _kv(context, 'Issue / band',
                        '${ipo['issue_price'] ?? '${ipo['price_band_low']}–${ipo['price_band_high']}'}'),
                    _kv(context, 'Lot size', '${ipo['lot_size'] ?? '-'}'),
                    _kv(context, 'Open', '${ipo['open_date'] ?? '-'}'),
                    _kv(context, 'Close', '${ipo['close_date'] ?? '-'}'),
                    _kv(context, 'Allotment', '${ipo['allotment_date'] ?? '-'}'),
                    _kv(context, 'Listing', '${ipo['listing_date'] ?? '-'}'),
                    _kv(context, 'Subscription',
                        '${ipo['subscription_total'] ?? '-'}x'),
                  ],
                ),
              ),
              const SizedBox(height: 12),
              GlassCard(
                borderColor: Colors.amber.withValues(alpha: 0.6),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      children: [
                        const Icon(Icons.warning_amber_rounded, color: Colors.amber),
                        const SizedBox(width: 8),
                        Text(
                          'Grey Market Premium',
                          style: Theme.of(context).textTheme.titleSmall?.copyWith(
                                fontWeight: FontWeight.w700,
                              ),
                        ),
                        const Spacer(),
                        Container(
                          padding: const EdgeInsets.symmetric(
                            horizontal: 8,
                            vertical: 4,
                          ),
                          decoration: BoxDecoration(
                            color: Colors.amber.withValues(alpha: 0.2),
                            borderRadius: BorderRadius.circular(8),
                          ),
                          child: const Text('UNOFFICIAL',
                              style: TextStyle(
                                  fontSize: 10, fontWeight: FontWeight.w800)),
                        ),
                      ],
                    ),
                    const SizedBox(height: 8),
                    Text(
                      '₹${gmp?['value'] ?? ipo['gmp_value'] ?? '—'}',
                      style: Theme.of(context).textTheme.headlineSmall,
                    ),
                    const SizedBox(height: 8),
                    Text(
                      gmp?['disclaimer']?.toString() ?? AppConstants.gmpDisclaimer,
                      style: Theme.of(context).textTheme.bodySmall?.copyWith(
                            color: scheme.onSurfaceVariant,
                          ),
                    ),
                  ],
                ),
              ),
              const SizedBox(height: 12),
              GlassCard(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text('Pros', style: Theme.of(context).textTheme.titleSmall),
                    ..._listItems(ipo['pros']),
                    const SizedBox(height: 12),
                    Text('Risks', style: Theme.of(context).textTheme.titleSmall),
                    ..._listItems(ipo['risks']),
                  ],
                ),
              ),
              const SizedBox(height: 12),
              if (ipo['ai_summary'] != null)
                GlassCard(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text('AI Summary',
                          style: Theme.of(context).textTheme.titleSmall),
                      const SizedBox(height: 8),
                      Text(ipo['ai_summary'].toString()),
                      const SizedBox(height: 8),
                      Text(
                        AppConstants.investmentDisclaimer,
                        style: Theme.of(context).textTheme.labelSmall,
                      ),
                    ],
                  ),
                )
              else
                FilledButton.tonalIcon(
                  onPressed: () async {
                    final dio = ref.read(dioProvider);
                    await dio.get('/ipos/$id/ai-summary');
                    ref.invalidate(ipoDetailProvider(id));
                  },
                  icon: const Icon(Icons.auto_awesome),
                  label: const Text('Generate AI summary'),
                ),
              const SizedBox(height: 12),
              Row(
                children: [
                  Expanded(
                    child: OutlinedButton.icon(
                      onPressed: () async {
                        final dio = ref.read(dioProvider);
                        await dio.post('/ipos/$id/watch');
                        if (context.mounted) {
                          ScaffoldMessenger.of(context).showSnackBar(
                            const SnackBar(content: Text('Added to IPO watchlist')),
                          );
                        }
                      },
                      icon: const Icon(Icons.bookmark_add_outlined),
                      label: const Text('Watch'),
                    ),
                  ),
                  const SizedBox(width: 8),
                  Expanded(
                    child: FilledButton.icon(
                      onPressed: () async {
                        final dio = ref.read(dioProvider);
                        final res = await dio.post(
                          '/ipos/$id/allotment-check',
                          data: {},
                        );
                        final data = unwrapData(
                          res,
                          (d) => Map<String, dynamic>.from(d as Map),
                        );
                        if (context.mounted) {
                          showDialog(
                            context: context,
                            builder: (_) => AlertDialog(
                              title: const Text('Allotment'),
                              content: Text(data['message']?.toString() ?? ''),
                              actions: [
                                TextButton(
                                  onPressed: () => Navigator.pop(context),
                                  child: const Text('OK'),
                                ),
                              ],
                            ),
                          );
                        }
                      },
                      icon: const Icon(Icons.fact_check_outlined),
                      label: const Text('Allotment'),
                    ),
                  ),
                ],
              ),
            ],
          );
        },
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (e, _) => Center(child: Text('$e')),
      ),
    );
  }

  Widget _kv(BuildContext context, String k, String v) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        children: [
          Expanded(child: Text(k, style: Theme.of(context).textTheme.bodyMedium)),
          Text(v, style: const TextStyle(fontWeight: FontWeight.w600)),
        ],
      ),
    );
  }

  List<Widget> _listItems(dynamic raw) {
    if (raw is! List) return [const Text('—')];
    return raw.map((e) => Text('• $e')).toList().cast<Widget>();
  }
}
