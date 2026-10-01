// Inspector parts. What the Inspector must DO comes from research/intents/inspector.md (Yes cards built, Maybe cards behind cfg.maybe).
// How it LOOKS is the lab's (DESIGN.md): hairlines, type carries hierarchy, grey levels, accent only C.mode. The old Motolii is not a source.
// Open questions are Cfg fields (knobs in inspector_set.dart); every behaviour below follows them.
import 'dart:async';
import 'dart:math' as math;

import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../parts/controls.dart';
import '../parts/glyphs.dart';
import '../parts/relations.dart';
import '../tokens.dart';

// ---- 0. open values -------------------------------------------------------------------------------------------------
// [open] a value cell's height (the lab's ValueWell is 22; rows are 24 on the 4px rhythm; B uses 22).
const _cellH = 22.0;
// [open] one narrowing rule for the whole panel (H1): a cell narrower than _wUnit drops its unit, narrower than _wTight drops decimals (never to a false 0).
const _wUnit = 72.0, _wTight = 60.0;
// [open] a third value stacks under the first two when a cell would be narrower than this.
const _stack3 = 72.0;
// [open] the label column (wide / narrow panel).
const _labelW = 88.0, _labelWNarrow = 76.0;
// [open] the width of the key mark's and reset mark's columns.
const _keyW = 22.0, _resetW = 18.0;
// [open] scroll: one wheel notch is about 53 px of scrollDelta on a mouse; a step needs this much so a trackpad does not race.
const _notch = 40.0;
// [open] axis colours: X / Y / Z ticks reuse relation-family hues (the only hue outside relations).
const _axX = Color(0xFFE974AB), _axY = Color(0xFF7DD5B1), _axZ = Color(0xFF4781E5);
const _axes = [_axX, _axY, _axZ];
// [open] two clicks within this are a double click.
const _dbl = Duration(milliseconds: 320);

// ---- 1. the open questions as one config ----------------------------------------------------------------------------
enum Editorial { a, b, c }

enum ModRule { shiftBig, shiftFine }

enum ResetRule { dbl, del, arrow, both }

enum KeyS { off, anim, at }

enum Kind { shape, text, camera, group, several, none }

enum Space { d2, d25, d3 }

class Cfg {
  const Cfg({this.ed = Editorial.a, this.mod = ModRule.shiftBig, this.rightUp = true, this.reset = ResetRule.dbl, this.tabs = true, this.maybe = false, this.look = Look.quiet, this.locked = false});
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
  double get hair => switch (e) { Editorial.a => .12, Editorial.b => .06, Editorial.c => .14 };
  Color get rule => Color.fromRGBO(255, 255, 255, hair);
  double get row => e == Editorial.b ? 22 : 24;
  double get gap => switch (e) { Editorial.a => 12, Editorial.b => 8, Editorial.c => 16 };

  /// Option A: caps section labels. Option C: caps row labels and big headings. Option B: none.
  String head(String s) => e == Editorial.a ? s.toUpperCase() : s;
  String lab(String s) => s;
  String kick(String s) => e == Editorial.b ? s : s.toUpperCase();
  TextStyle kickStyle(Color c) => e == Editorial.b ? T.label(c) : T.micro(c).copyWith(letterSpacing: .8);
  TextStyle subStyle() => T.name(N.g91);
  TextStyle headStyle() => switch (e) {
        Editorial.a => T.micro(N.g56).copyWith(letterSpacing: 1.0),
        Editorial.b => T.name(N.g63),
        Editorial.c => T.title(N.g91).copyWith(fontSize: 13, fontWeight: FontWeight.w400),
      };
  TextStyle labStyle(Color c) => e == Editorial.c ? T.label(c).copyWith(letterSpacing: .3) : T.label(c);
  TextStyle nameStyle() => switch (e) {
        Editorial.a => T.title().copyWith(fontSize: 14, fontWeight: FontWeight.w500),
        Editorial.b => T.title().copyWith(fontSize: 13, fontWeight: FontWeight.w500),
        Editorial.c => T.title().copyWith(fontSize: 17, fontWeight: FontWeight.w300),
      };
}

// ---- 2. the document (mock values) ----------------------------------------------------------------------------------
String propOf(String id) => RegExp(r'\.[xyz]$').hasMatch(id) ? id.substring(0, id.length - 2) : id;

