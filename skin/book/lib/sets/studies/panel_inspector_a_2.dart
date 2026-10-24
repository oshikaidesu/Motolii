// Part of panel_inspector_a.dart (cut there by size, as it was).
part of 'panel_inspector_a.dart';

/// A reset target (I6-d): its name, what it is, how many values it would change. Rows inside a gutter-padded body: the fill bleeds, the tick sits in the gutter.
/// rest: bare. hover g15 (the ghost values show on the rows). focus g20 + tick (the keyboard cursor; ghosts show too). pressed g26 + tick.
class _Target extends StatelessWidget {
  const _Target(this.name, this.sub, this.n, {this.hover = false, this.focus = false, this.pressed = false});
  final String name, sub;
  final int n;
  final bool hover, focus, pressed;
  @override
  Widget build(BuildContext context) => _Bleed(
        height: 40,
        fill: pressed ? N.g26 : (focus ? N.g20 : (hover ? N.g15 : null)),
        tick: focus || pressed,
        child: Row(children: [
          Expanded(child: Column(mainAxisAlignment: MainAxisAlignment.center, crossAxisAlignment: CrossAxisAlignment.start, children: [Text(name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(N.g95)), const SizedBox(height: 4), Text(sub, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g76))])),
          const SizedBox(width: 8),
          Text('$n values', style: T.label(n == 0 ? N.g76 : N.g95)),
        ]),
      );
}

/// The reset word in a group header (I6-c): "3 changed" and a button. Nothing changed = no count, and a dead button.
class _GroupReset extends StatelessWidget {
  const _GroupReset(this.k, {this.hover = false, this.focus = false, this.pressed = false});
  final int k;
  final bool hover, focus, pressed;
  @override
  Widget build(BuildContext context) => Row(mainAxisSize: MainAxisSize.min, children: [
        if (k > 0) ...[Text('$k', style: T.value(N.g95)), const SizedBox(width: 8)],
        _Btn('Reset', h: 20, on: k > 0, hover: hover, focus: focus, pressed: pressed),
      ]);
}

Widget _i6Label(_R r, _M m, {bool dock = false, Map<String, String>? ghost, Widget? trail, Widget? lead, double? trailW}) => _prow(
      r, m,
      dock: dock,
      labelDbl: () => m.resetMany(r.cells.map((c) => c.id), 'reset ${r.label}'),
      ghost: ghost,
      trail: trail,
      lead: lead,
      trailW: trailW ?? 0,
    );

class _I6b extends StatefulWidget {
  const _I6b({required this.empty, this.w = _panelW});
  final bool empty;
  final double w;
  @override
  State<_I6b> createState() => _I6bState();
}

class _I6bState extends State<_I6b> {
  late final _M m = widget.empty ? _M(start: const {}, keys: const {}) : _M(keys: const {});
  bool _only = true;
  @override
  void initState() {
    super.initState();
    m.addListener(() => setState(() {}));
  }

  @override
  void dispose() {
    m.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final dock = widget.w < 250;
    final all = [for (final g in _allGroups) for (final r in g.rows) r];
    final n = all.where((r) => r.cells.any((c) => m.changed(c.id))).length;
    bool shown(_R r) => !_only || r.cells.any((c) => m.changed(c.id));
    final gs = [for (final g in _allGroups) if (g.rows.any(shown)) g];
    return _story(
      widget.empty ? 'I6-b empty result' : 'I6-b',
      _Hab.habit,
      widget.empty
          ? 'with nothing changed the panel says so and gives one way back (Show all rows).'
          : 'one switch: only what differs from default. The count lives in the bar; a reset takes the row out of the list.',
      _Frame(
        w: widget.w,
        head: _Head(name: dock ? _longName : 'Jewel Field'),
        foot: _Foot(m),
        child: _top(
          _FilterBar(on: _only, n: n, total: all.length, onChanged: (v) => setState(() => _only = v)),
          _only && n == 0
              ? _EmptyCard(onShow: () => setState(() => _only = false))
              : _Body(children: _secs(gs, (r) => shown(r) ? _i6Label(r, m, dock: dock) : const SizedBox.shrink(), trailing: (g) {
                  final k = g.rows.where((r) => r.cells.any((c) => m.changed(c.id))).length;
                  return _only || k == 0 ? null : _count('$k changed');
                })),
        ),
      ),
    );
  }
}

class _I6c extends StatefulWidget {
  const _I6c();
  @override
  State<_I6c> createState() => _I6cState();
}

class _I6cState extends State<_I6c> {
  late final _M m = _M(groups: _allGroups, keys: const {'pos.x': {0: 960, 48: 1180.5, 96: 1320}, 'op': {0: 0, 24: 100, 90: 80}})..seek(48);
  static const _driven = {'rot'};
  @override
  void initState() {
    super.initState();
    m.addListener(() => setState(() {}));
  }

