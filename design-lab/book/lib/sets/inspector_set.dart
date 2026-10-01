import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import '../kit.dart';
import '../tokens.dart';
import 'inspector_parts.dart';

// set: inspector_set. What the Inspector must do comes from research/intents/inspector.md.
// Every open question is a knob (default = the After Effects convention) and the behaviour really follows it:
//   Editorial A/B/C, Modifier rule, Drag direction, Reset gesture, Layout (tabs | one column), Show maybe-features.
// Built (Yes): A1 A2 A3 A4 A6 A7, B1 B2 B4 B5 B6 B7 B8, C1-C5 C7-C10, D1 D2 D3, E1-E4, F1-F5 (drag-reorder is Move earlier/later), G1-G3, H1 H2, I1 I2.
// Behind 'Show maybe-features' (Maybe): A5 layout, C6 precision ladder, D4 group key, H3 search, I3 anchor hover preview, I4 relation badge on a row, A6 composition settings.
// Left out: B3 (pad duplicates the Stage; card says No), the four-mode strip of B7, per-section colours of F3.

Cfg _cfg(BuildContext c, {bool lockedKnob = false}) => Cfg(
      ed: c.knobs.object.dropdown<Editorial>(label: 'Editorial', options: Editorial.values, initialOption: Editorial.a, labelBuilder: (e) => switch (e) { Editorial.a => 'A  ruled column', Editorial.b => 'B  quiet tone', Editorial.c => 'C  scale contrast' }),
      mod: c.knobs.object.dropdown<ModRule>(label: 'Modifier rule', options: ModRule.values, initialOption: ModRule.shiftBig, labelBuilder: (m) => m == ModRule.shiftBig ? 'Shift x10 + Alt fine' : 'Shift fine + Ctrl x10'),
      rightUp: c.knobs.object.dropdown<bool>(label: 'Drag direction', options: const [true, false], initialOption: true, labelBuilder: (b) => b ? 'right = up' : 'right = down'),
      reset: c.knobs.object.dropdown<ResetRule>(label: 'Reset gesture', options: ResetRule.values, initialOption: ResetRule.dbl, labelBuilder: (r) => switch (r) { ResetRule.dbl => 'double-click', ResetRule.del => 'Delete key', ResetRule.arrow => 'reset arrow', ResetRule.both => 'double-click + arrow' }),
      tabs: c.knobs.object.dropdown<bool>(label: 'Layout', options: const [true, false], initialOption: true, labelBuilder: (b) => b ? 'tabs' : 'one scrolling column'),
      maybe: c.knobs.boolean(label: 'Show maybe-features'),
      look: lookKnob(c),
      locked: lockedKnob ? c.knobs.boolean(label: 'Locked layer') : false,
    );

Widget _host(BuildContext c, Kind k, Widget child, {int count = 1, int effects = 2, bool lockedKnob = false}) =>
    InspHost(key: ValueKey('$k$count$effects'), cfg: _cfg(c, lockedKnob: lockedKnob), kind: k, count: count, effects: effects, child: child);

Kind _kindKnob(BuildContext c, {Kind initial = Kind.shape}) => c.knobs.object.dropdown<Kind>(label: 'Selection', options: Kind.values, initialOption: initial, labelBuilder: (k) => switch (k) { Kind.none => 'nothing', Kind.several => 'several', _ => k.name });

/// The rules the knobs chose, as a caption: what the owner is comparing.
class _Rules extends StatelessWidget {
  const _Rules();
  @override
  Widget build(BuildContext context) {
    final c = Ctx.of(context).cfg;
    return Cap('Drag ${c.rightUp ? 'right = up' : 'right = down'} · ${c.modWords} · wheel: one notch = one unit · arrows step · Esc cancels · reset: ${switch (c.reset) { ResetRule.dbl => 'double-click', ResetRule.del => 'Delete key', ResetRule.arrow => 'arrow at row end', ResetRule.both => 'double-click or arrow' }}');
  }
}

