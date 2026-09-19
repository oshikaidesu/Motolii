import 'dart:async';

import 'package:flutter/widgets.dart';

import 'leaves.dart';
import 'metrics.dart';

/// Colours of what is drawn rather than laid out — lanes, grids, ticks,
/// gizmos, the transparency grid, the ease desk's paper — carried by
/// [EditorLook] above the window so a painter's colours come from the theme
/// the way a widget's do. A widget passes [of] into the painter it builds;
/// code with no context reads [dark], the one look the window has today.
@immutable
class EditorInk {
  const EditorInk({
    required this.laneGround,
    required this.lane,
    required this.laneAlt,
    required this.grid,
    required this.gridMinor,
    required this.tick,
    required this.tickMinor,
    required this.headerInk,
    required this.camera,
    required this.focusRing,
    required this.easePaper,
    required this.easeInk,
    required this.easeTime,
    required this.checkerLight,
    required this.checkerDark,
    required this.collectionColors,
  });

  /// The timeline: the ground under the lanes, the two lane greys, the grid
  /// lines and the ruler's ticks, and the ink on an identity-coloured row.
  final Color laneGround, lane, laneAlt, grid, gridMinor, tick, tickMinor;
  final Color headerInk;

  /// A camera's line on the Stage and in the Depth desk.
  final Color camera;

  /// The ring on the pane that holds keyboard focus.
  final Color focusRing;

  /// The ease desk: its paper, the ink on it, and the time axis.
  final Color easePaper, easeInk, easeTime;

  /// The transparency grid: the picture editors' two greys.
  final Color checkerLight, checkerDark;

  /// The Browser's collections, one hue each.
  final List<Color> collectionColors;

  static const dark = EditorInk(
    laneGround: Color(0xff3c3c3c),
    lane: Color(0xff383838),
    laneAlt: Color(0xff3d3d3d),
    grid: Color(0xff262626),
    gridMinor: Color(0xff303030),
    tick: Color(0xff9b9b9b),
    tickMinor: Color(0xff929292),
    headerInk: Color(0xff202020),
    camera: Color(0xff8ed9e6),
    focusRing: Color(0xffacacac),
    easePaper: Color(0xffb7d8d1),
    easeInk: Color(0xff203f39),
    easeTime: Color(0xff854515),
    checkerLight: Color(0xff8c8c8c),
    checkerDark: Color(0xff666666),
    collectionColors: [
      Color(0xffe05252),
      Color(0xffe0a052),
      Color(0xffe0d452),
      Color(0xff6fd06f),
      Color(0xff52b9e0),
      Color(0xff8f7ae0),
      Color(0xff8a8a8a),
    ],
  );

  static EditorInk of(BuildContext context) => EditorLook.of(context).ink;

  EditorInk copyWith({
    Color? laneGround,
    Color? lane,
    Color? laneAlt,
    Color? grid,
    Color? gridMinor,
    Color? tick,
    Color? tickMinor,
    Color? headerInk,
    Color? camera,
    Color? focusRing,
    Color? easePaper,
    Color? easeInk,
    Color? easeTime,
    Color? checkerLight,
    Color? checkerDark,
    List<Color>? collectionColors,
  }) => EditorInk(
    laneGround: laneGround ?? this.laneGround,
    lane: lane ?? this.lane,
    laneAlt: laneAlt ?? this.laneAlt,
    grid: grid ?? this.grid,
    gridMinor: gridMinor ?? this.gridMinor,
    tick: tick ?? this.tick,
    tickMinor: tickMinor ?? this.tickMinor,
    headerInk: headerInk ?? this.headerInk,
    camera: camera ?? this.camera,
    focusRing: focusRing ?? this.focusRing,
    easePaper: easePaper ?? this.easePaper,
    easeInk: easeInk ?? this.easeInk,
    easeTime: easeTime ?? this.easeTime,
    checkerLight: checkerLight ?? this.checkerLight,
    checkerDark: checkerDark ?? this.checkerDark,
    collectionColors: collectionColors ?? this.collectionColors,
  );