/// A pure-Dart mock of the selected layer(s). Not the product's model; only enough to make every control really do its job.
class Doc extends ChangeNotifier {
  Doc(this.kind, {this.count = 1, int effects = 2}) {
    void s(String id, double v) {
      this.v[id] = v;
      d[id] = v;
    }

    s('pos.x', 960); s('pos.y', 540); s('pos.z', 0);
    s('scale.x', 100); s('scale.y', 100); s('scale.z', 100);
    s('rot.x', 0); s('rot.y', 0); s('rot.z', 0);
    s('anchor.x', 50); s('anchor.y', 50);
    s('op', 100); s('depth', 0);
    s('cam.t.x', 960); s('cam.t.y', 540); s('cam.t.z', 0);
    s('cam.pitch', 12); s('cam.yaw', -20); s('cam.dist', 1800); s('cam.zoom', 100); s('cam.roll', 0);
    s('text.size', 48); s('stroke.w', 2);
    s('lay.cols', 3); s('lay.rows', 2); s('lay.gap', 8); s('lay.pad', 16); s('lay.w', 480);
    s('rel.d', 72); s('rel.s', 48); s('rel.f', 36);
    s('comp.w', 1920); s('comp.h', 1080); s('comp.fps', 30);
    for (final f in kEffects) {
      for (final p in f.params) {
        s('${f.id}_${p.id}', p.def);
      }
    }
    fxOrder.addAll(kEffects.take(effects).map((f) => f.id));
    // seeded states so the use cases open on something real: a changed value, a keyed one, an animated one.
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
  String query = '';
  Timer? _t;

  @override
  void dispose() {
    _t?.cancel();
    super.dispose();
  }

  String get name => switch (kind) {
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
  void set(String id, double x) {
    v[id] = x;
    if (multi.containsKey(id)) multi[id] = List.filled(count, x);
    _touch(id);
    notifyListeners();
  }

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
    final p = propOf(id);
    if (animateAll || (keys[p] ?? KeyS.off) != KeyS.off) keys[p] = KeyS.at;
  }

  void setKey(String prop, KeyS s) {
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

  void moveFx(String id, int by) {
    final i = fxOrder.indexOf(id), j = i + by;
    if (i < 0 || j < 0 || j >= fxOrder.length) return;
    fxOrder
      ..removeAt(i)
      ..insert(j, id);
    notifyListeners();
  }

  void removeFx(String id) {
    fxOrder.remove(id);
    notifyListeners();
  }

  void randomFx(Fx f) {
    final r = math.Random(f.id.hashCode + (v['${f.id}_seed'] = (v['${f.id}_seed'] ?? 0) + 1).toInt());
    for (final p in f.params) {
      if (p.kind == PK.num) v['${f.id}_${p.id}'] = (p.min + (p.max - p.min) * r.nextDouble()).roundToDouble();
    }
    notifyListeners();
  }
}

/// Set by a row: its controls are off (locked, driven or disabled) and whether the panel is narrow (units go everywhere at once).
class RowOff extends InheritedWidget {
  const RowOff({super.key, required this.off, required this.narrow, required super.child});
  final bool off, narrow;
  static RowOff? maybe(BuildContext c) => c.dependOnInheritedWidgetOfExactType<RowOff>();
  static bool offOf(BuildContext c) => maybe(c)?.off ?? false;
  @override
  bool updateShouldNotify(RowOff o) => o.off != off || o.narrow != narrow;
}

/// For controls that cannot recolour themselves (Segmented): off = a fixed dim and no pointer.
class Dis extends StatelessWidget {
  const Dis({super.key, required this.child});
  final Widget child;
  @override
  Widget build(BuildContext context) {
    final off = RowOff.offOf(context) || Ctx.of(context).cfg.locked;
    return IgnorePointer(ignoring: off, child: Opacity(opacity: off ? .6 : 1, child: child));
  }
}

class Ctx extends InheritedWidget {
  const Ctx({super.key, required this.cfg, required this.doc, required super.child});
  final Cfg cfg;
  final Doc doc;
  static Ctx of(BuildContext c) => c.dependOnInheritedWidgetOfExactType<Ctx>()!;
  static Ctx read(BuildContext c) => c.getInheritedWidgetOfExactType<Ctx>()!;
  @override
  bool updateShouldNotify(Ctx old) => true;
}

/// Owns the mock document and re-provides it whenever it changes. A different kind or count makes a new document (give it a key).
class InspHost extends StatefulWidget {
  const InspHost({super.key, required this.cfg, required this.kind, this.count = 1, this.effects = 2, required this.child});
  final Cfg cfg;
  final Kind kind;
  final int count, effects;
  final Widget child;
  @override
  State<InspHost> createState() => _InspHostState();
}

class _InspHostState extends State<InspHost> {
  late Doc doc = _make();
  Doc _make() => Doc(widget.kind, count: widget.count, effects: widget.effects)..addListener(_r);
  void _r() {
    if (mounted) setState(() {});
  }

  @override
  void didUpdateWidget(InspHost o) {
    super.didUpdateWidget(o);
    if (o.kind != widget.kind || o.count != widget.count || o.effects != widget.effects) {
      doc.dispose();
      doc = _make();
    }
  }

  @override
  void dispose() {
    doc.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Ctx(cfg: widget.cfg, doc: doc, child: widget.child);
}

// ---- 3. small things ------------------------------------------------------------------------------------------------
/// A contract written as a 10px caption under a control.
class Cap extends StatelessWidget {
  const Cap(this.text, {super.key, this.color = N.g56});
  final String text;
  final Color color;
  @override
  Widget build(BuildContext context) => Padding(padding: const EdgeInsets.only(top: 6), child: Text(text, style: T.label(color).copyWith(height: 1.3)));
}

class Hov extends StatefulWidget {
  const Hov({super.key, required this.builder, this.cursor = SystemMouseCursors.click, this.onTap});
  final Widget Function(BuildContext, bool) builder;
  final MouseCursor cursor;
  final VoidCallback? onTap;
  @override
  State<Hov> createState() => _HovState();
}

class _HovState extends State<Hov> {
  bool h = false;
  @override
  Widget build(BuildContext context) => MouseRegion(
        cursor: widget.cursor,
        onEnter: (_) => setState(() => h = true),
        onExit: (_) => setState(() => h = false),
        child: GestureDetector(behavior: HitTestBehavior.opaque, onTap: widget.onTap, child: widget.builder(context, h)),
      );
}

String fmt(double v, int dec, {bool tight = false}) {
  var d = tight ? 0 : dec;
  var s = v.toStringAsFixed(d);
  while (v != 0 && double.parse(s) == 0 && d < 4) {
    d++;
    s = v.toStringAsFixed(d);
  }
  return s == '-0' ? '0' : s;
}

/// The key mark. Three states differ by SHAPE, not by tone alone (D1, do-not-repeat 11): off = hollow diamond; animated = diamond with its link line; keyed here = filled playhead diamond.
class KeyMark extends StatelessWidget {
  const KeyMark({super.key, required this.state, this.onTap, this.enabled = true});
  final KeyS state;
  final VoidCallback? onTap;
  final bool enabled;
  @override
  Widget build(BuildContext context) => Hov(
        onTap: enabled ? onTap : null,
        builder: (_, h) => SizedBox(width: _keyW, height: _keyW, child: CustomPaint(painter: _KeyPaint(state, h && enabled, enabled))),
      );
}

class _KeyPaint extends CustomPainter {
  const _KeyPaint(this.s, this.hover, this.enabled);
  final KeyS s;
  final bool hover, enabled;
  @override
  void paint(Canvas c, Size z) {
    final o = Offset(z.width / 2, z.height / 2), r = s == KeyS.at ? 4.6 : 3.8;
    final dia = Path()..moveTo(o.dx, o.dy - r)..lineTo(o.dx + r, o.dy)..lineTo(o.dx, o.dy + r)..lineTo(o.dx - r, o.dy)..close();
    final a = enabled ? 1.0 : .4;
    Paint line(Color col, double w) => Paint()..color = col.withValues(alpha: col.a * a)..style = PaintingStyle.stroke..strokeWidth = w;
    switch (s) {
      case KeyS.off:
        c.drawPath(dia, line(hover ? N.g76 : N.g44, 1));
      case KeyS.anim:
        c.drawPath(dia, line(hover ? N.g95 : N.g91, 1));
        c.drawLine(Offset(o.dx - r - 4, o.dy), Offset(o.dx - r, o.dy), line(N.g91, 1));
        c.drawLine(Offset(o.dx + r, o.dy), Offset(o.dx + r + 4, o.dy), line(N.g91, 1));
      case KeyS.at:
        c.drawPath(dia, Paint()..color = C.playhead.withValues(alpha: a));
        c.drawPath(dia, line(N.g100, 1.5));
    }
  }

  @override
  bool shouldRepaint(_KeyPaint o) => o.s != s || o.hover != hover || o.enabled != enabled;
}

/// The reset mark: appears only when the value differs (C7, E4). [open] the arrow is a text character, not a new glyph.
class ResetMark extends StatelessWidget {
  const ResetMark({super.key, required this.show, this.onTap});
  final bool show;
  final VoidCallback? onTap;
  @override
  Widget build(BuildContext context) => SizedBox(
        width: _resetW,
        child: show
            ? Hov(onTap: onTap, builder: (_, h) => Center(child: Text('↺', style: T.label(h ? N.g95 : N.g56).copyWith(fontSize: 12))))
            : null,
      );
}

/// The word-and-switch for On/Off (G2): the state is a word, never colour alone.
class OnOff extends StatelessWidget {
  const OnOff({super.key, required this.on, required this.onChanged, this.enabled = true});
  final bool on, enabled;
  final ValueChanged<bool> onChanged;
  @override
  Widget build(BuildContext context) {
    final live = enabled && !RowOff.offOf(context) && !Ctx.of(context).cfg.locked;
    return IgnorePointer(
      ignoring: !live,
      child: Row(mainAxisSize: MainAxisSize.min, children: [
        SizedBox(width: 20, child: Text(on ? 'On' : 'Off', textAlign: TextAlign.right, style: T.label(live && on ? N.g95 : N.g56))),
        const SizedBox(width: 8),
        Opacity(opacity: live ? 1 : .5, child: PillSwitch(on: on, onChanged: onChanged)),
      ]),
    );
  }
}

/// A choice chip in the lab's choice-control style: a g20 pill, g95 text, a 1px accent underline.
class Chip extends StatelessWidget {
  const Chip(this.text, {super.key, required this.on, this.onTap, this.enabled = true});
  final String text;
  final bool on, enabled;
  final VoidCallback? onTap;
  @override
  Widget build(BuildContext context) => Hov(
        onTap: enabled ? onTap : null,
        builder: (_, h) => Opacity(
          opacity: enabled ? 1 : .4,
          child: Container(
            height: 20,
            padding: const EdgeInsets.symmetric(horizontal: 7),
            alignment: Alignment.center,
            decoration: BoxDecoration(color: on ? N.g20 : (h ? N.g15 : null), borderRadius: BorderRadius.circular(5), border: Border(bottom: BorderSide(color: on ? C.mode : const Color(0x00000000)))),
            child: Text(text, style: T.label(on ? N.g95 : N.g63)),
          ),
        ),
      );
}

// ---- 4. the number field (the core control) ------------------------------------------------------------------------
class NumField extends StatefulWidget {
  const NumField({
    super.key,
    required this.id,
    this.label,
    this.unit = '',
    this.decimals = 0,
    this.min,
    this.max,
    this.step = 1,
    this.perPx = 1,
    this.zeroWord,
    this.enabled = true,
    this.axis,
    this.turns = false,
    this.onSet,
    this.onReset,
  });
  final String id;
  final String? label, unit, zeroWord;
  final int decimals;
  final double? min, max;
  final double step, perPx;
  final bool enabled, turns;
  final Color? axis;
  final ValueChanged<double>? onSet;
  final VoidCallback? onReset;
  @override
  State<NumField> createState() => _NumFieldState();
}

class _NumFieldState extends State<NumField> {
  final _node = FocusNode(debugLabel: 'num');
  late final FocusNode _ef = FocusNode(debugLabel: 'num-edit', onKeyEvent: _editKey);
  final _ctl = TextEditingController();
  bool _editing = false, _err = false, _hover = false, _focus = false, _drag = false, _moved = false, _cancelled = false;
  Offset _down = Offset.zero;
  double _start = 0, _acc = 0, _sum = 0, _wheel = 0, _rung = 1;
  DateTime? _lastUp;
  Timer? _pending;

  Ctx get _x => Ctx.read(context);
  bool get _on => widget.enabled && !_x.cfg.locked && !(context.getInheritedWidgetOfExactType<RowOff>()?.off ?? false);
  bool get _bounded => widget.min != null && widget.max != null;

  @override
  void initState() {
    super.initState();
    _ef.addListener(_editFocus);
  }

  @override
  void dispose() {
    _pending?.cancel();
    _ef.dispose();
    _node.dispose();
    _ctl.dispose();
    super.dispose();
  }

  double _clamp(double v) => math.min(widget.max ?? double.infinity, math.max(widget.min ?? double.negativeInfinity, v));
  double _tidy(double v) => double.parse(v.toStringAsFixed(6));

  void _apply(double v) {
    final c = _clamp(v);
    final f = widget.onSet;
    if (f != null) {
      f(c);
    } else {
      _x.doc.set(widget.id, c);
    }
  }

  void _reset() {
    final f = widget.onReset;
    if (f != null) {
      f();
    } else {
      _x.doc.resetIds([widget.id]);
    }
  }

  // contract: Esc during a drag puts the value back and ignores the rest of the drag; pointer cancel does the same (C3).
  void _abort() {
    final cur = _x.doc.get(widget.id);
    if (cur == null) {
      _x.doc.delta(widget.id, -_sum);
    } else {
      _apply(_start);
    }
    _cancelled = true;
    _sum = 0;
    setState(() => _drag = false);
  }

  double _ladder(double dy) => dy < -24 ? 10 : (dy > 64 ? .01 : (dy > 24 ? .1 : 1));

  void _onDown(PointerDownEvent e) {
    if (!_on || _editing) return;
    if (e.buttons != kPrimaryButton) return;
    _node.requestFocus();
    _pending?.cancel();
    _down = e.position;
    _moved = false;
    _cancelled = false;
    _sum = 0;
    _rung = 1;
    _start = _x.doc.get(widget.id) ?? 0;
    _acc = _start;
    _drag = true;
  }

  void _onMove(PointerMoveEvent e) {
    if (!_drag || _cancelled) return;
    if (!_moved && (e.position - _down).distance < 4) return;
    final cfg = _x.cfg;
    if (!_moved) setState(() => _moved = true);
    var m = cfg.mult();
    if (cfg.maybe) {
      final r = _ladder(e.position.dy - _down.dy);
      if (r != _rung) setState(() => _rung = r);
      m *= _rung;
    }
    final d = e.delta.dx * (cfg.rightUp ? 1 : -1) * widget.perPx * m;
    if (_x.doc.get(widget.id) == null) {
      _sum += d;
      _x.doc.delta(widget.id, d);
    } else {
      _acc = _clamp(_acc + d);
      _apply(_acc);
    }
  }

  void _onUp(PointerUpEvent e) {
    if (!_drag) return;
    _drag = false;
    if (_moved) {
      setState(() {
        _moved = false;
        _rung = 1;
      });
      return;
    }
    final now = DateTime.now(), dbl = _lastUp != null && now.difference(_lastUp!) < _dbl;
    _lastUp = dbl ? null : now;
    final cfg = _x.cfg;
    if (dbl) {
      _pending?.cancel();
      if (cfg.resetDbl) {
        _reset();
      } else {
        _startEdit();
      }
    } else if (cfg.resetDbl) {
      // contract: with double-click = reset, a single click types only after the double-click window, so no editor flashes (do-not-repeat 5).
      _pending = Timer(_dbl, _startEdit);
    }
  }

  void _onCancel(PointerCancelEvent e) {
    if (_drag && _moved) _abort();
    _drag = false;
    _moved = false;
  }

  void _onSignal(PointerSignalEvent e) {
    if (e is! PointerScrollEvent || !_on || _editing) return;
    GestureBinding.instance.pointerSignalResolver.register(e, (ev) {
      final s = ev as PointerScrollEvent, dy = s.scrollDelta.dy != 0 ? s.scrollDelta.dy : s.scrollDelta.dx;
      if (_wheel != 0 && (_wheel > 0) != (dy > 0)) _wheel = 0;
      _wheel += dy;
      // contract: one notch = one displayed unit, Shift/Alt per the panel's modifier rule; wheel up = up. No button needs holding (do-not-repeat 4).
      while (_wheel.abs() >= _notch) {
        _nudge(_wheel > 0 ? -1 : 1);
        _wheel -= _notch * (_wheel > 0 ? 1 : -1);
      }
    });
  }

  void _nudge(int dir) {
    final d = dir * widget.step * _x.cfg.mult();
    final cur = _x.doc.get(widget.id);
    if (cur == null) {
      _x.doc.delta(widget.id, d);
    } else {
      _apply(_tidy(cur + d));
    }
  }

  KeyEventResult _key(FocusNode n, KeyEvent e) {
    if (_editing || !_on) return KeyEventResult.ignored;
    if (e is! KeyDownEvent && e is! KeyRepeatEvent) return KeyEventResult.ignored;
    final k = e.logicalKey;
    if (k == LogicalKeyboardKey.escape && _drag && _moved) {
      _abort();
      return KeyEventResult.handled;
    }
    if (k == LogicalKeyboardKey.arrowUp || k == LogicalKeyboardKey.arrowRight) {
      _nudge(1);
      return KeyEventResult.handled;
    }
    if (k == LogicalKeyboardKey.arrowDown || k == LogicalKeyboardKey.arrowLeft) {
      _nudge(-1);
      return KeyEventResult.handled;
    }
    if (k == LogicalKeyboardKey.enter || k == LogicalKeyboardKey.numpadEnter) {
      _startEdit();
      return KeyEventResult.handled;
    }
    if (_x.cfg.resetDel && (k == LogicalKeyboardKey.delete || k == LogicalKeyboardKey.backspace)) {
      _reset();
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  void _startEdit() {
    if (!mounted || _editing || !_on) return;
    final v = _x.doc.get(widget.id);
    _ctl.text = v == null ? '' : fmt(v, widget.decimals);
    _ctl.selection = TextSelection(baseOffset: 0, extentOffset: _ctl.text.length);
    setState(() {
      _editing = true;
      _err = false;
    });
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted || !_editing) return;
      _ef.requestFocus();
      _ctl.selection = TextSelection(baseOffset: 0, extentOffset: _ctl.text.length);
    });
  }

  KeyEventResult _editKey(FocusNode n, KeyEvent e) {
    if (e is! KeyDownEvent) return KeyEventResult.ignored;
    if (e.logicalKey == LogicalKeyboardKey.escape) {
      // contract: Esc while typing discards and restores the old value (C3).
      _close();
      return KeyEventResult.handled;
    }
    if (e.logicalKey == LogicalKeyboardKey.enter || e.logicalKey == LogicalKeyboardKey.numpadEnter) {
      _commit();
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  void _commit() {
    final t = double.tryParse(_ctl.text.trim());
    if (t == null) {
      // contract: invalid text keeps the field open and says "Number required" (C2).
      setState(() => _err = true);
      return;
    }
    _apply(t);
    _close();
  }

  void _close({bool refocus = true}) {
    if (!_editing) return;
    setState(() => _editing = false);
    if (refocus) _node.requestFocus();
  }

  void _editFocus() {
    if (_editing && !_ef.hasFocus) {
      // contract: leaving the field applies a valid number and drops an invalid one.
      final t = double.tryParse(_ctl.text.trim());
      if (t != null) _apply(t);
      _close(refocus: false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), cfg = x.cfg, ed = Ed(cfg.ed), look = cfg.look;
    final v = x.doc.get(widget.id), on = _on, narrow = RowOff.maybe(context)?.narrow ?? false;
    final live = _drag && _moved;
    final Color idle = switch (look) {
      Look.concept || Look.glow => ed.rule,
      Look.quiet => cfg.ed == Editorial.a ? const Color(0x00000000) : ed.rule,
    };
    final Color edge = live
        ? C.mode.withValues(alpha: .8)
        : (_focus || _editing
            ? (look == Look.glow ? C.mode.withValues(alpha: .6) : N.g63)
            : (_hover && on ? N.g38 : idle));
    final fillC = !on ? N.g10 : (look == Look.quiet ? N.g13 : N.g07);
    return LayoutBuilder(builder: (context, box) {
      final w = box.maxWidth;
      final showUnit = w >= _wUnit && !narrow, tight = w < _wTight;
      final t = (_bounded && v != null) ? ((v - widget.min!) / (widget.max! - widget.min!)).clamp(0.0, 1.0) : 0.0;
      String unit = widget.unit ?? '';
      if (widget.turns && v != null && v.abs() >= 360 && w >= 120) unit = '° ${(v / 360).truncate()}x ${fmt(v.remainder(360), 0)}';
      final word = v == 0 && widget.zeroWord != null ? widget.zeroWord! : null;
      Widget valueText() {
        if (v == null) return Text('Mixed', style: T.label(N.g76));
        if (word != null) return Text(word, style: T.label(on ? N.g76 : N.g56));
        return Text.rich(TextSpan(children: [
          TextSpan(text: fmt(v, widget.decimals, tight: tight), style: T.value(on ? N.g95 : N.g56)),
          if (showUnit && unit.isNotEmpty) TextSpan(text: ' $unit', style: T.label(N.g56)),
        ]), maxLines: 1, softWrap: false, overflow: TextOverflow.clip);
      }

      final lead = _err && w >= 150 ? Text('Number required', style: T.label(C.danger)) : (live && cfg.maybe && _rung != 1 ? Text('x${_rung == .01 ? '.01' : (_rung == .1 ? '.1' : '10')}', style: T.micro(N.g95)) : (widget.label != null ? Text(ed.lab(widget.label!), maxLines: 1, softWrap: false, style: ed.labStyle(on ? N.g63 : N.g56)) : null));
      final body = Container(
        height: _cellH,
        decoration: BoxDecoration(color: fillC, borderRadius: BorderRadius.circular(4), border: Border.all(color: edge)),
        child: ClipRRect(
          borderRadius: BorderRadius.circular(3),
          child: Stack(children: [
            // the range tick sits on the field's bottom edge, never behind the text
            if (_bounded) Positioned(left: 0, bottom: 0, height: 2, width: math.max(0, (w - 2) * t), child: ColoredBox(color: look == Look.glow ? C.mode : (on ? N.g56 : N.g44))),
            if (widget.axis != null) Positioned(left: 0, top: 5, bottom: 5, width: 2, child: ColoredBox(color: widget.axis!)),
            Positioned.fill(
              child: Padding(
                padding: EdgeInsets.only(left: widget.axis != null ? 8 : 6, right: 6),
                child: Row(crossAxisAlignment: CrossAxisAlignment.center, children: [
                  if (lead != null) Flexible(child: lead),
                  const Spacer(),
                  if (_editing)
                    SizedBox(
                      width: math.min(72, math.max(28, w - 36)),
                      child: EditableText(
                        controller: _ctl,
                        focusNode: _ef,
                        style: T.value(N.g95),
                        cursorColor: N.g95,
                        backgroundCursorColor: N.g20,
                        selectionColor: C.mode.withValues(alpha: .4),
                        textAlign: TextAlign.right,
                        maxLines: 1,
                        keyboardType: const TextInputType.numberWithOptions(decimal: true, signed: true),
                        inputFormatters: [FilteringTextInputFormatter.allow(RegExp(r'[0-9.\-+eE]'))],
                        onChanged: (_) {
                          if (_err) setState(() => _err = false);
                        },
                      ),
                    )
                  else
                    valueText(),
                ]),
              ),
            ),
          ]),
        ),
      );
      return Focus(
        focusNode: _node,
        onKeyEvent: _key,
        onFocusChange: (f) => setState(() => _focus = f),
        child: Listener(
          onPointerDown: _onDown,
          onPointerMove: _onMove,
          onPointerUp: _onUp,
          onPointerCancel: _onCancel,
          onPointerSignal: _onSignal,
          child: MouseRegion(
            cursor: !on ? SystemMouseCursors.basic : (_editing ? SystemMouseCursors.text : SystemMouseCursors.resizeLeftRight),
            onEnter: (_) => setState(() => _hover = true),
            onExit: (_) => setState(() => _hover = false),
            child: body,
          ),
        ),
      );
    });
  }
}

// ---- 5. sections and rows -------------------------------------------------------------------------------------------
class Rule extends StatelessWidget {
  const Rule({super.key});
  @override
  Widget build(BuildContext context) => Container(height: 1, color: Ed(Ctx.of(context).cfg.ed).rule);
}

/// A group: a heading and its rows. How groups are told apart is the Editorial knob: A a rule above, B a grey step, C a rule under a large heading.
class Sect extends StatelessWidget {
  const Sect({super.key, this.title, this.children = const [], this.first = false, this.trailing, this.open = true, this.onToggle, this.hint, this.dim = false, this.sub = false});
  final String? title, hint;
  final List<Widget> children;
  final bool first, open, dim, sub;
  final Widget? trailing;
  final VoidCallback? onToggle;

  @override
  Widget build(BuildContext context) {
    final ed = Ed(Ctx.of(context).cfg.ed);
    final head = (title == null && trailing == null)
        ? null
        : GestureDetector(
            behavior: HitTestBehavior.opaque,
            onTap: onToggle,
            child: SizedBox(
              height: 24,
              child: Row(children: [
                if (title != null) Expanded(child: Text(sub ? title! : ed.head(title!), maxLines: 1, overflow: TextOverflow.ellipsis, style: sub ? ed.subStyle() : ed.headStyle())) else const Spacer(),
                if (!open && hint != null) Padding(padding: const EdgeInsets.only(right: 8), child: Text(hint!, style: T.label(N.g56))),
                ?trailing,
              ]),
            ),
          );
    final body = [if (open) ...children];
    Widget col(List<Widget> kids) => Opacity(opacity: dim ? .45 : 1, child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, mainAxisSize: MainAxisSize.min, children: kids));
    switch (ed.e) {
      case Editorial.a:
        return col([
          if (!first) Padding(padding: EdgeInsets.only(top: ed.gap), child: const Rule()),
          SizedBox(height: first ? 0 : 4),
          ?head,
          ...body,
        ]);
      case Editorial.b:
        return Padding(
          padding: EdgeInsets.only(bottom: ed.gap),
          child: Container(padding: const EdgeInsets.fromLTRB(8, 4, 8, 6), decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(6)), child: col([?head, ...body])),
        );
      case Editorial.c:
        return Padding(
          padding: EdgeInsets.only(top: first ? 0 : ed.gap),
          child: col([
            if (head != null) ...[head, const SizedBox(height: 2), const Rule(), const SizedBox(height: 6)],
            ...body,
          ]),
        );
    }
  }
}

/// One property: label, its value cells, the key mark, the reset mark. The columns are the same on every row (B1) so X/Y/Z line up.
/// contract: right-click a row for Key this frame / Remove key / Remove animation; unavailable ones say why (D3).
class PropRow extends StatefulWidget {
  const PropRow({super.key, required this.id, required this.label, required this.cells, this.ids, this.keyable = true, this.enabled = true, this.under, this.note, this.rel, this.relName});
  final String id, label;
  final List<Widget> cells;
  final List<String>? ids;
  final bool keyable, enabled;
  final Widget? under;
  final String? note, relName;
  final Fam? rel;
  @override
  State<PropRow> createState() => _PropRowState();
}

class _PropRowState extends State<PropRow> {
  bool _menu = false, _was = false;

  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), cfg = x.cfg, doc = x.doc, ed = Ed(cfg.ed);
    final ids = widget.ids ?? [widget.id];
    final changed = ids.any(doc.changed), ks = doc.keys[widget.id] ?? KeyS.off;
    final active = doc.revealed == widget.id;
    if (active && !_was) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted) Scrollable.ensureVisible(context, alignment: .2, duration: const Duration(milliseconds: 160), curve: Curves.easeOut);
      });
    }
    _was = active;
    final locked = cfg.locked;
    final showRel = cfg.maybe && widget.rel != null;
    final cellsOn = widget.enabled && !showRel && !locked;
    return LayoutBuilder(builder: (context, box) {
      final lw = box.maxWidth < 300 ? _labelWNarrow : _labelW;
      final slots = _keyW + (cfg.resetArrow ? _resetW : 0);
      final cellsW = box.maxWidth - lw - slots - 8;
      final stack = widget.cells.length == 3 && (cellsW - 8) / 3 < _stack3;
      Widget gap(double w) => SizedBox(width: w);
      Widget cellsRow(List<Widget> cs) => Row(children: [for (var i = 0; i < cs.length; i++) ...[if (i > 0) gap(4), Expanded(child: cs[i])]]);
      final cells = stack
          ? Column(mainAxisSize: MainAxisSize.min, children: [cellsRow(widget.cells.take(2).toList()), const SizedBox(height: 4), Row(children: [Expanded(child: widget.cells[2]), gap(4), const Expanded(child: SizedBox())])])
          : cellsRow(widget.cells);
      final label = Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
        Row(children: [
          Flexible(child: Text(ed.lab(widget.label), maxLines: 1, overflow: TextOverflow.ellipsis, style: ed.labStyle(!cellsOn ? N.g56 : (changed ? N.g95 : N.g63)))),
          if (showRel) Padding(padding: const EdgeInsets.only(left: 4), child: Container(width: 5, height: 5, decoration: BoxDecoration(color: widget.rel!.c, shape: BoxShape.circle))),
        ]),
        if (widget.under != null) Padding(padding: const EdgeInsets.only(top: 4, bottom: 2), child: widget.under),
      ]);
      Widget item(String s, bool ok, String why, VoidCallback go) => Hov(
            cursor: ok ? SystemMouseCursors.click : SystemMouseCursors.basic,
            onTap: ok && !locked
                ? () {
                    go();
                    setState(() => _menu = false);
                  }
                : null,
            builder: (_, h) => Container(
              height: 22,
              padding: const EdgeInsets.symmetric(horizontal: 8),
              color: h && ok ? N.g15 : null,
              child: Row(children: [
                Text(s, style: T.name(ok ? N.g91 : N.g56)),
                const Spacer(),
                if (!ok) Text(why, style: T.label(N.g56)),
              ]),
            ),
          );
      return Listener(
        behavior: HitTestBehavior.translucent,
        onPointerDown: (e) {
          if (e.buttons == kSecondaryButton && widget.keyable) setState(() => _menu = !_menu);
        },
        child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          Container(
            constraints: BoxConstraints(minHeight: ed.row),
            padding: const EdgeInsets.symmetric(vertical: 1),
            decoration: BoxDecoration(color: active ? N.g20 : null, border: active ? const Border(left: BorderSide(color: N.g95, width: 2)) : null),
            child: Row(crossAxisAlignment: CrossAxisAlignment.center, children: [
              SizedBox(width: lw, child: label),
              Expanded(child: RowOff(off: !cellsOn, narrow: box.maxWidth < 300, child: IgnorePointer(ignoring: !cellsOn, child: cells))),
              // the key column is reserved on every row so every field ends at one edge
              if (widget.keyable)
                KeyMark(
                  state: ks,
                  enabled: !locked,
                  // contract: click = key at the playhead; on a key here it removes that key (the track stays animated).
                  onTap: () => doc.setKey(widget.id, ks == KeyS.at ? KeyS.anim : KeyS.at),
                )
              else
                const SizedBox(width: _keyW),
              if (cfg.resetArrow) ResetMark(show: changed && !locked, onTap: () => doc.resetIds(ids)),
            ]),
          ),
          if (widget.note != null) Padding(padding: EdgeInsets.only(left: lw, bottom: 2), child: Text(widget.note!, style: T.label(N.g56))),
          if (showRel) Padding(padding: EdgeInsets.only(left: lw, bottom: 2), child: Text('Driven by ${widget.rel!.name}${widget.relName != null ? ' · ${widget.relName}' : ''}. Edit it in Relations.', style: T.label(widget.rel!.c))),
          if (_menu)
            Container(
              margin: const EdgeInsets.only(left: 0, top: 2, bottom: 4),
              decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(4), border: Border.all(color: ed.rule)),
              child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
                item('Key this frame', ks != KeyS.at, 'Already keyed', () => doc.setKey(widget.id, KeyS.at)),
                item('Remove key', ks == KeyS.at, 'No key here', () => doc.setKey(widget.id, KeyS.anim)),
                item('Remove animation', ks != KeyS.off, 'Not animated', () => doc.setKey(widget.id, KeyS.off)),
                if (cfg.maybe) item('Relate…', true, '', () {}),
              ]),
            ),
        ]),
      );
    });
  }
}

