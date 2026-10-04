part of 'inspector_parts.dart';

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

/// A rounded flat chip that toggles or acts.
class _LyChip extends StatelessWidget {
  const _LyChip(this.label, {super.key, this.on = false, this.onTap, this.lead});
  final String label;
  final bool on;
  final VoidCallback? onTap;
  final Widget? lead;
  @override
  Widget build(BuildContext context) => Hov(
    onTap: Ctx.of(context).cfg.locked ? null : onTap,
    builder: (_, h) => Container(
      height: Pop.cell,
      padding: const EdgeInsets.symmetric(horizontal: 8),
      decoration: BoxDecoration(
        color: on ? Grey.g20 : (h ? Grey.g20 : Pop.well),
        borderRadius: BorderRadius.circular(6),
        border: Border(bottom: BorderSide(color: on ? Pop.accentInk : const Color(0x00000000))),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          if (lead != null) ...[lead!, const SizedBox(width: 5)],
          Text(label, style: T.label(on || h ? Grey.g95 : Grey.g63)),
        ],
      ),
    ),
  );
}

Widget _lyCells(List<Widget> cs, {List<int>? flex}) => Padding(
  padding: const EdgeInsets.only(top: 4),
  child: Row(
    children: [
      for (var i = 0; i < cs.length; i++) ...[if (i > 0) const SizedBox(width: 4), Expanded(flex: flex?[i] ?? 1, child: cs[i])],
    ],
  ),
);

Widget _lySeg(Doc doc, String id, List<String> items, {List<String>? words}) => SizedBox(
  height: Pop.cell,
  child: Dis(
    child: Segmented(
      items: words ?? items,
      index: math.max(0, items.indexOf(doc.s2[id] ?? items.first)),
      expand: true,
      onChanged: (i) => doc.str(id, items[i]),
    ),
  ),
);

Widget _lyPick(String id, List<String> options, String word) => SizedBox(
  height: Pop.cell,
  child: Chooser(id: id, options: options, word: word),
);

Widget _lyNum(
  String id,
  String label, {
  double min = 0,
  double max = 100000,
  double perPx = 1,
  String unit = '',
  bool enabled = true,
  String? zero,
  int dec = 0,
  double step = 1,
}) => NumField(id: id, label: label, unit: unit, min: min, max: max, perPx: perPx, enabled: enabled, zeroWord: zero, decimals: dec, step: step);

/// The card shell every idea wears, folding on its own flag so the three never fold together.
Widget _lyCard(Doc doc, String id, {required List<String> brief, required List<Widget> children}) {
  final open = doc.b['$id.open'] ?? true;
  return Sect(
    title: 'Layout',
    tone: Pop.toneLayout,
    mark: CardMark.layout,
    open: open,
    onToggle: () => doc.flag('$id.open', !open),
    brief: brief,
    children: children,
  );
}

class LayoutIdeas extends StatelessWidget {
  const LayoutIdeas({super.key});
  @override
  Widget build(BuildContext context) => const Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    mainAxisSize: MainAxisSize.min,
    children: [
      IdeaLabel('A', 'Box model', 'Padding, gap and margin are bands you grab on the solved box; values sit where they act.'),
      _LyA(),
      IdeaLabel('B', 'Grid tracks', 'Size columns and rows on their headers; drag a child across cells to place and span it.'),
      _LyB(),
      IdeaLabel('C', 'Flex playground', 'Click where children should gather, rotate the flow, narrow the box and watch it wrap.'),
      _LyC(),
    ],
  );
}

// ---- A: box model ---------------------------------------------------------------------------------------------------
const _lyAKids = [_LyKid('Logo', 64, 56), _LyKid('Title', 132, 36), _LyKid('Body', 96, 72), _LyKid('Tag', 52, 28)];

/// The stage's fixed scale: the widest box (560 px) fills the stage, so resizing the box is seen as resizing.
const _lyAMaxW = 560.0, _lyAMaxH = 260.0;

class _LyA extends StatelessWidget {
  const _LyA();
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    doc.seed(
      {
        'ly_a_w': 480,
        'ly_a_h': 200,
        'ly_a_px': 20,
        'ly_a_py': 16,
        'ly_a_gap': 12,
        'ly_a_cols': 2,
        for (final (i, k) in _lyAKids.indexed) ...{'ly_a_k$i.m': i == 2 ? 8 : 0, 'ly_a_k$i.fw': k.w},
      },
      strs: {
        'ly_a_mode': 'Row',
        'ly_a_justify': 'Start',
        'ly_a_align': 'Center',
        'ly_a_sel': '2',
        for (var i = 0; i < _lyAKids.length; i++) ...{'ly_a_k$i.sz': i == 1 ? 'Fill' : 'Hug', 'ly_a_k$i.self': 'Auto'},
      },
    );
    final mode = doc.s2['ly_a_mode'] ?? 'Row', grid = mode == 'Grid';
    final sel = int.tryParse(doc.s2['ly_a_sel'] ?? '');
    final k = 'ly_a_k$sel';
    return _lyCard(
      doc,
      'ly_a',
      brief: [
        grid ? 'Grid ${brief(doc.get('ly_a_cols'), 0)} cols' : mode,
        'Pad ${brief(doc.get('ly_a_px'), 0)} · ${brief(doc.get('ly_a_py'), 0)}',
        'Gap ${brief(doc.get('ly_a_gap'), 0)}',
      ],
      children: [
        const _LyAStage(),
        _lyCells(
          [
            _lySeg(doc, 'ly_a_mode', const ['Row', 'Column', 'Grid']),
            grid ? _lyNum('ly_a_cols', 'Cols', min: 1, max: 6, perPx: .1) : _lyPick('ly_a_justify', _lyJustify, 'Justify'),
          ],
          flex: const [3, 2],
        ),
        _lyCells([_lySeg(doc, 'ly_a_align', _lyAlign)]),
        _lyCells(
          [
            _lyNum('ly_a_w', 'W', min: 160, max: _lyAMaxW),
            _lyNum('ly_a_h', 'H', min: 80, max: _lyAMaxH),
            _lyNum('ly_a_px', 'Pad X', max: 64),
            _lyNum('ly_a_py', 'Pad Y', max: 64),
            _lyNum('ly_a_gap', 'Gap', max: 64),
          ],
          flex: const [4, 4, 5, 5, 4],
        ),
        if (sel != null && sel < _lyAKids.length) ...[
          Padding(
            padding: const EdgeInsets.only(top: 8),
            child: Row(
              children: [
                Text(_lyAKids[sel].name, style: T.name(Grey.g95).copyWith(fontWeight: FontWeight.w700)),
                const SizedBox(width: 6),
                Expanded(
                  child: Text('as a child · click empty space to leave', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(Grey.g56)),
                ),
              ],
            ),
          ),
          _lyCells(
            [_lySeg(doc, '$k.sz', _lySizing), _lyNum('$k.fw', mode == 'Column' ? 'H' : 'W', min: 8, max: 400, enabled: !grid && doc.s2['$k.sz'] == 'Fixed')],
            flex: const [3, 2],
          ),
          _lyCells([_lyPick('$k.self', _lySelf, 'Align self'), _lyNum('$k.m', 'Margin', max: 64)]),
        ],
      ],
    );
  }
}

