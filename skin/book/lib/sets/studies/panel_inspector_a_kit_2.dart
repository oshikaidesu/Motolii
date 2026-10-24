// Part of panel_inspector_a_kit.dart (cut there by size, as it was).
part of 'panel_inspector_a.dart';

class _Grp {
  const _Grp(this.title, this.rows);
  final String title;
  final List<_R> rows;
}

const _gTransform = _Grp('Transform  トランスフォーム', [
  _R('Anchor Point', [_P('anchor.x', 'X', 960, unit: 'px', dec: 1), _P('anchor.y', 'Y', 540, unit: 'px', dec: 1)]),
  _R('Position', [_P('pos.x', 'X', 960, unit: 'px', dec: 1), _P('pos.y', 'Y', 540, unit: 'px', dec: 1)]),
  _R('Scale', [_P('scale.x', 'X', 100, unit: '%', dec: 1), _P('scale.y', 'Y', 100, unit: '%', dec: 1)]),
  _R('Rotation', [_P('rot', '', 0, unit: '°', dec: 1)]),
  _R('Opacity', [_P('op', '', 100, unit: '%', min: 0, max: 100)]),
]);

const _gBlur = _Grp('Gaussian Blur  ガウスぼかし', [
  _R('Blurriness', [_P('blur.a', '', 0, unit: 'px', dec: 1, min: 0, max: 250)]),
  _R('Blur Dimensions', [_P('blur.d', '', 0, kind: _K.choice, options: ['Horizontal and Vertical', 'Horizontal', 'Vertical'])], keyable: false),
  _R('Repeat Edge Pixels', [_P('blur.e', '', 0, kind: _K.toggle)], keyable: false),
]);

const _gGlow = _Grp('Glow  グロー', [
  _R('Glow Threshold', [_P('glow.t', '', 60, unit: '%', min: 0, max: 100)]),
  _R('Glow Radius', [_P('glow.r', '', 10, unit: 'px', dec: 1, min: 0)]),
  _R('Glow Intensity', [_P('glow.i', '', 1, dec: 2, min: 0)]),
  _R('Glow Colors', [_P('glow.c', '', 0, kind: _K.choice, options: ['Original Colors', 'A & B Colors', 'Arbitrary Map'])], keyable: false),
]);

const _gWave = _Grp('Wave Warp  波形ワープ', [
  _R('Wave Type', [_P('wave.k', '', 0, kind: _K.choice, options: ['Sine', 'Triangle', 'Square', 'Noise'])], keyable: false),
  _R('Wave Height', [_P('wave.h', '', 20, unit: 'px', dec: 1)]),
  _R('Wave Width', [_P('wave.w', '', 20, unit: 'px', dec: 1)]),
  _R('Direction', [_P('wave.d', '', 90, unit: '°', dec: 1)]),
  _R('Wave Speed', [_P('wave.s', '', 1, dec: 2)]),
  _R('Phase', [_P('wave.p', '', 0, unit: '°', dec: 1)]),
]);

const _gTurb = _Grp('Turbulent Displace  タービュレント', [
  _R('Amount', [_P('turb.a', '', 50, dec: 1)]),
  _R('Size', [_P('turb.s', '', 100, dec: 1)]),
  _R('Offset (Turbulence)', [_P('turb.x', 'X', 960, unit: 'px', dec: 1), _P('turb.y', 'Y', 540, unit: 'px', dec: 1)]),
  _R('Complexity', [_P('turb.c', '', 1, dec: 1, min: 1, max: 10)]),
  _R('Evolution', [_P('turb.e', '', 0, unit: '°', dec: 1)]),
  _R('Evolution Options: Cycle Evolution Revolutions', [_P('turb.r', '', 1, dec: 1)]),
]);

const _gShadow = _Grp('Drop Shadow  ドロップシャドウ', [
  _R('Shadow Color', [_P('sh.c', '', 0, kind: _K.choice, options: ['Black', 'Brand 04', 'Custom'])], keyable: false),
  _R('Opacity', [_P('sh.o', '', 50, unit: '%', min: 0, max: 100)]),
  _R('Direction', [_P('sh.d', '', 135, unit: '°', dec: 1)]),
  _R('Distance', [_P('sh.x', '', 5, unit: 'px', dec: 1)]),
  _R('Softness', [_P('sh.f', '', 0, unit: 'px', dec: 1)]),
]);

