import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/network/api_client.dart';

final portfoliosProvider = FutureProvider<List<Map<String, dynamic>>>((ref) async {
  final dio = ref.watch(dioProvider);
  final res = await dio.get('/portfolios');
  final data = unwrapData(res, (d) => d);
  if (data is List) {
    return data.map((e) => Map<String, dynamic>.from(e as Map)).toList();
  }
  return [];
});

final portfolioDashboardProvider =
    FutureProvider<Map<String, dynamic>?>((ref) async {
  final portfolios = await ref.watch(portfoliosProvider.future);
  if (portfolios.isEmpty) return null;
  final id = portfolios.first['id'] as String;
  final dio = ref.watch(dioProvider);
  final res = await dio.get('/portfolios/$id');
  return unwrapData(res, (d) => Map<String, dynamic>.from(d as Map));
});
