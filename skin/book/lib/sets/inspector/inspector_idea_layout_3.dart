// Part of inspector_idea_layout.dart (cut there by size, as it was).
part of 'inspector_idea_layout.dart';

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