// ---- 6. choosers and hand-overs -------------------------------------------------------------------------------------
/// A short list lives in a segmented control; a long one opens a searchable list (G2, G3). Never prev/next cycling (do-not-repeat 6, 7).
class Chooser extends StatefulWidget {
  const Chooser({super.key, required this.id, required this.options, this.searchable = false, this.word = 'Choose', this.enabled = true});
  final String id, word;
  final List<String> options;
  final bool searchable, enabled;
  @override
  State<Chooser> createState() => _ChooserState();
}

class _ChooserState extends State<Chooser> {
  bool _open = false;
  final _q = TextEditingController();
  final _qf = FocusNode(debugLabel: 'chooser-q');

  @override
  void dispose() {
    _q.dispose();
    _qf.dispose();
    super.dispose();
  }

  void _toggle() {
    setState(() => _open = !_open);
    _q.clear();
    if (_open && widget.searchable) WidgetsBinding.instance.addPostFrameCallback((_) => _qf.requestFocus());
  }

  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), ed = Ed(x.cfg.ed), cur = x.doc.s2[widget.id] ?? widget.options.first;
    final on = widget.enabled && !x.cfg.locked && !RowOff.offOf(context);
    final hits = [for (final o in widget.options) if (o.toLowerCase().contains(_q.text.toLowerCase())) o];
    void pick(String o) {
      x.doc.str(widget.id, o);
      setState(() => _open = false);
    }

    return Focus(
      onKeyEvent: (n, e) {
        if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape && _open) {
          setState(() => _open = false);
          return KeyEventResult.handled;
        }
        return KeyEventResult.ignored;
      },
      child: Opacity(
        opacity: 1,
        child: IgnorePointer(
          ignoring: !on,
          child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            Hov(
              onTap: _toggle,
              builder: (_, h) => Container(
                height: _cellH,
                padding: const EdgeInsets.symmetric(horizontal: 8),
                decoration: BoxDecoration(color: !on ? N.g10 : (x.cfg.look == Look.quiet ? N.g13 : N.g07), borderRadius: BorderRadius.circular(4), border: Border.all(color: _open ? N.g63 : (h && on ? N.g38 : (x.cfg.look == Look.quiet && x.cfg.ed == Editorial.a ? const Color(0x00000000) : ed.rule)))),
                child: Row(children: [
                  // the default value (the first option) is dim; a chosen one is bright (dim = default)
                  Expanded(child: Text(cur, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(!on || cur == widget.options.first ? N.g56 : N.g91))),
                  Text(_open ? 'Close' : widget.word, style: T.label(N.g56)),
                ]),
              ),
            ),
            if (_open)
              Container(
                margin: const EdgeInsets.only(top: 2),
                decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(4), border: Border.all(color: ed.rule)),
                child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
                  if (widget.searchable)
                    Container(
                      height: 24,
                      padding: const EdgeInsets.symmetric(horizontal: 8),
                      decoration: BoxDecoration(border: Border(bottom: BorderSide(color: ed.rule))),
                      alignment: Alignment.centerLeft,
                      child: Stack(alignment: Alignment.centerLeft, children: [
                        if (_q.text.isEmpty) Text('Search', style: T.label(N.g56)),
                        EditableText(
                          controller: _q,
                          focusNode: _qf,
                          style: T.name(N.g95),
                          cursorColor: N.g95,
                          backgroundCursorColor: N.g20,
                          maxLines: 1,
                          onChanged: (_) => setState(() {}),
                          onSubmitted: (_) {
                            if (hits.isNotEmpty) pick(hits.first);
                          },
                        ),
                      ]),
                    ),
                  SizedBox(
                    height: math.min(math.max(hits.length, 1), 6) * 22.0,
                    child: hits.isEmpty
                        ? Padding(padding: const EdgeInsets.all(6), child: Text('No match', style: T.label(N.g56)))
                        : ListView.builder(
                            padding: EdgeInsets.zero,
                            itemExtent: 22,
                            itemCount: hits.length,
                            itemBuilder: (_, i) {
                              final o = hits[i], sel = o == cur;
                              return Hov(
                                onTap: () => pick(o),
                                builder: (_, h) => Container(
                                  padding: const EdgeInsets.only(left: 8),
                                  decoration: BoxDecoration(color: sel ? N.g20 : (h ? N.g15 : null), border: sel ? const Border(left: BorderSide(color: N.g95, width: 2)) : null),
                                  alignment: Alignment.centerLeft,
                                  child: Text(o, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(sel ? N.g95 : N.g76)),
                                ),
                              );
                            },
                          ),
                  ),
                ]),
              ),
          ]),
        ),
      ),
    );
  }
}