_LyOut _lyASolve(Doc doc) {
  final mode = doc.s2['ly_a_mode'] ?? 'Row', box = Size(_lyG(doc, 'ly_a_w'), _lyG(doc, 'ly_a_h'));
  final its = [
    for (final (i, k) in _lyAKids.indexed)
      _LyIt(k.w, k.h, sz: doc.s2['ly_a_k$i.sz'] ?? 'Hug', fw: _lyG(doc, 'ly_a_k$i.fw'), m: _lyG(doc, 'ly_a_k$i.m'), self: doc.s2['ly_a_k$i.self'] ?? 'Auto'),
  ];
  final px = _lyG(doc, 'ly_a_px'), py = _lyG(doc, 'ly_a_py'), gap = _lyG(doc, 'ly_a_gap'), align = doc.s2['ly_a_align'] ?? 'Stretch';
  if (mode == 'Grid') {
    final n = _lyG(doc, 'ly_a_cols').round().clamp(1, 6);
    final g = _lyGrid(box, px, py, gap, List.filled(n, const _LyTr('fr', 1)), const [], its, [for (final _ in its) (0, 0, 1, 1)], align: align);
    return (boxes: g.boxes, gutters: g.gutters);
  }
  return _lyFlex(box, px, py, its, col: mode == 'Column', gap: gap, justify: doc.s2['ly_a_justify'] ?? 'Start', align: align);
}

class _LyAStage extends StatelessWidget {
  const _LyAStage();
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    return LayoutBuilder(
      builder: (context, box) {
        final s = box.maxWidth / _lyAMaxW, out = _lyASolve(doc);
        final w = _lyG(doc, 'ly_a_w'), h = _lyG(doc, 'ly_a_h'), px = _lyG(doc, 'ly_a_px'), py = _lyG(doc, 'ly_a_py'), gap = _lyG(doc, 'ly_a_gap');
        final sel = int.tryParse(doc.s2['ly_a_sel'] ?? '');
        final outer = Rect.fromLTWH(0, 0, w * s, h * s), inner = Rect.fromLTRB(px * s, py * s, (w - px) * s, (h - py) * s);
        Rect sc(Rect r) => Rect.fromLTRB(r.left * s, r.top * s, r.right * s, r.bottom * s);
        final kids = [for (final b in out.boxes) sc(b)];
        final guts = [for (final (r, v) in out.gutters) (sc(r), v)];
        final m = sel == null ? 0.0 : _lyG(doc, 'ly_a_k$sel.m');
        void set(String id, double v, double lo, double hi) => doc.set(id, v.roundToDouble().clamp(lo, hi));
        _LyHit? hit(Offset p) {
          if ((p - outer.bottomRight).distance < 8) {
            return _LyHit(
              'size',
              cursor: SystemMouseCursors.resizeUpLeftDownRight,
              drag: (t) =>
                  doc.setMany({'ly_a_w': (w + t.dx / s).roundToDouble().clamp(160, _lyAMaxW), 'ly_a_h': (h + t.dy / s).roundToDouble().clamp(80, _lyAMaxH)}),
            );
          }
          if (sel != null && sel < kids.length) {
            final k = kids[sel], ring = k.inflate(math.max(m * s, 5));
            if (ring.contains(p) && !k.contains(p)) {
              final sign = p.dx < k.center.dx ? -1 : 1;
              return _LyHit('margin', cursor: SystemMouseCursors.resizeLeftRight, drag: (t) => set('ly_a_k$sel.m', m + sign * t.dx / s, 0, 64));
            }
          }
          for (final (i, k) in kids.indexed) {
            if (k.contains(p)) return _LyHit('k$i', tap: () => doc.str('ly_a_sel', sel == i ? '' : '$i'));
          }
          for (final (r, v) in guts) {
            final hr = v
                ? Rect.fromCenter(center: r.center, width: math.max(r.width, 6), height: r.height)
                : Rect.fromCenter(center: r.center, width: r.width, height: math.max(r.height, 6));
            if (hr.contains(p)) {
              return _LyHit(
                'gap',
                cursor: v ? SystemMouseCursors.resizeLeftRight : SystemMouseCursors.resizeUpDown,
                drag: (t) => set('ly_a_gap', gap + (v ? t.dx : t.dy) / s, 0, 64),
              );
            }
          }
          if (outer.contains(p) && !inner.deflate(4).contains(p)) {
            if (p.dx < inner.left + 4) return _LyHit('padX', cursor: SystemMouseCursors.resizeLeftRight, drag: (t) => set('ly_a_px', px + t.dx / s, 0, 64));
            if (p.dx > inner.right - 4) return _LyHit('padX', cursor: SystemMouseCursors.resizeLeftRight, drag: (t) => set('ly_a_px', px - t.dx / s, 0, 64));
            if (p.dy < inner.top + 4) return _LyHit('padY', cursor: SystemMouseCursors.resizeUpDown, drag: (t) => set('ly_a_py', py + t.dy / s, 0, 64));
            return _LyHit('padY', cursor: SystemMouseCursors.resizeUpDown, drag: (t) => set('ly_a_py', py - t.dy / s, 0, 64));
          }
          return null;
        }

        final sig = '$w $h $px $py $gap $sel $m ${box.maxWidth} ${kids.join()}';
        return Padding(
          padding: const EdgeInsets.only(top: 2),
          child: _LyStage(
            key: const ValueKey('ly_a_stage'),
            height: _lyAMaxH * s + 14,
            hit: hit,
            onEmpty: (_) => doc.str('ly_a_sel', ''),
            paint: (hov, act) => _LyAPaint(sig, outer, inner, kids, guts, sel, m * s, (px: px, py: py, gap: gap, m: m, w: w, h: h), hov ?? act, act),
          ),
        );
      },
    );
  }
}

