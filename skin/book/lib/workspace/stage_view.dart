// The stage's viewport: the composition centred on a dark surround, rulers and guides, overlays, the selection, the zoom HUD and the navigator.
// contract: paints only when the scene, the selection, the hover or the view changes; the pointer readout repaints the rulers alone.
import 'dart:math' as math;

import 'package:flutter/foundation.dart' show listEquals;
import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart' show HardwareKeyboard;
import 'package:flutter/widgets.dart';

import '../tokens.dart';
import 'stage_art.dart';
import '../motolii.dart' show Motolii;
import '../sets/panels/timeline_parts_parts.dart' show TpGlyph, TpGlyphView;
import 'relate.dart';
import 'ws.dart';

/// Zoom steps of the header chip: fit, then fixed scales.
const stageZooms = ['Fit', '50%', '100%'];
const _fixed = [0.0, .5, 1.0];

/// Viewport sizes: the ruler band, the margin a fit leaves around the composition, the HUD pill and the navigator.
abstract final class _V {
  static double get ruler => Shape.z(16.0);
  static double get fitMargin => Shape.s(28.0);
  static double get hud => Shape.z(22.0);
  static double get mapW => Shape.z(136.0);
  static double get pad => Shape.s(8.0);
  static double get handle => Shape.z(7.0);
  static double get stem => Shape.z(18.0);
  static double get tagH => Shape.z(16.0);
  static Color get surround => Grey.g10;
  static Color get rulerGround => Grey.g13;
  static Color get rulerSpan => Grey.g15;
  static Color get tick => Grey.g38;
  static Color get guide => WsT.toneAudio;
  static const overlay = Color(0x40FFFFFF), safe = Color(0x99F6EFDD);
}


/// Composition px to screen px: the composition's top-left at [o], [s] screen px per composition px, inside [area].
@immutable
class _View {
  const _View(this.o, this.s, this.area);
  final Offset o;
  final double s;
  final Rect area;
  Rect get comp => o & WsArt.comp * s;
  Offset toScreen(Offset p) => o + p * s;
  Offset toComp(Offset p) => (p - o) / s;
  @override
  bool operator ==(Object x) => x is _View && x.o == o && x.s == s && x.area == area;
  @override
  int get hashCode => Object.hash(o, s, area);
}

class StageView extends StatefulWidget {
  const StageView({
    super.key,
    required this.scene,
    required this.selected,
    required this.zoom,
    required this.grid,
    required this.safe,
    required this.cursor,
    this.onPoke,
    this.texture,
    this.make,
    this.axis = 0,
    this.fromCentre = false,
    this.snap = false,
    this.pivots = false,
    this.group = false,
    this.mask = false,
  });

  /// Joined: the Move tool's way (0 free, 1 along x only, 2 along y only), and whether an ellipse is drawn out from its middle.
  final int axis;
  final bool fromCentre;

  /// Joined: a thing carried sticks to the lines of the others, of the picture and of the guides (snap); every thing's
  /// point it turns about is shown (pivots); a thing taken hold of is the one it goes with, outermost (group); what is
  /// drawn cuts the picked thing (mask: it lies just behind it, and that one is clipped by it).
  final bool snap, pivots, group, mask;

  /// Joined: what the tool in hand makes where the hand is drawn across the stage (rect, circle, line, text; null:
  /// the hand takes hold of things).
  final String? make;

  /// The inside's picture of the stage (a Flutter texture). null: the stand-in art is painted.
  final int? texture;
  final ArtScene scene;
  final String? selected;
  final int zoom;
  final bool grid, safe;

  /// The pointer in composition px while it is over the viewport; the HUD and the rulers read it.
  final ValueNotifier<Offset?> cursor;
  final VoidCallback? onPoke;
  @override
  State<StageView> createState() => _StageViewState();
}

/// The picked thing's handles where the stage draws them now, by name, in the stage's own coordinates: what a hand
/// (or a test, as tldraw's and Excalidraw's do) presses to take hold of each.
abstract interface class StageHandles {
  Map<String, Offset> get handles;
}

/// The selection's handles where they are drawn, by name: its four corners, the knob it turns by, the point it turns
/// about and its middle (the body). The painter draws them from here; [StageHandles] hands them out from here.
Map<String, Offset> _handles(Offset Function(Offset doc) scr, ArtBox b, {required bool knob}) {
  final top = scr(b.top), dir = top - scr(b.c), n = dir.distance < 1 ? const Offset(0, -1) : dir / dir.distance;
  return {
    for (final (i, p) in b.corners.indexed) 'corner $i': scr(p),
    if (knob) 'turn': top + n * _V.stem,
    'anchor': scr(b.anchor),
    'body': scr(b.c),
  };
}

class _StageViewState extends State<StageView> implements StageHandles {
  Offset _pan = Offset.zero;
  String? _hover;
  _View? _shown; // (the view as last laid out: where the handles are now)

  @override
  Map<String, Offset> get handles {
    final v = _shown, scene = widget.scene, b = WsArt.boxOf(scene, widget.selected), l = scene.layer(widget.selected);
    if (v == null || b == null || l == null) return const {};
    return _handles((d) => v.toScreen(WsArt.toView(scene, d)), b, knob: l.kind != WsKind.camera && l.id != 'bg');
  }

  @override
  void didUpdateWidget(StageView o) {
    super.didUpdateWidget(o);
    if (o.zoom != widget.zoom) _pan = Offset.zero;
    if (o.make != widget.make) _penning = false; // (another tool: the pen's next point begins a new shape)
  }