  EditorInk lerp(EditorInk? other, double t) {
    if (other == null) return this;
    Color mix(Color a, Color b) => Color.lerp(a, b, t)!;
    return EditorInk(
      laneGround: mix(laneGround, other.laneGround),
      lane: mix(lane, other.lane),
      laneAlt: mix(laneAlt, other.laneAlt),
      grid: mix(grid, other.grid),
      gridMinor: mix(gridMinor, other.gridMinor),
      tick: mix(tick, other.tick),
      tickMinor: mix(tickMinor, other.tickMinor),
      headerInk: mix(headerInk, other.headerInk),
      camera: mix(camera, other.camera),
      focusRing: mix(focusRing, other.focusRing),
      easePaper: mix(easePaper, other.easePaper),
      easeInk: mix(easeInk, other.easeInk),
      easeTime: mix(easeTime, other.easeTime),
      checkerLight: mix(checkerLight, other.checkerLight),
      checkerDark: mix(checkerDark, other.checkerDark),
      collectionColors: [
        for (var i = 0; i < collectionColors.length; i++)
          i < other.collectionColors.length
              ? mix(collectionColors[i], other.collectionColors[i])
              : collectionColors[i],
      ],
    );
  }
}

/// What the window looks like, set once above it: the painters' ink and
/// whether tooltips show. Panels read it through [EditorInk.of] and
/// [EditorTooltip]; where nothing is set (a test that mounts one control) the
/// window's one look applies and tooltips are on.
class EditorLook extends InheritedWidget {
  const EditorLook({
    super.key,
    this.ink = EditorInk.dark,
    this.tooltips = true,
    required super.child,
  });
  final EditorInk ink;
  final bool tooltips;
  static const _none = EditorLook(child: SizedBox.shrink());
  static EditorLook of(BuildContext context) =>
      context.dependOnInheritedWidgetOfExactType<EditorLook>() ?? _none;
  @override
  bool updateShouldNotify(EditorLook old) =>
      old.ink != ink || old.tooltips != tooltips;
}

abstract final class EditorTheme {
  // Lifts are Material 3 state layers: white over the surface at the M3
  // opacities (reference/ui-lift.tsv). Every rise in the window is one of
  // these, and the grey ladder below is `app` lifted by one hover step
  // per rung, so a hover, a chosen row and a panel edge all climb the
  // same stair.
  static const hoverLift = .08, pressedLift = .10, draggedLift = .16;

  /// A chosen surface: two hover steps, the M3 dragged layer.
  static const lift = draggedLift;
  static const app = Color(0xff292929),
      panel = Color(0xff3a3a3a),
      raised = Color(0xff4a4a4a),
      hover = Color(0xff585858);
  static const line = Color(0xff242424),
      border = Color(0xff666666),
      ink = Color(0xffdddddd),
      muted = Color(0xffaaaaaa),
      accent = Color(0xffffaa61);

  /// Where a touch lands as a key — key lamps, the Animate switch, the
  /// playhead and keys — the accent turns to its hue complement while
  /// Animate is on. Everything else keeps the accent for "chosen".
  static const animate = Color(0xff61b6ff);
  static final animating = ValueNotifier<bool>(false);
  static Color get keyAccent => animating.value ? animate : accent;
  static const tab = Color(0xffb7b7b7), tabInk = Color(0xff262626);
  // Menus: a darker sheet, a pale edge, a pale hover row with dark ink.
  static const menu = Color(0xff222222),
      menuEdge = Color(0xffbbbbbb),
      select = Color(0xffaedce8),
      selectInk = Color(0xff172126),
      disabledInk = Color(0xff888888),
      error = Color(0xffcf6679);
  // Character families: what a number is for, told by hue (the OP-1 rule:
  // one colour per family, on the glyph, the track and the handle alike).
  static const spatial = Color(0xff93a5f5),
      amount = Color(0xffe6a275),
      time = Color(0xffc18bd3),
      count = Color(0xff79c4ca),
      seed = Color(0xffeedb73),
      angle = Color(0xff95c78b);
  static const identityColors = [
    Color(0xff93a5f5),
    Color(0xffeedb73),
    Color(0xffc18bd3),
    Color(0xff79c4ca),
    Color(0xffe6a275),
    Color(0xff95c78b),
    Color(0xffdd879e),
  ];
  static Color layerColor(dynamic id) =>
      identityColors[(id is num ? id.toInt() : 0).abs() %
          identityColors.length];
  static Color kindColor(String kind) => switch (kind.toLowerCase()) {
    'text' => const Color(0xffeedb73),
    'rectangle' ||
    'roundedrectangle' ||
    'ellipse' ||
    'star' ||
    'polygon' ||
    'shape' => const Color(0xff93a5f5),
    'bezier' || 'line' || 'path' => const Color(0xff79c4ca),
    'video' => const Color(0xffdd879e),
    'audio' => const Color(0xff95c78b),
    '3d' => const Color(0xffe6a275),
    'hdr' => const Color(0xffe6c96a),
    '2d' || 'images' => const Color(0xff93a5f5),
    _ => const Color(0xffc18bd3),
  };