class _LyAPaint extends CustomPainter {
  const _LyAPaint(this.sig, this.outer, this.inner, this.kids, this.guts, this.sel, this.ms, this.v, this.hot, this.act);
  final String sig;
  final Rect outer, inner;
  final List<Rect> kids;
  final List<(Rect, bool)> guts;
  final int? sel;
  final double ms;
  final ({double px, double py, double gap, double m, double w, double h}) v;
  final String? hot, act;
  @override
  void paint(Canvas c, Size s) {
    c.clipRect(Offset.zero & s);
    final px = hot == 'padX', py = hot == 'padY';
    final band = Path()
      ..fillType = PathFillType.evenOdd
      ..addRect(outer)
      ..addRect(inner);
    c.drawRect(outer, Paint()..color = _lyPadC);
    if (px || py) {
      c.save();
      c.clipPath(band);
      c.drawRect(
        px ? Rect.fromLTRB(outer.left, outer.top, inner.left, outer.bottom) : Rect.fromLTRB(outer.left, outer.top, outer.right, inner.top),
        Paint()..color = _lyPadHot,
      );
      c.drawRect(
        px ? Rect.fromLTRB(inner.right, outer.top, outer.right, outer.bottom) : Rect.fromLTRB(outer.left, inner.bottom, outer.right, outer.bottom),
        Paint()..color = _lyPadHot,
      );
      c.restore();
    }
    c.drawRect(inner, Paint()..color = Grey.g10);
    c.drawRect(inner, _lyLine(Grey.g20));
    for (final (r, _) in guts) {
      c.drawRect(r, Paint()..color = hot == 'gap' ? _lyGapHot : _lyGapC);
    }
    final i = sel;
    if (i != null && i < kids.length) {
      final k = kids[i], ring = k.inflate(ms);
      if (ms > 0) c.drawRRect(RRect.fromRectAndRadius(ring, const Radius.circular(3)), Paint()..color = hot == 'margin' ? _lyMarHot : _lyMarC);
      c.drawRRect(
        RRect.fromRectAndRadius(k.inflate(math.max(ms, 4)), const Radius.circular(4)),
        _lyLine(hot == 'margin' ? Pop.tonePlace : Pop.tonePlace.withValues(alpha: .5)),
      );
    }
    for (final (j, k) in kids.indexed) {
      _lyKidBox(c, k, _lyAKids[j].name, sel: j == sel, hot: hot == 'k$j');
    }
    c.drawRect(outer, _lyLine(Grey.g44));
    final ink = T.micro(Grey.g95).copyWith(fontWeight: FontWeight.w700);
    final mid = outer.center;
    _lyText(c, brief(v.px, 0), Offset((outer.left + inner.left) / 2, mid.dy), ink);
    _lyText(c, brief(v.px, 0), Offset((outer.right + inner.right) / 2, mid.dy), ink);
    _lyText(c, brief(v.py, 0), Offset(mid.dx, (outer.top + inner.top) / 2), ink);
    _lyText(c, brief(v.py, 0), Offset(mid.dx, (outer.bottom + inner.bottom) / 2), ink);
    if (guts.isNotEmpty) _lyText(c, brief(v.gap, 0), guts.first.$1.center, T.micro(Grey.g07).copyWith(fontWeight: FontWeight.w700), bg: Pop.toneTime);
    if (i != null && i < kids.length) {
      _lyText(c, 'm ${brief(v.m, 0)}', kids[i].inflate(math.max(ms, 4)).topCenter, T.micro(Grey.g07).copyWith(fontWeight: FontWeight.w700), bg: Pop.tonePlace);
    }
    final h = outer.bottomRight;
    c.drawRect(Rect.fromCenter(center: h, width: 7, height: 7), Paint()..color = hot == 'size' ? Grey.g95 : Grey.g63);
    _lyText(c, '${brief(v.w, 0)} × ${brief(v.h, 0)}', Offset(math.max(h.dx - 30, 32), h.dy + 8), T.micro(hot == 'size' ? Grey.g95 : Grey.g56));
  }

  @override
  bool shouldRepaint(_LyAPaint o) => o.sig != sig || o.hot != hot || o.act != act;
}

// ---- B: grid tracks -------------------------------------------------------------------------------------------------
const _lyBKids = [_LyKid('Logo', 56, 44), _LyKid('Title', 120, 32), _LyKid('Body', 96, 56), _LyKid('Tag', 48, 24), _LyKid('Button', 80, 32)];
const _lyBMaxC = 6, _lyBMaxR = 5, _lyBHdrW = 34.0, _lyBHdrH = 18.0;
const _lyBKinds = ['fr', 'px', 'auto'];

