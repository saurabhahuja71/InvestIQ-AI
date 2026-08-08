import 'package:flutter/material.dart';
import 'package:flutter_animate/flutter_animate.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:shimmer/shimmer.dart';
import 'package:url_launcher/url_launcher.dart';

import '../../../core/constants/app_constants.dart';
import '../../../core/network/api_client.dart';
import '../../../core/widgets/glass_card.dart';
import '../../watchlist/presentation/watchlist_providers.dart';
import 'ipo_providers.dart';

class IpoDetailScreen extends ConsumerWidget {
  const IpoDetailScreen({super.key, required this.id});
  final String id;

  static const _na = 'Not Available';

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final async = ref.watch(ipoDetailProvider(id));
    final watched = ref.watch(watchedIpoIdsProvider).contains(id);
    final scheme = Theme.of(context).colorScheme;

    return Scaffold(
      appBar: AppBar(
        title: const Text('IPO Details'),
        actions: [
          IconButton(
            tooltip: watched ? 'Remove from watchlist' : 'Add to watchlist',
            onPressed: () async {
              final snap = async.asData?.value;
              await toggleWatchlist(ref, id, watched, snapshot: snap);
              if (context.mounted) {
                ScaffoldMessenger.of(context).showSnackBar(
                  SnackBar(
                    content: Text(
                      watched ? 'Removed from watchlist' : 'Added to watchlist',
                    ),
                  ),
                );
              }
            },
            icon: Icon(
              watched ? Icons.star_rounded : Icons.star_outline_rounded,
              color: watched ? scheme.primary : null,
            ),
          ),
          IconButton(
            tooltip: 'Refresh',
            onPressed: () {
              ref.invalidate(ipoDetailProvider(id));
              ref.invalidate(ipoScoreProvider(id));
              ref.invalidate(ipoSubscriptionProvider(id));
              ref.invalidate(ipoFinancialsProvider(id));
            },
            icon: const Icon(Icons.refresh),
          ),
        ],
      ),
      body: async.when(
        data: (ipo) => RefreshIndicator(
          onRefresh: () async {
            ref.invalidate(ipoDetailProvider(id));
            ref.invalidate(ipoScoreProvider(id));
            ref.invalidate(ipoSubscriptionProvider(id));
            ref.invalidate(ipoFinancialsProvider(id));
          },
          child: ListView(
            padding: const EdgeInsets.all(16),
            children: [
              _Header(ipo: ipo),
              const SizedBox(height: 12),
              _Section(
                title: 'Overview',
                child: Text(
                  _str(ipo['description']),
                  style: Theme.of(context).textTheme.bodyMedium,
                ),
              ),
              const SizedBox(height: 12),
              _Section(
                title: 'IPO Details',
                child: Column(
                  children: [
                    _kv(context, 'Exchange', _str(ipo['exchange'])),
                    _kv(context, 'Board', _str(ipo['board'])),
                    _kv(context, 'Status', _str(ipo['status'])),
                    _kv(context, 'Industry', _str(ipo['industry'] ?? ipo['sector'])),
                    _kv(context, 'Issue Type', _str(ipo['issue_type'])),
                    _kv(context, 'Price Band', _priceBand(ipo)),
                    _kv(context, 'Issue Price', _money(ipo['issue_price'])),
                    _kv(context, 'Lot Size', _str(ipo['lot_size'])),
                    _kv(context, 'Minimum Investment', _money(ipo['min_investment'])),
                    _kv(context, 'Face Value', _money(ipo['face_value'])),
                    _kv(context, 'Issue Size (Rs Cr)', _str(ipo['issue_size_cr'])),
                    _kv(context, 'Registrar', _str(ipo['registrar'])),
                    _kv(context, 'Lead Managers', _leadManagers(ipo['lead_managers'])),
                    _kv(context, 'Data source', _str(ipo['source'] ?? 'nse')),
                    _kv(context, 'Last synced', _str(ipo['source_synced_at'])),
                  ],
                ),
              ),
              const SizedBox(height: 12),
              _Section(
                title: 'Important Dates',
                child: Column(
                  children: [
                    _kv(context, 'Open Date', _str(ipo['open_date'])),
                    _kv(context, 'Close Date', _str(ipo['close_date'])),
                    _kv(context, 'Allotment Date', _str(ipo['allotment_date'])),
                    _kv(context, 'Refund Date', _str(ipo['refund_date'])),
                    _kv(context, 'Listing Date', _str(ipo['listing_date'])),
                  ],
                ),
              ),
              const SizedBox(height: 12),
              _Section(
                title: 'Subscription',
                child: _SubscriptionSection(id: id),
              ),
              const SizedBox(height: 12),
              _Section(
                title: 'Financial Performance',
                child: _FinancialsSection(id: id),
              ),
              const SizedBox(height: 12),
              _Section(
                title: 'Growth Analysis',
                child: _GrowthSection(id: id),
              ),
              const SizedBox(height: 12),
              _Section(
                title: 'Valuation',
                child: _ValuationSection(id: id),
              ),
              const SizedBox(height: 12),
              _Section(
                title: 'InvestIQ Score',
                child: _ScoreSection(id: id),
              ),
              if (_hasList(ipo['pros']) || _hasList(ipo['risks'])) ...[
                const SizedBox(height: 12),
                _Section(
                  title: 'Risk Factors',
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      if (_hasList(ipo['pros'])) ...[
                        Text('Pros', style: Theme.of(context).textTheme.titleSmall),
                        ..._listItems(ipo['pros']),
                        const SizedBox(height: 12),
                      ],
                      if (_hasList(ipo['risks'])) ...[
                        Text('Risks', style: Theme.of(context).textTheme.titleSmall),
                        ..._listItems(ipo['risks']),
                      ],
                    ],
                  ),
                ),
              ],
              const SizedBox(height: 12),
              _Section(
                title: 'Official Documents',
                child: Column(
                  children: [
                    _LinkTile(
                      label: 'Prospectus (RHP/DRHP)',
                      url: _firstUrl(ipo['prospectus_url'], ipo['rhp_url'], ipo['drhp_url']),
                    ),
                    _LinkTile(
                      label: 'Company website',
                      url: ipo['website']?.toString(),
                    ),
                    _LinkTile(
                      label: 'Ratios / basis of issue price',
                      url: _nestedUrl(ipo['financials'], 'ratios_basis_of_issue_price_url'),
                    ),
                  ],
                ),
              ),
              const SizedBox(height: 12),
              if (ipo['ai_summary'] != null)
                _Section(
                  title: 'AI summary',
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
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
                        await toggleWatchlist(
                          ref,
                          id,
                          watched,
                          snapshot: ipo,
                        );
                        if (context.mounted) {
                          ScaffoldMessenger.of(context).showSnackBar(
                            SnackBar(
                              content: Text(
                                watched
                                    ? 'Removed from watchlist'
                                    : 'Added to watchlist',
                              ),
                            ),
                          );
                        }
                      },
                      icon: Icon(
                        watched
                            ? Icons.star_rounded
                            : Icons.star_outline_rounded,
                      ),
                      label: Text(watched ? 'Watching' : 'Watch'),
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
              const SizedBox(height: 24),
            ]
                .animate(interval: 40.ms)
                .fadeIn(duration: 200.ms)
                .slideY(begin: 0.03, end: 0),
          ),
        ),
        loading: () => const _DetailSkeleton(),
        error: (e, _) => _DetailError(
          message: '$e',
          onRetry: () => ref.invalidate(ipoDetailProvider(id)),
        ),
      ),
    );
  }

  static String _str(dynamic v) {
    if (v == null) return _na;
    final s = v.toString().trim();
    if (s.isEmpty || s == 'null' || s == '[]' || s == '{}') return _na;
    return s;
  }

  static String _num(dynamic v) {
    if (v == null) return _na;
    final d = double.tryParse('$v');
    if (d == null) return v.toString();
    if (d == d.roundToDouble()) return d.toInt().toString();
    var s = d.toStringAsFixed(2);
    s = s.replaceAll(RegExp(r'0+$'), '').replaceAll(RegExp(r'\.$'), '');
    return s;
  }

  static String _money(dynamic v) {
    if (v == null) return _na;
    return 'Rs ${_num(v)}';
  }

  static String _times(dynamic v) {
    if (v == null) return _na;
    return '${_num(v)}x';
  }

  static String _pct(dynamic v) {
    if (v == null) return _na;
    return '${_num(v)}%';
  }

  static String _priceBand(Map<String, dynamic> ipo) {
    final low = ipo['price_band_low'];
    final high = ipo['price_band_high'];
    if (low == null && high == null) return _na;
    return 'Rs ${_num(low)} - Rs ${_num(high)}';
  }

  static String _leadManagers(dynamic raw) {
    if (raw is List && raw.isNotEmpty) {
      return raw.map((e) => e.toString()).join(', ');
    }
    return _na;
  }

  static bool _hasList(dynamic raw) => raw is List && raw.isNotEmpty;

  static String? _firstUrl(dynamic a, dynamic b, dynamic c) {
    for (final v in [a, b, c]) {
      final s = v?.toString();
      if (s != null && s.startsWith('http')) return s;
    }
    return null;
  }

  static String? _nestedUrl(dynamic financials, String key) {
    if (financials is Map && financials[key] != null) {
      final s = financials[key].toString();
      if (s.startsWith('http')) return s;
    }
    return null;
  }

  static Future<void> _checkAllotment(
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
              decoration: const InputDecoration(labelText: 'Application number'),
            ),
            const SizedBox(height: 8),
            Text(
              'Indicative only - confirm with the registrar.',
              style: Theme.of(ctx).textTheme.labelSmall,
            ),
          ],
        ),
        actions: [
          TextButton(onPressed: () => Navigator.pop(ctx, false), child: const Text('Cancel')),
          FilledButton(onPressed: () => Navigator.pop(ctx, true), child: const Text('Check')),
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
        ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text('$e')));
      }
    }
  }

  static Widget _kv(BuildContext context, String k, String v) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 6),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Expanded(
            flex: 4,
            child: Text(k, style: Theme.of(context).textTheme.bodyMedium),
          ),
          Expanded(
            flex: 6,
            child: Text(
              v,
              textAlign: TextAlign.end,
              style: TextStyle(
                fontWeight: FontWeight.w600,
                color: v == _na
                    ? Theme.of(context).colorScheme.outline
                    : null,
              ),
            ),
          ),
        ],
      ),
    );
  }

  static List<Widget> _listItems(dynamic raw) {
    if (raw is! List || raw.isEmpty) {
      return [const Text(_na)];
    }
    return raw.map((e) => Text('• $e')).toList().cast<Widget>();
  }
}

