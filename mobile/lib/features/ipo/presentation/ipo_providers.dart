import 'package:dio/dio.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/network/api_client.dart';
import '../../../core/network/api_exception.dart';
import '../../../core/offline/offline_cache.dart';

class IpoListResult {
  const IpoListResult({
    required this.items,
    required this.page,
    required this.perPage,
    required this.total,
    this.fromCache = false,
  });

  final List<Map<String, dynamic>> items;
  final int page;
  final int perPage;
  final int total;
  final bool fromCache;

  bool get hasMore => page * perPage < total;
}

class IpoListParams {
  const IpoListParams({
    this.status,
    this.board,
    this.query = '',
    this.page = 1,
    this.perPage = 20,
    this.refresh = false,
  });

  final String? status;
  final String? board;
  final String query;
  final int page;
  final int perPage;
  final bool refresh;

  IpoListParams copyWith({
    String? status,
    String? board,
    String? query,
    int? page,
    int? perPage,
    bool? refresh,
    bool clearStatus = false,
    bool clearBoard = false,
  }) {
    return IpoListParams(
      status: clearStatus ? null : (status ?? this.status),
      board: clearBoard ? null : (board ?? this.board),
      query: query ?? this.query,
      page: page ?? this.page,
      perPage: perPage ?? this.perPage,
      refresh: refresh ?? this.refresh,
    );
  }

  String get cacheKey =>
      'ipos:${status ?? 'all'}:${board ?? 'all'}:${query.trim().toLowerCase()}:$page';

  @override
  bool operator ==(Object other) =>
      other is IpoListParams &&
      other.status == status &&
      other.board == board &&
      other.query == query &&
      other.page == page &&
      other.perPage == perPage &&
      other.refresh == refresh;

  @override
  int get hashCode => Object.hash(status, board, query, page, perPage, refresh);
}

final openIposProvider = FutureProvider<List<Map<String, dynamic>>>((ref) async {
  final result = await ref.watch(
    ipoListQueryProvider(const IpoListParams(status: 'open', perPage: 10)).future,
  );
  return result.items;
});

final ipoListQueryProvider =
    FutureProvider.family<IpoListResult, IpoListParams>((ref, params) async {
  final dio = ref.watch(dioProvider);
  final cache = ref.watch(offlineCacheProvider);

  try {
    final res = await dio.get('/ipos', queryParameters: {
      if (params.status != null) 'status': params.status,
      if (params.board != null) 'board': params.board,
      if (params.query.trim().isNotEmpty) 'q': params.query.trim(),
      'page': params.page,
      'per_page': params.perPage,
      if (params.refresh) 'refresh': true,
    });
    final body = res.data;
    final data = unwrapData(res, (d) => d);
    final list = data is List
        ? data.map((e) => Map<String, dynamic>.from(e as Map)).toList()
        : <Map<String, dynamic>>[];
    final meta = body is Map ? body['meta'] as Map? : null;
    final result = IpoListResult(
      items: list,
      page: (meta?['page'] as num?)?.toInt() ?? params.page,
      perPage: (meta?['per_page'] as num?)?.toInt() ?? params.perPage,
      total: (meta?['total'] as num?)?.toInt() ?? list.length,
    );
    await cache.putJson(params.cacheKey, {
      'items': list,
      'page': result.page,
      'per_page': result.perPage,
      'total': result.total,
    });
    return result;
  } on DioException catch (e) {
    final cached = cache.getJson(params.cacheKey);
    if (cached is Map) {
      final items = (cached['items'] as List?)
              ?.map((e) => Map<String, dynamic>.from(e as Map))
              .toList() ??
          [];
      return IpoListResult(
        items: items,
        page: (cached['page'] as num?)?.toInt() ?? params.page,
        perPage: (cached['per_page'] as num?)?.toInt() ?? params.perPage,
        total: (cached['total'] as num?)?.toInt() ?? items.length,
        fromCache: true,
      );
    }
    throw AppException.fromDio(e);
  }
});

/// Legacy helper kept for home screen / older call sites.
final ipoListProvider =
    FutureProvider.family<List<Map<String, dynamic>>, String?>((ref, status) async {
  final result = await ref.watch(
    ipoListQueryProvider(IpoListParams(status: status, perPage: 50)).future,
  );
  return result.items;
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

Future<Map<String, dynamic>> _fetchIntel(
  Ref ref,
  String path,
  String cacheKey,
) async {
  final dio = ref.watch(dioProvider);
  final cache = ref.watch(offlineCacheProvider);
  try {
    final res = await dio.get(path);
    final data = unwrapData(res, (d) => Map<String, dynamic>.from(d as Map));
    await cache.putJson(cacheKey, data);
    return data;
  } on DioException catch (e) {
    final cached = cache.getJson(cacheKey);
    if (cached is Map) {
      return Map<String, dynamic>.from(cached);
    }
    // 404 = intelligence for this IPO is not computed yet. Signal "no data"
    // so sections render their Not Available state instead of an error.
    if (e.response?.statusCode == 404) {
      return <String, dynamic>{};
    }
    throw AppException.fromDio(e);
  }
}

/// InvestIQ IPO Score (fundamentals-driven) for an IPO.
final ipoScoreProvider =
    FutureProvider.family<Map<String, dynamic>, String>((ref, id) {
  return _fetchIntel(ref, '/ipos/$id/score', 'ipo_score:$id');
});

/// Live subscription data (NSE official feed) for an IPO.
///
/// Falls back to the live subscription fields on the IPO detail payload when
/// the intel endpoint has no snapshot yet (e.g. older API deployments).
final ipoSubscriptionProvider =
    FutureProvider.family<Map<String, dynamic>, String>((ref, id) async {
  final sub = await _fetchIntel(
    ref,
    '/ipos/$id/subscription',
    'ipo_subscription:$id',
  );
  if (sub['available'] == true && sub['overall'] != null) return sub;
  final detail = await ref.watch(ipoDetailProvider(id).future);
  final overall = detail['subscription_total'];
  if (overall == null) return sub;
  return <String, dynamic>{
    'available': true,
    'overall': overall,
    'qib': detail['subscription_qib'],
    'nii': detail['subscription_nii'],
    'retail': detail['subscription_retail'],
    'employee': null,
    'shareholder': null,
    'is_final': false,
    'source_type': detail['source'],
    'updated_at': detail['source_synced_at'],
  };
});

/// Financial performance + growth + valuation analysis for an IPO.
final ipoFinancialsProvider =
    FutureProvider.family<Map<String, dynamic>, String>((ref, id) {
  return _fetchIntel(ref, '/ipos/$id/financials', 'ipo_financials:$id');
});

Future<void> refreshIpoFeed(Dio dio) async {
  try {
    await dio.post('/ipos/sync');
  } catch (_) {
    // Best-effort; list refresh still loads DB / cache.
  }
}