/// The value is shown; the editing is done by the specialist tool, already aimed at this slot (I1). No popup.
class HandOver extends StatefulWidget {
  const HandOver({super.key, required this.id, required this.tool, this.swatch});
  final String id, tool;
  final Color? swatch;
  @override
  State<HandOver> createState() => _HandOverState();
}

class _HandOverState extends State<HandOver> {
  bool _sent = false;
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), ed = Ed(x.cfg.ed), val = x.doc.s2[widget.id] ?? '', off = x.cfg.locked || RowOff.offOf(context);
    return Hov(
      onTap: off ? null : () => setState(() => _sent = !_sent),
      builder: (_, h) => Opacity(
        opacity: 1,
        child: Container(
          height: _cellH,
          padding: const EdgeInsets.symmetric(horizontal: 8),
          decoration: BoxDecoration(color: x.cfg.look == Look.quiet ? N.g13 : N.g07, borderRadius: BorderRadius.circular(4), border: Border.all(color: _sent ? N.g63 : (h ? N.g38 : (x.cfg.look == Look.quiet && x.cfg.ed == Editorial.a ? const Color(0x00000000) : ed.rule)))),
          child: Row(children: [
            // [open] a swatch shows the property's own data colour; it is not chrome.
            if (widget.swatch != null) Padding(padding: const EdgeInsets.only(right: 8), child: Container(width: 10, height: 10, decoration: BoxDecoration(color: widget.swatch, borderRadius: BorderRadius.circular(2), border: Border.all(color: ed.rule)))),
            Expanded(child: Text(val.isEmpty ? '—' : val, maxLines: 1, overflow: TextOverflow.ellipsis, textAlign: TextAlign.start, style: widget.swatch != null ? T.value(off ? N.g56 : N.g91) : T.name(off ? N.g56 : N.g91))),
            Text(_sent ? '${widget.tool} open' : widget.tool, style: T.label(_sent ? N.g95 : N.g56)),
          ]),
        ),
      ),
    );
  }
}

