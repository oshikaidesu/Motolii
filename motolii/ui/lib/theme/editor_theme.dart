import 'package:flutter/widgets.dart';
import 'package:flutter/material.dart'
    show Theme, ThemeData, ThemeExtension, ColorScheme;

import '../controls/leaves.dart';
import 'metrics.dart';

/// Colours of what is drawn rather than laid out — the camera's line on the Stage, the transparency grid — carried by
/// [EditorTheme] above the window so a painter's colours come from the theme the way a widget's do. A widget passes [of]
/// into the painter it builds; code with no context reads [dark], the one look the window has.
@immutable
class EditorInk {
  const EditorInk({
    required this.camera,
    required this.checkerLight,
    required this.checkerDark,
  });

  /// A camera's line on the Stage.
  final Color camera;

  /// The transparency grid: the picture editors' two greys.
  final Color checkerLight, checkerDark;

  static const dark = EditorInk(
    camera: Color(0xff8ed9e6),
    checkerLight: Color(0xff8c8c8c),
    checkerDark: Color(0xff666666),
  );

  static EditorInk of(BuildContext context) => EditorTheme.of(context).drawing;

  EditorInk lerp(EditorInk? other, double t) {
    if (other == null) return this;
    return EditorInk(
      camera: Color.lerp(camera, other.camera, t)!,
      checkerLight: Color.lerp(checkerLight, other.checkerLight, t)!,
      checkerDark: Color.lerp(checkerDark, other.checkerDark, t)!,
    );
  }
}

@immutable
class EditorTheme extends ThemeExtension<EditorTheme> {
  const EditorTheme._({
    required this.name,
    required this.colors,
    required this.drawing,
  });
  final String name;
  final Map<String, Color> colors;
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
      'hoverWash': Color(0x0affffff),
      'focusWash': Color(0x1fffffff),
      'inkDisabled': Color(0x61ffffff),
      'tickActive': Color(0x61000000),
      'tickInactive': Color(0x61ffffff),
      'tooltip': Color(0xe6ffffff),
      'selection': Color(0x6659c9df),
      'caret': Color(0xffffbc53),
    },
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
  Color get hoverWash => colors['hoverWash']!;
  Color get focusWash => colors['focusWash']!;
  Color get inkDisabled => colors['inkDisabled']!;
  Color get tickActive => colors['tickActive']!;
  Color get tickInactive => colors['tickInactive']!;
  Color get tooltip => colors['tooltip']!;
  Color get selection => colors['selection']!;
  Color get caret => colors['caret']!;
  static const white = Color(0xffffffff),
      black = Color(0xff000000),
      clear = Color(0x00000000);
  static const hoverLift = .08, pressedLift = .10;
  static const fontFamily = 'Inter';
  TextStyle get text => TextStyle(
    inherit: false,
    fontFamily: fontFamily,
    fontSize: Dn.labelSize,
    fontWeight: FontWeight.w500,
    color: ink,
    textBaseline: TextBaseline.alphabetic,
  );
  IconThemeData get icon => IconThemeData(size: Surface.px(14), color: ink);
  static EdgeInsets get menuPadding => EdgeInsets.symmetric(vertical: Surface.labelGap);
  static const menuMinWidth = 112.0;
  static EdgeInsets get menuRowPadding => EdgeInsets.symmetric(horizontal: Surface.px(8));

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
    EditorInk? drawing,
  }) => EditorTheme._(
    name: name ?? this.name,
    colors: Map.unmodifiable({...this.colors, ...?colors}),
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
      drawing: drawing.lerp(other.drawing, t),
    );
  }
}

/// A tooltip: each one that stands costs a hover region, a long-press detector and a semantics node, so a control
/// that has no message does not wrap itself in one.
class EditorTooltip extends StatelessWidget {
  const EditorTooltip({super.key, required this.message, required this.child});
  final String message;
  final Widget child;
  @override
  Widget build(BuildContext context) => RawTooltip(
    semanticsTooltip: message,
    ignorePointer: true,
    touchDelay: const Duration(milliseconds: 1500),
    tooltipBuilder: (context, animation) => FadeTransition(
      opacity: animation,
      child: EditorTooltipSheet(message: message),
    ),
    child: child,
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
        height: Surface.workRow - 2,
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