class _Header extends StatelessWidget {
  const _Header({required this.ipo});
  final Map<String, dynamic> ipo;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    final name = ipo['company_name']?.toString() ?? 'Not Available';
    final logo = ipo['logo_url']?.toString();
    final hasLogo = logo != null && logo.startsWith('http');

    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        CircleAvatar(
          radius: 28,
          backgroundColor: scheme.primaryContainer,
          foregroundColor: scheme.onPrimaryContainer,
          backgroundImage: hasLogo ? NetworkImage(logo) : null,
          onBackgroundImageError: hasLogo ? (_, __) {} : null,
          child: hasLogo
              ? null
              : Text(
                  _initials(name),
                  style: const TextStyle(fontWeight: FontWeight.w800),
                ),
        ),
        const SizedBox(width: 14),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                name,
                style: Theme.of(context).textTheme.headlineSmall?.copyWith(
                      fontWeight: FontWeight.w700,
                    ),
              ),
              const SizedBox(height: 8),
              Wrap(
                spacing: 8,
                runSpacing: 6,
                children: [
                  if (ipo['symbol'] != null) Chip(label: Text('${ipo['symbol']}')),
                  Chip(label: Text('${ipo['board'] ?? 'Not Available'}')),
                  Chip(label: Text('${ipo['status'] ?? 'Not Available'}')),
                  if (ipo['sector'] != null) Chip(label: Text('${ipo['sector']}')),
                ],
              ),
            ],
          ),
        ),
      ],
    );
  }

  String _initials(String name) {
    final parts = name.trim().split(RegExp(r'\s+'));
    if (parts.isEmpty || parts.first.isEmpty) return '?';
    if (parts.length == 1) {
      final s = parts.first;
      return s.substring(0, s.length >= 2 ? 2 : 1).toUpperCase();
    }
    return '${parts[0][0]}${parts[1][0]}'.toUpperCase();
  }
}

