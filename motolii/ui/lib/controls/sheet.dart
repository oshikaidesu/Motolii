// A sheet over the window: the dialog's dark box with a title and a close key, holding a body the caller builds.
// Esc, the close key or a press outside closes it.
import 'dart:async';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import '../theme/glyphs.dart';
import '../theme/metrics.dart';
import '../theme/identity.dart';
import '../theme/neutral.dart';

enum HfActionKind { primary, secondary, destructive }

/// An action: the one primary of a surface (filled in the mode colour), a secondary (a raised key), or a destructive
/// one (its words in the record red). A pointer over it lifts it; no [onTap] draws it quiet. Visual height is
/// [Surface.control]; the key never shrinks below [Surface.hit] to the pointer.
class HfAction extends StatefulWidget {
  const HfAction(this.label, {super.key, this.onTap, this.kind = HfActionKind.secondary, this.chosen = false});
  final String label;
  final VoidCallback? onTap;
  final HfActionKind kind;
  final bool chosen;
  @override
  State<HfAction> createState() => _HfActionState();
}

class _HfActionState extends State<HfAction> {
  bool _over = false;
  @override
  Widget build(BuildContext context) {
    final live = widget.onTap != null;
    final primary = widget.kind == HfActionKind.primary && live;
    final ground = primary
        ? (_over ? Color.lerp(H.mode, N.g100, .1)! : H.mode)
        : (widget.chosen ? N.g26 : (_over && live ? Surface.hover : Surface.raised));
    final edge = primary ? H.mode : (widget.chosen ? N.g44 : Surface.divider);
    final ink = !live
        ? N.g63
        : primary
            ? N.g100
            : widget.kind == HfActionKind.destructive
                ? H.record
                : N.g82;
    return MouseRegion(
      cursor: live ? SystemMouseCursors.click : SystemMouseCursors.basic,
      onEnter: (_) {
        if (mounted) setState(() => _over = true);
      },
      onExit: (_) {
        if (mounted) setState(() => _over = false);
      },
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: widget.onTap,
        child: Container(
          height: Surface.control,
          constraints: const BoxConstraints(minWidth: 42),
          margin: const EdgeInsets.only(left: 6),
          padding: const EdgeInsets.symmetric(horizontal: 9),
          alignment: Alignment.center,
          decoration: BoxDecoration(color: ground, border: Border.all(color: edge), borderRadius: BorderRadius.circular(2)),
          child: Text(widget.label, softWrap: false, style: H.s(Dn.nameSize, w: primary ? FontWeight.w600 : FontWeight.w500, color: ink)),
        ),
      ),
    );
  }
}

/// One of a few mutually exclusive options, as one segmented control: the chosen segment is a lighter ground inside
/// the track (a selection, not an action). A segment with no [enabled] is quiet and cannot be chosen.
class HfChoice<T> extends StatelessWidget {
  const HfChoice({super.key, required this.options, required this.value, required this.onChanged});
  final List<(T value, String label, bool enabled)> options;
  final T value;
  final ValueChanged<T>? onChanged;
  @override
  Widget build(BuildContext context) => Container(
        height: Surface.control,
        padding: const EdgeInsets.all(1.5),
        decoration: BoxDecoration(color: N.g07, border: Border.all(color: Surface.divider), borderRadius: BorderRadius.circular(Surface.controlRadius)),
        child: Row(mainAxisSize: MainAxisSize.min, children: [
          for (final (v, label, enabled) in options)
            _Segment(label, on: v == value, onTap: enabled && onChanged != null && v != value ? () => onChanged!(v) : null, enabled: enabled),
        ]),
      );
}

class _Segment extends StatefulWidget {
  const _Segment(this.label, {required this.on, required this.onTap, required this.enabled});
  final String label;
  final bool on, enabled;
  final VoidCallback? onTap;
  @override
  State<_Segment> createState() => _SegmentState();
}

class _SegmentState extends State<_Segment> {
  bool _over = false;
  @override
  Widget build(BuildContext context) => MouseRegion(
        cursor: widget.onTap == null ? SystemMouseCursors.basic : SystemMouseCursors.click,
        onEnter: (_) {
        if (mounted) setState(() => _over = true);
      },
        onExit: (_) {
        if (mounted) setState(() => _over = false);
      },
        child: GestureDetector(
          behavior: HitTestBehavior.opaque,
          onTap: widget.onTap,
          child: Container(
            padding: const EdgeInsets.symmetric(horizontal: 9),
            alignment: Alignment.center,
            decoration: BoxDecoration(color: widget.on ? N.g26 : (_over && widget.onTap != null ? Surface.raised : null), borderRadius: BorderRadius.circular(2)),
            child: Text(widget.label, softWrap: false, style: H.s(Dn.nameSize, w: widget.on ? FontWeight.w600 : FontWeight.w400, color: !widget.enabled ? N.g44 : (widget.on ? Surface.ink : N.g82))),
          ),
        ),
      );
}

