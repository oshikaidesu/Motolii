// A short list that drops from a bar box (the Stage's view box): the reference's dark box, one line per choice.
import 'dart:async';
import 'package:flutter/widgets.dart';
import 'place.dart';

Future<T?> showHfMenu<T>(BuildContext context, Rect at, List<(T, String)> items, {T? selected, Set<T> disabled = const {}}) {
  final done = Completer<T?>();
  late final OverlayEntry entry;
  void close(T? v) {
    entry.remove();
    if (!done.isCompleted) done.complete(v);
  }

  entry = OverlayEntry(
    builder: (_) => Stack(children: [
      Positioned.fill(child: GestureDetector(behavior: HitTestBehavior.opaque, onTap: () => close(null))),
      Positioned(
        left: at.left,
        top: at.bottom + 2,
        width: at.width,
        child: Container(
          padding: const EdgeInsets.symmetric(vertical: 3),
          decoration: BoxDecoration(color: const Color(0xFF1D1D1D), border: Border.all(color: const Color(0xFF3E3E3D)), borderRadius: BorderRadius.circular(3)),
          child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            for (final (v, label) in items)
              MouseRegion(
                cursor: disabled.contains(v) ? SystemMouseCursors.basic : SystemMouseCursors.click,
                child: GestureDetector(
                  behavior: HitTestBehavior.opaque,
                  onTap: disabled.contains(v) ? null : () => close(v),
                  child: Container(
                    height: 28,
                    padding: const EdgeInsets.symmetric(horizontal: 10),
                    alignment: Alignment.centerLeft,
                    color: v == selected ? H.selHi : null,
                    child: Text(label, softWrap: false, style: H.s(13, color: disabled.contains(v) ? H.text3 : H.text)),
                  ),
                ),
              ),
          ]),
        ),
      ),
    ]),
  );
  Overlay.of(context).insert(entry);
  return done.future;
}
