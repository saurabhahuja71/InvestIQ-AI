import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:intl/intl.dart';

import 'ipo_providers.dart';

/// InvestIQ IPO View block for the IPO Detail screen.
///
/// Renders three cards from `GET /ipos/{id}/analysis`: the InvestIQ IPO View
/// (overall score, long-term view, listing view, confidence, completeness),
/// "Why InvestIQ Thinks This" (positive/negative factors) and Data Quality.
///
/// Degrades gracefully: renders nothing while loading, on error, or when the
/// IPO has no analyzable data, so it never blocks the rest of the screen.
class IpoAnalysisSection extends ConsumerWidget {
  final String ipoId;
  const IpoAnalysisSection({super.key, required this.ipoId});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final data = ref.watch(ipoAnalysisProvider(ipoId)).valueOrNull;
    if (data == null || data.isEmpty) return const SizedBox.shrink();
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        _AnalysisHeaderCard(data: data),
        _WhyCard(data: data),
        _DataQualityCard(data: data),
      ],
    );
  }
}

// ── Card 1 — InvestIQ IPO View ───────────────────────────────

class _AnalysisHeaderCard extends StatelessWidget {
  final Map<String, dynamic> data;
  const _AnalysisHeaderCard({required this.data});

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    final text = Theme.of(context).textTheme;
    final score = (data['overall_score'] as num?)?.toDouble();
    final completeness = (data['data_completeness'] as num?)?.toDouble();
    final periods = data['financial_periods'] as String?;

    return Card(
      margin: const EdgeInsets.only(bottom: 16),
      child: Padding(
        padding: const EdgeInsets.all(20),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(Icons.psychology_alt_outlined,
                    color: scheme.primary, size: 20),
                const SizedBox(width: 8),
                Text(
                  'InvestIQ IPO View',
                  style:
                      text.titleMedium?.copyWith(fontWeight: FontWeight.w700),
                ),
              ],
            ),
            const SizedBox(height: 14),
            Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                _ScoreBadge(score: score),
                const SizedBox(width: 16),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      _ChipRow(
                        label: 'Long-Term',
                        value: data['long_term_view'] as String?,
                      ),
                      const SizedBox(height: 8),
                      _ChipRow(
                        label: 'Listing',
                        value: data['listing_view'] as String?,
                      ),
                      const SizedBox(height: 8),
                      _ChipRow(
                        label: 'Confidence',
                        value: data['confidence'] as String?,
                        isConfidence: true,
                      ),
                      const SizedBox(height: 10),
                      if (completeness != null)
                        Text(
                          'Data completeness ${completeness.toStringAsFixed(0)}%',
                          style: text.bodyMedium
                              ?.copyWith(color: scheme.onSurfaceVariant),
                        ),
                      if (periods != null && periods.isNotEmpty)
                        Padding(
                          padding: const EdgeInsets.only(top: 2),
                          child: Text(
                            'Financial periods: $periods',
                            style: text.bodySmall
                                ?.copyWith(color: scheme.outline),
                          ),
                        ),
                    ],
                  ),
                ),
              ],
            ),
            const SizedBox(height: 14),
            Container(
              padding: const EdgeInsets.all(10),
              decoration: BoxDecoration(
                color: scheme.surfaceContainerHighest.withValues(alpha: 0.6),
                borderRadius: BorderRadius.circular(12),
              ),
              child: Text(
                'InvestIQ Analysis is based on available public data and can be '
                'wrong. It is not investment advice and does not guarantee '
                'returns.',
                style: text.labelSmall?.copyWith(
                  color: scheme.onSurfaceVariant,
                  height: 1.4,
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _ScoreBadge extends StatelessWidget {
  final double? score;
  const _ScoreBadge({this.score});

  @override
  Widget build(BuildContext context) {
    final s = score;
    final color = s == null
        ? Theme.of(context).colorScheme.outline
        : s >= 70
            ? const Color(0xFF2E7D32)
            : s >= 55
                ? const Color(0xFF7CB342)
                : s >= 40
                    ? const Color(0xFFF9A825)
                    : s >= 25
                        ? const Color(0xFFEF6C00)
                        : const Color(0xFFC62828);
    return Container(
      width: 92,
      height: 92,
      decoration: BoxDecoration(
        shape: BoxShape.circle,
        border: Border.all(color: color.withValues(alpha: 0.6), width: 5),
        color: color.withValues(alpha: 0.12),
      ),
      alignment: Alignment.center,
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Text(
            s == null ? 'NA' : s.toStringAsFixed(0),
            style: Theme.of(context).textTheme.headlineSmall?.copyWith(
                  color: color,
                  fontWeight: FontWeight.w800,
                ),
          ),
          Text(
            '/ 100',
            style: Theme.of(context)
                .textTheme
                .labelSmall
                ?.copyWith(color: Theme.of(context).colorScheme.outline),
          ),
        ],
      ),
    );
  }
}

class _ChipRow extends StatelessWidget {
  final String label;
  final String? value;
  final bool isConfidence;
  const _ChipRow({
    required this.label,
    this.value,
    this.isConfidence = false,
  });

  @override
  Widget build(BuildContext context) {
    final v = (value ?? '').trim();
    if (v.isEmpty) return const SizedBox.shrink();
    final color = _viewColor(v, isConfidence: isConfidence);
    return Row(
      children: [
        SizedBox(
          width: 86,
          child: Text(
            label,
            style: Theme.of(context).textTheme.bodySmall?.copyWith(
                  color: Theme.of(context).colorScheme.onSurfaceVariant,
                ),
          ),
        ),
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
          decoration: BoxDecoration(
            color: color.withValues(alpha: 0.15),
            borderRadius: BorderRadius.circular(20),
            border: Border.all(color: color.withValues(alpha: 0.6)),
          ),
          child: Text(
            v,
            style: Theme.of(context).textTheme.labelSmall?.copyWith(
                  color: color,
                  fontWeight: FontWeight.w700,
                ),
          ),
        ),
      ],
    );
  }
}