/// A form row: the label on the left in one column, what it labels on the right (a fact, a choice, a field).
class HfFormRow extends StatelessWidget {
  const HfFormRow(this.label, this.child, {super.key, this.labelWidth = 64});
  final String label;
  final Widget child;
  final double labelWidth;
  @override
  Widget build(BuildContext context) => Padding(
        padding: EdgeInsets.only(bottom: Surface.sectionGap),
        child: Row(crossAxisAlignment: CrossAxisAlignment.center, children: [
          SizedBox(width: labelWidth, child: Text(label, softWrap: false, style: H.s(Dn.nameSize, color: N.g63))),
          Flexible(child: Align(alignment: Alignment.centerLeft, child: child)),
        ]),
      );
}

/// A read-only value: the same monospace as the readouts, no box (a box would say "edit me").
class HfFact extends StatelessWidget {
  const HfFact(this.value, {super.key, this.color = N.g82});
  final String value;
  final Color color;
  @override
  Widget build(BuildContext context) => Text(value, softWrap: false, overflow: TextOverflow.ellipsis, style: H.m(11, color: color));
}

/// A compact housing for a short task: anchored under the control that asked for it (its right edge on the
/// control's), no dimmed backdrop and nothing over the Stage's middle; a press outside or Esc closes it, Enter runs its
/// [primary] action when there is one.
Future<void> showHfPopover(BuildContext context, {required Rect anchor, required String title, required Widget Function(BuildContext context, VoidCallback close) body, double width = 300, VoidCallback? Function()? primary}) {
  final done = Completer<void>();
  final overlay = Overlay.of(context);
  final box = overlay.context.findRenderObject() as RenderBox?;
  final a = box == null ? anchor : Rect.fromPoints(box.globalToLocal(anchor.topLeft), box.globalToLocal(anchor.bottomRight));
  late final OverlayEntry entry;
  // its own focus, taken when it opens and on any press inside it, so Esc and Enter are its keys until it closes
  final focus = FocusNode(debugLabel: 'hf popover');
  void close() {
    if (done.isCompleted) return;
    entry.remove();
    focus.dispose();
    done.complete();
  }

  entry = OverlayEntry(
    builder: (context) => LayoutBuilder(builder: (context, space) {
      final left = (a.right - width).clamp(8.0, (space.maxWidth - width - 8).clamp(8.0, double.infinity));
      return Focus(
        focusNode: focus,
        autofocus: true,
        onKeyEvent: (_, e) {
          if (e is! KeyDownEvent) return KeyEventResult.ignored;
          if (e.logicalKey == LogicalKeyboardKey.escape) {
            close();
            return KeyEventResult.handled;
          }
          if (e.logicalKey == LogicalKeyboardKey.enter || e.logicalKey == LogicalKeyboardKey.numpadEnter) {
            final run = primary?.call();
            if (run != null) {
              run();
              return KeyEventResult.handled;
            }
          }
          return KeyEventResult.ignored;
        },
        child: Stack(children: [
          Positioned.fill(child: GestureDetector(behavior: HitTestBehavior.translucent, onTap: close)),
          Positioned(
            left: left,
            top: a.bottom + 6,
            width: width,
            child: Listener(
              onPointerDown: (_) => focus.requestFocus(),
              child: Container(
              key: const ValueKey('hf-popover'),
              decoration: BoxDecoration(
                color: N.g13,
                border: Border.all(color: N.g26),
                borderRadius: BorderRadius.circular(Surface.controlRadius),
                boxShadow: const [BoxShadow(color: N.shade40, blurRadius: 16, offset: Offset(0, 6))],
              ),
              child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
                Container(
                  height: Surface.chromeRow,
                  padding: const EdgeInsets.only(left: 9, right: 3),
                  decoration: const BoxDecoration(border: Border(bottom: BorderSide(color: Surface.dividerFine))),
                  child: Row(children: [
                    Expanded(child: Text(title, style: H.s(Dn.nameSize, w: FontWeight.w600, color: Surface.ink))),
                    GestureDetector(
                      behavior: HitTestBehavior.opaque,
                      onTap: close,
                      child: MouseRegion(cursor: SystemMouseCursors.click, child: SizedBox(width: Surface.hit, height: Surface.hit, child: Center(child: SizedBox(width: 7.5, height: 7.5, child: CustomPaint(painter: HgPainter(HG.cross, N.g63, N.g13)))))),
                    ),
                  ]),
                ),
                body(context, close),
              ]),
            ),
            ),
          ),
        ]),
      );
    }),
  );
  overlay.insert(entry);
  WidgetsBinding.instance.addPostFrameCallback((_) {
    if (!done.isCompleted) focus.requestFocus();
  });
  return done.future;
}
