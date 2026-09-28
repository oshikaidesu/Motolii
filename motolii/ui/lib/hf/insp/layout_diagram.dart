// The Layout diagram: one abstract container with its children, touched at once.
// Columns and rows arrange the children, Gap spaces them, Padding holds them off the container's wall,
// dragging the group inside places it (Justify and Align together), and the container's own edges size it.
// Not a mini Stage: no picture, just the relationships.
import 'dart:math' as math;
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import '../bp/common.dart';
import '../desk/common.dart' show kMint, kBlue, kViolet, kPink;
import 'layout_model.dart';
import '../neutral.dart';

const arrangeColor = kMint, spaceColor = kBlue, alignColor = kViolet, sizeColor = kPink;

/// Where everything is, from the store's numbers. Public so a test can reach the same handles a pointer would.
class LayoutGeom {
  LayoutGeom(this.size, this.s);
  final Size size;
  final LayoutStore s;
  static const k = .45, cw = 34.0, ch = 24.0;

  Rect get avail => Rect.fromLTRB(10, 24, size.width - 10, size.height - 12); // the top margin holds the readout
  int get n => s.children;
  int get cols => math.max(1, s.gi('layout.grid_columns'));
  int get autoRows => (n / cols).ceil();
  int get rows => s.gi('layout.grid_rows') > 0 ? s.gi('layout.grid_rows') : autoRows;
  double get gap => s.gd('layout.gap') * k;
  double get padX => s.padX * k;
  double get padY => s.padY * k;
  int get hs => s.gi('layout.horizontal_sizing');
  int get vs => s.gi('layout.vertical_sizing');
  Size get content => Size(cols * cw + (cols - 1) * gap, rows * ch + (rows - 1) * gap);

  Rect get parent {
    final w = hs == 0 ? content.width + 2 * padX : (hs == 1 ? avail.width : (s.gd('layout.width') * k).clamp(50.0, avail.width));
    final h = vs == 0 ? content.height + 2 * padY : (vs == 1 ? avail.height : (s.gd('layout.height') * k).clamp(40.0, avail.height));
    return Rect.fromLTWH(avail.left, avail.top, math.min(w, avail.width), math.min(h, avail.height));
  }

  Rect get inner => Rect.fromLTRB(parent.left + padX, parent.top + padY, math.max(parent.left + padX, parent.right - padX), math.max(parent.top + padY, parent.bottom - padY));

  int get justify => s.gi('layout.justify_content');
  int get align => s.gi('layout.align_items');

  /// The children, placed as the layout rules place them (Start / End / Center / Between / Around / Evenly; Stretch).
  List<Rect> get boxes {
    final extraX = math.max(0.0, inner.width - content.width), extraY = math.max(0.0, inner.height - content.height);
    var gx = gap, x0 = 0.0;
    switch (justify) {
      case 1: x0 = extraX;
      case 2: x0 = extraX / 2;
      case 3: if (cols > 1) gx = gap + extraX / (cols - 1);
      case 4: gx = gap + extraX / cols; x0 = extraX / (2 * cols);
      case 5: gx = gap + extraX / (cols + 1); x0 = extraX / (cols + 1);
    }
    var y0 = 0.0, h = ch;
    switch (align) {
      case 0: h = math.max(ch, (inner.height - (rows - 1) * gap) / rows);
      case 2: y0 = extraY;
      case 3: y0 = extraY / 2;
    }
    return [
      for (var i = 0; i < n; i++)
        if (i ~/ cols < rows) Rect.fromLTWH(inner.left + x0 + (i % cols) * (cw + gx), inner.top + y0 + (i ~/ cols) * (h + gap), cw, h),
    ];
  }

  // ---- handles ------------------------------------------------------------------------------------------------
  Offset? get gapHandle {
    final b = boxes;
    if (b.length < 2) return null;
    if (cols > 1) return Offset((b[0].right + b[1].left) / 2, b[0].center.dy);
    return Offset(b[0].center.dx, (b[0].bottom + b[1].top) / 2);
  }

