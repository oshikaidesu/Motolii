part of 'inspector_parts.dart';

enum Editorial { a, b, c }

enum ModRule { shiftBig, shiftFine }

enum ResetRule { dbl, del, arrow, both }

enum KeyS { off, anim, at }

enum Kind { shape, text, camera, group, several, none }

enum Space { d2, d25, d3 }

const pickGrid = [
  ['pos.x', 'pos.y', 'pos.z'],
  ['scale.x', 'scale.y', 'scale.z'],
  ['rot.x', 'rot.y', 'rot.z'],
  ['op'],
  ['depth'],
];

class Cfg {
  const Cfg({
    this.ed = Editorial.a,
    this.mod = ModRule.shiftBig,
    this.rightUp = true,
    this.reset = ResetRule.dbl,
    this.tabs = true,
    this.maybe = false,
    this.look = Look.quiet,
    this.locked = false,
  });
  final Editorial ed;
  final ModRule mod;
  final bool rightUp, tabs, maybe, locked;
  final ResetRule reset;
  final Look look;

  bool get resetDbl => reset == ResetRule.dbl || reset == ResetRule.both;
  bool get resetDel => reset == ResetRule.del;
  bool get resetArrow => reset == ResetRule.arrow || reset == ResetRule.both;

  /// One rule for scrub, wheel and arrows (C4): the same keys mean the same thing in every control.
  double mult() {
    final k = HardwareKeyboard.instance;
    if (mod == ModRule.shiftBig) {
      if (k.isShiftPressed) return 10;
      if (k.isAltPressed) return .1;
    } else {
      if (k.isShiftPressed) return .1;
      if (k.isControlPressed) return 10;
    }
    return 1;
  }

  String get modWords => mod == ModRule.shiftBig ? 'Shift x10 · Alt fine' : 'Shift fine · Ctrl x10';
}

/// The editorial option (research/editorial.md): hairline alpha, caps/tracking, type ratio, how sections are told apart.
class Ed {
  const Ed(this.e);
  final Editorial e;
  double get hair => switch (e) {
    Editorial.a => .12,
    Editorial.b => .06,
    Editorial.c => .14,
  };
  Color get rule => Color.fromRGBO(255, 255, 255, hair);
  double get row => e == Editorial.b ? 22 : 24;
  double get gap => switch (e) {
    Editorial.a => 12,
    Editorial.b => 8,
    Editorial.c => 16,
  };

  /// Option A: caps section labels. Option C: caps row labels and big headings. Option B: none.
  String head(String s) => e == Editorial.a ? s.toUpperCase() : s;
  String lab(String s) => s;
  String kick(String s) => e == Editorial.b ? s : s.toUpperCase();
  TextStyle kickStyle(Color c) => e == Editorial.b ? T.label(c) : T.micro(c).copyWith(letterSpacing: .8);
  TextStyle subStyle() => T.name(Grey.g91);
  TextStyle headStyle() => switch (e) {
    Editorial.a => T.micro(Grey.g56).copyWith(letterSpacing: 1.0),
    Editorial.b => T.name(Grey.g63),
    Editorial.c => T.title(Grey.g91).copyWith(fontSize: 13, fontWeight: FontWeight.w400),
  };
  TextStyle labStyle(Color c) => e == Editorial.c ? T.label(c).copyWith(letterSpacing: .3) : T.label(c);
  TextStyle nameStyle() => switch (e) {
    Editorial.a => T.title().copyWith(fontSize: 14, fontWeight: FontWeight.w500),
    Editorial.b => T.title().copyWith(fontSize: 13, fontWeight: FontWeight.w500),
    Editorial.c => T.title().copyWith(fontSize: 17, fontWeight: FontWeight.w300),
  };
}

String propOf(String id) => RegExp(r'\.[xyz]$').hasMatch(id) ? id.substring(0, id.length - 2) : id;