class _Section extends StatelessWidget {
  const _Section({required this.title, required this.child});
  final String title;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    return GlassCard(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(title, style: Theme.of(context).textTheme.titleSmall?.copyWith(
                fontWeight: FontWeight.w700,
              )),
          const SizedBox(height: 8),
          child,
        ],
      ),
    );
  }
}

class _InlineLoading extends StatelessWidget {
  const _InlineLoading();

  @override
  Widget build(BuildContext context) {
    return const Padding(
      padding: EdgeInsets.symmetric(vertical: 16),
      child: Center(
        child: SizedBox(
          width: 22,
          height: 22,
          child: CircularProgressIndicator(strokeWidth: 2),
        ),
      ),
    );
  }
}

class _InlineError extends StatelessWidget {
  const _InlineError({required this.message, required this.onRetry});
  final String message;
  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        Expanded(
          child: Text(
            message,
            style: Theme.of(context).textTheme.bodySmall?.copyWith(
                  color: Theme.of(context).colorScheme.error,
                ),
          ),
        ),
        TextButton(onPressed: onRetry, child: const Text('Retry')),
      ],
    );
  }
}

class _SubscriptionSection extends ConsumerWidget {
  const _SubscriptionSection({required this.id});
  final String id;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final async = ref.watch(ipoSubscriptionProvider(id));
    return async.when(
      data: (d) {
        final map = Map<String, dynamic>.from(d);
        if (map['available'] != true) {
          return Text(
            'No subscription data yet',
            style: TextStyle(color: Theme.of(context).colorScheme.outline),
          );
        }
        return Column(
          children: [
            IpoDetailScreen._kv(context, 'Overall', IpoDetailScreen._times(map['overall'])),
            IpoDetailScreen._kv(context, 'QIB', IpoDetailScreen._times(map['qib'])),
            IpoDetailScreen._kv(context, 'NII', IpoDetailScreen._times(map['nii'])),
            IpoDetailScreen._kv(context, 'Retail', IpoDetailScreen._times(map['retail'])),
            IpoDetailScreen._kv(context, 'Employee', IpoDetailScreen._times(map['employee'])),
            IpoDetailScreen._kv(context, 'Shareholder', IpoDetailScreen._times(map['shareholder'])),
            IpoDetailScreen._kv(
              context,
              'Status',
              map['is_final'] == true ? 'Final' : 'Live',
            ),
            IpoDetailScreen._kv(context, 'Source', IpoDetailScreen._str(map['source_type'] ?? map['source'])),
            IpoDetailScreen._kv(context, 'Updated at', IpoDetailScreen._str(map['updated_at'])),
          ],
        );
      },
      loading: () => const _InlineLoading(),
      error: (e, _) => _InlineError(
        message: '$e',
        onRetry: () => ref.invalidate(ipoSubscriptionProvider(id)),
      ),
    );
  }
}

