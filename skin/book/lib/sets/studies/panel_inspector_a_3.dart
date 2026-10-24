// Part of panel_inspector_a.dart (cut there by size, as it was).
part of 'panel_inspector_a.dart';

/// Release order (simplest / most valuable first): I2 Number, I4 Key mark, I6 Tone, I6 Reset, I4 Animate, I2 Grammars, I5 Curve, I5 Words, I3 Space.
/// Every panel use case has the Panel height knob (see [_uc]); a caption is one sentence and lives outside the panel.
List<WidgetbookComponent> panelInspectorASets() => [
      WidgetbookComponent(name: 'Inspector I2 Number', useCases: [
        _uc('Parts @200% number cell', (c) => _pI2Cells()),
        _uc('I2-b Value face, three jobs', (c) => _i2Case('I2-b Face grab', _Hab.habit, 'one surface, three jobs: a move scrubs, a still click types, a double-click goes to default. $_i2Hint', g: _Gr.face, groups: c.knobs.int.slider(label: 'Effect groups (rows)', initialValue: 7, min: 1, max: 7))),
        _uc('I2-b three layers, mixed', (c) => _i2Case('I2-b mixed', _Hab.habit, 'three layers: a differing value reads —; dragging moves all by the same amount, typing sets all. Position X, Opacity, Rotation, Blurriness differ.', g: _Gr.face, name: '3 layers', kind: 'Shape groups', several: const {'pos.x': [800, 960, 1180.5], 'op': [100, 80, 60], 'rot': [0, 15, 15], 'blur.a': [0, 24, 24]})),
        _uc('I2 grammar switch (knob)', (c) {
          final g = c.knobs.object.dropdown<_Gr>(label: 'Grammar', options: _Gr.values, initialOption: _Gr.face, labelBuilder: (g) => switch (g) { _Gr.face => 'I2-b face', _Gr.label => 'I2-a label', _Gr.ladder => 'I2-c ladder', _Gr.stepper => 'I2-d stepper' });
          return _Host(make: () => _M(keys: const {}), build: (ctx, m) => _story('I2 switch', _Hab.habit, 'the same panel and values under each grammar: change the knob and compare the hand, not the picture.', _i2Panel(m, g: g)));
        }),
        _uc('I2 states: several, locked, driven, limit, invalid', (c) => _i2States()),
      ]),
      WidgetbookComponent(name: 'Inspector I2 Grammars', useCases: [
        _uc('Parts @200% row, ladder, step', (c) => _pI2Rows()),
        _uc('I2-a Label grab, value click', (c) => _i2Case('I2-a Label drag, value click', _Hab.habit, 'the label is the scrub handle (↔ cursor); a click on the number types; Enter commits, Esc keeps the old value; double-click a label goes to default.', g: _Gr.label, groups: c.knobs.int.slider(label: 'Effect groups (rows)', initialValue: 7, min: 1, max: 7))),
        _uc('I2-a dock 220, long names', (c) => _i2Case('I2-a dock', _Hab.habit, 'the same grammar in a 220 px dock: label column 72 px, long names cut with …, units drop below 96 px, decimals below 62 px, no digit is cut.', g: _Gr.label, w: c.knobs.double.slider(label: 'Panel width', initialValue: 220, min: 180, max: 282), groups: 5, name: _longName)),
        _uc('I2-c Ladder while held', (c) => _i2Case('I2-c Ladder', _Hab.addition, 'hold a value and move up (coarser) or down (finer): the step x10 / x1 / x0.1 / x0.01 shows beside the cell while held; Esc puts it back.', g: _Gr.ladder)),
        _uc('I2-d Hover stepper', (c) => _i2Case('I2-d ± beside the number', _Hab.addition, 'hover a value: − and + appear inside it; one click is one unit (Shift x10, Alt x0.1), hold to repeat. Scrubbing and typing still work.', g: _Gr.stepper)),
      ]),
      WidgetbookComponent(name: 'Inspector I3 Space', useCases: [
        _uc('Parts @200% instruments', (c) => _pI3()),
        _uc('I3-a Stage leads, numbers only', (c) => _Host(
              make: _spaceModel,
              build: (ctx, m) => _story('I3-a', _Hab.habit, 'the hand is the Stage (move the box, pull a corner, turn from outside one); the Inspector keeps one number row each; Esc undoes the drag.', _i3aPanel(m), side: _Stage(m)),
            )),
        _uc('I3-a dock 220', (c) => _Host(
              make: _spaceModel,
              build: (ctx, m) => _story('I3-a dock', _Hab.habit, 'the same in a 220 px dock where the Stage is elsewhere: X and Y stay side by side, units drop, long layer names cut.', _i3aPanel(m, w: _dockW)),
            )),
        _uc('I3-b Pad, dial and box in the panel', (c) {
          final sp = c.knobs.object.dropdown<int>(label: 'Space', options: const [0, 1, 2], initialOption: 1, labelBuilder: (i) => const ['2D', '2.5D', '3D'][i]);
          return _Host(make: _spaceModel, build: (ctx, m) => _story('I3-b', _Hab.addition, 'a pad for X/Y, a dial, a box for scale, nine points for the anchor; the number under each is the same value (Shift = finer). 3D adds Z and X/Y rotation.', _I3b(m, space: sp)));
        }),
        _uc('I3-b dock 220', (c) {
          final sp = c.knobs.object.dropdown<int>(label: 'Space', options: const [0, 1, 2], initialOption: 1, labelBuilder: (i) => const ['2D', '2.5D', '3D'][i]);
          return _Host(make: _spaceModel, build: (ctx, m) => _story('I3-b dock', _Hab.addition, 'the instruments shrink to 84 px dials and a 196 x 84 pad in a 220 px dock; the numbers keep their size.', _I3b(m, space: sp, w: _dockW)));
        }),
        _uc('I3-c Unfold a row', (c) => const _I3c()),
        _uc('I3-c dock 220', (c) => const _I3c(w: _dockW)),
        _uc('I3-d Grab a label, a hand on the Stage', (c) => const _I3d()),
      ]),
      WidgetbookComponent(name: 'Inspector I4 Key mark', useCases: [
        _uc('Parts @200% key mark', (c) => _pI4Marks()),
        _uc('Parts @200% rows and playhead', (c) => _pI4Rows()),
        _uc('I4-a Key mark on every row', (c) {
          final animated = c.knobs.boolean(label: 'Layer is animated', initialValue: true);
          return _Host(key: ValueKey(animated), make: () => _M(animated: animated), build: (ctx, m) => _story('I4-a', _Hab.habit, 'stopwatch type: the mark is hollow / linked / filled at the playhead (hollow = static, link ticks = animated, filled = a key here, dashed = not available); a click keys here, a click on a filled one removes it; scrub the ruler and values follow.', _i4aPanel(m)));
        }),
        _uc('I4-a dock 220', (c) => _Host(make: _M.new, build: (ctx, m) => _story('I4-a dock', _Hab.habit, 'the key column stays 24 px wide and every row ends at one edge in a 220 px dock; hit area 24 x 24, diamond 8 px.', _i4aPanel(m, w: _dockW)))),
        _uc('I4-d Mark jumps to the next key', (c) => _Host(make: _M.new, build: (ctx, m) => _story('I4-d', _Hab.habit, 'the mark navigates: click = playhead to the next key, Shift+click = the previous; keying is a long-press or right-click, so the two never share a gesture.', _i4dPanel(m)))),
        _uc('I4-d dock 220', (c) => _Host(make: _M.new, build: (ctx, m) => _story('I4-d dock', _Hab.habit, 'the same navigator in a 220 px dock: the mark is the only control in its column.', _i4dPanel(m, w: _dockW)))),
      ]),
      WidgetbookComponent(name: 'Inspector I4 Animate', useCases: [
        _uc('Parts @200% switch, strip, will-key', (c) => _pI4Switch()),
        _uc('Parts @200% time lane', (c) => _pI4Lane()),
        _uc('I4-b Animate off', (c) => _Host(make: () => _M(), build: (ctx, m) => _story('I4-b off', _Hab.habit, 'Animate Off: grey switch, grey hollow diamond with the number of still rows; an edit on a still row changes the value only, a row that already has keys writes a key (as in AE).', _i4bPanel(m)))),
        _uc('I4-b Animate on, edit becomes key', (c) => _Host(make: () => _M()..animate = true, build: (ctx, m) => _story('I4-b on', _Hab.habit, 'Animate On: the switch, the bar edge and the diamond take the accent, and every still row shows the accent hollow diamond: the first edit on it writes its first key.', _i4bPanel(m)))),
        _uc('I4-b dock 220', (c) => _Host(make: () => _M()..animate = true, build: (ctx, m) => _story('I4-b dock', _Hab.habit, 'the same bar in a 220 px dock: switch, noun, diamond and count keep their places; no digit is cut.', _i4bPanel(m, w: _dockW)))),
        _uc('I4-c Time lane in the row', (c) => _Host(make: () => _M(groups: [_gLane, _gLane2], keys: const {'pos.x': {0: 960, 40: 1180, 100: 1320}, 'pos.y': {20: 540, 100: 400}, 'rot': {12: 0, 84: 15}, 'op': {0: 0, 24: 100, 90: 80}, 'blur.a': {30: 0, 60: 24}, 'glow.r': {0: 10, 72: 38}}), build: (ctx, m) => _story('I4-c', _Hab.withdrawn, 'each row has its own short time line: click empty = key, click a key = playhead there, drag = move, right-click = remove. The Timeline, made small.', _i4cPanel(m)))),
      ]),
      WidgetbookComponent(name: 'Inspector I5 Curve', useCases: [
        _uc('Parts @200% plot, thumb, track', (c) => _pI5Curve()),
        _uc('Parts @200% preset and key lists', (c) => _pI5List()),
        _uc('I5-a Named presets + handles', (c) => _story('I5-a', _Hab.habit, 'hover or ↑↓ only peeks, click or Enter writes the ease to both keys, a handle writes on release (Shift locks an axis), Esc takes the peek away.', const _EaseA())),
        _uc('I5-a dock 220', (c) => _story('I5-a dock', _Hab.habit, 'in a 220 px dock the plot is 120 px tall and the Japanese word drops; list, peek and handles are the same.', const _EaseA(w: _dockW))),
        _uc('I5-b Direction and one strength', (c) => _story('I5-b', _Hab.addition, 'ease in / out / both and one strength; the two numbers they imply are shown, not edited. No handles, no names: a shape these cannot make is not offered.', const _EaseB())),
      ]),
      WidgetbookComponent(name: 'Inspector I5 Words', useCases: [
        _uc('Parts @200% motion tiles', (c) => _pI5Words()),
        _uc('Parts @200% copy and paste', (c) => _pI5Copy()),
        _uc('I5-c Motion words, tiles that move', (c) => _story('I5-c', _Hab.addition, 'ten motions named by how they feel; every tile runs its own 1 s movement under a maths name; a click writes it to both keys.', const _EaseC())),
        _uc('I5-c dock 220', (c) => _story('I5-c dock', _Hab.addition, 'one tile per row in a 220 px dock; the movement and the maths line stay.', const _EaseC(w: _dockW))),
        _uc('I5-d Copy ease, paste to many', (c) => _story('I5-d', _Hab.addition, 'Copy ease (only the curve, never values or times), select others with Shift or Cmd, Paste to n: 1 undo step. The clipboard says what it holds.', const _EaseD())),
        _uc('I5-d dock 220', (c) => _story('I5-d dock', _Hab.addition, 'the interval list loses its time column in a 220 px dock; Copy and Paste keep their words and counts.', const _EaseD(w: _dockW))),
      ]),
      WidgetbookComponent(name: 'Inspector I6 Tone', useCases: [
        _uc('Parts @200% tone, count, filter', (c) => _pI6Tone()),
        _uc('I6-a Label tone, double-click to reset', (c) => _Host(
              make: () => _M(keys: const {'pos.x': {0: 960, 48: 1180.5, 96: 1320}, 'op': {0: 0, 24: 100, 90: 80}}),
              build: (ctx, m) => _story('I6-a', _Hab.departs, 'a row at default has a dim label, a changed row a bright label with a dotted underline and a corner mark on its field (shape, not tone alone). Double-click a label to reset (on a keyed row, only the key at the playhead).', _Frame(
                w: _panelW,
                head: _Head(name: 'Jewel Field', trailing: _count('${m.changedCount(m.byId.keys)} changed', tone: N.g95)),
                foot: _Foot(m),
                child: _Body(children: _secs(_allGroups, (r) => _i6Label(r, m))),
              )),
            )),
        _uc('I6-a dock 220', (c) => _Host(
              make: () => _M(keys: const {}),
              build: (ctx, m) => _story('I6-a dock', _Hab.departs, 'the same rule in a 220 px dock: the dotted underline and the field corner are the marks; no dot or arrow per row.', _Frame(
                w: _dockW,
                head: _Head(name: _longName, trailing: _count('${m.changedCount(m.byId.keys)} changed', tone: N.g95)),
                foot: _Foot(m),
                child: _Body(children: _secs(_allGroups, (r) => _i6Label(r, m, dock: true))),
              )),
            )),
        _uc('I6-b Show only what changed', (c) => _I6b(empty: false)),
        _uc('I6-b nothing changed', (c) => _I6b(empty: true)),
        _uc('I6-b dock 220', (c) => const _I6b(empty: false, w: _dockW)),
      ]),
      WidgetbookComponent(name: 'Inspector I6 Reset', useCases: [
        _uc('Parts @200% reset, targets, ghost', (c) => _pI6Reset()),
        _uc('I6-c Reset in the group header', (c) => const _I6c()),
        _uc('I6-d Reset to… default, previous, snapshot', (c) => const _I6d()),
        _uc('I6-d hovering a target', (c) => const _I6d(hover: 2)),
        _uc('I6-d dock 220', (c) => const _I6d(hover: 1, w: _dockW)),
      ]),
    ];
