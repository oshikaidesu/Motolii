part of 'panel_inspector_a.dart';

// "Parts @200%": the small parts of each option group drawn twice as large, in every state at once, on a labelled grid.
// A part is laid out at its real width, takes its own height, and is scaled 2x by a transform, so lines are re-drawn (not blurred): a 1 px line is 2 px here, a 4 px gap is 8.
// Every cell is the real widget of the panel in a forced state (a pose), never a copy. Nothing on a sheet takes input; the sheet itself scrolls both ways and has a JUMP TO bar.
// A state that does not exist is written down at the foot of the sheet ("Not designed"), so a missing specimen is never a hole.

class _Spec {
  const _Spec(this.cap, this.w, this.child);
  final String cap;
  final double w; // the real width of the part, in px; its height is whatever the part needs
  final Widget child;
}

/// Lays its child out at [w] and paints it at 2x. The size it reports is the doubled size.
class _Z2 extends SingleChildRenderObjectWidget {
  const _Z2({required this.w, required Widget super.child});
  final double w;
  @override
  RenderObject createRenderObject(BuildContext context) => _RenderZ2(w);
  @override
  void updateRenderObject(BuildContext context, _RenderZ2 renderObject) => renderObject.w = w;
}

class _RenderZ2 extends RenderProxyBox {
  _RenderZ2(this._w);
  double _w;
  set w(double v) {
    if (v != _w) {
      _w = v;
      markNeedsLayout();
    }
  }

  @override
  void performLayout() {
    child!.layout(BoxConstraints(minWidth: _w, maxWidth: _w), parentUsesSize: true);
    size = Size(_w * 2, child!.size.height * 2);
  }

  @override
  void paint(PaintingContext context, Offset offset) {
    layer = context.pushTransform(needsCompositing, offset, Matrix4.diagonal3Values(2, 2, 1), (c, o) => c.paintChild(child!, o), oldLayer: layer as TransformLayer?);
  }

  @override
  void applyPaintTransform(RenderBox child, Matrix4 transform) => transform.scaleByDouble(2, 2, 1, 1);

  @override
  bool hitTestChildren(BoxHitTestResult result, {required Offset position}) => false;
}

/// One plate: the part at 200 % on g10 and its caption. Every plate of one group has the width of the widest part of that group.
Widget _plate(_Spec s, double pw) => Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
      Container(
        padding: const EdgeInsets.all(8),
        color: N.g10,
        child: IgnorePointer(child: SizedBox(width: pw * 2, child: Align(alignment: Alignment.topLeft, child: _Z2(w: s.w, child: s.child)))),
      ),
      const SizedBox(height: 4),
      SizedBox(width: pw * 2 + 16, child: Text(s.cap, style: T.label(N.g76).copyWith(height: 1.3))),
    ]);

/// A sheet word that scrolls to its group (hover g100 + underline, focus 1 px g76 edge, Enter / click).
class _Jump extends StatelessWidget {
  const _Jump(this.text, this.onTap);
  final String text;
  final VoidCallback onTap;
  @override
  Widget build(BuildContext context) => _Ctl(
        onTap: onTap,
        builder: (_, h, f, p) => Container(
          padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 4),
          decoration: BoxDecoration(borderRadius: BorderRadius.circular(4), border: Border.all(color: f ? N.g76 : const Color(0x00000000))),
          child: Text(text, style: T.label(h || p ? N.g100 : N.g76).copyWith(decoration: h || p ? TextDecoration.underline : TextDecoration.none, decorationColor: N.g100)),
        ),
      );
}

class _Sheet extends StatefulWidget {
  const _Sheet(this.id, this.note, this.groups, this.none);
  final String id, note;
  final List<(String, List<_Spec>)> groups;
  final String? none; // the states that do not exist
  @override
  State<_Sheet> createState() => _SheetState();
}

class _SheetState extends State<_Sheet> {
  late final _keys = [for (final _ in widget.groups) GlobalKey()];
  void _go(int i) {
    final c = _keys[i].currentContext;
    if (c != null) {
      Scrollable.ensureVisible(c, duration: Duration.zero, alignment: 0);
    }
  }

  @override
  Widget build(BuildContext context) => LayoutBuilder(
        builder: (_, box) => _Ground(
          keys: true,
          child: SizedBox(
            width: math.min(1200, math.max(360, box.maxWidth - 48 - 12)),
            child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
              Text.rich(TextSpan(children: [
                TextSpan(text: '${widget.id}   ', style: T.name(N.g95)),
                TextSpan(text: 'PARTS @200%   ', style: T.micro(N.g76)),
                TextSpan(text: widget.note, style: T.label(N.g76).copyWith(height: 1.35)),
              ])),
              const SizedBox(height: 8),
              Wrap(crossAxisAlignment: WrapCrossAlignment.center, children: [
                Padding(padding: const EdgeInsets.only(right: 8), child: Text('JUMP TO', style: T.micro(N.g76).copyWith(letterSpacing: 1))),
                for (final (i, g) in widget.groups.indexed) _Jump(g.$1, () => _go(i)),
                Padding(padding: const EdgeInsets.only(left: 8), child: Text('(PageUp, PageDown, Home, End and the arrows scroll)', style: T.label(N.g76))),
              ]),
              for (final (i, (title, specs)) in widget.groups.indexed) ...[
                const SizedBox(height: 24),
                KeyedSubtree(key: _keys[i], child: Text(title.toUpperCase(), style: T.micro(N.g76).copyWith(letterSpacing: 1))),
                const SizedBox(height: 8),
                Wrap(spacing: 16, runSpacing: 16, crossAxisAlignment: WrapCrossAlignment.start, children: [for (final sp in specs) _plate(sp, specs.map((e) => e.w).reduce(math.max))]),
              ],
              if (widget.none != null) ...[
                const SizedBox(height: 24),
                Text('NOT DESIGNED', style: T.micro(N.g76).copyWith(letterSpacing: 1)),
                const SizedBox(height: 8),
                Text(widget.none!, style: T.label(N.g76).copyWith(height: 1.35)),
              ],
              const SizedBox(height: 24),
              Text('END OF SHEET', style: T.micro(N.g76).copyWith(letterSpacing: 1)),
            ]),
          ),
        ),
      );
}

