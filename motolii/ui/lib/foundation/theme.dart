import 'dart:async';
import 'dart:convert';

import 'package:flutter/widgets.dart';
import 'package:flutter/material.dart'
    show Theme, ThemeData, ThemeExtension, ColorScheme;

import 'leaves.dart';
import 'metrics.dart';

/// Colours of what is drawn rather than laid out — lanes, grids, ticks,
/// gizmos, the transparency grid, the ease desk's paper — carried by
/// [EditorTheme] above the window so a painter's colours come from the theme
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

  Map<String, Color> get namedColors => {
    'laneGround': laneGround,
    'lane': lane,
    'laneAlt': laneAlt,
    'grid': grid,
    'gridMinor': gridMinor,
    'tick': tick,
    'tickMinor': tickMinor,
    'headerInk': headerInk,
    'camera': camera,
    'focusRing': focusRing,
    'easePaper': easePaper,
    'easeInk': easeInk,
    'easeTime': easeTime,
    'checkerLight': checkerLight,
    'checkerDark': checkerDark,
  };

  EditorInk withColors(Map<String, Color> colors, List<Color> collections) =>
      copyWith(
        laneGround: colors['laneGround'],
        lane: colors['lane'],
        laneAlt: colors['laneAlt'],
        grid: colors['grid'],
        gridMinor: colors['gridMinor'],
        tick: colors['tick'],
        tickMinor: colors['tickMinor'],
        headerInk: colors['headerInk'],
        camera: colors['camera'],
        focusRing: colors['focusRing'],
        easePaper: colors['easePaper'],
        easeInk: colors['easeInk'],
        easeTime: colors['easeTime'],
        checkerLight: colors['checkerLight'],
        checkerDark: colors['checkerDark'],
        collectionColors: collections,
      );

  static const dark = EditorInk(
    laneGround: Color(0xff2c2c2c),
    lane: Color(0xff2b2b2b),
    laneAlt: Color(0xff323232),
    grid: Color(0xff1c1c1c),
    gridMinor: Color(0xff272727),
    tick: Color(0xffbababa),
    tickMinor: Color(0xffaaaaaa),
    headerInk: Color(0xff1d1d1d),
    camera: Color(0xff8ed9e6),
    focusRing: Color(0xffacacac),
    easePaper: Color(0xffd2d2d2),
    easeInk: Color(0xff333333),
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

  static EditorInk of(BuildContext context) => EditorTheme.of(context).drawing;

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

/// Whether tooltips show; color inheritance belongs to [EditorTheme].
class EditorLook extends InheritedWidget {
  const EditorLook({super.key, this.tooltips = true, required super.child});
  final bool tooltips;
  static const _none = EditorLook(child: SizedBox.shrink());
  static EditorLook of(BuildContext context) =>
      context.dependOnInheritedWidgetOfExactType<EditorLook>() ?? _none;
  @override
  bool updateShouldNotify(EditorLook old) => old.tooltips != tooltips;
}

@immutable
class EditorTheme extends ThemeExtension<EditorTheme> {
  const EditorTheme._({
    required this.name,
    required this.colors,
    required this.identityColors,
    required this.drawing,
  });
  final String name;
  final Map<String, Color> colors;
  final List<Color> identityColors;
  final EditorInk drawing;

  static const chromatic = EditorTheme._(
    name: 'Chromatic Workshop',
    colors: {
      'app': Color(0xff252525),
      'panel': Color(0xff323232),
      'raised': Color(0xff414141),
      'hover': Color(0xff4e4e4e),
      'line': Color(0xff1b1b1b),
      'border': Color(0xff676767),
      'ink': Color(0xfff0f0f0),
      'muted': Color(0xffbdbdbd),
      'accent': Color(0xffffbc53),
      'animate': Color(0xff5396ff),
      'tab': Color(0xff59c9df),
      'tabInk': Color(0xff202020),
      'menu': Color(0xff252525),
      'menuEdge': Color(0xff7e7e7e),
      'select': Color(0xffb0e3ef),
      'selectInk': Color(0xff202020),
      'disabledInk': Color(0xff929292),
      'error': Color(0xffff8899),
      'spatial': Color(0xff819fff),
      'amount': Color(0xfff5ad79),
      'time': Color(0xffc08ee4),
      'count': Color(0xff60cedb),
      'seed': Color(0xffffdf56),
      'angle': Color(0xffa0d292),
      'scrim': Color(0x8a000000),
      'scrimLight': Color(0x73000000),
      'hoverWash': Color(0x0affffff),
      'focusWash': Color(0x1fffffff),
      'inkDisabled': Color(0x61ffffff),
      'washDisabled': Color(0x1fffffff),
      'scrollThumb': Color(0x4dffffff),
      'scrollThumbHovered': Color(0xa6ffffff),
      'scrollThumbDragged': Color(0xbfffffff),
      'tickActive': Color(0x61000000),
      'tickInactive': Color(0x61ffffff),
      'tooltip': Color(0xe6ffffff),
      'selection': Color(0x6659c9df),
      'kindText': Color(0xffffdf56),
      'kindShape': Color(0xff819fff),
      'kindPath': Color(0xff60cedb),
      'kindVideo': Color(0xffef87ae),
      'kindAudio': Color(0xffa0d292),
      'kind3d': Color(0xffcf8eef),
      'kindHdr': Color(0xffffdf78),
      'kindImage': Color(0xff819fff),
      'kindOther': Color(0xffc08ee4),
      'caret': Color(0xffffbc53),
      'timelineWash': Color(0x4d606060),
    },
    identityColors: [
      Color(0xff819fff),
      Color(0xffffdf56),
      Color(0xffc08ee4),
      Color(0xff60cedb),
      Color(0xfff5ad79),
      Color(0xffa0d292),
      Color(0xffef87ae),
    ],
    drawing: EditorInk.dark,
  );

  /// Standalone controls use the same complete palette as the application.
  static EditorTheme of(BuildContext context) =>
      Theme.of(context).extension<EditorTheme>() ?? chromatic;

  Color get app => colors['app']!;
  Color get panel => colors['panel']!;
  Color get raised => colors['raised']!;
  Color get hover => colors['hover']!;
  Color get line => colors['line']!;
  Color get border => colors['border']!;
  Color get ink => colors['ink']!;
  Color get muted => colors['muted']!;
  Color get accent => colors['accent']!;
  Color get animate => colors['animate']!;
  Color get tab => colors['tab']!;
  Color get tabInk => colors['tabInk']!;
  Color get menu => colors['menu']!;
  Color get menuEdge => colors['menuEdge']!;
  Color get select => colors['select']!;
  Color get selectInk => colors['selectInk']!;
  Color get disabledInk => colors['disabledInk']!;
  Color get error => colors['error']!;
  Color get spatial => colors['spatial']!;
  Color get amount => colors['amount']!;
  Color get time => colors['time']!;
  Color get count => colors['count']!;
  Color get seed => colors['seed']!;
  Color get angle => colors['angle']!;
  Color get scrim => colors['scrim']!;
  Color get scrimLight => colors['scrimLight']!;
  Color get hoverWash => colors['hoverWash']!;
  Color get focusWash => colors['focusWash']!;
  Color get inkDisabled => colors['inkDisabled']!;
  Color get washDisabled => colors['washDisabled']!;
  Color get scrollThumb => colors['scrollThumb']!;
  Color get scrollThumbHovered => colors['scrollThumbHovered']!;
  Color get scrollThumbDragged => colors['scrollThumbDragged']!;
  Color get tickActive => colors['tickActive']!;
  Color get tickInactive => colors['tickInactive']!;
  Color get tooltip => colors['tooltip']!;
  Color get selection => colors['selection']!;
  Color get kindText => colors['kindText']!;
  Color get kindShape => colors['kindShape']!;
  Color get kindPath => colors['kindPath']!;
  Color get kindVideo => colors['kindVideo']!;
  Color get kindAudio => colors['kindAudio']!;
  Color get kind3d => colors['kind3d']!;
  Color get kindHdr => colors['kindHdr']!;
  Color get kindImage => colors['kindImage']!;
  Color get kindOther => colors['kindOther']!;
  Color get caret => colors['caret']!;
  Color get timelineWash => colors['timelineWash']!;
  static const white = Color(0xffffffff),
      black = Color(0xff000000),
      clear = Color(0x00000000);
  static const hoverLift = .08,
      pressedLift = .10,
      draggedLift = .16,
      lift = draggedLift;
  static const fontFamily = 'Inter';
  static final animating = ValueNotifier<bool>(false);
  Color get keyAccent => animating.value ? animate : accent;
  Color layerColor(dynamic id) =>
      identityColors[(id is num ? id.toInt() : 0).abs() %
          identityColors.length];
  static final _timelinePalettes = Expando<List<Color>>();
  Color timelineColor(dynamic id) {
    final palette = _timelinePalettes[this] ??= List<Color>.unmodifiable(
      identityColors.map((color) => Color.alphaBlend(timelineWash, color)),
    );
    return palette[(id is num ? id.toInt() : 0).abs() % palette.length];
  }

  Color kindColor(String kind) => switch (kind.toLowerCase()) {
    'text' => kindText,
    'rectangle' ||
    'roundedrectangle' ||
    'ellipse' ||
    'star' ||
    'polygon' ||
    'shape' => kindShape,
    'bezier' || 'line' || 'path' => kindPath,
    'video' => kindVideo,
    'audio' => kindAudio,
    '3d' => kind3d,
    'hdr' => kindHdr,
    '2d' || 'images' => kindImage,
    _ => kindOther,
  };
  TextStyle get text => TextStyle(
    inherit: false,
    fontFamily: fontFamily,
    fontSize: EditorMetrics.font,
    fontWeight: FontWeight.w500,
    color: ink,
    textBaseline: TextBaseline.alphabetic,
  );
  IconThemeData get icon => IconThemeData(size: EditorMetrics.s14, color: ink);
  static const menuPadding = EdgeInsets.symmetric(vertical: EditorMetrics.s2);
  static const menuMinWidth = 112.0;
  static const menuRowPadding = EdgeInsets.symmetric(
    horizontal: EditorMetrics.s8,
  );

  factory EditorTheme.fromJson(Object? source) {
    if (source is! Map || source.keys.any((key) => key is! String))
      throw const FormatException('Theme must be a JSON object');
    final value = Map<String, dynamic>.from(source);
    const fields = {
      'schemaVersion',
      'name',
      'colors',
      'identityColors',
      'drawing',
      'collectionColors',
    };
    for (final key in value.keys) {
      if (!fields.contains(key))
        throw FormatException('Unknown theme field: $key');
    }
    if (value['schemaVersion'] != 1)
      throw const FormatException('Theme schemaVersion must be 1');
    final name = value['name'];
    if (name is! String || name.trim().isEmpty || name.length > 80) {
      throw const FormatException('Theme name must contain 1 to 80 characters');
    }
    Map<String, Color> group(String name, Map<String, Color> defaults) {
      if (!value.containsKey(name)) return defaults;
      final raw = value[name];
      if (raw is! Map) throw FormatException('$name must be an object');
      final result = Map<String, Color>.of(defaults);
      for (final entry in raw.entries) {
        if (!defaults.containsKey(entry.key))
          throw FormatException('Unknown $name token: ${entry.key}');
        result[entry.key] = _readColor(entry.value, '$name.${entry.key}');
      }
      return Map.unmodifiable(result);
    }

    List<Color> palette(String name, List<Color> defaults, {int? count}) {
      if (!value.containsKey(name)) return defaults;
      final raw = value[name];
      if (raw is! List ||
          raw.isEmpty ||
          raw.length > 64 ||
          (count != null && raw.length != count)) {
        throw FormatException(
          '$name must contain ${count ?? "1 to 64"} colors',
        );
      }
      return List.unmodifiable([
        for (var i = 0; i < raw.length; i++) _readColor(raw[i], '$name[$i]'),
      ]);
    }

    return EditorTheme._(
      name: name.trim(),
      colors: group('colors', chromatic.colors),
      identityColors: palette('identityColors', chromatic.identityColors),
      drawing: chromatic.drawing.withColors(
        group('drawing', chromatic.drawing.namedColors),
        palette(
          'collectionColors',
          chromatic.drawing.collectionColors,
          count: 7,
        ),
      ),
    );
  }

  static Color _readColor(Object? value, String token) {
    if (value is! String ||
        !RegExp(r'^#[0-9a-fA-F]{6}([0-9a-fA-F]{2})?$').hasMatch(value)) {
      throw FormatException('$token must be #RRGGBB or #RRGGBBAA');
    }
    final number = int.parse(value.substring(1), radix: 16);
    return Color(
      value.length == 7
          ? 0xff000000 | number
          : ((number & 0xff) << 24) | (number >> 8),
    );
  }

  Map<String, Object> toJson() {
    String hex(Color color) {
      final argb = color.toARGB32();
      final rgb = (argb & 0xffffff).toRadixString(16).padLeft(6, '0');
      final alpha = argb >> 24;
      return '#$rgb${alpha == 255 ? "" : alpha.toRadixString(16).padLeft(2, "0")}';
    }

    return {
      'schemaVersion': 1,
      'name': name,
      'colors': colors.map((key, value) => MapEntry(key, hex(value))),
      'identityColors': identityColors.map(hex).toList(),
      'drawing': drawing.namedColors.map(
        (key, value) => MapEntry(key, hex(value)),
      ),
      'collectionColors': drawing.collectionColors.map(hex).toList(),
    };
  }

  String get json => const JsonEncoder.withIndent('  ').convert(toJson());

  Widget wrap(Widget child) => Theme(
    data: ThemeData(
      brightness: Brightness.dark,
      colorScheme: ColorScheme.dark(
        primary: accent,
        secondary: spatial,
        surface: panel,
        onSurface: ink,
        onPrimary: tabInk,
        error: error,
      ),
      extensions: [this],
    ),
    child: child,
  );

  @override
  EditorTheme copyWith({
    String? name,
    Map<String, Color>? colors,
    List<Color>? identityColors,
    EditorInk? drawing,
  }) => EditorTheme._(
    name: name ?? this.name,
    colors: Map.unmodifiable({...this.colors, ...?colors}),
    identityColors: List.unmodifiable(identityColors ?? this.identityColors),
    drawing: drawing ?? this.drawing,
  );
  @override
  EditorTheme lerp(covariant EditorTheme? other, double t) {
    if (other == null) return this;
    return copyWith(
      name: t < .5 ? name : other.name,
      colors: {
        for (final key in colors.keys)
          key: Color.lerp(colors[key], other.colors[key], t)!,
      },
      identityColors: identityColors.length == other.identityColors.length
          ? [
              for (var i = 0; i < identityColors.length; i++)
                Color.lerp(identityColors[i], other.identityColors[i], t)!,
            ]
          : (t < .5 ? identityColors : other.identityColors),
      drawing: drawing.lerp(other.drawing, t),
    );
  }
}

class EditorAppearance extends InheritedNotifier<ValueNotifier<EditorTheme>> {
  const EditorAppearance({
    super.key,
    required super.notifier,
    required super.child,
  });
  static ValueNotifier<EditorTheme>? of(BuildContext context) =>
      context.getInheritedWidgetOfExactType<EditorAppearance>()?.notifier;
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
          background: selected
              ? EditorTheme.of(context).accent
              : EditorTheme.of(context).panel,
          foreground: selected
              ? EditorTheme.of(context).tabInk
              : EditorTheme.of(context).ink,
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
        decoration: BoxDecoration(
          color: EditorTheme.of(context).raised,
          border: Border(
            bottom: BorderSide(color: EditorTheme.of(context).line),
          ),
        ),
        alignment: Alignment.centerLeft,
        padding: const EdgeInsets.symmetric(horizontal: 8),
        child: Text(
          title.toUpperCase(),
          style: TextStyle(
            fontSize: 10,
            fontWeight: FontWeight.w600,
            letterSpacing: .5,
            color: EditorTheme.of(context).ink,
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