  // Joined: the inside draws the whole pane (the desk), the stage lying in it where this view puts it; told again when
  // that changes (after the frame: the inside is not asked while building).
  int? _desk;
  _View? _toldDesk;
  Motolii? _deskOf;
  double? _toldDpr;
  void _deskTo(_View v) {
    final dpr = MediaQuery.devicePixelRatioOf(context);
    if (widget.texture == null || (v == _toldDesk && dpr == _toldDpr)) return; // (told again when the window's zoom changes its pixels too)
    _toldDesk = v;
    _toldDpr = dpr;
    final m = WsScope.read(context).inside;
    if ((_deskOf = m) == null) return;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted || _toldDesk != v || _toldDpr != dpr) return;
      final m = _deskOf!;
      if (!m.desk((v.area.width * dpr).round(), (v.area.height * dpr).round(), (v.o.dx - v.area.left) * dpr, (v.o.dy - v.area.top) * dpr, v.s * dpr, dpr)) return;
      _toldPick = null;
      if (_desk == null) m.deskTexture().then((id) => mounted && id >= 0 ? setState(() => _desk = id) : null);
    });
  }

  // Joined, the desk on: which thing is picked and which is under the hand, told when it changes (the inside marks them).
  (int, int)? _toldPick;
  void _pickTo() {
    final m = _deskOf;
    if (_desk == null || m == null) return;
    final now = (int.tryParse(widget.selected ?? '') ?? 0, int.tryParse(_hover ?? '') ?? 0);
    if (now == _toldPick) return;
    _toldPick = now;
    m.pick(now.$1, now.$2);
  }

  @override
  void dispose() {
    _deskOf?.desk(0, 0, 0, 0, 0, 0); // (the stage's own picture again)
    super.dispose();
  }

  _View _view(Size size) {
    final area = Rect.fromLTRB(_V.ruler, _V.ruler, size.width, size.height);
    final fit = math.max(.02, math.min((area.width - _V.fitMargin * 2) / WsArt.comp.width, (area.height - _V.fitMargin * 2) / WsArt.comp.height));
    final s = widget.zoom == 0 ? fit : _fixed[widget.zoom];
    final half = Offset(WsArt.comp.width * s, WsArt.comp.height * s) / 2;
    return _View(area.center - half + _pan, s, area);
  }

  Offset _doc(_View v, Offset local) => WsArt.toDoc(widget.scene, v.toComp(local));

  String? _hit(_View v, Offset local) {
    // (a thing that lies off the picture can still be held by its frame and carried back)
    final d = _doc(v, local), id = WsArt.hit(widget.scene, d), m = WsScope.read(context).inside;
    if (widget.group && m != null && id != null) { // (Group: the outermost of the things it goes with, by the document's parent)
      var t = int.parse(id);
      for (var p = (m.value(t, 'parent', 0) ?? 0).round(), n = 0; p > 0 && n < 64; p = (m.value(t, 'parent', 0) ?? 0).round(), n++) {
        t = p;
      }
      return '$t';
    }
    return id;
  }

  // Joined: a thing held on the stage, carried, and let go. The stage only says how far the hand has gone, in points of
  // the picture; where that puts the thing (at its own depth), and what it is to the document, is the inside's.
  ({Offset from, bool moved, String? by})? _held;   // (by: a handle's way of carrying it, scale, turn or pivot; null: placed)

  // Joined, Snap (the past repository's, After Effects' snapping): the held thing's left, middle and right (top, middle,
  // bottom) stick to the nearest of the same three lines of every other thing it does not go with and of the picture,
  // or to a guide, within six points of the screen. What it holds to, and the lines it met.
  ({Rect box, List<(double, double)> xs, List<(double, double)> ys})? _snapTo;
  (double?, double?) _met = (null, null);

  Rect _bounds(ArtBox b) {
    final p = [for (final c in b.corners) WsArt.toView(widget.scene, c)];
    return p.skip(1).fold(Rect.fromPoints(p[0], p[0]), (r, q) => r.expandToInclude(Rect.fromPoints(q, q)));
  }

  void _snapFrom(String id) {
    final boxes = widget.scene.boxes, m = WsScope.read(context).inside;
    final held = boxes[id];
    if (!widget.snap || held == null || m == null) return _snapTo = null;
    int top(int t) {
      for (var p = (m.value(t, 'parent', 0) ?? 0).round(), n = 0; p > 0 && n < 64; p = (m.value(t, 'parent', 0) ?? 0).round(), n++) {
        t = p;
      }
      return t;
    }
    final mine = top(int.parse(id)), c = WsArt.comp;
    final xs = <(double, double)>[(0, c.width)], ys = <(double, double)>[(0, c.height)];
    for (final g in widget.scene.guidesX) {
      xs.add((g, g));
    }
    for (final g in widget.scene.guidesY) {
      ys.add((g, g));
    }
    for (final MapEntry(:key, :value) in boxes.entries) {
      if (top(int.parse(key)) == mine) continue;
      final r = _bounds(value);
      xs.add((r.left, r.right));
      ys.add((r.top, r.bottom));
    }
    _snapTo = (box: _bounds(held), xs: xs, ys: ys);
  }

  Offset _snapped(Offset by, double s) {
    final to = _snapTo;
    if (to == null) return by;
    (double, double?) one(double lo, double size, double d, List<(double, double)> lines) {
      (double, double)? best;
      for (final (a, b) in lines) {
        for (final line in [a, (a + b) / 2, b]) {
          for (final mine in [lo + d, lo + d + size / 2, lo + d + size]) {
            final diff = line - mine;
            if (diff.abs() <= 6 / s && (best == null || diff.abs() < best.$1.abs())) best = (diff, line);
          }
        }
      }
      return best == null ? (d, null) : (d + best.$1, best.$2);
    }
    final (x, mx) = widget.axis == 2 ? (by.dx, null) : one(to.box.left, to.box.width, by.dx, to.xs);
    final (y, my) = widget.axis == 1 ? (by.dy, null) : one(to.box.top, to.box.height, by.dy, to.ys);
    _met = (mx, my);
    return Offset(x, y);
  }

  // Joined: a guide pulled out of a ruler (from the top one, a level line; from the left one, an upright one) or taken
  // hold of where it lies; let go on the picture it is the document's there, let go back over its ruler it is gone.
  ({String axis, double? was})? _pulling;

  bool _pull(_View v, Offset local) {
    final ws = WsScope.read(context), m = ws.inside;
    if (m == null) return false;
    final at = v.toComp(local), tol = 4 / v.s;
    if (local.dy < _V.ruler || local.dx < _V.ruler) {
      _pulling = (axis: local.dy < _V.ruler ? 'y' : 'x', was: null);
      return true;
    }
    for (final (axis, g) in m.guides) {
      if (((axis == 'x' ? at.dx : at.dy) - g).abs() > tol) continue;
      _pulling = (axis: axis, was: g);
      return true;
    }
    return false;
  }

  void _pullLetGo(_View v, Offset local) {
    final p = _pulling, ws = WsScope.read(context), m = ws.inside;
    _pulling = null;
    if (p == null || m == null) return;
    final over = p.axis == 'y' ? local.dy < _V.ruler : local.dx < _V.ruler, at = v.toComp(local);
    m.later('together', number: 1);   // (one step, without waiting)
    if (p.was != null) m.later('guide', name: p.axis, number: p.was!, word: 'off');
    if (!over) m.later('guide', name: p.axis, number: ((p.axis == 'x' ? at.dx : at.dy)).roundToDouble());
    ws.after(m.later('together'));
  }

  Offset? _last;

  // Joined: a thing drawn with the tool in hand: from where the hand came down to where it is let go (a rect or a
  // circle that box, a line from the one point to the other, a text where it came down); only come down, one as it
  // comes. One step; it is picked.
  Offset? _drawFrom;

  // Joined: drawn round on the stage where no thing is (no tool in hand): the things whose middles it goes round are
  // picked together (Ws.many), and what they may do together is offered beside them (relate.dart).
  List<Offset>? _lasso;

  void _lassoLetGo(_View v) {
    final round = _lasso, ws = WsScope.read(context);
    setState(() => _lasso = null);
    if (round == null || round.length < 3) return;
    // (drawn straight, it is a box from where the hand came down to where it let go (Figma's, AE's); else the line it drew round)
    final a = round.first, z = round.last, along = z - a;
    final straight = along.distance > 4 && round.every((p) => ((p - a).dx * along.dy - (p - a).dy * along.dx).abs() / along.distance < 12);
    final path = straight ? (Path()..addRect(Rect.fromPoints(a, z))) : (Path()..addPolygon(round, true));
    ws.pickMany([for (final l in widget.scene.layers) if (widget.scene.box(l.id) case final b? when path.contains(v.toScreen(WsArt.toView(widget.scene, b.c)))) l.id]);
  }

  void _draw(_View v, Offset to) {
    final from = _drawFrom, ws = WsScope.read(context), m = ws.inside, kind = widget.make;
    _drawFrom = null;
    setState(() {});
    if (from == null || m == null || kind == null) return;
    final a = v.toComp(from), b = v.toComp(to), p = m.point, c = WsArt.comp;
    final centred = kind == 'circle' && widget.fromCentre; // (drawn out from its middle: twice as large about where the hand came down)
    final mid = centred ? a : (a + b) / 2, d = centred ? (b - a) * 2 : b - a, tiny = d.distance < 6 / v.s;
    final x = ((kind == 'text' || tiny ? a.dx : mid.dx) - c.width / 2) / p, y = (c.height / 2 - (kind == 'text' || tiny ? a.dy : mid.dy)) / p;
    final cut = widget.mask ? int.tryParse(ws.selected ?? '') : null;
    // (one step, asked without waiting: the new thing is the one the add made (thing -1), picked once it is there)
    final before = m.things.length;
    m.later('together', number: 1);
    var made = 0;
    ws.after(m.later('add', name: kind == 'text' ? '' : 'shape', word: kind == 'text' ? wsTextFont : kind), null, (t) => made = t);   // (a text in the font the Create shelf's Text has)
    m
      ..later('set', thing: -1, name: 'x', number: x)
      ..later('set', thing: -1, name: 'y', number: y);
    if (ws.ink.isNotEmpty) m.later('say', thing: -1, name: 'fill', word: ws.ink); // (the drawing inks)
    if (ws.edge.isNotEmpty && kind != 'line') {
      m
        ..later('say', thing: -1, name: 'stroke', word: ws.edge)
        ..later('set', thing: -1, name: 'stroke_width', number: 6);
    }
    if (cut != null) { // (Mask: it lies just behind the picked thing, and that one is clipped by it; not seen itself: it only cuts, as After Effects' matte)
      final all = m.things;
      if (all.contains(cut)) {
        m
          ..later('raise', thing: -1, number: (all.indexOf(cut) - before).toDouble())
          ..later('set', thing: cut, name: 'clip', number: 1)
          ..later('set', thing: -1, name: 'opacity', number: 0);
      }
    }
    if (!tiny && kind != 'text') {
      if (kind == 'line') {   // (a line is a twenty-fifth of a unit thick: as long as drawn, turned as drawn)
        m
          ..later('set', thing: -1, name: 'wide', number: d.distance / p / .04)
          ..later('set', thing: -1, name: 'rotate', number: math.atan2(d.dy, d.dx) * 180 / math.pi);
      } else {
        m
          ..later('set', thing: -1, name: 'size', number: d.dy.abs() / p)
          ..later('set', thing: -1, name: 'wide', number: d.dx.abs() / math.max(d.dy.abs(), 1e-3));
      }
    }
    ws.after(m.later('together'), (done, _) => done && made > 0 ? ws.select('$made') : null);
  }

  // Joined: the pen: a point where the hand came down, its handle drawn out as far as the hand was dragged (a tap: a
  // corner). Its first goes to the picked thing, the rest to the one the pen made (thing -1); the inside reckons all of
  // it (a new path begun, closed on its first point, a point put on or taken off a closed one) from where the hand is.
  bool _penning = false;

  void _pen(_View v, Offset down, Offset up) {
    final ws = WsScope.read(context), m = ws.inside;
    _drawFrom = null;
    setState(() {});
    if (m == null) return;
    final a = _doc(v, down), h = _doc(v, up) - a;
    final first = !_penning;
    _penning = true;
    if (first) m.later('together', number: 1);
    var made = 0;
    ws.after(m.later('pen', thing: first ? int.tryParse(widget.selected ?? '') ?? 0 : -1, word: '${a.dx} ${a.dy} ${h.dx} ${h.dy}'), null, (t) => made = t); // (first on the picked thing: a closed path gains or loses a point, anything else begins a new one)
    if (first) {
      if (ws.ink.isNotEmpty) m.later('say', thing: -1, name: 'fill', word: ws.ink);
      if (ws.edge.isNotEmpty) {
        m
          ..later('say', thing: -1, name: 'stroke', word: ws.edge)
          ..later('set', thing: -1, name: 'stroke_width', number: 6);
      }
      ws.after(m.later('together'), (done, _) => done && made > 0 ? ws.select('$made') : null);
    }
  }

  void _hold(_View v, Offset local) {
    if (widget.make != null) {
      setState(() => _drawFrom = local);
      return;
    }
    final ws = WsScope.read(context), m = ws.inside;
    // (joined: a hand on one of the picked thing's handles carries it the handle's way: the inside reckons it from where it took hold)
    final picked = int.tryParse(widget.selected ?? '');
    if (m != null && picked != null) {
      String? by;
      var near = 8.0;
      for (final MapEntry(:key, :value) in handles.entries) {
        final d = (value - local).distance;
        if (d > near) continue;
        near = d;
        by = switch (key.split(' ').first) { 'corner' => 'scale', 'turn' => 'turn', 'anchor' => 'pivot', _ => null };
      }
      for (final p in [...m.points(picked), for (final (h, _) in m.handles(picked)) h]) { // (a path's own point, moved with its handles; or a handle alone: the inside takes the nearest)
        final d = (v.toScreen(WsArt.toView(widget.scene, p)) - local).distance;
        if (d > near) continue;
        near = d;
        by = 'point';
      }
      if (by != null) {
        m.take(picked);
        final at = _doc(v, local); // (points of the picture, as the inside counts them)
        m.carry('grip', at.dx, at.dy);
        if (ws.playing) ws.playing = false;
        _held = (from: local, moved: false, by: by);
        return;
      }
    }
    if (_pull(v, local)) return; // (a guide out of a ruler, or one taken hold of: after the handles, which lie over everything)
    final id = m == null ? null : _hit(v, local), t = int.tryParse(id ?? '');
    if (m != null && t == null) { // (where no thing is: drawn round)
      ws.select(null);
      setState(() => _lasso = [local]);
      return;
    }
    if (m == null || t == null) return;
    m.take(t);
    ws.select(id);
    if (ws.playing) ws.playing = false; // (a hand on a thing stops the play)
    _held = (from: local, moved: false, by: null);
    _snapFrom(id!);
  }

  void _carry(_View v, Offset local) {
    final h = _held, ws = WsScope.read(context), m = ws.inside;
    if (h == null || m == null) return;
    final all = (local - h.from) / v.s; // (screen px -> points of the picture)
    if (h.by case final way?) {
      final gone = _doc(v, local) - _doc(v, h.from);
      m.carry(way, gone.dx, gone.dy);
      _held = (from: h.from, moved: true, by: way);
      setState(() {});
      ws.shown();
      return;
    }
    final by = _snapped(switch (widget.axis) { 1 => Offset(all.dx, 0), 2 => Offset(0, all.dy), _ => all }, v.s); // (the Move tool's way)
    m.carry('place', by.dx, by.dy);
    _held = (from: h.from, moved: true, by: null);
    setState(() {});
    ws.shown();
  }

  void _letGo({bool keep = true}) {
    final h = _held, ws = WsScope.read(context), m = ws.inside;
    _held = null;
    _snapTo = null;
    if (_met != (null, null)) setState(() => _met = (null, null));
    if (h == null || m == null) return;
    ws.after(m.later('let_go', number: keep && h.moved ? 1 : 0));   // (let go without waiting: the document read again once the inside has it)
  }

  void _scroll(PointerSignalEvent e, _View v) {
    if (e is! PointerScrollEvent || widget.zoom == 0) return;
    final lim = Offset(WsArt.comp.width * v.s / 2 + v.area.width / 2 - 60, WsArt.comp.height * v.s / 2 + v.area.height / 2 - 60);
    final p = _pan - e.scrollDelta;
    setState(() => _pan = Offset(p.dx.clamp(-lim.dx, lim.dx), p.dy.clamp(-lim.dy, lim.dy)));
  }

  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, box) {
      final v = _shown = _view(box.biggest), scene = widget.scene;
      _deskTo(v);
      _pickTo();
      return Listener(
        onPointerSignal: (e) => _scroll(e, v),
        child: MouseRegion(
          onHover: (e) {
            widget.cursor.value = v.toComp(e.localPosition);
            final h = _hit(v, e.localPosition);
            if (h != _hover) setState(() => _hover = h);
          },
          onExit: (_) {
            widget.cursor.value = null;
            if (_hover != null) setState(() => _hover = null);
          },
          child: GestureDetector(
            behavior: HitTestBehavior.opaque,
            onTapUp: (d) {
              widget.onPoke?.call();
              if (widget.make == 'pen') return _pen(v, d.localPosition, d.localPosition); // (a tap: a corner)
              if (widget.make != null) return _draw(v, _drawFrom = d.localPosition); // (only come down: one as it comes, there)
              final hit = _hit(v, d.localPosition), ws = WsScope.read(context);
              if (HardwareKeyboard.instance.isShiftPressed || HardwareKeyboard.instance.isMetaPressed) return hit == null ? null : wsPickAlso(ws, hit); // (Shift or Cmd: one more picked together, or one less; on nothing, nothing lost)
              ws.select(hit);
            },
            dragStartBehavior: DragStartBehavior.down, // (a thing is held where the hand came down, not where it had got to when the drag was known for one)
            onPanStart: (d) => _hold(v, _last = d.localPosition),
            onPanUpdate: (d) => _lasso != null
                ? setState(() => _lasso!.add(d.localPosition))
                : _drawFrom != null
                ? setState(() => _last = d.localPosition)
                : _pulling != null
                ? (widget.cursor.value = v.toComp(_last = d.localPosition))
                : _carry(v, _last = d.localPosition),
            onPanEnd: (_) => _lasso != null
                ? _lassoLetGo(v)
                : _drawFrom != null
                ? (widget.make == 'pen' ? _pen(v, _drawFrom!, _last ?? _drawFrom!) : _draw(v, _last ?? _drawFrom!))
                : _pulling != null
                ? _pullLetGo(v, _last!)
                : _letGo(),
            onPanCancel: () => _lasso != null ? setState(() => _lasso = null) : _drawFrom != null ? setState(() => _drawFrom = null) : _pulling != null ? _pulling = null : _letGo(keep: false),
            child: Stack(
              fit: StackFit.expand,
              children: [
                RepaintBoundary(
                  child: CustomPaint(
                    painter: _ViewPainter(scene, v, widget.selected, _hover == widget.selected ? null : _hover, grid: widget.grid, safe: widget.safe, part: widget.texture == null ? _Part.all : _Part.under, pivots: widget.pivots, met: _met),
                  ),
                ),
                if (_desk case final desk? when widget.texture != null) // (the whole pane as the inside draws it: off the stage covered half way, the picked thing marked)
                  Positioned.fromRect(rect: v.area, child: IgnorePointer(child: Texture(textureId: desk, filterQuality: FilterQuality.medium)))
                else
                Positioned.fromRect(
                  rect: v.comp,
                  child: IgnorePointer(
                    child: widget.texture == null
                        ? const SizedBox(key: ValueKey('ws-stage-comp'))
                        : ClipRect(key: const ValueKey('ws-stage-comp'), child: Texture(textureId: widget.texture!, filterQuality: FilterQuality.medium)),
                  ),
                ),
                if (int.tryParse(widget.selected ?? '') case final picked? when WsScope.read(context).inside != null) // (the picked path's own points, to be taken hold of)
                  IgnorePointer(child: CustomPaint(size: Size.infinite, painter: _PointsPainter([for (final p in WsScope.read(context).inside!.points(picked)) v.toScreen(WsArt.toView(widget.scene, p))],
                      [for (final (h, o) in WsScope.read(context).inside!.handles(picked)) (v.toScreen(WsArt.toView(widget.scene, h)), v.toScreen(WsArt.toView(widget.scene, o)))]))),
                if (_drawFrom case final from? when _last != null && widget.make == 'pen') // (the pen's handle being drawn out: through its point, both ways)
                  IgnorePointer(child: CustomPaint(size: Size.infinite, painter: _HandlePainter(from, _last!)))
                else if (_drawFrom case final from? when _last != null) // (what is being drawn: its box)
                  Positioned.fromRect(
                    rect: Rect.fromPoints(from, _last!),
                    child: IgnorePointer(child: DecoratedBox(decoration: BoxDecoration(border: Border.all(color: WsT.accent)))),
                  ),
                if (widget.texture != null)
                  IgnorePointer(
                    child: CustomPaint(
                      painter: _ViewPainter(scene, v, widget.selected, _hover == widget.selected ? null : _hover, grid: widget.grid, safe: widget.safe, part: _Part.over, pivots: widget.pivots, met: _met, marked: _desk != null),
                    ),
                  ),
                IgnorePointer(child: CustomPaint(painter: _RulerPainter(v, widget.cursor, scene))),
                _RelationMarks(scene: scene, v: v, lasso: _lasso), // (drawn round, the things picked together, the relation picked: its shape, its handles, what it moves)
                Positioned(
                  left: _V.ruler + _V.pad,
                  bottom: _V.pad,
                  child: _Hud(scale: v.s, cursor: widget.cursor),
                ),
                if (widget.zoom != 0 && box.maxWidth > 360 && box.maxHeight > 220)
                  Positioned(
                    right: _V.pad,
                    bottom: _V.pad,
                    child: _Navigator(scene: scene, view: v, texture: widget.texture),
                  ),
              ],
            ),
          ),
        ),
      );
    },
  );
}