Widget _sheet(String id, String note, List<(String, List<_Spec>)> groups, {String? none}) => _Sheet(id, note, groups, none);

/// A part that needs a model of its own: [start] values (changed ones), [keys], [mix] (several layers).
Widget _one(Widget Function(_M m) f, {Map<String, double> start = const {}, Map<String, Map<int, double>> keys = const {}, Map<String, List<double>>? mix, List<_Grp>? groups, bool animate = false}) => _Host(
      make: () {
        final m = groups == null ? _M(start: start, keys: keys) : _M(groups: groups, start: start, keys: keys);
        m.animate = animate;
        if (mix != null) {
          m.mix(mix);
        }
        return m;
      },
      build: (_, m) => f(m),
    );

const _panelIn = 280.0, _dockIn = 218.0; // inner widths: panel 282 / dock 220 minus the 1 px frame. A row specimen is a gutter-padded _panelIn / _dockIn.
const _rowW = 256.0; // the content width of a row (inner width minus the two 12 px gutters)

void _noop() {}

// ================= I2 =================

_Spec _cs(String cap, String id, {_Pose? pose, bool en = true, _Gr g = _Gr.face, String? tag, Map<String, double> start = const {}, List<double>? several, double w = 164}) =>
    _Spec(cap, w, _one((m) => _Cell(m, m.byId[id]!, g: g, pose: pose ?? _Pose.rest, enabled: en, tag: tag), start: start, mix: several == null ? null : {id: several}));

/// A row as it sits in a panel body: gutter-padded, so a selected or hovered row can bleed and its tick can sit in the gutter.
_Spec _rowSpec(String cap, String id, {_Gr g = _Gr.face, bool dock = false, Map<String, double> start = const {}, bool en = true, Color? tone, Widget? lead, String? note, bool hot = false, bool key = false, String? label}) => _Spec(
      cap,
      dock ? _dockIn : _panelIn,
      _one((m) => _gut(_prow(
            label == null ? m.rows[id]! : _R(label, m.rows[id]!.cells),
            m,
            g: g,
            dock: dock,
            enabled: en,
            tone: tone,
            lead: lead,
            labelHot: hot,
            trailW: key ? null : 0,
            trail: key ? const _Key(KeyS.off, enabled: false) : null,
            under: note == null ? null : Padding(padding: EdgeInsets.only(left: dock ? _labWDock : _labW, bottom: 4), child: Text(note, maxLines: 2, overflow: TextOverflow.ellipsis, style: T.label(N.g76).copyWith(height: 1.3))),
          )), start: start),
    );

