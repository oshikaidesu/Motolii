// Inspector parts. What the Inspector must DO comes from research/intents/inspector.md (Yes cards built, Maybe cards behind cfg.maybe).
// How it LOOKS is the lab's (DESIGN.md): hairlines, type carries hierarchy, grey levels, accent only C.mode. The old Motolii is not a source.
// Open questions are Cfg fields (knobs in inspector_set.dart); every behaviour below follows them.

import 'package:flutter/widgets.dart';

import '../../tokens.dart';
import 'inspector_state.dart';

export 'inspector_advanced.dart';
export 'inspector_anchor.dart';
export 'inspector_badges.dart';
export 'inspector_camera_layout.dart';
export 'inspector_choosers_header.dart';
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
const axX = Color(0xFFE974AB), axY = Color(0xFF7DD5B1), axZ = Color(0xFF4781E5);
const axesParts = [axX, axY, axZ];
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
  static const accent = Color(0xFFB3C66B), onAccent = Color(0xFF15170D), keyDot = Color(0xFFF2C94C);

  /// The accent as a line or text on a ground: lime vanishes on white, so the light shade inks it darker. Fills stay [accent].
  static Color get accentInk => Grey.light ? const Color(0xFF6B7A1E) : accent;
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
  static const tile = 46.0, tileMin = 112.0, tileGap = 4.0, glyph = 22.0, tilePad = 8.0;

  /// Each card wears its kind as a colour, so a folded stack reads by hue before it reads by word.
  /// The layer's own cards are muted; effects are saturated, since they are the ones that pile up.
  static const toneTransform = Color(0xFF6F9CF0), toneLayer = Color(0xFF9AA4B2), toneText = Color(0xFFD9D2C3);
  static const toneFill = Color(0xFFD9A47A), toneBlend = Color(0xFF7FB8B0);

  /// Host features (no shader): one hue per family, so a Repeater and a Trim Paths never read as the same kind of card.
  static const tonePlace = Color(0xFFF08A5D), tonePath = Color(0xFF5CC8E8), toneSolid = Color(0xFFC9A0DC);
  static const toneTextAnim = Color(0xFFE8C76A), toneMask = Color(0xFF8FBF73), toneTime = Color(0xFFE58FB4);
  static const toneLayout = Color(0xFF7C9CBF), toneParticles = Color(0xFFF2A65A), toneCamera = Color(0xFFA7B4C8);
  static const toneOutput = Color(0xFF6FD6C2), toneLink = Color(0xFFB0A6F0);
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
