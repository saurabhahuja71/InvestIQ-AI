class AppConstants {
  /// Primary API origin (no trailing slash). Override with:
  /// `--dart-define=API_BASE_URL=https://…`
  static const apiBaseUrl = String.fromEnvironment(
    'API_BASE_URL',
    defaultValue: 'http://10.0.2.2:8080', // Android emulator → host
  );

  /// Comma-separated extra origins tried if [apiBaseUrl] is unreachable.
  /// Example: `http://127.0.0.1:8080,http://192.168.0.9:8080`
  static const apiFallbackBaseUrls = String.fromEnvironment(
    'API_FALLBACK_BASE_URLS',
    defaultValue: '',
  );

  /// Ordered unique candidates for API connectivity.
  static List<String> apiBaseCandidates() {
    final raw = <String>[
      apiBaseUrl,
      ...apiFallbackBaseUrls.split(','),
      // GCP static host serving the InvestIQ API (docs/15-gcp-api-host.md).
      'http://136.67.97.86:8080',
      // onenova.in tunnel — website only until proxied to the API.
      'https://onenova.in',
      // Always try common local-dev endpoints last.
      'http://127.0.0.1:8080',
      'http://10.0.2.2:8080',
    ];
    final seen = <String>{};
    final out = <String>[];
    for (final u in raw) {
      final t = u.trim().replaceAll(RegExp(r'/+$'), '');
      if (t.isEmpty || seen.contains(t)) continue;
      seen.add(t);
      out.add(t);
    }
    return out;
  }

  static const accessTokenKey = 'access_token';
  static const refreshTokenKey = 'refresh_token';

  static const investmentDisclaimer =
      'This is not financial advice. Past performance does not guarantee future results. '
      'InvestIQ AI does not provide guaranteed returns. Markets involve risk of loss.';
}