Widget _pI2Cells() => _sheet('I2 number cell', 'one number field in every state, drawn by the code the panel runs (a forced pose). Edge ladder: rest g20, hover g38, pressed g63 (fill g15), focus g76, editing accent, rejected danger. Marks: two hairlines at the left edge = it is dragged, a corner = changed, hatch = unavailable.', [
      ('Face grammar, one cell 164 px', [
        _cs('rest', 'anchor.x'),
        _cs('hover', 'anchor.x', pose: _Pose.hover),
        _cs('focused (↑↓ keys work)', 'anchor.x', pose: _Pose.focus),
        _cs('pressed (scrubbing)', 'anchor.x', pose: _Pose.press),
        _cs('editing (digits selected)', 'anchor.x', pose: _Pose.edit),
        _cs('rejected (kept 960.0)', 'anchor.x', pose: _Pose.bad),
        _cs('disabled (locked layer): hatched, no outline', 'anchor.x', en: false),
        _cs('changed (g95 + corner mark)', 'pos.x', start: const {'pos.x': 1180.5}),
        _cs('several layers: differs reads —', 'pos.y', several: const [400, 540, 700]),
      ]),
      ('Pair widths: X Y prefix from 66 px, unit from 96 px if the whole value fits, the value shrinks to 10 px before a decimal drops', [
        _Spec('pair 164: units dropped, X Y prefix', 164, _one((m) => _cellsOf(m.rows['pos.x']!, m), start: const {'pos.x': 1180.5})),
        _Spec('pair in a dock 126: decimals dropped', 126, _one((m) => _cellsOf(m.rows['pos.x']!, m, dock: true), start: const {'pos.x': 1180.5})),
        _Spec('long number, pair 164', 164, _one((m) => _cellsOf(m.rows['turb.x']!, m), start: const {'turb.x': 12345678.9, 'turb.y': 98765.4})),
        _Spec('long number, pair 126', 126, _one((m) => _cellsOf(m.rows['turb.x']!, m, dock: true), start: const {'turb.x': 12345678.9, 'turb.y': 98765.4})),
      ]),
      ('Single widths', [
        _cs('single 164: unit shown', 'op', start: const {'op': 80}),
        _Spec('single 80: unit dropped', 80, _one((m) => _Cell(m, m.byId['op']!, pose: _Pose.rest), start: const {'op': 80})),
        _Spec('single 56: nothing but the digits', 56, _one((m) => _Cell(m, m.byId['op']!, pose: _Pose.rest), start: const {'op': 80})),
      ]),
      ('Label and stepper grammars', [
        _Spec('label grammar, rest (tags X Y)', 164, _one((m) => _cellsOf(m.rows['pos.x']!, m, g: _Gr.label))),
        _Spec('label grammar, hover (tag g100)', 164, _one((m) => Row(children: [for (final (i, id) in const ['pos.x', 'pos.y'].indexed) ...[if (i > 0) const SizedBox(width: 4), Expanded(child: _Cell(m, m.byId[id]!, g: _Gr.label, tag: i == 0 ? 'X' : 'Y', pose: _Pose.hover))]]))),
        _cs('stepper, rest (no buttons yet)', 'anchor.x', g: _Gr.stepper),
        _cs('stepper, hover (− and + appear)', 'anchor.x', g: _Gr.stepper, pose: _Pose.hover),
        _cs('stepper, pressed (held repeats)', 'anchor.x', g: _Gr.stepper, pose: _Pose.press),
      ]),
      ('Choice: a value that cycles', [
        _cs('rest (› g63, the one end mark)', 'blur.d'),
        _cs('hover (edge g38, › g95)', 'blur.d', pose: _Pose.hover),
        _cs('focused (edge g76)', 'blur.d', pose: _Pose.focus),
        _cs('pressed (fill g15, edge g63)', 'blur.d', pose: _Pose.press),
        _Spec('long option in 80 px: cut with …', 80, _one((m) => _Cell(m, m.byId['blur.d']!, pose: _Pose.rest))),
        _cs('disabled: hatched, no outline', 'blur.d', en: false),
      ]),
      ('Toggle: switch, then its word', [
        _cs('off', 'blur.e'),
        _cs('on', 'blur.e', start: const {'blur.e': 1}),
        _cs('disabled (locked layer)', 'blur.e', en: false, start: const {'blur.e': 1}),
      ]),
    ], none: 'A toggle has no hover, focus or pressed look of its own: the switch is the whole control. A choice has no editing or rejected state, it only cycles. A number cell is never empty (a cleared field keeps its old value) and has no loading state. The ladder of an I2-c scrub is on the next sheet.');

Widget _pI2Rows() => _sheet('I2 row, ladder, step', 'the row (label column + cells), the ladder shown while a value is held, and the − + buttons. Label ink: g76 at default, g95 changed, g100 while it is the scrub handle, g63 locked.', [
      ('Row label, 280 px (gutters 12)', [
        _rowSpec('label at default (g76)', 'anchor.x'),
        _rowSpec('label changed (g95, dotted underline)', 'pos.x', start: const {'pos.x': 1180.5}),
        _rowSpec('label grammar, rest', 'op', g: _Gr.label, start: const {'op': 80}),
        _rowSpec('label grammar, hover: the label is the handle (g100)', 'op', g: _Gr.label, start: const {'op': 80}, hot: true),
        _rowSpec('long English name, one line, …', 'turb.r'),
        _rowSpec('Japanese name (same ink, same …)', 'tx.s', label: 'フォントサイズ'),
        _rowSpec('Japanese long name, one line, …', 'tx.s', label: 'フォントサイズ（書き出し用・メインタイトル）'),
        _rowSpec('with a key mark column', 'rot', start: const {'rot': 15}, key: true),
        _rowSpec('locked: hatched field, no outline', 'scale.x', en: false),
        _rowSpec('driven by a relation: a hue dot and the driver', 'blur.a', en: false, lead: Container(width: 8, height: 8, decoration: BoxDecoration(color: Role.linkedFor(Fam.follow), shape: BoxShape.circle)), note: 'Follow · Jewel 02'),
        _rowSpec('dock 220: label column 72 px', 'turb.r', dock: true),
      ]),
      ('Ladder: the step while a value is held', [
        for (var i = 0; i < 4; i++) _Spec('rung ${_bandWords[i]}', 60, _Ladder(band: i)),
      ]),
      ('− and + buttons inside a cell', [
        for (final (cap, hot, down) in const [('idle: glyph g63', false, false), ('hovered: g20 fill, glyph g95', true, false), ('pressed: g26 fill, glyph g100', false, true)])
          _Spec(cap, 164, _one((m) => Container(decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(4), border: Border.all(color: _faceLine(_Pose.hover))), padding: const EdgeInsets.symmetric(horizontal: 8), height: _cellH, child: Row(children: [_StepBtn('−', visible: true, hot: hot, down: down, onDown: _noop, onUp: _noop), Expanded(child: Text('960.0', textAlign: TextAlign.right, style: T.value())), const _StepBtn('+', visible: true, onDown: _noop, onUp: _noop)])))),
      ]),
    ], none: 'A row has no hover or pressed look of its own: the cell, the label handle and the key mark each carry theirs. The ladder is a popup that exists only while held, so it has no rest. A − or + has no focus look: the stepper is a pointer grammar, the keyboard uses ↑↓ on the cell.');