class _FinancialsSection extends ConsumerWidget {
  const _FinancialsSection({required this.id});
  final String id;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final async = ref.watch(ipoFinancialsProvider(id));
    return async.when(
      data: (d) {
        final map = Map<String, dynamic>.from(d);
        final periods = (map['periods'] as List? ?? [])
            .map((e) => Map<String, dynamic>.from(e as Map))
            .toList();
        if (map['available'] != true || periods.isEmpty) {
          return Text(
            'No structured financial data yet. Use the official documents '
            'section for the prospectus.',
            style: TextStyle(color: Theme.of(context).colorScheme.outline),
          );
        }
        return Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            for (final p in periods) ...[
              if (p != periods.first) const SizedBox(height: 16),
              _PeriodCard(period: p),
            ],
          ],
        );
      },
      loading: () => const _InlineLoading(),
      error: (e, _) => _InlineError(
        message: '$e',
        onRetry: () => ref.invalidate(ipoFinancialsProvider(id)),
      ),
    );
  }
}

class _PeriodCard extends StatelessWidget {
  const _PeriodCard({required this.period});
  final Map<String, dynamic> period;

  @override
  Widget build(BuildContext context) {
    final title = [
      period['period']?.toString(),
      if (period['audited'] == true) 'Audited',
    ].where((e) => e != null && e.isNotEmpty).join(' · ');
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          title,
          style: Theme.of(context).textTheme.titleSmall?.copyWith(
                fontWeight: FontWeight.w700,
              ),
        ),
        const SizedBox(height: 4),
        _kv(context, 'Revenue', _money(period['revenue'])),
        _kv(context, 'Revenue growth', _pct(period['revenue_growth_pct'])),
        _kv(context, 'EBITDA', _money(period['ebitda'])),
        _kv(context, 'EBITDA margin', _pct(period['ebitda_margin_pct'])),
        _kv(context, 'PAT', _money(period['pat'])),
        _kv(context, 'PAT growth', _pct(period['pat_growth_pct'])),
        _kv(context, 'EPS', _money(period['eps'])),
        _kv(context, 'P/E ratio', _times(period['pe_ratio'])),
        _kv(context, 'ROE', _pct(period['roe_pct'])),
        _kv(context, 'ROCE', _pct(period['roce_pct'])),
        _kv(context, 'Debt', _money(period['debt'])),
        _kv(context, 'Debt / Equity', _str(period['debt_to_equity'])),
      ],
    );
  }

  static String _str(dynamic v) =>
      v == null ? IpoDetailScreen._na : IpoDetailScreen._num(v);
  static String _money(dynamic v) =>
      v == null ? IpoDetailScreen._na : 'Rs ${IpoDetailScreen._num(v)}';
  static String _pct(dynamic v) =>
      v == null ? IpoDetailScreen._na : '${IpoDetailScreen._num(v)}%';
  static String _times(dynamic v) =>
      v == null ? IpoDetailScreen._na : '${IpoDetailScreen._num(v)}x';
  static Widget _kv(BuildContext context, String k, String v) {
    return IpoDetailScreen._kv(context, k, v);
  }
}

