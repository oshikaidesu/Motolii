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

  // Kept inside the window: it opens upward from the box when there is no room below, and slides left at the edge.
  final overlay = Overlay.of(context);
  final box = overlay.context.findRenderObject() as RenderBox?;
  final space = box?.size ?? Size.infinite;
  final origin = box == null ? Offset.zero : box.globalToLocal(Offset.zero);
  final local = at.shift(origin);
  final height = items.length * 28.0 + 8;
  final top = local.bottom + 2 + height <= space.height
      ? local.bottom + 2
      : (local.top - 2 - height).clamp(0.0, double.infinity);
  final left = local.left.clamp(0.0, (space.width - at.width).clamp(0.0, double.infinity));
  entry = OverlayEntry(
    builder: (_) => Stack(children: [
      Positioned.fill(child: GestureDetector(behavior: HitTestBehavior.opaque, onTap: () => close(null))),
      Positioned(
        left: left,
        top: top,
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
  overlay.insert(entry);
  return done.future;
}
