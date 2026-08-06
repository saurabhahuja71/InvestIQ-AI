import 'package:dio/dio.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/network/api_client.dart';
import '../../../core/network/api_exception.dart';
import '../../../core/offline/offline_cache.dart';

const _watchlistCacheKey = 'watchlist:ipos';
const _alertPrefsCacheKey = 'alerts:preferences';

class WatchlistResult {
  const WatchlistResult({
    required this.items,
    this.fromCache = false,
  });

  final List<Map<String, dynamic>> items;
  final bool fromCache;

  int get count => items.length;

  Set<String> get ids => items
      .map((e) => e['id']?.toString())
      .whereType<String>()
      .where((id) => id.isNotEmpty)
      .toSet();
}

final watchlistProvider = FutureProvider<WatchlistResult>((ref) async {
  final dio = ref.watch(dioProvider);
  final cache = ref.watch(offlineCacheProvider);

  try {
    final res = await dio.get('/watchlist');
    final data = unwrapData(res, (d) => d);
    final list = data is List
        ? data.map((e) => Map<String, dynamic>.from(e as Map)).toList()
        : <Map<String, dynamic>>[];
    await cache.putJson(_watchlistCacheKey, list);
    return WatchlistResult(items: list);
  } on DioException catch (e) {
    final cached = cache.getJson(_watchlistCacheKey);
    if (cached is List) {
      return WatchlistResult(
        items: cached.map((e) => Map<String, dynamic>.from(e as Map)).toList(),
        fromCache: true,
      );
    }
    // Fallback legacy path
    try {
      final res = await dio.get('/ipos/watchlist');
      final data = unwrapData(res, (d) => d);
      final list = data is List
          ? data.map((e) => Map<String, dynamic>.from(e as Map)).toList()
          : <Map<String, dynamic>>[];
      await cache.putJson(_watchlistCacheKey, list);
      return WatchlistResult(items: list);
    } catch (_) {
      throw AppException.fromDio(e);
    }
  }
});

final watchedIpoIdsProvider = Provider<Set<String>>((ref) {
  final async = ref.watch(watchlistProvider);
  return async.maybeWhen(
    data: (r) => r.ids,
    orElse: () {
      final cache = ref.watch(offlineCacheProvider);
      final cached = cache.getJson(_watchlistCacheKey);
      if (cached is List) {
        return cached
            .map((e) => (e as Map)['id']?.toString())
            .whereType<String>()
            .where((id) => id.isNotEmpty)
            .toSet();
      }
      return <String>{};
    },
  );
});

final watchlistCountProvider = Provider<int>((ref) {
  return ref.watch(watchedIpoIdsProvider).length;
});

Future<void> addToWatchlist(
  WidgetRef ref,
  String ipoId, {
  Map<String, dynamic>? snapshot,
}) async {
  final dio = ref.read(dioProvider);
  final cache = ref.read(offlineCacheProvider);
  try {
    await dio.post('/watchlist', data: {'ipo_id': ipoId});
  } on DioException catch (e) {
    if (!_isOffline(e)) rethrow;
    // Offline queue + optimistic local cache (syncs when connectivity returns)
    await cache.enqueueWrite('/watchlist', 'POST', {'ipo_id': ipoId});
    final existing = cache.getJson(_watchlistCacheKey);
    final list = existing is List
        ? existing.map((e) => Map<String, dynamic>.from(e as Map)).toList()
        : <Map<String, dynamic>>[];
    if (!list.any((e) => e['id']?.toString() == ipoId)) {
      list.insert(0, {
        ...?snapshot,
        'id': ipoId,
        'company_name':
            snapshot?['company_name']?.toString() ?? 'Pending sync',
        'status': snapshot?['status']?.toString() ?? 'Not Available',
      });
      await cache.putJson(_watchlistCacheKey, list);
    }
  }
  ref.invalidate(watchlistProvider);
}

Future<void> removeFromWatchlist(WidgetRef ref, String ipoId) async {
  final dio = ref.read(dioProvider);
  final cache = ref.read(offlineCacheProvider);
  try {
    await dio.delete('/watchlist/$ipoId');
  } on DioException catch (e) {
    if (!_isOffline(e)) rethrow;
    await cache.enqueueWrite('/watchlist/$ipoId', 'DELETE', {});
    final existing = cache.getJson(_watchlistCacheKey);
    if (existing is List) {
      final list = existing
          .map((e) => Map<String, dynamic>.from(e as Map))
          .where((e) => e['id']?.toString() != ipoId)
          .toList();
      await cache.putJson(_watchlistCacheKey, list);
    }
  }
  ref.invalidate(watchlistProvider);
}

bool _isOffline(DioException e) {
  return e.type == DioExceptionType.connectionError ||
      e.type == DioExceptionType.connectionTimeout ||
      e.type == DioExceptionType.receiveTimeout ||
      e.type == DioExceptionType.sendTimeout ||
      e.response == null;
}

Future<void> toggleWatchlist(
  WidgetRef ref,
  String ipoId,
  bool currentlyWatched, {
  Map<String, dynamic>? snapshot,
}) async {
  if (currentlyWatched) {
    await removeFromWatchlist(ref, ipoId);
  } else {
    await addToWatchlist(ref, ipoId, snapshot: snapshot);
  }
}

final alertPreferencesProvider =
    FutureProvider<Map<String, dynamic>>((ref) async {
  final dio = ref.watch(dioProvider);
  final cache = ref.watch(offlineCacheProvider);
  try {
    final res = await dio.get('/alerts/preferences');
    final data = unwrapData(res, (d) => Map<String, dynamic>.from(d as Map));
    await cache.putJson(_alertPrefsCacheKey, data);
    return data;
  } on DioException catch (e) {
    final cached = cache.getJson(_alertPrefsCacheKey);
    if (cached is Map) {
      return Map<String, dynamic>.from(cached);
    }
    // Fallback to notifications prefs
    try {
      final res = await dio.get('/notifications/prefs');
      final data = unwrapData(res, (d) => Map<String, dynamic>.from(d as Map));
      await cache.putJson(_alertPrefsCacheKey, data);
      return data;
    } catch (_) {
      throw AppException.fromDio(e);
    }
  }
});

Future<void> saveAlertPreferences(
  WidgetRef ref,
  Map<String, dynamic> prefs,
) async {
  final dio = ref.read(dioProvider);
  final cache = ref.read(offlineCacheProvider);
  await dio.put('/alerts/preferences', data: {'preferences': prefs});
  await cache.putJson(_alertPrefsCacheKey, prefs);
  ref.invalidate(alertPreferencesProvider);
}

Future<void> syncWatchlistAlerts(Dio dio) async {
  try {
    await dio.post('/alerts/sync');
  } catch (_) {
    try {
      await dio.post('/notifications/sync-ipo-events');
    } catch (_) {}
  }
}