const _gText = _Grp('Text  テキスト', [
  _R('Font Size', [_P('tx.s', '', 48, unit: 'px', dec: 1, min: 1)]),
  _R('Tracking', [_P('tx.t', '', 0, dec: 0)]),
  _R('Leading', [_P('tx.l', '', 120, unit: '%', dec: 0)]),
]);

const _allGroups = [_gTransform, _gBlur, _gGlow, _gWave, _gTurb, _gShadow, _gText];

/// What the layer looks like when the panel opens: some values already changed, some animated (so the diff and the key states have something to say).
const _startValues = <String, double>{
  'pos.x': 1180.5, 'pos.y': 540, 'scale.x': 112, 'scale.y': 112, 'rot': 15, 'op': 80, 'blur.a': 24, 'blur.e': 1, 'glow.r': 38, 'glow.i': 1.35,
  'wave.h': 34, 'turb.a': 62, 'sh.o': 65, 'sh.f': 12, 'tx.t': 40, 'blur.d': 1,
};

const _startKeys = <String, Map<int, double>>{
  'pos.x': {0: 960, 48: 1180.5, 96: 1320},
  'pos.y': {0: 540, 48: 540, 96: 400},
  'rot': {12: 0, 84: 15},
  'op': {0: 0, 24: 100, 90: 80},
  'blur.a': {30: 0, 60: 24},
  'glow.r': {0: 10, 72: 38},
};

String _two(int n) => n.toString().padLeft(2, '0');
String _fmtT(int f) => '00:${_two(f ~/ 30)}:${_two(f % 30)}';
const _fps = 30, _end = 120;

double _textW(String t, TextStyle st) => (TextPainter(text: TextSpan(text: t, style: st), textDirection: TextDirection.ltr, maxLines: 1)..layout()).width;

String _fmtNum(double v, int dec) {
  var d = dec;
  var s = v.toStringAsFixed(d);
  while (v != 0 && double.parse(s) == 0 && d < 4) {
    d++;
    s = v.toStringAsFixed(d);
  }
  return s == '-0' ? '0' : s;
}

double _round(double v) => (v * 10000).round() / 10000;

/// The scrub multiplier of the one rule (D-I3): Shift x10, Alt x0.1.
double _mult() {
  final k = HardwareKeyboard.instance;
  if (k.isShiftPressed) {
    return 10;
  }
  if (k.isAltPressed) {
    return .1;
  }
  return 1;
}

class _M extends ChangeNotifier {
  _M({List<_Grp> groups = _allGroups, Map<String, double> start = _startValues, Map<String, Map<int, double>> keys = _startKeys, this.animated = true}) {
    for (final g in groups) {
      for (final r in g.rows) {
        rows[r.id] = r;
        for (final p in r.cells) {
          byId[p.id] = p;
          def[p.id] = p.def;
          v[p.id] = start[p.id] ?? p.def;
        }
      }
    }
    if (animated) {
      keys.forEach((id, k) {
        if (byId.containsKey(id)) {
          tracks[id] = SplayTreeMap<int, double>.of(k);
        }
      });
    }
  }
  final bool animated;
  final v = <String, double>{}, def = <String, double>{};
  final byId = <String, _P>{};
  final rows = <String, _R>{};
  final tracks = <String, SplayTreeMap<int, double>>{};
  final sev = <String, List<double>>{}; // several layers selected: one value per layer
  int frame = 48, steps = 0;
  bool animate = false;
  String log = 'nothing yet';
  Map<String, double>? _sv;
  Map<String, Map<int, double>>? _st;
  Map<String, List<double>>? _ss;

  bool isMixed(String id) {
    final s = sev[id];
    return s != null && s.any((x) => x != s.first);
  }

  double at(String id) {
    final t = tracks[id];
    if (t == null || t.isEmpty) {
      return v[id]!;
    }
    final f = frame;
    if (f <= t.firstKey()!) {
      return t[t.firstKey()]!;
    }
    if (f >= t.lastKey()!) {
      return t[t.lastKey()]!;
    }
    final a = t.lastKeyBefore(f + 1)!, b = t.firstKeyAfter(f)!;
    final u = (f - a) / (b - a);
    return t[a]! + (t[b]! - t[a]!) * u;
  }

