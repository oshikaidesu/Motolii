// A question over the window: the reference's dark box, a title, one line, and its answers as keys (the last is the
// default). Esc or a press outside answers null.
import 'dart:async';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'place.dart';
import 'sheet.dart' show HfAction, HfActionKind;
import '../neutral.dart';

/// [destructive] answers (discarding work) are drawn in the record red and set apart on the left; the last answer is
/// the primary (and Enter's).
Future<T?> showHfDialog<T>(BuildContext context, {required String title, required String body, required List<(T, String)> answers, Set<T> destructive = const {}}) {
  final done = Completer<T?>();
  late final OverlayEntry entry;
  // its own focus, taken when it opens: Enter and Esc are the question's until it is answered
  final focus = FocusNode(debugLabel: 'hf dialog');
  void close(T? v) {
    if (done.isCompleted) return;
    entry.remove();
    focus.dispose();
    done.complete(v);
  }

  Widget key(T v, String label, bool main) => HfAction(label, onTap: () => close(v), kind: main ? HfActionKind.primary : (destructive.contains(v) ? HfActionKind.destructive : HfActionKind.secondary));

  entry = OverlayEntry(
    builder: (_) => Focus(
      focusNode: focus,
      autofocus: true,
      onKeyEvent: (_, e) {
        if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape) {
          close(null);
          return KeyEventResult.handled;
        }
        if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.enter) {
          close(answers.last.$1);
          return KeyEventResult.handled;
        }
        return KeyEventResult.ignored;
      },
      child: Stack(children: [
        Positioned.fill(child: GestureDetector(behavior: HitTestBehavior.opaque, onTap: () => close(null), child: const ColoredBox(color: N.shade55))),
        Center(
          child: Container(
            width: 380,
            padding: const EdgeInsets.fromLTRB(16, 14, 16, 14),
            decoration: BoxDecoration(color: N.g13, border: Border.all(color: N.g26), borderRadius: BorderRadius.circular(3)),
            child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
              Text(title, style: H.s(13, w: FontWeight.w600, color: H.text)),
              const SizedBox(height: 8),
              Text(body, style: H.s(12, color: H.text2).copyWith(height: 1.4)),
              const SizedBox(height: 16),
              Row(children: [
                for (final (v, label) in answers)
                  if (destructive.contains(v)) key(v, label, false),
                const Spacer(),
                for (final (i, (v, label)) in answers.indexed)
                  if (!destructive.contains(v)) key(v, label, i == answers.length - 1),
              ]),
            ]),
          ),
        ),
      ]),
    ),
  );
  Overlay.of(context).insert(entry);
  WidgetsBinding.instance.addPostFrameCallback((_) {
    if (!done.isCompleted) focus.requestFocus();
  });
  return done.future;
}
