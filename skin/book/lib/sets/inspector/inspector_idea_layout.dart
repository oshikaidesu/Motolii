import 'dart:math' as math;
import 'inspector_studies.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_rows_sect.dart';
import '../../parts/controls.dart';
import '../../tokens.dart';
part 'inspector_idea_layout_2.dart';
part 'inspector_idea_layout_3.dart';
part 'inspector_idea_layout_4.dart';

// ---- Layout ideas: the container's CSS flex / grid lines edited on a live, solved preview ----------------------------
// Model: motolii-doc store/layout.rs (GROUP_ROWS / ITEM_ROWS). The preview solves its own small flex + grid (CSS rules, below).

const _lyJustify = ['Start', 'End', 'Center', 'Space Between', 'Space Around', 'Space Evenly'];
const _lyAlign = ['Stretch', 'Start', 'Center', 'End'];
const _lySelf = ['Auto', 'Stretch', 'Start', 'End', 'Center'];
const _lySizing = ['Hug', 'Fill', 'Fixed'];

Color get _lyPadC => Color.alphaBlend(Pop.toneLayout.withValues(alpha: .22), Pop.well);
Color get _lyPadHot => Color.alphaBlend(Pop.toneLayout.withValues(alpha: .42), Pop.well);
Color get _lyGapC => Color.alphaBlend(Pop.toneTime.withValues(alpha: .20), Grey.g10);
Color get _lyGapHot => Color.alphaBlend(Pop.toneTime.withValues(alpha: .45), Grey.g10);
Color get _lyMarC => Color.alphaBlend(Pop.tonePlace.withValues(alpha: .28), Grey.g10);
Color get _lyMarHot => Color.alphaBlend(Pop.tonePlace.withValues(alpha: .5), Grey.g10);
Color get _lyKidSel => Color.alphaBlend(Pop.toneLayout.withValues(alpha: .35), Grey.g26);

/// A mock child: its name and its content size in px.
class _LyKid {
  const _LyKid(this.name, this.w, this.h);
  final String name;
  final double w, h;
}

// ---- solver ---------------------------------------------------------------------------------------------------------
/// One child as the solver reads it. [sz] is the main-axis sizing (Hug = content, Fill = flex 1 1 0, Fixed = [fw]).
class _LyIt {
  const _LyIt(this.w, this.h, {this.sz = 'Hug', this.fw = 0, this.m = 0, this.self = 'Auto'});
  final double w, h, fw, m;
  final String sz, self;
}

/// Solved boxes (container space, px, padding included) and the gutters between them; a gutter's bool = it is a vertical strip.
typedef _LyOut = ({List<Rect> boxes, List<(Rect, bool)> gutters});