Widget _panel(BuildContext c, Kind k, {double w = 372, double h = 720, int count = 1, int effects = 2}) =>
    _host(c, k, InspectorPanel(width: w, height: h), count: count, effects: effects, lockedKnob: true);

WidgetbookComponent inspectorSetSet() => WidgetbookComponent(name: 'inspector', useCases: [
      uc('Header — kind, name, Animate all', (c) {
        final k = _kindKnob(c);
        return _host(c, k, Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          if (k == Kind.none) const EmptyState() else const InspHeader(),
          const Cap('One subject at a time, named in the panel (A1). Animate all: while on, an edit writes a key at the playhead (D2). The state is a word, not colour alone.'),
        ]), count: 3, lockedKnob: true);
      }, width: 372),
      uc('Number field — the core control', (c) => _host(c, Kind.shape, Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            const Sect(first: true, title: 'Try every gesture', children: [
              PropRow(id: 'op', label: 'Opacity', cells: [NumField(id: 'op', unit: '%', min: 0, max: 100, perPx: .5)]),
              PropRow(id: 'pos', label: 'Position', ids: ['pos.x', 'pos.y'], cells: [NumField(id: 'pos.x', label: 'X', unit: 'px', decimals: 1, axis: Color(0xFFE974AB)), NumField(id: 'pos.y', label: 'Y', unit: 'px', decimals: 1, axis: Color(0xFF7DD5B1))]),
              PropRow(id: 'rot', label: 'Rotation', cells: [NumField(id: 'rot.z', unit: '°', decimals: 1, perPx: .5, turns: true)]),
              PropRow(id: 'depth', label: 'Depth', cells: [NumField(id: 'depth', unit: 'px', zeroWord: 'Flat')]),
            ]),
            const _Rules(),
            const Cap('Drag the field. Double-click types (with reset = double-click, a single click types after the double-click window, so no editor flashes). Enter types. Release = one undo step. Rotation 450 stays 450. A bounded value shows a fill; an open one does not. Hover and wheel steps one unit. Tab goes to the next field. Maybe: drag up/down changes precision.'),
          ]), lockedKnob: true), width: 372),
      uc('Number field — states and narrowing', (c) {
        final w = c.knobs.double.slider(label: 'Cell width', initialValue: 140, min: 48, max: 240);
        return _host(c, Kind.several, Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          SizedBox(width: w, child: const Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            NumField(id: 'op', label: 'Opacity', unit: '%', min: 0, max: 100),
            SizedBox(height: 6),
            NumField(id: 'pos.x', label: 'Mixed', unit: 'px', decimals: 1),
            SizedBox(height: 6),
            NumField(id: 'depth', label: 'Depth', unit: 'px', zeroWord: 'Flat'),
            SizedBox(height: 6),
            NumField(id: 'rot.z', label: 'Rot', unit: '°', decimals: 1, turns: true),
            SizedBox(height: 6),
            NumField(id: 'scale.y', label: 'Scale', unit: '%', decimals: 2, enabled: false),
          ])),
          const Cap('Narrow rule (one for the whole panel): below ~92 px the unit goes, below ~60 px the decimals go; a value that is not zero never reads 0 and no digit is cut (C9). Mixed shows —; a scrub on it moves every layer by the same amount, a typed number sets all (A7). The last field is disabled.'),
          const _Rules(),
        ]), count: 3);
      }, width: 372),
      uc('Key mark and reset mark', (c) => _host(c, Kind.shape, Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            const Row(children: [
              Expanded(child: Cap('Not animated')),
              Expanded(child: Cap('Animated, no key here')),
              Expanded(child: Cap('Keyed at the playhead')),
            ]),
            const Row(children: [
              Expanded(child: Align(alignment: Alignment.centerLeft, child: KeyMark(state: KeyS.off))),
              Expanded(child: Align(alignment: Alignment.centerLeft, child: KeyMark(state: KeyS.anim))),
              Expanded(child: Align(alignment: Alignment.centerLeft, child: KeyMark(state: KeyS.at))),
            ]),
            const SizedBox(height: 8),
            const Sect(first: true, title: 'Live rows', children: [
              PropRow(id: 'pos', label: 'Position', ids: ['pos.x', 'pos.y'], cells: [NumField(id: 'pos.x', label: 'X', decimals: 1), NumField(id: 'pos.y', label: 'Y', decimals: 1)]),
              PropRow(id: 'rot', label: 'Rotation', cells: [NumField(id: 'rot.z', unit: '°', decimals: 1)]),
              PropRow(id: 'op', label: 'Opacity', cells: [NumField(id: 'op', unit: '%', min: 0, max: 100)]),
            ]),
            const Cap('The three states differ by shape (hollow diamond / diamond with its link line / filled playhead diamond), not by tone alone (D1). Click keys at the playhead; click on a key removes it. Right-click a row for Key this frame / Remove key / Remove animation, with the reason when one is unavailable (D3). Changed values turn their label brighter; the reset arrow (knob) appears only on those (E4).'),
          ]), lockedKnob: true), width: 372),
      uc('Transform group', (c) => _host(c, Kind.shape, Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            const TransformGroup(),
            const Cap('Fixed order, every row has the same columns so X/Y/Z line up (B1). 2D / 2.5D / 3D reveals Z, the other rotation axes and Depth without changing a value (B2). A third value stacks under the first two when the cell is narrow (H1). Axis ticks reuse family hues. [open] axis colours'),
          ]), lockedKnob: true), width: 372),
      uc('Scale with linked proportions', (c) => _host(c, Kind.shape, Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            const Sect(first: true, title: 'Scale', children: [ScaleRow(), SpaceRow()]),
            const Cap('Link on: a factor applies to every axis (set X to double and Y doubles). Toggling Link changes nothing in the artwork. Off: only the touched axis moves. If an axis is 0, typing a value sets both (E1).'),
          ]), lockedKnob: true), width: 372),
      uc('Anchor and parent', (c) => _host(c, Kind.shape, Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            const Sect(first: true, title: 'Pivot and parent', children: [AnchorRow(), ParentRow()]),
            const Cap('Nine points, one click (B4); a typed value outside them reads Custom. Hover preview on the Stage is a maybe-card (I3). The parent is chosen from a searchable list, never by cycling (G3). Neither moves the layer visually.'),
          ]), lockedKnob: true), width: 372),
      uc('Opacity and depth', (c) => _host(c, Kind.shape, Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            const Sect(first: true, title: 'Opacity and depth', children: [SpaceRow(), OpacityRow(), DepthRow()]),
            const Cap('Opacity reads 0-100 %, display only. Depth appears only outside 2D (switch Space to 2.5D) and reads Flat at 0 (B6, E3).'),
          ]), lockedKnob: true), width: 372),
      uc('Blend, ghost, clip to below', (c) => _host(c, Kind.shape, Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            const BlendRows(first: true),
            const Cap('How this layer composites with those below, in one small block (G1). Blend opens a searchable chooser; Ghost and Clip to below are On/Off words with a switch (G2). Not shown for a camera.'),
          ]), lockedKnob: true), width: 372),
      uc('Effect — parameters, enable, Advanced', (c) => _host(c, Kind.shape, Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            EffectCard(kEffects.first, first: true),
            const Cap('Heroes first, the rest behind Advanced with a count (F2). Two small controls pair up when there is room (F4). The switch bypasses the effect and dims its body; the name folds it; More: move, reset, randomise, remove (F1, F5).'),
          ]), lockedKnob: true), width: 372),
      uc('Effects stack', (c) => _host(c, Kind.shape, Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            const EffectsStack(),
            const Cap('Order matters and is visible; More moves an effect earlier or later (drag-to-reorder is not built in this lab). Maybe: search flattens the groups.'),
          ]), effects: 4, lockedKnob: true), width: 372, height: 720),
      uc('Camera group', (c) => _host(c, Kind.camera, Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            const CameraGroup(),
            const Cap('Target, Orbit, Framing, Roll (A4). Pick a target layer: the point fields disable and say why. Maybe: a group key mark keys a whole group (D4).'),
          ]), lockedKnob: true), width: 372),
      uc('Layout group (maybe)', (c) {
        final k = _cfg(c).maybe;
        return _host(c, Kind.group, Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          if (k) const LayoutGroup(first: true) else const Cap('A5 is a Maybe card (does the new tool have auto-layout?). Turn on "Show maybe-features".'),
          const Cap('Layout off dims everything but its switch. The size number counts only when Size is Fixed. A child can opt out.'),
        ]), lockedKnob: true);
      }, width: 372),
      uc('Relations section', (c) => _host(c, Kind.group, Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            const RelationsBlock(first: true),
            const Cap('One relation open as its card (the lab\'s RelationCard), the others one row each; clicking a row opens it. Hue only on the relation mark. Maybe: a driven row says who drives it (I4), see the next case.'),
          ]), lockedKnob: true), width: 372),
      uc('Driven value (maybe)', (c) {
        final on = _cfg(c).maybe;
        return _host(c, Kind.shape, Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          Sect(first: true, title: 'Transform', children: [
            const PosRow(),
            PropRow(id: 'rot', label: 'Rotation', rel: Fam.face, relName: 'Jewel 02', cells: const [NumField(id: 'rot.z', unit: '°', decimals: 1)]),
            PropRow(id: 'scale', label: 'Scale', rel: Fam.stagger, relName: 'Field', ids: const ['scale.x', 'scale.y'], cells: const [NumField(id: 'scale.x', label: 'X', unit: '%'), NumField(id: 'scale.y', label: 'Y', unit: '%')]),
          ]),
          Cap(on ? 'A mark names the driver; the value cannot be edited here and says so (I4).' : 'I4 is a Maybe card. Turn on "Show maybe-features".'),
        ]), lockedKnob: true);
      }, width: 372),
      uc('Text layer group', (c) => _host(c, Kind.text, Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            const TextGroup(),
            const FillStroke(),
            const Cap('Text is edited as text and applies when you leave the box; Esc restores. Font and colours hand over to their tool, aimed at this slot (A3, I1).'),
          ]), lockedKnob: true), width: 372),
      uc('Fill and stroke — hand-over', (c) => _host(c, Kind.shape, Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            const FillStroke(),
            const Cap('The field shows the value; the tool that edits it opens aimed at this slot of this layer (I1). No popup. 0 stroke reads None (E3).'),
          ]), lockedKnob: true), width: 372),
      uc('Empty state — nothing selected', (c) => _host(c, Kind.none, const Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [EmptyState(), Cap('One plain muted line, no controls, no stale values (A6). Maybe: the composition settings take its place.')])), width: 372),
      uc('Mixed values — several selected', (c) => _host(c, Kind.several, Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            const InspHeader(),
            const Sect(first: true, title: 'Transform', children: [PosRow(), ScaleRow(), OpacityRow()]),
            const Cap('"3 layers". A value that differs shows —, never one layer\'s number. A scrub moves each by the same amount; a typed value sets all to that absolute value (A7). Position X, Scale X and Opacity are mixed here.'),
          ]), count: 3, lockedKnob: true), width: 372),
      uc('Inspector — shape layer', (c) => _panel(c, Kind.shape), width: 372, height: 720),
      uc('Inspector — text layer', (c) => _panel(c, Kind.text), width: 372, height: 720),
      uc('Inspector — effects, long', (c) => _panel(c, Kind.shape, effects: 4, h: 760), width: 372, height: 760),
      uc('Inspector — camera', (c) => _panel(c, Kind.camera), width: 372, height: 720),
      uc('Inspector — narrow (280px)', (c) => _panel(c, Kind.shape, w: 280), width: 280, height: 720),
      uc('Inspector — nothing chosen', (c) => _panel(c, Kind.none, h: 360), width: 372, height: 360),
    ]);