class _LyB extends StatelessWidget {
  const _LyB();
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    doc.seed(
      {
        'ly_b_w': 480,
        'ly_b_h': 240,
        'ly_b_pad': 12,
        'ly_b_gap': 8,
        'ly_b_cols': 3,
        'ly_b_rows': 3,
        for (var i = 0; i < _lyBMaxC; i++) 'ly_b_c$i': i < 3 ? const [1.0, 2.0, 96.0][i] : 1.0,
        for (var j = 0; j < _lyBMaxR; j++) 'ly_b_r$j': 1,
        for (final (i, p) in const [(1, 1, 1, 2), (2, 1, 2, 1), (0, 0, 1, 1), (0, 0, 1, 1), (3, 3, 1, 1)].indexed) ...{
          'ly_b_k$i.c': p.$1.toDouble(),
          'ly_b_k$i.r': p.$2.toDouble(),
          'ly_b_k$i.cs': p.$3.toDouble(),
          'ly_b_k$i.rs': p.$4.toDouble(),
        },
      },
      strs: {
        for (var i = 0; i < _lyBMaxC; i++) 'ly_b_c$i.k': i == 2 ? 'px' : 'fr',
        for (var j = 0; j < _lyBMaxR; j++) 'ly_b_r$j.k': j == 0 ? 'auto' : 'fr',
        'ly_b_sel': '1',
        'ly_b_track': 'c1',
      },
    );
    final g = _lyBSolve(doc), nc = _lyBCount(doc, 'cols'), nr = _lyBCount(doc, 'rows');
    final sel = int.tryParse(doc.s2['ly_b_sel'] ?? ''), k = 'ly_b_k$sel';
    final tr = doc.s2['ly_b_track'] ?? '', col = tr.startsWith('c'), ti = int.tryParse(tr.substring(math.min(1, tr.length))) ?? -1;
    final tOk = ti >= 0 && ti < (col ? nc : nr), tid = 'ly_b_$tr';
    return _lyCard(
      doc,
      'ly_b',
      brief: [
        'Grid $nc×$nr',
        [for (var i = 0; i < nc; i++) _lyBTrack(doc, 'c$i').word].join(' '),
        'Gap ${brief(doc.get('ly_b_gap'), 0)}',
      ],
      children: [
        _LyBStage(g),
        _lyCells([
          _lyNum('ly_b_cols', 'Cols', min: 1, max: _lyBMaxC.toDouble(), perPx: .1),
          _lyNum('ly_b_rows', 'Rows', min: 1, max: _lyBMaxR.toDouble(), perPx: .1),
          _lyNum('ly_b_gap', 'Gap', max: 48),
        ]),
        if (tOk)
          _lyCells(
            [
              SizedBox(
                height: Pop.cell,
                child: Align(
                  alignment: Alignment.centerLeft,
                  child: Text('${col ? 'Column' : 'Row'} ${ti + 1}', style: T.name(Grey.g91).copyWith(fontWeight: FontWeight.w700)),
                ),
              ),
              _lyBKindSeg(doc, tr, col ? g.ws[ti] : g.hs[ti]),
              _lyNum(
                tid,
                doc.s2['$tid.k'] == 'px' ? 'px' : 'fr',
                min: doc.s2['$tid.k'] == 'px' ? 8 : .1,
                max: doc.s2['$tid.k'] == 'px' ? 480 : 12,
                dec: doc.s2['$tid.k'] == 'px' ? 0 : 1,
                perPx: doc.s2['$tid.k'] == 'px' ? 1 : .02,
                step: doc.s2['$tid.k'] == 'px' ? 1 : .1,
                enabled: doc.s2['$tid.k'] != 'auto',
              ),
            ],
            flex: const [3, 4, 3],
          ),
        if (sel != null && sel < _lyBKids.length) ...[
          Padding(
            padding: const EdgeInsets.only(top: 8),
            child: Row(
              children: [
                Text(_lyBKids[sel].name, style: T.name(Grey.g95).copyWith(fontWeight: FontWeight.w700)),
                const SizedBox(width: 6),
                Expanded(
                  child: Text(
                    'col ${g.at[sel].$1 + 1} · row ${g.at[sel].$2 + 1} · ${g.at[sel].$3}×${g.at[sel].$4}',
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: T.label(Grey.g56),
                  ),
                ),
                _LyChip('Auto place', onTap: () => doc.setMany({'$k.c': 0, '$k.r': 0})),
              ],
            ),
          ),
          _lyCells([
            _lyNum('$k.c', 'Col', max: nc.toDouble(), perPx: .1, zero: 'Auto'),
            _lyNum('$k.r', 'Row', max: 8, perPx: .1, zero: 'Auto'),
            _lyNum('$k.cs', 'Span', min: 1, max: nc.toDouble(), perPx: .1),
            _lyNum('$k.rs', 'Rows', min: 1, max: 5, perPx: .1),
          ]),
        ],
      ],
    );
  }
}

int _lyBCount(Doc doc, String n) => _lyG(doc, 'ly_b_$n').round().clamp(1, n == 'cols' ? _lyBMaxC : _lyBMaxR);
_LyTr _lyBTrack(Doc doc, String t) => _LyTr(doc.s2['ly_b_$t.k'] ?? 'fr', _lyG(doc, 'ly_b_$t'));

/// contract: switching a track's kind keeps its current width where it can (fr -> px takes the solved px; anything -> fr starts at 1).
void _lyBSetKind(Doc doc, String t, String kind, double px) {
  final id = 'ly_b_$t';
  if (doc.s2['$id.k'] == kind) return;
  doc.s2['$id.k'] = kind;
  doc.set(id, kind == 'px' ? px.roundToDouble().clamp(8, 480) : (kind == 'fr' ? 1 : _lyG(doc, id)));
}

Widget _lyBKindSeg(Doc doc, String t, double px) => SizedBox(
  height: Pop.cell,
  child: Dis(
    child: Segmented(
      items: _lyBKinds,
      index: math.max(0, _lyBKinds.indexOf(doc.s2['ly_b_$t.k'] ?? 'fr')),
      expand: true,
      onChanged: (i) => _lyBSetKind(doc, t, _lyBKinds[i], px),
    ),
  ),
);

_LyGridOut _lyBSolve(Doc doc) {
  final nc = _lyBCount(doc, 'cols'), nr = _lyBCount(doc, 'rows');
  return _lyGrid(
    Size(_lyG(doc, 'ly_b_w'), _lyG(doc, 'ly_b_h')),
    _lyG(doc, 'ly_b_pad'),
    _lyG(doc, 'ly_b_pad'),
    _lyG(doc, 'ly_b_gap'),
    [for (var i = 0; i < nc; i++) _lyBTrack(doc, 'c$i')],
    [for (var j = 0; j < nr; j++) _lyBTrack(doc, 'r$j')],
    [for (final k in _lyBKids) _LyIt(k.w, k.h)],
    [
      for (var i = 0; i < _lyBKids.length; i++)
        (_lyG(doc, 'ly_b_k$i.c').round(), _lyG(doc, 'ly_b_k$i.r').round(), _lyG(doc, 'ly_b_k$i.cs').round(), _lyG(doc, 'ly_b_k$i.rs').round()),
    ],
  );
}