  /// The two ends of the scale and no colour at all: named so a painter's
  /// white wash or a clear drag feedback reads as a token, not a literal.
  static const white = Color(0xffffffff),
      black = Color(0xff000000),
      clear = Color(0x00000000);

  /// Black at the two opacities the picture editors' scrims use: under a
  /// dialog, and around the colour wheel's cursor.
  static const scrim = Color(0x8a000000), scrimLight = Color(0x73000000);

  /// Lifts Material drew under a hovered, pressed or keyboard-focused
  /// surface: white at the M3 state-layer opacities (hover 4%, focus 12%),
  /// and the pressed lift at [hover].
  static const hoverWash = Color(0x0affffff), focusWash = Color(0x1fffffff);

  /// What cannot be pressed: ink at the M3 disabled 38%, a filled ground at
  /// 12%.
  static const inkDisabled = Color(0x61ffffff),
      washDisabled = Color(0x1fffffff);

  /// The scrollbar's thumb: white at 30%, 65% under the pointer, 75% while
  /// dragged (the dark theme's desktop scrollbar).
  static const scrollThumb = Color(0x4dffffff),
      scrollThumbHovered = Color(0xa6ffffff),
      scrollThumbDragged = Color(0xbfffffff);

  /// A slider's division marks: black and white at 38%.
  static const tickActive = Color(0x61000000), tickInactive = Color(0x61ffffff);

  /// The tooltip's sheet: white at 90%.
  static const tooltip = Color(0xe6ffffff);

  /// Where the text is read and written: the caret and the selection.
  static const caret = accent;
  static const selection = Color(0x66ffaa61);

  /// What the window's text is when nothing says otherwise: the body size
  /// in ink, without a font family (the platform's UI face).
  static const text = TextStyle(
    inherit: false,
    fontSize: EditorMetrics.font,
    fontWeight: FontWeight.w400,
    color: ink,
    textBaseline: TextBaseline.alphabetic,
  );
  static const icon = IconThemeData(size: 14, color: ink);

  /// The menu sheet: a darker ground, a pale edge, rows [EditorMetrics.row]
  /// high, a pale hover row with dark ink. Both the choice and the context
  /// menu draw from these.
  static const menuPadding = EdgeInsets.symmetric(vertical: EditorMetrics.s2);
  static const menuMinWidth = 112.0;
  static const menuRowPadding = EdgeInsets.symmetric(
    horizontal: EditorMetrics.s8,
  );
}

/// A tooltip where tooltips are shown, and nothing at all where they are not.
/// [EditorLook.tooltips] turns the window off but leaves the widget standing,
/// and each one that stands costs a hover region, a long-press detector and a
/// semantics node in every layout of the window.
class EditorTooltip extends StatelessWidget {
  const EditorTooltip({super.key, required this.message, required this.child});
  final String message;
  final Widget child;
  @override
  Widget build(BuildContext context) => EditorLook.of(context).tooltips
      ? RawTooltip(
          semanticsTooltip: message,
          ignorePointer: true,
          touchDelay: const Duration(milliseconds: 1500),
          tooltipBuilder: (context, animation) => FadeTransition(
            opacity: animation,
            child: EditorTooltipSheet(message: message),
          ),
          child: child,
        )
      : child;
}

