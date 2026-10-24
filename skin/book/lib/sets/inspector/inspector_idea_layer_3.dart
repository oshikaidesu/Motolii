// Part of inspector_idea_layer.dart (cut there by size, as it was).
part of 'inspector_idea_layer.dart';

class _LiTimePaint extends CustomPainter {
  const _LiTimePaint(this.k, this.hx, this.hot);
  final _LiClock k;
  final double? hx;
  final _LiTGrab? hot;

  static Color _shade(double s) => Color.lerp(const Color(0xFF2A1A22), Pop.toneTime, (s / _liSrcLen).clamp(0.0, 1.0))!;

  /// The bar as columns coloured by the source time they play, so speed reads as compression and Ping-pong as a mirrored ramp.
  void _bar(Canvas c, Size s, double top, double bot, double shift, double alpha) {
    double? prev;
    for (var px = math.max(8.0, _liX(s, k.delay + shift)); px < s.width - 8; px += 2) {
      final sv = k.src(_liT(s, px) - shift);
      if (sv == null) {
        if (prev != null) break;
        continue;
      }
      final first = _liT(s, px) - shift <= k.cycleEnd;
      c.drawRect(Rect.fromLTWH(px, top, 2, bot - top), Paint()..color = _shade(sv).withValues(alpha: alpha * (first ? 1 : .6)));
      if (prev != null && (sv * 4).floor() != (prev * 4).floor()) {
        c.drawLine(Offset(px, top), Offset(px, top + (bot - top) * .35), _liLine(Grey.g07.withValues(alpha: .6 * alpha)));
      }
      prev = sv;
    }
  }

