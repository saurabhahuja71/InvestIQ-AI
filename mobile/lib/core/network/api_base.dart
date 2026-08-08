import 'package:dio/dio.dart';
import 'package:flutter/foundation.dart';

import '../constants/app_constants.dart';

/// Resolves a reachable InvestIQ API origin (cached after first success).
class ApiBase {
  ApiBase._();

  static String? _resolved;
  static final Dio _probe = Dio(
    BaseOptions(
      connectTimeout: const Duration(seconds: 3),
      receiveTimeout: const Duration(seconds: 3),
      sendTimeout: const Duration(seconds: 3),
      // Only a real 2xx means this base serves the API. A website that
      // responds 404 to /health must NOT be picked as the API origin.
      validateStatus: (s) => s != null && s >= 200 && s < 300,
    ),
  );

  static String get current =>
      _resolved ?? AppConstants.apiBaseUrl.replaceAll(RegExp(r'/+$'), '');

  static void reset() => _resolved = null;

  /// Probes `/health` on each candidate; returns first that responds.
  static Future<String> resolve({bool force = false}) async {
    if (!force && _resolved != null) return _resolved!;

    final candidates = AppConstants.apiBaseCandidates();
    Object? lastError;

    for (final base in candidates) {
      final url = '$base/health';
      try {
        if (kDebugMode) {
          debugPrint('ApiBase probe $url');
        }
        // ignore: avoid_print
        print('InvestIQ-API: probe $url');
        final res = await _probe.get<dynamic>(url);
        // A real 2xx that looks like our API. A website that returns 200 for
        // every path (e.g. onenova.in) must NOT be picked — it would 405 the
        // API routes later.
        if (res.statusCode != null &&
            res.statusCode! >= 200 &&
            res.statusCode! < 300 &&
            _looksLikeApi(res.data)) {
          _resolved = base;
          // ignore: avoid_print
          print('InvestIQ-API: using $base');
          return base;
        }
      } catch (e) {
        lastError = e;
        // ignore: avoid_print
        print('InvestIQ-API: probe failed $base → $e');
      }
    }

    // Keep primary so callers still attempt the request and surface a clear error.
    _resolved = AppConstants.apiBaseUrl.replaceAll(RegExp(r'/+$'), '');
    // ignore: avoid_print
    print('InvestIQ-API: no candidate healthy; defaulting to $_resolved lastError=$lastError');
    return _resolved!;
  }

  /// True when the health payload is the InvestIQ API (e.g.
  /// `{"service":"investiq-api","status":"ok"}`) rather than an HTML page.
  static bool _looksLikeApi(dynamic body) {
    if (body is! Map) return false;
    final service = body['service']?.toString();
    if (service == 'investiq-api') return true;
    return body['status']?.toString() == 'ok' &&
        body.keys.length <= 4 &&
        !body.containsKey('html') &&
        !body.containsKey('title');
  }
}