  Offset get colHandle { final b = boxes; final row0 = [for (var i = 0; i < b.length && i < cols; i++) b[i]]; return Offset((row0.isEmpty ? inner.left : row0.last.right) + 7, (row0.isEmpty ? inner.top : row0.first.center.dy)); }
  Offset get rowHandle { final b = boxes; final col0 = [for (var i = 0; i < b.length; i += cols) b[i]]; return Offset(col0.isEmpty ? inner.left : col0.first.center.dx, (col0.isEmpty ? inner.top : col0.last.bottom) + 7); }
  Offset get padL => Offset(inner.left, parent.center.dy);
  Offset get padR => Offset(inner.right, parent.center.dy);
  Offset get padT => Offset(parent.center.dx, inner.top);
  Offset get padB => Offset(parent.center.dx, inner.bottom);
  Offset get padCorner => Offset(inner.left, inner.top);
  Offset get sizeW => Offset(parent.right + 6, parent.center.dy);
  Offset get sizeH => Offset(parent.center.dx, parent.bottom + 6);
}

enum _G { none, gap, cols, rows, padL, padR, padT, padB, padBoth, sizeW, sizeH, align }

class LayoutDiagram extends StatefulWidget {
  const LayoutDiagram(this.store, {super.key, this.size = const Size(286, 190)});
  final LayoutStore store;
  final Size size;
  @override
  State<LayoutDiagram> createState() => _LayoutDiagramState();
}

class _LayoutDiagramState extends State<LayoutDiagram> {
  _G grab = _G.none;
  Offset _down = Offset.zero;
  double _v0 = 0, _w0 = 0;
  bool _moved = false;
  LayoutStore get s => widget.store;
  LayoutGeom get g => LayoutGeom(widget.size, s);
  bool get fine => HardwareKeyboard.instance.isShiftPressed;

  void _pick(Offset p) {
    grab = _G.none;
    final geo = g;
    bool near(Offset? o, double r) => o != null && (p - o).distance < r;
    // Columns and Rows work with Grid off, as Classic's wells do; everything else is gated by Grid
    if (near(geo.colHandle, 11)) { grab = _G.cols; _v0 = s.gi('layout.grid_columns').toDouble(); }
    else if (near(geo.rowHandle, 11)) { grab = _G.rows; _v0 = s.gi('layout.grid_rows').toDouble(); }
    else if (s.gridOn) {
      if (near(geo.gapHandle, 10)) { grab = _G.gap; _v0 = s.gd('layout.gap'); }
      else if (near(geo.padCorner, 9)) { grab = _G.padBoth; _v0 = s.padX; _w0 = s.padY; }
      else if (near(geo.padL, 9)) { grab = _G.padL; _v0 = s.padX; }
      else if (near(geo.padR, 9)) { grab = _G.padR; _v0 = s.padX; }
      else if (near(geo.padT, 9)) { grab = _G.padT; _v0 = s.padY; }
      else if (near(geo.padB, 9)) { grab = _G.padB; _v0 = s.padY; }
      else if (near(geo.sizeW, 10) && s.fixed('w')) { grab = _G.sizeW; _v0 = s.gd('layout.width'); }
      else if (near(geo.sizeH, 10) && s.fixed('h')) { grab = _G.sizeH; _v0 = s.gd('layout.height'); }
      else if (geo.boxes.any((b) => b.inflate(3).contains(p)) || geo.inner.contains(p)) { grab = _G.align; }
    }
  }

  double _fineK() => fine ? .1 : 1;

  void _drag(Offset p) {
    final geo = g;
    final d = (p - _down) * _fineK();
    double px(double v) => v.roundToDouble();
    switch (grab) {
      case _G.gap:
        s.preview('layout.gap', px((_v0 + (geo.cols > 1 ? d.dx : d.dy) / LayoutGeom.k).clamp(0.0, 400.0)));
      case _G.cols:
        final c = ((p.dx - geo.boxes.first.left + geo.gap) / (LayoutGeom.cw + geo.gap)).round().clamp(1, 12);
        s.preview('layout.grid_columns', c);
      case _G.rows:
        final r = ((p.dy - geo.boxes.first.top + geo.gap) / (LayoutGeom.ch + geo.gap)).round();
        s.preview('layout.grid_rows', r < 1 ? 0 : r.clamp(1, 12));
      case _G.padL:
        s.preview('layout.padding', [px((_v0 + d.dx / LayoutGeom.k).clamp(0.0, 200.0)), s.padY]);
      case _G.padR:
        s.preview('layout.padding', [px((_v0 - d.dx / LayoutGeom.k).clamp(0.0, 200.0)), s.padY]);
      case _G.padT:
        s.preview('layout.padding', [s.padX, px((_v0 + d.dy / LayoutGeom.k).clamp(0.0, 200.0))]);
      case _G.padB:
        s.preview('layout.padding', [s.padX, px((_v0 - d.dy / LayoutGeom.k).clamp(0.0, 200.0))]);
      case _G.padBoth:
        s.preview('layout.padding', [px((_v0 + d.dx / LayoutGeom.k).clamp(0.0, 200.0)), px((_w0 + d.dy / LayoutGeom.k).clamp(0.0, 200.0))]);
      case _G.sizeW:
        s.preview('layout.width', px((_v0 + d.dx / LayoutGeom.k).clamp(20.0, 2000.0)));
      case _G.sizeH:
        s.preview('layout.height', px((_v0 + d.dy / LayoutGeom.k).clamp(20.0, 2000.0)));
      case _G.align:
        // the group follows the pointer to the nearest of nine places: Justify and Align are one gesture
        final r = geo.inner;
        final ix = ((p.dx - r.left) / math.max(1, r.width) * 3).floor().clamp(0, 2), iy = ((p.dy - r.top) / math.max(1, r.height) * 3).floor().clamp(0, 2);
        s.applyMany({'layout.justify_content': const [0, 2, 1][ix], 'layout.align_items': const [1, 3, 2][iy]});
      case _G.none:
        break;
    }
  }