class _GrowthSection extends ConsumerWidget {
  const _GrowthSection({required this.id});
  final String id;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final async = ref.watch(ipoFinancialsProvider(id));
    return async.when(
      data: (d) {
        final map = Map<String, dynamic>.from(d);
        final growth = map['growth'] is Map
            ? Map<String, dynamic>.from(map['growth'] as Map)
            : null;
        if (growth == null) {
          return Text(
            'Growth analysis requires financial data.',
            style: TextStyle(color: Theme.of(context).colorScheme.outline),
          );
        }
        return Column(
          children: [
            for (final key in const ['revenue', 'pat', 'eps'])
              if (growth[key] is Map) ...[
                _GrowthRow(
                  metricKey: key,
                  metric: Map<String, dynamic>.from(growth[key] as Map),
                ),
                const SizedBox(height: 8),
              ],
          ],
        );
      },
      loading: () => const _InlineLoading(),
      error: (e, _) => _InlineError(
        message: '$e',
        onRetry: () => ref.invalidate(ipoFinancialsProvider(id)),
      ),
    );
  }
}

class _GrowthRow extends StatelessWidget {
  const _GrowthRow({required this.metricKey, required this.metric});
  final String metricKey;
  final Map<String, dynamic> metric;

  @override
  Widget build(BuildContext context) {
    final label = metric['label']?.toString() ?? metricKey.toUpperCase();
    final latest = metric['latest_value'];
    final period = metric['latest_period']?.toString();
    final yoy = metric['yoy_growth_pct'];
    final cagr = metric['cagr_pct'];
    final years = metric['cagr_years'];

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Expanded(
              child: Text(
                label,
                style: const TextStyle(fontWeight: FontWeight.w700),
              ),
            ),
            if (latest != null)
              Text(
                IpoDetailScreen._money(latest),
                style: const TextStyle(fontWeight: FontWeight.w700),
              ),
          ],
        ),
        if (period != null)
          Text(
            'Latest: $period',
            style: Theme.of(context).textTheme.bodySmall,
          ),
        const SizedBox(height: 4),
        Wrap(
          spacing: 8,
          runSpacing: 4,
          children: [
            _GrowthChip(
              label: 'YoY',
              value: yoy == null ? null : IpoDetailScreen._pct(yoy),
            ),
            _GrowthChip(
              label: 'CAGR',
              value: cagr == null
                  ? null
                  : '${IpoDetailScreen._pct(cagr)}${years != null ? ' (${IpoDetailScreen._num(years)} yrs)' : ''}',
            ),
          ],
        ),
      ],
    );
  }
}

