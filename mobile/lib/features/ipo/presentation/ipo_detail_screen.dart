import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/constants/app_constants.dart';
import '../../../core/network/api_client.dart';
import '../../../core/widgets/glass_card.dart';
import 'ipo_providers.dart';

class IpoDetailScreen extends ConsumerWidget {
  const IpoDetailScreen({super.key, required this.id});
  final String id;

  String _na(dynamic v) {
    if (v == null) return 'Not available';
    final s = v.toString().trim();
    if (s.isEmpty || s == 'null') return 'Not available';
    return s;
  }

  String _priceLine(Map<String, dynamic> ipo) {
    if (ipo['issue_price'] != null) return '₹${ipo['issue_price']}';
    final low = ipo['price_band_low'];
    final high = ipo['price_band_high'];
    if (low == null && high == null) return 'Not available';
    return '₹${low ?? '—'} – ₹${high ?? '—'}';
  }

  bool _hasList(dynamic raw) => raw is List && raw.isNotEmpty;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final async = ref.watch(ipoDetailProvider(id));
    final scheme = Theme.of(context).colorScheme;

    return Scaffold(
      appBar: AppBar(title: const Text('IPO Details')),
      body: async.when(
        data: (ipo) {
          final gmp = ipo['gmp'] as Map<String, dynamic>?;
          final gmpValue = gmp?['value'] ?? ipo['gmp_value'];
          final shares = ipo['shares_offered'];

          return RefreshIndicator(
            onRefresh: () async {
              ref.invalidate(ipoDetailProvider(id));
              await ref.read(ipoDetailProvider(id).future);
            },
            child: ListView(
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
                  runSpacing: 8,
                  children: [
                    if (ipo['symbol'] != null)
                      Chip(label: Text('${ipo['symbol']}')),
                    Chip(label: Text(ipo['board']?.toString() ?? '')),
                    Chip(label: Text(ipo['status']?.toString() ?? '')),
                    if (ipo['exchange'] != null)
                      Chip(label: Text('${ipo['exchange']}')),
                    if (ipo['sector'] != null)
                      Chip(label: Text('${ipo['sector']}')),
                  ],
                ),
                const SizedBox(height: 16),
                GlassCard(
                  child: Column(
                    children: [
                      _kv(context, 'Issue / band', _priceLine(ipo)),
                      _kv(context, 'Lot size', _na(ipo['lot_size'])),
                      _kv(
                        context,
                        'Shares offered',
                        shares == null ? 'Not available' : '$shares',
                      ),
                      _kv(context, 'Issue size (₹ cr)', _na(ipo['issue_size_cr'])),
                      _kv(context, 'Open', _na(ipo['open_date'])),
                      _kv(context, 'Close', _na(ipo['close_date'])),
                      _kv(context, 'Allotment', _na(ipo['allotment_date'])),
                      _kv(context, 'Listing', _na(ipo['listing_date'])),
                      _kv(
                        context,
                        'Subscription',
                        ipo['subscription_total'] == null
                            ? 'Not available'
                            : '${ipo['subscription_total']}x',
                      ),
                      _kv(context, 'Exchange', _na(ipo['exchange'])),
                      _kv(context, 'Registrar', _na(ipo['registrar'])),
                    ],
                  ),
                ),
                const SizedBox(height: 12),
                if (gmpValue != null)
                  GlassCard(
                    borderColor: Colors.amber.withValues(alpha: 0.6),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Row(
                          children: [
                            const Icon(Icons.warning_amber_rounded,
                                color: Colors.amber),
                            const SizedBox(width: 8),
                            Text(
                              'Grey Market Premium',
                              style: Theme.of(context)
                                  .textTheme
                                  .titleSmall
                                  ?.copyWith(fontWeight: FontWeight.w700),
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
                              child: const Text(
                                'UNOFFICIAL',
                                style: TextStyle(
                                  fontSize: 10,
                                  fontWeight: FontWeight.w800,
                                ),
                              ),
                            ),
                          ],
                        ),
                        const SizedBox(height: 8),
                        Text(
                          '₹$gmpValue',
                          style: Theme.of(context).textTheme.headlineSmall,
                        ),
                        const SizedBox(height: 8),
                        Text(
                          gmp?['disclaimer']?.toString() ??
                              AppConstants.gmpDisclaimer,
                          style:
                              Theme.of(context).textTheme.bodySmall?.copyWith(
                                    color: scheme.onSurfaceVariant,
                                  ),
                        ),
                      ],
                    ),
                  )
                else
                  GlassCard(
                    child: Text(
                      'Grey Market Premium is not published by NSE and is not available for this issue.',
                      style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                            color: scheme.onSurfaceVariant,
                          ),
                    ),
                  ),
                if (_hasList(ipo['pros']) || _hasList(ipo['risks'])) ...[
                  const SizedBox(height: 12),
                  GlassCard(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        if (_hasList(ipo['pros'])) ...[
                          Text('Pros',
                              style: Theme.of(context).textTheme.titleSmall),
                          ..._listItems(ipo['pros']),
                          const SizedBox(height: 12),
                        ],
                        if (_hasList(ipo['risks'])) ...[
                          Text('Risks',
                              style: Theme.of(context).textTheme.titleSmall),
                          ..._listItems(ipo['risks']),
                        ],
                      ],
                    ),
                  ),
                ],
                if (ipo['drhp_url'] != null || ipo['rhp_url'] != null) ...[
                  const SizedBox(height: 12),
                  GlassCard(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text('Documents',
                            style: Theme.of(context).textTheme.titleSmall),
                        if (ipo['drhp_url'] != null)
                          Text('DRHP: ${ipo['drhp_url']}'),
                        if (ipo['rhp_url'] != null)
                          Text('RHP: ${ipo['rhp_url']}'),
                      ],
                    ),
                  ),
                ],
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
                              const SnackBar(
                                content: Text('Added to IPO watchlist'),
                              ),
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
                        onPressed: () => _checkAllotment(context, ref, id),
                        icon: const Icon(Icons.fact_check_outlined),
                        label: const Text('Allotment'),
                      ),
                    ),
                  ],
                ),
                const SizedBox(height: 16),
                Text(
                  'Source: NSE India public IPO APIs. Fields not published by the exchange show as “Not available”.',
                  style: Theme.of(context).textTheme.labelSmall?.copyWith(
                        color: scheme.onSurfaceVariant,
                      ),
                ),
              ],
            ),
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
                  onPressed: () => ref.invalidate(ipoDetailProvider(id)),
                  child: const Text('Retry'),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  Future<void> _checkAllotment(
    BuildContext context,
    WidgetRef ref,
    String ipoId,
  ) async {
    final pan = TextEditingController();
    final appNo = TextEditingController();
    final submitted = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Allotment check'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(
              controller: pan,
              maxLength: 4,
              decoration: const InputDecoration(
                labelText: 'PAN last 4',
                counterText: '',
              ),
            ),
            TextField(
              controller: appNo,
              decoration:
                  const InputDecoration(labelText: 'Application number'),
            ),
            const SizedBox(height: 8),
            Text(
              'Indicative only — confirm with the registrar.',
              style: Theme.of(ctx).textTheme.labelSmall,
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx, false),
            child: const Text('Cancel'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(ctx, true),
            child: const Text('Check'),
          ),
        ],
      ),
    );
    if (submitted != true || !context.mounted) return;
    try {
      final dio = ref.read(dioProvider);
      final res = await dio.post(
        '/ipos/$ipoId/allotment-check',
        data: {
          'pan_last4': pan.text.trim().isEmpty ? null : pan.text.trim(),
          'application_number':
              appNo.text.trim().isEmpty ? null : appNo.text.trim(),
        },
      );
      final data = unwrapData(res, (d) => Map<String, dynamic>.from(d as Map));
      if (context.mounted) {
        showDialog(
          context: context,
          builder: (_) => AlertDialog(
            title: Text('Status: ${data['status']}'),
            content: Text(
              '${data['message']}${data['shares'] != null ? '\n\nShares: ${data['shares']}' : ''}',
            ),
            actions: [
              TextButton(
                onPressed: () => Navigator.pop(context),
                child: const Text('OK'),
              ),
            ],
          ),
        );
      }
    } catch (e) {
      if (context.mounted) {
        ScaffoldMessenger.of(context)
            .showSnackBar(SnackBar(content: Text('$e')));
      }
    }
  }

  Widget _kv(BuildContext context, String k, String v) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        children: [
          Expanded(
            child: Text(k, style: Theme.of(context).textTheme.bodyMedium),
          ),
          Flexible(
            child: Text(
              v,
              textAlign: TextAlign.end,
              style: const TextStyle(fontWeight: FontWeight.w600),
            ),
          ),
        ],
      ),
    );
  }

  List<Widget> _listItems(dynamic raw) {
    if (raw is! List) return [const Text('Not available')];
    return raw.map((e) => Text('• $e')).toList().cast<Widget>();
  }
}
