// Panel drafts, Inspector A: options-inspector.md I2 (numbers), I3 (space), I4 (key state), I5 (ease), I6 (diff / reset).
// A draft is a PANEL (282 px, or a 220 px dock) with real names and 30+ rows, not a flow. State is local to the panel; the knobs only switch variants.
// Registered by the orchestrator: panelInspectorASets() (a List of components, one per problem group). Parts: _kit (model, number cell, rows), _space (instruments, mock Stage), _ease (curves).
import 'dart:async';
import 'dart:collection';
import 'dart:math' as math;

import 'package:flutter/gestures.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import '../../parts/controls.dart' show PillSwitch, Segmented;
import '../../tokens.dart';
import '../inspector/inspector_parts.dart' show Chip, Hov, KeyS;

part 'panel_inspector_a_kit.dart';
part 'panel_inspector_a_space.dart';
part 'panel_inspector_a_ease.dart';
part 'panel_inspector_a_parts.dart';

// ---- shared panel builders ----

List<Widget> _secs(List<_Grp> gs, Widget Function(_R) row, {Widget? Function(_Grp)? trailing, String? Function(_Grp)? hint, bool Function(_Grp)? skip, bool leadRule = false}) => [
      for (final (i, g) in gs.indexed)
        if (!(skip?.call(g) ?? false)) _Sec(g.title, [for (final r in g.rows) row(r)], first: i == 0 && !leadRule, trailing: trailing?.call(g), hint: hint?.call(g)),
    ];

/// A fixed top (strip, ruler, bar) over a scrolling body. Fit list: the body is as tall as its rows, up to the window; 700 tall: the body fills the frame.
Widget _top(Widget top, Widget body) => Builder(
      builder: (context) => _FitMode.of(context)
          ? Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [top, Flexible(child: body)])
          : Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [top, Expanded(child: body)]),
    );

const _longName = 'Hero title – メインタイトル（書き出し用）_v12 final';

// ---- I2: one number cell, four grammars ----

const _i2Hint = 'Also wheel, ↑↓, Shift x10, Alt x0.1, Esc.';

Widget _i2Panel(_M m, {_Gr g = _Gr.face, double w = _panelW, String name = 'Jewel Field', String kind = 'Shape group', int groups = 5, Widget? headTrailing}) {
  final dock = w < 250;
  return _Frame(
    w: w,
    head: _Head(name: name, kind: kind, trailing: headTrailing),
    foot: _Foot(m),
    child: _Body(children: _secs(_allGroups.take(groups).toList(), (r) => _prow(r, m, g: g, dock: dock, trailW: 0))),
  );
}

Widget _i2Case(String id, _Hab hab, String contract, {required _Gr g, double w = _panelW, int groups = 5, String name = 'Jewel Field', String kind = 'Shape group', Map<String, List<double>>? several, Widget? side}) =>
    _Host(
      make: () => _M(keys: const {})..mix(several ?? const {}),
      build: (ctx, m) => _story(id, hab, contract, _i2Panel(m, g: g, w: w, groups: groups, name: name, kind: kind), side: side),
    );