class _GrowthChip extends StatelessWidget {
  const _GrowthChip({required this.label, required this.value});
  final String label;
  final String? value;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: scheme.primaryContainer.withValues(alpha: 0.5),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Text(
        value == null ? '$label: Not Available' : '$label $value',
        style: Theme.of(context).textTheme.labelSmall?.copyWith(
              fontWeight: FontWeight.w600,
            ),
      ),
    );
  }
}

class _ValuationSection extends ConsumerWidget {
  const _ValuationSection({required this.id});
  final String id;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final async = ref.watch(ipoFinancialsProvider(id));
    return async.when(
      data: (d) {
        final map = Map<String, dynamic>.from(d);
        final v = map['valuation'] is Map
            ? Map<String, dynamic>.from(map['valuation'] as Map)
            : null;
        if (v == null || v['available'] != true) {
          return Text(
            'Valuation is not available yet.',
            style: TextStyle(color: Theme.of(context).colorScheme.outline),
          );
        }
        return Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            _kv(context, 'P/E ratio', _str(v['pe_ratio'])),
            _kv(context, 'EPS', _money(v['eps'])),
            _kv(context, 'Issue price', _money(v['issue_price'])),
            _kv(context, 'Implied P/E', _times(v['implied_pe'])),
            _kv(context, 'Sector P/E', _str(v['sector_pe'])),
            _kv(context, 'Premium / Discount', _pct(v['premium_discount_pct'])),
            if (v['note'] != null) ...[
              const SizedBox(height: 8),
              Text(
                '${v['note']}',
                style: Theme.of(context).textTheme.bodySmall,
              ),
            ],
          ],
        );
      },
      loading: () => const _InlineLoading(),
      error: (e, _) => _InlineError(
        message: '$e',
        onRetry: () => ref.invalidate(ipoFinancialsProvider(id)),
      ),
    );
  }

  static String _str(dynamic v) =>
      v == null ? IpoDetailScreen._na : IpoDetailScreen._num(v);
  static String _money(dynamic v) =>
      v == null ? IpoDetailScreen._na : 'Rs ${IpoDetailScreen._num(v)}';
  static String _times(dynamic v) =>
      v == null ? IpoDetailScreen._na : '${IpoDetailScreen._num(v)}x';
  static String _pct(dynamic v) =>
      v == null ? IpoDetailScreen._na : '${IpoDetailScreen._num(v)}%';
  static Widget _kv(BuildContext context, String k, String v) =>
      IpoDetailScreen._kv(context, k, v);
}

class _ScoreSection extends ConsumerWidget {
  const _ScoreSection({required this.id});
  final String id;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final async = ref.watch(ipoScoreProvider(id));
    return async.when(
      data: (d) => _ScoreBody(data: Map<String, dynamic>.from(d)),
      loading: () => const _InlineLoading(),
      error: (e, _) => _InlineError(
        message: '$e',
        onRetry: () => ref.invalidate(ipoScoreProvider(id)),
      ),
    );
  }
}

