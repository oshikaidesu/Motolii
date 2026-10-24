// Part of inspector_gui_set.dart (cut there by size, as it was).
part of 'inspector_gui_set.dart';

WidgetbookComponent inspectorGuiSet() => WidgetbookComponent(name: 'inspector-gui', useCases: [
      uc('Transform gui', (c) => _hosted(c, (ctx, d) => _story(_col(TransformGui(d)), d,
          'Pad: press anywhere and the body jumps there (Shift: no jump). Dial: turn from the ring or inside it; 450 stays 450 and shows 1×. Scale: corners scale both (one factor while Link is on), edges one axis. Anchor: one click; a typed value off the nine reads Custom. 2D / 2.5D / 3D reveals Z, the other rotation axes and the Z rail without changing a value. Mixed: X, scale X and rotation read — and the pad moves every ghost by the same amount.')), width: 640),
      uc('Camera face', (c) => _hosted(c, layerKnob: true, (ctx, d) => _story(_col(CameraGui(d)), d,
          'Target (dot) · Orbit (camera, pitch and yaw around the target; hollow = on the far side) · Framing (bar on the ray, outward = farther, exponential) · Roll (ring, turns kept). Turn on "Target is a layer": the point fields disable and say which layer.')), width: 640),
      uc('Layout mini-diagram', (c) => _hosted(c, (ctx, d) => _story(_col(LayoutGui(d)), d,
          'Columns bar (right of row one) · Gap dot · Padding corner · pull the right or bottom edge: Hug / Fill becomes Fixed at the size it already has. Hug · Fill · Fixed are three drawings, not words. Grid off dims the diagram and disables everything but Columns; only Fixed enables its size number.')), width: 640),
      uc('Assembled: (a) fields, (b) instruments, (c) both', (c) => _hosted(c, layerKnob: true, (ctx, d) {
            Widget scroll(String head, Widget Function() build) => SizedBox(height: 700, child: SingleChildScrollView(child: _col(build(), head: head)));
            Widget stack(List<Widget> kids) => Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [for (final (i, k) in kids.indexed) ...[if (i > 0) _gap(16), k]]);
            return SingleChildScrollView(scrollDirection: Axis.horizontal, child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
              Row(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
                scroll('(a) Fields only', () => stack([TransformFields(d), CameraGui(d, mode: Mode.fields), LayoutGui(d, mode: Mode.fields)])),
                const SizedBox(width: 16),
                scroll('(b) Pad, dial, handles', () => stack([TransformGui(d), CameraGui(d), LayoutGui(d)])),
                const SizedBox(width: 16),
                scroll('(c) Both', () => stack([TransformBoth(d), CameraGui(d, mode: Mode.both), LayoutGui(d, mode: Mode.both)])),
              ]),
              const SizedBox(height: 12),
              _help(d, 'Fields: drag a field sideways to scrub, press and drag across the stack to edit several at once, the small tab at the right locks an axis, Alt = apply to all selected, a click types. All three columns edit ONE document: change a value in any of them and the other two follow.'),
            ]));
          }), width: 920),
      uc('Hierarchy options', (c) => _hosted(c, weightKnob: true, (ctx, d) {
            Widget option(String head, String note, Widget body) => Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
                  SizedBox(width: Gui.content + 26, child: Text(head, style: T.title())),
                  const SizedBox(height: 6),
                  SizedBox(width: Gui.content + 26, child: Text(note, style: T.label(N.g63).copyWith(height: 1.4))),
                  const SizedBox(height: 10),
                  SizedBox(height: 640, child: SingleChildScrollView(child: _col(body))),
                ]);
            return SingleChildScrollView(scrollDirection: Axis.horizontal, child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
              Row(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
                option('Option 1  Instruments first', 'precedents-gui.md: Pattern 1C (lines 26-30), P1-c (75), P2-c (81). The pad leads; dial, scale, anchor share one cell size. Numbers are a readout and become fields on hover or drag.', TransformInstrumentsFirst(d)),
                const SizedBox(width: 16),
                option('Option 2  Fields first', 'precedents-gui.md: Pattern 1B (lines 20-24), P2-d (82). Full-weight fields, one 72 px helper of one size beside each.', TransformFieldsFirst(d)),
                const SizedBox(width: 16),
                option('Option 3  One cell grid', 'precedents-gui.md: Pattern 2C (47-48), finding (52), P2-a (79). Four equal square cells, an equal-weight readout under each (hover = fields).', TransformGrid(d)),
              ]),
              const SizedBox(height: 12),
              _help(d, 'The same values in all three. Knob "Number weight" moves Gui.readoutInk / readoutSize / graphicInk: 0 graphic leads, 1 balanced, 2 numbers lead. The instrument strokes follow it in all three; the readouts follow it in option 1 only (option 3 keeps its readout at the balanced step on purpose, option 2 fields are always full weight).'),
            ]));
          }), width: 920),
      uc('UX options', (c) => _hosted(c, (ctx, d) {
            Widget option(String head, String contract, String facts, Widget body) => Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
                  SizedBox(width: Gui.content + 26, child: Text(head, style: T.title())),
                  const SizedBox(height: 6),
                  SizedBox(width: Gui.content + 26, child: Text('Contract: $contract\n\n$facts', style: T.label(N.g63).copyWith(height: 1.4))),
                  const SizedBox(height: 10),
                  SizedBox(height: 560, child: SingleChildScrollView(child: _col(body))),
                ]);
            return SingleChildScrollView(scrollDirection: Axis.horizontal, child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
              Row(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
                option(
                  'U1  Unified object gizmo',
                  'one canvas, five zones: body moves, corner or edge scales (Link keeps the ratio), the ring just outside a corner rotates, the centre dot moves the anchor; hovering names the zone.',
                  'In 1 gesture: move, scale, rotate or anchor, with no mode change.\nClicks before the first drag: X 0, rotation 0, scale 0, anchor 0. Typing a number: not possible here (readout only).\nHidden at rest: every number except one readout line; the rotate zone is drawn only while hovered.',
                  TransformUnified(d),
                ),
                const SizedBox(width: 16),
                option(
                  'U2  Mode switch, one canvas',
                  'a tool row Move / Scale / Rotate / Anchor (keys 1-4); the canvas draws and accepts only the active mode\'s handles, and only that mode\'s numbers are shown.',
                  'In 1 gesture: the active mode\'s change only.\nClicks (or one key) to change: X 1 (Move, if not active) + 1 drag; rotation 1 + drag; scale 1 + drag; anchor 1 + click on a point. Typing: 1 more click on a field.\nHidden at rest: the other three modes\' handles and numbers.',
                  TransformModes(d),
                ),
                const SizedBox(width: 16),
                option(
                  'U3  Scrub-first fields',
                  'Blender-style stacks are the only controls (scrub sideways, sweep vertically to edit several, lock per axis); a preview of the resulting box has no handles.',
                  'In 1 gesture: scrub one field, or sweep several and scrub them together.\nClicks to change: X 0 + 1 scrub; rotation 0 + 1 scrub; scale 0 + 1 scrub; anchor 0 + 1 scrub (or 1 click to type).\nHidden at rest: nothing but the lock tabs (shown on hover); there is nothing to grab on the preview.',
                  TransformScrub(d),
                ),
                const SizedBox(width: 16),
                option(
                  'U4  Summon on focus',
                  'numbers only at rest; hovering or focusing a row opens its instrument inline (Position pad, Rotation dial, Scale box, Anchor nine points); Esc or leaving the row closes it.',
                  'In 1 gesture: scrub a field, or hover a row and drag its instrument.\nClicks to change: X 0 (hover opens the pad) + 1 drag; rotation 0 + 1 drag; scale 0 + 1 drag; anchor 0 + 1 click on a point. Typing: 1 click on a field.\nHidden at rest: all four instruments.',
                  TransformSummon(d),
                ),
              ]),
              const SizedBox(height: 12),
              _help(d, 'All four edit the one document, so a change in any frame shows in the other three. Common to all: Esc during a drag reverts, Shift is fine, a locked layer is inert, a mixed value reads — with a neutral ring, one gesture is one undo step.'),
            ]));
          }), width: 920),
    ]);

// ---- public hooks for the workflow use case (sets/workflow_set.dart): the unified gizmo and its box, nothing else is exposed ----------

/// U1 on a given canvas size: the one canvas that moves, scales, rotates and re-anchors the layer in [d].
Widget unifiedGizmo(GDoc d, Size size, {ValueChanged<Object?>? onHot}) => _gizmoHand(d, size, GMode.all, onHot: onHot);

/// The four corners (clockwise from top left) of [d]'s box on a gizmo canvas of [size], so other panels can draw and hit-test the same box.
List<Offset> gizmoCorners(GDoc d, Size size) {
  final g = _PadGeo(size, d, boost: Gui.boost);
  return g.corners(g.puck, g.half);
}

/// The zone name the gizmo shows while hovered (move, scale, rotate, anchor).
String gizmoZoneName(Object? hot) => _zoneName(hot);

/// One scrub rule for the number fields. false = the lab's first rule (Shift = fine). true = options-inspector.md D-I3: Shift x10, Alt x0.1 (the R1 run).
abstract final class GuiRule {
  static bool shiftBig = false;
}