Widget _i2States() => _Host(
      make: () => _M(keys: const {})..mix({'pos.x': [800, 960, 1180], 'op': [100, 80, 60]})..log = '"abc" is not a number: kept 1.35',
      build: (ctx, m) {
        final r = m.rows;
        Widget row(String id, {Widget? lead, bool enabled = true, String? note, bool bad = false}) => _shell(
              label: Padding(padding: const EdgeInsets.only(right: 8), child: Row(children: [if (lead != null) ...[lead, const SizedBox(width: 4)], Expanded(child: Text(r[id]!.label, maxLines: 1, overflow: TextOverflow.ellipsis, style: _labStyle(changed: _chg(m, id), enabled: enabled)))])),
              cells: bad ? _Cell(m, r[id]!.cells.first, startBad: true) : _cellsOf(r[id]!, m, enabled: enabled),
              trailW: 0,
              under: note == null ? null : Padding(padding: const EdgeInsets.only(left: _labW, bottom: 4), child: Text(note, maxLines: 2, overflow: TextOverflow.ellipsis, style: T.label(N.g76).copyWith(height: 1.3))),
            );
        Widget title(String t) => Padding(
              padding: const EdgeInsets.only(top: 12, bottom: 4),
              child: Text(t, style: T.micro(N.g76).copyWith(letterSpacing: 1)),
            );
        return _story(
          'I2 states',
          _Hab.habit,
          'the number field in the states it must say out loud: several layers (a differing value reads —), locked (hatched, no outline), driven (hue dot and the driver\'s name; edit it in Relations), at its limit, rejected text (type abc to retry). A changed value carries a corner mark and a dotted underline on its label.',
          _Frame(
            w: _panelW,
            head: const _Head(name: '3 layers', kind: 'Shape groups'),
            foot: _Foot(m),
            child: _Body(children: [
              title('NORMAL'),
              row('rot'),
              title('SEVERAL'),
              row('pos.x'),
              row('op'),
              title('LOCKED'),
              row('scale.x', enabled: false),
              title('DRIVEN'),
              row('blur.a', enabled: false, lead: Container(width: 8, height: 8, decoration: BoxDecoration(color: Role.linkedFor(Fam.follow), shape: BoxShape.circle)), note: 'Follow · Jewel 02'),
              title('AT LIMIT'),
              row('turb.c'),
              title('NOT A NUMBER'),
              row('glow.i', bad: true),
            ]),
          ),
        );
      },
    );

// ---- I4: key state, shared pieces ----

/// The playhead of the draft: a time, a scrubbable ruler with every key of the layer on it.
class _Playhead extends StatelessWidget {
  const _Playhead(this.m, {this.dock = false});
  final _M m;
  final bool dock;
  @override
  Widget build(BuildContext context) {
    final keys = <int>{for (final t in m.tracks.values) ...t.keys};
    return Container(
      padding: const EdgeInsets.fromLTRB(_gutter, 8, _gutter, 8),
      decoration: BoxDecoration(color: N.g13, border: Border(bottom: BorderSide(color: _rule))),
      child: Row(children: [
        SizedBox(width: dock ? 56 : 64, child: Text(_fmtT(m.frame), style: T.value(N.g95))),
        Expanded(
          child: LayoutBuilder(builder: (context, box) {
            void at(Offset p) => m.seek(((p.dx / box.maxWidth) * _end).round());
            return GestureDetector(
              behavior: HitTestBehavior.opaque,
              onPanDown: (d) => at(d.localPosition),
              onPanUpdate: (d) => at(d.localPosition),
              child: CustomPaint(size: Size(box.maxWidth, 24), painter: _RulerPaint(m.frame, keys)),
            );
          }),
        ),
      ]),
    );
  }
}

class _RulerPaint extends CustomPainter {
  const _RulerPaint(this.frame, this.keys);
  final int frame;
  final Set<int> keys;
  @override
  void paint(Canvas c, Size z) {
    double x(int f) => f / _end * z.width;
    for (var s = 0; s <= 4; s++) {
      c.drawLine(Offset(x(s * _fps), 12), Offset(x(s * _fps), z.height), Paint()..color = N.g38);
    }
    for (final k in keys) {
      final o = Offset(x(k), 16), r = 3.2;
      c.drawPath(Path()..moveTo(o.dx, o.dy - r)..lineTo(o.dx + r, o.dy)..lineTo(o.dx, o.dy + r)..lineTo(o.dx - r, o.dy)..close(), Paint()..color = N.g76);
    }
    c.drawLine(Offset(x(frame), 0), Offset(x(frame), z.height), Paint()..color = C.playhead..strokeWidth = 1.5);
    c.drawCircle(Offset(x(frame), 3), 3, Paint()..color = C.playhead);
  }

  @override
  bool shouldRepaint(_RulerPaint o) => o.frame != frame || o.keys.length != keys.length;
}

Widget _keyRow(_R r, _M m, {bool dock = false, bool will = false, Widget? trailOverride, double? trailW}) => _prow(
      r, m,
      dock: dock,
      trailW: trailW,
      trail: trailOverride ?? (r.keyable && r.isNum ? _Key(m.stateOf(r.id), will: will && m.stateOf(r.id) == KeyS.off, onTap: () => m.toggleKey(r)) : null),
    );

