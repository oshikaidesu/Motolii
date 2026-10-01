import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../../foundation/theme.dart';
import '../../foundation/metrics.dart';
import '../../session/editor_session.dart';
import '../../foundation/glyphs.dart';
import '../../foundation/leaves.dart';

/// A bordered action beside the search field.
Widget shelfAction(String label, VoidCallback? press) => Builder(
  builder: (context) => EditorTooltip(
    message: press == null ? '$label · unavailable' : label,
    child: EditorPress(
      onTap: press,
      child: Container(
        height: EditorMetrics.row,
        alignment: Alignment.center,
        padding: const EdgeInsets.symmetric(horizontal: EditorMetrics.s8),
        decoration: BoxDecoration(
          color: EditorTheme.of(context).app,
          border: Border.all(
            color: press == null
                ? EditorTheme.of(context).line
                : EditorTheme.of(context).border,
          ),
        ),
        child: Text(
          label,
          maxLines: 1,
          style: TextStyle(
            fontSize: EditorMetrics.font,
            color: press == null
                ? EditorTheme.of(context).disabledInk
                : EditorTheme.of(context).ink,
          ),
        ),
      ),
    ),
  ),
);

Widget shelfButton(
  String label,
  VoidCallback? press, {
  bool selected = false,
}) => Builder(
  builder: (context) => EditorTooltip(
    message: press == null ? '$label · unavailable' : label,
    child: EditorPress(
      onTap: press,
      child: Container(
        height: EditorMetrics.control,
        alignment: Alignment.centerLeft,
        padding: const EdgeInsets.symmetric(horizontal: EditorMetrics.s6),
        color: selected ? EditorTheme.of(context).raised : EditorTheme.clear,
        child: Text(
          label,
          maxLines: 1,
          overflow: TextOverflow.ellipsis,
          style: TextStyle(
            fontSize: EditorMetrics.font,
            color: press == null
                ? EditorTheme.of(context).muted.withValues(alpha: .45)
                : selected
                ? EditorTheme.of(context).accent
                : EditorTheme.of(context).ink,
          ),
        ),
      ),
    ),
  ),
);

/// Grid / List / Thumbnails, beside the search field.
Widget shelfViews(EditorSession controller, int viewMode) => Builder(
  builder: (context) => DecoratedBox(
    decoration: BoxDecoration(
      color: EditorTheme.of(context).app,
      border: Border.all(color: EditorTheme.of(context).line),
    ),
    child: Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        for (final (mode, icon, label) in [
          (0, Glyph.grid_view, 'Grid'),
          (1, Glyph.view_list, 'List'),
          (2, Glyph.crop_landscape, 'Thumbnails'),
        ])
          Container(
            width: EditorMetrics.control,
            height: EditorMetrics.row,
            color: viewMode == mode
                ? EditorTheme.of(context).raised
                : EditorTheme.clear,
            child: EditorTooltip(
              message: label,
              child: EditorIconButton(
                key: ValueKey('browser:view:$mode'),
                iconSize: EditorMetrics.s14,
                color: viewMode == mode
                    ? EditorTheme.of(context).ink
                    : EditorTheme.of(context).muted,
                onPressed: () => controller.storeDesk('browserView', mode),
                icon: Icon(icon),
              ),
            ),
          ),
      ],
    ),
  ),
);

/// A thin line you drag; it reports the movement along its axis.
Widget shelfGrip({
  required Key key,
  required void Function(double delta) onDrag,
  required VoidCallback onEnd,
  VoidCallback? onDoubleTap,
  bool vertical = false,
}) => Builder(
  builder: (context) => MouseRegion(
    cursor: vertical
        ? SystemMouseCursors.resizeUpDown
        : SystemMouseCursors.resizeLeftRight,
    child: GestureDetector(
      key: key,
      behavior: HitTestBehavior.opaque,
      onHorizontalDragUpdate: vertical ? null : (d) => onDrag(d.delta.dx),
      onHorizontalDragEnd: vertical ? null : (_) => onEnd(),
      onVerticalDragUpdate: vertical ? (d) => onDrag(d.delta.dy) : null,
      onVerticalDragEnd: vertical ? (_) => onEnd() : null,
      onDoubleTap: onDoubleTap,
      child: Container(
        width: vertical ? null : EditorMetrics.s4,
        height: vertical ? EditorMetrics.s4 : null,
        color: EditorTheme.of(context).line,
        alignment: Alignment.center,
        child: Container(
          width: vertical ? EditorMetrics.s16 : EditorMetrics.s2,
          height: vertical ? EditorMetrics.s2 : EditorMetrics.s16,
          color: EditorTheme.of(context).raised,
        ),
      ),
    ),
  ),
);

