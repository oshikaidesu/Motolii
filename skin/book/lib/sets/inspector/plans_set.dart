// Inspector plans: one star ring (12 stars round a circle) under four ways of showing its values, touched side by side.
// A today's list (every card open), B four verbs in a diamond + four knobs + a strip (hold on the stage: the verbs as a ring, time stops),
// C only what was touched + verbs that add more, D four macros, each moving several values at once. A mock: not joined to the inside.
import 'dart:math' as math;

import 'package:flutter/scheduler.dart';
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import '../../parts/controls.dart';
import '../../tokens.dart';
part 'plans_set_2.dart';

class _S extends ChangeNotifier {
  final v = <String, double>{
    'count': 12, 'radius': 180, 'size': 44, 'points': 5, 'round': 0, 'turn': 0, 'spin': 30, 'hue': 72, 'light': .62,
    'opacity': 1, 'glow': 0, 'waveH': 0, 'waveS': 1, 'delay': .3, 'scatter': 0, 'layout': 2,
  };
  final other = <String, double>{}; // A's rows that move nothing on the stage
  final touched = <String>{'count', 'layout', 'radius', 'spin', 'hue'};
  final macro = <double>[0, 0, 0, 0];
  bool frozen = false;
  double operator [](String k) => v[k]!;
  void set(String k, double x) {
    final r = _ranges[k]!;
    v[k] = x.clamp(r.$1, r.$2);
    touched.add(k);
    notifyListeners();
  }

  void forget(String k) {
    touched.remove(k);
    notifyListeners();
  }

  void setOther(String k, double x) {
    other[k] = x;
    notifyListeners();
  }

  void freeze(bool f) {
    frozen = f;
    notifyListeners();
  }

  // D: a macro moves each of its values from its low to its high end
  void setMacro(int i, double m) {
    macro[i] = m.clamp(0.0, 1.0);
    for (final (k, lo, hi) in _macros[i].$2) {
      v[k] = lo + (hi - lo) * macro[i];
    }
    notifyListeners();
  }
}

const _ranges = <String, (double, double)>{
  'count': (3, 48), 'radius': (20, 300), 'size': (10, 120), 'points': (3, 12), 'round': (0, 1), 'turn': (0, 360), 'spin': (0, 180),
  'hue': (0, 360), 'light': (.2, .9), 'opacity': (0, 1), 'glow': (0, 1), 'waveH': (0, 90), 'waveS': (0, 4), 'delay': (0, 1),
  'scatter': (0, 90), 'layout': (0, 2),
};
const _jp = {
  'count': '数', 'radius': '広がり', 'size': '大きさ', 'points': '角', 'round': '丸み', 'turn': '回転', 'spin': 'ずれ', 'hue': '色相',
  'light': '明るさ', 'opacity': '不透明', 'glow': '光', 'waveH': '揺れ', 'waveS': '速さ', 'delay': '遅れ', 'scatter': '散らし', 'layout': '並び',
};
const _layouts = ['線', '格子', '輪'];

String _say(String k, double x) => switch (k) {
  'count' || 'points' => '${x.round()}',
  'layout' => _layouts[x.round()],
  'hue' || 'turn' || 'spin' => '${x.round()}°',
  'radius' || 'size' || 'waveH' || 'scatter' => '${x.round()}',
  _ => x.toStringAsFixed(2),
};

const _verbs = ['形', '色', '複製', '動き'];
const _verbKeys = [
  ['size', 'points', 'round', 'turn'],
  ['hue', 'light', 'opacity', 'glow'],
  ['count', 'radius', 'spin', 'layout'],
  ['waveH', 'waveS', 'delay', 'scatter'],
];
const _strip = ['場', '散らす', '線', 'パス', '変形', '再生', 'つなぐ', '光り方', '重ね方'];

const _macros = <(String, List<(String, double, double)>)>[
  ('まとまり', [('radius', 260, 70), ('size', 30, 64), ('spin', 0, 60)]),
  ('うねり', [('waveH', 0, 70), ('waveS', .5, 2.6), ('delay', 0, .8)]),
  ('ざわめき', [('scatter', 0, 60), ('glow', 0, .7), ('round', 0, .6)]),
  ('いろ', [('hue', 20, 340), ('light', .5, .78)]),
];