// ── Card 2 — Why InvestIQ Thinks This ────────────────────────

class _WhyCard extends StatelessWidget {
  final Map<String, dynamic> data;
  const _WhyCard({required this.data});

  @override
  Widget build(BuildContext context) {
    final positives = _factors(data['positive_factors']);
    final negatives = _factors(data['negative_factors']);
    if (positives.isEmpty && negatives.isEmpty) return const SizedBox.shrink();

    return Card(
      margin: const EdgeInsets.only(bottom: 16),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(Icons.tips_and_updates_outlined,
                    color: Theme.of(context).colorScheme.primary, size: 18),
                const SizedBox(width: 8),
                Text(
                  'Why InvestIQ Thinks This',
                  style: Theme.of(context)
                      .textTheme
                      .titleSmall
                      ?.copyWith(fontWeight: FontWeight.w700),
                ),
              ],
            ),
            const SizedBox(height: 12),
            for (final f in positives) _FactorRow(factor: f, positive: true),
            for (final f in negatives) _FactorRow(factor: f, positive: false),
          ],
        ),
      ),
    );
  }

  List<Map<String, dynamic>> _factors(dynamic v) =>
      (v as List? ?? const []).cast<Map<String, dynamic>>();
}

class _FactorRow extends StatelessWidget {
  final Map<String, dynamic> factor;
  final bool positive;
  const _FactorRow({required this.factor, required this.positive});

