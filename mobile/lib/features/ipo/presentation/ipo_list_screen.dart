import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_animate/flutter_animate.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:shimmer/shimmer.dart';

import '../../../core/network/api_client.dart';
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
  final _scroll = ScrollController();
  Timer? _debounce;
  String _query = '';
  String? _boardFilter;
  int _page = 1;
  bool _loadingMore = false;
  final List<Map<String, dynamic>> _items = [];
  int _total = 0;
  bool _initialized = false;
  Object? _error;
  bool _fromCache = false;

  static const _statuses = ['open', 'upcoming', 'closed', 'listed'];

  @override
  void initState() {
    super.initState();
    _tabs = TabController(length: 4, vsync: this);
    _tabs.addListener(_onTabChanged);
    _scroll.addListener(_onScroll);
    WidgetsBinding.instance.addPostFrameCallback((_) => _load(reset: true));
  }

  @override
  void dispose() {
    _debounce?.cancel();
    _tabs.removeListener(_onTabChanged);
    _tabs.dispose();
    _search.dispose();
    _scroll.dispose();
    super.dispose();
  }

  String? get _status => _statuses[_tabs.index];

  void _onTabChanged() {
    if (_tabs.indexIsChanging) return;
    _load(reset: true);
  }

  void _onScroll() {
    if (_loadingMore || _items.length >= _total) return;
    if (_scroll.position.pixels >= _scroll.position.maxScrollExtent - 240) {
      _loadMore();
    }
  }

  IpoListParams _params({required int page, bool refresh = false}) {
    return IpoListParams(
      status: _status,
      board: _boardFilter,
      query: _query,
      page: page,
      perPage: 20,
      refresh: refresh,
    );
  }

  Future<void> _load({required bool reset, bool refresh = false}) async {
    if (reset) {
      setState(() {
        _page = 1;
        _error = null;
        _initialized = false;
        if (refresh) _items.clear();
      });
    }
    try {
      final result = await ref.read(
        ipoListQueryProvider(_params(page: 1, refresh: refresh)).future,
      );
      if (!mounted) return;
      setState(() {
        _items
          ..clear()
          ..addAll(result.items);
        _total = result.total;
        _page = result.page;
        _fromCache = result.fromCache;
        _initialized = true;
        _error = null;
      });
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _error = e;
        _initialized = true;
      });
    }
  }

  Future<void> _loadMore() async {
    if (_loadingMore || _items.length >= _total) return;
    setState(() => _loadingMore = true);
    final next = _page + 1;
    try {
      final result =
          await ref.read(ipoListQueryProvider(_params(page: next)).future);
      if (!mounted) return;
      setState(() {
        _items.addAll(result.items);
        _page = result.page;
        _total = result.total;
        _loadingMore = false;
      });
    } catch (_) {
      if (!mounted) return;
      setState(() => _loadingMore = false);
    }
  }

  Future<void> _onRefresh() async {
    final dio = ref.read(dioProvider);
    await refreshIpoFeed(dio);
    ref.invalidate(ipoListQueryProvider);
    await _load(reset: true, refresh: true);
  }

  void _onSearchChanged(String value) {
    _debounce?.cancel();
    _debounce = Timer(const Duration(milliseconds: 350), () {
      setState(() => _query = value.trim());
      _load(reset: true);
    });
  }

  Future<void> _pickBoard() async {
    final board = await showModalBottomSheet<String>(
      context: context,
      showDragHandle: true,
      builder: (ctx) => SafeArea(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            ListTile(
              leading: const Icon(Icons.all_inclusive),
              title: const Text('All boards'),
              onTap: () => Navigator.pop(ctx, ''),
            ),
            ListTile(
              leading: const Icon(Icons.account_balance),
              title: const Text('Mainboard'),
              onTap: () => Navigator.pop(ctx, 'mainboard'),
            ),
            ListTile(
              leading: const Icon(Icons.storefront_outlined),
              title: const Text('SME'),
              onTap: () => Navigator.pop(ctx, 'sme'),
            ),
          ],
        ),
      ),
    );
    if (board == null) return;
    setState(() => _boardFilter = board.isEmpty ? null : board);
    await _load(reset: true);
  }

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;

    return Scaffold(
      appBar: AppBar(
        title: const Text('IPO Tracker'),
        actions: [
          IconButton(
            tooltip: 'Refresh from exchange',
            onPressed: _onRefresh,
            icon: const Icon(Icons.sync),
          ),
        ],
        bottom: TabBar(
          controller: _tabs,
          isScrollable: true,
          tabs: const [
            Tab(text: 'Open'),
            Tab(text: 'Upcoming'),
            Tab(text: 'Closed'),
            Tab(text: 'Listed'),
          ],
        ),
      ),
      body: Column(
        children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(16, 12, 16, 8),
            child: TextField(
              controller: _search,
              onChanged: _onSearchChanged,
              decoration: InputDecoration(
                hintText: 'Search by company or symbol',
                prefixIcon: const Icon(Icons.search),
                suffixIcon: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    if (_boardFilter != null)
                      Padding(
                        padding: const EdgeInsets.only(right: 4),
                        child: InputChip(
                          label: Text(_boardFilter!),
                          onDeleted: () {
                            setState(() => _boardFilter = null);
                            _load(reset: true);
                          },
                          visualDensity: VisualDensity.compact,
                        ),
                      ),
                    IconButton(
                      tooltip: 'Filter board',
                      icon: Icon(
                        Icons.filter_list,
                        color: _boardFilter != null ? scheme.primary : null,
                      ),
                      onPressed: _pickBoard,
                    ),
                  ],
                ),
              ),
            ),
          ),
          if (_fromCache)
            Container(
              width: double.infinity,
              color: scheme.secondaryContainer.withValues(alpha: 0.55),
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
              child: Row(
                children: [
                  const Icon(Icons.cloud_off_outlined, size: 18),
                  const SizedBox(width: 8),
                  const Expanded(
                    child: Text('Showing offline cache - pull to refresh'),
                  ),
                  TextButton(onPressed: _onRefresh, child: const Text('Retry')),
                ],
              ),
            ),
          Expanded(child: _buildBody(scheme)),
        ],
      ),
    );
  }

  Widget _buildBody(ColorScheme scheme) {
    if (!_initialized) {
      return const _IpoListSkeleton();
    }
    if (_error != null && _items.isEmpty) {
      return _ErrorState(
        message: '$_error',
        onRetry: () => _load(reset: true, refresh: true),
      );
    }
    if (_items.isEmpty) {
      return _EmptyState(
        status: _status ?? 'all',
        query: _query,
        onClear: _query.isEmpty && _boardFilter == null
            ? null
            : () {
                _search.clear();
                setState(() {
                  _query = '';
                  _boardFilter = null;
                });
                _load(reset: true);
              },
      );
    }

    return RefreshIndicator(
      onRefresh: _onRefresh,
      child: ListView.separated(
        controller: _scroll,
        physics: const AlwaysScrollableScrollPhysics(),
        padding: const EdgeInsets.fromLTRB(16, 4, 16, 24),
        itemCount: _items.length + (_loadingMore ? 1 : 0),
        separatorBuilder: (_, __) => const SizedBox(height: 10),
        itemBuilder: (context, i) {
          if (i >= _items.length) {
            return const Padding(
              padding: EdgeInsets.all(16),
              child: Center(child: CircularProgressIndicator()),
            );
          }
          final ipo = _items[i];
          return _IpoCard(ipo: ipo)
              .animate()
              .fadeIn(duration: 220.ms, delay: (20 * (i % 8)).ms)
              .slideY(begin: 0.04, end: 0, duration: 220.ms);
        },
      ),
    );
  }
}