// Today's cards for a shape (doc.h doc_cards), every value a row
const _cardsA = <(String, List<String>)>[
  ('Fill', ['fill', 'stroke', 'stroke_width']),
  ('Transform', ['space', 'parent', 'anchor_x', 'anchor_y', 'anchor_z', 'x', 'y', 'z', 'scale_x', 'scale_y', 'scale_z', 'rotate_x', 'rotate_y', 'rotate', 'opacity']),
  ('Blending', ['blend', 'clip']),
  ('Copies', ['copies', 'copy_columns', 'copy_x', 'copy_y', 'copy_rotate', 'copy_rotate_x', 'copy_rotate_y', 'copy_turn', 'copy_size', 'copy_wide', 'copy_z', 'tall', 'copy_tall', 'stagger', 'stagger_from', 'copy_fill', 'copy_fill_way']),
  ('Wave', ['wave_x', 'wave_y', 'wave_z', 'wave_rotate', 'wave_size', 'wave_speed', 'wave_phase', 'wave_noise', 'noise_scale']),
  ('Field', ['field_x', 'field_y', 'field_z', 'field_radius', 'field_band', 'field_size', 'field_rotate', 'field_push']),
  ('Scatter', ['scatter_x', 'scatter_y', 'scatter_z', 'scatter_rotate', 'scatter_size', 'seed', 'drift_x', 'drift_y']),
  ('Shine', ['emissive']),
  ('Motion', ['orient']),
  ('Play', ['marker', 'animation', 'speed']),
  ('Connect', ['connect', 'connect_to', 'connect_reach']),
  ('Line', ['cap', 'join', 'dash', 'dash_offset']),
  ('Path', ['round', 'bloat', 'offset', 'zigzag', 'zigzag_ridges', 'zigzag_smooth']),
  ('Deform', ['warp', 'bend', 'twist', 'pinch', 'skew', 'squash', 'wiggle', 'stretch', 'bend_direction', 'pin_tl_x', 'pin_tl_y', 'pin_tr_x', 'pin_tr_y', 'pin_bl_x', 'pin_bl_y', 'pin_br_x', 'pin_br_y', 'warp_width', 'warp_angle', 'warp_speed', 'skew_axis', 'wiggle_detail', 'wiggle_speed']),
];
const _aKey = {
  'copies': 'count', 'copy_x': 'radius', 'copy_rotate': 'spin', 'scale_x': 'size', 'rotate': 'turn', 'opacity': 'opacity', 'wave_y': 'waveH',
  'wave_speed': 'waveS', 'stagger': 'delay', 'scatter_y': 'scatter', 'emissive': 'glow', 'fill': 'hue', 'round': 'round',
};
final _rowsA = _cardsA.fold<int>(0, (n, c) => n + c.$2.length);

// ---------------------------------------------------------------- the stage

class _Stage extends StatefulWidget {
  const _Stage(this.s, this.tag, {this.onVerb});
  final _S s;
  final String tag;
  final ValueChanged<int>? onVerb; // set: a hold opens the verbs as a ring
  @override
  State<_Stage> createState() => _StageState();
}

class _StageState extends State<_Stage> with SingleTickerProviderStateMixin {
  late final Ticker tk;
  double t = 0;
  Duration last = Duration.zero;
  Offset? hold, at;

  @override
  void initState() {
    super.initState();
    tk = createTicker((d) {
      final dt = (d - last).inMicroseconds / 1e6;
      last = d;
      if (!widget.s.frozen) setState(() => t += dt);
    })..start();
  }

  @override
  void dispose() {
    tk.dispose();
    super.dispose();
  }

  // up 形, left 色, right 複製, down 動き: the diamond's own places
  int? _aim() {
    if (hold == null || at == null) return null;
    final d = at! - hold!;
    if (d.distance < 18) return null;
    if (d.dx.abs() > d.dy.abs()) return d.dx < 0 ? 1 : 2;
    return d.dy < 0 ? 0 : 3;
  }