/// Tells its child whether the pointer rests on it.
class Hover extends StatefulWidget {
  const Hover({super.key, required this.builder});
  final Widget Function(bool hovered) builder;
  @override
  State<Hover> createState() => _HoverState();
}

class _HoverState extends State<Hover> {
  bool hovered = false;
  @override
  Widget build(BuildContext context) => MouseRegion(
    onEnter: (_) => setState(() => hovered = true),
    onExit: (_) => setState(() => hovered = false),
    child: widget.builder(hovered),
  );
}

/// A name at full size. One that does not fit is cut at the edge, and while
/// the pointer rests on it the name slides left until its end shows, then
/// slides back — so nothing is ever shrunk to fit.
class FittedName extends StatefulWidget {
  const FittedName(
    this.name,
    this.width, {
    super.key,
    this.size = EditorMetrics.font,
    this.sliding,
  });
  final String name;
  final double width;
  final double size;

  /// When the parent already knows the pointer is on the card, it drives the
  /// slide; otherwise the name watches for the pointer itself.
  final bool? sliding;

  /// The laid-out width of a name at a size. Names repeat across tiles and
  /// survive rebuilds, so lay each one out once.
  static final _natural = <String, double>{};
  static double widthOf(String name, double size) {
    if (_natural.length > 4096) _natural.clear();
    return _natural['$size:$name'] ??= () {
      final painter = TextPainter(
        text: TextSpan(
          text: name,
          style: TextStyle(fontFamily: EditorTheme.fontFamily, fontSize: size),
        ),
        maxLines: 1,
        textDirection: TextDirection.ltr,
      )..layout();
      final width = painter.width;
      painter.dispose();
      return width;
    }();
  }

  @override
  State<FittedName> createState() => _FittedNameState();
}

class _FittedNameState extends State<FittedName>
    with SingleTickerProviderStateMixin {
  late final _slide = AnimationController(vsync: this);

  double get _overflow =>
      math.max(0, FittedName.widthOf(widget.name, widget.size) - widget.width);

  @override
  void initState() {
    super.initState();
    if (widget.sliding == true) _enter();
  }

  @override
  void didUpdateWidget(FittedName old) {
    super.didUpdateWidget(old);
    if (widget.sliding != old.sliding) {
      if (widget.sliding == true) _enter();
      if (widget.sliding == false) _leave();
    }
  }

  @override
  void dispose() {
    _slide.dispose();
    super.dispose();
  }

  void _enter() {
    if (_overflow <= 0) return;
    // A brisk reading pace: ninety pixels a second, at least a third of one.
    _slide.duration = Duration(
      milliseconds: math.max(300, (_overflow / 90 * 1000).round()),
    );
    _slide.forward();
  }

  /// Leaving snaps the name back; only the reading is animated.
  void _leave() => _slide.value = 0;

  @override
  Widget build(BuildContext context) {
    final overflow = _overflow;
    final text = Text(
      widget.name,
      maxLines: 1,
      softWrap: false,
      overflow: TextOverflow.visible,
      style: TextStyle(
        color: EditorTheme.of(context).ink,
        fontSize: widget.size,
      ),
    );
    if (overflow <= 0) return text;
    return MouseRegion(
      onEnter: widget.sliding == null ? (_) => _enter() : null,
      onExit: widget.sliding == null ? (_) => _leave() : null,
      child: ClipRect(
        child: AnimatedBuilder(
          animation: _slide,
          // At rest the name says it is longer than its room; sliding reads it.
          builder: (context, child) =>
              _slide.value == 0 && widget.sliding != true
              ? SizedBox(
                  width: widget.width,
                  child: Text(
                    widget.name,
                    maxLines: 1,
                    softWrap: false,
                    overflow: TextOverflow.ellipsis,
                    style: TextStyle(
                      color: EditorTheme.of(context).ink,
                      fontSize: widget.size,
                    ),
                  ),
                )
              : Transform.translate(
                  offset: Offset(
                    -overflow * Curves.easeInOut.transform(_slide.value),
                    0,
                  ),
                  child: child,
                ),
          child: OverflowBox(
            alignment: Alignment.centerLeft,
            maxWidth: double.infinity,
            child: text,
          ),
        ),
      ),
    );
  }
}
