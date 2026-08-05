import 'dart:ui';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';

class GlassCard extends StatelessWidget {
  const GlassCard({
    super.key,
    required this.child,
    this.padding = const EdgeInsets.all(16),
    this.onTap,
    this.borderColor,
  });

  final Widget child;
  final EdgeInsets padding;
  final VoidCallback? onTap;
  final Color? borderColor;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final fill = isDark
        ? const Color(0xFF111827).withValues(alpha: kIsWeb ? 0.95 : 0.72)
        : Colors.white.withValues(alpha: kIsWeb ? 0.96 : 0.78);

    // BackdropFilter is expensive / flaky on Flutter web CanvasKit and can
    // make cards look blank or washed out when fonts/GPU path are stressed.
    final content = Material(
      color: fill,
      child: InkWell(
        onTap: onTap,
        child: Container(
          padding: padding,
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(20),
            border: Border.all(
              color: borderColor ?? scheme.outline.withValues(alpha: 0.12),
            ),
          ),
          child: child,
        ),
      ),
    );

    return ClipRRect(
      borderRadius: BorderRadius.circular(20),
      child: kIsWeb
          ? content
          : BackdropFilter(
              filter: ImageFilter.blur(sigmaX: 12, sigmaY: 12),
              child: content,
            ),
    );
  }
}
