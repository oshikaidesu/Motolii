// A question over the window: the reference's dark box, a title, one line, and its answers as keys (the last is the
// default). Esc or a press outside answers null.
import 'dart:async';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'place.dart';

Future<T?> showHfDialog<T>(BuildContext context, {required String title, required String body, required List<(T, String)> answers}) {
  final done = Completer<T?>();
  late final OverlayEntry entry;
  void close(T? v) {
    entry.remove();
    if (!done.isCompleted) done.complete(v);
  }

  Widget key(T v, String label, bool main) => MouseRegion(
        cursor: SystemMouseCursors.click,
        child: GestureDetector(
          behavior: HitTestBehavior.opaque,
          onTap: () => close(v),
          child: Container(
            height: 30,
            margin: const EdgeInsets.only(left: 8),
            padding: const EdgeInsets.symmetric(horizontal: 14),
            alignment: Alignment.center,
            decoration: BoxDecoration(color: main ? H.mode : H.raised, border: Border.all(color: main ? H.mode : H.rule), borderRadius: BorderRadius.circular(3)),
            child: Text(label, softWrap: false, style: H.s(12, w: FontWeight.w600, color: main ? const Color(0xFFFCFCFE) : H.text2)),
          ),
        ),
      );

  entry = OverlayEntry(
    builder: (_) => Focus(
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
        Positioned.fill(child: GestureDetector(behavior: HitTestBehavior.opaque, onTap: () => close(null), child: const ColoredBox(color: Color(0x88000000)))),
        Center(
          child: Container(
            width: 380,
            padding: const EdgeInsets.fromLTRB(18, 16, 18, 16),
            decoration: BoxDecoration(color: const Color(0xFF1D1D1D), border: Border.all(color: const Color(0xFF3E3E3D)), borderRadius: BorderRadius.circular(3)),
            child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
              Text(title, style: H.s(14, w: FontWeight.w600, color: H.text)),
              const SizedBox(height: 8),
              Text(body, style: H.s(12, color: H.text2).copyWith(height: 1.4)),
              const SizedBox(height: 16),
              Wrap(alignment: WrapAlignment.end, runSpacing: 8, children: [
                for (final (i, (v, label)) in answers.indexed) key(v, label, i == answers.length - 1),
              ]),
            ]),
          ),
        ),
      ]),
    ),
  );
  Overlay.of(context).insert(entry);
  return done.future;
}