  @override
  Widget build(BuildContext context) {
    final title = (factor['factor'] ?? '').toString();
    final detail = factor['detail'] as String?;
    final color =
        positive ? const Color(0xFF2E7D32) : const Color(0xFFF9A825);
    return Padding(
      padding: const EdgeInsets.only(bottom: 10),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Icon(
            positive ? Icons.check_circle_outline : Icons.warning_amber_rounded,
            color: color,
            size: 18,
          ),
          const SizedBox(width: 8),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  title,
                  style: Theme.of(context)
                      .textTheme
                      .bodyMedium
                      ?.copyWith(fontWeight: FontWeight.w600),
                ),
                if (detail != null && detail.isNotEmpty)
                  Padding(
                    padding: const EdgeInsets.only(top: 2),
                    child: Text(
                      detail,
                      style: Theme.of(context).textTheme.bodySmall?.copyWith(
                            color: Theme.of(context).colorScheme.onSurfaceVariant,
                            height: 1.35,
                          ),
                    ),
                  ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

// ── Card 3 — Data Quality ────────────────────────────────────

class _DataQualityCard extends StatelessWidget {
  final Map<String, dynamic> data;
  const _DataQualityCard({required this.data});

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    final text = Theme.of(context).textTheme;
    final completeness = (data['data_completeness'] as num?)?.toDouble();
    final missing = (data['missing_data'] as List? ?? const []).cast<String>();
    final methodVersion = data['methodology_version'] as String?;
    final generatedAt = data['generated_at'] as String?;
    final ipoSyncedAt = data['ipo_synced_at'] as String?;
    final financialsRetrievedAt = data['financials_retrieved_at'] as String?;
    final subUpdatedAt = data['subscription_updated_at'] as String?;

    return Card(
      margin: const EdgeInsets.only(bottom: 16),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(Icons.data_usage_outlined,
                    color: scheme.primary, size: 18),
                const SizedBox(width: 8),
                Text(
                  'Data Quality',
                  style: text.titleSmall?.copyWith(fontWeight: FontWeight.w700),
                ),
              ],
            ),
            const SizedBox(height: 12),
            if (completeness != null) ...[
              ClipRRect(
                borderRadius: BorderRadius.circular(6),
                child: LinearProgressIndicator(
                  value: (completeness / 100).clamp(0.0, 1.0),
                  minHeight: 8,
                  backgroundColor: scheme.surfaceContainerHighest,
                ),
              ),
              const SizedBox(height: 6),
              Text(
                'Data completeness ${completeness.toStringAsFixed(0)}%',
                style:
                    text.bodySmall?.copyWith(color: scheme.onSurfaceVariant),
              ),
              const SizedBox(height: 10),
            ],
            if (missing.isNotEmpty) ...[
              Text(
                'Missing / not evaluated',
                style: text.bodySmall?.copyWith(color: scheme.onSurfaceVariant),
              ),
              const SizedBox(height: 6),
              Wrap(
                spacing: 6,
                runSpacing: 6,
                children: [
                  for (final m in missing)
                    Container(
                      padding: const EdgeInsets.symmetric(
                          horizontal: 8, vertical: 3),
                      decoration: BoxDecoration(
                        color:
                            scheme.surfaceContainerHighest.withValues(alpha: 0.6),
                        borderRadius: BorderRadius.circular(20),
                      ),
                      child: Text(
                        m,
                        style: text.labelSmall?.copyWith(color: scheme.outline),
                      ),
                    ),
                ],
              ),
              const SizedBox(height: 10),
            ],
            _MetaLine(
              icon: Icons.science_outlined,
              text: 'Methodology v${methodVersion ?? '?'} · deterministic, no LLM',
            ),
            _MetaLine(
              icon: Icons.schedule,
              text: 'Generated ${_fmtTs(generatedAt)}',
            ),
            _MetaLine(
              icon: Icons.cloud_sync_outlined,
              text: 'IPO synced ${_fmtTs(ipoSyncedAt)} · financials '
                  '${_fmtTs(financialsRetrievedAt)}',
            ),
            _MetaLine(
              icon: Icons.subscriptions_outlined,
              text: 'Subscription updated ${_fmtTs(subUpdatedAt)}',
            ),
          ],
        ),
      ),
    );
  }
}

class _MetaLine extends StatelessWidget {
  final IconData icon;
  final String text;
  const _MetaLine({required this.icon, required this.text});

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    return Padding(
      padding: const EdgeInsets.only(top: 5),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Icon(icon, color: scheme.outline, size: 14),
          const SizedBox(width: 6),
          Expanded(
            child: Text(
              text,
              style: Theme.of(context).textTheme.labelSmall?.copyWith(
                    color: scheme.onSurfaceVariant,
                    height: 1.3,
                  ),
            ),
          ),
        ],
      ),
    );
  }
}

// ── Helpers ──────────────────────────────────────────────────

Color _viewColor(String value, {bool isConfidence = false}) {
  final v = value.toUpperCase();
  switch (v) {
    case 'STRONG POSITIVE':
    case 'POSITIVE':
    case 'HIGH':
      return const Color(0xFF2E7D32);
    case 'NEUTRAL':
    case 'MEDIUM':
      return const Color(0xFF1565C0);
    case 'CAUTION':
    case 'LOW':
      return const Color(0xFFF9A825);
    case 'NEGATIVE':
      return const Color(0xFFC62828);
    default:
      return const Color(0xFF616161);
  }
}

String _fmtTs(String? iso) {
  if (iso == null || iso.isEmpty) return 'n/a';
  try {
    final dt = DateTime.tryParse(iso);
    if (dt == null) return iso;
    final local = dt.toLocal();
    final now = DateTime.now();
    if (local.year == now.year &&
        local.month == now.month &&
        local.day == now.day) {
      return DateFormat('h:mm a').format(local);
    }
    return DateFormat('d MMM, h:mm a').format(local);
  } catch (_) {
    return iso;
  }
}