// ================= I3 =================

Widget _pI3() {
  const s = .0889, sd = .063; // the pad's scale for 258 x 112 and 196 x 84 (as _PadState computes it)
  Widget pad(double x, double y, {bool ch = false, int hot = 0, double w = 258, double h = 112, double sc = s}) => CustomPaint(size: Size(w, h), painter: _PadPaint(x, y, sc, ch, hot));
  Widget dial(double d, {bool ch = false, int hot = 0, double size = 100}) => CustomPaint(size: Size.square(size), painter: _DialPaint(d, ch, hot));
  Widget box(double k, {bool ch = false, int hot = 0, double size = 100}) => CustomPaint(size: Size.square(size), painter: _ScalePaint(k, ch, hot));
  _Spec anchor(String cap, {Map<String, double> start = const {}, int? hot}) => _Spec(cap, 72, _one((m) => _Anchor9(m, hotPoint: hot), start: start));
  _Spec fold(String cap, {bool open = false, bool? hot, bool? focus, bool? press, String id = 'pos.x', String title = 'Position', Widget Function(_M)? inst}) => _Spec(
        cap,
        _panelIn,
        _one((m) => _gut(_UnfoldRow(title: title, row: m.rows[id]!, m: m, open: open, hotArrow: hot ?? false, focusArrow: focus ?? false, pressArrow: press ?? false, onToggle: _noop, instrument: inst == null ? const SizedBox.shrink() : inst(m))), groups: [..._allGroups, _gSpace]),
      );
  return _sheet('I3 instruments', 'pad, dial, scale box, the nine anchor points and the fold row. Edge ladder: rest g20, hover g38, held g63; the handle brightens to g95 once changed or touched and grows 1 px while held.', [
    ('Pad 258 x 112', [
      _Spec('rest: at default, handle g76', 258, pad(960, 540)),
      _Spec('changed: handle g95', 258, pad(1500, 300, ch: true)),
      _Spec('hover', 258, pad(1500, 300, ch: true, hot: 1)),
      _Spec('held (handle +1 px)', 258, pad(1500, 300, ch: true, hot: 2)),
      _Spec('at the edge: handle stays inside', 258, pad(1920, 0, ch: true)),
    ]),
    ('Pad, dock 196 x 84', [
      _Spec('rest', 196, pad(960, 540, w: 196, h: 84, sc: sd)),
      _Spec('changed', 196, pad(1500, 300, ch: true, w: 196, h: 84, sc: sd)),
    ]),
    ('Dial 100', [
      _Spec('rest 0°', 100, dial(0)),
      _Spec('changed 15°', 100, dial(15, ch: true)),
      _Spec('hover', 100, dial(15, ch: true, hot: 1)),
      _Spec('held', 100, dial(15, ch: true, hot: 2)),
      _Spec('past 180°', 100, dial(250, ch: true)),
    ]),
    ('Dial, dock 84', [
      _Spec('rest 0°', 84, dial(0, size: 84)),
      _Spec('changed 15°', 84, dial(15, ch: true, size: 84)),
    ]),
    ('Scale box 100', [
      _Spec('rest 100 %', 100, box(1)),
      _Spec('changed 140 % (clamped at 160)', 100, box(1.4, ch: true)),
      _Spec('small 30 %', 100, box(.3, ch: true)),
      _Spec('hover', 100, box(1.4, ch: true, hot: 1)),
      _Spec('held (corners 8 px)', 100, box(1.4, ch: true, hot: 2)),
    ]),
    ('Scale box, dock 84', [
      _Spec('rest 100 %', 84, box(1, size: 84)),
      _Spec('changed 140 %', 84, box(1.4, ch: true, size: 84)),
    ]),
    ('Anchor, nine points', [
      anchor('centre selected (8 px, g95)', start: const {'anchor.x': 960, 'anchor.y': 540}),
      anchor('top left selected', start: const {'anchor.x': 0, 'anchor.y': 0}),
      anchor('none selected (anchor is elsewhere)', start: const {'anchor.x': 300, 'anchor.y': 200}),
      anchor('hover on another point (7 px, g76)', start: const {'anchor.x': 960, 'anchor.y': 540}, hot: 2),
    ]),
    ('Fold row (I3-c), 280 px', [
      fold('closed, arrow at rest (g63)'),
      fold('closed, arrow hovered (plate g15, g95)', hot: true),
      fold('closed, arrow focused (1 px g76 edge)', focus: true),
      fold('closed, arrow pressed (plate g26)', press: true),
      fold('open: row selected (g20 + tick), instrument under it', open: true, id: 'rot', title: 'Rotation', inst: (m) => Align(alignment: Alignment.centerLeft, child: _Dial(m, size: 84))),
    ]),
  ], none: 'The pad, dial, box and nine points are pointer instruments: they have no keyboard focus look (the same value is a number cell with focus, one row below each) and no disabled look (a locked layer hides them). The handle has no pressed look beyond "held".');
}

// ================= I4 =================

Widget _keyMark(KeyS st, {bool? hover, bool? focus, bool? press, bool will = false, bool enabled = true}) => _Key(st, will: will, enabled: enabled, onTap: _noop, hover: hover ?? false, focus: focus ?? false, pressed: press ?? false);