// ---- 7. header and empty ----------------------------------------------------------------------------------------------
class InspHeader extends StatelessWidget {
  const InspHeader({super.key});
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, ed = Ed(x.cfg.ed);
    final (word, glyph) = switch (doc.kind) {
      Kind.shape => ('Shape', G.shape),
      Kind.text => ('Text', G.text),
      Kind.camera => ('Camera', G.camera),
      Kind.group => ('Group', G.grid),
      Kind.several => ('Layers', null),
      Kind.none => ('', null),
    };
    return Padding(
      padding: const EdgeInsets.only(bottom: 4),
      child: Row(crossAxisAlignment: CrossAxisAlignment.center, children: [
        if (glyph != null) ...[Glyph(glyph, size: 16, color: N.g76), const SizedBox(width: 8)],
        Expanded(
          child: Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
            Text(ed.kick(doc.kind == Kind.several ? 'Several selected' : word) + (x.cfg.locked ? (ed.e == Editorial.b ? ' · Locked' : ' · LOCKED') : ''), maxLines: 1, overflow: TextOverflow.ellipsis, style: ed.kickStyle(N.g56)),
            const SizedBox(height: 4),
            Text(doc.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: ed.nameStyle().copyWith(color: x.cfg.locked ? N.g76 : null)),
          ]),
        ),
        const SizedBox(width: 8),
        // contract: while on, editing an unkeyed value writes its first key at the playhead (D2).
        Column(crossAxisAlignment: CrossAxisAlignment.end, mainAxisSize: MainAxisSize.min, children: [
          Text('Animate all', style: T.label(x.cfg.locked ? N.g56 : (doc.animateAll ? N.g95 : N.g63))),
          const SizedBox(height: 4),
          OnOff(on: doc.animateAll, enabled: !x.cfg.locked, onChanged: (v) {
            doc.animateAll = v;
            doc.poke();
          }),
        ]),
      ]),
    );
  }
}

class EmptyState extends StatelessWidget {
  const EmptyState({super.key});
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), ed = Ed(x.cfg.ed);
    // contract: one plain muted line, no controls, no stale values (A6). With the maybe-knob on, the composition's own settings take its place.
    return Column(crossAxisAlignment: CrossAxisAlignment.stretch, mainAxisSize: MainAxisSize.min, children: [
      Padding(padding: const EdgeInsets.only(top: 4, bottom: 8), child: Text('Nothing selected. Pick a layer on the Stage or in the Timeline.', style: T.label(N.g56).copyWith(height: 1.4))),
      if (x.cfg.maybe)
        Sect(title: 'Composition', first: true, children: [
          PropRow(id: 'comp.w', label: 'Width', keyable: false, cells: const [NumField(id: 'comp.w', unit: 'px', min: 16, max: 8192)]),
          PropRow(id: 'comp.h', label: 'Height', keyable: false, cells: const [NumField(id: 'comp.h', unit: 'px', min: 16, max: 8192)]),
          PropRow(id: 'comp.fps', label: 'Frame rate', keyable: false, cells: const [NumField(id: 'comp.fps', unit: 'fps', min: 1, max: 240)]),
        ])
      else
        const SizedBox(height: 1),
      if (!x.cfg.maybe) Container(height: 1, color: ed.rule),
    ]);
  }
}

// ---- 8. the transform group ------------------------------------------------------------------------------------------
class SpaceRow extends StatelessWidget {
  const SpaceRow({super.key});
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc;
    // contract: 2D / 2.5D / 3D is one switch; it reveals Z, the other rotation axes and Depth, and changes no existing value (B2).
    return PropRow(id: 'space', label: 'Space', keyable: false, enabled: !x.cfg.locked, cells: [
      Dis(
        child: Segmented(items: const ['2D', '2.5D', '3D'], index: doc.space.index, expand: true, onChanged: (i) {
          doc.space = Space.values[i];
          doc.poke();
        }),
      ),
    ]);
  }
}

class PosRow extends StatelessWidget {
  const PosRow({super.key});
  @override
  Widget build(BuildContext context) {
    final three = Ctx.of(context).doc.space != Space.d2;
    return PropRow(id: 'pos', label: 'Position', ids: [if (true) ...['pos.x', 'pos.y'], if (three) 'pos.z'], cells: [
      const NumField(id: 'pos.x', label: 'X', axis: _axX, unit: 'px', decimals: 1),
      const NumField(id: 'pos.y', label: 'Y', axis: _axY, unit: 'px', decimals: 1),
      if (three) const NumField(id: 'pos.z', label: 'Z', axis: _axZ, unit: 'px', decimals: 1),
    ]);
  }
}

class ScaleRow extends StatelessWidget {
  const ScaleRow({super.key});
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, three = doc.space != Space.d2;
    final axes = ['x', 'y', if (three) 'z'];
    // contract: with the link on, a factor applies to every axis (2:3, X=4 gives 4:6); unlinked writes only the touched axis; a zero axis gets the typed value (E1). Toggling the link changes nothing.
    void set(String a, double v) {
      final id = 'scale.$a';
      if (!doc.link) {
        doc.set(id, v);
        return;
      }
      final old = doc.get(id) ?? 100;
      if (old == 0) {
        doc.setMany({for (final o in axes) 'scale.$o': v});
        return;
      }
      final f = v / old;
      doc.setMany({for (final o in axes) 'scale.$o': o == a ? v : (doc.get('scale.$o') ?? 100) * f});
    }

    NumField cell(int i) => NumField(id: 'scale.${axes[i]}', label: axes[i].toUpperCase(), axis: _axes[i], unit: '%', decimals: 1, onSet: (v) => set(axes[i], v));
    return PropRow(
      id: 'scale',
      label: 'Scale',
      ids: [for (final a in axes) 'scale.$a'],
      under: Chip('Link', on: doc.link, enabled: !x.cfg.locked, onTap: () {
        doc.link = !doc.link;
        doc.poke();
      }),
      cells: [for (var i = 0; i < axes.length; i++) cell(i)],
    );
  }
}

class RotRow extends StatelessWidget {
  const RotRow({super.key});
  @override
  Widget build(BuildContext context) {
    final three = Ctx.of(context).doc.space != Space.d2;
    // contract: rotation keeps its turns, typed 450 stays 450 and reads "1x 90" (B5).
    return PropRow(id: 'rot', label: 'Rotation', ids: [if (three) ...['rot.x', 'rot.y'], 'rot.z'], cells: [
      if (three) const NumField(id: 'rot.x', label: 'X', axis: _axX, unit: '°', decimals: 1, perPx: .5),
      if (three) const NumField(id: 'rot.y', label: 'Y', axis: _axY, unit: '°', decimals: 1, perPx: .5),
      const NumField(id: 'rot.z', label: 'Z', axis: _axZ, unit: '°', decimals: 1, perPx: .5, turns: true),
    ]);
  }
}

const _anchorNames = ['Top left', 'Top', 'Top right', 'Left', 'Centre', 'Right', 'Bottom left', 'Bottom', 'Bottom right'];

class AnchorRow extends StatefulWidget {
  const AnchorRow({super.key});
  @override
  State<AnchorRow> createState() => _AnchorRowState();
}

class _AnchorRowState extends State<AnchorRow> {
  int? _hover;
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, ax = doc.get('anchor.x'), ay = doc.get('anchor.y');
    int? sel;
    if (ax != null && ay != null) {
      for (var i = 0; i < 9; i++) {
        if (ax == (i % 3) * 50 && ay == (i ~/ 3) * 50) sel = i;
      }
    }
    // contract: one click sets the pivot; a custom pivot is named Custom and typed values still work; the layer does not move visually (B4). Hover preview on the Stage is a maybe-card (I3).
    final name = _hover != null && x.cfg.maybe ? 'Preview: ${_anchorNames[_hover!]}' : (sel != null ? _anchorNames[sel] : 'Custom');
    Widget cell(int i) => Hov(
          onTap: x.cfg.locked ? null : () => doc.setMany({'anchor.x': (i % 3) * 50.0, 'anchor.y': (i ~/ 3) * 50.0}),
          builder: (_, h) {
            if (h && _hover != i) {
              WidgetsBinding.instance.addPostFrameCallback((_) {
                if (mounted && _hover != i) setState(() => _hover = i);
              });
            } else if (!h && _hover == i) {
              WidgetsBinding.instance.addPostFrameCallback((_) {
                if (mounted && _hover == i) setState(() => _hover = null);
              });
            }
            return SizedBox(
              width: 16,
              height: 16,
              child: Center(child: Container(width: 10, height: 10, decoration: BoxDecoration(color: sel == i ? N.g95 : null, borderRadius: BorderRadius.circular(2), border: Border.all(color: sel == i ? N.g95 : (h ? N.g76 : N.g44))))),
            );
          },
        );
    final grid = Column(mainAxisSize: MainAxisSize.min, children: [
      for (var r = 0; r < 3; r++) Row(mainAxisSize: MainAxisSize.min, children: [for (var c = 0; c < 3; c++) cell(r * 3 + c)]),
    ]);
    return PropRow(id: 'anchor', label: 'Anchor', ids: const ['anchor.x', 'anchor.y'], cells: [
      Row(crossAxisAlignment: CrossAxisAlignment.center, children: [
        Padding(padding: const EdgeInsets.symmetric(vertical: 2), child: grid),
        const SizedBox(width: 8),
        Expanded(
          child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, mainAxisSize: MainAxisSize.min, children: [
            Text(name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(sel == null ? N.g76 : N.g56)),
            const SizedBox(height: 2),
            const Row(children: [
              Expanded(child: NumField(id: 'anchor.x', label: 'X', unit: '%', decimals: 0)),
              SizedBox(width: 4),
              Expanded(child: NumField(id: 'anchor.y', label: 'Y', unit: '%', decimals: 0)),
            ]),
          ]),
        ),
      ]),
    ]);
  }
}

