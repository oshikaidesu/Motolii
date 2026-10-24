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
part 'panel_inspector_a_kit_2.dart';
part 'panel_inspector_a_kit_3.dart';
part 'panel_inspector_a_kit_4.dart';
part 'panel_inspector_a_space.dart';
part 'panel_inspector_a_space_2.dart';
part 'panel_inspector_a_ease.dart';
part 'panel_inspector_a_ease_2.dart';
part 'panel_inspector_a_parts.dart';
part 'panel_inspector_a_parts_2.dart';
part 'panel_inspector_a_2.dart';
part 'panel_inspector_a_3.dart';

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