Widget _pI4Marks() {
  List<_Spec> states(String name, KeyS st, {bool will = false}) => [
        _Spec('$name, rest', _keyW, _keyMark(st)),
        _Spec('$name, hover (plate g15)', _keyW, _keyMark(st, hover: true)),
        _Spec('$name, focus (edge g76)', _keyW, _keyMark(st, focus: true)),
        _Spec('$name, pressed (plate g26)', _keyW, _keyMark(st, press: true)),
        _Spec('$name, disabled', _keyW, _keyMark(st, enabled: false)),
      ];
  return _sheet('I4 key mark', 'the mark in its states: hollow = no keys, with its link = animated, filled = a key at the playhead; the accent hollow = the next edit writes the first key.', [
    ('Static: hollow, 24 x 24', [...states('static', KeyS.off), _Spec('static, will key (accent)', _keyW, _keyMark(KeyS.off, will: true))]),
    ('Animated: with its link', states('animated', KeyS.anim)),
    ('Key here: filled (grows 1.6 px on hover)', states('key here', KeyS.at)),
  ], none: 'A mark has no selected state (the filled diamond is a fact about the playhead, not a selection) and no loading state.');
}

Widget _pI4Rows() => _sheet('I4 rows and playhead', 'the key column in a row (all rows end at one edge) and the playhead strip with the layer\'s keys on its ruler.', [
      ('Row with a key column, 280 px', [
        for (final (cap, id) in const [('key here (pos.x has a key at the playhead)', 'pos.x'), ('animated (rot: keys before and after)', 'rot'), ('static (no keys)', 'tx.t')])
          _Spec(cap, _panelIn, _one((m) => _gut(_keyRow(m.rows[id]!, m)), keys: _startKeys)),
        _Spec('static, Animate On: will key', _panelIn, _one((m) => _gut(_keyRow(m.rows['tx.t']!, m, will: true)), keys: _startKeys)),
        _Spec('row without a mark (not keyable)', _panelIn, _one((m) => _gut(_keyRow(m.rows['blur.e']!, m)), keys: _startKeys)),
        _Spec('dock 220, long name', _dockIn, _one((m) => _gut(_keyRow(m.rows['turb.r']!, m, dock: true)), keys: _startKeys)),
      ]),
      ('Playhead strip', [
        _Spec('playhead, 280', _panelIn, _one((m) => _Playhead(m), keys: _startKeys)),
        _Spec('playhead, dock 218', _dockIn, _one((m) => _Playhead(m, dock: true), keys: _startKeys)),
      ]),
    ], none: 'The ruler has no hover or focus look: it is scrubbed by the pointer, and a key mark on it is not a control (the row\'s own mark is)..');

Widget _pI4Switch() => _sheet('I4 Animate bar', 'the bar that holds the Animate switch: the switch with its noun, the hollow diamond every still row will wear and how many rows that is, and the playhead under it. On takes the accent (bar edge, diamond); Off is grey. The row that will take its first key shows the accent diamond.', [
      ('Bar, 280 px', [
        _Spec('Animate Off', _panelIn, _one((m) => _AnimateBar(m), keys: _startKeys)),
        _Spec('Animate On', _panelIn, _one((m) => _AnimateBar(m), keys: _startKeys, animate: true)),
        _Spec('dock 218', _dockIn, _one((m) => _AnimateBar(m, dock: true), keys: _startKeys, animate: true)),
      ]),
      ('Row trail, 24 px column, 280 px', [
        _Spec('Animate On, will key', _panelIn, _one((m) => _gut(_prow(m.rows['tx.t']!, m, trail: _Key(KeyS.off, will: true, onTap: _noop))), animate: true)),
        _Spec('Animate On, will key, hover', _panelIn, _one((m) => _gut(_prow(m.rows['tx.t']!, m, trail: _Key(KeyS.off, will: true, onTap: _noop, hover: true))), animate: true)),
        _Spec('Animate Off, still row', _panelIn, _one((m) => _gut(_prow(m.rows['tx.t']!, m, trail: _Key(KeyS.off, onTap: _noop))))),
        _Spec('animated row, Animate On', _panelIn, _one((m) => _gut(_prow(m.rows['rot']!, m, trail: _Key(KeyS.anim, onTap: _noop))), keys: _startKeys, animate: true)),
      ]),
    ], none: 'The switch is the lab\'s PillSwitch: it has no hover, focus or pressed look of its own (on and off only). There is no "Animate unavailable" state: a locked layer hides the bar.');