class OpacityRow extends StatelessWidget {
  const OpacityRow({super.key});
  @override
  // contract: opacity is 0-100 %, display only (0.5 stored reads 50 %); the fill shows where it sits in its range (B6, E2).
  Widget build(BuildContext context) => const PropRow(id: 'op', label: 'Opacity', cells: [NumField(id: 'op', unit: '%', decimals: 0, min: 0, max: 100, perPx: .5)]);
}

class DepthRow extends StatelessWidget {
  const DepthRow({super.key});
  @override
  // contract: Depth exists only outside 2D; 0 reads as the word "Flat" (E3).
  Widget build(BuildContext context) => Ctx.of(context).doc.space == Space.d2 ? const SizedBox.shrink() : const PropRow(id: 'depth', label: 'Depth', cells: [NumField(id: 'depth', unit: 'px', decimals: 0, zeroWord: 'Flat')]);
}

const _layers = ['None', 'Backdrop', 'Hero title', 'Jewel Field', 'Jewel 01', 'Jewel 02', 'Jewel 03', 'Jewel 04', 'Jewel 05', 'Jewel 06', 'Glow plate', 'Lower third', 'Logo', 'Camera 1', 'Audio bars', 'Vignette', 'Null 1', 'Null 2'];

class ParentRow extends StatelessWidget {
  const ParentRow({super.key});
  @override
  // contract: shows the parent's name or None; a list with search, never prev/next arrows; changing it does not move the layer (G3).
  Widget build(BuildContext context) => const PropRow(id: 'parent', label: 'Parent', keyable: false, cells: [Chooser(id: 'parent', options: _layers, searchable: true)]);
}

class TransformGroup extends StatelessWidget {
  const TransformGroup({super.key, this.titled = true, this.first = true});
  final bool titled, first;
  @override
  Widget build(BuildContext context) => Sect(title: titled ? 'Transform' : null, first: first, children: const [SpaceRow(), PosRow(), ScaleRow(), RotRow(), AnchorRow(), OpacityRow(), DepthRow(), ParentRow()]);
}

// ---- 9. material -------------------------------------------------------------------------------------------------------
const _blends = ['Normal', 'Add', 'Multiply', 'Screen', 'Overlay', 'Soft light', 'Hard light', 'Difference'];

class BlendRows extends StatelessWidget {
  const BlendRows({super.key, this.titled = true, this.first = false});
  final bool titled, first;
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    // contract: Blend opens the blend chooser; Ghost and Clip to below are On/Off words; hidden for a camera (G1).
    return Sect(title: titled ? 'Blend' : null, first: first, children: [
      const PropRow(id: 'blend', label: 'Blend', keyable: false, cells: [Chooser(id: 'blend', options: _blends, word: 'Modes')]),
      PropRow(id: 'ghost', label: 'Ghost', keyable: false, cells: [Align(alignment: Alignment.centerLeft, child: OnOff(on: doc.b['ghost'] ?? false, onChanged: (v) => doc.flag('ghost', v)))]),
      PropRow(id: 'clip', label: 'Clip to below', keyable: false, cells: [Align(alignment: Alignment.centerLeft, child: OnOff(on: doc.b['clip'] ?? false, onChanged: (v) => doc.flag('clip', v)))]),
    ]);
  }
}

class FillStroke extends StatelessWidget {
  const FillStroke({super.key, this.first = true});
  final bool first;
  @override
  // contract: the field shows the value and hands over to the Colours tool aimed at THIS slot (I1). Stroke width counts only while a stroke exists.
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc, none = (doc.s2['stroke'] ?? 'None') == 'None';
    return Sect(title: 'Fill and stroke', first: false, children: [
      const PropRow(id: 'fill', label: 'Fill', keyable: false, cells: [HandOver(id: 'fill', tool: 'Colours', swatch: Color(0xFFD9D2C3))]),
      const PropRow(id: 'stroke', label: 'Stroke', keyable: false, cells: [Chooser(id: 'stroke', options: ['None', '#F2F2F2', '#C1C1C1', '#8E8E8E'], word: 'Colours')]),
      PropRow(id: 'stroke.w', label: 'Stroke width', enabled: !none, note: none ? 'Off while Stroke is None' : null, cells: [NumField(id: 'stroke.w', unit: 'px', decimals: 1, min: 0, max: 64, perPx: .1)]),
    ]);
  }
}

class TextGroup extends StatefulWidget {
  const TextGroup({super.key, this.first = true});
  final bool first;
  @override
  State<TextGroup> createState() => _TextGroupState();
}

class _TextGroupState extends State<TextGroup> {
  final _c = TextEditingController();
  late final FocusNode _f = FocusNode(debugLabel: 'text-content', onKeyEvent: (n, e) {
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape) {
      // contract: Esc restores the text as it was when the field was entered (A3).
      _c.text = _entered;
      _f.unfocus();
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  });
  String _entered = '';
  bool _init = false;

  @override
  void initState() {
    super.initState();
    _f.addListener(() {
      final x = Ctx.read(context);
      if (_f.hasFocus) {
        _entered = _c.text;
      } else {
        x.doc.str('text.content', _c.text);
      }
      setState(() {});
    });
  }

  @override
  void dispose() {
    _f.dispose();
    _c.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, ed = Ed(x.cfg.ed);
    if (!_init) {
      _c.text = doc.s2['text.content'] ?? '';
      _init = true;
    }
    const aligns = ['Left', 'Centre', 'Right', 'Justify'];
    return Sect(title: 'Text', first: widget.first, children: [
      Padding(
        padding: const EdgeInsets.symmetric(vertical: 2),
        child: Opacity(
          opacity: x.cfg.locked ? .5 : 1,
          child: IgnorePointer(
            ignoring: x.cfg.locked,
            child: Container(
              constraints: const BoxConstraints(minHeight: 48),
              padding: const EdgeInsets.all(8),
              decoration: BoxDecoration(color: x.cfg.look == Look.quiet ? N.g13 : N.g07, borderRadius: BorderRadius.circular(4), border: Border.all(color: _f.hasFocus ? N.g63 : (x.cfg.look == Look.quiet && x.cfg.ed == Editorial.a ? const Color(0x00000000) : ed.rule))),
              child: EditableText(controller: _c, focusNode: _f, style: T.name(N.g95).copyWith(fontWeight: FontWeight.w400, height: 1.4), cursorColor: N.g95, backgroundCursorColor: N.g20, minLines: 2, maxLines: 4, selectionColor: C.mode.withValues(alpha: .4)),
            ),
          ),
        ),
      ),
      Cap('Applies when you leave the box · Esc restores', color: N.g56),
      const PropRow(id: 'font', label: 'Font', keyable: false, cells: [HandOver(id: 'font', tool: 'Fonts')]),
      const PropRow(id: 'text.size', label: 'Size', cells: [NumField(id: 'text.size', unit: 'px', decimals: 0, min: 1, max: 2000)]),
      PropRow(id: 'align', label: 'Alignment', keyable: false, enabled: !x.cfg.locked, cells: [
        Dis(child: Segmented(items: aligns, index: aligns.indexOf(doc.s2['align'] ?? 'Left'), expand: true, onChanged: (i) => doc.str('align', aligns[i]))),
      ]),
    ]);
  }
}

// ---- 10. effects ------------------------------------------------------------------------------------------------------
enum PK { num, toggle, choice }

class Prm {
  const Prm(this.id, this.label, {this.min = 0, this.max = 100, this.def = 0, this.unit = '', this.dec = 0, this.kind = PK.num, this.opts = const [], this.hero = false});
  final String id, label, unit;
  final double min, max, def;
  final int dec;
  final PK kind;
  final List<String> opts;
  final bool hero;
}

class Fx {
  const Fx(this.id, this.name, this.params);
  final String id, name;
  final List<Prm> params;
}

const kEffects = [
  Fx('blur', 'Blur', [
    Prm('radius', 'Radius', max: 200, def: 12, unit: 'px', hero: true),
    Prm('angle', 'Angle', min: -180, max: 180, unit: '°', hero: true),
    Prm('mix', 'Mix', def: 100, unit: '%', hero: true),
    Prm('quality', 'Quality', kind: PK.choice, opts: ['Low', 'Med', 'High'], def: 1, hero: true),
    Prm('edge', 'Repeat edges', kind: PK.toggle, def: 1),
    Prm('gamma', 'Gamma', min: .1, max: 4, def: 1, dec: 2),
    Prm('dx', 'Offset X', min: -100, max: 100, unit: 'px'),
    Prm('dy', 'Offset Y', min: -100, max: 100, unit: 'px'),
  ]),
  Fx('glow', 'Glow', [
    Prm('thr', 'Threshold', def: 60, unit: '%', hero: true),
    Prm('size', 'Size', max: 300, def: 40, unit: 'px', hero: true),
    Prm('gain', 'Gain', max: 400, def: 100, unit: '%', hero: true),
    Prm('knee', 'Knee', def: 20, unit: '%', hero: true),
    Prm('tint', 'Tint', def: 0, unit: '%'),
    Prm('aspect', 'Aspect', min: .1, max: 4, def: 1, dec: 2),
    Prm('falloff', 'Falloff', def: 50, unit: '%'),
    Prm('add', 'Additive', kind: PK.toggle, def: 1),
  ]),
  Fx('color', 'Colour', [
    Prm('exp', 'Exposure', min: -4, max: 4, dec: 2, hero: true),
    Prm('con', 'Contrast', min: -100, max: 100, hero: true),
    Prm('sat', 'Saturation', max: 200, def: 100, unit: '%', hero: true),
    Prm('hue', 'Hue', min: -180, max: 180, unit: '°', hero: true),
    Prm('temp', 'Temperature', min: -100, max: 100),
    Prm('tintc', 'Tint', min: -100, max: 100),
    Prm('gam', 'Gamma', min: .1, max: 4, def: 1, dec: 2),
    Prm('clampc', 'Clamp', kind: PK.toggle),
  ]),
  Fx('warp', 'Distort', [
    Prm('amt', 'Amount', min: -100, max: 100, def: 20, unit: '%', hero: true),
    Prm('freq', 'Frequency', max: 40, def: 6, dec: 1, hero: true),
    Prm('speed', 'Speed', min: -10, max: 10, def: 1, dec: 1, hero: true),
    Prm('seed', 'Seed', max: 999, hero: true),
    Prm('oct', 'Octaves', min: 1, max: 8, def: 2),
    Prm('lac', 'Lacunarity', min: 1, max: 4, def: 2, dec: 2),
    Prm('rough', 'Roughness', def: 50, unit: '%'),
    Prm('pin', 'Pin edges', kind: PK.toggle),
  ]),
];