Widget _i4aPanel(_M m, {double w = _panelW}) {
  final dock = w < 250;
  return _Frame(
    w: w,
    head: _Head(name: dock ? _longName : 'Jewel Field', trailing: Text('${m.tracks.length} animated', style: T.label(N.g76))),
    foot: _Foot(m),
    child: _top(Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [_Playhead(m, dock: dock)]), _Body(children: _secs(_allGroups, (r) => _keyRow(r, m, dock: dock)))),
  );
}

// ---- I4-b ----

/// The Animate bar: the switch with its noun, and on the right the mark every static row will wear (a hollow diamond) with how many rows that is. On: the bar's lower edge and the diamond take the accent; Off: both are grey. Over it the playhead.
class _AnimateBar extends StatelessWidget {
  const _AnimateBar(this.m, {this.dock = false});
  final _M m;
  final bool dock;
  @override
  Widget build(BuildContext context) {
    final on = m.animate;
    final will = {for (final r in m.rows.values) if (r.keyable && r.isNum && m.stateOf(r.id) == KeyS.off) r}.length;
    return Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      Container(
        height: 32,
        padding: const EdgeInsets.symmetric(horizontal: _gutter),
        decoration: BoxDecoration(color: N.g13, border: Border(bottom: BorderSide(color: on ? Role.selected.withValues(alpha: .85) : _rule))),
        child: Row(children: [
          PillSwitch(on: on, onChanged: (_) => m.toggleAnimate()),
          const SizedBox(width: 8),
          Text('Animate', style: T.name(on ? N.g95 : N.g76)),
          const Spacer(),
          SizedBox(width: _keyW, height: _cellH, child: CustomPaint(painter: _KeyPaint(KeyS.off, false, on, true))),
          SizedBox(width: 20, child: Text('$will', textAlign: TextAlign.right, style: T.value(on ? N.g95 : N.g76))),
        ]),
      ),
      _Playhead(m, dock: dock),
    ]);
  }
}

Widget _i4bPanel(_M m, {double w = _panelW}) => _Frame(
      w: w,
      head: _Head(name: w < 250 ? _longName : 'Jewel Field', trailing: Text('${m.tracks.length} animated', style: T.label(N.g76))),
      foot: _Foot(m),
      child: _top(
        _AnimateBar(m, dock: w < 250),
        _Body(
          children: _secs(_allGroups, (r) {
            final show = r.keyable && r.isNum;
            final st = m.stateOf(r.id), will = m.animate && show && st == KeyS.off;
            return _prow(r, m, dock: w < 250, trail: !show ? null : _Key(st, will: will, onTap: () => m.toggleKey(r)));
          }),
        ),
      ),
    );

// ---- I4-c: a lane in the row (withdrawn in the ledger, kept as a draft) ----

const _gLane = _Grp('Transform  トランスフォーム', [
  _R('Position X', [_P('pos.x', '', 960, unit: 'px', dec: 1)]),
  _R('Position Y', [_P('pos.y', '', 540, unit: 'px', dec: 1)]),
  _R('Scale', [_P('scale.x', '', 100, unit: '%', dec: 1)]),
  _R('Rotation', [_P('rot', '', 0, unit: '°', dec: 1)]),
  _R('Opacity', [_P('op', '', 100, unit: '%', min: 0, max: 100)]),
]);

const _gLane2 = _Grp('Effects  エフェクト', [
  _R('Blurriness', [_P('blur.a', '', 0, unit: 'px', dec: 1, min: 0, max: 250)]),
  _R('Glow Radius', [_P('glow.r', '', 10, unit: 'px', dec: 1, min: 0)]),
  _R('Glow Intensity', [_P('glow.i', '', 1, dec: 2, min: 0)]),
  _R('Wave Height', [_P('wave.h', '', 20, unit: 'px', dec: 1)]),
  _R('Amount', [_P('turb.a', '', 50, dec: 1)]),
  _R('Evolution', [_P('turb.e', '', 0, unit: '°', dec: 1)]),
  _R('Glow Threshold', [_P('glow.t', '', 60, unit: '%', min: 0, max: 100)]),
  _R('Wave Speed', [_P('wave.s', '', 1, dec: 2)]),
  _R('Direction', [_P('wave.d', '', 90, unit: '°', dec: 1)]),
  _R('Phase', [_P('wave.p', '', 0, unit: '°', dec: 1)]),
  _R('Softness', [_P('sh.f', '', 0, unit: 'px', dec: 1)]),
  _R('Distance', [_P('sh.x', '', 5, unit: 'px', dec: 1)]),
  _R('Tracking', [_P('tx.t', '', 0)]),
  _R('Font Size', [_P('tx.s', '', 48, unit: 'px', dec: 1, min: 1)]),
]);

