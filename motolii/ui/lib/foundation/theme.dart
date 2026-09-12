import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';

import 'metrics.dart';

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

  /// The menu sheet and its rows, for the ThemeData and for EditorChoice
  /// (a MenuAnchor sets them itself, so a stray Theme cannot lose them).
  static const menuSheet = MenuStyle(
    backgroundColor: WidgetStatePropertyAll(menu),
    surfaceTintColor: WidgetStatePropertyAll(Colors.transparent),
    shadowColor: WidgetStatePropertyAll(Colors.transparent),
    elevation: WidgetStatePropertyAll(0),
    padding: WidgetStatePropertyAll(
      EdgeInsets.symmetric(vertical: EditorMetrics.s2),
    ),
    shape: WidgetStatePropertyAll(
      RoundedRectangleBorder(
        borderRadius: BorderRadius.zero,
        side: BorderSide(color: menuEdge),
      ),
    ),
    visualDensity: VisualDensity.compact,
  );
  static final menuRow = ButtonStyle(
    minimumSize: const WidgetStatePropertyAll(Size(0, EditorMetrics.row)),
    maximumSize: const WidgetStatePropertyAll(
      Size(double.infinity, EditorMetrics.row),
    ),
    padding: const WidgetStatePropertyAll(
      EdgeInsets.symmetric(horizontal: EditorMetrics.s8),
    ),
    textStyle: const WidgetStatePropertyAll(
      TextStyle(fontSize: EditorMetrics.font),
    ),
    tapTargetSize: MaterialTapTargetSize.shrinkWrap,
    // The app-wide compact density would take 8 off the row.
    visualDensity: VisualDensity.standard,
    shape: const WidgetStatePropertyAll(RoundedRectangleBorder()),
    overlayColor: const WidgetStatePropertyAll(Colors.transparent),
    backgroundColor: WidgetStateProperty.resolveWith(
      (s) => s.contains(WidgetState.hovered) ? select : Colors.transparent,
    ),
    foregroundColor: WidgetStateProperty.resolveWith(
      (s) => s.contains(WidgetState.disabled)
          ? disabledInk
          : s.contains(WidgetState.hovered)
          ? selectInk
          : ink,
    ),
  );
  static ThemeData get data => ThemeData.dark(useMaterial3: true).copyWith(
    scaffoldBackgroundColor: app,
    canvasColor: panel,
    dividerColor: line,
    splashFactory: NoSplash.splashFactory,
    highlightColor: hover,
    // MenuAnchor menus (choices, context menus) share one sheet.
    menuTheme: const MenuThemeData(style: menuSheet),
    menuButtonTheme: MenuButtonThemeData(style: menuRow),
    // The icon is the button: no minimum square, no padding, no stadium ink.
    // Every IconButton therefore measures exactly its own `iconSize`.
    iconButtonTheme: IconButtonThemeData(
      style: ButtonStyle(
        padding: const WidgetStatePropertyAll(EdgeInsets.zero),
        minimumSize: const WidgetStatePropertyAll(Size.zero),
        maximumSize: const WidgetStatePropertyAll(Size.infinite),
        tapTargetSize: MaterialTapTargetSize.shrinkWrap,
        visualDensity: VisualDensity.standard,
        shape: const WidgetStatePropertyAll(RoundedRectangleBorder()),
        backgroundColor: const WidgetStatePropertyAll(Colors.transparent),
        overlayColor: WidgetStateProperty.resolveWith(
          (s) =>
              s.contains(WidgetState.hovered) || s.contains(WidgetState.pressed)
              ? hover
              : Colors.transparent,
        ),
        iconColor: WidgetStateProperty.resolveWith(
          (s) => s.contains(WidgetState.disabled) ? disabledInk : ink,
        ),
      ),
    ),
    sliderTheme: SliderThemeData(
      trackHeight: EditorMetrics.s2,
      // M3's own track draws a gap and a stop indicator; the flat rectangle is
      // the one this editor has always shown.
      trackShape: const RoundedRectSliderTrackShape(),
      thumbShape: const RoundSliderThumbShape(
        enabledThumbRadius: EditorMetrics.s5,
      ),
      overlayShape: SliderComponentShape.noOverlay,
      padding: EdgeInsets.zero,
      activeTrackColor: muted,
      inactiveTrackColor: line,
      thumbColor: ink,
    ),
    // M3 forces a padded 40dp tap target on these two regardless of the
    // app-wide `materialTapTargetSize`, so they say it again here.
    checkboxTheme: CheckboxThemeData(
      materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
      visualDensity: VisualDensity.compact,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(1)),
    ),
    switchTheme: const SwitchThemeData(
      materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
      trackOutlineColor: WidgetStatePropertyAll(Colors.transparent),
      thumbIcon: WidgetStatePropertyAll(null),
    ),
    dividerTheme: const DividerThemeData(
      color: line,
      thickness: 0,
      space: EditorMetrics.s16,
    ),
    dialogTheme: const DialogThemeData(
      backgroundColor: panel,
      surfaceTintColor: Colors.transparent,
      elevation: 0,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.zero,
        side: BorderSide(color: border),
      ),
      titleTextStyle: TextStyle(fontSize: EditorMetrics.title, color: ink),
      contentTextStyle: TextStyle(fontSize: EditorMetrics.font, color: ink),
    ),
    colorScheme: const ColorScheme.dark(
      primary: accent,
      secondary: accent,
      surface: panel,
    ),
    textTheme: const TextTheme(
      bodyLarge: TextStyle(fontSize: EditorMetrics.font, color: ink),
      bodyMedium: TextStyle(fontSize: EditorMetrics.font, color: ink),
      bodySmall: TextStyle(fontSize: EditorMetrics.dense, color: muted),
      titleMedium: TextStyle(fontSize: EditorMetrics.font, color: ink),
      labelLarge: TextStyle(fontSize: EditorMetrics.font, color: ink),
    ),
    visualDensity: VisualDensity.compact,
    // M3's 2021 geometry puts a 1.5 line height on body text, which grows
    // every field and label. The 2014 geometry is the one these sizes were
    // tuned against; only the shapes below move to M3.
    typography: Typography.material2014(platform: defaultTargetPlatform),
    materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
    inputDecorationTheme: const InputDecorationTheme(
      isDense: true,
      filled: true,
      fillColor: app,
      contentPadding: EdgeInsets.symmetric(horizontal: 4, vertical: 3),
      enabledBorder: OutlineInputBorder(
        borderRadius: BorderRadius.zero,
        borderSide: BorderSide(color: border),
      ),
      focusedBorder: OutlineInputBorder(
        borderRadius: BorderRadius.zero,
        borderSide: BorderSide(color: accent),
      ),
      disabledBorder: OutlineInputBorder(
        borderRadius: BorderRadius.zero,
        borderSide: BorderSide(color: line),
      ),
      // errorText is used by EditorField; M3 would round these corners.
      errorBorder: OutlineInputBorder(
        borderRadius: BorderRadius.zero,
        borderSide: BorderSide(color: error),
      ),
      focusedErrorBorder: OutlineInputBorder(
        borderRadius: BorderRadius.zero,
        borderSide: BorderSide(color: error),
      ),
    ),
    textButtonTheme: TextButtonThemeData(
      style: TextButton.styleFrom(
        minimumSize: const Size(18, 20),
        shape: const RoundedRectangleBorder(
          borderRadius: BorderRadius.zero,
          side: BorderSide(color: line),
        ),
        padding: const EdgeInsets.symmetric(horizontal: 4),
        tapTargetSize: MaterialTapTargetSize.shrinkWrap,
        textStyle: const TextStyle(fontSize: EditorMetrics.font),
      ),
    ),
    iconTheme: const IconThemeData(size: 14, color: ink),
  );
}