  @override
  void paint(Canvas c, Size s) {
    c.drawRRect(RRect.fromRectAndRadius(Offset.zero & s, const Radius.circular(6)), Paint()..color = Pop.well);
    for (var i = 0; i <= _liWin; i++) {
      final x = _liX(s, i.toDouble());
      c.drawLine(Offset(x, 2), Offset(x, _liGhBot), _liLine(i == 0 ? Grey.g38 : Grey.g20));
      if (i < _liWin) _liText(c, '${i}s', Offset(x + 3, 2), T.label(Grey.g44));
    }
    final x0 = _liX(s, k.delay), xe = _liX(s, k.cycleEnd), mid = (_liBrTop + _liBrBot) / 2;
    // loop bracket
    if (k.mode > 0) {
      for (var t = k.cycleEnd; t < _liWin; t += k.cycle / k.sp) {
        final x = _liX(s, t);
        c.drawLine(Offset(x, mid), Offset(x, _liBrBot), _liLine(Grey.g38));
      }
      final hot = this.hot == _LiTGrab.loop, ink = hot ? Grey.g100 : Pop.toneTime;
      c.drawPath(
        Path()
          ..moveTo(x0, _liBrBot)
          ..lineTo(x0, _liBrTop + 2)
          ..lineTo(xe, _liBrTop + 2)
          ..lineTo(xe, _liBrBot),
        _liLine(ink, hot ? 1.8 : 1.3),
      );
      c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: Offset(xe, mid + 1), width: 5, height: 12), const Radius.circular(2)), Paint()..color = ink);
      _liText(c, '${k.mode == 2 ? '↔' : '→'} ${brief(k.loop, 2)} s', Offset((x0 + xe) / 2, _liBrTop + 4), T.label(Grey.g95), ax: .5, bg: Pop.well);
    } else {
      _liText(c, '+ Loop: drag here', Offset(x0 + 2, _liBrTop + 3), T.label(hot == _LiTGrab.loop ? Grey.g91 : Grey.g44));
    }
    // the bar, its outline, the speed grip
    _bar(c, s, _liBarTop, _liBarBot, 0, 1);
    final xEnd = k.mode == 0 ? xe : s.width - 8;
    c.drawRRect(
      RRect.fromRectAndRadius(Rect.fromLTRB(math.max(8, x0), _liBarTop, math.min(xEnd, s.width - 8), _liBarBot), const Radius.circular(3)),
      _liLine(hot == _LiTGrab.bar ? Grey.g91 : Grey.g44),
    );
    final sHot = hot == _LiTGrab.speed;
    c.drawRRect(
      RRect.fromRectAndRadius(
        Rect.fromCenter(center: Offset(xe, (_liBarTop + _liBarBot) / 2), width: sHot ? 6 : 4, height: _liBarBot - _liBarTop + 6),
        const Radius.circular(2),
      ),
      Paint()..color = sHot ? Pop.accent : Grey.g95,
    );
    final spw = '${brief(k.sp * 100, 0)}%', right = xe + 40 < s.width - 8;
    _liText(
      c,
      spw,
      Offset(right ? xe + 6 : xe - 6, _liBarTop + 5),
      T.label(Grey.g95).copyWith(fontWeight: FontWeight.w700),
      ax: right ? 0 : 1,
      bg: Pop.well,
    );
    // ghost
    final g = k.ghost;
    if (g != null) {
      _bar(c, s, _liGhTop, _liGhBot, g, hot == _LiTGrab.ghost ? .7 : .45);
      _liText(c, 'Ghost ${_liSigned(g * _liFps, 0)} f', Offset(math.max(10, _liX(s, k.delay + g)) + 2, _liGhTop + 1), T.label(Grey.g76));
    } else {
      _liDash(
        c,
        Path()..addRRect(RRect.fromRectAndRadius(Rect.fromLTRB(math.max(8, x0), _liGhTop, math.min(xe, s.width - 8), _liGhBot), const Radius.circular(3))),
        _liLine(hot == _LiTGrab.ghost ? Grey.g76 : Grey.g38),
      );
      _liText(c, 'Ghost: drag or click', Offset(math.max(8, x0) + 4, _liGhTop + 1), T.label(Grey.g44));
    }
    // hover: which source frame plays at this comp time
    final h = hx;
    final ry = _liGhBot + 7;
    if (h != null && h >= 8 && h <= s.width - 8) {
      final t = _liT(s, h), sv = k.src(t), gv = g == null ? null : k.src(t - g);
      c.drawLine(Offset(h, 2), Offset(h, _liGhBot), _liLine(Grey.g95.withValues(alpha: .7)));
      if (sv != null) c.drawCircle(Offset(h, (_liBarTop + _liBarBot) / 2), 3, Paint()..color = Grey.g100);
      _liText(
        c,
        '${t.toStringAsFixed(2)} s  →  ${sv == null ? 'not playing' : 'source f ${(sv * _liFps).round()}'}${gv == null ? '' : '  ·  ghost f ${(gv * _liFps).round()}'}',
        Offset(8, ry),
        T.label(Grey.g91),
      );
      if (sv != null) {
        final o = Offset(s.width - 16, ry + 5);
        c.save();
        c.translate(o.dx, o.dy);
        c.rotate(sv / _liSrcLen * math.pi);
        c.drawRRect(
          RRect.fromRectAndRadius(Rect.fromCenter(center: Offset.zero, width: 10, height: 10), const Radius.circular(2)),
          Paint()..color = _shade(sv),
        );
        c.restore();
      }
    } else {
      final end = k.mode == 0 ? 'ends ${brief(k.cycleEnd, 2)} s' : 'loops to the end';
      _liText(c, 'Source ${brief(_liSrcLen, 1)} s · ${brief(k.sp * 100, 0)}% · $end · hover to read', Offset(8, ry), T.label(Grey.g56));
    }
  }

  @override
  bool shouldRepaint(_LiTimePaint o) =>
      o.hx != hx || o.hot != hot || o.k.sp != k.sp || o.k.loop != k.loop || o.k.delay != k.delay || o.k.mode != k.mode || o.k.ghost != k.ghost;
}

// ---- C. Link wires --------------------------------------------------------------------------------------------------
const _liSrcs = [
  ('Logo', 'Opacity', 80.0),
  ('Logo', 'Rotation', 30.0),
  ('Null 1', 'Position X', 240.0),
  ('Null 1', 'Rotation', 90.0),
  ('Audio', 'Level', 60.0),
];
const _liTgts = [('Opacity', 62.0), ('Position X', 420.0), ('Position Y', 540.0), ('Rotation', 0.0), ('Scale', 100.0), ('Blur', 24.0)];
const _liPitch = 24.0, _liPillH = 20.0, _liSrcW = 118.0, _liTgtW = 112.0, _liNodeH = 6 * _liPitch + 4, _liTagW = 96.0;