/// Joined: what is picked together and the relation picked, over the stage. Picked together: a ring each, and along the
/// top of the stage what they may do together. A relation picked: its shape (a ring round its middle, a row's line, how
/// far a push reaches), handles that carry its middle and size, a ring on each thing it moves with a small × that takes
/// that one out, a crown on its leader (another's crown makes that one the leader), and along the top its name and
/// Release. A thing picked: the relations it is in, beside it, as outlined tags.
class _RelationMarks extends StatelessWidget {
  const _RelationMarks({required this.scene, required this.v, required this.lasso});
  final ArtScene scene;
  final _View v;
  final List<Offset>? lasso;

  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: WsScope.read(context),
    builder: (context, _) {
      final ws = WsScope.read(context), m = ws.inside;
      Offset? on(String id) => scene.box(id) == null ? null : v.toScreen(WsArt.toView(scene, scene.box(id)!.c));
      final picked = [for (final id in ws.many) if (on(id) case final p?) p];
      final rel = int.tryParse(ws.selected ?? ''), kind = rel == null || m == null ? '' : m.word(rel, 'file');
      final isRelation = rel != null && m != null && m.word(rel, 'kind') == 'behaviour' && wsRelations.contains(kind);
      final members = [for (final id in wsMembersOf(ws)) if (on(id) case final p?) (id, p)];
      final now = ws.frame / Ws.fps;
      // (a place of the stage in its own units, on the screen; and back)
      Offset screen(double x, double y) => v.toScreen(WsArt.toView(scene, Offset(WsArt.comp.width / 2 + x * m!.point, WsArt.comp.height / 2 - y * m.point)));
      (double, double) units(Offset p) {
        final c = v.toComp(p);
        return ((c.dx - WsArt.comp.width / 2) / m!.point, (WsArt.comp.height / 2 - c.dy) / m.point);
      }

      final shape = <_Shape>[];
      final handles = <Widget>[];
      String? leader;
      if (isRelation) {
        double val(String n, [double or = 0]) => m.value(rel, n, now) ?? or;
        final off = val('visible', 1) <= .5;
        leader = val('leader') > 0 ? '${val('leader').round()}' : null;
        final cx = val('centre_x'), cy = val('centre_y'), mid = screen(cx, cy), px = m.point * v.s;
        Widget handle(String key, Offset at, void Function((double, double) u) carry) => Positioned(   // (the middle a cross, a size an arrow out)
          left: at.dx - 7,
          top: at.dy - 7,
          child: GestureDetector(
            key: ValueKey('ws-relation-$key'),
            onPanStart: (_) => m.take(rel),
            onPanUpdate: (d) {
              final box = context.findRenderObject() as RenderBox?;
              if (box != null) carry(units(box.globalToLocal(d.globalPosition)));
            },
            onPanEnd: (_) => ws.after(m.later('let_go', number: 1), (_, _) => ws.reread()),
            child: Container(width: 16, height: 16, decoration: BoxDecoration(color: WsT.onAccent, shape: BoxShape.circle, border: Border.all(color: WsT.accent, width: 2)), child: CustomPaint(painter: _GlyphPaint(key == 'centre' ? 'cross' : 'arrow', WsT.accent))),
          ),
        );
        if (kind == 'around') {
          final r = val('radius', .5), turn = val('offset') + .5 / math.max(1, members.length);
          shape.add(_Shape.ring(mid, r * px, off: off));
          handles
            ..add(handle('centre', mid, (u) => m..carry('centre_x', u.$1)..carry('centre_y', u.$2)))
            ..add(handle('size', screen(cx + r * math.sin(turn * 2 * math.pi), cy + r * math.cos(turn * 2 * math.pi)), (u) => m.carry('radius', math.sqrt(math.pow(u.$1 - cx, 2) + math.pow(u.$2 - cy, 2)))));
        } else if (kind == 'row') {
          final gap = val('gap', .2), n = math.max(1, m.word(rel, 'members').split(' ').length), half = gap * (n - 1) / 2;
          shape.add(_Shape.line(screen(cx - half, cy), screen(cx + half, cy), off: off));   // (a row about its middle)
          const above = Offset(0, -30);   // (its handles over the line, not on the things in it)
          handles
            ..add(handle('centre', mid + above, (u) {
              final (_, lift) = units(above);
              final (_, zero) = units(Offset.zero);
              m..carry('centre_x', u.$1)..carry('centre_y', u.$2 - (lift - zero));
            }))
            ..add(handle('size', screen(cx + half, cy) + above, (u) => m.carry('gap', math.max(0, u.$1 - cx) * 2 / math.max(1, n - 1))));   // (its right end out: wider both ways)
        } else if (kind == 'push' && leader != null && on(leader) != null) {
          final l = on(leader)!, r = val('radius', .5);
          shape.add(_Shape.ring(l, r * px, off: off));
          handles.add(handle('size', l + Offset(r * px, 0), (u) {
            final (lx, ly) = units(l);
            m.carry('radius', math.sqrt(math.pow(u.$1 - lx, 2) + math.pow(u.$2 - ly, 2)));
          }));
        }
      }
      final tags = wsRelationsOf(ws), sel = ws.selected == null ? null : on(ws.selected!);
      if (sel != null && m != null && !isRelation) // (a thing picked: the shape of each relation it is in, its leader's ring too, while it is carried)
        for (final r in tags.map(int.parse)) {
          double val(String n, [double or = 0]) => m.value(r, n, now) ?? or;
          final px = m.point * v.s, l = val('leader') > 0 ? on('${val('leader').round()}') : null, off = val('visible', 1) <= .5;
          switch (m.word(r, 'file')) {
            case 'around':
              shape.add(_Shape.ring(screen(val('centre_x'), val('centre_y')), val('radius', .5) * px, off: off));
            case 'row':
              final half = val('gap', .2) * (m.word(r, 'members').split(' ').length - 1) / 2;
              shape.add(_Shape.line(screen(val('centre_x') - half, val('centre_y')), screen(val('centre_x') + half, val('centre_y')), off: off));
            case 'push' when l != null:
              shape.add(_Shape.ring(l, val('radius', .5) * px, off: off));
          }
        }
      return Stack(
        children: [
          IgnorePointer(child: CustomPaint(size: Size.infinite, painter: _MarksPainter(lasso, [...picked, for (final (_, p) in members) p], shape))),
          ...handles,
          for (final (id, p) in members) ...[
            Positioned(left: p.dx + 12, top: p.dy - 24, child: _Badge('×', () => wsFree(ws, id), key: ValueKey('ws-free-$id'))), // (one taken out of it where it stands: the others stay)
            if ((kind == 'follow' || kind == 'push') && id != leader) Positioned(left: p.dx - 24, top: p.dy - 24, child: _Badge('crown', () => wsLead(ws, id), faint: true, key: ValueKey('ws-lead-$id'))), // (another's crown: that one leads, nothing moving)
          ],
          if (on(leader ?? '') case final l? when leader != null) Positioned(left: l.dx - 24, top: l.dy - 24, child: _Badge('crown', () {}, key: const ValueKey('ws-leader'))), // (the leader)
          if (sel != null && tags.isNotEmpty) // (the relations the picked thing is in, the one on top first: a touch picks it)
            Positioned(left: sel.dx + 20, top: sel.dy + 18, child: Row(children: [for (final r in tags) _RelateChip(m!.word(int.parse(r), 'file'), () => ws.select(r), outlined: r != wsWinner(ws), key: ValueKey('ws-in-$r'))])),   // (filled: the one that shows; the rest under it, or switched off, outlined)
          if (lasso == null && (picked.length > 1 || isRelation)) // (along the top of the stage, never over a thing)
            Positioned(
              left: v.area.left,
              right: v.area.right - v.area.width,
              top: v.area.top + 8,
              child: Center(
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: picked.length > 1
                      ? [_Said('${picked.length} picked'), for (final k in wsRelations) _RelateChip(k, () => wsRelate(ws, k))]
                      : [
                          _Said('${kind[0].toUpperCase()}${kind.substring(1)} · ${members.length}'),
                          _RelateChip('eye', () => wsSwitch(ws), glyph: (m!.value(rel!, 'visible', 0) ?? 1) > .5 ? TpGlyph.eye : TpGlyph.eyeOff, key: const ValueKey('ws-relation-switch')), // (its eye, as on its row: shut, what it is on top of shows)
                          _RelateChip('bake', () => wsRelease(ws), key: const ValueKey('ws-relate-release')),   // (baked: kept where they show, the relation gone; × and Delete: back to what is under it)
                        ],
                ),
              ),
            ),
        ],
      );
    },
  );
}

