// Inspector parts. What the Inspector must DO comes from research/intents/inspector.md (Yes cards built, Maybe cards behind cfg.maybe).
// How it LOOKS is the lab's (DESIGN.md): hairlines, type carries hierarchy, grey levels, accent only C.mode. The old Motolii is not a source.
// Open questions are Cfg fields (knobs in inspector_set.dart); every behaviour below follows them.

import 'package:flutter/widgets.dart';

import '../../theme.dart';
import '../../tokens.dart';
import 'inspector_state.dart';

export 'inspector_advanced.dart';
export 'inspector_anchor.dart';
export 'inspector_badges.dart';
export 'inspector_camera_layout.dart';
export 'inspector_choosers_header.dart';
export 'inspector_copies.dart';
export 'inspector_effects.dart';
export 'inspector_layer_tiles.dart';
export 'inspector_light.dart';
export 'inspector_material.dart';
export 'inspector_number_field.dart';
export 'inspector_panel.dart';
export 'inspector_panel_link.dart';
export 'inspector_param_controls.dart';
export 'inspector_relations.dart';
export 'inspector_rows.dart';
export 'inspector_small.dart';
export 'inspector_state.dart';
export 'inspector_transform_rows.dart';
export 'inspector_value_marks.dart';
export 'param_infer.dart';

// ---- 0. open values -------------------------------------------------------------------------------------------------
// [open] a value cell's height (the lab's ValueWell is 22; rows are 24 on the 4px rhythm; B uses 22).
const cellH = 22.0;
// [open] one narrowing rule for the whole panel (H1): a cell narrower than _wUnit drops its unit, narrower than _wTight drops decimals (never to a false 0).
const wUnit = 72.0, wTight = 60.0;
// [open] a third value stacks under the first two when a cell would be narrower than this.
const stack3 = 72.0;
// [open] the label column (wide / narrow panel).
const labelW = 88.0, labelWNarrow = 76.0, labelWPop = 104.0;
// [open] the width of the key mark's and reset mark's columns.
const keyW = 22.0, resetW = 18.0;
// [open] axis colours: X / Y / Z ticks reuse relation-family hues (the only hue outside relations).
Color get axX => ColorTheme.of('AxisX', const Color(0xFFE974AB)); // (the skin's axis colours: Blender's names)
Color get axY => ColorTheme.of('AxisY', const Color(0xFF7DD5B1));
Color get axZ => ColorTheme.of('AxisZ', const Color(0xFF4781E5));
List<Color> get axesParts => [axX, axY, axZ];
// [open] two clicks within this are a double click.
const dblParts = Duration(milliseconds: 320);

/// Set by a row: its controls are off (locked, driven or disabled) and whether the panel is narrow (units go everywhere at once).
class RowOff extends InheritedWidget {
  const RowOff({super.key, required this.off, required this.narrow, required super.child});
  final bool off, narrow;
  static RowOff? maybe(BuildContext c) => c.dependOnInheritedWidgetOfExactType<RowOff>();
  static bool offOf(BuildContext c) => maybe(c)?.off ?? false;
  @override
  bool updateShouldNotify(RowOff o) => o.off != off || o.narrow != narrow;
}

/// For controls that cannot recolour themselves (Segmented): off = a fixed dim and no pointer.
class Dis extends StatelessWidget {
  const Dis({super.key, required this.child});
  final Widget child;
  @override
  Widget build(BuildContext context) {
    final off = RowOff.offOf(context) || Ctx.of(context).cfg.locked;
    return IgnorePointer(
      ignoring: off,
      child: Opacity(opacity: off ? .6 : 1, child: child),
    );
  }
}

/// The bold skin of the direct-manipulation prototype: rounded cards, filled badges, one lime fill for "selected".
class Pop extends InheritedWidget {
  const Pop({super.key, required super.child});
  static bool on(BuildContext c) => c.getInheritedWidgetOfExactType<Pop>() != null;
  static Color get accent => Accent.fill; // (the skin's: one accent for the window)
  static Color get onAccent => Accent.on;
  static Color get keyDot => Accent.key;

