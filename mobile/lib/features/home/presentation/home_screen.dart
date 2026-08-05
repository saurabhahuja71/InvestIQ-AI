import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../core/widgets/glass_card.dart';
import '../../auth/presentation/auth_controller.dart';
import '../../ipo/presentation/ipo_providers.dart';
import '../../portfolio/presentation/portfolio_providers.dart';

class HomeScreen extends ConsumerWidget {
  const HomeScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final user = ref.watch(authControllerProvider).user;
    final ipos = ref.watch(openIposProvider);
    final portfolio = ref.watch(portfolioDashboardProvider);
    final scheme = Theme.of(context).colorScheme;

    return Scaffold(
      appBar: AppBar(
        title: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Hello${user?.fullName != null ? ', ${user!.fullName}' : ''}',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            Text(
              'IPO / Portfolio / Journal / AI',
              style: Theme.of(context).textTheme.labelSmall?.copyWith(
                    color: scheme.onSurfaceVariant,
                  ),
            ),
          ],
        ),
        actions: [
          IconButton(
            icon: const Icon(Icons.settings_outlined),
            onPressed: () => context.push('/settings'),
          ),
        ],
      ),
      body: RefreshIndicator(
        onRefresh: () async {
          ref.invalidate(openIposProvider);
          ref.invalidate(portfolioDashboardProvider);
        },
        child: ListView(
          padding: const EdgeInsets.all(16),
          children: [
            portfolio.when(
              data: (dash) {
                final value = dash?['analytics']?['total_value']?.toString() ?? '0';
                final ret = dash?['analytics']?['overall_return_pct'];
                return GlassCard(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text('Portfolio value',
                          style: Theme.of(context).textTheme.labelLarge),
                      const SizedBox(height: 6),
                      Text(
                        '₹$value',
                        style: Theme.of(context).textTheme.headlineMedium?.copyWith(
                              fontWeight: FontWeight.w700,
                              fontFeatures: const [FontFeature.tabularFigures()],
                            ),
                      ),
                      if (ret != null)
                        Text(
                          'Overall ${ret is num ? ret.toStringAsFixed(2) : ret}%',
                          style: TextStyle(
                            color: (ret is num && ret >= 0)
                                ? Colors.green
                                : Colors.redAccent,
                          ),
                        ),
                    ],
                  ),
                );
              },
              loading: () => const GlassCard(
                child: SizedBox(height: 72, child: Center(child: CircularProgressIndicator())),
              ),
              error: (_, __) => GlassCard(
                onTap: () => context.go('/portfolio'),
                child: const Text('Add holdings to see portfolio pulse'),
              ),
            ),
            const SizedBox(height: 16),
            Row(
              children: [
                Expanded(
                  child: _QuickChip(
                    icon: Icons.rocket_launch,
                    label: 'Open IPOs',
                    onTap: () => context.go('/ipos'),
                  ),
                ),
                const SizedBox(width: 8),
                Expanded(
                  child: _QuickChip(
                    icon: Icons.auto_awesome,
                    label: 'Ask AI',
                    onTap: () => context.go('/ai'),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 20),
            Text('Open IPOs', style: Theme.of(context).textTheme.titleMedium),
            const SizedBox(height: 8),
            ipos.when(
              data: (list) {
                if (list.isEmpty) {
                  return const GlassCard(child: Text('No open IPOs right now'));
                }
                return Column(
                  children: list.take(5).map((ipo) {
                    return Padding(
                      padding: const EdgeInsets.only(bottom: 10),
                      child: GlassCard(
                        onTap: () => context.push('/ipos/${ipo['id']}'),
                        child: Row(
                          children: [
                            CircleAvatar(
                              backgroundColor: scheme.primaryContainer,
                              child: Text(
                                (ipo['company_name'] as String? ?? '?')
                                    .substring(0, 1),
                              ),
                            ),
                            const SizedBox(width: 12),
                            Expanded(
                              child: Column(
                                crossAxisAlignment: CrossAxisAlignment.start,
                                children: [
                                  Text(
                                    ipo['company_name']?.toString() ?? '',
                                    style: const TextStyle(fontWeight: FontWeight.w600),
                                  ),
                                  Text(
                                    '${ipo['board']} · Lot ${ipo['lot_size'] ?? '-'}',
                                    style: Theme.of(context).textTheme.bodySmall,
                                  ),
                                ],
                              ),
                            ),
                            if (ipo['gmp_value'] != null)
                              Chip(
                                label: Text('GMP ${ipo['gmp_value']}*'),
                                visualDensity: VisualDensity.compact,
                                backgroundColor: Colors.amber.withValues(alpha: 0.15),
                              ),
                          ],
                        ),
                      ),
                    );
                  }).toList(),
                );
              },
              loading: () => const Center(child: CircularProgressIndicator()),
              error: (e, _) => Text('Failed to load IPOs: $e'),
            ),
            const SizedBox(height: 8),
            Text(
              '* GMP is unofficial',
              style: Theme.of(context).textTheme.labelSmall?.copyWith(
                    color: scheme.onSurfaceVariant,
                  ),
            ),
          ],
        ),
      ),
    );
  }
}

class _QuickChip extends StatelessWidget {
  const _QuickChip({
    required this.icon,
    required this.label,
    required this.onTap,
  });
  final IconData icon;
  final String label;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return GlassCard(
      onTap: onTap,
      padding: const EdgeInsets.symmetric(vertical: 14, horizontal: 12),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(icon, size: 18),
          const SizedBox(width: 8),
          Text(label, style: const TextStyle(fontWeight: FontWeight.w600)),
        ],
      ),
    );
  }
}