  @override
  Widget build(BuildContext context) => GestureDetector(
    onLongPressStart: widget.onVerb == null
        ? null
        : (d) {
            setState(() => hold = at = d.localPosition);
            widget.s.freeze(true);
          },
    onLongPressMoveUpdate: (d) => setState(() => at = d.localPosition),
    onLongPressEnd: (d) {
      final a = _aim();
      if (a != null) widget.onVerb?.call(a);
      setState(() => hold = at = null);
      widget.s.freeze(false);
    },
    child: ClipRect(
      child: ColoredBox(
        color: N.g07,
        child: CustomPaint(
          painter: _Ring(widget.s, t),
          child: Stack(children: [
            Positioned(left: 14, top: 12, child: Text(widget.tag, style: T.micro(N.g76))),
            if (widget.onVerb != null && hold == null)
              Positioned(left: 14, bottom: 12, child: Text('長押しで輪', style: T.label(N.g56))),
            if (hold != null) Positioned(left: hold!.dx - 90, top: hold!.dy - 90, child: _HoldRing(_aim())),
          ]),
        ),
      ),
    ),
  );
}

class _Ring extends CustomPainter {
  _Ring(this.s, this.t);
  final _S s;
  final double t;
  @override
  void paint(Canvas c, Size size) {
    final m = size.center(Offset.zero), k = math.min(size.width, size.height) / 720, n = s['count'].round(), lay = s['layout'].round();
    final r = s['radius'], col = HSLColor.fromAHSL(s['opacity'], s['hue'], .62, s['light']).toColor();
    final cols = math.max(1, math.sqrt(n).ceil());
    for (var i = 0; i < n; i++) {
      var p = switch (lay) {
        0 => Offset(n == 1 ? 0 : -r + 2 * r * i / (n - 1), 0),
        1 => Offset(cols == 1 ? 0 : -r + 2 * r * (i % cols) / (cols - 1), cols == 1 ? 0 : -r + 2 * r * (i ~/ cols) / (cols - 1)),
        _ => Offset.fromDirection(2 * math.pi * i / n + s['turn'] * math.pi / 180 - math.pi / 2, r),
      };
      p += Offset(0, -s['waveH'] * math.sin(t * s['waveS'] * 2.4 - i * s['delay'] * 1.4));
      final h = math.sin(i * 12.9898) * 43758.5453;
      p += Offset(math.cos(h), math.sin(h * 1.7)) * s['scatter'];
      final o = m + p * k, rr = s['size'] * k / 2;
      final path = _star(o, rr, s['points'].round(), s['turn'] * math.pi / 180 + i * s['spin'] * math.pi / 180);
      if (s['glow'] > 0) {
        c.drawPath(path, Paint()..color = col.withValues(alpha: s['glow'] * .8)..maskFilter = MaskFilter.blur(BlurStyle.normal, 14 * k * (1 + s['glow'] * 2)));
      }
      c.drawPath(path, Paint()..color = col);
      if (s['round'] > 0) {
        c.drawPath(path, Paint()..color = col..style = PaintingStyle.stroke..strokeJoin = StrokeJoin.round..strokeWidth = rr * s['round'] * .6);
      }
    }
  }

  Path _star(Offset o, double r, int pts, double turn) {
    final p = Path();
    for (var j = 0; j < pts * 2; j++) {
      final a = turn - math.pi / 2 + j * math.pi / pts, q = o + Offset.fromDirection(a, j.isEven ? r : r * .45);
      j == 0 ? p.moveTo(q.dx, q.dy) : p.lineTo(q.dx, q.dy);
    }
    return p..close();
  }

  @override
  bool shouldRepaint(_Ring o) => true;
}

/// A rotated square with an upright word: the shape the verbs share on the stage ring and in B's diamond.
class _Diamond extends StatelessWidget {
  const _Diamond(this.word, {this.on = false, this.size = 54, this.onTap});
  final String word;
  final bool on;
  final double size;
  final VoidCallback? onTap;
  @override
  Widget build(BuildContext context) => GestureDetector(
    onTap: onTap,
    child: SizedBox.square(
      dimension: size * 1.42,
      child: Stack(alignment: Alignment.center, children: [
        Transform.rotate(
          angle: math.pi / 4,
          child: AnimatedContainer(
            duration: Mo.dur,
            curve: Mo.ease,
            width: size,
            height: size,
            decoration: BoxDecoration(
              color: on ? N.g91 : N.g15,
              border: Border.all(color: on ? N.g100 : N.g26),
              borderRadius: BorderRadius.circular(5),
            ),
          ),
        ),
        Text(word, style: T.title(on ? N.g07 : N.g91).copyWith(fontSize: word.length > 1 ? 12 : 15)),
      ]),
    ),
  );
}