Widget _pI4Lane() {
  Widget lane(List<int> keys, int frame) => CustomPaint(size: const Size(104, _cellH), painter: _LanePaint(keys, frame, keys.isNotEmpty));
  return _sheet('I4 time lane', 'the row\'s own short time line (104 px = 4 s). A line g20 when static, g44 between the first and last key; keys g95, the one at the playhead accent and 1 px larger; the playhead a 1 px accent line.', [
    ('Lane 104 x 24', [
      _Spec('static: no keys, line g20', 104, lane(const [], 48)),
      _Spec('one key, playhead elsewhere', 104, lane(const [24], 48)),
      _Spec('two keys, playhead between', 104, lane(const [12, 84], 48)),
      _Spec('key at the playhead', 104, lane(const [12, 48, 84], 48)),
      _Spec('dense: six keys', 104, lane(const [6, 18, 30, 42, 54, 66], 48)),
      _Spec('playhead at the end', 104, lane(const [12, 84], 120)),
    ]),
    ('Lane row, 280 px', [
      for (final (cap, id) in const [('animated', 'pos.x'), ('static', 'pos.y'), ('long name (80 px label)', 'blur.a')])
        _Spec(cap, _panelIn, _one((m) => _gut(_laneRow(m.rows[id]!, m)), groups: [_gLane, _gLane2], keys: const {'pos.x': {0: 960, 48: 1180, 100: 1320}, 'blur.a': {30: 0, 60: 24}})),
      _Spec('Glow Intensity, 80 px label', _panelIn, _one((m) => _gut(_laneRow(m.rows['glow.i']!, m)), groups: [_gLane, _gLane2])),
    ]),
  ], none: 'A lane has no hover, focus or pressed look: a key under the pointer is found by distance (8 px) and the drag moves the key itself. It has no disabled look: a locked layer shows no lanes.');
}

// ================= I5 =================

_E _pe(String name) => _presets.firstWhere((e) => e.name == name);

Widget _pI5Curve() {
  Widget plot(_E e, {_E? peek, bool handles = true, int? hot}) => CustomPaint(size: const Size(256, 150), painter: _PlotPaint(e, peek, handles, hot));
  Widget track(double t, _E a, [_E? b]) => CustomPaint(size: const Size(256, 24), painter: _TrackPaint(t, a, b));
  Widget slide(double v, {bool hot = false, bool focus = false, bool press = false}) => _Slide(label: 'Strength', value: v, onChanged: (_) {}, hot: hot, focus: focus, press: press);
  return _sheet('I5 curve parts', 'plot (grid g8, the 0 and 1 lines, the curve g95 1.6 px, handles with their stems), thumb, track and slider. A peek draws the other curve over a dimmed applied one and hides the handles.', [
    ('Plot 256 x 150', [
      _Spec('applied + handles (Back Out)', 256, plot(_pe('Back Out'))),
      _Spec('handle hover (6 px, g100)', 256, plot(_pe('Back Out'), hot: 1)),
      _Spec('peek over applied: handles hidden', 256, plot(_pe('Ease In Out'), peek: _pe('Back In Out'))),
      _Spec('no handles (I5-b)', 256, plot(_pe('Ease Out'), handles: false)),
      _Spec('linear', 256, plot(_pe('Linear'))),
      _Spec('undershoot + overshoot (Back In Out)', 256, plot(_pe('Back In Out'))),
    ]),
    ('Thumb 36 x 20', [
      _Spec('rest (g63, 1 px)', 36, _thumb(_pe('Ease Out'), w: 36, h: 20)),
      _Spec('selected / peeked (g95, 1.5 px)', 36, _thumb(_pe('Ease Out'), w: 36, h: 20, on: true)),
      _Spec('with a moving dot', 36, _thumb(_pe('Ease Out'), w: 36, h: 20, on: true, t: .5)),
      _Spec('overshoot', 36, _thumb(_pe('Back Out'), w: 36, h: 20)),
      _Spec('undershoot', 36, _thumb(_pe('Back In'), w: 36, h: 20)),
    ]),
    ('Track 256 x 24 (one loop: 1 s of movement)', [
      _Spec('t = 0', 256, track(0, _pe('Ease Out'))),
      _Spec('t = 0.5', 256, track(.5, _pe('Ease Out'))),
      _Spec('t = 1', 256, track(1, _pe('Ease Out'))),
      _Spec('peek ring over the applied dot', 256, track(.5, _pe('Ease Out'), _pe('Back Out'))),
    ]),
    ('Interval card, 256 px', [
      _Spec('two keys, two values', 256, _interval(256)),
    ]),
    ('Strength slider, 256 px', [
      _Spec('55 %: rest (edge g20)', 256, slide(.55)),
      _Spec('hover (edge g38)', 256, slide(.55, hot: true)),
      _Spec('focused (edge g76; ← → move it)', 256, slide(.55, focus: true)),
      _Spec('pressed (edge g63)', 256, slide(.55, press: true)),
      _Spec('0 %', 256, slide(0)),
      _Spec('100 %', 256, slide(1)),
    ]),
  ], none: 'A handle has no focus look (the keyboard path is the preset list and the slider). The plot and the track are not controls on their own: no disabled look. The interval card is a read-out.');
}

Widget _pI5List() => _sheet('I5 lists', 'a preset row (selected = g20 + tick, peeked = g15, pressed = g26) and the key list the interval belongs to. Rows sit in a gutter-padded list: the fill bleeds, the tick is in the gutter.', [
      ('Preset row, 280 px', [
        _Spec('rest', _panelIn, _gut(_PresetRow(_pe('Ease Out')))),
        _Spec('peeked: hover or ↑↓ (the keyboard cursor)', _panelIn, _gut(_PresetRow(_pe('Ease Out'), peek: true))),
        _Spec('pressed', _panelIn, _gut(_PresetRow(_pe('Ease Out'), press: true))),
        _Spec('selected (applied)', _panelIn, _gut(_PresetRow(_pe('Ease Out'), on: true))),
        _Spec('long Japanese word', _panelIn, _gut(_PresetRow(_pe('Back In Out')))),
        _Spec('dock 218: the Japanese drops', _dockIn, _gut(_PresetRow(_pe('Back In Out'), dock: true))),
        _Spec('long name', _panelIn, _gut(const _PresetRow(_E('Cubic Bezier Overshoot In Out Extra', '行き過ぎて戻る長い名前', _lin, bez: [.1, 1.4, .2, 1])))),
      ]),
      ('Key list, 280 px', [
        _Spec('the two keys of the interval lit (g13)', _panelIn, _gut(_keysList(_rowW))),
      ]),
    ], none: 'A preset row has no focus look of its own: the peek (g15) is the keyboard cursor, and Enter writes it. A key row is a read-out: no hover, no pressed, no disabled.');