class _ScoreBody extends StatelessWidget {
  const _ScoreBody({required this.data});
  final Map<String, dynamic> data;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    final total = data['total'];
    final maxPoints = (data['max_points'] as num?)?.toInt() ?? 100;
    final version = data['methodology_version']?.toString();
    final dq = data['data_quality'] is Map
        ? Map<String, dynamic>.from(data['data_quality'] as Map)
        : <String, dynamic>{};
    final overall = dq['overall']?.toString() ?? 'insufficient';
    final missing = (dq['missing'] as List?) ?? const [];
    final components = (data['components'] as List?)
            ?.map((e) => Map<String, dynamic>.from(e as Map))
            .toList() ??
        [];
    final positives = (data['positive_factors'] as List?) ?? const [];
    final concerns = (data['concerns'] as List?) ?? const [];
    final disclaimer = data['disclaimer']?.toString();

    final score = total == null ? null : IpoDetailScreen._num(total);
    final hasScore = score != null;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          crossAxisAlignment: CrossAxisAlignment.center,
          children: [
            Text(
              hasScore ? score : '—',
              style: Theme.of(context).textTheme.headlineMedium?.copyWith(
                    fontWeight: FontWeight.w800,
                    color: hasScore ? scheme.primary : scheme.outline,
                  ),
            ),
            const SizedBox(width: 4),
            Text(
              '/ $maxPoints',
              style: Theme.of(context).textTheme.titleMedium?.copyWith(
                    color: scheme.outline,
                  ),
            ),
            const Spacer(),
            _QualityChip(overall: overall),
          ],
        ),
        const SizedBox(height: 12),
        ClipRRect(
          borderRadius: BorderRadius.circular(6),
          child: LinearProgressIndicator(
            value: hasScore ? (double.tryParse('$total') ?? 0) / 100 : 0,
            minHeight: 8,
            backgroundColor: scheme.surfaceContainerHighest,
          ),
        ),
        const SizedBox(height: 8),
        if (version != null)
          Text(
            'Methodology v$version · fundamentals-based, deterministic',
            style: Theme.of(context).textTheme.labelSmall?.copyWith(
                  color: scheme.onSurfaceVariant,
                ),
          ),
        if (missing.isNotEmpty) ...[
          const SizedBox(height: 8),
          Text(
            'Not scored (no data): ${missing.join(', ')}',
            style: Theme.of(context).textTheme.labelSmall?.copyWith(
                  color: scheme.outline,
                ),
          ),
        ],
        if (components.isNotEmpty) ...[
          const SizedBox(height: 12),
          for (final c in components) _ComponentRow(component: c),
        ],
        if (positives.isNotEmpty) ...[
          const SizedBox(height: 12),
          Text(
            'Positive factors',
            style: Theme.of(context).textTheme.titleSmall?.copyWith(
                  fontWeight: FontWeight.w700,
                ),
          ),
          const SizedBox(height: 4),
          ...positives.map((e) => Text('• $e')),
        ],
        if (concerns.isNotEmpty) ...[
          const SizedBox(height: 12),
          Text(
            'Concerns',
            style: Theme.of(context).textTheme.titleSmall?.copyWith(
                  fontWeight: FontWeight.w700,
                ),
          ),
          const SizedBox(height: 4),
          ...concerns.map((e) => Text('• $e')),
        ],
        if (disclaimer != null && disclaimer.isNotEmpty) ...[
          const SizedBox(height: 12),
          Text(
            disclaimer,
            style: Theme.of(context).textTheme.labelSmall?.copyWith(
                  color: scheme.onSurfaceVariant,
                ),
          ),
        ],
      ],
    );
  }
}

class _QualityChip extends StatelessWidget {
  const _QualityChip({required this.overall});
  final String overall;

  @override
  Widget build(BuildContext context) {
    final (color, label) = switch (overall) {
      'complete' => (Colors.green, 'Complete data'),
      'partial' => (Colors.amber.shade700, 'Partial data'),
      _ => (Colors.redAccent, 'Insufficient data'),
    };
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.15),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: color.withValues(alpha: 0.4)),
      ),
      child: Text(
        label,
        style: TextStyle(
          fontSize: 10,
          fontWeight: FontWeight.w800,
          color: color,
        ),
      ),
    );
  }
}