  bool changed(String id) {
    final s = sev[id];
    if (s != null) {
      return s.any((x) => x != def[id]);
    }
    return _round(at(id)) != _round(def[id]!);
  }

  int changedCount(Iterable<String> ids) => ids.where(changed).length;

  KeyS stateOf(String id) {
    final t = tracks[id];
    if (t == null || t.isEmpty) {
      return KeyS.off;
    }
    return t.containsKey(frame) ? KeyS.at : KeyS.anim;
  }

  void begin() {
    _sv ??= Map.of(v);
    _st ??= {for (final e in tracks.entries) e.key: Map.of(e.value)};
    _ss ??= {for (final e in sev.entries) e.key: List.of(e.value)};
  }

  void end(String what) {
    final a = _sv, b = _st, c = _ss;
    _sv = null;
    _st = null;
    _ss = null;
    if (a == null) {
      return;
    }
    final same = a.entries.every((e) => v[e.key] == e.value) && _sameTracks(b!) && _sameSev(c!);
    if (!same) {
      steps++;
      log = what;
    }
    notifyListeners();
  }

  bool _sameTracks(Map<String, Map<int, double>> b) {
    if (b.length != tracks.length) {
      return false;
    }
    for (final e in tracks.entries) {
      final o = b[e.key];
      if (o == null || o.length != e.value.length || e.value.entries.any((x) => o[x.key] != x.value)) {
        return false;
      }
    }
    return true;
  }

  bool _sameSev(Map<String, List<double>> c) {
    for (final e in sev.entries) {
      final o = c[e.key];
      if (o == null) {
        return false;
      }
      for (var i = 0; i < o.length; i++) {
        if (o[i] != e.value[i]) {
          return false;
        }
      }
    }
    return c.length == sev.length;
  }

  /// Esc during a gesture: everything is where the gesture found it.
  void cancel() {
    if (_sv == null) {
      return;
    }
    v
      ..clear()
      ..addAll(_sv!);
    tracks
      ..clear()
      ..addEntries(_st!.entries.map((e) => MapEntry(e.key, SplayTreeMap<int, double>.of(e.value))));
    sev
      ..clear()
      ..addEntries(_ss!.entries.map((e) => MapEntry(e.key, List.of(e.value))));
    _sv = null;
    _st = null;
    _ss = null;
    log = 'Esc: put back';
    notifyListeners();
  }

  double clamp(String id, double x) {
    final p = byId[id]!;
    var y = x;
    if (p.min != null && y < p.min!) {
      y = p.min!;
    }
    if (p.max != null && y > p.max!) {
      y = p.max!;
    }
    return _round(y);
  }

  /// Write one value. A value with a track writes a key at the playhead; a value without one only creates a track when Animate is on.
  void set(String id, double x, {bool notify = true}) {
    final y = clamp(id, x);
    final t = tracks[id];
    if (t != null) {
      t[frame] = y;
    } else if (animate && byId[id]!.kind == _K.num && (rows[_rowOf(id)]?.keyable ?? false)) {
      tracks[id] = SplayTreeMap<int, double>()..[frame] = y;
    } else {
      v[id] = y;
    }
    if (notify) {
      notifyListeners();
    }
  }

  String _rowOf(String id) {
    for (final r in rows.values) {
      if (r.cells.any((c) => c.id == id)) {
        return r.id;
      }
    }
    return id;
  }

  void add(String id, double dx) {
    final s = sev[id];
    if (s != null) {
      for (var i = 0; i < s.length; i++) {
        s[i] = clamp(id, s[i] + dx);
      }
      notifyListeners();
      return;
    }
    set(id, at(id) + dx);
  }

  void typed(String id, double x) {
    begin();
    sev.remove(id);
    set(id, x, notify: false);
    end('typed ${byId[id]!.label.isEmpty ? rows[_rowOf(id)]!.label : byId[id]!.label} = ${_fmtNum(clamp(id, x), byId[id]!.dec)}');
  }

  void reset(String id) {
    begin();
    sev.remove(id);
    set(id, def[id]!, notify: false);
    end('reset ${rows[_rowOf(id)]?.label ?? id}');
  }

  void resetMany(Iterable<String> ids, String what) {
    begin();
    for (final id in ids) {
      sev.remove(id);
      if (byId[id]!.kind == _K.num) {
        set(id, def[id]!, notify: false);
      } else {
        v[id] = def[id]!;
      }
    }
    end(what);
  }