Widget _pI5Words() {
  const tw = (_rowW - 8) / 2;
  final w0 = _words[0], w5 = _words[5];
  return _sheet('I5 motion words', 'a tile: the word, its maths name (up to two lines), a 1 s movement. Selected = g20 fill + accent edge. The loop is frozen at t = 0.5 here so the dot can be measured.', [
    ('Tile 124 x 80', [
      _Spec('rest', tw, _Tile(w0, w: tw, t: .5)),
      _Spec('hover (edge g44)', tw, _Tile(w0, w: tw, hot: true, t: .5)),
      _Spec('focus (edge g95)', tw, _Tile(w0, w: tw, focus: true, t: .5)),
      _Spec('pressed (fill g07, edge g63)', tw, _Tile(w0, w: tw, press: true, t: .5)),
      _Spec('selected', tw, _Tile(w0, w: tw, sel: true, t: .5)),
      _Spec('long maths line wraps to two lines', tw, _Tile(w5, w: tw, t: .5)),
      _Spec('overshoot (ばね)', tw, _Tile(w5, w: tw, sel: true, t: .9)),
    ]),
    ('Tile, dock: one per row, 256', [
      _Spec('rest', _rowW, _Tile(w0, w: _rowW, t: .5)),
      _Spec('selected', _rowW, _Tile(w0, w: _rowW, sel: true, t: .5)),
    ]),
  ], none: 'A tile has no disabled look: a locked layer shows no motion words. Hover and focus are both an edge, two steps apart (g44 and g95).');
}

Widget _pI5Copy() {
  final iv = _Iv('Position X', '00:00', '00:12', _pe('Back Out'));
  final ivLong = _Iv('Wave Height Offset', '01:00', '02:00', _pe('Quart In Out'));
  return _sheet('I5 copy and paste', 'the two actions (24 px high, 4 px corners like every field), the clipboard chip that shows what it holds, and the interval list rows.', [
    ('Action button 122 x 24', [
      _Spec('on, rest (g20, edge g44)', 122, _Act('Copy ease', on: true, onTap: _noop)),
      _Spec('hover (g26, edge g63)', 122, _Act('Copy ease', on: true, hot: true, onTap: _noop)),
      _Spec('focus (edge g95)', 122, _Act('Copy ease', on: true, focus: true, onTap: _noop)),
      _Spec('pressed (sinks to g13, accent line)', 122, _Act('Copy ease', on: true, pressed: true, onTap: _noop)),
      _Spec('off (needs one interval)', 122, _Act('Copy ease', on: false, onTap: _noop)),
      _Spec('with a count', 122, _Act('Paste to 12', on: true, onTap: _noop)),
      _Spec('dock label', 122, _Act('Paste 12', on: true, onTap: _noop)),
    ]),
    ('Clipboard chip, 280 px', [
      _Spec('empty: a dashed slot', _panelIn, const _Clip(null)),
      _Spec('holds an ease: its thumb, name, numbers', _panelIn, _Clip(iv.e)),
    ]),
    ('Interval row, 280 px', [
      _Spec('rest', _panelIn, _gut(_IvRow(iv))),
      _Spec('hover (g15)', _panelIn, _gut(_IvRow(iv, hot: true))),
      _Spec('focus (1 px g76 edge)', _panelIn, _gut(_IvRow(iv, focus: true))),
      _Spec('pressed (g26)', _panelIn, _gut(_IvRow(iv, press: true))),
      _Spec('selected (g20 + tick)', _panelIn, _gut(_IvRow(iv, on: true))),
      _Spec('long property name', _panelIn, _gut(_IvRow(ivLong))),
      _Spec('dock 218: no time column', _dockIn, _gut(_IvRow(iv, dock: true))),
    ]),
  ], none: 'The clipboard chip is a read-out: no hover or focus. A button has no selected state (it acts once).');
}

// ================= I6 =================