class EffectCard extends StatefulWidget {
  const EffectCard(this.fx, {super.key, this.first = false});
  final Fx fx;
  final bool first;
  @override
  State<EffectCard> createState() => _EffectCardState();
}

class _EffectCardState extends State<EffectCard> {
  bool _menu = false;
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, ed = Ed(x.cfg.ed), fx = widget.fx, k = fx.id;
    final enabled = doc.b['$k.on'] ?? true, open = doc.b['$k.open'] ?? true, adv = doc.b['$k.adv'] ?? false;
    final q = doc.query.toLowerCase();
    final shown = [for (final p in fx.params) if (q.isEmpty || p.label.toLowerCase().contains(q)) p];
    if (q.isNotEmpty && shown.isEmpty) return const SizedBox.shrink();
    final heroes = [for (final p in shown) if (p.hero || q.isNotEmpty) p], rest = [for (final p in shown) if (!(p.hero || q.isNotEmpty)) p];
    final i = doc.fxOrder.indexOf(k), last = doc.fxOrder.length - 1;
    final locked = x.cfg.locked;
    Widget row(Prm p) {
      final id = '${k}_${p.id}';
      switch (p.kind) {
        case PK.num:
          return PropRow(id: id, label: p.label, enabled: enabled, cells: [NumField(id: id, unit: p.unit, decimals: p.dec, min: p.min, max: p.max, perPx: (p.max - p.min) / 200, step: p.dec > 0 ? math.pow(10, -p.dec).toDouble() * 10 : 1)]);
        case PK.toggle:
          return PropRow(id: id, label: p.label, keyable: false, enabled: enabled, cells: [Align(alignment: Alignment.centerLeft, child: OnOff(on: (doc.get(id) ?? 0) > 0, onChanged: (v) => doc.set(id, v ? 1 : 0)))]);
        case PK.choice:
          return PropRow(id: id, label: p.label, keyable: false, enabled: enabled, cells: [Dis(child: Segmented(items: p.opts, index: (doc.get(id) ?? 0).round(), expand: true, onChanged: (i) => doc.set(id, i.toDouble())))]);
      }
    }

    List<Widget> pairs(List<Prm> ps) => [for (final p in ps) row(p)];

    Widget mi(String s, bool ok, VoidCallback go, {bool danger = false}) => Hov(
          cursor: ok ? SystemMouseCursors.click : SystemMouseCursors.basic,
          onTap: ok && !locked
              ? () {
                  go();
                  setState(() => _menu = false);
                }
              : null,
          builder: (_, h) => Container(height: 22, padding: const EdgeInsets.symmetric(horizontal: 8), color: h && ok ? N.g15 : null, alignment: Alignment.centerLeft, child: Text(s, style: T.name(!ok ? N.g56 : (danger ? C.danger : N.g91)))),
        );
    return Sect(
      title: fx.name,
      sub: true,
      first: widget.first,
      open: open,
      hint: '${fx.params.length} parameters',
      // contract: clicking the name folds; the switch bypasses (the body dims); More holds order, reset, randomise, remove (F1, F5).
      onToggle: () => doc.flag('$k.open', !open),
      trailing: Row(mainAxisSize: MainAxisSize.min, children: [
        Hov(onTap: () => setState(() => _menu = !_menu), builder: (_, h) => Padding(padding: const EdgeInsets.only(right: 12), child: Text('More', style: T.label(_menu || h ? N.g95 : N.g56)))),
        OnOff(on: enabled, onChanged: (v) => doc.flag('$k.on', v)),
      ]),
      children: [
        if (_menu)
          Container(
            margin: const EdgeInsets.only(bottom: 4),
            decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(4), border: Border.all(color: ed.rule)),
            child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
              mi('Move earlier', i > 0, () => doc.moveFx(k, -1)),
              mi('Move later', i < last, () => doc.moveFx(k, 1)),
              mi('Reset to defaults', true, () => doc.resetIds([for (final p in fx.params) '${k}_${p.id}'])),
              mi('Randomise values', true, () => doc.randomFx(fx)),
              mi('Remove', true, () => doc.removeFx(k), danger: true),
            ]),
          ),
        Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: pairs(heroes)),
        if (rest.isNotEmpty) ...[
          Padding(
            padding: const EdgeInsets.only(top: 4),
            child: Hov(
              onTap: () => doc.flag('$k.adv', !adv),
              builder: (_, h) => Container(
                height: 24,
                decoration: BoxDecoration(border: Border(top: BorderSide(color: ed.rule))),
                child: Row(children: [
                  Text('Advanced', style: ed.labStyle(h || adv ? N.g95 : N.g63)),
                  const Spacer(),
                  Text(adv ? 'Fold' : '${rest.length} more', style: T.label(N.g56)),
                ]),
              ),
            ),
          ),
          if (adv || q.isNotEmpty) Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: pairs(rest)),
        ],
      ],
    );
  }
}

class EffectsStack extends StatelessWidget {
  const EffectsStack({super.key});
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc;
    final cards = [for (var i = 0; i < doc.fxOrder.length; i++) EffectCard(kEffects.firstWhere((f) => f.id == doc.fxOrder[i]), key: ValueKey(doc.fxOrder[i]), first: i == 0)];
    final none = doc.query.isNotEmpty && !doc.fxOrder.any((id) => kEffects.firstWhere((f) => f.id == id).params.any((p) => p.label.toLowerCase().contains(doc.query.toLowerCase())));
    return Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      if (x.cfg.maybe) const ParamSearch(),
      if (doc.fxOrder.isEmpty) Padding(padding: const EdgeInsets.symmetric(vertical: 8), child: Text('No effects. Add one from the Browser.', style: T.label(N.g56))),
      if (none) Padding(padding: const EdgeInsets.symmetric(vertical: 8), child: Text('No parameter matches', style: T.label(N.g56))) else ...cards,
    ]);
  }
}

/// H3 (maybe): search flattens the groups, unfolds Advanced, and says so when nothing matches.
class ParamSearch extends StatefulWidget {
  const ParamSearch({super.key});
  @override
  State<ParamSearch> createState() => _ParamSearchState();
}

class _ParamSearchState extends State<ParamSearch> {
  final _c = TextEditingController();
  final _f = FocusNode(debugLabel: 'param-search');
  @override
  void dispose() {
    _c.dispose();
    _f.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), ed = Ed(x.cfg.ed);
    return Container(
      height: _cellH,
      margin: const EdgeInsets.only(bottom: 4),
      padding: const EdgeInsets.symmetric(horizontal: 8),
      alignment: Alignment.centerLeft,
      decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(4), border: Border.all(color: ed.rule)),
      child: Stack(alignment: Alignment.centerLeft, children: [
        if (_c.text.isEmpty) Text('Search parameters', style: T.label(N.g56)),
        EditableText(
          controller: _c,
          focusNode: _f,
          style: T.name(N.g95),
          cursorColor: N.g95,
          backgroundCursorColor: N.g20,
          maxLines: 1,
          onChanged: (s) {
            x.doc.query = s;
            x.doc.poke();
            setState(() {});
          },
        ),
      ]),
    );
  }
}

// ---- 11. camera and layout ------------------------------------------------------------------------------------------
/// D4 (maybe): a group's key mark keys every member at once.
class GroupKey extends StatelessWidget {
  const GroupKey(this.group, this.members, {super.key});
  final String group;
  final List<String> members;
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc;
    if (!x.cfg.maybe) return const SizedBox.shrink();
    final all = members.every((m) => doc.keys[m] == KeyS.at), any = members.any((m) => (doc.keys[m] ?? KeyS.off) != KeyS.off);
    return KeyMark(state: all ? KeyS.at : (any ? KeyS.anim : KeyS.off), enabled: !x.cfg.locked, onTap: () {
      for (final m in members) {
        doc.keys[m] = all ? KeyS.anim : KeyS.at;
      }
      doc.poke();
    });
  }
}

class CameraGroup extends StatelessWidget {
  const CameraGroup({super.key, this.first = true});
  final bool first;
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc, driven = (doc.s2['camTarget'] ?? 'None') != 'None';
    // contract: Target / Orbit / Framing / Roll. When a target layer drives the target its fields are disabled and say why (A4).
    return Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      Sect(title: 'Target', first: first, trailing: const GroupKey('Target', ['cam.t']), children: [
        const PropRow(id: 'camTarget', label: 'Layer', keyable: false, cells: [Chooser(id: 'camTarget', options: _layers, searchable: true)]),
        PropRow(id: 'cam.t', label: 'Point', enabled: !driven, note: driven ? 'Set by ${doc.s2['camTarget']}. Choose None to edit.' : null, ids: const ['cam.t.x', 'cam.t.y', 'cam.t.z'], cells: [
          NumField(id: 'cam.t.x', label: 'X', axis: _axX, unit: 'px', enabled: !driven),
          NumField(id: 'cam.t.y', label: 'Y', axis: _axY, unit: 'px', enabled: !driven),
          NumField(id: 'cam.t.z', label: 'Z', axis: _axZ, unit: 'px', enabled: !driven),
        ]),
      ]),
      Sect(title: 'Orbit', trailing: const GroupKey('Orbit', ['cam.pitch', 'cam.yaw']), children: const [
        PropRow(id: 'cam.pitch', label: 'Pitch', cells: [NumField(id: 'cam.pitch', unit: '°', decimals: 1, min: -90, max: 90, perPx: .5)]),
        PropRow(id: 'cam.yaw', label: 'Yaw', cells: [NumField(id: 'cam.yaw', unit: '°', decimals: 1, perPx: .5)]),
      ]),
      const Sect(title: 'Framing', children: [
        PropRow(id: 'cam.dist', label: 'Distance', cells: [NumField(id: 'cam.dist', unit: 'px', min: 10, max: 10000, perPx: 5, step: 10)]),
        PropRow(id: 'cam.zoom', label: 'Zoom', cells: [NumField(id: 'cam.zoom', unit: '%', min: 10, max: 1000, perPx: .5)]),
      ]),
      const Sect(title: 'Roll', children: [
        PropRow(id: 'cam.roll', label: 'Roll', cells: [NumField(id: 'cam.roll', unit: '°', decimals: 1, perPx: .5, turns: true)]),
      ]),
    ]);
  }
}

