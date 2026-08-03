import 'package:dio/dio.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../constants/app_constants.dart';
import '../storage/secure_storage.dart';
import 'api_exception.dart';

final dioProvider = Provider<Dio>((ref) {
  final storage = ref.watch(secureStorageProvider);
  final dio = Dio(
    BaseOptions(
      baseUrl: '${AppConstants.apiBaseUrl}/api/v1',
      connectTimeout: const Duration(seconds: 20),
      receiveTimeout: const Duration(seconds: 30),
      headers: {
        'Content-Type': 'application/json',
        'Accept': 'application/json',
      },
      validateStatus: (s) => s != null && s >= 200 && s < 300,
    ),
  );

  dio.interceptors.add(
    InterceptorsWrapper(
      onRequest: (options, handler) async {
        final token = await storage.getAccessToken();
        if (token != null && token.isNotEmpty) {
          options.headers['Authorization'] = 'Bearer $token';
        }
        handler.next(options);
      },
      onError: (error, handler) async {
        if (error.response?.statusCode == 401) {
          final refreshed = await _tryRefresh(dio, storage);
          if (refreshed) {
            final token = await storage.getAccessToken();
            final req = error.requestOptions;
            req.headers['Authorization'] = 'Bearer $token';
            try {
              final clone = await dio.fetch(req);
              return handler.resolve(clone);
            } catch (e) {
              return handler.reject(
                DioException(
                  requestOptions: error.requestOptions,
                  error: AppException.fromDio(e),
                  response: error.response,
                  type: error.type,
                ),
              );
            }
          }
        }
        handler.reject(
          DioException(
            requestOptions: error.requestOptions,
            error: AppException.fromDio(error),
            response: error.response,
            type: error.type,
            message: AppException.fromDio(error).message,
          ),
        );
      },
    ),
  );

  return dio;
});

Future<bool> _tryRefresh(Dio dio, SecureStorageService storage) async {
  final refresh = await storage.getRefreshToken();
  if (refresh == null) return false;
  try {
    final res = await Dio(BaseOptions(baseUrl: dio.options.baseUrl)).post(
      '/auth/refresh',
      data: {'refresh_token': refresh},
    );
    final data = res.data['data'] as Map<String, dynamic>;
    await storage.saveTokens(
      access: data['access_token'] as String,
      refresh: data['refresh_token'] as String,
    );
    return true;
  } catch (_) {
    await storage.clear();
    return false;
  }
}

T unwrapData<T>(Response response, T Function(dynamic json) map) {
  final body = response.data;
  if (body is Map && body['success'] == true) {
    return map(body['data']);
  }
  final err = body is Map ? body['error'] : null;
  throw AppException(
    err is Map
        ? (err['message']?.toString() ?? 'Request failed')
        : 'Request failed',
    code: err is Map ? err['code']?.toString() : null,
    statusCode: response.statusCode,
  );
}
