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
          error.type == DioExceptionType.receiveTimeout ||
          error.type == DioExceptionType.sendTimeout) {
        final base = error.requestOptions.baseUrl;
        return AppException(
          'Cannot reach InvestIQ server ($base). '
          'Check Wi‑Fi, keep the phone USB-connected for local dev, '
          'and ensure the API is running on the PC.',
          offline: true,
          statusCode: error.response?.statusCode,
        );
      }
      if (error.type == DioExceptionType.unknown) {
        final msg = error.message ?? error.error?.toString() ?? '';
        if (msg.contains('SocketException') ||
            msg.contains('Connection') ||
            msg.contains('Failed host lookup') ||
            msg.contains('Network is unreachable')) {
          final base = error.requestOptions.baseUrl;
          return AppException(
            'Cannot reach InvestIQ server ($base). '
            'Check Wi‑Fi and that the API is running on the PC.',
            offline: true,
            statusCode: error.response?.statusCode,
          );
        }
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