  @override
  void dispose() {
    m.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    Widget row(_R r) {
      final drv = _driven.contains(r.id);
      return _prow(
        r, m,
        trailW: 24,
        enabled: !drv,
        lead: drv ? Container(width: 8, height: 8, decoration: BoxDecoration(color: Role.linkedFor(Fam.follow), shape: BoxShape.circle)) : null,
        trail: r.keyable && r.isNum ? _Key(m.stateOf(r.id), onTap: () => m.toggleKey(r)) : null,
        under: drv ? Padding(padding: const EdgeInsets.only(left: _labW, bottom: 4), child: Text('Driven by Follow · Jewel 02', style: T.label(Role.linkedFor(Fam.follow)))) : null,
      );
    }

    return _story(
      'I6-c',
      _Hab.habit,
      'a group header resets only its own group: keys away from the playhead (${_fmtT(m.frame)}) and driven rows stay, and the foot says what was kept. One undo step.',
      _Frame(
        w: _panelW,
        head: const _Head(name: 'Jewel Field'),
        foot: _Foot(m),
        child: _Body(
          children: _secs(_allGroups, row, trailing: (g) {
            final ids = [for (final r in g.rows) for (final c in r.cells) c.id];
            final k = m.changedCount(ids);
            return _Ctl(
              enabled: k > 0,
              onTap: () => m.resetKeep(ids, _driven, g.title.split('  ').first),
              builder: (_, h, f, p) => _GroupReset(k, hover: h, focus: f, pressed: p),
            );
          }),
        ),
      ),
    );
  }
}

class _I6d extends StatefulWidget {
  const _I6d({this.hover, this.w = _panelW});
  final int? hover;
  final double w;
  @override
  State<_I6d> createState() => _I6dState();
}

class _I6dState extends State<_I6d> {
  late final _M m = _M(keys: const {});
  static const _prev = <String, double>{'pos.x': 1100, 'scale.x': 104, 'scale.y': 104, 'rot': 10, 'op': 90, 'blur.a': 12, 'glow.r': 22, 'glow.i': 1.1, 'wave.h': 34, 'turb.a': 62, 'sh.o': 50, 'sh.f': 12, 'tx.t': 40, 'blur.d': 1, 'blur.e': 1};
  static const _snap = <String, double>{'pos.x': 1020, 'pos.y': 560, 'scale.x': 105, 'scale.y': 105, 'rot': 15, 'op': 80, 'blur.a': 8, 'glow.r': 30, 'glow.i': 1.35, 'wave.h': 20, 'turb.a': 50, 'sh.o': 65, 'sh.f': 12, 'tx.t': 40, 'blur.d': 0, 'blur.e': 1};
  bool _open = true;
  int? _hov;

  @override
  void initState() {
    super.initState();
    _hov = widget.hover;
    m.addListener(() => setState(() {}));
  }

  @override
  void dispose() {
    m.dispose();
    super.dispose();
  }

  Map<String, double> _target(int i) => switch (i) { 0 => m.def, 1 => {...m.def, ..._prev}, _ => {...m.def, ..._snap} };

  Map<String, String> _diff(int i) {
    final t = _target(i);
    final o = <String, String>{};
    for (final e in t.entries) {
      if (_round(m.at(e.key)) != _round(e.value)) {
        final p = m.byId[e.key]!;
        o[e.key] = p.kind == _K.num ? _fmtNum(e.value, p.dec) : (p.kind == _K.toggle ? (e.value > .5 ? 'On' : 'Off') : p.options[e.value.round().clamp(0, p.options.length - 1)]);
      }
    }
    return o;
  }

  void _go(int i) {
    final d = _diff(i);
    m.begin();
    for (final e in _target(i).entries) {
      m.v[e.key] = e.value;
    }
    m.end('reset to ${const ['default', 'before this session', 'snapshot'][i]}: ${d.length} values, 1 undo step');
    setState(() => _hov = null);
  }