class _LyBStage extends StatelessWidget {
  const _LyBStage(this.g);
  final _LyGridOut g;
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    return LayoutBuilder(
      builder: (context, box) {
        final w = _lyG(doc, 'ly_b_w'), h = _lyG(doc, 'ly_b_h'), gap = _lyG(doc, 'ly_b_gap');
        final s = (box.maxWidth - _lyBHdrW) / w, o = const Offset(_lyBHdrW, _lyBHdrH);
        final nc = _lyBCount(doc, 'cols'), nr = _lyBCount(doc, 'rows');
        final sel = int.tryParse(doc.s2['ly_b_sel'] ?? '');
        Rect sc(Rect r) => Rect.fromLTRB(o.dx + r.left * s, o.dy + r.top * s, o.dx + r.right * s, o.dy + r.bottom * s);
        final kids = [for (final b in g.boxes) sc(b)];
        final guts = [for (final (r, v) in g.gutters) (sc(r), v)];
        final cols = [for (var i = 0; i < nc; i++) Rect.fromLTWH(o.dx + g.xs[i] * s, 1, g.ws[i] * s, _lyBHdrH - 4)];
        final rows = [for (var j = 0; j < g.ys.length; j++) Rect.fromLTWH(1, o.dy + g.ys[j] * s, _lyBHdrW - 5, g.hs[j] * s)];
        int cellAt(List<double> st, double q) {
          var i = 0;
          while (i + 1 < st.length && st[i + 1] - gap / 2 <= q) {
            i++;
          }
          return i;
        }

        (int, int) cell(Offset p) {
          final q = (p - o) / s;
          final r = q.dy > g.ys.last + g.hs.last + gap ? g.ys.length : cellAt(g.ys, q.dy);
          return (cellAt(g.xs, q.dx), math.min(r, 7));
        }

        void resize(bool col, int i, double d) {
          final t = '${col ? 'c' : 'r'}$i', id = 'ly_b_$t', sizes = col ? g.ws : g.hs, n = col ? nc : nr;
          final tr = _lyBTrack(doc, t), nw = math.max(8.0, sizes[i] + d / s);
          var frSum = 0.0, frSpace = 0.0;
          for (var k = 0; k < n; k++) {
            final x = _lyBTrack(doc, '${col ? 'c' : 'r'}$k');
            if (x.k == 'fr') {
              frSum += x.v;
              frSpace += sizes[k];
            }
          }
          if (tr.k == 'fr' && frSum - tr.v > 1e-6 && frSpace - nw > 8) {
            doc.set(id, double.parse((nw * (frSum - tr.v) / (frSpace - nw)).toStringAsFixed(2)).clamp(.1, 12));
          } else {
            doc.s2['$id.k'] = 'px';
            doc.set(id, nw.roundToDouble().clamp(8, 480));
          }
        }

        _LyHit? hit(Offset p) {
          for (final (i, r) in cols.indexed) {
            if (p.dy > _lyBHdrH) break;
            final edge = r.right + gap * s / 2;
            if ((p.dx - edge).abs() < 5) {
              return _LyHit('cd$i', cursor: SystemMouseCursors.resizeColumn, drag: (t) => resize(true, i, t.dx));
            }
            if (p.dx >= r.left && p.dx <= r.right) {
              final k = doc.s2['ly_b_c$i.k'] ?? 'fr';
              return _LyHit(
                'ch$i',
                tap: () {
                  doc.s2['ly_b_track'] = 'c$i';
                  _lyBSetKind(doc, 'c$i', _lyBKinds[(_lyBKinds.indexOf(k) + 1) % 3], g.ws[i]);
                },
              );
            }
          }
          if (p.dx < _lyBHdrW) {
            for (final (j, r) in rows.indexed) {
              if (j >= nr) break;
              final edge = r.bottom + gap * s / 2;
              if ((p.dy - edge).abs() < 5) return _LyHit('rd$j', cursor: SystemMouseCursors.resizeRow, drag: (t) => resize(false, j, t.dy));
              if (p.dy >= r.top && p.dy <= r.bottom) {
                final k = doc.s2['ly_b_r$j.k'] ?? 'fr';
                return _LyHit(
                  'rh$j',
                  tap: () {
                    doc.s2['ly_b_track'] = 'r$j';
                    _lyBSetKind(doc, 'r$j', _lyBKinds[(_lyBKinds.indexOf(k) + 1) % 3], g.hs[j]);
                  },
                );
              }
            }
            return null;
          }
          for (final (i, k) in kids.indexed.toList().reversed) {
            final at = g.at[i];
            if ((p - k.bottomRight).distance < 7) {
              return _LyHit(
                'sp$i',
                cursor: SystemMouseCursors.resizeUpLeftDownRight,
                drag: (t) {
                  final (ce, re) = cell(k.bottomRight + t);
                  doc.setMany({
                    'ly_b_k$i.c': at.$1 + 1.0,
                    'ly_b_k$i.r': at.$2 + 1.0,
                    'ly_b_k$i.cs': math.max(1, ce - at.$1 + 1).clamp(1, nc - at.$1).toDouble(),
                    'ly_b_k$i.rs': math.max(1, re - at.$2 + 1).clamp(1, 5).toDouble(),
                  });
                },
              );
            }
            if (k.contains(p)) {
              final from = cell(p);
              return _LyHit(
                'k$i',
                cursor: SystemMouseCursors.move,
                tap: () => doc.str('ly_b_sel', '$i'),
                drag: (t) {
                  final now = cell(p + t);
                  doc.s2['ly_b_sel'] = '$i';
                  doc.setMany({
                    'ly_b_k$i.c': (at.$1 + now.$1 - from.$1).clamp(0, nc - at.$3) + 1.0,
                    'ly_b_k$i.r': math.max(0, at.$2 + now.$2 - from.$2) + 1.0,
                    'ly_b_k$i.cs': at.$3.toDouble(),
                    'ly_b_k$i.rs': at.$4.toDouble(),
                  });
                },
              );
            }
          }
          for (final (r, v) in guts) {
            final hr = v
                ? Rect.fromCenter(center: r.center, width: math.max(r.width, 6), height: r.height)
                : Rect.fromCenter(center: r.center, width: r.width, height: math.max(r.height, 6));
            if (hr.contains(p)) {
              return _LyHit(
                'gap',
                cursor: v ? SystemMouseCursors.resizeLeftRight : SystemMouseCursors.resizeUpDown,
                drag: (t) => doc.set('ly_b_gap', (gap + (v ? t.dx : t.dy) / s).roundToDouble().clamp(0, 48)),
              );
            }
          }
          return null;
        }

        final words = [for (var i = 0; i < nc; i++) _lyBTrack(doc, 'c$i').word];
        final rWords = [for (var j = 0; j < g.ys.length; j++) j < nr ? _lyBTrack(doc, 'r$j').word : 'auto'];
        final cellsR = [
          for (var j = 0; j < g.ys.length; j++)
            for (var i = 0; i < nc; i++) sc(Rect.fromLTWH(g.xs[i], g.ys[j], g.ws[i], g.hs[j])),
        ];
        final sig = '${kids.join()} ${cols.join()} ${rows.join()} $words $rWords $sel ${doc.s2['ly_b_track']} ${box.maxWidth}';
        return Padding(
          padding: const EdgeInsets.only(top: 2),
          child: _LyStage(
            key: const ValueKey('ly_b_stage'),
            height: _lyBHdrH + h * s + 2,
            hit: hit,
            paint: (hov, act) =>
                _LyBPaint(sig, sc(Rect.fromLTWH(0, 0, w, h)), kids, guts, cols, rows, cellsR, words, rWords, nr, sel, doc.s2['ly_b_track'], hov ?? act),
          ),
        );
      },
    );
  }
}

