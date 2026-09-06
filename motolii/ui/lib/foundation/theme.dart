import 'package:flutter/material.dart';

abstract final class EditorTheme {
  static const app = Color(0xff292929),
      panel = Color(0xff3c3c3c),
      raised = Color(0xff484848),
      hover = Color(0xff585858);
  static const line = Color(0xff242424),
      border = Color(0xff666666),
      ink = Color(0xffdddddd),
      muted = Color(0xffaaaaaa),
      accent = Color(0xffffaa61);
  static const tab = Color(0xffb7b7b7), tabInk = Color(0xff262626);
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
    'rectangle' || 'shape' => const Color(0xff93a5f5),
    'bezier' || 'path' => const Color(0xff79c4ca),
    'video' => const Color(0xffdd879e),
    'audio' => const Color(0xff95c78b),
    '3d' => const Color(0xffe6a275),
    '2d' || 'images' => const Color(0xff93a5f5),
    _ => const Color(0xffc18bd3),
  };
  static const double row = 20, section = 26, font = 11, dense = 10;
  static ThemeData get data => ThemeData.dark(useMaterial3: false).copyWith(
    scaffoldBackgroundColor: app,
    canvasColor: panel,
    dividerColor: line,
    splashFactory: NoSplash.splashFactory,
    highlightColor: hover,
    popupMenuTheme: const PopupMenuThemeData(
      color: Color(0xff222222),
      surfaceTintColor: Colors.transparent,
      shadowColor: Colors.transparent,
      menuPadding: EdgeInsets.symmetric(vertical: 2),
      textStyle: TextStyle(fontSize: 11, color: ink),
      elevation: 0,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.zero,
        side: BorderSide(color: Color(0xffbbbbbb)),
      ),
    ),
    colorScheme: const ColorScheme.dark(
      primary: accent,
      secondary: accent,
      surface: panel,
    ),
    textTheme: const TextTheme(
      bodyMedium: TextStyle(fontSize: font, color: ink),
      bodySmall: TextStyle(fontSize: dense, color: muted),
      titleMedium: TextStyle(fontSize: font, color: ink),
      labelLarge: TextStyle(fontSize: font, color: ink),
    ),
    visualDensity: VisualDensity.compact,
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
        textStyle: const TextStyle(fontSize: font),
      ),
    ),
    iconTheme: const IconThemeData(size: 14, color: ink),
    tooltipTheme: const TooltipThemeData(
      waitDuration: Duration(milliseconds: 500),
      textStyle: TextStyle(fontSize: 11, color: Colors.white),
    ),
  );
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
        height: EditorTheme.row - 2,
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
    return tooltip == null ? button : Tooltip(message: tooltip!, child: button);
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
        height: EditorTheme.section,
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

class EditorMenuItem<T> extends PopupMenuItem<T> {
  const EditorMenuItem({
    super.key,
    super.value,
    super.enabled,
    required super.child,
  }) : super(height: 20, padding: const EdgeInsets.symmetric(horizontal: 8));
  @override
  PopupMenuItemState<T, EditorMenuItem<T>> createState() =>
      _EditorMenuItemState<T>();
}

class _EditorMenuItemState<T> extends PopupMenuItemState<T, EditorMenuItem<T>> {
  bool hovered = false;
  @override
  Widget build(BuildContext context) => MouseRegion(
    onEnter: (_) => setState(() => hovered = true),
    onExit: (_) => setState(() => hovered = false),
    child: Theme(
      data: Theme.of(context).copyWith(
        hoverColor: const Color(0xffaedce8),
        highlightColor: const Color(0xffaedce8),
      ),
      child: super.build(context),
    ),
  );
  @override
  Widget buildChild() => DefaultTextStyle.merge(
    style: TextStyle(
      fontSize: 11,
      color: !widget.enabled
          ? const Color(0xff888888)
          : hovered
          ? const Color(0xff172126)
          : EditorTheme.ink,
    ),
    child: widget.child!,
  );
}