class _ComponentRow extends StatelessWidget {
  const _ComponentRow({required this.component});
  final Map<String, dynamic> component;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    final label = component['label']?.toString() ?? '—';
    final maxPts = (component['max_points'] as num?)?.toInt() ?? 0;
    final score = component['score'];
    final status = component['status']?.toString() ?? 'insufficient_data';
    final explanation = component['explanation']?.toString();
    final scored = score != null;
    final icon = scored
        ? Icons.check_circle_outline
        : Icons.remove_circle_outline;
    final iconColor = scored ? scheme.primary : scheme.outline;

    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 6),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Icon(icon, size: 18, color: iconColor),
          const SizedBox(width: 8),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  label,
                  style: const TextStyle(fontWeight: FontWeight.w600),
                ),
                if (explanation != null && explanation.isNotEmpty)
                  Text(
                    explanation,
                    style: Theme.of(context).textTheme.bodySmall?.copyWith(
                          color: scheme.onSurfaceVariant,
                        ),
                  ),
              ],
            ),
          ),
          const SizedBox(width: 8),
          Column(
            crossAxisAlignment: CrossAxisAlignment.end,
            children: [
              Text(
                scored ? '${IpoDetailScreen._num(score)}/$maxPts' : '—/$maxPts',
                style: const TextStyle(fontWeight: FontWeight.w700),
              ),
              Text(
                status == 'scored' ? 'Scored' : 'No data',
                style: Theme.of(context).textTheme.labelSmall?.copyWith(
                      color: scored ? scheme.primary : scheme.outline,
                    ),
              ),
            ],
          ),
        ],
      ),
    );
  }
}

class _LinkTile extends StatelessWidget {
  const _LinkTile({required this.label, required this.url});
  final String label;
  final String? url;

  @override
  Widget build(BuildContext context) {
    final has = url != null && url!.startsWith('http');
    return ListTile(
      contentPadding: EdgeInsets.zero,
      dense: true,
      title: Text(label),
      subtitle: Text(
        has ? url! : 'Not Available',
        maxLines: 2,
        overflow: TextOverflow.ellipsis,
        style: TextStyle(
          color: has
              ? Theme.of(context).colorScheme.primary
              : Theme.of(context).colorScheme.outline,
        ),
      ),
      trailing: has ? const Icon(Icons.open_in_new, size: 18) : null,
      onTap: has
          ? () async {
              final uri = Uri.parse(url!);
              if (await canLaunchUrl(uri)) {
                await launchUrl(uri, mode: LaunchMode.externalApplication);
              }
            }
          : null,
    );
  }
}

class _DetailSkeleton extends StatelessWidget {
  const _DetailSkeleton();

  @override
  Widget build(BuildContext context) {
    final base = Theme.of(context).colorScheme.surfaceContainerHighest;
    final highlight = Theme.of(context).colorScheme.surface;
    return Shimmer.fromColors(
      baseColor: base,
      highlightColor: highlight,
      child: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          Row(
            children: [
              const CircleAvatar(radius: 28),
              const SizedBox(width: 12),
              Expanded(
                child: Container(height: 48, color: Colors.white),
              ),
            ],
          ),
          const SizedBox(height: 16),
          ...List.generate(
            4,
            (_) => Padding(
              padding: const EdgeInsets.only(bottom: 12),
              child: Container(
                height: 120,
                decoration: BoxDecoration(
                  color: Colors.white,
                  borderRadius: BorderRadius.circular(20),
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}

class _DetailError extends StatelessWidget {
  const _DetailError({required this.message, required this.onRetry});
  final String message;
  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(32),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(Icons.error_outline, size: 48),
            const SizedBox(height: 12),
            Text(message, textAlign: TextAlign.center),
            const SizedBox(height: 16),
            FilledButton.icon(
              onPressed: onRetry,
              icon: const Icon(Icons.refresh),
              label: const Text('Retry'),
            ),
          ],
        ),
      ),
    );
  }
}