class _HoldRing extends StatelessWidget {
  const _HoldRing(this.aim);
  final int? aim;
  @override
  Widget build(BuildContext context) => SizedBox.square(
    dimension: 180,
    child: Stack(children: [
      Positioned(left: 60, top: 0, child: _Diamond(_verbs[0], on: aim == 0, size: 42)),
      Positioned(left: 0, top: 60, child: _Diamond(_verbs[1], on: aim == 1, size: 42)),
      Positioned(left: 120, top: 60, child: _Diamond(_verbs[2], on: aim == 2, size: 42)),
      Positioned(left: 60, top: 120, child: _Diamond(_verbs[3], on: aim == 3, size: 42)),
    ]),
  );
}

// ---------------------------------------------------------------- knobs

/// A round knob, dragged up and down; the arc is the value, the word sits under it.
class _Dial extends StatelessWidget {
  const _Dial(this.s, this.k);
  final _S s;
  final String k;
  static const size = 54.0;
  @override
  Widget build(BuildContext context) {
    if (k == 'layout') {
      return SizedBox(
        width: size + 12,
        child: Column(mainAxisSize: MainAxisSize.min, children: [
          SizedBox(
            height: size,
            child: Center(child: Segmented(items: _layouts, index: s['layout'].round(), expand: true, onChanged: (i) => s.set('layout', i.toDouble()))),
          ),
          const SizedBox(height: 6),
          Text(_jp[k]!, style: T.label(N.g76)),
        ]),
      );
    }
    final (lo, hi) = _ranges[k]!;
    return GestureDetector(
      onVerticalDragUpdate: (d) => s.set(k, s[k] - d.delta.dy / 160 * (hi - lo)),
      child: SizedBox(
        width: size + 12,
        child: Column(mainAxisSize: MainAxisSize.min, children: [
          SizedBox.square(
            dimension: size,
            child: CustomPaint(
              painter: _DialPaint((s[k] - lo) / (hi - lo)),
              child: Center(child: Text(_say(k, s[k]), style: T.value(N.g95).copyWith(fontSize: size > 60 ? 13 : 11))),
            ),
          ),
          const SizedBox(height: 6),
          Text(_jp[k] ?? k, style: T.label(N.g76)),
        ]),
      ),
    );
  }
}

class _DialPaint extends CustomPainter {
  _DialPaint(this.t, {this.color});
  final double t;
  final Color? color;
  @override
  void paint(Canvas c, Size s) {
    final rect = Offset.zero & s, r = rect.deflate(3), a0 = math.pi * .75, sweep = math.pi * 1.5;
    c.drawCircle(s.center(Offset.zero), s.width / 2 - 7, Paint()..color = N.g13);
    final st = Paint()..style = PaintingStyle.stroke..strokeWidth = 3..strokeCap = StrokeCap.round;
    c.drawArc(r, a0, sweep, false, st..color = N.g26);
    c.drawArc(r, a0, sweep * t.clamp(0.0, 1.0), false, st..color = color ?? N.g95);
  }

  @override
  bool shouldRepaint(_DialPaint o) => o.t != t || o.color != color;
}

Widget _inspector(List<Widget> children, {bool scroll = false}) {
  final body = Column(crossAxisAlignment: CrossAxisAlignment.stretch, mainAxisSize: MainAxisSize.min, children: children);
  return Container(
    width: 300,
    decoration: BoxDecoration(color: N.g10, border: Border(left: BorderSide(color: N.g20))),
    padding: const EdgeInsets.fromLTRB(14, 14, 14, 10),
    child: scroll ? SingleChildScrollView(child: body) : body,
  );
}

Widget _head() => Padding(
  padding: const EdgeInsets.only(bottom: 12),
  child: Row(children: [
    Container(width: 22, height: 22, decoration: BoxDecoration(color: N.g20, borderRadius: BorderRadius.circular(6)), child: const Center(child: Text('★', style: TextStyle(fontSize: 12, color: N.g91)))),
    const SizedBox(width: 9),
    Text('星 ×12 の輪', style: T.title()),
  ]),
);