  /// The accent as a line or text on a ground (the skin's, made deep enough to read on white). Fills stay [accent].
  static Color get accentInk => Accent.ink;
  static Color get card => Grey.g13;
  static Color get well => Grey.g15;

  /// The one spacing rule: cards are inset [inset] at the sides and [insetY] top and bottom,
  /// a card's title sits [titleGap] above its content, and cards stand [gap] apart and from the panel edge.
  /// Tight on purpose: the panel is read as a list, so more rows on screen beats air.
  /// Shape carries meaning: containers (cards, the canvas) are square and tile edge to edge; what you touch or read (fields, chips, badges, switches) is round.
  /// Colour is flat and opaque, never a tint laid over grey.
  static const inset = 8.0, insetY = 6.0, titleGap = 4.0, gap = 2.0, titleH = 16.0;
  static const row = 24.0, cell = 22.0;

  /// A parameter block: name on top, value large at the bottom, its picture in the top-right corner; text always starts at the same x.
  static double get tile => Shape.z(46); // (the skin's Size and Spacing)
  static double get tileMin => Shape.z(112);
  static double get glyph => Shape.z(22);
  static double get tileGap => Shape.s(4);
  static double get tilePad => Shape.s(8);

  /// Each card wears its kind as a colour, so a folded stack reads by hue before it reads by word.
  /// The layer's own cards are muted; effects are saturated, since they are the ones that pile up.
  static Color get toneTransform => ColorTheme.of('ToneTransform', const Color(0xFF6F9CF0));
  static Color get toneLayer => ColorTheme.of('ToneLayer', const Color(0xFF9AA4B2));
  static Color get toneText => ColorTheme.of('ToneText', const Color(0xFFD9D2C3));
  static Color get toneFill => ColorTheme.of('ToneFill', const Color(0xFFD9A47A));
  static Color get toneBlend => ColorTheme.of('ToneBlend', const Color(0xFF7FB8B0));

  /// Host features (no shader): one hue per family, so a Repeater and a Trim Paths never read as the same kind of card.
  static Color get tonePlace => ColorTheme.of('TonePlace', const Color(0xFFF08A5D));
  static Color get tonePath => ColorTheme.of('TonePath', const Color(0xFF5CC8E8));
  static Color get toneSolid => ColorTheme.of('ToneSolid', const Color(0xFFC9A0DC));
  static Color get toneTextAnim => ColorTheme.of('ToneTextAnim', const Color(0xFFE8C76A));
  static Color get toneMask => ColorTheme.of('ToneMask', const Color(0xFF8FBF73));
  static Color get toneTime => ColorTheme.of('ToneTime', const Color(0xFFE58FB4));
  static Color get toneLayout => ColorTheme.of('ToneLayout', const Color(0xFF7C9CBF));
  static Color get toneParticles => ColorTheme.of('ToneParticles', const Color(0xFFF2A65A));
  static Color get toneCamera => ColorTheme.of('ToneCamera', const Color(0xFFA7B4C8));
  static Color get toneOutput => ColorTheme.of('ToneOutput', const Color(0xFF6FD6C2));
  static Color get toneLink => ColorTheme.of('ToneLink', const Color(0xFFB0A6F0));
  @override
  bool updateShouldNotify(Pop old) => false;
}

class Ctx extends InheritedWidget {
  const Ctx({super.key, required this.cfg, required this.doc, this.tick, required super.child});
  final Cfg cfg;
  final Doc doc;

  /// Bumped by the host when more than numbers changed (its shape, its words, a pick): only then is everything that
  /// reads the inspector built again; a number alone tells the field that shows it (Doc.watch).
  final int? tick; // (none: a host that does not count, everything again each time, as before)
  static Ctx of(BuildContext c) => c.dependOnInheritedWidgetOfExactType<Ctx>()!;
  static Ctx read(BuildContext c) => c.getInheritedWidgetOfExactType<Ctx>()!;
  @override
  bool updateShouldNotify(Ctx old) => tick == null || old.tick != tick || old.doc != doc || old.cfg != cfg;
}