/// A relation's shape on the stage: a ring (its middle, how far) or a line.
class _Shape {
  _Shape.ring(this.a, this.r, {this.off = false}) : b = null;
  _Shape.line(this.a, Offset this.b, {this.off = false}) : r = 0;
  final Offset a;
  final Offset? b;
  final double r;
  final bool off;   // (its relation's eye shut: drawn faint)
}

/// Drawn round: the line the hand drew; a ring round each thing picked together or moved by the relation picked; the
/// relation's shape, thin and broken.
class _MarksPainter extends CustomPainter {
  _MarksPainter(this.lasso, this.at, this.shape);
  final List<Offset>? lasso;
  final List<Offset> at;
  final List<_Shape> shape;
  @override
  void paint(Canvas c, Size s) {
    final ink = Paint()
      ..color = WsT.accent
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.5;
    if (lasso case final l? when l.length > 1) c.drawPath(Path()..addPolygon(l, false), ink);
    for (final p in at) {
      c.drawCircle(p, 14, ink);
    }
    final thin = Paint()
      ..color = WsT.accent.withValues(alpha: .8)
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1;
    for (final sh in shape) {
      thin.color = WsT.accent.withValues(alpha: sh.off ? .25 : .8);
      final path = sh.b == null ? (Path()..addOval(Rect.fromCircle(center: sh.a, radius: sh.r))) : (Path()..moveTo(sh.a.dx, sh.a.dy)..lineTo(sh.b!.dx, sh.b!.dy));
      for (final m in path.computeMetrics()) {
        for (var d = 0.0; d < m.length; d += 9) {
          c.drawPath(m.extractPath(d, math.min(d + 5, m.length)), thin);
        }
      }
    }
  }