Widget _pI6Tone() {
  _Spec row(String cap, String id, {Map<String, double> start = const {}, bool dock = false}) => _Spec(cap, dock ? _dockIn : _panelIn, _one((m) => _gut(_i6Label(m.rows[id]!, m, dock: dock)), start: start));
  return _sheet('I6 tone and filter', 'a label is g76 at default; changed = g95 with a dotted underline (and a corner mark on its field). Plus the count words, the filter bar and the empty answer.', [
    ('Row label, 280 px', [
      row('default (g76)', 'anchor.x'),
      row('changed (g95, dotted underline)', 'pos.x', start: const {'pos.x': 1180.5}),
      row('changed, one of a pair (the row lights)', 'turb.x', start: const {'turb.y': 700}),
      row('long name', 'turb.r', start: const {'turb.r': 3}),
      row('dock 218, changed', 'pos.x', start: const {'pos.x': 1180.5}, dock: true),
    ]),
    ('Count words', [
      _Spec('head: changed (g95)', 64, _count('7 changed', tone: N.g95)),
      _Spec('head: none (g76)', 64, _count('0 changed', tone: N.g76)),
      _Spec('group: "3 changed"', 64, _count('3 changed')),
      _Spec('group header with the count', _panelIn, _gut(_Sec('Transform  トランスフォーム', const [], first: true, trailing: _count('3 changed')))),
      _Spec('panel head with the count', _panelIn, _Head(name: 'Jewel Field', trailing: _count('7 changed', tone: N.g95))),
    ]),
    ('Filter bar', [
      _Spec('on: changed only, 280', _panelIn, const _FilterBar(on: true, n: 7, total: 31)),
      _Spec('off: all rows, 280', _panelIn, const _FilterBar(on: false, n: 7, total: 31)),
      _Spec('dock 218', _dockIn, const _FilterBar(on: true, n: 7, total: 31)),
    ]),
    ('Empty answer', [
      _Spec('nothing changed: one word, one button (rest)', _panelIn, const _EmptyCard()),
      _Spec('the button, hover', _panelIn, const _EmptyCard(hover: true)),
      _Spec('the button, focused', _panelIn, const _EmptyCard(focus: true)),
      _Spec('the button, pressed', _panelIn, const _EmptyCard(pressed: true)),
    ]),
  ], none: 'A label has no hover look except in the label grammar (I2-a): in I6 the label is read, and a double-click resets it. The count words are text: no states. The filter switch is the lab\'s PillSwitch.');
}

Widget _pI6Reset() {
  Widget ghostRow(String id, Map<String, String> ghost, {Map<String, double> start = const {}, bool dock = false}) => _one((m) => _gut(_i6Label(m.rows[id]!, m, ghost: ghost, dock: dock)), start: start);
  const names = ['Default', 'Before this session', 'Snapshot “v2 soft glow”'], subs = ['the layer\'s own defaults', 'the state when you opened the layer', 'saved 00:41 ago'];
  Widget head(Widget t) => _Head(name: 'Jewel Field', trailing: t);
  Widget gh(int k, {bool hover = false, bool focus = false, bool pressed = false}) => _gut(_Sec('Transform  トランスフォーム', const [], first: true, trailing: _GroupReset(k, hover: hover, focus: focus, pressed: pressed)));
  return _sheet('I6 reset', 'the group reset button, the three targets, the header trigger, and the ghost values a hovered or focused target previews on the rows.', [
    ('Group reset button, 280 px', [
      _Spec('3 changed: rest', _panelIn, gh(3)),
      _Spec('hover', _panelIn, gh(3, hover: true)),
      _Spec('focus', _panelIn, gh(3, focus: true)),
      _Spec('pressed', _panelIn, gh(3, pressed: true)),
      _Spec('nothing to reset: no count, a dead button', _panelIn, gh(0)),
    ]),
    ('Header trigger, 280 px', [
      _Spec('rest', _panelIn, head(const _Btn('Reset to…'))),
      _Spec('hover', _panelIn, head(const _Btn('Reset to…', hover: true))),
      _Spec('focus', _panelIn, head(const _Btn('Reset to…', focus: true))),
      _Spec('pressed', _panelIn, head(const _Btn('Reset to…', pressed: true))),
      _Spec('open (the targets are out)', _panelIn, head(const _Btn('Reset to…', open: true))),
    ]),
    ('Target, 280 px', [
      _Spec('rest', _panelIn, _gut(_Target(names[0], subs[0], 5))),
      _Spec('hover (g15; ghosts show on the rows)', _panelIn, _gut(_Target(names[0], subs[0], 5, hover: true))),
      _Spec('focus (g20 + tick; ghosts show too)', _panelIn, _gut(_Target(names[0], subs[0], 5, focus: true))),
      _Spec('pressed (g26 + tick)', _panelIn, _gut(_Target(names[0], subs[0], 5, pressed: true))),
      _Spec('nothing would change', _panelIn, _gut(_Target(names[1], subs[1], 0))),
      _Spec('long name', _panelIn, _gut(_Target(names[2], subs[2], 12))),
      _Spec('dock 218: long name cut', _dockIn, _gut(_Target(names[2], subs[2], 12, focus: true))),
    ]),
    ('Ghost: the value a target would give, 280 px', [
      _Spec('a pair: → 1020 → 560', _panelIn, ghostRow('pos.x', const {'pos.x': '1020', 'pos.y': '560'}, start: const {'pos.x': 1180.5})),
      _Spec('a single number', _panelIn, ghostRow('op', const {'op': '80'}, start: const {'op': 100})),
      _Spec('a choice', _panelIn, ghostRow('blur.d', const {'blur.d': 'Vertical'})),
      _Spec('a toggle', _panelIn, ghostRow('blur.e', const {'blur.e': 'On'})),
      _Spec('dock 218, a pair (numbers only)', _dockIn, ghostRow('pos.x', const {'pos.x': '1020', 'pos.y': '560'}, start: const {'pos.x': 1180.5}, dock: true)),
    ]),
  ], none: 'A ghost value has no hover or focus look: it is a preview, never a control. A target has no selected state (a click acts at once). The count of values on a target is text.');
}
