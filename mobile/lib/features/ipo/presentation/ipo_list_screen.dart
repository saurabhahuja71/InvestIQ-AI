import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../core/widgets/glass_card.dart';
import 'ipo_providers.dart';

class IpoListScreen extends ConsumerStatefulWidget {
  const IpoListScreen({super.key});

  @override
  ConsumerState<IpoListScreen> createState() => _IpoListScreenState();
}

class _IpoListScreenState extends ConsumerState<IpoListScreen>
    with SingleTickerProviderStateMixin {
  late final TabController _tabs;
  final _search = TextEditingController();
  String? _boardFilter;

  @override
  void initState() {
    super.initState();
    _tabs = TabController(length: 4, vsync: this);
  }

  @override
  void dispose() {
    _tabs.dispose();
    _search.dispose();
    super.dispose();
  }

  String? _statusForIndex(int i) =>
      switch (i) { 0 => 'open', 1 => 'upcoming', 2 => 'closed', 3 => null, _ => null };

  String _band(Map<String, dynamic> ipo) {
    final low = ipo['price_band_low'];
    final high = ipo['price_band_high'];
    final issue = ipo['issue_price'];
    if (issue != null) return '₹$issue';
    if (low == null && high == null) return 'Not available';
    return '₹${low ?? '—'}–${high ?? '—'}';
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('IPO Tracker'),
        bottom: TabBar(
          controller: _tabs,
          isScrollable: true,
          tabs: const [
            Tab(text: 'Open'),
            Tab(text: 'Upcoming'),
            Tab(text: 'Closed'),
            Tab(text: 'All / SME'),
          ],
          onTap: (_) => setState(() {}),
        ),
      ),
      body: Column(
        children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(16, 12, 16, 8),
            child: TextField(
              controller: _search,
              decoration: InputDecoration(
                hintText: 'Search IPO',
                prefixIcon: const Icon(Icons.search),
                suffixIcon: IconButton(
                  icon: const Icon(Icons.filter_list),
                  onPressed: () async {
                    final board = await showModalBottomSheet<String>(
                      context: context,
                      builder: (ctx) => SafeArea(
                        child: Column(
                          mainAxisSize: MainAxisSize.min,
                          children: [
                            ListTile(
                              title: const Text('All boards'),
                              onTap: () => Navigator.pop(ctx, ''),
                            ),
                            ListTile(
                              title: const Text('Mainboard'),
                              onTap: () => Navigator.pop(ctx, 'mainboard'),
                            ),
                            ListTile(
                              title: const Text('SME'),
                              onTap: () => Navigator.pop(ctx, 'sme'),
                            ),
                          ],
                        ),
                      ),
                    );
                    if (board != null) {
                      setState(() => _boardFilter = board.isEmpty ? null : board);
                    }
                  },
                ),
              ),
              onChanged: (_) => setState(() {}),
            ),
          ),
          Expanded(
            child: AnimatedBuilder(
              animation: _tabs,
              builder: (context, _) {
                final status = _statusForIndex(_tabs.index);
                final async = ref.watch(ipoListProvider(status));
                return async.when(
                  data: (list) {
                    var filtered = list;
                    if (_boardFilter != null) {
                      filtered = filtered
                          .where((e) => e['board'] == _boardFilter)
                          .toList();
                    }
                    if (_tabs.index == 3) {
                      filtered =
                          filtered.where((e) => e['board'] == 'sme').toList();
                    }
                    final q = _search.text.trim().toLowerCase();
                    if (q.isNotEmpty) {
                      filtered = filtered
                          .where((e) =>
                              (e['company_name']?.toString().toLowerCase() ?? '')
                                  .contains(q) ||
                              (e['symbol']?.toString().toLowerCase() ?? '')
                                  .contains(q))
                          .toList();
                    }
                    if (filtered.isEmpty) {
                      return RefreshIndicator(
                        onRefresh: () => refreshIpoFeed(ref, status: status),
                        child: ListView(
                          physics: const AlwaysScrollableScrollPhysics(),
                          children: const [
                            SizedBox(height: 120),
                            Center(
                              child: Text(
                                'No IPOs from the exchange for this filter.\nPull to refresh.',
                                textAlign: TextAlign.center,
                              ),
                            ),
                          ],
                        ),
                      );
                    }
                    return RefreshIndicator(
                      onRefresh: () => refreshIpoFeed(ref, status: status),
                      child: ListView.separated(
                        padding: const EdgeInsets.all(16),
                        itemCount: filtered.length,
                        separatorBuilder: (_, __) => const SizedBox(height: 10),
                        itemBuilder: (context, i) {
                          final ipo = filtered[i];
                          final sub = ipo['subscription_total'];
                          return GlassCard(
                            onTap: () => context.push('/ipos/${ipo['id']}'),
                            child: Column(
                              crossAxisAlignment: CrossAxisAlignment.start,
                              children: [
                                Row(
                                  children: [
                                    Expanded(
                                      child: Text(
                                        ipo['company_name']?.toString() ?? '',
                                        style: const TextStyle(
                                          fontWeight: FontWeight.w700,
                                          fontSize: 16,
                                        ),
                                      ),
                                    ),
                                    Chip(
                                      label:
                                          Text(ipo['status']?.toString() ?? ''),
                                      visualDensity: VisualDensity.compact,
                                    ),
                                  ],
                                ),
                                const SizedBox(height: 6),
                                Text(
                                  '${ipo['symbol'] ?? ''} · ${ipo['board']} · ${_band(ipo)}',
                                  style: Theme.of(context).textTheme.bodySmall,
                                ),
                                if (ipo['open_date'] != null ||
                                    ipo['close_date'] != null) ...[
                                  const SizedBox(height: 4),
                                  Text(
                                    'Open ${ipo['open_date'] ?? '—'} · Close ${ipo['close_date'] ?? '—'}',
                                    style:
                                        Theme.of(context).textTheme.labelSmall,
                                  ),
                                ],
                                if (sub != null) ...[
                                  const SizedBox(height: 6),
                                  Text(
                                    'Subscription ${sub}x',
                                    style: Theme.of(context)
                                        .textTheme
                                        .labelMedium
                                        ?.copyWith(fontWeight: FontWeight.w600),
                                  ),
                                ],
                                if (ipo['gmp_value'] != null) ...[
                                  const SizedBox(height: 8),
                                  Container(
                                    padding: const EdgeInsets.symmetric(
                                      horizontal: 10,
                                      vertical: 6,
                                    ),
                                    decoration: BoxDecoration(
                                      color:
                                          Colors.amber.withValues(alpha: 0.12),
                                      borderRadius: BorderRadius.circular(10),
                                      border: Border.all(
                                        color: Colors.amber
                                            .withValues(alpha: 0.5),
                                      ),
                                    ),
                                    child: Text(
                                      'GMP ${ipo['gmp_value']} · Unofficial',
                                      style: const TextStyle(
                                        fontWeight: FontWeight.w600,
                                        fontSize: 12,
                                      ),
                                    ),
                                  ),
                                ],
                              ],
                            ),
                          );
                        },
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
                            onPressed: () =>
                                refreshIpoFeed(ref, status: status),
                            child: const Text('Retry'),
                          ),
                        ],
                      ),
                    ),
                  ),
                );
              },
            ),
          ),
        ],
      ),
    );
  }
}