  void _commit() {
    if (grab != _G.none && _moved) {
      s.commit(switch (grab) {
        _G.gap => 'layout.gap',
        _G.cols => 'layout.grid_columns',
        _G.rows => 'layout.grid_rows',
        _G.padL || _G.padR || _G.padT || _G.padB || _G.padBoth => 'layout.padding',
        _G.sizeW => 'layout.width',
        _G.sizeH => 'layout.height',
        _ => 'layout.justify_content',
      });
    }
    grab = _G.none;
  }

  final _focus = FocusNode(debugLabel: 'layout diagram');

  @override
  void dispose() {
    _focus.dispose();
    super.dispose();
  }

  /// Esc during a drag: every property the gesture touched goes back and the rest of the drag does nothing.
  void _abort() {
    final ids = switch (grab) {
      _G.gap => ['layout.gap'],
      _G.cols => ['layout.grid_columns'],
      _G.rows => ['layout.grid_rows'],
      _G.padL || _G.padR || _G.padT || _G.padB || _G.padBoth => ['layout.padding'],
      _G.sizeW => ['layout.width'],
      _G.sizeH => ['layout.height'],
      _G.align => ['layout.justify_content', 'layout.align_items'],
      _G.none => <String>[],
    };
    for (final id in ids) { s.cancel(id); }
    setState(() => grab = _G.none);
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: s,
        builder: (context, _) => Focus(
          focusNode: _focus,
          onKeyEvent: (_, e) {
            if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape && grab != _G.none) {
              _abort();
              return KeyEventResult.handled;
            }
            return KeyEventResult.ignored;
          },
          child: Listener(
          key: const ValueKey('layout-diagram'),
          onPointerDown: (e) { _focus.requestFocus(); _down = e.localPosition; _moved = false; if (!s.frozen) _pick(e.localPosition); },
          onPointerMove: (e) { if (grab == _G.none) return; if ((e.localPosition - _down).distance > 2) _moved = true; _drag(e.localPosition); },
          onPointerUp: (_) { _commit(); setState(() {}); },
          onPointerCancel: (_) => grab = _G.none,
          child: CustomPaint(size: widget.size, painter: _Painter(g, s, grab)),
        )),
      );
}

class _Painter extends CustomPainter {
  _Painter(this.g, this.s, this.grab);
  final LayoutGeom g;
  final LayoutStore s;
  final _G grab;