Rect _liSrcRect(int i) => Rect.fromLTWH(0, 4 + (_liTgts.length - _liSrcs.length) * _liPitch / 2 + i * _liPitch, _liSrcW, _liPillH);
Rect _liTgtRect(double w, int i) => Rect.fromLTWH(w - _liTgtW, 4 + i * _liPitch, _liTgtW, _liPillH);

/// contract: one source per target (PropertyLink lives on the driven property); the shown value is source × Multiply + Offset, Delay in frames.
List<(int, int)> _liLinks(Doc doc) => [
  for (var t = 0; t < _liTgts.length; t++)
    if (int.tryParse(doc.s2['li_l.$t.src'] ?? '') case final s?) (t, s),
];

(Offset, Offset, Offset, Offset) _liWire(Offset a, Offset b) {
  final d = math.max(24.0, (b.dx - a.dx) * .5);
  return (a, a + Offset(d, 0), b - Offset(d, 0), b);
}

Offset _liBez((Offset, Offset, Offset, Offset) w, double t) {
  final u = 1 - t;
  return w.$1 * (u * u * u) + w.$2 * (3 * u * u * t) + w.$3 * (3 * u * t * t) + w.$4 * (t * t * t);
}

class _LiLinkCard extends StatelessWidget {
  const _LiLinkCard();
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    doc.seed(
      {
        for (var t = 0; t < _liTgts.length; t++) ...{'li_l.$t.mul': t == 0 ? .5 : 1, 'li_l.$t.add': 0, 'li_l.$t.f': t == 0 ? 5 : 0},
      },
      strs: const {'li_l.0.src': '0', 'li_l.3.src': '3', 'li_l.sel': '0'},
    );
    final links = _liLinks(doc), sel = int.tryParse(doc.s2['li_l.sel'] ?? '');
    final cur = links.where((l) => l.$1 == sel).firstOrNull;
    return Sect(
      title: 'Link wires',
      tone: Pop.toneLink,
      mark: CardMark.link,
      keyIds: [
        for (final (t, _) in links) ...['li_l.$t.mul', 'li_l.$t.add'],
      ],
      brief: [
        links.isEmpty ? 'No links' : '${links.length} link${links.length == 1 ? '' : 's'}',
        for (final (t, s) in links.take(2)) '${_liTgts[t].$1} ← ${_liSrcs[s].$1}',
      ],
      children: [
        const _LiWires(),
        if (cur == null)
          const Cap('Drag from a property on the left to one of this layer\'s on the right. Click a wire to edit it.')
        else ...[
          Padding(
            padding: const EdgeInsets.only(top: 6),
            child: SizedBox(
              height: 18,
              child: Row(
                children: [
                  Expanded(
                    child: Text(
                      '${_liTgts[cur.$1].$1}  ←  ${_liSrcs[cur.$2].$1} · ${_liSrcs[cur.$2].$2}',
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                      style: T.name(Grey.g91).copyWith(fontWeight: FontWeight.w600),
                    ),
                  ),
                  _LiChip(
                    'Unlink',
                    on: false,
                    onTap: () {
                      doc.s2['li_l.sel'] = '';
                      doc.str('li_l.${cur.$1}.src', '');
                    },
                  ),
                ],
              ),
            ),
          ),
          KeyedSubtree(
            key: ValueKey('li_l${cur.$1}'),
            child: _liRow([
              ldNum(doc, 'li_l.${cur.$1}.mul', 'Multiply', dec: 2, min: -4, max: 4, perPx: .01, step: .1, bipolar: true),
              ldNum(doc, 'li_l.${cur.$1}.add', 'Offset', dec: 1, perPx: .5),
              ldNum(doc, 'li_l.${cur.$1}.f', 'Delay', unit: 'f', min: -120, max: 120, perPx: .25, zero: 'None', bipolar: true),
            ]),
          ),
        ],
      ],
    );
  }
}

/// contract: a press on a left pill draws a wire; let go on a right pill to link (replacing that property's link). A click on a wire,
/// its tag or a driven property selects that link; the tag's two numbers scrub sideways.
class _LiWires extends StatefulWidget {
  const _LiWires();
  @override
  State<_LiWires> createState() => _LiWiresState();
}

