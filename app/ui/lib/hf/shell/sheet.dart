// A sheet over the window: the dialog's dark box with a title and a close key, holding a body the caller builds.
// Esc, the close key or a press outside closes it.
import 'dart:async';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import '../glyphs.dart';
import 'place.dart';

/// A key of a dialog or sheet: the reference's raised box, or lit in the mode colour ([main]); [on] marks the chosen
/// one of a set; no [onTap] draws it quiet.
class HfKey extends StatelessWidget {
  const HfKey(this.label, {super.key, this.onTap, this.main = false, this.on = false});
  final String label;
  final VoidCallback? onTap;
  final bool main, on;
  @override
  Widget build(BuildContext context) {
    final lit = main || on;
    return MouseRegion(
      cursor: onTap == null ? SystemMouseCursors.basic : SystemMouseCursors.click,
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: onTap,
        child: Container(
          height: 30,
          margin: const EdgeInsets.only(left: 8),
          padding: const EdgeInsets.symmetric(horizontal: 14),
          alignment: Alignment.center,
          decoration: BoxDecoration(color: lit ? H.mode : H.raised, border: Border.all(color: lit ? H.mode : H.rule), borderRadius: BorderRadius.circular(3)),
          child: Text(label, softWrap: false, style: H.s(12, w: FontWeight.w600, color: lit ? const Color(0xFFFCFCFE) : (onTap == null ? H.text3 : H.text2))),
        ),
      ),
    );
  }
}

Future<void> showHfSheet(BuildContext context, {required String title, required Widget Function(BuildContext context, VoidCallback close) body, double width = 420}) {
  final done = Completer<void>();
  late final OverlayEntry entry;
  void close() {
    if (done.isCompleted) return;
    entry.remove();
    done.complete();
  }

  entry = OverlayEntry(
    builder: (context) => Focus(
      autofocus: true,
      onKeyEvent: (_, e) {
        if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape) {
          close();
          return KeyEventResult.handled;
        }
        return KeyEventResult.ignored;
      },
      child: Stack(children: [
        Positioned.fill(child: GestureDetector(behavior: HitTestBehavior.opaque, onTap: close, child: const ColoredBox(color: Color(0x88000000)))),
        Center(
          child: Container(
            width: width,
            decoration: BoxDecoration(color: const Color(0xFF1D1D1D), border: Border.all(color: const Color(0xFF3E3E3D)), borderRadius: BorderRadius.circular(3)),
            child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
              Container(
                height: 40,
                padding: const EdgeInsets.only(left: 18, right: 8),
                decoration: const BoxDecoration(border: Border(bottom: BorderSide(color: H.rule))),
                child: Row(children: [
                  Expanded(child: Text(title, style: H.s(14, w: FontWeight.w600, color: H.text))),
                  GestureDetector(
                    behavior: HitTestBehavior.opaque,
                    onTap: close,
                    child: SizedBox(width: 28, height: 28, child: Center(child: SizedBox(width: 12, height: 12, child: CustomPaint(painter: HgPainter(HG.cross, H.text2, const Color(0xFF1D1D1D)))))),
                  ),
                ]),
              ),
              body(context, close),
            ]),
          ),
        ),
      ]),
    ),
  );
  Overlay.of(context).insert(entry);
  return done.future;
}