  /// Group reset (I6-c): values go to default; keys, animated rows' other keys and driven values are left alone. Returns what was kept.
  String resetKeep(Iterable<String> ids, Set<String> driven, String name) {
    begin();
    var n = 0, anim = 0, drv = 0;
    for (final id in ids) {
      if (driven.contains(id)) {
        drv++;
        continue;
      }
      if (tracks.containsKey(id)) {
        if (tracks[id]!.containsKey(frame)) {
          tracks[id]![frame] = def[id]!;
          n++;
        } else {
          anim++;
        }
        continue;
      }
      if (byId[id]!.kind == _K.num) {
        v[id] = def[id]!;
      } else {
        v[id] = def[id]!;
      }
      n++;
    }
    final kept = [if (anim > 0) '$anim animated', if (drv > 0) '$drv driven'].join(' + ');
    end('reset $name: $n values${kept.isEmpty ? '' : ', kept $kept'}');
    return kept;
  }

  void toggleKey(_R r) {
    begin();
    final here = stateOf(r.id) == KeyS.at;
    for (final p in r.cells) {
      final t = tracks[p.id];
      if (here) {
        t?.remove(frame);
        if (t != null && t.isEmpty) {
          v[p.id] = v[p.id]!;
          tracks.remove(p.id);
        }
      } else if (t == null) {
        tracks[p.id] = SplayTreeMap<int, double>()..[frame] = v[p.id]!;
      } else {
        t[frame] = at(p.id);
      }
    }
    end(here ? 'removed key ${r.label} @ ${_fmtT(frame)}' : 'key ${r.label} @ ${_fmtT(frame)}');
  }

  int? neighbour(_R r, {required bool back}) {
    final t = tracks[r.id];
    if (t == null || t.isEmpty) {
      return null;
    }
    return back ? t.lastKeyBefore(frame) : t.firstKeyAfter(frame);
  }

  void seek(int f, {String? why}) {
    frame = f.clamp(0, _end);
    if (why != null) {
      log = why;
    }
    notifyListeners();
  }

  void poke() => notifyListeners();

  void note(String s) {
    log = s;
    notifyListeners();
  }

  void toggleAnimate() {
    animate = !animate;
    log = animate ? 'Animate on' : 'Animate off';
    notifyListeners();
  }

  /// Mix the value of three layers so that the field has something to say about "several".
  void mix(Map<String, List<double>> m) {
    sev
      ..clear()
      ..addAll(m);
    notifyListeners();
  }
}

/// A model that lives as long as its panel and rebuilds it when something changes.
class _Host extends StatefulWidget {
  const _Host({super.key, required this.make, required this.build});
  final _M Function() make;
  final Widget Function(BuildContext, _M) build;
  @override
  State<_Host> createState() => _HostState();
}

class _HostState extends State<_Host> {
  late final _M m = widget.make();
  @override
  void dispose() {
    m.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(listenable: m, builder: (context, _) => widget.build(context, m));
}

// ---- the number cell: one control, five grammars ----

enum _Gr { face, label, ladder, stepper }

const _bands = [10.0, 1.0, .1, .01];
const _bandWords = ['x10', 'x1', 'x0.1', 'x0.01'];

/// The one ladder of a field's edge: rest g20, hover g38, focused / pressed g63, editing accent, rejected danger. Pressed also lifts the fill.
enum _Pose { rest, hover, focus, press, edit, bad }

Color _faceLine(_Pose p) => switch (p) { _Pose.rest => N.g20, _Pose.hover => N.g38, _Pose.focus => N.g76, _Pose.press => N.g63, _Pose.edit => Role.selected, _Pose.bad => Role.error };

class _Cell extends StatefulWidget {
  const _Cell(this.m, this.p, {this.g = _Gr.face, this.tag, this.enabled = true, this.narrow = false, this.ghost, this.pose, this.startBad = false});
  final _M m;
  final _P p;
  final _Gr g;
  final _Pose? pose; // a specimen: the cell is drawn in this pose with the same code (the Parts sheets); null = live
  final String? tag, ghost; // ghost: the value this cell would have after a pending reset (I6-d), shown instead of the real one
  final bool enabled, narrow, startBad; // startBad: the rejected look from the first frame (a field that already refused a value)
  @override
  State<_Cell> createState() => _CellState();
}