/// contract: CSS flexbox for solid boxes: min-size auto = content, so nothing shrinks below its content (it overflows instead);
/// Fill grows from 0 but not below content; lines stretch to share the cross axis (align-content normal); Space * falls back to Start when overflowing.
_LyOut _lyFlex(
  Size box,
  double px,
  double py,
  List<_LyIt> its, {
  bool col = false,
  bool wrap = false,
  double gap = 0,
  String justify = 'Start',
  String align = 'Stretch',
}) {
  final ms = math.max(0.0, (col ? box.height - 2 * py : box.width - 2 * px)), cs = math.max(0.0, col ? box.width - 2 * px : box.height - 2 * py);
  double mainOf(_LyIt it) => col ? it.h : it.w;
  double crossOf(_LyIt it) => col ? it.w : it.h;
  final basis = [for (final it in its) it.sz == 'Fixed' ? it.fw : mainOf(it)];
  final lines = <List<int>>[[]];
  var used = 0.0;
  for (var i = 0; i < its.length; i++) {
    final o = basis[i] + 2 * its[i].m;
    if (wrap && lines.last.isNotEmpty && used + gap + o > ms) {
      lines.add([]);
      used = 0;
    }
    used += (lines.last.isEmpty ? 0 : gap) + o;
    lines.last.add(i);
  }
  final size = List<double>.of(basis), lineC = <double>[];
  final mainAt = List<double>.filled(its.length, 0), crossAt = List<double>.filled(its.length, 0), crossSz = List<double>.filled(its.length, 0);
  final gut = <(double, double, int)>[];
  for (final l in lines) {
    final fills = [
      for (final i in l)
        if (its[i].sz == 'Fill') i,
    ];
    var rest = ms - gap * (l.length - 1);
    for (final i in l) {
      rest -= 2 * its[i].m + (its[i].sz == 'Fill' ? 0 : size[i]);
    }
    final open = [...fills];
    while (open.isNotEmpty) {
      final share = rest / open.length,
          low = [
            for (final i in open)
              if (mainOf(its[i]) > share) i,
          ];
      if (low.isEmpty) {
        for (final i in open) {
          size[i] = share;
        }
        break;
      }
      for (final i in low) {
        size[i] = mainOf(its[i]);
        rest -= size[i];
        open.remove(i);
      }
    }
    var free = ms - gap * (l.length - 1);
    for (final i in l) {
      free -= size[i] + 2 * its[i].m;
    }
    final n = l.length;
    var lead = 0.0, sp = gap;
    final spread = justify.startsWith('Space') && free < 0 ? 'Start' : justify;
    switch (spread) {
      case 'End':
        lead = free;
      case 'Center':
        lead = free / 2;
      case 'Space Between':
        if (n > 1) sp = gap + free / (n - 1);
      case 'Space Around':
        lead = free / n / 2;
        sp = gap + free / n;
      case 'Space Evenly':
        lead = free / (n + 1);
        sp = gap + free / (n + 1);
    }
    var p = lead;
    for (final (k, i) in l.indexed) {
      if (k > 0) gut.add((p - sp, p, lineC.length));
      mainAt[i] = p + its[i].m;
      p += size[i] + 2 * its[i].m + sp;
    }
    lineC.add(l.fold(0.0, (a, i) => math.max(a, crossOf(its[i]) + 2 * its[i].m)));
  }
  if (lines.length == 1 && !wrap) {
    lineC[0] = cs;
  } else {
    final extra = cs - lineC.fold(0.0, (a, b) => a + b) - gap * (lines.length - 1);
    for (var k = 0; k < lineC.length && extra > 0; k++) {
      lineC[k] += extra / lines.length;
    }
  }
  final lineAt = <double>[];
  var c = 0.0;
  for (var k = 0; k < lines.length; k++) {
    lineAt.add(c);
    for (final i in lines[k]) {
      final it = its[i], a = it.self == 'Auto' ? align : it.self, own = crossOf(it);
      crossSz[i] = a == 'Stretch' ? math.max(0, lineC[k] - 2 * it.m) : own;
      crossAt[i] = switch (a) {
        'End' => c + lineC[k] - it.m - own,
        'Center' => c + (lineC[k] - own) / 2,
        _ => c + it.m,
      };
    }
    c += lineC[k] + gap;
  }
  Rect at(double m, double cr, double mw, double cw) => col ? Rect.fromLTWH(px + cr, py + m, cw, mw) : Rect.fromLTWH(px + m, py + cr, mw, cw);
  return (
    boxes: [for (var i = 0; i < its.length; i++) at(mainAt[i], crossAt[i], size[i], crossSz[i])],
    gutters: [
      for (final (a, b, k) in gut) (at(a, lineAt[k], b - a, lineC[k]), !col),
      for (var k = 1; k < lines.length; k++) (at(0, lineAt[k] - gap, ms, gap), col),
    ],
  );
}

/// A grid track: 'fr', 'px' or 'auto', and its value.
class _LyTr {
  const _LyTr(this.k, this.v);
  final String k;
  final double v;
  String get word => switch (k) {
    'px' => '${v.round()}',
    'auto' => 'auto',
    _ => '${trimZeros(v.toStringAsFixed(1))}fr',
  };
}

/// contract: CSS track sizing without spanning items: px fixed, auto = largest single-span content (24 px floor in this lab so an
/// empty track stays grabbable), fr shares what is left (a fr sum under 1 takes only that share), auto tracks stretch when there is no fr.
List<double> _lyTracks(List<_LyTr> ts, double avail, double gap, List<double> content) {
  final out = [for (final (i, t) in ts.indexed) t.k == 'px' ? t.v : (t.k == 'auto' ? math.max(24.0, content[i]) : 0.0)];
  final left = avail - gap * (ts.length - 1) - out.fold(0.0, (a, b) => a + b);
  final fr = ts.where((t) => t.k == 'fr').fold(0.0, (a, t) => a + t.v), autos = ts.where((t) => t.k == 'auto').length;
  if (left <= 0) return out;
  for (final (i, t) in ts.indexed) {
    if (fr > 0 && t.k == 'fr') out[i] = t.v * left / math.max(fr, 1);
    if (fr == 0 && t.k == 'auto') out[i] += left / autos;
  }
  return out;
}