  @override
  Widget build(BuildContext context) {
    final names = ['Default', 'Before this session', 'Snapshot “v2 soft glow”'];
    final sub = ['the layer\'s own defaults', 'the state when you opened the layer', 'saved 00:41 ago'];
    final ghost = _hov == null ? null : _diff(_hov!);
    return _story(
      widget.w < 250 ? 'I6-d dock' : 'I6-d',
      _Hab.addition,
      'Reset opens three targets; hovering or focusing one shows its values on the rows (→ 72) without writing; a click is one undo step. Double-click a label still goes to Default.',
      _Frame(
        w: widget.w,
        head: _Head(name: widget.w < 250 ? _longName : 'Jewel Field', trailing: _Ctl(onTap: () => setState(() => _open = !_open), builder: (_, h, f, p) => _Btn('Reset to…', open: _open, hover: h, focus: f, pressed: p))),
        foot: _Foot(m),
        child: _top(
          _open
              ? Container(
                  padding: const EdgeInsets.fromLTRB(_gutter, 8, _gutter, 8),
                  decoration: BoxDecoration(color: N.g13, border: Border(bottom: BorderSide(color: _rule))),
                  child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
                    for (var i = 0; i < 3; i++)
                      _Ctl(
                        onTap: () => _go(i),
                        onHover: (v) => setState(() => _hov = v ? i : (_hov == i ? null : _hov)),
                        onFocus: (v) => setState(() => _hov = v ? i : (_hov == i ? null : _hov)),
                        builder: (_, h, f, p) => _Target(names[i], sub[i], _diff(i).length, hover: h || (_hov == i && !f && !p && widget.hover != null), focus: f, pressed: p),
                      ),
                  ]),
                )
              : const SizedBox.shrink(),
          _Body(children: _secs(_allGroups, (r) => _i6Label(r, m, ghost: ghost, dock: widget.w < 250))),
        ),
      ),
    );
  }
}

// ---- I3 ----

Widget _i3Rows(_M m, {double w = _panelW, _Gr g = _Gr.face, Widget Function(_R, Widget)? wrap}) => _Body(
      children: _secs([_gTransform, _gBlur, _gGlow, _gWave], (r) => _prow(r, m, g: g, dock: w < 250, trailW: 0, wrapLabel: wrap == null ? null : (t) => wrap(r, t))),
    );

Widget _i3aPanel(_M m, {double w = _panelW, Widget? head}) => _Frame(
      w: w,
      head: head ?? _Head(name: w < 250 ? _longName : 'Jewel Field'),
      foot: _Foot(m),
      child: _i3Rows(m, w: w),
    );

class _I3b extends StatelessWidget {
  const _I3b(this.m, {required this.space, this.w = _panelW});
  final _M m;
  final int space; // 0 2D, 1 2.5D, 2 3D
  final double w;
  @override
  Widget build(BuildContext context) {
    final dock = w < 250, cw = w - 2 * _gutter - 2, half = (cw - 8) / 2;
    final pos = m.rows['pos.x']!, rot = m.rows['rot']!, sc = m.rows['scale.x']!, an = m.rows['anchor.x']!;
    Widget lab(String t, {Widget? trailing}) => SizedBox(height: 24, child: Row(children: [Expanded(child: Text(t, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(_labRest))), ?trailing]));
    final body = _Body(children: [
      lab('Position'),
      _Pad(m, w: cw, h: dock ? 84 : 112),
      _gap(4),
      _cellsOf(pos, m, dock: dock),
      _gap(8),
      Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
        SizedBox(width: half, child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [lab('Rotation'), Center(child: _Dial(m, size: dock ? 84 : 100)), _gap(4), _cellsOf(rot, m, dock: true)])),
        const SizedBox(width: 8),
        SizedBox(width: half, child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [lab('Scale'), Center(child: _ScaleBox(m, size: dock ? 84 : 100)), _gap(4), _cellsOf(sc, m, dock: true)])),
      ]),
      _gap(8),
      lab('Anchor Point'),
      Row(crossAxisAlignment: CrossAxisAlignment.start, children: [_Anchor9(m), const SizedBox(width: 8), Expanded(child: _cellsOf(an, m, dock: true))]),
      if (space == 2) _Sec(_gSpace.title, [for (final r in _gSpace.rows) _prow(r, m, dock: dock, trailW: 0)]),
      _Sec('Layer  レイヤー', [_prow(m.rows['op']!, m, dock: dock, trailW: 0)]),
      ..._secs([_gBlur, _gGlow], (r) => _prow(r, m, dock: dock, trailW: 0), leadRule: true),
    ]);
    return _Frame(w: w, head: _Head(name: dock ? _longName : 'Jewel Field', trailing: _count(const ['2D', '2.5D', '3D'][space], tone: N.g76)), foot: _Foot(m), child: body);
  }
}

class _I3c extends StatefulWidget {
  const _I3c({this.w = _panelW});
  final double w;
  @override
  State<_I3c> createState() => _I3cState();
}

class _I3cState extends State<_I3c> {
  final _ms = [_spaceModel(), _spaceModel()..v['pos.x'] = 540..v['pos.y'] = 300..v['rot'] = -8..v['scale.x'] = 140..v['scale.y'] = 140];
  final Set<String> _open = {'pos.x'};
  int _layer = 0;
  @override
  void initState() {
    super.initState();
    for (final m in _ms) {
      m.addListener(() => setState(() {}));
    }
  }

