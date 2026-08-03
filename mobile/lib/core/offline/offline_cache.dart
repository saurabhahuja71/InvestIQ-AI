import 'dart:convert';

import 'package:connectivity_plus/connectivity_plus.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:hive_flutter/hive_flutter.dart';

final offlineCacheProvider = Provider<OfflineCache>((ref) => OfflineCache());

final connectivityProvider = StreamProvider<List<ConnectivityResult>>((ref) {
  return Connectivity().onConnectivityChanged;
});

class OfflineCache {
  Box get _box => Hive.box('cache');

  Future<void> putJson(String key, Object data) async {
    await _box.put(key, jsonEncode(data));
  }

  dynamic getJson(String key) {
    final raw = _box.get(key);
    if (raw is! String) return null;
    try {
      return jsonDecode(raw);
    } catch (_) {
      return null;
    }
  }

  Future<void> enqueueWrite(String path, String method, Map<String, dynamic> body) async {
    final list = List<Map<String, dynamic>>.from(
      (getJson('write_queue') as List?)?.map((e) => Map<String, dynamic>.from(e as Map)) ??
          [],
    );
    list.add({
      'path': path,
      'method': method,
      'body': body,
      'queued_at': DateTime.now().toIso8601String(),
    });
    await putJson('write_queue', list);
  }

  List<Map<String, dynamic>> peekQueue() {
    final raw = getJson('write_queue');
    if (raw is! List) return [];
    return raw.map((e) => Map<String, dynamic>.from(e as Map)).toList();
  }

  Future<void> clearQueue() async => putJson('write_queue', <dynamic>[]);

  Future<void> setQueue(List<Map<String, dynamic>> items) async =>
      putJson('write_queue', items);
}