/// contract: CSS grid auto-placement, sparse: both lines definite first, then row-locked items at the earliest free column,
/// then the rest in order with one cursor (a definite column moves the cursor down a row when it lies left of it). Starts are 1-based, 0 = auto.
List<(int, int, int, int)> _lyPlace(int cols, List<(int, int, int, int)> want) {
  final busy = <(int, int)>{}, out = List<(int, int, int, int)?>.filled(want.length, null);
  bool fits(int c, int r, int cs, int rs) {
    if (c + cs > cols) return false;
    for (var y = r; y < r + rs; y++) {
      for (var x = c; x < c + cs; x++) {
        if (busy.contains((x, y))) return false;
      }
    }
    return true;
  }

  void put(int i, int c, int r, int cs, int rs) {
    out[i] = (c, r, cs, rs);
    for (var y = r; y < r + rs; y++) {
      for (var x = c; x < c + cs; x++) {
        busy.add((x, y));
      }
    }
  }

  (int, int, int, int) norm((int, int, int, int) w) {
    final cs = w.$3.clamp(1, cols);
    return (w.$1 > 0 ? math.min(w.$1 - 1, cols - cs) : -1, w.$2 - 1, cs, math.max(1, w.$4));
  }

  for (final (i, w) in want.indexed) {
    final (c, r, cs, rs) = norm(w);
    if (c >= 0 && r >= 0) put(i, c, r, cs, rs);
  }
  for (final (i, w) in want.indexed) {
    final (c, r, cs, rs) = norm(w);
    if (c >= 0 || r < 0) continue;
    var x = 0;
    while (x < cols - cs && !fits(x, r, cs, rs)) {
      x++;
    }
    put(i, x, r, cs, rs);
  }
  var cr = 0, cc = 0;
  for (final (i, w) in want.indexed) {
    final (c, r, cs, rs) = norm(w);
    if (r >= 0) continue;
    if (c >= 0) {
      if (c < cc) cr++;
      while (!fits(c, cr, cs, rs)) {
        cr++;
      }
      put(i, c, cr, cs, rs);
      cc = c + cs;
      continue;
    }
    while (true) {
      if (cc + cs > cols) {
        cc = 0;
        cr++;
      }
      if (fits(cc, cr, cs, rs)) break;
      cc++;
    }
    put(i, cc, cr, cs, rs);
    cc += cs;
  }
  return [for (final o in out) o!];
}

/// A solved grid: boxes and gutters, the tracks' starts and sizes (container space) and where each item landed (0-based).
typedef _LyGridOut = ({
  List<Rect> boxes,
  List<(Rect, bool)> gutters,
  List<double> xs,
  List<double> ws,
  List<double> ys,
  List<double> hs,
  List<(int, int, int, int)> at,
});

_LyGridOut _lyGrid(
  Size box,
  double px,
  double py,
  double gap,
  List<_LyTr> cols,
  List<_LyTr> rows,
  List<_LyIt> its,
  List<(int, int, int, int)> want, {
  String align = 'Stretch',
}) {
  final at = _lyPlace(cols.length, want);
  final nRows = at.fold(rows.length, (a, p) => math.max(a, p.$2 + p.$4));
  final rs = [...rows, for (var j = rows.length; j < nRows; j++) const _LyTr('auto', 0)];
  final cw = List<double>.filled(cols.length, 0), rh = List<double>.filled(rs.length, 0);
  for (final (i, p) in at.indexed) {
    if (p.$3 == 1) cw[p.$1] = math.max(cw[p.$1], its[i].w + 2 * its[i].m);
    if (p.$4 == 1) rh[p.$2] = math.max(rh[p.$2], its[i].h + 2 * its[i].m);
  }
  final ws = _lyTracks(cols, box.width - 2 * px, gap, cw), hs = _lyTracks(rs, box.height - 2 * py, gap, rh);
  List<double> starts(List<double> s, double from) {
    final out = <double>[];
    for (final v in s) {
      out.add(from);
      from += v + gap;
    }
    return out;
  }

  final xs = starts(ws, px), ys = starts(hs, py);
  final boxes = <Rect>[];
  for (final (i, p) in at.indexed) {
    final it = its[i];
    final area = Rect.fromLTRB(xs[p.$1], ys[p.$2], xs[p.$1 + p.$3 - 1] + ws[p.$1 + p.$3 - 1], ys[p.$2 + p.$4 - 1] + hs[p.$2 + p.$4 - 1]).deflate(it.m);
    final a = it.self == 'Auto' ? align : it.self;
    if (a == 'Stretch') {
      boxes.add(area);
    } else {
      final w = math.min(it.w, area.width), h = math.min(it.h, area.height);
      final y = switch (a) {
        'End' => area.bottom - h,
        'Center' => area.center.dy - h / 2,
        _ => area.top,
      };
      boxes.add(Rect.fromLTWH(area.center.dx - w / 2, y, w, h));
    }
  }
  final bottom = ys.last + hs.last, right = xs.last + ws.last;
  return (
    boxes: boxes,
    gutters: [
      for (var i = 0; i < xs.length - 1; i++) (Rect.fromLTRB(xs[i] + ws[i], py, xs[i + 1], bottom), true),
      for (var j = 0; j < ys.length - 1; j++) (Rect.fromLTRB(px, ys[j] + hs[j], right, ys[j + 1]), false),
    ],
    xs: xs,
    ws: ws,
    ys: ys,
    hs: hs,
    at: at,
  );
}