  @override
  void dispose() {
    for (final m in _ms) {
      m.dispose();
    }
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final m = _ms[_layer], dock = widget.w < 250, cw = widget.w - 2 * _gutter - 2;
    void toggle(String id) => setState(() => _open.contains(id) ? _open.remove(id) : _open.add(id));
    Widget unfold(String id, String title, Widget instrument) => _UnfoldRow(title: title, row: m.rows[id]!, m: m, open: _open.contains(id), onToggle: () => toggle(id), instrument: instrument, dock: dock);
    return _story(
      'I3-c${dock ? ' dock' : ''}',
      _Hab.addition,
      'one number row each; the arrow at its right opens that row\'s instrument under it. The number stays put and the open folds are kept when you pick another layer.',
      _Frame(
        w: widget.w,
        head: _Head(
          name: dock ? _longName : (_layer == 0 ? 'Jewel Field' : 'Hero title'),
          trailing: Row(mainAxisSize: MainAxisSize.min, children: [
            Chip('Field', on: _layer == 0, onTap: () => setState(() => _layer = 0)),
            const SizedBox(width: 4),
            Chip('Title', on: _layer == 1, onTap: () => setState(() => _layer = 1)),
          ]),
        ),
        foot: _Foot(m),
        child: _Body(children: [
            const _Sec('Transform  トランスフォーム', [], first: true),
            unfold('anchor.x', 'Anchor', Align(alignment: Alignment.centerLeft, child: _Anchor9(m))),
            unfold('pos.x', 'Position', _Pad(m, w: cw, h: dock ? 80 : 108)),
            unfold('scale.x', 'Scale', Align(alignment: Alignment.centerLeft, child: _ScaleBox(m, size: dock ? 84 : 100))),
            unfold('rot', 'Rotation', Align(alignment: Alignment.centerLeft, child: _Dial(m, size: dock ? 84 : 100))),
            _prow(m.rows['op']!, m, dock: dock),
            ..._secs([_gBlur, _gGlow, _gWave], (r) => _prow(r, m, dock: dock), leadRule: true),
        ]),
      ),
    );
  }
}

/// A label that is a handle that pulls a hand out on the Stage (I3-d).
class _Grab extends StatefulWidget {
  const _Grab({required this.kind, required this.m, required this.onHand, required this.child});
  final _Hand kind;
  final _M m;
  final void Function(_Hand) onHand;
  final Widget child;
  static const k = 1920 / 336; // comp px per Stage px of the mock Stage
  @override
  State<_Grab> createState() => _GrabState();
}

class _GrabState extends State<_Grab> {
  bool _live = false;
  @override
  Widget build(BuildContext context) => MouseRegion(
        cursor: SystemMouseCursors.grab,
        child: GestureDetector(
          behavior: HitTestBehavior.opaque,
          onPanStart: (_) {
            _live = true;
            widget.m.begin();
            widget.onHand(widget.kind);
          },
          onPanUpdate: (d) {
            if (!_live) {
              return;
            }
            final m = widget.m, f = HardwareKeyboard.instance.isAltPressed ? .1 : 1.0;
            switch (widget.kind) {
              case _Hand.pos:
                m.add('pos.x', d.delta.dx * _Grab.k * f);
                m.add('pos.y', d.delta.dy * _Grab.k * f);
              case _Hand.rot:
                m.add('rot', d.delta.dx * f);
              case _Hand.scale:
                m.add('scale.x', d.delta.dx * .5 * f);
                m.add('scale.y', d.delta.dx * .5 * f);
              case _Hand.none:
                break;
            }
          },
          onPanEnd: (_) {
            _live = false;
            widget.m.end('grab ${widget.kind.name}');
            widget.onHand(_Hand.none);
          },
          onPanCancel: () {
            _live = false;
            widget.m.cancel();
            widget.onHand(_Hand.none);
          },
          child: widget.child,
        ),
      );
}

class _I3d extends StatefulWidget {
  const _I3d();
  @override
  State<_I3d> createState() => _I3dState();
}

class _I3dState extends State<_I3d> {
  final m = _M(keys: const {});
  _Hand _hand = _Hand.none;
  @override
  void initState() {
    super.initState();
    m.addListener(() => setState(() {}));
  }

  @override
  void dispose() {
    m.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final grab = {'pos.x': _Hand.pos, 'rot': _Hand.rot, 'scale.x': _Hand.scale};
    return _story(
      'I3-d',
      _Hab.withdrawn,
      'hold the label Position, Rotation or Scale: its hand appears on the Stage and the drag goes on there; release and it is gone, Esc puts it back.',
      _Frame(
        w: _panelW,
        head: const _Head(name: 'Jewel Field'),
        foot: _Foot(m),
        child: _i3Rows(m, wrap: (r, t) => grab.containsKey(r.id) ? _Grab(kind: grab[r.id]!, m: m, onHand: (h) => setState(() => _hand = h), child: t) : t),
      ),
      side: _Stage(m, hand: _hand, interactive: false),
    );
  }
}

// ---- the set: one component per problem group, 'Parts @200%' first in each ----
