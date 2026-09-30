import 'dart:async';

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../../../controls/leaves.dart';
import '../../../theme/editor_metrics.dart';
import '../../../theme/editor_theme.dart';

/// A one-line question asked where the pointer is: Enter keeps the answer, Escape or leaving drops it.
Future<String?> promptText(BuildContext context, Offset at, String title) {
  final overlay = Overlay.of(context);
  final answer = Completer<String?>();
  late OverlayEntry entry;
  void done(String? value) {
    if (answer.isCompleted) return;
    entry.remove();
    answer.complete(value == null || value.trim().isEmpty ? null : value.trim());
  }

  entry = OverlayEntry(
    builder: (context) => Positioned(
      left: at.dx,
      top: at.dy,
      width: 200,
      child: TapRegion(
        onTapOutside: (_) => done(null),
        child: Focus(
          onKeyEvent: (_, e) {
            if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape) {
              done(null);
              return KeyEventResult.handled;
            }
            return KeyEventResult.ignored;
          },
          child: Container(
            padding: const EdgeInsets.all(EditorMetrics.s6),
            decoration: BoxDecoration(color: EditorTheme.of(context).panel, border: Border.all(color: EditorTheme.of(context).border)),
            child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
              Text(title, style: TextStyle(fontSize: EditorMetrics.dense, color: EditorTheme.of(context).muted)),
              const SizedBox(height: EditorMetrics.s4),
              SizedBox(
                height: EditorMetrics.control,
                child: EditorTextField(key: const ValueKey('browser:prompt'), autofocus: true, onSubmitted: done),
              ),
            ]),
          ),
        ),
      ),
    ),
  );
  overlay.insert(entry);
  return answer.future;
}
