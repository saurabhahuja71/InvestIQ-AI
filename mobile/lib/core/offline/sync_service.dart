import 'package:dio/dio.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../network/api_client.dart';
import 'offline_cache.dart';

final syncServiceProvider = Provider<SyncService>((ref) {
  return SyncService(ref.watch(dioProvider), ref.watch(offlineCacheProvider));
});

class SyncService {
  SyncService(this._dio, this._cache);
  final Dio _dio;
  final OfflineCache _cache;

  /// Flush offline write queue (FIFO). Failed items remain at head.
  Future<int> flush() async {
    final queue = _cache.peekQueue();
    if (queue.isEmpty) return 0;
    var flushed = 0;
    final remaining = <Map<String, dynamic>>[];
    var failed = false;

    for (final item in queue) {
      if (failed) {
        remaining.add(item);
        continue;
      }
      try {
        final method = (item['method'] as String? ?? 'POST').toUpperCase();
        final path = item['path'] as String;
        final body = item['body'] as Map<String, dynamic>?;
        switch (method) {
          case 'POST':
            await _dio.post(path, data: body);
            break;
          case 'PATCH':
            await _dio.patch(path, data: body);
            break;
          case 'PUT':
            await _dio.put(path, data: body);
            break;
          case 'DELETE':
            await _dio.delete(path, data: body);
            break;
          default:
            await _dio.post(path, data: body);
        }
        flushed++;
      } catch (_) {
        failed = true;
        remaining.add(item);
      }
    }
    await _cache.setQueue(remaining);
    return flushed;
  }
}