  @override
  bool shouldRepaint(_MarksPainter o) => true;
}

/// A word in the bar over the stage, on the panel's ground (the bar lies over the stage's surround, dark or light).
class _Said extends StatelessWidget {
  const _Said(this.text);
  final String text;
  @override
  Widget build(BuildContext context) => Container(
    margin: EdgeInsets.only(right: Shape.s(6)),
    padding: EdgeInsets.symmetric(horizontal: Shape.s(8), vertical: Shape.s(5)),
    decoration: BoxDecoration(color: WsT.body, borderRadius: BorderRadius.all(Shape.r(12))),
    child: Text(text, style: T.label(Grey.g91)),
  );
}

/// What things picked together may do together, or a relation a thing is in (outlined): a word on a pill.
class _RelateChip extends StatelessWidget {
  const _RelateChip(this.kind, this.onTap, {this.outlined = false, this.glyph, super.key});
  final String kind;
  final VoidCallback onTap;
  final bool outlined;
  final TpGlyph? glyph;   // (drawn instead of its word)
  @override
  Widget build(BuildContext context) => GestureDetector(
    key: key == null ? ValueKey('ws-relate-$kind') : null,
    onTap: onTap,
    child: Container(
      margin: EdgeInsets.only(right: Shape.s(4)),
      padding: EdgeInsets.symmetric(horizontal: Shape.s(10), vertical: Shape.s(5)),
      decoration: BoxDecoration(color: outlined ? WsT.body : WsT.accent, border: outlined ? Border.all(color: WsT.accent) : null, borderRadius: BorderRadius.all(Shape.r(12))),
      child: glyph != null ? TpGlyphView(glyph!, WsT.onAccent, size: Shape.z(12)) : Text(kind[0].toUpperCase() + kind.substring(1), style: T.label(outlined ? WsT.accent : WsT.onAccent)),
    ),
  );
}