class _IpoCard extends StatelessWidget {
  const _IpoCard({required this.ipo});
  final Map<String, dynamic> ipo;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    final name = ipo['company_name']?.toString() ?? 'Not Available';
    final symbol = ipo['symbol']?.toString();
    final board = ipo['board']?.toString() ?? 'Not Available';
    final status = ipo['status']?.toString() ?? 'Not Available';
    final low = ipo['price_band_low'];
    final high = ipo['price_band_high'];
    final band = (low == null && high == null)
        ? 'Not Available'
        : '${low ?? '-'} - ${high ?? '-'}';
    final lot = ipo['lot_size']?.toString() ?? 'Not Available';
    final exchange = ipo['exchange']?.toString();
    final open = ipo['open_date']?.toString() ?? 'Not Available';
    final close = ipo['close_date']?.toString() ?? 'Not Available';

    return GlassCard(
      onTap: () => context.push('/ipos/${ipo['id']}'),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              CircleAvatar(
                radius: 22,
                backgroundColor: scheme.primaryContainer,
                foregroundColor: scheme.onPrimaryContainer,
                child: Text(
                  _initials(name),
                  style: const TextStyle(fontWeight: FontWeight.w700, fontSize: 12),
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      name,
                      style: const TextStyle(
                        fontWeight: FontWeight.w700,
                        fontSize: 16,
                      ),
                    ),
                    if (symbol != null && symbol.isNotEmpty)
                      Text(
                        symbol,
                        style: Theme.of(context).textTheme.bodySmall?.copyWith(
                              color: scheme.onSurfaceVariant,
                            ),
                      ),
                  ],
                ),
              ),
              Chip(
                label: Text(status),
                visualDensity: VisualDensity.compact,
                side: BorderSide.none,
                backgroundColor: scheme.secondaryContainer,
              ),
            ],
          ),
          const SizedBox(height: 10),
          Wrap(
            spacing: 8,
            runSpacing: 6,
            children: [
              _MetaChip(label: board),
              if (exchange != null) _MetaChip(label: exchange),
              _MetaChip(label: 'Band $band'),
              _MetaChip(label: 'Lot $lot'),
            ],
          ),
          const SizedBox(height: 8),
          Text(
            'Open $open | Close $close',
            style: Theme.of(context).textTheme.bodySmall?.copyWith(
                  color: scheme.onSurfaceVariant,
                ),
          ),
        ],
      ),
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

