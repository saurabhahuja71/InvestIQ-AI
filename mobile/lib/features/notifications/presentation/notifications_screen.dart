import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../core/network/api_client.dart';
import '../../../core/network/api_exception.dart';
import '../../../core/widgets/glass_card.dart';
import '../../watchlist/presentation/watchlist_providers.dart';

final notificationsProvider =
    FutureProvider<List<Map<String, dynamic>>>((ref) async {
  final dio = ref.watch(dioProvider);
  try {
    await syncWatchlistAlerts(dio);
  } catch (_) {}
  try {
    final res = await dio.get('/alerts');
    final data = unwrapData(res, (d) => d);
    if (data is List) {
      return data.map((e) => Map<String, dynamic>.from(e as Map)).toList();
    }
  } catch (_) {
    // Fall back to full notification inbox
  }
  final res = await dio.get('/notifications');
  final data = unwrapData(res, (d) => d);
  if (data is List) {
    return data.map((e) => Map<String, dynamic>.from(e as Map)).toList();
  }
  return [];
});

class NotificationsScreen extends ConsumerWidget {
  const NotificationsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final async = ref.watch(notificationsProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('IPO Alerts'),
        actions: [
          IconButton(
            tooltip: 'Alert settings',
            onPressed: () => context.push('/notifications/prefs'),
            icon: const Icon(Icons.tune),
          ),
          TextButton(
            onPressed: () async {
              final dio = ref.read(dioProvider);
              await dio.post('/notifications/read-all');
              ref.invalidate(notificationsProvider);
            },
            child: const Text('Mark all read'),
          ),
        ],
      ),
      body: async.when(
        data: (list) {
          if (list.isEmpty) {
            return Center(
              child: Padding(
                padding: const EdgeInsets.all(24),
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    const Icon(Icons.notifications_none, size: 48),
                    const SizedBox(height: 12),
                    Text(
                      'No IPO alerts yet',
                      style: Theme.of(context).textTheme.titleMedium,
                    ),
                    const SizedBox(height: 8),
                    const Text(
                      'Add IPOs to your watchlist and enable alert types in settings. '
                      'Alerts fire for open, close today, allotment, listing tomorrow, and listing today.',
                      textAlign: TextAlign.center,
                    ),
                  ],
                ),
              ),
            );
          }
          return RefreshIndicator(
            onRefresh: () async => ref.invalidate(notificationsProvider),
            child: ListView.separated(
              padding: const EdgeInsets.all(16),
              itemCount: list.length,
              separatorBuilder: (_, __) => const SizedBox(height: 8),
              itemBuilder: (context, i) {
                final n = list[i];
                final unread = n['read_at'] == null;
                return GlassCard(
                  onTap: () async {
                    if (unread) {
                      final dio = ref.read(dioProvider);
                      try {
                        await dio.post('/notifications/${n['id']}/read');
                        ref.invalidate(notificationsProvider);
                      } catch (e) {
                        if (context.mounted) {
                          ScaffoldMessenger.of(context).showSnackBar(
                            SnackBar(
                              content: Text(
                                e is Exception
                                    ? AppException.fromDio(e).message
                                    : '$e',
                              ),
                            ),
                          );
                        }
                      }
                    }
                  },
                  child: ListTile(
                    contentPadding: EdgeInsets.zero,
                    leading: Icon(
                      unread ? Icons.mark_email_unread : Icons.mark_email_read,
                      color: unread
                          ? Theme.of(context).colorScheme.primary
                          : null,
                    ),
                    title: Text(
                      n['title']?.toString() ?? '',
                      style: TextStyle(
                        fontWeight:
                            unread ? FontWeight.w700 : FontWeight.w500,
                      ),
                    ),
                    subtitle: Text(n['body']?.toString() ?? ''),
                  ),
                );
              },
            ),
          );
        },
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (e, _) => Center(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text('$e'),
              FilledButton(
                onPressed: () => ref.invalidate(notificationsProvider),
                child: const Text('Retry'),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class NotificationPrefsScreen extends ConsumerStatefulWidget {
  const NotificationPrefsScreen({super.key});

  @override
  ConsumerState<NotificationPrefsScreen> createState() =>
      _NotificationPrefsScreenState();
}

class _NotificationPrefsScreenState
    extends ConsumerState<NotificationPrefsScreen> {
  Map<String, dynamic>? _prefs;
  bool _loading = true;
  bool _saving = false;

  static const _ipoAlertKeys = <String, String>{
    'ipo_open': 'IPO opens',
    'ipo_close': 'IPO closes today',
    'allotment': 'Allotment announced',
    'listing_tomorrow': 'Listing tomorrow',
    'listing_day': 'Listing today',
  };

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    try {
      final dio = ref.read(dioProvider);
      Map<String, dynamic> prefs;
      try {
        final res = await dio.get('/alerts/preferences');
        prefs = Map<String, dynamic>.from(unwrapData(res, (d) => d as Map));
      } catch (_) {
        final res = await dio.get('/notifications/prefs');
        prefs = Map<String, dynamic>.from(unwrapData(res, (d) => d as Map));
      }
      // Ensure IPO keys exist with defaults
      for (final k in _ipoAlertKeys.keys) {
        prefs.putIfAbsent(k, () => true);
      }
      setState(() {
        _prefs = prefs;
        _loading = false;
      });
    } catch (e) {
      setState(() => _loading = false);
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(AppException.fromDio(e).message)),
        );
      }
    }
  }

  Future<void> _save() async {
    if (_prefs == null) return;
    setState(() => _saving = true);
    try {
      await saveAlertPreferences(ref, _prefs!);
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('Alert preferences saved')),
        );
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(AppException.fromDio(e).message)),
        );
      }
    } finally {
      if (mounted) setState(() => _saving = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_loading) {
      return const Scaffold(body: Center(child: CircularProgressIndicator()));
    }
    final prefs = _prefs ?? {};

    return Scaffold(
      appBar: AppBar(
        title: const Text('IPO alert settings'),
        actions: [
          TextButton(
            onPressed: _saving ? null : _save,
            child: _saving
                ? const SizedBox(
                    width: 18,
                    height: 18,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  )
                : const Text('Save'),
          ),
        ],
      ),
      body: ListView(
        children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(16, 16, 16, 8),
            child: Text(
              'Notifications for IPOs on your watchlist',
              style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                    color: Theme.of(context).colorScheme.onSurfaceVariant,
                  ),
            ),
          ),
          ..._ipoAlertKeys.entries.map((e) {
            return SwitchListTile(
              title: Text(e.value),
              subtitle: Text(e.key),
              value: prefs[e.key] == true,
              onChanged: (v) => setState(() => prefs[e.key] = v),
            );
          }),
          const Divider(),
          Padding(
            padding: const EdgeInsets.fromLTRB(16, 8, 16, 8),
            child: Text(
              'Other (optional)',
              style: Theme.of(context).textTheme.titleSmall,
            ),
          ),
          for (final k in [
            'portfolio_alert',
            'price_alert',
            'dividend_alert',
            'news_alert',
          ])
            SwitchListTile(
              title: Text(k.replaceAll('_', ' ')),
              value: prefs[k] == true,
              onChanged: (v) => setState(() => prefs[k] = v),
            ),
        ],
      ),
    );
  }
}
