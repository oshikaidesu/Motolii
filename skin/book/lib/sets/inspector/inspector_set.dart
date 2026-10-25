import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_effects_effect_card.dart';
import 'package:lab_book/sets/inspector/inspector_effects_k_effects.dart';
import 'package:lab_book/sets/inspector/inspector_parts_insp_host.dart';
import 'package:lab_book/sets/inspector/inspector_rows_prop_row.dart';
import 'package:lab_book/sets/inspector/inspector_rows_sect.dart';
import 'package:widgetbook/widgetbook.dart';

import '../../kit.dart';
import 'inspector_host_gallery.dart';
import 'inspector_idea_gallery.dart';
import 'inspector_parts.dart';
part 'inspector_set_2.dart';

// set: inspector_set. What the Inspector must do comes from research/intents/inspector.md.
// Every open question is a knob (default = the After Effects convention) and the behaviour really follows it:
//   Editorial A/B/C, Modifier rule, Drag direction, Reset gesture, Layout (tabs | one column), Show maybe-features.
// Built (Yes): A1 A2 A3 A4 A6 A7, B1 B2 B4 B5 B6 B7 B8, C1-C5 C7-C10, D1 D2 D3, E1-E4, F1-F5 (drag-reorder is Move earlier/later), G1-G3, H1 H2, I1 I2.
// Behind 'Show maybe-features' (Maybe): A5 layout, C6 precision ladder, D4 group key, H3 search, I3 anchor hover preview, I4 relation badge on a row, A6 composition settings.
// Left out: B3 (pad duplicates the Stage; card says No), the four-mode strip of B7, per-section colours of F3.

Cfg _cfg(BuildContext c, {bool lockedKnob = false}) => Cfg(
  ed: c.knobs.object.dropdown<Editorial>(
    label: 'Editorial',
    options: Editorial.values,
    initialOption: Editorial.a,
    labelBuilder: (e) => switch (e) {
      Editorial.a => 'A  ruled column',
      Editorial.b => 'B  quiet tone',
      Editorial.c => 'C  scale contrast',
    },
  ),
  mod: c.knobs.object.dropdown<ModRule>(
    label: 'Modifier rule',
    options: ModRule.values,
    initialOption: ModRule.shiftBig,
    labelBuilder: (m) => m == ModRule.shiftBig
        ? 'Shift x10 + Alt fine'
        : 'Shift fine + Ctrl x10',
  ),
  rightUp: c.knobs.object.dropdown<bool>(
    label: 'Drag direction',
    options: const [true, false],
    initialOption: true,
    labelBuilder: (b) => b ? 'right = up' : 'right = down',
  ),
  reset: c.knobs.object.dropdown<ResetRule>(
    label: 'Reset gesture',
    options: ResetRule.values,
    initialOption: ResetRule.dbl,
    labelBuilder: (r) => switch (r) {
      ResetRule.dbl => 'double-click',
      ResetRule.del => 'Delete key',
      ResetRule.arrow => 'reset arrow',
      ResetRule.both => 'double-click + arrow',
      ResetRule.live => 'Ableton: Delete or double-click, type to start',
    },
  ),
  tabs: c.knobs.object.dropdown<bool>(
    label: 'Layout',
    options: const [true, false],
    initialOption: true,
    labelBuilder: (b) => b ? 'tabs' : 'one scrolling column',
  ),
  maybe: c.knobs.boolean(label: 'Show maybe-features'),
  look: lookKnob(c),
  locked: lockedKnob ? c.knobs.boolean(label: 'Locked layer') : false,
);

Widget _host(
  BuildContext c,
  Kind k,
  Widget child, {
  int count = 1,
  int effects = 2,
  bool lockedKnob = false,
}) => InspHost(
  key: ValueKey('$k$count$effects'),
  cfg: _cfg(c, lockedKnob: lockedKnob),
  kind: k,
  count: count,
  effects: effects,
  child: child,
);

Kind _kindKnob(BuildContext c, {Kind initial = Kind.shape}) =>
    c.knobs.object.dropdown<Kind>(
      label: 'Selection',
      options: Kind.values,
      initialOption: initial,
      labelBuilder: (k) => switch (k) {
        Kind.none => 'nothing',
        Kind.several => 'several',
        _ => k.name,
      },
    );

/// A board wider and taller than the window: scroll it either way.
class _BothWays extends StatelessWidget {
  const _BothWays(this.child);
  final Widget child;
  @override
  Widget build(BuildContext context) => SingleChildScrollView(
    child: SingleChildScrollView(scrollDirection: Axis.horizontal, child: child),
  );
}

/// The rules the knobs chose, as a caption: what the owner is comparing.
class _Rules extends StatelessWidget {
  const _Rules();
  @override
  Widget build(BuildContext context) {
    final c = Ctx.of(context).cfg;
    return Cap(
      'Drag ${c.rightUp ? 'right = up' : 'right = down'} · ${c.modWords} · slide sideways: right = more, at the drag rate · arrows step · Esc cancels · reset: ${switch (c.reset) {
        ResetRule.dbl => 'double-click',
        ResetRule.del => 'Delete key',
        ResetRule.arrow => 'arrow at row end',
        ResetRule.both => 'double-click or arrow',
       ResetRule.live => 'Delete or double-click (Ableton)',}}',
    );
  }
}

Widget _panel(
  BuildContext c,
  Kind k, {
  double w = 372,
  double h = 720,
  int count = 1,
  int effects = 2,
}) => _host(
  c,
  k,
  InspectorPanel(width: w, height: h),
  count: count,
  effects: effects,
  lockedKnob: true,
);
