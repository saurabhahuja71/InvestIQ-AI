class User {
  const User({
    required this.id,
    required this.email,
    this.fullName,
    this.preferredCurrency = 'INR',
    this.themePreference = 'system',
    this.biometricEnabled = false,
  });

  final String id;
  final String email;
  final String? fullName;
  final String preferredCurrency;
  final String themePreference;
  final bool biometricEnabled;

  factory User.fromJson(Map<String, dynamic> json) {
    return User(
      id: json['id'] as String,
      email: json['email'] as String,
      fullName: json['full_name'] as String?,
      preferredCurrency: json['preferred_currency'] as String? ?? 'INR',
      themePreference: json['theme_preference'] as String? ?? 'system',
      biometricEnabled: json['biometric_enabled'] as bool? ?? false,
    );
  }
}
