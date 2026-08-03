import 'package:dio/dio.dart';

class AppException implements Exception {
  AppException(this.message, {this.code, this.statusCode, this.offline = false});

  final String message;
  final String? code;
  final int? statusCode;
  final bool offline;

  @override
  String toString() => message;

  static AppException fromDio(Object error) {
    if (error is DioException) {
      if (error.type == DioExceptionType.connectionError ||
          error.type == DioExceptionType.connectionTimeout ||
          error.type == DioExceptionType.unknown) {
        return AppException(
          'No network connection. Showing cached data when available.',
          offline: true,
          statusCode: error.response?.statusCode,
        );
      }
      final data = error.response?.data;
      if (data is Map && data['error'] is Map) {
        final err = data['error'] as Map;
        return AppException(
          err['message']?.toString() ?? 'Request failed',
          code: err['code']?.toString(),
          statusCode: error.response?.statusCode,
        );
      }
      return AppException(
        error.message ?? 'Request failed',
        statusCode: error.response?.statusCode,
      );
    }
    return AppException(error.toString());
  }
}