class _MetaChip extends StatelessWidget {
  const _MetaChip({required this.label});
  final String label;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
      decoration: BoxDecoration(
        color: scheme.surfaceContainerHighest.withValues(alpha: 0.65),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Text(label, style: Theme.of(context).textTheme.labelSmall),
    );
  }
}

class _IpoListSkeleton extends StatelessWidget {
  const _IpoListSkeleton();

  @override
  Widget build(BuildContext context) {
    final base = Theme.of(context).colorScheme.surfaceContainerHighest;
    final highlight = Theme.of(context).colorScheme.surface;
    return Shimmer.fromColors(
      baseColor: base,
      highlightColor: highlight,
      child: ListView.separated(
        padding: const EdgeInsets.all(16),
        itemCount: 6,
        separatorBuilder: (_, __) => const SizedBox(height: 10),
        itemBuilder: (_, __) => Container(
          height: 118,
          decoration: BoxDecoration(
            color: Colors.white,
            borderRadius: BorderRadius.circular(20),
          ),
        ),
      ),
    );
  }
}

class _EmptyState extends StatelessWidget {
  const _EmptyState({
    required this.status,
    required this.query,
    this.onClear,
  });

  final String status;
  final String query;
  final VoidCallback? onClear;

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(32),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(
              Icons.inbox_outlined,
              size: 56,
              color: Theme.of(context).colorScheme.outline,
            ),
            const SizedBox(height: 12),
            Text(
              query.isNotEmpty
                  ? 'No IPOs match “$query”'
                  : 'No $status IPOs right now',
              textAlign: TextAlign.center,
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 8),
            Text(
              'Data is synced from NSE India. Pull to refresh when the exchange updates.',
              textAlign: TextAlign.center,
              style: Theme.of(context).textTheme.bodySmall,
            ),
            if (onClear != null) ...[
              const SizedBox(height: 16),
              FilledButton.tonal(
                onPressed: onClear,
                child: const Text('Clear filters'),
              ),
            ],
          ],
        ),
      ),
    );
  }
}

class _ErrorState extends StatelessWidget {
  const _ErrorState({required this.message, required this.onRetry});
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
            Icon(
              Icons.error_outline,
              size: 56,
              color: Theme.of(context).colorScheme.error,
            ),
            const SizedBox(height: 12),
            Text(
              'Could not load IPOs',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 8),
            Text(
              message,
              textAlign: TextAlign.center,
              style: Theme.of(context).textTheme.bodySmall,
            ),
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