/// A small round mark on a thing a relation moves: × takes it out, a crown is its leader.
class _Badge extends StatelessWidget {
  const _Badge(this.glyph, this.onTap, {this.faint = false, super.key});
  final String glyph;
  final VoidCallback onTap;
  final bool faint;
  @override
  Widget build(BuildContext context) => GestureDetector(
    onTap: onTap,
    child: Container(
      width: 18,
      height: 18,
      decoration: BoxDecoration(color: faint ? WsT.body.withValues(alpha: .7) : WsT.accent, shape: BoxShape.circle),
      child: CustomPaint(painter: _GlyphPaint(glyph == '×' ? 'x' : glyph, faint ? Grey.g63 : WsT.onAccent)),
    ),
  );
}

/// A small mark drawn, not a letter (no font has to carry it): a cross (a middle), an arrow out (a size), an x (out of
/// it), a crown (the leader).
class _GlyphPaint extends CustomPainter {
  _GlyphPaint(this.what, this.ink);
  final String what;
  final Color ink;
  @override
  void paint(Canvas c, Size s) {
    final m = s.center(Offset.zero), r = s.shortestSide * .28;
    final pen = Paint()
      ..color = ink
      ..strokeWidth = 1.6
      ..style = PaintingStyle.stroke
      ..strokeCap = StrokeCap.round;
    switch (what) {
      case 'cross':
        c
          ..drawLine(m - Offset(r, 0), m + Offset(r, 0), pen)
          ..drawLine(m - Offset(0, r), m + Offset(0, r), pen);
      case 'arrow':
        c
          ..drawLine(m - Offset(r, 0), m + Offset(r, 0), pen)
          ..drawLine(m + Offset(r, 0), m + Offset(r * .4, -r * .6), pen)
          ..drawLine(m + Offset(r, 0), m + Offset(r * .4, r * .6), pen);
      case 'x':
        c
          ..drawLine(m - Offset(r, r), m + Offset(r, r), pen)
          ..drawLine(m - Offset(r, -r), m + Offset(r, -r), pen);
      case 'crown':
        c.drawPath(
          Path()..addPolygon([m + Offset(-r * 1.2, r * .8), m + Offset(-r * 1.2, -r * .6), m + Offset(-r * .5, 0), m + Offset(0, -r), m + Offset(r * .5, 0), m + Offset(r * 1.2, -r * .6), m + Offset(r * 1.2, r * .8)], true),
          Paint()..color = ink,
        );
    }
  }

  @override
  bool shouldRepaint(_GlyphPaint o) => o.what != what || o.ink != ink;
}

/// The pill at the bottom-left of the viewport: zoom and where the pointer is, in composition px.
class _Hud extends StatelessWidget {
  _Hud({required this.scale, required this.cursor});
  final double scale;
  final ValueNotifier<Offset?> cursor;
  @override
  Widget build(BuildContext context) => Container(
    height: _V.hud,
    padding: EdgeInsets.symmetric(horizontal: Shape.s(8)),
    decoration: BoxDecoration(color: WsT.card, borderRadius: BorderRadius.all(Shape.r(WsT.radius))),
    child: Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Text('${(scale * 100).round()}%', style: T.value(Grey.g95)),
        Container(width: 1, height: Shape.z(10), margin: EdgeInsets.symmetric(horizontal: Shape.s(8)), color: WsT.line),
        ValueListenableBuilder<Offset?>(
          valueListenable: cursor,
          builder: (_, p, _) => Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text('X ', style: T.micro(Grey.g56)),
              SizedBox(width: Shape.z(34), child: Text(p == null ? '—' : '${p.dx.round()}', style: T.value(p == null ? Grey.g56 : Grey.g91))),
              Text('Y ', style: T.micro(Grey.g56)),
              SizedBox(width: Shape.z(34), child: Text(p == null ? '—' : '${p.dy.round()}', style: T.value(p == null ? Grey.g56 : Grey.g91))),
            ],
          ),
        ),
      ],
    ),
  );
}

/// The navigator: the whole frame small, the part the viewport shows outlined in the accent. Shown only when zoomed past fit.
class _Navigator extends StatelessWidget {
  const _Navigator({required this.scene, required this.view, this.texture});
  final ArtScene scene;
  final _View view;

  /// Joined: the inside's own picture, the whole frame, under the outline (the same texture the stage shows).
  final int? texture;
  @override
  Widget build(BuildContext context) {
    final h = _V.mapW * WsArt.comp.height / WsArt.comp.width;
    return Container(
      width: _V.mapW + 8,
      height: h + 8,
      padding: EdgeInsets.all(Shape.s(4)),
      color: WsT.card,
      child: Stack(
        fit: StackFit.expand,
        children: [
          if (texture != null) Texture(textureId: texture!, filterQuality: FilterQuality.medium),
          CustomPaint(painter: _MapPainter(scene, Rect.fromPoints(view.toComp(view.area.topLeft), view.toComp(view.area.bottomRight)))),
        ],
      ),
    );
  }
}

class _MapPainter extends CustomPainter {
  _MapPainter(this.scene, this.shown);
  final ArtScene scene;
  final Rect shown;
  @override
  void paint(Canvas c, Size s) {
    final k = s.width / WsArt.comp.width;
    c.save();
    c.clipRect(Offset.zero & s);
    c.scale(k);
    c.restore();
    final r = Rect.fromLTRB(shown.left * k, shown.top * k, shown.right * k, shown.bottom * k).intersect(Offset.zero & s);
    c.drawRect(
      r.deflate(.75),
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.5
        ..color = WsT.accent,
    );
  }

  @override
  bool shouldRepaint(_MapPainter o) => o.scene != scene || o.shown != shown;
}

