import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/network/api_client.dart';

final openIposProvider = FutureProvider<List<Map<String, dynamic>>>((ref) async {
  final dio = ref.watch(dioProvider);
  final res = await dio.get('/ipos', queryParameters: {'status': 'open', 'per_page': 20});
  final data = unwrapData(res, (d) => d);
  if (data is List) {
    return data.map((e) => Map<String, dynamic>.from(e as Map)).toList();
  }
  return [];
});

final ipoListProvider =
    FutureProvider.family<List<Map<String, dynamic>>, String?>((ref, status) async {
  final dio = ref.watch(dioProvider);
  final res = await dio.get('/ipos', queryParameters: {
    if (status != null) 'status': status,
    'per_page': 50,
  });
  final data = unwrapData(res, (d) => d);
  if (data is List) {
    return data.map((e) => Map<String, dynamic>.from(e as Map)).toList();
  }
  return [];
});

final ipoDetailProvider =
    FutureProvider.family<Map<String, dynamic>, String>((ref, id) async {
  final dio = ref.watch(dioProvider);
  final res = await dio.get('/ipos/$id');
  return unwrapData(res, (d) => Map<String, dynamic>.from(d as Map));
});