class _LiWiresState extends State<_LiWires> {
  int? _from, _over;
  Offset? _wire;
  Offset _down = Offset.zero;
  bool _moved = false;

  int? _tgtAt(double w, Offset p) {
    for (var t = 0; t < _liTgts.length; t++) {
      if (_liTgtRect(w, t).inflate(3).contains(p)) return t;
    }
    return null;
  }

  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, on = !x.cfg.locked;
    final links = _liLinks(doc), sel = int.tryParse(doc.s2['li_l.sel'] ?? '');
    return LayoutBuilder(
      builder: (context, box) {
        final w = box.maxWidth, midX = (_liSrcW + w - _liTgtW) / 2;
        (Offset, Offset, Offset, Offset) wire((int, int) l) =>
            _liWire(Offset(_liSrcRect(l.$2).right, _liSrcRect(l.$2).center.dy), Offset(_liTgtRect(w, l.$1).left, _liTgtRect(w, l.$1).center.dy));
        final tags = <(int, double)>[];
        for (final l in [...links]..sort((a, b) => _liBez(wire(a), .5).dy.compareTo(_liBez(wire(b), .5).dy))) {
          final y = _liBez(wire(l), .5).dy.clamp(9.0, _liNodeH - 9);
          tags.add((l.$1, tags.isEmpty ? y : math.max(y, tags.last.$2 + 19)));
        }
        double value(int t) {
          final s = int.tryParse(doc.s2['li_l.$t.src'] ?? '');
          return s == null ? _liTgts[t].$2 : _liSrcs[s].$3 * (doc.get('li_l.$t.mul') ?? 1) + (doc.get('li_l.$t.add') ?? 0);
        }

        return Padding(
          padding: const EdgeInsets.only(top: 2),
          child: Listener(
            onPointerDown: (e) {
              final p = e.localPosition;
              _down = p;
              _moved = false;
              if (!on || e.buttons != kPrimaryButton) return;
              for (var i = 0; i < _liSrcs.length; i++) {
                if (_liSrcRect(i).inflate(2).contains(p)) {
                  setState(() {
                    _from = i;
                    _wire = p;
                  });
                  return;
                }
              }
            },
            onPointerMove: (e) {
              if ((e.localPosition - _down).distance > 3) _moved = true;
              if (_from == null) return;
              setState(() {
                _wire = e.localPosition;
                _over = _tgtAt(w, e.localPosition);
              });
            },
            onPointerUp: (e) {
              final p = e.localPosition, f = _from;
              if (f != null) {
                final t = _tgtAt(w, p);
                setState(() {
                  _from = null;
                  _wire = null;
                  _over = null;
                });
                if (t != null && _moved) {
                  doc.s2['li_l.sel'] = '$t';
                  doc.str('li_l.$t.src', '$f');
                }
                return;
              }
              if (_moved || !on) return;
              final t = _tgtAt(w, p);
              if (t != null) {
                if (links.any((l) => l.$1 == t)) doc.str('li_l.sel', '$t');
                return;
              }
              for (final l in links) {
                final c = wire(l);
                for (var i = 0; i <= 24; i++) {
                  if ((_liBez(c, i / 24) - p).distance < 6) {
                    doc.str('li_l.sel', '${l.$1}');
                    return;
                  }
                }
              }
            },
            onPointerCancel: (_) => setState(() {
              _from = null;
              _wire = null;
              _over = null;
            }),
            child: SizedBox(
              height: _liNodeH,
              child: Stack(
                children: [
                  Positioned.fill(
                    child: CustomPaint(
                      painter: _LiWirePaint([for (final l in links) (wire(l), l.$1 == sel)], _from == null ? null : (_liSrcRect(_from!).centerRight, _wire!)),
                    ),
                  ),
                  for (var i = 0; i < _liSrcs.length; i++)
                    Positioned.fromRect(
                      rect: _liSrcRect(i),
                      child: _LiSrcPill(i, used: links.any((l) => l.$2 == i), live: _from == i, key: ValueKey('li_src$i')),
                    ),
                  for (var t = 0; t < _liTgts.length; t++)
                    Positioned.fromRect(
                      rect: _liTgtRect(w, t),
                      child: _LiTgtPill(
                        t,
                        value(t),
                        driven: links.any((l) => l.$1 == t),
                        hot: _over == t || sel == t && links.any((l) => l.$1 == t),
                        key: ValueKey('li_tgt$t'),
                      ),
                    ),
                  for (final (t, y) in tags)
                    Positioned(
                      left: midX - _liTagW / 2,
                      top: y - 9,
                      width: _liTagW,
                      height: 18,
                      child: _LiTag(t, sel: sel == t),
                    ),
                ],
              ),
            ),
          ),
        );
      },
    );
  }
}

