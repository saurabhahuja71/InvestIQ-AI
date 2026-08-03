import 'package:dio/dio.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/network/api_client.dart';
import '../../../core/network/api_exception.dart';
import '../../../core/offline/offline_cache.dart';

final portfoliosProvider = FutureProvider<List<Map<String, dynamic>>>((ref) async {
  final dio = ref.watch(dioProvider);
  final cache = ref.watch(offlineCacheProvider);
  try {
    final res = await dio.get('/portfolios');
    final data = unwrapData(res, (d) => d);
    final list = data is List
        ? data.map((e) => Map<String, dynamic>.from(e as Map)).toList()
        : <Map<String, dynamic>>[];
    await cache.putJson('portfolios', list);
    return list;
  } on DioException catch (e) {
    final cached = cache.getJson('portfolios');
    if (cached is List) {
      return cached.map((e) => Map<String, dynamic>.from(e as Map)).toList();
    }
    throw AppException.fromDio(e);
  }
});

final portfolioDashboardProvider =
    FutureProvider<Map<String, dynamic>?>((ref) async {
  final portfolios = await ref.watch(portfoliosProvider.future);
  if (portfolios.isEmpty) return null;
  final id = portfolios.first['id'] as String;
  final dio = ref.watch(dioProvider);
  final cache = ref.watch(offlineCacheProvider);
  final key = 'portfolio_dash:$id';
  try {
    final res = await dio.get('/portfolios/$id');
    final data = unwrapData(res, (d) => Map<String, dynamic>.from(d as Map));
    await cache.putJson(key, data);
    return data;
  } on DioException catch (e) {
    final cached = cache.getJson(key);
    if (cached is Map) return Map<String, dynamic>.from(cached);
    throw AppException.fromDio(e);
  }
});