/// A tooltip where tooltips are shown, and nothing at all where they are not.
/// [TooltipVisibility] turns the panel off but leaves the widget standing, and
/// each one that stands costs a hover region, a long-press detector and a
/// semantics node in every layout of the window.
class EditorTooltip extends StatelessWidget {
  const EditorTooltip({super.key, required this.message, required this.child});
  final String message;
  final Widget child;
  @override
  Widget build(BuildContext context) => TooltipVisibility.of(context)
      ? Tooltip(message: message, child: child)
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
        child: TextButton(
          onPressed: onPressed,
          style: TextButton.styleFrom(
            backgroundColor: selected ? EditorTheme.accent : EditorTheme.panel,
            foregroundColor: selected ? EditorTheme.tabInk : EditorTheme.ink,
          ),
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
/// ahead of it — it drops itself into the nearest [Overlay] (every
/// [MaterialApp] already has one) and removes itself when it closes, the
/// way [showMenu] does. No animation; as wide as its widest row. The rows
/// are [EditorMenuItem]s and [EditorMenuDivider]s.
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
  var picked = false;
  void settle(Object? value) {
    if (!completer.isCompleted) completer.complete(value as T?);
    entry.remove();
  }

  final controller = MenuController();
  entry = OverlayEntry(
    builder: (context) => Positioned(
      left: position.dx,
      top: position.dy,
      child: _EditorMenuHost(
        controller: controller,
        items: items,
        onPick: (v) {
          picked = true;
          settle(v);
        },
        onClose: () {
          if (!picked) settle(null);
        },
      ),
    ),
  );
  overlay.insert(entry);
  WidgetsBinding.instance.addPostFrameCallback((_) => controller.open());
  return completer.future;
}

/// The MenuAnchor a [showEditorMenu] call briefly owns: it opens itself once
/// mounted and reports back however it closed (a pick, or an outside tap).
class _EditorMenuHost extends StatelessWidget {
  const _EditorMenuHost({
    required this.controller,
    required this.items,
    required this.onPick,
    required this.onClose,
  });
  final MenuController controller;
  final List<Widget> items;
  final ValueChanged<Object?> onPick;
  final VoidCallback onClose;
  @override
  Widget build(BuildContext context) => _EditorMenuScope(
    pick: onPick,
    child: MenuAnchor(
      controller: controller,
      animated: false,
      consumeOutsideTap: true,
      crossAxisUnconstrained: false,
      style: EditorTheme.menuSheet.copyWith(
        minimumSize: const WidgetStatePropertyAll(Size.zero),
      ),
      onClose: onClose,
      menuChildren: items,
      child: const SizedBox.shrink(),
    ),
  );
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
  Widget build(BuildContext context) => MenuItemButton(
    style: EditorTheme.menuRow,
    closeOnActivate: false,
    onPressed: enabled ? () => _EditorMenuScope.pickFrom(context, value) : null,
    child: child,
  );
}

class EditorMenuDivider extends StatelessWidget {
  const EditorMenuDivider({super.key});
  @override
  Widget build(BuildContext context) =>
      const Divider(height: EditorMetrics.s8, thickness: 1);
}