// ---- the picture and what sits on it ----------------------------------------------------------------------------------

Paint _line(Color c, [double w = 1]) => Paint()
  ..style = PaintingStyle.stroke
  ..strokeWidth = w
  ..color = c;

/// A line that reads on any artwork: a dark under-stroke, then the colour.
void _legible(Canvas c, Path p, Color col, [double w = 1.5]) {
  c.drawPath(p, _line(Grey.g00, w + 2));
  c.drawPath(p, _line(col, w));
}

TextPainter _tp(String t, TextStyle s) => TextPainter(
  text: TextSpan(text: t, style: s),
  textDirection: TextDirection.ltr,
  maxLines: 1,
)..layout();

/// What one painter draws: everything (the stand-in art too), or, when the inside's picture sits in the composition,
/// what lies under it (the surround, the captions) and what lies over it (grid, guides, the selection).
enum _Part { all, under, over }

class _PointsPainter extends CustomPainter {
  _PointsPainter(this.at, this.handles);
  final List<Offset> at;
  final List<(Offset, Offset)> handles;
  @override
  void paint(Canvas c, Size _) {
    final edge = Paint()..color = WsT.accent..style = PaintingStyle.stroke..strokeWidth = 1, inside = Paint()..color = const Color(0xFFFFFFFF);
    for (final (h, o) in handles) {
      if ((h - o).distance < 1) continue; // (a handle lying on its point: none to see)
      c..drawLine(o, h, edge)..drawCircle(h, 3, inside)..drawCircle(h, 3, edge);
    }
    for (final p in at) {
      final r = Rect.fromCenter(center: p, width: Shape.z(6), height: Shape.z(6));
      c..drawRect(r, inside)..drawRect(r, edge);
    }
  }

  @override
  bool shouldRepaint(_PointsPainter o) => !listEquals(o.at, at) || !listEquals(o.handles, handles);
}

class _HandlePainter extends CustomPainter {
  _HandlePainter(this.at, this.out);
  final Offset at, out;
  @override
  void paint(Canvas c, Size _) {
    final line = Paint()..color = WsT.accent..strokeWidth = 1, dot = Paint()..color = WsT.accent;
    c.drawLine(at * 2 - out, out, line);
    for (final p in [at * 2 - out, out]) c.drawCircle(p, 3, dot);
    c.drawRect(Rect.fromCenter(center: at, width: Shape.z(6), height: Shape.z(6)), dot);
  }

  @override
  bool shouldRepaint(_HandlePainter o) => o.at != at || o.out != out;
}

class _ViewPainter extends CustomPainter {
  _ViewPainter(this.scene, this.v, this.sel, this.hover, {required this.grid, required this.safe, this.part = _Part.all, this.pivots = false, this.met = (null, null), this.marked = false});

  /// The inside marks the picked thing and the one under the hand on its desk: not drawn here.
  final bool marked;
  final _Part part;

  /// Joined: every thing's point it turns about shown (Pivots), and the lines a carried thing has stuck to (Snap).
  final bool pivots;
  final (double?, double?) met;
  final ArtScene scene;
  final _View v;
  final String? sel, hover;
  final bool grid, safe;

  Offset _scr(Offset doc) => v.toScreen(WsArt.toView(scene, doc));
  Path _boxPath(ArtBox b) => Path()..addPolygon([for (final p in b.corners) _scr(p)], true);

  @override
  void paint(Canvas c, Size size) {
    final comp = v.comp;
    if (part != _Part.over) c.drawRect(Offset.zero & size, Paint()..color = _V.surround);
    c.save();
    c.clipRect(v.area);
    if (part != _Part.over) _captions(c, comp);
    if (part == _Part.under) {
      c.restore();
      return;
    }
    c.save();
    c.clipRect(comp);
    if (grid) _grid(c, comp);
    if (safe) _safe(c, comp);
    c.restore();
    _guides(c);
    if (met.$1 case final x?) c.drawLine(Offset(v.toScreen(Offset(x, 0)).dx, v.area.top), Offset(v.toScreen(Offset(x, 0)).dx, v.area.bottom), _line(WsT.accent));
    if (met.$2 case final y?) c.drawLine(Offset(v.area.left, v.toScreen(Offset(0, y)).dy), Offset(v.area.right, v.toScreen(Offset(0, y)).dy), _line(WsT.accent));
    if (pivots) {
      for (final MapEntry(:key, :value) in scene.boxes.entries) {
        if (key == sel) continue;
        final a = _scr(value.anchor);
        c.drawCircle(a, 3.5, _line(Grey.g00, 3));
        c.drawCircle(a, 3.5, _line(Grey.g91, 1.2));
      }
    }
    if (marked) return c.restore();
    if (WsArt.boxOf(scene, hover) case final b?) _legible(c, _boxPath(b), wsTone(scene.layer(hover)!.kind), 1);
    if (WsArt.boxOf(scene, sel) case final b?) _selection(c, b, scene.layer(sel)!);
    c.restore();
  }

  void _captions(Canvas c, Rect comp) {
    if (comp.top - v.area.top < 18) return;
    final l = _tp(scene.title, T.micro(Grey.g76).copyWith(fontWeight: FontWeight.w700)), d = _tp('${WsArt.comp.width.round()} × ${WsArt.comp.height.round()}  ·  ${Ws.fps} fps', T.micro(Grey.g56));
    final y = comp.top - 6 - l.height;
    l.paint(c, Offset(comp.left, y));
    d.paint(c, Offset(comp.left + l.width + 8, y));
    final f = scene.frame, tc = _tp('F ${f.toString().padLeft(3, '0')}', T.value(Grey.g76));
    tc.paint(c, Offset(comp.right - tc.width, comp.top - 6 - tc.height));
  }

  void _grid(Canvas c, Rect comp) {
    final p = _line(_V.overlay);
    for (var i = 1; i < 12; i++) {
      final x = comp.left + comp.width * i / 12;
      c.drawLine(Offset(x, comp.top), Offset(x, comp.bottom), p);
    }
    for (var j = 1; j < 7; j++) {
      final y = comp.top + comp.height * j / 7;
      c.drawLine(Offset(comp.left, y), Offset(comp.right, y), p);
    }
  }

  void _safe(Canvas c, Rect comp) {
    final p = _line(_V.safe);
    c.drawRect(Rect.fromCenter(center: comp.center, width: comp.width * .9, height: comp.height * .9), _line(_V.overlay));
    final t = Rect.fromCenter(center: comp.center, width: comp.width * .8, height: comp.height * .8), k = comp.width * .03;
    for (final (o, dx, dy) in [(t.topLeft, 1.0, 1.0), (t.topRight, -1.0, 1.0), (t.bottomRight, -1.0, -1.0), (t.bottomLeft, 1.0, -1.0)]) {
      c.drawPath(
        Path()
          ..moveTo(o.dx + dx * k, o.dy)
          ..lineTo(o.dx, o.dy)
          ..lineTo(o.dx, o.dy + dy * k),
        p,
      );
    }
    final m = comp.center;
    c.drawLine(m - const Offset(8, 0), m + const Offset(8, 0), p);
    c.drawLine(m - const Offset(0, 8), m + const Offset(0, 8), p);
  }

  void _guides(Canvas c) {
    final p = _line(_V.guide);
    for (final gx in scene.guidesX) {
      final x = v.toScreen(Offset(gx, 0)).dx.roundToDouble() + .5;
      c.drawLine(Offset(x, v.area.top), Offset(x, v.area.bottom), p);
    }
    for (final gy in scene.guidesY) {
      final y = v.toScreen(Offset(0, gy)).dy.roundToDouble() + .5;
      c.drawLine(Offset(v.area.left, y), Offset(v.area.right, y), p);
    }
  }