  @override
  void paint(Canvas c, Size sz) {
    final on = s.gridOn, off = s.frozen;
    final a = off ? .5 : 1.0;
    c.drawRRect(RRect.fromRectAndRadius(Offset.zero & sz, const Radius.circular(6)), Paint()..color = N.g07);
    // the space the container is offered
    final dash = Paint()..color = N.g20..strokeWidth = 1;
    final av = g.avail;
    for (var x = av.left; x < av.right; x += 8) { c.drawLine(Offset(x, av.top), Offset(math.min(x + 4, av.right), av.top), dash); c.drawLine(Offset(x, av.bottom), Offset(math.min(x + 4, av.right), av.bottom), dash); }
    for (var y = av.top; y < av.bottom; y += 8) { c.drawLine(Offset(av.left, y), Offset(av.left, math.min(y + 4, av.bottom)), dash); c.drawLine(Offset(av.right, y), Offset(av.right, math.min(y + 4, av.bottom)), dash); }
    final dim = on ? 1.0 : .45;
    final p = g.parent, inn = g.inner;
    c.drawRRect(RRect.fromRectAndRadius(p, const Radius.circular(4)), Paint()..color = N.g13);
    c.drawRRect(RRect.fromRectAndRadius(p, const Radius.circular(4)), Paint()..color = sizeColor.withValues(alpha: .8 * dim * a)..style = PaintingStyle.stroke..strokeWidth = 1.6);
    // padding: the wall's thickness, shown by the line it holds the children off with
    final pd = Paint()..color = spaceColor.withValues(alpha: .7 * dim * a)..strokeWidth = 1.2;
    for (var x = inn.left; x < inn.right; x += 6) { c.drawLine(Offset(x, inn.top), Offset(math.min(x + 3, inn.right), inn.top), pd); c.drawLine(Offset(x, inn.bottom), Offset(math.min(x + 3, inn.right), inn.bottom), pd); }
    for (var y = inn.top; y < inn.bottom; y += 6) { c.drawLine(Offset(inn.left, y), Offset(inn.left, math.min(y + 3, inn.bottom)), pd); c.drawLine(Offset(inn.right, y), Offset(inn.right, math.min(y + 3, inn.bottom)), pd); }
    // the nine places the group can rest in, quiet until it is being moved
    final dots = Paint()..color = alignColor.withValues(alpha: grab == _G.align ? .9 : .28 * dim);
    for (var iy = 0; iy < 3; iy++) { for (var ix = 0; ix < 3; ix++) { c.drawCircle(Offset(inn.left + inn.width * (ix + .5) / 3, inn.top + inn.height * (iy + .5) / 3), grab == _G.align ? 2.4 : 1.4, dots); } }
    // children
    for (final b in g.boxes) {
      final rr = RRect.fromRectAndRadius(b, const Radius.circular(3));
      c.drawRRect(rr, Paint()..color = (on ? N.g20 : N.g15));
      c.drawRRect(rr, Paint()..color = arrangeColor.withValues(alpha: .75 * dim * a)..style = PaintingStyle.stroke..strokeWidth = 1.3);
    }
    void bar(Offset o, bool vertical, Color col, {bool hollow = false}) {
      final r = RRect.fromRectAndRadius(Rect.fromCenter(center: o, width: vertical ? 6 : 16, height: vertical ? 16 : 6), const Radius.circular(3));
      c.drawRRect(r, Paint()..color = hollow ? N.g07 : col.withValues(alpha: a));
      c.drawRRect(r, Paint()..color = col.withValues(alpha: a)..style = PaintingStyle.stroke..strokeWidth = 1.4);
    }
    void dot(Offset o, Color col, {bool hollow = false, double r = 4.6}) {
      c.drawCircle(o, r + 3, Paint()..color = col.withValues(alpha: .18 * a));
      c.drawCircle(o, r, Paint()..color = hollow ? N.g07 : col.withValues(alpha: a));
      c.drawCircle(o, r, Paint()..color = col.withValues(alpha: a)..style = PaintingStyle.stroke..strokeWidth = 1.4);
    }
    // arrangement handles: they work even with Grid off, as Classic's Columns and Rows wells do
    bar(g.colHandle, true, arrangeColor);
    bar(g.rowHandle, false, arrangeColor);
    if (on) {
      final gh = g.gapHandle;
      if (gh != null) dot(gh, spaceColor, r: 4);
      dot(g.padCorner, spaceColor, r: 4);
      for (final o in [g.padL, g.padR, g.padT, g.padB]) { c.drawRect(Rect.fromCenter(center: o, width: 6, height: 6), Paint()..color = spaceColor); }
      // sizing: solid where the number counts (Fixed), hollow where the container hugs or fills
      dot(g.sizeW, sizeColor, hollow: !s.fixed('w'));
      dot(g.sizeH, sizeColor, hollow: !s.fixed('h'));
    }
    final t = on
        ? '${g.cols} × ${s.gi('layout.grid_rows') == 0 ? 'auto' : s.gi('layout.grid_rows')}   gap ${s.gd('layout.gap').round()}   pad ${s.padX.round()}·${s.padY.round()}   ${justifyNames[s.gi('layout.justify_content').clamp(0, 5)]} / ${alignNames[s.gi('layout.align_items').clamp(0, 3)]}'
        : 'Grid off';
    final tp = TextPainter(text: TextSpan(text: t, style: mono(8.5, c: N.g51)), textDirection: TextDirection.ltr, maxLines: 1, ellipsis: '…')..layout(maxWidth: sz.width - 20);
    tp.paint(c, const Offset(10, 7));
  }

  @override
  bool shouldRepaint(_Painter o) => true;
}