class _LyBPaint extends CustomPainter {
  const _LyBPaint(this.sig, this.box, this.kids, this.guts, this.cols, this.rows, this.cells, this.words, this.rWords, this.nr, this.sel, this.track, this.hot);
  final String sig;
  final Rect box;
  final List<Rect> kids, cols, rows, cells;
  final List<(Rect, bool)> guts;
  final List<String> words, rWords;
  final int nr;
  final int? sel;
  final String? track, hot;
  @override
  void paint(Canvas c, Size s) {
    c.clipRect(Offset.zero & s);
    c.drawRect(box, Paint()..color = Pop.well);
    c.drawRect(box, _lyLine(Grey.g38));
    for (final r in cells) {
      c.drawRect(r, Paint()..color = Grey.g10);
    }
    for (final (r, _) in guts) {
      c.drawRect(r, Paint()..color = hot == 'gap' ? _lyGapHot : _lyGapC);
    }
    void chip(Rect r, String w, bool on, bool hov, bool dim) {
      final rr = RRect.fromRectAndRadius(r.deflate(1), const Radius.circular(4));
      c.drawRRect(rr, Paint()..color = on ? _lyKidSel : (hov ? Grey.g26 : (dim ? Grey.g13 : Grey.g20)));
      if (r.width > 18 && r.height > 10) _lyText(c, w, r.center, T.micro(dim ? Grey.g44 : (on ? Grey.g100 : Grey.g76)), maxW: math.max(4, r.width - 4));
    }

    for (final (i, r) in cols.indexed) {
      chip(r, words[i], track == 'c$i', hot == 'ch$i', false);
      final x = r.right + (i + 1 < cols.length ? (cols[i + 1].left - r.right) / 2 : 0);
      final on = hot == 'cd$i';
      c.drawLine(Offset(x, 2), Offset(x, box.top - 2), _lyLine(on ? Pop.accentInk : Grey.g56, on ? 2 : 1.2));
      if (on) c.drawLine(Offset(x, box.top), Offset(x, box.bottom), _lyLine(Pop.accentInk.withValues(alpha: .6)));
    }
    for (final (j, r) in rows.indexed) {
      chip(r, rWords[j], track == 'r$j', hot == 'rh$j', j >= nr);
      if (j >= nr) continue;
      final y = r.bottom + (j + 1 < rows.length ? (rows[j + 1].top - r.bottom) / 2 : 0);
      final on = hot == 'rd$j';
      c.drawLine(Offset(2, y), Offset(box.left - 2, y), _lyLine(on ? Pop.accentInk : Grey.g56, on ? 2 : 1.2));
      if (on) c.drawLine(Offset(box.left, y), Offset(box.right, y), _lyLine(Pop.accentInk.withValues(alpha: .6)));
    }
    for (final (i, k) in kids.indexed) {
      _lyKidBox(c, k, _lyBKids[i].name, sel: i == sel, hot: hot == 'k$i');
      final hs = hot == 'sp$i';
      if (i == sel || hs) {
        c.drawRect(Rect.fromCenter(center: k.bottomRight - const Offset(3, 3), width: 6, height: 6), Paint()..color = hs ? Grey.g100 : Pop.accent);
      }
    }
  }

  @override
  bool shouldRepaint(_LyBPaint o) => o.sig != sig || o.hot != hot;
}

// ---- C: flex playground ---------------------------------------------------------------------------------------------
const _lyCKids = [_LyKid('Logo', 48, 40), _LyKid('Title', 112, 28), _LyKid('Body', 80, 52), _LyKid('Tag', 40, 20), _LyKid('Button', 64, 28)];
const _lyCMaxW = 520.0, _lyCMinW = 140.0;
const _lyCDist = ['Pack', 'Between', 'Around', 'Evenly'];
const _lyCPos = ['Start', 'Center', 'End'];

