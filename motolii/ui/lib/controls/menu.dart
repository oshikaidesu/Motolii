// A short list that drops from a bar box (the Stage's view box): the reference's dark box, one line per choice.
import 'dart:async';
import 'dart:math' as math;
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import '../theme/metrics.dart';
import '../theme/tokens.dart';
import '../theme/neutral.dart';

/// [shortcuts] names the key for a line (shown at its right, as menus do); [dividers] ends a group after a line.
/// [info] lines tell rather than do (a title, a fact): drawn as information, never lit, never greyed like an action
/// that cannot run now ([disabled]). The line under the pointer is lit as the keys light it; [selected] (the current
/// value) carries a check, so "current" and "under the pointer" never look alike.
Future<T?> showHfMenu<T>(BuildContext context, Rect at, List<(T, String)> items, {T? selected, Set<T> disabled = const {}, Map<T, String> shortcuts = const {}, Set<T> dividers = const {}, Set<T> info = const {}}) {
  final done = Completer<T?>();
  late final OverlayEntry entry;
  final back = FocusManager.instance.primaryFocus;
  final keys = FocusNode(debugLabel: 'hf menu');
  final hot = ValueNotifier(-1); // the line the keys walked to
  void close(T? v) {
    if (done.isCompleted) return;
    entry.remove();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      keys.dispose();
      hot.dispose();
    });
    done.complete(v);
    // the keys go back where they were before the menu took them
    if (back != null && back.context != null) back.requestFocus();
  }

  // Kept inside the window: it opens upward from the box when there is no room below, and slides left at the edge.
  final overlay = Overlay.of(context);
  final box = overlay.context.findRenderObject() as RenderBox?;
  final space = box?.size ?? Size.infinite;
  final origin = box == null ? Offset.zero : box.globalToLocal(Offset.zero);
  final local = at.shift(origin);
  final full = items.length * Surface.menuRow + 8 + 9.0 * items.where((i) => dividers.contains(i.$1) && i != items.last).length;
  // a menu taller than the window scrolls inside it
  final height = math.min(full, math.max(40.0, space.height - 8));
  final top = local.bottom + 2 + height <= space.height
      ? local.bottom + 2
      : (local.top - 2 - height).clamp(0.0, double.infinity);
  // a menu is never narrower than a line can be read in (a caller may only know a point)
  final width = math.max(at.width, 180.0);
  final left = local.left.clamp(0.0, (space.width - width).clamp(0.0, double.infinity));
  hot.addListener(() {
    if (!done.isCompleted) entry.markNeedsBuild();
  });
  entry = OverlayEntry(
    // Escape, or a press of either button outside, closes it without a choice (as the editor menu does).
    builder: (_) => Focus(
      focusNode: keys,
      onKeyEvent: (_, e) {
        if (e is! KeyDownEvent && e is! KeyRepeatEvent) return KeyEventResult.ignored;
        final k = e.logicalKey;
        if (k == LogicalKeyboardKey.escape) {
          close(null);
          return KeyEventResult.handled;
        }
        // ↑/↓ walk the lines that can be chosen, Enter takes the one walked to (the editor menu's keys)
        if (k == LogicalKeyboardKey.arrowDown || k == LogicalKeyboardKey.arrowUp) {
          final open = [for (final (i, it) in items.indexed) if (!disabled.contains(it.$1) && !info.contains(it.$1)) i];
          if (open.isEmpty) return KeyEventResult.handled;
          final at = open.indexOf(hot.value);
          hot.value = k == LogicalKeyboardKey.arrowDown
              ? open[at < 0 ? 0 : (at + 1) % open.length]
              : open[at < 0 ? open.length - 1 : (at - 1 + open.length) % open.length];
          return KeyEventResult.handled;
        }
        if ((k == LogicalKeyboardKey.enter || k == LogicalKeyboardKey.numpadEnter) && hot.value >= 0) {
          close(items[hot.value].$1);
          return KeyEventResult.handled;
        }
        return KeyEventResult.ignored;
      },
      child: Stack(children: [
      Positioned.fill(child: GestureDetector(behavior: HitTestBehavior.opaque, onTap: () => close(null), onSecondaryTap: () => close(null))),
      Positioned(
        left: left,
        top: top,
        width: width,
        child: Container(
          padding: const EdgeInsets.symmetric(vertical: 2),
          decoration: BoxDecoration(color: N.g13, border: Border.all(color: N.g26), borderRadius: BorderRadius.circular(2)),
          constraints: BoxConstraints(maxHeight: height),
          child: SingleChildScrollView(child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            for (final (i, (v, label)) in items.indexed) ...[
              if (info.contains(v))
                Container(
                  height: Surface.menuRow - 4,
                  padding: const EdgeInsets.symmetric(horizontal: 7.5),
                  alignment: Alignment.centerLeft,
                  child: Text(label, softWrap: false, overflow: TextOverflow.ellipsis, style: i == 0 ? H.s(Dn.nameSize, w: FontWeight.w600, color: N.g82) : H.m(11, color: N.g63)),
                )
              else
              MouseRegion(
                cursor: disabled.contains(v) ? SystemMouseCursors.basic : SystemMouseCursors.click,
                onEnter: (_) {
                  if (!disabled.contains(v) && !done.isCompleted) hot.value = i;
                },
                onExit: (_) {
                  if (!done.isCompleted && hot.value == i) hot.value = -1;
                },
                child: GestureDetector(
                  behavior: HitTestBehavior.opaque,
                  onTap: disabled.contains(v) ? null : () => close(v),
                  child: Container(
                    height: Surface.menuRow,
                    padding: const EdgeInsets.only(left: 3, right: 7.5),
                    alignment: Alignment.centerLeft,
                    color: hot.value == i ? N.g26 : null,
                    child: Row(children: [
                      SizedBox(width: 12, child: v == selected ? Text('✓', textAlign: TextAlign.center, style: H.s(Dn.nameSize, color: N.g82)) : null),
                      Expanded(child: Text(label, softWrap: false, overflow: TextOverflow.ellipsis, style: H.s(Dn.nameSize, color: disabled.contains(v) ? N.g44 : Surface.ink))),
                      if (shortcuts[v] case final key?) Padding(padding: const EdgeInsets.only(left: 12), child: Text(key, style: H.s(Dn.nameSize, color: N.g63))),
                    ]),
                  ),
                ),
              ),
              if (dividers.contains(v) && v != items.last.$1) Container(height: 1, margin: const EdgeInsets.symmetric(vertical: 3), color: Surface.divider),
            ],
          ])),
        ),
      ),
    ]),
    ),
  );
  overlay.insert(entry);
  WidgetsBinding.instance.addPostFrameCallback((_) {
    if (!done.isCompleted) keys.requestFocus();
  });
  return done.future;
}