// ---------------------------------------------------------------- A: today's list

class _PlanA extends StatelessWidget {
  const _PlanA(this.s);
  final _S s;
  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: s,
    builder: (context, _) => Row(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      Expanded(child: _Stage(s, 'A  今のまま　行 $_rowsA')),
      _inspector(scroll: true, [
        _head(),
        for (final (name, rows) in _cardsA) ...[
          Padding(
            padding: const EdgeInsets.only(top: 8, bottom: 4),
            child: Row(children: [Text('▾  $name', style: T.name()), const Spacer(), Text('${rows.length}', style: T.label(N.g56))]),
          ),
          for (final r in rows) _RowA(s, r),
          Container(height: 1, margin: const EdgeInsets.only(top: 6), color: N.g20),
        ],
      ]),
    ]),
  );
}

class _RowA extends StatelessWidget {
  const _RowA(this.s, this.name);
  final _S s;
  final String name;
  @override
  Widget build(BuildContext context) {
    final k = _aKey[name];
    final x = k != null ? s[k] : (s.other[name] ?? 0);
    return GestureDetector(
      onHorizontalDragUpdate: (d) {
        if (k != null) {
          final (lo, hi) = _ranges[k]!;
          s.set(k, x + d.delta.dx / 200 * (hi - lo));
        } else {
          s.setOther(name, x + d.delta.dx / 4);
        }
      },
      child: Padding(
        padding: const EdgeInsets.only(left: 12, top: 2, bottom: 2),
        child: Row(children: [
          Text(name, style: T.label(N.g63)),
          const Spacer(),
          Container(
            width: 64,
            height: 16,
            alignment: Alignment.centerRight,
            padding: const EdgeInsets.symmetric(horizontal: 5),
            decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(3)),
            child: Text(k != null ? _say(k, x) : x.toStringAsFixed(0), style: T.value(N.g91).copyWith(fontSize: 10)),
          ),
        ]),
      ),
    );
  }
}

// ---------------------------------------------------------------- B: verbs in a diamond

class _PlanB extends StatefulWidget {
  const _PlanB(this.s);
  final _S s;
  @override
  State<_PlanB> createState() => _PlanBState();
}

class _PlanBState extends State<_PlanB> {
  int verb = 2, strip = -1;
  @override
  Widget build(BuildContext context) {
    final s = widget.s;
    return ListenableBuilder(
      listenable: s,
      builder: (context, _) => Row(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        Expanded(child: _Stage(s, 'B  動詞4つ＋つまみ4つ＋帯　行 8', onVerb: (i) => setState(() => verb = i))),
        _inspector([
          _head(),
          Center(
            child: SizedBox.square(
              dimension: 200,
              child: Stack(children: [
                for (final (i, x, y) in const [(0, 61.0, 0.0), (1, 0.0, 61.0), (2, 122.0, 61.0), (3, 61.0, 122.0)])
                  Positioned(left: x, top: y, child: _Diamond(_verbs[i], on: verb == i, onTap: () => setState(() => verb = i))),
              ]),
            ),
          ),
          const SizedBox(height: 18),
          Row(mainAxisAlignment: MainAxisAlignment.spaceBetween, children: [for (final k in _verbKeys[verb]) _Dial(s, k)]),
          const Spacer(),
          Container(height: 1, color: N.g20, margin: const EdgeInsets.only(bottom: 8)),
          Wrap(spacing: 5, runSpacing: 5, children: [
            for (final (i, w) in _strip.indexed)
              GestureDetector(
                onTap: () => setState(() => strip = strip == i ? -1 : i),
                child: Container(
                  padding: const EdgeInsets.symmetric(horizontal: 7, vertical: 4),
                  decoration: BoxDecoration(color: strip == i ? N.g26 : N.g13, borderRadius: BorderRadius.circular(3)),
                  child: Text(w, style: T.label(strip == i ? N.g95 : N.g63)),
                ),
              ),
          ]),
        ]),
      ]),
    );
  }
}

// ---------------------------------------------------------------- C: only what was touched
