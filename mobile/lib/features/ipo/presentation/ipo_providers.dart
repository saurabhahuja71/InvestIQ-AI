import 'package:dio/dio.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/network/api_client.dart';
import '../../../core/network/api_exception.dart';
import '../../../core/offline/offline_cache.dart';

final openIposProvider = FutureProvider<List<Map<String, dynamic>>>((ref) async {
  return ref.watch(ipoListProvider('open').future);
});

final ipoListProvider =
    FutureProvider.family<List<Map<String, dynamic>>, String?>((ref, status) async {
  final dio = ref.watch(dioProvider);
  final cache = ref.watch(offlineCacheProvider);
  final cacheKey = 'ipos:${status ?? 'all'}';

  try {
    final res = await dio.get('/ipos', queryParameters: {
      if (status != null) 'status': status,
      'per_page': 50,
    });
    final data = unwrapData(res, (d) => d);
    final list = data is List
        ? data.map((e) => Map<String, dynamic>.from(e as Map)).toList()
        : <Map<String, dynamic>>[];
    await cache.putJson(cacheKey, list);
    return list;
  } on DioException catch (e) {
    final cached = cache.getJson(cacheKey);
    if (cached is List) {
      return cached.map((e) => Map<String, dynamic>.from(e as Map)).toList();
    }
    throw AppException.fromDio(e);
  }
});

final ipoDetailProvider =
    FutureProvider.family<Map<String, dynamic>, String>((ref, id) async {
  final dio = ref.watch(dioProvider);
  final cache = ref.watch(offlineCacheProvider);
  final cacheKey = 'ipo:$id';
  try {
    final res = await dio.get('/ipos/$id');
    final data = unwrapData(res, (d) => Map<String, dynamic>.from(d as Map));
    await cache.putJson(cacheKey, data);
    return data;
  } on DioException catch (e) {
    final cached = cache.getJson(cacheKey);
    if (cached is Map) {
      return Map<String, dynamic>.from(cached);
    }
    throw AppException.fromDio(e);
  }
});

/// Pull-to-refresh: ask API to re-sync from NSE, then reload lists.
Future<void> refreshIpoFeed(WidgetRef ref, {String? status}) async {
  final dio = ref.read(dioProvider);
  try {
    await dio.post('/ipos/sync');
  } catch (_) {
    // Non-fatal: still reload whatever is in DB.
  }
  ref.invalidate(ipoListProvider(status));
  if (status != null) {
    ref.invalidate(ipoListProvider(null));
  }
  ref.invalidate(openIposProvider);
}