class _LiSrcPill extends StatelessWidget {
  const _LiSrcPill(this.i, {required this.used, required this.live, super.key});
  final int i;
  final bool used, live;
  @override
  Widget build(BuildContext context) => MouseRegion(
    cursor: SystemMouseCursors.grab,
    child: Container(
      padding: const EdgeInsets.only(left: 7, right: 4),
      decoration: BoxDecoration(color: live ? Grey.g26 : Grey.g20, borderRadius: BorderRadius.circular(10)),
      child: Row(
        children: [
          Text(_liSrcs[i].$1, maxLines: 1, style: T.label(Grey.g56)),
          const SizedBox(width: 4),
          Expanded(
            child: Text(
              _liSrcs[i].$2,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: T.label(Grey.g91).copyWith(fontWeight: FontWeight.w600),
            ),
          ),
          Container(
            width: 8,
            height: 8,
            decoration: BoxDecoration(
              color: used || live ? Pop.toneLink : Grey.g07,
              shape: BoxShape.circle,
              border: Border.all(color: Pop.toneLink),
            ),
          ),
        ],
      ),
    ),
  );
}

class _LiTgtPill extends StatelessWidget {
  const _LiTgtPill(this.t, this.v, {required this.driven, required this.hot, super.key});
  final int t;
  final double v;
  final bool driven, hot;
  @override
  Widget build(BuildContext context) => Container(
    padding: const EdgeInsets.only(left: 4, right: 7),
    decoration: BoxDecoration(
      color: hot ? Grey.g26 : Grey.g20,
      borderRadius: BorderRadius.circular(10),
      border: Border.all(color: hot ? Pop.toneLink : const Color(0x00000000)),
    ),
    child: Row(
      children: [
        Container(
          width: 8,
          height: 8,
          decoration: BoxDecoration(
            color: driven ? Pop.toneLink : Grey.g07,
            shape: BoxShape.circle,
            border: Border.all(color: driven ? Pop.toneLink : Grey.g44),
          ),
        ),
        const SizedBox(width: 5),
        Expanded(
          child: Text(
            _liTgts[t].$1,
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
            style: T.label(Grey.g91).copyWith(fontWeight: FontWeight.w600),
          ),
        ),
        Text(brief(v, 1), maxLines: 1, style: T.value(driven ? Pop.toneLink : Grey.g56).copyWith(fontSize: 10)),
      ],
    ),
  );
}

/// The wire's map on the wire: Multiply and Delay, each a sideways scrub; a click selects the link.
class _LiTag extends StatelessWidget {
  const _LiTag(this.t, {required this.sel});
  final int t;
  final bool sel;
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc, f = doc.get('li_l.$t.f') ?? 0;
    void pick() {
      if (doc.s2['li_l.sel'] != '$t') doc.str('li_l.sel', '$t');
    }

    return Container(
      decoration: BoxDecoration(
        color: Grey.g07,
        borderRadius: BorderRadius.circular(9),
        border: Border.all(color: sel ? Pop.toneLink : Grey.g38, width: sel ? 1.4 : 1),
      ),
      child: Row(
        children: [
          Expanded(child: _LiScrub('li_l.$t.mul', '× ${brief(doc.get('li_l.$t.mul'), 2)}', perPx: .01, min: -4, max: 4, dec: 2, onTap: pick)),
          Container(width: 1, height: 10, color: Grey.g38),
          Expanded(child: _LiScrub('li_l.$t.f', f == 0 ? '0 f' : '${_liSigned(f, 0)} f', perPx: .25, min: -120, max: 120, dec: 0, onTap: pick)),
        ],
      ),
    );
  }
}