  void _selection(Canvas c, ArtBox b, ArtLayer l) {
    final h = _handles(_scr, b, knob: l.kind != WsKind.camera && l.id != 'bg'), pts = [for (var i = 0; i < 4; i++) h['corner $i']!];
    _legible(c, Path()..addPolygon(pts, true), WsT.accent);
    final top = _scr(b.top);
    if (h['turn'] case final knob?) {
      _legible(
        c,
        Path()
          ..moveTo(top.dx, top.dy)
          ..lineTo(knob.dx, knob.dy),
        WsT.accent,
        1,
      );
      c.drawCircle(knob, 4.5, Paint()..color = Grey.g00);
      c.drawCircle(knob, 3.5, Paint()..color = WsT.accent);
    }
    for (final p in pts) {
      final r = Rect.fromCenter(center: p, width: _V.handle, height: _V.handle);
      c.drawRect(r.inflate(1), Paint()..color = Grey.g00);
      c.drawRect(r, Paint()..color = WsT.accent);
    }
    final a = h['anchor']!;
    c.drawCircle(a, 6, _line(Grey.g00, 3.5));
    c.drawCircle(a, 6, _line(WsT.accent, 1.5));
    for (final d in const [Offset(1, 0), Offset(-1, 0), Offset(0, 1), Offset(0, -1)]) {
      c.drawLine(a + d * 3, a + d * 10, _line(Grey.g00, 3));
      c.drawLine(a + d * 3, a + d * 10, _line(WsT.accent, 1.2));
    }
    final bounds = pts.skip(1).fold(Rect.fromPoints(pts[0], pts[0]), (r, p) => r.expandToInclude(Rect.fromPoints(p, p)));
    _tag(c, bounds.topLeft, l.name, WsT.accent, WsT.onAccent, wsTone(l.kind));
  }

  /// A small rounded name tag sitting on top of [at] (moved inside the viewport when it would leave it).
  void _tag(Canvas c, Offset at, String text, Color fill, Color ink, Color? pip) {
    final tp = _tp(text, T.micro(ink).copyWith(fontWeight: FontWeight.w700));
    final w = tp.width + 12 + (pip == null ? 0 : 10);
    var o = Offset(at.dx, at.dy - _V.tagH - 4);
    o = Offset(
      o.dx.clamp(v.area.left + 4, math.max(v.area.left + 4, v.area.right - w - 4)),
      o.dy.clamp(v.area.top + 4, math.max(v.area.top + 4, v.area.bottom - _V.tagH - 4)),
    );
    final r = RRect.fromRectAndRadius(Rect.fromLTWH(o.dx, o.dy, w, _V.tagH), Shape.r(WsT.chipRadius));
    c.drawRRect(r.inflate(1), Paint()..color = Grey.g00);
    c.drawRRect(r, Paint()..color = fill);
    var x = o.dx + 6;
    if (pip != null) {
      c.drawRect(Rect.fromLTWH(x, o.dy + 5, 6, 6), Paint()..color = Grey.g00);
      c.drawRect(Rect.fromLTWH(x + 1, o.dy + 6, 4, 4), Paint()..color = pip);
      x += 10;
    }
    tp.paint(c, Offset(x, o.dy + (_V.tagH - tp.height) / 2));
  }

  @override
  bool shouldRepaint(_ViewPainter o) => o.scene != scene || o.v != v || o.sel != sel || o.hover != hover || o.grid != grid || o.safe != safe || o.part != part || o.pivots != pivots || o.met != met || o.marked != marked;
}

// ---- rulers -----------------------------------------------------------------------------------------------------------

class _RulerPainter extends CustomPainter {
  _RulerPainter(this.v, this.cursor, this.scene) : super(repaint: cursor);
  final _View v;
  final ArtScene scene;
  final ValueNotifier<Offset?> cursor;

  static double _step(double s) {
    for (final st in [25.0, 50.0, 100.0, 200.0, 250.0, 500.0, 1000.0]) {
      if (st * s >= 56) return st;
    }
    return 2000;
  }

  @override
  void paint(Canvas c, Size size) {
    final r = _V.ruler;
    final comp = v.comp, bg = Paint()..color = _V.rulerGround, span = Paint()..color = _V.rulerSpan;
    c.drawRect(Rect.fromLTWH(0, 0, size.width, r), bg);
    c.drawRect(Rect.fromLTWH(0, 0, r, size.height), bg);
    c.drawRect(Rect.fromLTRB(math.max(comp.left, r), 0, math.min(comp.right, size.width), r), span);
    c.drawRect(Rect.fromLTRB(0, math.max(comp.top, r), r, math.min(comp.bottom, size.height)), span);
    final edge = _line(WsT.line);
    c.drawLine(Offset(0, r - .5), Offset(size.width, r - .5), edge);
    c.drawLine(Offset(r - .5, 0), Offset(r - .5, size.height), edge);
    final step = _step(v.s), minor = step / 5, tick = Paint()..color = _V.tick;
    final label = T.micro(Grey.g56);

    c.save();
    c.clipRect(Rect.fromLTWH(r, 0, size.width - r, r));
    final x0 = (v.toComp(Offset(r, 0)).dx / minor).floor() * minor, x1 = v.toComp(Offset(size.width, 0)).dx;
    for (var x = x0; x <= x1; x += minor) {
      final sx = v.toScreen(Offset(x, 0)).dx.roundToDouble() + .5, major = (x / step).round() * step == x;
      c.drawLine(Offset(sx, major ? 2 : r - 4), Offset(sx, r), tick);
      if (major) _tp('${x.round()}', label).paint(c, Offset(sx + 3, 2));
    }
    c.restore();

    c.save();
    c.clipRect(Rect.fromLTWH(0, r, r, size.height - r));
    final y0 = (v.toComp(Offset(0, r)).dy / minor).floor() * minor, y1 = v.toComp(Offset(0, size.height)).dy;
    for (var y = y0; y <= y1; y += minor) {
      final sy = v.toScreen(Offset(0, y)).dy.roundToDouble() + .5, major = (y / step).round() * step == y;
      c.drawLine(Offset(major ? 2 : r - 4, sy), Offset(r, sy), tick);
      if (major) {
        c.save();
        c.translate(2, sy - 3);
        c.rotate(-math.pi / 2);
        _tp('${y.round()}', label).paint(c, Offset.zero);
        c.restore();
      }
    }
    c.restore();

    final g = Paint()..color = _V.guide;
    for (final gx in scene.guidesX) {
      final x = v.toScreen(Offset(gx, 0)).dx;
      if (x > r) c.drawPath(Path()..addPolygon([Offset(x - 4, r - 5), Offset(x + 4, r - 5), Offset(x, r)], true), g);
    }
    for (final gy in scene.guidesY) {
      final y = v.toScreen(Offset(0, gy)).dy;
      if (y > r) c.drawPath(Path()..addPolygon([Offset(r - 5, y - 4), Offset(r - 5, y + 4), Offset(r, y)], true), g);
    }

    if (cursor.value case final p?) {
      final s = v.toScreen(p),
          hot = Paint()
            ..color = WsT.accent
            ..strokeWidth = 1;
      if (s.dx > r) c.drawLine(Offset(s.dx, 0), Offset(s.dx, r), hot);
      if (s.dy > r) c.drawLine(Offset(0, s.dy), Offset(r, s.dy), hot);
    }
    c.drawRect(Rect.fromLTWH(0, 0, r, r), bg);
  }

  @override
  bool shouldRepaint(_RulerPainter o) => o.v != v || o.cursor != cursor || o.scene != scene;
}
