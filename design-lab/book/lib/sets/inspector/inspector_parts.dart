// Inspector parts. What the Inspector must DO comes from research/intents/inspector.md (Yes cards built, Maybe cards behind cfg.maybe).
// How it LOOKS is the lab's (DESIGN.md): hairlines, type carries hierarchy, grey levels, accent only C.mode. The old Motolii is not a source.
// Open questions are Cfg fields (knobs in inspector_set.dart); every behaviour below follows them.
import 'dart:async';
import 'dart:math' as math;

import 'package:flutter/foundation.dart' show mapEquals;
import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../../parts/controls.dart';
import '../../parts/glyphs.dart';
import '../../parts/relations.dart';
import '../../tokens.dart';
import 'inspector_direct_transform.dart';
import 'param_infer.dart';

export 'param_infer.dart';

part 'inspector_state.dart';
part 'inspector_number_field.dart';
part 'inspector_small.dart';
part 'inspector_rows.dart';
part 'inspector_badges.dart';
part 'inspector_choosers_header.dart';
part 'inspector_transform_rows.dart';
part 'inspector_anchor.dart';
part 'inspector_material.dart';
part 'inspector_effects.dart';
part 'inspector_param_controls.dart';
part 'inspector_camera_layout.dart';
part 'inspector_relations.dart';
part 'inspector_panel.dart';
part 'inspector_host_gallery.dart';
part 'inspector_host_placement.dart';
part 'inspector_host_paths.dart';
part 'inspector_host_text.dart';
part 'inspector_host_layer.dart';
part 'inspector_host_particles.dart';
part 'inspector_host_scene.dart';
part 'inspector_idea_gallery.dart';
part 'inspector_idea_text.dart';
part 'inspector_idea_paths.dart';
part 'inspector_idea_layout.dart';
part 'inspector_idea_placement.dart';
part 'inspector_idea_layer.dart';
part 'inspector_panel_link.dart';
part 'inspector_advanced.dart';

// ---- 0. open values -------------------------------------------------------------------------------------------------
// [open] a value cell's height (the lab's ValueWell is 22; rows are 24 on the 4px rhythm; B uses 22).
const _cellH = 22.0;
// [open] one narrowing rule for the whole panel (H1): a cell narrower than _wUnit drops its unit, narrower than _wTight drops decimals (never to a false 0).
const _wUnit = 72.0, _wTight = 60.0;
// [open] a third value stacks under the first two when a cell would be narrower than this.
const _stack3 = 72.0;
// [open] the label column (wide / narrow panel).
const _labelW = 88.0, _labelWNarrow = 76.0, _labelWPop = 104.0;
// [open] the width of the key mark's and reset mark's columns.
const _keyW = 22.0, _resetW = 18.0;
// [open] axis colours: X / Y / Z ticks reuse relation-family hues (the only hue outside relations).
const _axX = Color(0xFFE974AB), _axY = Color(0xFF7DD5B1), _axZ = Color(0xFF4781E5);
const _axes = [_axX, _axY, _axZ];
// [open] two clicks within this are a double click.
const _dbl = Duration(milliseconds: 320);

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
  const Ctx({super.key, required this.cfg, required this.doc, required super.child});
  final Cfg cfg;
  final Doc doc;
  static Ctx of(BuildContext c) => c.dependOnInheritedWidgetOfExactType<Ctx>()!;
  static Ctx read(BuildContext c) => c.getInheritedWidgetOfExactType<Ctx>()!;
  @override
  bool updateShouldNotify(Ctx old) => true;
}

/// Owns the mock document and re-provides it whenever it changes. A different kind or count makes a new document (give it a key).
class InspHost extends StatefulWidget {
  const InspHost({
    super.key,
    required this.cfg,
    required this.kind,
    this.count = 1,
    this.effects = 2,
    this.title,
    this.onRoute,
    this.values = const {},
    this.strs = const {},
    required this.child,
  });
  final Cfg cfg;
  final Kind kind;
  final int count, effects;
  final Widget child;

  /// The selected layer's name, when the host knows it.
  final String? title;

  /// A panel link asked for a panel (`Browser › Fonts`, `Ease`); the host opens it.
  final ValueChanged<String>? onRoute;

  /// Values the host owns (the layer's transform at this frame): written in silently when they change, so edits made here last until then.
  final Map<String, double> values;
  final Map<String, String> strs;
  @override
  State<InspHost> createState() => _InspHostState();
}

class _InspHostState extends State<InspHost> {
  late Doc doc = _make();
  Doc _make() => Doc(widget.kind, count: widget.count, effects: widget.effects)
    ..title = widget.title
    ..v.addAll(widget.values)
    ..s2.addAll(widget.strs)
    ..addListener(_r);
  void _r() {
    final r = doc.s2['route'], go = widget.onRoute;
    if (r != null && go != null) {
      doc.s2.remove('route');
      go(r);
    }
    if (mounted) setState(() {});
  }

  @override
  void didUpdateWidget(InspHost o) {
    super.didUpdateWidget(o);
    if (o.kind != widget.kind || o.count != widget.count || o.effects != widget.effects) {
      doc.dispose();
      doc = _make();
    } else {
      if (!mapEquals(o.values, widget.values)) doc.v.addAll(widget.values);
      if (!mapEquals(o.strs, widget.strs)) doc.s2.addAll(widget.strs);
    }
    doc.title = widget.title;
  }

  @override
  void dispose() {
    doc.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Ctx(cfg: widget.cfg, doc: doc, child: widget.child);
}