class _LyC extends StatelessWidget {
  const _LyC();
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    doc.seed(
      {'ly_c_w': 440, 'ly_c_h': 180, 'ly_c_pad': 12, 'ly_c_gap': 10, for (final (i, k) in _lyCKids.indexed) 'ly_c_k$i.fw': k.w},
      strs: {
        'ly_c_dir': 'Row',
        'ly_c_justify': 'Center',
        'ly_c_align': 'Center',
        'ly_c_dist': 'Pack',
        'ly_c_sel': '',
        for (var i = 0; i < _lyCKids.length; i++) 'ly_c_k$i.sz': i == 2 ? 'Fill' : 'Hug',
      },
      flags: const {'ly_c_wrap': true, 'ly_c_stretch': false},
    );
    final col = doc.s2['ly_c_dir'] == 'Column', wrap = doc.b['ly_c_wrap'] ?? false, stretch = doc.b['ly_c_stretch'] ?? false;
    final dist = doc.s2['ly_c_dist'] ?? 'Pack', sel = int.tryParse(doc.s2['ly_c_sel'] ?? '');
    final fixed = sel != null && sel < _lyCKids.length && doc.s2['ly_c_k$sel.sz'] == 'Fixed';
    return _lyCard(
      doc,
      'ly_c',
      brief: [
        '${col ? 'Column' : 'Row'}${wrap ? ' · Wrap' : ''}',
        '${dist == 'Pack' ? doc.s2['ly_c_justify'] : dist} · ${stretch ? 'Stretch' : doc.s2['ly_c_align']}',
        'Gap ${brief(doc.get('ly_c_gap'), 0)}',
      ],
      children: [
        Row(
          children: [
            _LyChip(
              col ? 'Column' : 'Row',
              key: const ValueKey('ly_c_rotate'),
              lead: AnimatedRotation(
                turns: col ? .25 : 0,
                duration: const Duration(milliseconds: 140),
                child: const CustomPaint(size: Size(10, 10), painter: _LyArrow()),
              ),
              onTap: () => doc.str('ly_c_dir', col ? 'Row' : 'Column'),
            ),
            const SizedBox(width: 4),
            _LyChip('Wrap', on: wrap, onTap: () => doc.flag('ly_c_wrap', !wrap)),
            const SizedBox(width: 4),
            _LyChip('Stretch', on: stretch, onTap: () => doc.flag('ly_c_stretch', !stretch)),
            Expanded(
              child: Text('click to gather · drag edge', textAlign: TextAlign.right, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(Grey.g44)),
            ),
          ],
        ),
        const SizedBox(height: 4),
        const _LyCStage(),
        _lyCells([_lySeg(doc, 'ly_c_dist', _lyCDist)]),
        _lyCells([for (var i = 0; i < _lyCKids.length; i++) _LyCKidChip(i, sel == i)]),
        _lyCells([
          _lyNum('ly_c_w', 'Width', min: _lyCMinW, max: _lyCMaxW),
          _lyNum('ly_c_gap', 'Gap', max: 48),
          _lyNum('ly_c_pad', 'Pad', max: 48),
          if (fixed) _lyNum('ly_c_k$sel.fw', '${_lyCKids[sel].name} ${col ? 'H' : 'W'}', min: 8, max: 300),
        ]),
      ],
    );
  }
}

/// One child's main-axis sizing as a chip: a click selects it and steps Hug -> Fill -> Fixed.
class _LyCKidChip extends StatelessWidget {
  const _LyCKidChip(this.i, this.sel);
  final int i;
  final bool sel;
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc, id = 'ly_c_k$i.sz', sz = doc.s2[id] ?? 'Hug';
    return Hov(
      onTap: Ctx.of(context).cfg.locked
          ? null
          : () {
              doc.s2['ly_c_sel'] = '$i';
              doc.str(id, sel ? _lySizing[(_lySizing.indexOf(sz) + 1) % 3] : sz);
            },
      builder: (_, h) => Container(
        height: 30,
        padding: const EdgeInsets.symmetric(horizontal: 6),
        decoration: BoxDecoration(
          color: h ? Grey.g20 : Pop.well,
          borderRadius: BorderRadius.circular(6),
          border: Border.all(color: sel ? Pop.accentInk.withValues(alpha: .8) : const Color(0x00000000)),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Text(_lyCKids[i].name, maxLines: 1, overflow: TextOverflow.clip, style: T.micro(Grey.g56)),
            const SizedBox(height: 3),
            Text(sz, maxLines: 1, style: T.label(sz == 'Hug' ? Grey.g76 : Grey.g95).copyWith(fontWeight: FontWeight.w700)),
          ],
        ),
      ),
    );
  }
}

class _LyArrow extends CustomPainter {
  const _LyArrow();
  @override
  void paint(Canvas c, Size s) {
    final p = _lyLine(Grey.g91, 1.4)
      ..strokeCap = StrokeCap.round
      ..strokeJoin = StrokeJoin.round;
    c.drawLine(Offset(1, s.height / 2), Offset(s.width - 1, s.height / 2), p);
    c.drawPath(
      Path()
        ..moveTo(s.width - 4.5, s.height / 2 - 3.5)
        ..lineTo(s.width - 1, s.height / 2)
        ..lineTo(s.width - 4.5, s.height / 2 + 3.5),
      p,
    );
  }

  @override
  bool shouldRepaint(_LyArrow o) => false;
}

