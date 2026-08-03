import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/network/api_client.dart';
import '../../../core/network/api_exception.dart';
import '../../../core/widgets/glass_card.dart';

final notificationsProvider =
    FutureProvider<List<Map<String, dynamic>>>((ref) async {
  final dio = ref.watch(dioProvider);
  try {
    await dio.post('/notifications/sync-ipo-events');
  } catch (_) {}
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
        title: const Text('Notifications'),
        actions: [
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
            return const Center(child: Text('No notifications yet'));
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
                            SnackBar(content: Text(AppException.fromDio(e).message)),
                          );
                        }
                      }
                    }
                  },
                  child: ListTile(
                    contentPadding: EdgeInsets.zero,
                    leading: Icon(
                      unread ? Icons.mark_email_unread : Icons.mark_email_read,
                      color: unread ? Theme.of(context).colorScheme.primary : null,
                    ),
                    title: Text(
                      n['title']?.toString() ?? '',
                      style: TextStyle(
                        fontWeight: unread ? FontWeight.w700 : FontWeight.w500,
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
        error: (e, _) => Center(child: Text('$e')),
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

class _NotificationPrefsScreenState extends ConsumerState<NotificationPrefsScreen> {
  Map<String, dynamic>? _prefs;
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    try {
      final dio = ref.read(dioProvider);
      final res = await dio.get('/notifications/prefs');
      setState(() {
        _prefs = Map<String, dynamic>.from(
          unwrapData(res, (d) => d as Map),
        );
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
    final dio = ref.read(dioProvider);
    await dio.put('/notifications/prefs', data: {'prefs': _prefs});
    if (mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Preferences saved')),
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_loading) {
      return const Scaffold(body: Center(child: CircularProgressIndicator()));
    }
    final prefs = _prefs ?? {};
    final keys = [
      'ipo_open',
      'ipo_close',
      'allotment',
      'listing_day',
      'portfolio_alert',
      'price_alert',
      'dividend_alert',
      'news_alert',
    ];

    return Scaffold(
      appBar: AppBar(
        title: const Text('Notification prefs'),
        actions: [
          TextButton(onPressed: _save, child: const Text('Save')),
        ],
      ),
      body: ListView(
        children: keys.map((k) {
          return SwitchListTile(
            title: Text(k.replaceAll('_', ' ')),
            value: prefs[k] == true,
            onChanged: (v) => setState(() => prefs[k] = v),
          );
        }).toList(),
      ),
    );
  }
}