class EditorButton extends StatelessWidget {
  const EditorButton(
    this.label,
    this.onPressed, {
    super.key,
    this.selected = false,
    this.tooltip,
  });
  final String label;
  final VoidCallback? onPressed;
  final bool selected;
  final String? tooltip;
  @override
  Widget build(BuildContext context) {
    final button = Padding(
      padding: const EdgeInsets.symmetric(horizontal: 1, vertical: 1),
      child: SizedBox(
        height: EditorMetrics.row - 2,
        child: EditorTextButton(
          onPressed: onPressed,
          background: selected ? EditorTheme.accent : EditorTheme.panel,
          foreground: selected ? EditorTheme.tabInk : EditorTheme.ink,
          child: Text(label, maxLines: 1),
        ),
      ),
    );
    return tooltip == null
        ? button
        : EditorTooltip(message: tooltip!, child: button);
  }
}

class EditorSection extends StatelessWidget {
  const EditorSection(this.title, this.child, {super.key});
  final String title;
  final Widget child;
  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    mainAxisSize: MainAxisSize.min,
    children: [
      Container(
        height: EditorMetrics.section,
        decoration: const BoxDecoration(
          color: EditorTheme.raised,
          border: Border(bottom: BorderSide(color: EditorTheme.line)),
        ),
        alignment: Alignment.centerLeft,
        padding: const EdgeInsets.symmetric(horizontal: 8),
        child: Text(
          title.toUpperCase(),
          style: const TextStyle(
            fontSize: 10,
            fontWeight: FontWeight.w600,
            letterSpacing: .5,
            color: EditorTheme.ink,
          ),
        ),
      ),
      child,
    ],
  );
}

/// A context menu at a pointer's screen position. Needs nothing set up
/// ahead of it — it drops itself into the nearest [Overlay] (the app's
/// Navigator has one) and removes itself when it closes. No animation; as
/// wide as its widest row. The rows are [EditorMenuItem]s and
/// [EditorMenuDivider]s.
Future<T?> showEditorMenu<T>(
  BuildContext context,
  Offset at,
  List<Widget> items,
) {
  final overlay = Overlay.of(context);
  final overlayBox = overlay.context.findRenderObject() as RenderBox;
  final position = overlayBox.globalToLocal(at);
  final completer = Completer<T?>();
  late final OverlayEntry entry;
  void settle(Object? value) {
    if (completer.isCompleted) return;
    completer.complete(value as T?);
    entry.remove();
  }

  entry = OverlayEntry(
    builder: (context) => EditorMenuSheet(
      anchor: Rect.fromLTWH(position.dx, position.dy, 0, 0),
      minWidth: 0,
      consumeOutsideTap: true,
      onClose: () => settle(null),
      child: _EditorMenuScope(
        pick: settle,
        child: Column(mainAxisSize: MainAxisSize.min, children: items),
      ),
    ),
  );
  overlay.insert(entry);
  return completer.future;
}

/// Threads a row's pick back out to the [showEditorMenu] call that opened it.
class _EditorMenuScope extends InheritedWidget {
  const _EditorMenuScope({required this.pick, required super.child});
  final ValueChanged<Object?> pick;
  @override
  bool updateShouldNotify(_EditorMenuScope old) => false;
  static void pickFrom(BuildContext context, Object? value) => context
      .dependOnInheritedWidgetOfExactType<_EditorMenuScope>()!
      .pick(value);
}

class EditorMenuItem<T> extends StatelessWidget {
  const EditorMenuItem({
    super.key,
    this.value,
    this.enabled = true,
    required this.child,
  });
  final T? value;
  final bool enabled;
  final Widget child;
  @override
  Widget build(BuildContext context) => EditorMenuRow(
    onPressed: enabled ? () => _EditorMenuScope.pickFrom(context, value) : null,
    child: child,
  );
}

class EditorMenuDivider extends StatelessWidget {
  const EditorMenuDivider({super.key});
  @override
  Widget build(BuildContext context) =>
      const EditorRule(height: EditorMetrics.s8, thickness: 1);
}
