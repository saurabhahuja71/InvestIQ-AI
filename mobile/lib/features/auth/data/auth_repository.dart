import 'package:dio/dio.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/network/api_client.dart';
import '../../../core/storage/secure_storage.dart';
import '../domain/user.dart';
import 'google_auth_service.dart';

final authRepositoryProvider = Provider<AuthRepository>((ref) {
  return AuthRepository(
    ref.watch(dioProvider),
    ref.watch(secureStorageProvider),
    googleAuthService,
  );
});

class AuthRepository {
  AuthRepository(this._dio, this._storage, this._google);
  final Dio _dio;
  final SecureStorageService _storage;
  final GoogleAuthService _google;

  Future<User> register({
    required String email,
    required String password,
    String? fullName,
  }) async {
    final res = await _dio.post('/auth/register', data: {
      'email': email,
      'password': password,
      'full_name': fullName,
    });
    return _persistAuth(res);
  }

  Future<User> login({required String email, required String password}) async {
    final res = await _dio.post('/auth/login', data: {
      'email': email,
      'password': password,
    });
    return _persistAuth(res);
  }

  Future<User> loginWithGoogle() async {
    final idToken = await _google.signInAndGetIdToken();
    final res = await _dio.post('/auth/google', data: {
      'id_token': idToken,
    });
    return _persistAuth(res);
  }

  /// Starts a password reset. Returns the one-time reset code when the
  /// account exists, or null when no matching account was found.
  Future<String?> forgotPassword(String email) async {
    final res = await _dio.post('/auth/forgot-password', data: {
      'email': email.trim(),
    });
    final data = unwrapData(res, (d) => d as Map<String, dynamic>);
    if (data['sent'] != true) return null;
    return data['reset_token'] as String?;
  }

  Future<void> resetPassword({
    required String email,
    required String token,
    required String newPassword,
  }) async {
    final res = await _dio.post('/auth/reset-password', data: {
      'email': email.trim(),
      'token': token.trim(),
      'new_password': newPassword,
    });
    unwrapData(res, (d) => d);
  }

  Future<User> _persistAuth(Response res) async {
    final data = unwrapData(res, (d) => d as Map<String, dynamic>);
    await _storage.saveTokens(
      access: data['access_token'] as String,
      refresh: data['refresh_token'] as String,
    );
    return User.fromJson(data['user'] as Map<String, dynamic>);
  }

  Future<User?> me() async {
    final token = await _storage.getAccessToken();
    if (token == null) return null;
    try {
      final res = await _dio.get('/auth/me');
      return unwrapData(res, (d) => User.fromJson(d as Map<String, dynamic>));
    } catch (_) {
      return null;
    }
  }

  Future<void> logout() async {
    final refresh = await _storage.getRefreshToken();
    try {
      if (refresh != null) {
        await _dio.post('/auth/logout', data: {'refresh_token': refresh});
      }
    } catch (_) {}
    try {
      await _google.signOut();
    } catch (_) {}
    await _storage.clear();
  }

  Future<bool> hasSession() async {
    final t = await _storage.getAccessToken();
    return t != null && t.isNotEmpty;
  }
}