class _LyCStage extends StatelessWidget {
  const _LyCStage();
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    return LayoutBuilder(
      builder: (context, box) {
        final s = box.maxWidth / _lyCMaxW;
        final w = _lyG(doc, 'ly_c_w'), h = _lyG(doc, 'ly_c_h'), pad = _lyG(doc, 'ly_c_pad'), gap = _lyG(doc, 'ly_c_gap');
        final col = doc.s2['ly_c_dir'] == 'Column', dist = doc.s2['ly_c_dist'] ?? 'Pack', stretch = doc.b['ly_c_stretch'] ?? false;
        final j = doc.s2['ly_c_justify'] ?? 'Start', a = doc.s2['ly_c_align'] ?? 'Center';
        final out = _lyFlex(
          Size(w, h),
          pad,
          pad,
          [for (final (i, k) in _lyCKids.indexed) _LyIt(k.w, k.h, sz: doc.s2['ly_c_k$i.sz'] ?? 'Hug', fw: _lyG(doc, 'ly_c_k$i.fw'))],
          col: col,
          wrap: doc.b['ly_c_wrap'] ?? false,
          gap: gap,
          justify: dist == 'Pack' ? j : 'Space $dist',
          align: stretch ? 'Stretch' : a,
        );
        Rect sc(Rect r) => Rect.fromLTRB(r.left * s, r.top * s, r.right * s, r.bottom * s);
        final outer = Rect.fromLTWH(0, 0, w * s, h * s), inner = outer.deflate(pad * s);
        final kids = [for (final b in out.boxes) sc(b)];
        final guts = [for (final (r, v) in out.gutters) (sc(r), v)];
        final sel = int.tryParse(doc.s2['ly_c_sel'] ?? '');
        (int, int) zone(Offset p) => (((p.dx - inner.left) / inner.width * 3).floor().clamp(0, 2), ((p.dy - inner.top) / inner.height * 3).floor().clamp(0, 2));
        void gather(Offset p) {
          final (zx, zy) = zone(p);
          doc.s2['ly_c_dist'] = 'Pack';
          doc.b['ly_c_stretch'] = false;
          doc.s2['ly_c_justify'] = _lyCPos[col ? zy : zx];
          doc.str('ly_c_align', _lyCPos[col ? zx : zy]);
        }

        _LyHit? hit(Offset p) {
          if ((p.dx - outer.right).abs() < 6 && p.dy >= outer.top && p.dy <= outer.bottom + 12) {
            return _LyHit(
              'w',
              cursor: SystemMouseCursors.resizeLeftRight,
              drag: (t) => doc.set('ly_c_w', (w + t.dx / s).roundToDouble().clamp(_lyCMinW, _lyCMaxW)),
            );
          }
          for (final (i, k) in kids.indexed) {
            if (k.contains(p)) return _LyHit('k$i', tap: () => doc.str('ly_c_sel', sel == i ? '' : '$i'));
          }
          for (final (r, v) in guts) {
            final hr = v
                ? Rect.fromCenter(center: r.center, width: math.max(r.width, 6), height: r.height)
                : Rect.fromCenter(center: r.center, width: r.width, height: math.max(r.height, 6));
            if (hr.contains(p)) {
              return _LyHit(
                'gap',
                cursor: v ? SystemMouseCursors.resizeLeftRight : SystemMouseCursors.resizeUpDown,
                drag: (t) => doc.set('ly_c_gap', (gap + (v ? t.dx : t.dy) / s).roundToDouble().clamp(0, 48)),
              );
            }
          }
          if (outer.contains(p)) {
            final (zx, zy) = zone(p);
            return _LyHit('z$zx$zy', tap: () => gather(p));
          }
          return null;
        }

        final mi = dist == 'Pack' ? _lyCPos.indexOf(j) : -1, ci = stretch ? -1 : _lyCPos.indexOf(a);
        final lit = (col ? (ci, mi) : (mi, ci));
        final over = kids.any((k) => k.right > outer.right + .5 || k.bottom > outer.bottom + .5);
        final sig = '${kids.join()} ${guts.length} $w $sel $lit $over ${box.maxWidth}';
        return _LyStage(
          key: const ValueKey('ly_c_stage'),
          height: h * s + 14,
          hit: hit,
          paint: (hov, act) => _LyCPaint(sig, outer, inner, kids, guts, sel, lit, over, w, hov ?? act),
        );
      },
    );
  }
}

class _LyCPaint extends CustomPainter {
  const _LyCPaint(this.sig, this.outer, this.inner, this.kids, this.guts, this.sel, this.lit, this.over, this.w, this.hot);
  final String sig;
  final Rect outer, inner;
  final List<Rect> kids;
  final List<(Rect, bool)> guts;
  final int? sel;

  /// The gathered spot (x, y) in thirds; -1 on an axis = spread (Space *) or Stretch.
  final (int, int) lit;
  final bool over;
  final double w;
  final String? hot;
  @override
  void paint(Canvas c, Size s) {
    c.clipRect(Offset.zero & s);
    c.drawRect(outer, Paint()..color = _lyPadC);
    c.drawRect(inner, Paint()..color = Grey.g10);
    double at(double lo, double hi, int k) => lo + (hi - lo) * (k * 2 + 1) / 6;
    for (var y = 0; y < 3; y++) {
      for (var x = 0; x < 3; x++) {
        final p = Offset(at(inner.left, inner.right, x), at(inner.top, inner.bottom, y));
        c.drawCircle(p, 1.6, Paint()..color = Grey.g38);
        if (hot == 'z$x$y') c.drawCircle(p, 6, _lyLine(Pop.accentInk, 1.4));
      }
    }
    final (lx, ly) = lit;
    final mark = Rect.fromCenter(
      center: Offset(lx < 0 ? inner.center.dx : at(inner.left, inner.right, lx), ly < 0 ? inner.center.dy : at(inner.top, inner.bottom, ly)),
      width: lx < 0 ? inner.width - 8 : 10,
      height: ly < 0 ? inner.height - 8 : 10,
    );
    c.drawRRect(RRect.fromRectAndRadius(mark, const Radius.circular(3)), _lyLine(Pop.toneLayout, 1.4));
    for (final (r, _) in guts) {
      c.drawRect(r, Paint()..color = hot == 'gap' ? _lyGapHot : _lyGapC);
    }
    for (final (i, k) in kids.indexed) {
      _lyKidBox(c, k, _lyCKids[i].name, sel: i == sel, hot: hot == 'k$i');
    }
    c.drawRect(outer, _lyLine(over ? Pop.tonePlace : Grey.g44));
    final on = hot == 'w';
    final grip = Rect.fromCenter(center: Offset(outer.right, outer.center.dy), width: on ? 5 : 4, height: 22);
    c.drawRRect(RRect.fromRectAndRadius(grip, const Radius.circular(2)), Paint()..color = on ? Pop.accent : Grey.g76);
    _lyText(
      c,
      '${w.round()} px${over ? ' · overflows' : ''}',
      Offset(math.max(outer.right - 40, 44), outer.bottom + 7),
      T.micro(over ? Pop.tonePlace : Grey.g56),
    );
  }

  @override
  bool shouldRepaint(_LyCPaint o) => o.sig != sig || o.hot != hot;
}