class LayoutGroup extends StatelessWidget {
  const LayoutGroup({super.key, this.first = false});
  final bool first;
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, on = doc.b['lay.on'] ?? true, fixed = (doc.s2['size'] ?? 'Hug') == 'Fixed';
    const sizes = ['Hug', 'Fill', 'Fixed'];
    // contract (A5, maybe): turning Layout off dims everything but its switch; the size number counts only when the size mode is Fixed; a child can opt out.
    return Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      Sect(title: 'Layout', first: first, trailing: OnOff(on: on, onChanged: (v) => doc.flag('lay.on', v)), children: [
        Opacity(
          opacity: on ? 1 : .4,
          child: IgnorePointer(
            ignoring: !on,
            child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
              const PropRow(id: 'lay.cols', label: 'Columns', cells: [NumField(id: 'lay.cols', min: 1, max: 12, unit: '')]),
              const PropRow(id: 'lay.rows', label: 'Rows', cells: [NumField(id: 'lay.rows', min: 1, max: 12, zeroWord: null)]),
              const PropRow(id: 'lay.gap', label: 'Gap', cells: [NumField(id: 'lay.gap', unit: 'px', min: 0, max: 200)]),
              const PropRow(id: 'lay.pad', label: 'Padding', cells: [NumField(id: 'lay.pad', unit: 'px', min: 0, max: 200)]),
              PropRow(id: 'size', label: 'Size', keyable: false, cells: [Dis(child: Segmented(items: sizes, index: sizes.indexOf(doc.s2['size'] ?? 'Hug'), expand: true, onChanged: (i) => doc.str('size', sizes[i])))]),
              PropRow(id: 'lay.w', label: 'Width', note: fixed ? null : 'Counts only when Size is Fixed', cells: [NumField(id: 'lay.w', unit: 'px', min: 0, max: 8000, enabled: fixed)]),
            ]),
          ),
        ),
      ]),
      Sect(title: 'As a child', children: [
        PropRow(id: 'lay.ignore', label: 'Ignore layout', keyable: false, cells: [Align(alignment: Alignment.centerLeft, child: OnOff(on: doc.b['lay.ignore'] ?? false, onChanged: (v) => doc.flag('lay.ignore', v)))]),
      ]),
    ]);
  }
}

// ---- 12. relations ----------------------------------------------------------------------------------------------------
/// The relation card and rows come from the lab's relations part (RelationCard); only the section around them is here.
class RelationsBlock extends StatefulWidget {
  const RelationsBlock({super.key, this.titled = true, this.first = true});
  final bool titled, first;
  @override
  State<RelationsBlock> createState() => _RelationsBlockState();
}

class _RelationsBlockState extends State<RelationsBlock> {
  final on = <String, bool>{'Scatter': true, 'Stagger': true, 'Along Path': true, 'Face': false};
  String open = 'Scatter';
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), ed = Ed(x.cfg.ed), fams = [Fam.scatter, Fam.stagger, Fam.along, Fam.face], locked = x.cfg.locked;
    // contract: one relation open (its three values and a small plot), the others one hairline row each; the switch bypasses; clicking a row opens it (I4).
    // Same field language as everything else: flat fills, hairlines, no card, no gradient. Hue only on the relation's own mark and plot.
    final rows = <Widget>[
      for (final f in fams) ...[
        Hov(
          onTap: () => setState(() => open = f.name),
          builder: (_, h) => Container(
            height: 30,
            decoration: BoxDecoration(border: Border(bottom: BorderSide(color: ed.rule)), color: h && !locked ? N.g13 : null),
            child: Row(children: [
              Container(width: 6, height: 6, decoration: BoxDecoration(color: on[f.name]! ? f.c : N.g38, shape: BoxShape.circle)),
              const SizedBox(width: 10),
              Expanded(child: Text(f.name, style: T.name(on[f.name]! && !locked ? N.g95 : N.g56))),
              if (f.name == open) Padding(padding: const EdgeInsets.only(right: 12), child: Text('Open', style: T.label(N.g56))),
              OnOff(on: on[f.name]!, onChanged: (v) => setState(() => on[f.name] = v)),
            ]),
          ),
        ),
        if (f.name == open)
          Padding(
            padding: const EdgeInsets.only(top: 4, bottom: 6),
            child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
              const PropRow(id: 'rel.d', label: 'Density', keyable: false, cells: [NumField(id: 'rel.d', unit: '%', min: 0, max: 100, perPx: .5)]),
              const PropRow(id: 'rel.s', label: 'Spread', keyable: false, cells: [NumField(id: 'rel.s', unit: '%', min: 0, max: 100, perPx: .5)]),
              const PropRow(id: 'rel.f', label: 'Falloff', keyable: false, cells: [NumField(id: 'rel.f', unit: '%', min: 0, max: 100, perPx: .5)]),
              Padding(padding: const EdgeInsets.only(top: 6), child: Opacity(opacity: on[f.name]! && !locked ? 1 : .5, child: Align(alignment: Alignment.centerLeft, child: Gadget(f, size: 72)))),
            ]),
          ),
      ],
      Hov(builder: (_, h) => Container(height: 28, alignment: Alignment.centerLeft, child: Text('Add relation', style: T.name(h && !locked ? N.g95 : N.g63)))),
    ];
    return Sect(title: widget.titled ? 'Relations' : null, first: widget.first, children: [
      IgnorePointer(ignoring: locked, child: Opacity(opacity: locked ? .6 : 1, child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: rows))),
    ]);
  }
}

// ---- 13. the whole panel ----------------------------------------------------------------------------------------------
/// The Inspector. Needs an InspHost above it. Tabs or one scrolling column is the Layout knob; P / S / R / T / A reveal their row (B7, I2).
class InspectorPanel extends StatefulWidget {
  const InspectorPanel({super.key, this.width = 372, this.height = 720});
  final double width, height;
  @override
  State<InspectorPanel> createState() => _InspectorPanelState();
}

class _InspectorPanelState extends State<InspectorPanel> {
  int _tab = 0;
  final _node = FocusNode(debugLabel: 'inspector');
  final _scroll = ScrollController();

  @override
  void dispose() {
    _node.dispose();
    _scroll.dispose();
    super.dispose();
  }

  List<String> _tabs(Kind k) => k == Kind.camera ? const ['Camera', 'Relations', 'Effects'] : (k == Kind.group ? const ['Transform', 'Relations', 'Effects'] : const ['Transform', 'Relations', 'Effects', 'Material']);

  KeyEventResult _key(FocusNode n, KeyEvent e) {
    if (e is! KeyDownEvent) return KeyEventResult.ignored;
    final doc = Ctx.read(context).doc;
    if (doc.kind == Kind.camera || doc.kind == Kind.none) return KeyEventResult.ignored;
    final k = HardwareKeyboard.instance;
    if (k.isShiftPressed || k.isControlPressed || k.isAltPressed || k.isMetaPressed) return KeyEventResult.ignored;
    final f = FocusManager.instance.primaryFocus;
    if (f?.context?.findAncestorWidgetOfExactType<EditableText>() != null) return KeyEventResult.ignored;
    final id = {LogicalKeyboardKey.keyP: 'pos', LogicalKeyboardKey.keyS: 'scale', LogicalKeyboardKey.keyR: 'rot', LogicalKeyboardKey.keyT: 'op', LogicalKeyboardKey.keyA: 'anchor'}[e.logicalKey];
    if (id == null) return KeyEventResult.ignored;
    setState(() => _tab = 0);
    doc.reveal(id);
    return KeyEventResult.handled;
  }

  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), cfg = x.cfg, doc = x.doc, ed = Ed(cfg.ed), k = doc.kind;
    final tabs = _tabs(k), tab = tabs[_tab.clamp(0, tabs.length - 1)];
    final titled = !cfg.tabs;
    List<Widget> section(String t, {required bool first}) {
      switch (t) {
        case 'Transform':
          return [TransformGroup(titled: titled, first: first), if (k == Kind.group && cfg.maybe) const LayoutGroup()];
        case 'Camera':
          return [CameraGroup(first: first)];
        case 'Relations':
          return [RelationsBlock(titled: titled, first: first)];
        case 'Effects':
          return [if (titled) Sect(title: 'Effects', first: first, children: const []), const EffectsStack()];
        default:
          return [
            if (k == Kind.text) TextGroup(first: first),
            if (k == Kind.shape || k == Kind.text) FillStroke(first: first),
            BlendRows(titled: true, first: first && k != Kind.text && k != Kind.shape),
          ];
      }
    }

    final body = <Widget>[
      if (k == Kind.none) const EmptyState() else ...[
        const InspHeader(),
        if (cfg.locked) Padding(padding: const EdgeInsets.only(bottom: 4), child: Text('Locked. Values are readable; unlock the layer to edit.', style: T.label(N.g56))),
        if (cfg.tabs) ...[
          const SizedBox(height: 4),
          Segmented(items: tabs, index: _tab.clamp(0, tabs.length - 1), expand: true, onChanged: (i) => setState(() => _tab = i)),
          SizedBox(height: ed.gap - 4),
          ...section(tab, first: true),
        ] else ...[
          const SizedBox(height: 4),
          for (var i = 0; i < tabs.length; i++) ...section(tabs[i], first: i == 0),
        ],
      ],
    ];
    return Focus(
      focusNode: _node,
      onKeyEvent: _key,
      child: Listener(
        onPointerDown: (_) => WidgetsBinding.instance.addPostFrameCallback((_) {
          if (mounted && !_node.hasFocus) _node.requestFocus();
        }),
        child: Container(
          width: widget.width,
          height: widget.height,
          color: N.g10,
          child: SingleChildScrollView(controller: _scroll, padding: const EdgeInsets.all(12), child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, mainAxisSize: MainAxisSize.min, children: body)),
        ),
      ),
    );
  }
}
