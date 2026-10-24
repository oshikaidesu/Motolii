// Part of inspector_idea_layout.dart (cut there by size, as it was).
part of 'inspector_idea_layout.dart';

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

Widget _lyPick(String id, List<String> options) => SizedBox(
  height: Pop.cell,
  child: Chooser(id: id, options: options),
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
            grid ? _lyNum('ly_a_cols', 'Cols', min: 1, max: 6, perPx: .1) : _lyPick('ly_a_justify', _lyJustify),
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
          _lyCells([_lyPick('$k.self', _lySelf), _lyNum('$k.m', 'Margin', max: 64)]),
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