/// A pure-Dart mock of the selected layer(s). Not the product's model; only enough to make every control really do its job.
class Doc extends ChangeNotifier {
  Doc(this.kind, {this.count = 1, int effects = 2}) {
    void s(String id, double v) {
      this.v[id] = v;
      d[id] = v;
    }

    s('pos.x', 960);
    s('pos.y', 540);
    s('pos.z', 0);
    s('scale.x', 100);
    s('scale.y', 100);
    s('scale.z', 100);
    s('rot.x', 0);
    s('rot.y', 0);
    s('rot.z', 0);
    s('anchor.x', 50);
    s('anchor.y', 50);
    s('anchor.z', 0);
    s('op', 100);
    s('depth', 0);
    s('cam.t.x', 960);
    s('cam.t.y', 540);
    s('cam.t.z', 0);
    s('cam.pitch', 12);
    s('cam.yaw', -20);
    s('cam.dist', 1800);
    s('cam.zoom', 100);
    s('cam.roll', 0);
    s('text.size', 48);
    s('stroke.w', 2);
    s('lay.cols', 3);
    s('lay.rows', 2);
    s('lay.gap', 8);
    s('lay.pad', 16);
    s('lay.w', 480);
    s('rel.d', 72);
    s('rel.s', 48);
    s('rel.f', 36);
    s('comp.w', 1920);
    s('comp.h', 1080);
    s('comp.fps', 30);
    for (final f in kEffects) {
      for (final (i, p) in f.params.indexed) {
        paramDefaults(f.id, p).forEach(s);
        if (p.look == ParamLook.colour) s2['${f.id}_${p.id}'] = paramColour(f.decls[i]);
        if (p.look == ParamLook.choice) s2['${f.id}_${p.id}'] = p.options[p.def.round().clamp(0, p.options.length - 1)];
      }
    }
    fxOrder.addAll(kEffects.take(effects).map((f) => f.id));
    v['pos.x'] = 420.5;
    v['op'] = 62;
    v['rot.z'] = 450;
    v['blur_radius'] = 24;
    keys['pos'] = KeyS.at;
    keys['rot'] = KeyS.anim;
    s2['parent'] = 'None';
    s2['blend'] = 'Normal';
    s2['camTarget'] = 'None';
    s2['text.content'] = 'Hero title\nSmall move changes everything.';
    s2['font'] = 'Inter Display';
    s2['align'] = 'Left';
    s2['size'] = 'Hug';
    s2['fill'] = '#D9D2C3';
    s2['stroke'] = 'None';
    b['ghost'] = false;
    b['clip'] = false;
    b['lay.on'] = true;
    b['lay.ignore'] = false;
    if (kind == Kind.several) {
      void m(String id, List<double> xs) => multi[id] = xs;
      for (final e in v.entries) {
        multi[e.key] = List.filled(count, e.value);
      }
      m('pos.x', [120, 360, -40]);
      m('op', [100, 62, 100]);
      m('rot.z', [0, 0, 0]);
      m('scale.x', [100, 100, 80]);
    }
  }

  final Kind kind;
  final int count;
  final Map<String, double> v = {}, d = {};
  final Map<String, List<double>> multi = {};
  final Map<String, KeyS> keys = {};
  final Map<String, String> s2 = {};
  final Map<String, bool> b = {};
  final List<String> fxOrder = [];
  bool animateAll = false, link = true;
  Space space = Space.d2;
  String? revealed;
  String? directHover;
  String query = '';
  Timer? _t;

  @override
  void dispose() {
    _t?.cancel();
    super.dispose();
  }

  /// The selected layer's own name, when a host (the workspace) knows it.
  String? title;

  String get name =>
      title ??
      switch (kind) {
        Kind.shape => 'Jewel',
        Kind.text => 'Hero title',
        Kind.camera => 'Camera 1',
        Kind.group => 'Jewel Field',
        Kind.several => '$count layers',
        Kind.none => '',
      };

  /// null = the layers differ (mixed).
  double? get(String id) {
    final m = multi[id];
    if (m == null) return v[id];
    return m.every((x) => x == m.first) ? m.first : null;
  }

  bool changed(String id) {
    final x = get(id);
    return x == null || x != d[id];
  }

  /// A typed or reset value is absolute for every selected layer (A7).
  void set(String id, double x) => setMany({id: x});

  /// A scrub on a mixed value moves each layer by the same amount (A7).
  void delta(String id, double dx) {
    final m = multi[id];
    if (m != null) {
      multi[id] = [for (final e in m) e + dx];
    } else {
      v[id] = (v[id] ?? 0) + dx;
    }
    _touch(id);
    notifyListeners();
  }

  ({Map<String, double> values, Map<String, List<double>> mixed, Map<String, KeyS> keys})? _gesture;
  final _undo = <({Map<String, double> values, Map<String, List<double>> mixed, Map<String, KeyS> keys})>[];
  int get undoSteps => _undo.length;

  void beginGesture() {
    _gesture ??= (values: Map.of(v), mixed: {for (final e in multi.entries) e.key: List.of(e.value)}, keys: Map.of(keys));
  }

  void endGesture() {
    final before = _gesture;
    _gesture = null;
    if (before != null && before.values.entries.any((e) => v[e.key] != e.value)) {
      _undo.add(before);
    }
    notifyListeners();
  }

  void _restore(({Map<String, double> values, Map<String, List<double>> mixed, Map<String, KeyS> keys}) before) {
    v
      ..clear()
      ..addAll(before.values);
    multi
      ..clear()
      ..addAll(before.mixed);
    keys
      ..clear()
      ..addAll(before.keys);
    notifyListeners();
  }

  void cancelGesture() {
    final before = _gesture;
    _gesture = null;
    if (before != null) _restore(before);
  }

  void undoGesture() {
    if (_gesture != null || _undo.isEmpty) return;
    _restore(_undo.removeLast());
  }

