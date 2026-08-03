import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../core/network/api_client.dart';
import '../../../core/offline/offline_cache.dart';
import 'journal_screen.dart';

class TradeEntryScreen extends ConsumerStatefulWidget {
  const TradeEntryScreen({super.key});

  @override
  ConsumerState<TradeEntryScreen> createState() => _TradeEntryScreenState();
}

class _TradeEntryScreenState extends ConsumerState<TradeEntryScreen> {
  final _symbol = TextEditingController();
  final _entry = TextEditingController();
  final _exit = TextEditingController();
  final _qty = TextEditingController(text: '1');
  final _strategy = TextEditingController();
  final _notes = TextEditingController();
  final _rr = TextEditingController();
  String _side = 'long';
  String? _emotionBefore;
  String? _emotionAfter;
  bool _saving = false;

  final _emotions = const [
    'confident',
    'fearful',
    'greedy',
    'fomo',
    'calm',
    'anxious',
    'revenge',
    'neutral',
  ];

  @override
  void dispose() {
    _symbol.dispose();
    _entry.dispose();
    _exit.dispose();
    _qty.dispose();
    _strategy.dispose();
    _notes.dispose();
    _rr.dispose();
    super.dispose();
  }

  Future<void> _save() async {
    setState(() => _saving = true);
    final body = {
      'symbol': _symbol.text.trim().toUpperCase(),
      'side': _side,
      'entry_price': _entry.text,
      'exit_price': _exit.text.isEmpty ? null : _exit.text,
      'quantity': _qty.text,
      'entry_at': DateTime.now().toUtc().toIso8601String(),
      'exit_at':
          _exit.text.isEmpty ? null : DateTime.now().toUtc().toIso8601String(),
      'strategy_name':
          _strategy.text.trim().isEmpty ? null : _strategy.text.trim(),
      'risk_reward': _rr.text.isEmpty ? null : _rr.text,
      'emotion_before': _emotionBefore,
      'emotion_after': _emotionAfter,
      'notes': _notes.text.trim().isEmpty ? null : _notes.text.trim(),
      'tags': <String>[],
    };
    try {
      final dio = ref.read(dioProvider);
      await dio.post('/journal/trades', data: body);
      ref.invalidate(journalTradesProvider);
      ref.invalidate(journalAnalyticsProvider);
      if (mounted) context.pop();
    } catch (e) {
      // Offline queue for later sync
      try {
        final cache = ref.read(offlineCacheProvider);
        await cache.enqueueWrite('/journal/trades', 'POST', body);
        if (mounted) {
          ScaffoldMessenger.of(context).showSnackBar(
            const SnackBar(
              content: Text('Saved offline — will sync when connected'),
            ),
          );
          context.pop();
        }
      } catch (_) {
        if (mounted) {
          ScaffoldMessenger.of(context)
              .showSnackBar(SnackBar(content: Text('$e')));
        }
      }
    } finally {
      if (mounted) setState(() => _saving = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('New trade')),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          TextField(
            controller: _symbol,
            decoration: const InputDecoration(labelText: 'Symbol'),
            textCapitalization: TextCapitalization.characters,
          ),
          const SizedBox(height: 12),
          DropdownButtonFormField<String>(
            initialValue: _side,
            items: const [
              DropdownMenuItem(value: 'long', child: Text('Long')),
              DropdownMenuItem(value: 'short', child: Text('Short')),
            ],
            onChanged: (v) => setState(() => _side = v ?? 'long'),
            decoration: const InputDecoration(labelText: 'Side'),
          ),
          const SizedBox(height: 12),
          Row(
            children: [
              Expanded(
                child: TextField(
                  controller: _entry,
                  keyboardType: TextInputType.number,
                  decoration: const InputDecoration(labelText: 'Entry'),
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: TextField(
                  controller: _exit,
                  keyboardType: TextInputType.number,
                  decoration: const InputDecoration(labelText: 'Exit (optional)'),
                ),
              ),
            ],
          ),
          const SizedBox(height: 12),
          TextField(
            controller: _qty,
            keyboardType: TextInputType.number,
            decoration: const InputDecoration(labelText: 'Quantity'),
          ),
          const SizedBox(height: 12),
          TextField(
            controller: _strategy,
            decoration: const InputDecoration(labelText: 'Strategy name'),
          ),
          const SizedBox(height: 12),
          TextField(
            controller: _rr,
            keyboardType: TextInputType.number,
            decoration: const InputDecoration(labelText: 'Risk : Reward'),
          ),
          const SizedBox(height: 12),
          DropdownButtonFormField<String>(
            initialValue: _emotionBefore,
            items: _emotions
                .map((e) => DropdownMenuItem(value: e, child: Text(e)))
                .toList(),
            onChanged: (v) => setState(() => _emotionBefore = v),
            decoration: const InputDecoration(labelText: 'Emotion before'),
          ),
          const SizedBox(height: 12),
          DropdownButtonFormField<String>(
            initialValue: _emotionAfter,
            items: _emotions
                .map((e) => DropdownMenuItem(value: e, child: Text(e)))
                .toList(),
            onChanged: (v) => setState(() => _emotionAfter = v),
            decoration: const InputDecoration(labelText: 'Emotion after'),
          ),
          const SizedBox(height: 12),
          TextField(
            controller: _notes,
            maxLines: 4,
            decoration: const InputDecoration(labelText: 'Notes'),
          ),
          const SizedBox(height: 20),
          FilledButton(
            onPressed: _saving ? null : _save,
            child: Text(_saving ? 'Saving…' : 'Save trade'),
          ),
        ],
      ),
    );
  }
}