class _Lane extends StatefulWidget {
  const _Lane(this.m, this.r);
  final _M m;
  final _R r;
  @override
  State<_Lane> createState() => _LaneState();
}

class _LaneState extends State<_Lane> {
  int? _drag;
  int _at(double dx, double w) => (dx / w * _end).round().clamp(0, _end);
  int? _near(double dx, double w) {
    final t = widget.m.tracks[widget.r.id];
    if (t == null) {
      return null;
    }
    int? best;
    var bd = 8.0;
    for (final k in t.keys) {
      final d = (k / _end * w - dx).abs();
      if (d < bd) {
        bd = d;
        best = k;
      }
    }
    return best;
  }

  @override
  Widget build(BuildContext context) {
    final m = widget.m, id = widget.r.id;
    final keys = m.tracks[id]?.keys.toList() ?? const <int>[];
    return LayoutBuilder(builder: (context, box) {
      final w = box.maxWidth;
      return GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTapUp: (d) {
          final k = _near(d.localPosition.dx, w);
          if (k != null) {
            m.seek(k, why: 'lane: playhead to a key of ${widget.r.label}');
          } else {
            m.begin();
            final f = _at(d.localPosition.dx, w);
            m.frame = f;
            m.tracks.putIfAbsent(id, () => SplayTreeMap<int, double>())[f] = m.at(id);
            m.end('lane: key ${widget.r.label} @ ${_fmtT(f)}');
          }
        },
        onSecondaryTapUp: (d) {
          final k = _near(d.localPosition.dx, w);
          if (k != null) {
            m.begin();
            m.tracks[id]!.remove(k);
            if (m.tracks[id]!.isEmpty) {
              m.tracks.remove(id);
            }
            m.end('lane: removed key of ${widget.r.label}');
          }
        },
        onPanStart: (d) {
          _drag = _near(d.localPosition.dx, w);
          if (_drag != null) {
            m.begin();
          }
        },
        onPanUpdate: (d) {
          final k = _drag;
          if (k == null) {
            return;
          }
          final f = _at(d.localPosition.dx, w), t = m.tracks[id]!;
          if (f != k && !t.containsKey(f)) {
            t[f] = t.remove(k)!;
            _drag = f;
            m.poke();
          }
        },
        onPanEnd: (_) {
          if (_drag != null) {
            _drag = null;
            m.end('lane: moved a key of ${widget.r.label}');
          }
        },
        child: CustomPaint(size: Size(w, _cellH), painter: _LanePaint(keys, m.frame, keys.isNotEmpty)),
      );
    });
  }
}

class _LanePaint extends CustomPainter {
  const _LanePaint(this.keys, this.frame, this.animated);
  final List<int> keys;
  final int frame;
  final bool animated;
  @override
  void paint(Canvas c, Size z) {
    c.drawRRect(RRect.fromRectAndRadius(Offset.zero & z, const Radius.circular(4)), Paint()..color = N.g07);
    final y = z.height / 2;
    double x(int f) => f / _end * z.width;
    if (animated) {
      c.drawLine(Offset(x(keys.first), y), Offset(x(keys.last), y), Paint()..color = N.g44..strokeWidth = 1);
    } else {
      c.drawLine(Offset(0, y), Offset(z.width, y), Paint()..color = N.g20..strokeWidth = 1);
    }
    for (final k in keys) {
      final o = Offset(x(k), y), r = k == frame ? 4.6 : 3.8;
      final d = Path()..moveTo(o.dx, o.dy - r)..lineTo(o.dx + r, o.dy)..lineTo(o.dx, o.dy + r)..lineTo(o.dx - r, o.dy)..close();
      c.drawPath(d, Paint()..color = k == frame ? C.playhead : N.g95);
    }
    c.drawLine(Offset(x(frame), 0), Offset(x(frame), z.height), Paint()..color = C.playhead..strokeWidth = 1);
  }