// ---- shared stage ---------------------------------------------------------------------------------------------------
/// What a press on a stage landed on: [drag] gets the total offset since the press (one undo per drag), [tap] a press without a move.
class _LyHit {
  const _LyHit(this.id, {this.cursor = SystemMouseCursors.click, this.drag, this.tap});
  final String id;
  final MouseCursor cursor;
  final void Function(Offset total)? drag;
  final VoidCallback? tap;
}

/// A small in-card stage: hit-tests on hover (repaints only when the hovered part changes) and turns presses into taps or drags.
class _LyStage extends StatefulWidget {
  const _LyStage({super.key, required this.height, required this.hit, required this.paint, this.onEmpty});
  final double height;
  final _LyHit? Function(Offset p) hit;
  final CustomPainter Function(String? hover, String? active) paint;
  final void Function(Offset p)? onEmpty;
  @override
  State<_LyStage> createState() => _LyStageState();
}

class _LyStageState extends State<_LyStage> {
  String? _hover, _active;
  MouseCursor _cursor = MouseCursor.defer;
  _LyHit? _h;
  Offset? _down;
  bool _moved = false;

  void _end(Doc doc, {bool cancel = false}) {
    if (_moved) {
      cancel ? doc.cancelGesture() : doc.endGesture();
      setState(() => _active = null);
    }
    _down = null;
    _h = null;
    _moved = false;
  }

  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, on = !x.cfg.locked;
    return MouseRegion(
      cursor: _active != null ? (_h?.cursor ?? _cursor) : _cursor,
      onHover: (e) {
        final h = on ? widget.hit(e.localPosition) : null;
        if (h?.id != _hover) {
          setState(() {
            _hover = h?.id;
            _cursor = h?.cursor ?? MouseCursor.defer;
          });
        }
      },
      onExit: (_) {
        if (_hover != null) setState(() => _hover = null);
      },
      child: Listener(
        behavior: HitTestBehavior.opaque,
        onPointerDown: (e) {
          if (!on || e.buttons != kPrimaryButton) return;
          _h = widget.hit(e.localPosition);
          _down = e.localPosition;
          _moved = false;
        },
        onPointerMove: (e) {
          final d = _down, h = _h;
          if (d == null || h?.drag == null) return;
          final t = e.localPosition - d;
          if (!_moved) {
            if (t.distance < 3) return;
            _moved = true;
            doc.beginGesture();
            setState(() => _active = h!.id);
          }
          h!.drag!(t);
        },
        onPointerUp: (e) {
          final d = _down, h = _h;
          if (!_moved && d != null) h == null ? widget.onEmpty?.call(d) : h.tap?.call();
          _end(doc);
        },
        onPointerCancel: (_) => _end(doc, cancel: true),
        child: SizedBox(
          height: widget.height,
          width: double.infinity,
          child: CustomPaint(painter: widget.paint(_hover, _active)),
        ),
      ),
    );
  }
}

Paint _lyLine(Color c, [double w = 1]) => Paint()
  ..color = c
  ..style = PaintingStyle.stroke
  ..strokeWidth = w;

void _lyText(Canvas c, String s, Offset centre, TextStyle st, {Color? bg, double? maxW}) {
  final tp = TextPainter(
    text: TextSpan(text: s, style: st),
    textDirection: TextDirection.ltr,
    maxLines: 1,
    ellipsis: '…',
  )..layout(maxWidth: maxW ?? 400);
  if (bg != null) {
    c.drawRRect(
      RRect.fromRectAndRadius(Rect.fromCenter(center: centre, width: tp.width + 8, height: tp.height + 4), const Radius.circular(4)),
      Paint()..color = bg,
    );
  }
  tp.paint(c, centre - Offset(tp.width / 2, tp.height / 2));
}

void _lyKidBox(Canvas c, Rect r, String name, {bool sel = false, bool hot = false}) {
  final rr = RRect.fromRectAndRadius(r, const Radius.circular(3));
  c.drawRRect(rr, Paint()..color = sel ? _lyKidSel : (hot ? Grey.g38 : Grey.g26));
  c.drawRRect(rr, sel ? _lyLine(Pop.accentInk, 1.4) : _lyLine(Grey.g38));
  if (r.width > 22 && r.height > 11) _lyText(c, name, r.center, T.label(sel ? Grey.g100 : Grey.g91), maxW: r.width - 4);
}

double _lyG(Doc d, String id) => d.get(id) ?? 0;
