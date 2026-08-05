import 'package:flutter/material.dart';
import 'package:flutter_animate/flutter_animate.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:shimmer/shimmer.dart';
import 'package:url_launcher/url_launcher.dart';

import '../../../core/constants/app_constants.dart';
import '../../../core/network/api_client.dart';
import '../../../core/widgets/glass_card.dart';
import 'ipo_providers.dart';

class IpoDetailScreen extends ConsumerWidget {
  const IpoDetailScreen({super.key, required this.id});
  final String id;

  static const _na = 'Not Available';

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final async = ref.watch(ipoDetailProvider(id));
    final scheme = Theme.of(context).colorScheme;

    return Scaffold(
      appBar: AppBar(
        title: const Text('IPO Details'),
        actions: [
          IconButton(
            tooltip: 'Refresh',
            onPressed: () => ref.invalidate(ipoDetailProvider(id)),
            icon: const Icon(Icons.refresh),
          ),
        ],
      ),
      body: async.when(
        data: (ipo) => RefreshIndicator(
          onRefresh: () async => ref.invalidate(ipoDetailProvider(id)),
          child: ListView(
            padding: const EdgeInsets.all(16),
            children: [
              _Header(ipo: ipo),
              const SizedBox(height: 12),
              _Section(
                title: 'Issue snapshot',
                child: Column(
                  children: [
                    _kv(context, 'Exchange', _str(ipo['exchange'])),
                    _kv(context, 'Status', _str(ipo['status'])),
                    _kv(context, 'Board', _str(ipo['board'])),
                    _kv(context, 'Industry', _str(ipo['industry'] ?? ipo['sector'])),
                    _kv(context, 'Open Date', _str(ipo['open_date'])),
                    _kv(context, 'Close Date', _str(ipo['close_date'])),
                    _kv(context, 'Listing Date', _str(ipo['listing_date'])),
                    _kv(context, 'Allotment Date', _str(ipo['allotment_date'])),
                    _kv(context, 'Price Band', _priceBand(ipo)),
                    _kv(context, 'Issue Price', _money(ipo['issue_price'])),
                    _kv(context, 'Lot Size', _str(ipo['lot_size'])),
                    _kv(context, 'Minimum Investment', _money(ipo['min_investment'])),
                    _kv(context, 'Issue Size (₹ Cr)', _str(ipo['issue_size_cr'])),
                    _kv(context, 'Face Value', _money(ipo['face_value'])),
                    _kv(context, 'Issue Type', _str(ipo['issue_type'])),
                    _kv(context, 'Registrar', _str(ipo['registrar'])),
                    _kv(context, 'Lead Managers', _leadManagers(ipo['lead_managers'])),
                    _kv(context, 'Data source', _str(ipo['source'] ?? 'nse')),
                    _kv(context, 'Last synced', _str(ipo['source_synced_at'])),
                  ],
                ),
              ),
              const SizedBox(height: 12),
              _Section(
                title: 'Subscription',
                child: Column(
                  children: [
                    _kv(context, 'Total', _times(ipo['subscription_total'])),
                    _kv(context, 'QIB', _times(ipo['subscription_qib'])),
                    _kv(context, 'NII', _times(ipo['subscription_nii'])),
                    _kv(context, 'Retail', _times(ipo['subscription_retail'])),
                  ],
                ),
              ),
              const SizedBox(height: 12),
              _GmpCard(ipo: ipo, scheme: scheme),
              const SizedBox(height: 12),
              _Section(
                title: 'Financial highlights',
                child: _Financials(raw: ipo['financials']),
              ),
              const SizedBox(height: 12),
              _Section(
                title: 'Business description',
                child: Text(
                  _str(ipo['description']),
                  style: Theme.of(context).textTheme.bodyMedium,
                ),
              ),
              const SizedBox(height: 12),
              _Section(
                title: 'Documents & links',
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
              if (_hasList(ipo['pros']) || _hasList(ipo['risks'])) ...[
                const SizedBox(height: 12),
                _Section(
                  title: 'Pros & risks',
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

  static String _money(dynamic v) {
    if (v == null) return _na;
    return '₹$v';
  }

  static String _times(dynamic v) {
    if (v == null) return _na;
    return '${v}x';
  }

  static String _priceBand(Map<String, dynamic> ipo) {
    final low = ipo['price_band_low'];
    final high = ipo['price_band_high'];
    if (low == null && high == null) return _na;
    return '₹${low ?? '—'} – ₹${high ?? '—'}';
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
              'Indicative only — confirm with the registrar.',
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

class _GmpCard extends StatelessWidget {
  const _GmpCard({required this.ipo, required this.scheme});
  final Map<String, dynamic> ipo;
  final ColorScheme scheme;

  @override
  Widget build(BuildContext context) {
    final gmp = ipo['gmp'] as Map<String, dynamic>?;
    final available = gmp?['available'] == true || gmp?['value'] != null;
    return GlassCard(
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
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                decoration: BoxDecoration(
                  color: Colors.amber.withValues(alpha: 0.2),
                  borderRadius: BorderRadius.circular(8),
                ),
                child: const Text(
                  'UNOFFICIAL',
                  style: TextStyle(fontSize: 10, fontWeight: FontWeight.w800),
                ),
              ),
            ],
          ),
          const SizedBox(height: 8),
          Text(
            available ? '₹${gmp?['value'] ?? ipo['gmp_value']}' : 'Not Available',
            style: Theme.of(context).textTheme.headlineSmall?.copyWith(
                  color: available ? null : scheme.outline,
                ),
          ),
          const SizedBox(height: 8),
          Text(
            available
                ? (gmp?['disclaimer']?.toString() ?? AppConstants.gmpDisclaimer)
                : 'NSE India does not publish GMP. InvestIQ does not invent grey-market figures.',
            style: Theme.of(context).textTheme.bodySmall?.copyWith(
                  color: scheme.onSurfaceVariant,
                ),
          ),
        ],
      ),
    );
  }
}

class _Financials extends StatelessWidget {
  const _Financials({required this.raw});
  final dynamic raw;

  @override
  Widget build(BuildContext context) {
    if (raw is! Map || raw.isEmpty) {
      return Text(
        'Not Available',
        style: TextStyle(color: Theme.of(context).colorScheme.outline),
      );
    }
    final map = Map<String, dynamic>.from(raw);
    // Prefer human labels; skip raw URL keys shown as links elsewhere
    final entries = map.entries.where((e) {
      final k = e.key.toLowerCase();
      return !k.endsWith('_url') && e.value != null && '${e.value}'.trim().isNotEmpty;
    }).toList();
    if (entries.isEmpty) {
      return Text(
        'Structured financials are Not Available from NSE. Use the ratios / prospectus links when present.',
        style: Theme.of(context).textTheme.bodySmall,
      );
    }
    return Column(
      children: entries
          .map(
            (e) => Padding(
              padding: const EdgeInsets.symmetric(vertical: 4),
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Expanded(child: Text(_label(e.key))),
                  Expanded(
                    child: Text(
                      '${e.value}',
                      textAlign: TextAlign.end,
                      style: const TextStyle(fontWeight: FontWeight.w600),
                    ),
                  ),
                ],
              ),
            ),
          )
          .toList(),
    );
  }

  String _label(String key) => key.replaceAll('_', ' ');
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