  void setMany(Map<String, double> xs) {
    for (final e in xs.entries) {
      v[e.key] = e.value;
      if (multi.containsKey(e.key)) multi[e.key] = List.filled(count, e.value);
      _touch(e.key);
    }
    notifyListeners();
  }

  /// Reset writes the default value. On an animated row it also writes a key at the playhead (so the arrow drops: value == default).
  void resetIds(Iterable<String> ids) {
    _undo.clear();
    for (final i in ids) {
      final dv = d[i] ?? 0;
      v[i] = dv;
      if (multi.containsKey(i)) multi[i] = List.filled(count, dv);
      final p = propOf(i);
      if (animateAll || (keys[p] ?? KeyS.off) != KeyS.off) keys[p] = KeyS.at;
    }
    notifyListeners();
  }

  /// D2: while Animate all is on (or the property is animated), an edit writes a key at the playhead.
  void _touch(String id) {
    if (_gesture == null) _undo.clear();
    final p = propOf(id);
    if (animateAll || (keys[p] ?? KeyS.off) != KeyS.off) keys[p] = KeyS.at;
  }

  void setKey(String prop, KeyS s) {
    _undo.clear();
    keys[prop] = s;
    notifyListeners();
  }

  void str(String id, String x) {
    s2[id] = x;
    notifyListeners();
  }

  void flag(String id, bool x) {
    b[id] = x;
    notifyListeners();
  }

  void poke() => notifyListeners();

  /// A card's own defaults, written once (absent ids only, no notify), so a card can bring its values without the constructor knowing it.
  void seed(Map<String, double> nums, {Map<String, String> strs = const {}, Map<String, bool> flags = const {}}) {
    nums.forEach((id, x) {
      if (v.containsKey(id)) return;
      v[id] = x;
      d[id] = x;
    });
    strs.forEach((id, x) => s2.putIfAbsent(id, () => x));
    flags.forEach((id, x) => b.putIfAbsent(id, () => x));
  }

  /// I2: a property shortcut reveals and marks the row for a moment.
  void reveal(String prop) {
    revealed = prop;
    _t?.cancel();
    _t = Timer(const Duration(milliseconds: 1600), () {
      revealed = null;
      notifyListeners();
    });
    notifyListeners();
  }

  /// Fields picked together; Shift-click spans a rectangle on [pickGrid] like a spreadsheet.
  final Set<String> picked = {};
  String? _pickFrom;
  int pickGen = 0;
  bool isPicked(String id) => picked.length > 1 && picked.contains(id);

  void pick(String id, {bool span = false}) {
    final from = _pickFrom, a = from == null ? null : _cell(from), b = _cell(id);
    pickGen++;
    picked.clear();
    if (span && a != null && b != null) {
      for (var r = math.min(a.$1, b.$1); r <= math.max(a.$1, b.$1); r++) {
        final row = pickGrid[r];
        for (var c = math.min(a.$2, b.$2); c <= math.max(a.$2, b.$2) && c < row.length; c++) {
          picked.add(row[c]);
        }
      }
    } else {
      picked.add(id);
      _pickFrom = id;
    }
    notifyListeners();
  }

  void unpick() {
    if (picked.isEmpty) return;
    picked.clear();
    notifyListeners();
  }

  static (int, int)? _cell(String id) {
    for (var r = 0; r < pickGrid.length; r++) {
      final c = pickGrid[r].indexOf(id);
      if (c >= 0) return (r, c);
    }
    return null;
  }

  void setDirectHover(String? prop) {
    if (directHover == prop) return;
    directHover = prop;
    notifyListeners();
  }

  void moveFx(String id, int by) {
    final i = fxOrder.indexOf(id), j = i + by;
    if (i < 0 || j < 0 || j >= fxOrder.length) return;
    fxOrder
      ..removeAt(i)
      ..insert(j, id);
    notifyListeners();
  }

  void placeFx(String id, int at) {
    final i = fxOrder.indexOf(id);
    if (i < 0 || at < 0 || at >= fxOrder.length || at == i) return;
    fxOrder
      ..removeAt(i)
      ..insert(at, id);
    notifyListeners();
  }

  void removeFx(String id) {
    fxOrder.remove(id);
    notifyListeners();
  }

  void randomFx(Fx f) {
    final r = math.Random(f.id.hashCode + (v['${f.id}_seed'] = (v['${f.id}_seed'] ?? 0) + 1).toInt());
    for (final p in f.params) {
      if (!p.bounded || p.look == ParamLook.toggle || p.look == ParamLook.choice || p.look == ParamLook.colour) continue;
      double roll() => double.parse((p.min! + (p.max! - p.min!) * r.nextDouble()).toStringAsFixed(p.dec));
      for (final id in paramDefaults(f.id, p).keys) {
        v[id] = roll();
      }
    }
    notifyListeners();
  }
}
