import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../core/constants/app_constants.dart';
import '../../../core/network/api_client.dart';
import '../../../core/widgets/glass_card.dart';

class _ChatMsg {
  _ChatMsg({required this.role, required this.content});
  final String role;
  final String content;
}

class AiChatScreen extends ConsumerStatefulWidget {
  const AiChatScreen({super.key});

  @override
  ConsumerState<AiChatScreen> createState() => _AiChatScreenState();
}

class _AiChatScreenState extends ConsumerState<AiChatScreen> {
  final _controller = TextEditingController();
  final _scroll = ScrollController();
  final _messages = <_ChatMsg>[];
  String? _conversationId;
  bool _sending = false;
  bool _showDisclaimer = true;

  final _suggestions = const [
    'Summarize open IPOs',
    'Review my portfolio risk',
    'Find mistakes in my trades',
    'Explain how XIRR works',
  ];

  @override
  void dispose() {
    _controller.dispose();
    _scroll.dispose();
    super.dispose();
  }

  Future<void> _send([String? preset]) async {
    final text = (preset ?? _controller.text).trim();
    if (text.isEmpty || _sending) return;
    setState(() {
      _sending = true;
      _messages.add(_ChatMsg(role: 'user', content: text));
      _controller.clear();
    });
    _scrollToEnd();

    try {
      final dio = ref.read(dioProvider);
      final res = await dio.post('/ai/chat', data: {
        'conversation_id': _conversationId,
        'message': text,
      });
      final data = unwrapData(res, (d) => Map<String, dynamic>.from(d as Map));
      setState(() {
        _conversationId = data['conversation_id']?.toString();
        _messages.add(_ChatMsg(
          role: 'assistant',
          content: data['reply']?.toString() ?? '',
        ));
      });
      _scrollToEnd();
    } catch (e) {
      setState(() {
        _messages.add(_ChatMsg(role: 'assistant', content: 'Error: $e'));
      });
    } finally {
      setState(() => _sending = false);
    }
  }

  void _scrollToEnd() {
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (_scroll.hasClients) {
        _scroll.animateTo(
          _scroll.position.maxScrollExtent + 80,
          duration: const Duration(milliseconds: 250),
          curve: Curves.easeOut,
        );
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;

    return Scaffold(
      appBar: AppBar(
        title: const Text('AI Assistant'),
        actions: [
          IconButton(
            icon: const Icon(Icons.info_outline),
            onPressed: () => setState(() => _showDisclaimer = !_showDisclaimer),
          ),
        ],
      ),
      body: Column(
        children: [
          if (_showDisclaimer)
            Material(
              color: scheme.errorContainer.withValues(alpha: 0.35),
              child: Padding(
                padding: const EdgeInsets.all(12),
                child: Row(
                  children: [
                    Icon(Icons.shield_outlined, color: scheme.error, size: 18),
                    const SizedBox(width: 8),
                    Expanded(
                      child: Text(
                        AppConstants.investmentDisclaimer,
                        style: Theme.of(context).textTheme.labelSmall,
                      ),
                    ),
                    IconButton(
                      icon: const Icon(Icons.close, size: 18),
                      onPressed: () => setState(() => _showDisclaimer = false),
                    ),
                  ],
                ),
              ),
            ),
          if (_messages.isEmpty)
            Padding(
              padding: const EdgeInsets.all(16),
              child: Wrap(
                spacing: 8,
                runSpacing: 8,
                children: _suggestions
                    .map(
                      (s) => ActionChip(
                        label: Text(s),
                        onPressed: () => _send(s),
                      ),
                    )
                    .toList(),
              ),
            ),
          Expanded(
            child: ListView.builder(
              controller: _scroll,
              padding: const EdgeInsets.all(16),
              itemCount: _messages.length,
              itemBuilder: (context, i) {
                final m = _messages[i];
                final isUser = m.role == 'user';
                return Align(
                  alignment:
                      isUser ? Alignment.centerRight : Alignment.centerLeft,
                  child: Container(
                    margin: const EdgeInsets.only(bottom: 10),
                    constraints: BoxConstraints(
                      maxWidth: MediaQuery.of(context).size.width * 0.85,
                    ),
                    child: GlassCard(
                      padding: const EdgeInsets.all(12),
                      borderColor: isUser
                          ? scheme.primary.withValues(alpha: 0.35)
                          : null,
                      child: Text(m.content),
                    ),
                  ),
                );
              },
            ),
          ),
          SafeArea(
            child: Padding(
              padding: const EdgeInsets.fromLTRB(12, 0, 12, 12),
              child: Row(
                children: [
                  Expanded(
                    child: TextField(
                      controller: _controller,
                      minLines: 1,
                      maxLines: 4,
                      decoration: const InputDecoration(
                        hintText: 'Ask about IPOs, portfolio, trades…',
                      ),
                      onSubmitted: (_) => _send(),
                    ),
                  ),
                  const SizedBox(width: 8),
                  IconButton.filled(
                    onPressed: _sending ? null : () => _send(),
                    icon: _sending
                        ? const SizedBox(
                            width: 18,
                            height: 18,
                            child: CircularProgressIndicator(strokeWidth: 2),
                          )
                        : const Icon(Icons.send_rounded),
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}
