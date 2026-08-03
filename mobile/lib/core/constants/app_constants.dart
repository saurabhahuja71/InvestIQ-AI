class AppConstants {
  /// Override with: flutter run --dart-define=API_BASE_URL=https://api.example.com
  static const apiBaseUrl = String.fromEnvironment(
    'API_BASE_URL',
    defaultValue: 'http://10.0.2.2:8080', // Android emulator → host
  );

  static const accessTokenKey = 'access_token';
  static const refreshTokenKey = 'refresh_token';

  static const investmentDisclaimer =
      'This is not financial advice. Past performance does not guarantee future results. '
      'InvestIQ AI does not provide guaranteed returns. Markets involve risk of loss.';

  static const gmpDisclaimer =
      'Grey Market Premium (GMP) is unofficial and not endorsed by any exchange or regulator.';
}