  @override
  bool shouldRepaint(_LanePaint o) => true;
}

Widget _i4cPanel(_M m) => _Frame(
      w: _panelW,
      head: const _Head(name: 'Jewel Field', trailing: null),
      foot: _Foot(m),
      child: _top(
        _Playhead(m),
        _Body(
          children: _secs([_gLane, _gLane2], (r) => _laneRow(r, m)),
        ),
      ),
    );

/// One row of I4-c: name, the number, and the row's own time line.
Widget _laneRow(_R r, _M m) => _shell(
      label: Padding(padding: const EdgeInsets.only(right: 8), child: Text(r.label, maxLines: 1, overflow: TextOverflow.ellipsis, style: _labStyle(changed: _chg(m, r.id)))),
      cells: Row(children: [SizedBox(width: 64, child: _Cell(m, r.cells.first)), const SizedBox(width: 8), Expanded(child: _Lane(m, r))]),
      labW: 80,
      trailW: 0,
    );

// ---- I4-d: the mark jumps to the next key ----

class _KeyNav extends StatelessWidget {
  const _KeyNav(this.m, this.r);
  final _M m;
  final _R r;
  @override
  Widget build(BuildContext context) {
    final st = m.stateOf(r.id);
    return _Key(
      st,
      onTap: () {
        final back = HardwareKeyboard.instance.isShiftPressed;
        final f = m.neighbour(r, back: back);
        if (st == KeyS.off) {
          m.note('${r.label}: no keys');
        } else if (f == null) {
          m.note('${r.label}: no ${back ? 'earlier' : 'later'} key');
        } else {
          m.seek(f, why: 'jump: ${r.label} → ${_fmtT(f)} (${back ? 'previous' : 'next'} key)');
        }
      },
      onLong: () => m.toggleKey(r),
      onSecondary: () => m.toggleKey(r),
    );
  }
}

Widget _i4dPanel(_M m, {double w = _panelW}) => _Frame(
      w: w,
      head: _Head(name: w < 250 ? _longName : 'Jewel Field', trailing: Text('${m.tracks.length} animated', style: T.label(N.g76))),
      foot: _Foot(m),
      child: _top(
        _Playhead(m),
        _Body(children: _secs(_allGroups, (r) => _keyRow(r, m, trailOverride: r.keyable && r.isNum ? _KeyNav(m, r) : null))),
      ),
    );

// ---- I6 ----

/// The strip above an I6-b list: one switch, one word, one count.
class _FilterBar extends StatelessWidget {
  const _FilterBar({required this.on, required this.n, required this.total, this.onChanged});
  final bool on;
  final int n, total;
  final ValueChanged<bool>? onChanged;
  @override
  Widget build(BuildContext context) => Container(
        height: 36,
        padding: const EdgeInsets.symmetric(horizontal: _gutter),
        decoration: BoxDecoration(color: N.g13, border: Border(bottom: BorderSide(color: _rule))),
        child: Row(children: [
          PillSwitch(on: on, onChanged: onChanged),
          const SizedBox(width: 8),
          Expanded(child: Text('Changed', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(on ? N.g95 : N.g76))),
          Text('$n/$total', style: T.label(N.g76)),
        ]),
      );
}

/// What I6-b says when nothing differs: a word, one reason, the one way back. The count of hidden rows is the filter bar's, not repeated here.
class _EmptyCard extends StatelessWidget {
  const _EmptyCard({this.onShow, this.focus = false, this.pressed = false, this.hover = false});
  final VoidCallback? onShow;
  final bool focus, pressed, hover; // pinned states of the button (Parts)
  @override
  Widget build(BuildContext context) => Padding(
        padding: const EdgeInsets.all(_gutter),
        child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
          const SizedBox(height: 24),
          Text('Nothing changed', style: T.title(N.g91)),
          const SizedBox(height: 12),
          Align(alignment: Alignment.centerLeft, child: _Act('Show all rows', on: true, onTap: onShow ?? () {}, hot: hover, focus: focus, pressed: pressed)),
        ]),
      );
}

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
