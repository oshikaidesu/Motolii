// Part of inspector_idea_paths.dart (cut there by size, as it was).
part of 'inspector_idea_paths.dart';

class _PiCTrimState extends State<_PiCTrim> {
  double? _peek;
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    doc.seed(
      {for (final e in _piPresets['Write & erase']!.entries) 'pi_c_${e.key}': e.value, 'pi_c_off': 0, 'pi_c_ease': 1, 'pi_c_t': .8},
      strs: const {'pi_c_sel': 'e1'},
    );
    final vals = _piCVals(doc), ease = (doc.get('pi_c_ease') ?? 0).round().clamp(0, 2), off = doc.get('pi_c_off') ?? 0;
    final t = _peek ?? doc.get('pi_c_t') ?? 0, s = _piLane(vals, 's', t, ease), e = _piLane(vals, 'e', t, ease);
    final open = doc.b['pi_c.open'] ?? true, sel = doc.s2['pi_c_sel']!;
    final p = _piScript.flat, l = _piLen(p);
    Offset at(double v) {
      final f = v / 100 + off / 360, r = f - f.floorToDouble();
      return _piAt(p, (r == 0 && f > 0 ? 1 : r) * l).$1;
    }

    final marks = <(Offset, Color, double)>[
      for (final lane in const ['s', 'e'])
        for (var f = 0; f <= 60; f += 2)
          if (f / 30 >= math.min(vals['${lane}0t']!, vals['${lane}1t']!) && f / 30 <= math.max(vals['${lane}0t']!, vals['${lane}1t']!))
            (at(_piLane(vals, lane, f / 30, ease)), lane == 'e' ? Grey.g56 : Grey.g38, 1.4),
      (at(s), Grey.g76, 3),
      (at(e), Pop.accent, 3.5),
    ];
    final preset = _piPresets.entries.where((m) => m.value.entries.every((x) => (vals[x.key]! - x.value).abs() < 1e-6)).firstOrNull?.key;
    final keyName = '${sel[0] == 's' ? 'Start' : 'End'} ${sel[1] == '0' ? 1 : 2}';
    return Sect(
      title: 'Trim Paths',
      tone: Pop.tonePath,
      mark: CardMark.path,
      open: open,
      onToggle: () => doc.flag('pi_c.open', !open),
      keyIds: const ['pi_c'],
      brief: [preset ?? 'Custom', _piEases[ease], '${brief(math.max(vals['s1t']!, vals['e1t']!), 1)}s'],
      children: [
        _PiStage(
          key: const ValueKey('pi_c_stage'),
          height: 120,
          bounds: _piBoundsC,
          inks: [
            _PiInk([_piScript], Grey.g26, 1.2),
            _PiInk(_piTrim(_piScript, s, e, off), Pop.tonePath, 2.6),
          ],
          marks: marks,
          note: '${t.toStringAsFixed(2)}s${_peek != null ? ' (hover)' : ''} · Start ${brief(s, 0)}% · End ${brief(e, 0)}%',
        ),
        const SizedBox(height: 6),
        _PiTimeline(
          onPeek: (v) {
            if (v != _peek) setState(() => _peek = v);
          },
        ),
        const SizedBox(height: 6),
        PanelLink(
          to: LinkTo.ease,
          label: 'Ease',
          value: _piEases[ease],
          lead: CustomPaint(size: const Size(16, 12), painter: _PiEaseGlyph(ease, Pop.tonePath)),
        ),
        const SizedBox(height: 6),
        Row(
          children: [
            for (final (i, name) in _piPresets.keys.indexed) ...[
              if (i > 0) const SizedBox(width: 4),
              Expanded(
                child: Hov(
                  onTap: () {
                    doc.beginGesture();
                    doc.setMany({for (final x in _piPresets[name]!.entries) 'pi_c_${x.key}': x.value});
                    doc.endGesture();
                  },
                  builder: (_, h) => Container(
                    height: 20,
                    padding: const EdgeInsets.symmetric(horizontal: 6),
                    alignment: Alignment.center,
                    decoration: BoxDecoration(
                      color: preset == name ? Grey.g26 : (h ? Grey.g20 : Pop.well),
                      borderRadius: BorderRadius.circular(10),
                      border: Border.all(color: preset == name ? Pop.accentInk.withValues(alpha: .7) : const Color(0x00000000)),
                    ),
                    child: Text(name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(preset == name || h ? Grey.g95 : Grey.g76)),
                  ),
                ),
              ),
            ],
          ],
        ),
        const SizedBox(height: 6),
        Row(
          children: [
            SizedBox(
              width: _PiTl.lab + 12,
              child: Text(keyName, style: T.label(Grey.g76).copyWith(fontWeight: FontWeight.w600)),
            ),
            Expanded(
              child: NumField(id: 'pi_c_${sel}t', label: 'Time', unit: 's', decimals: 2, min: 0, max: _PiTl.dur, perPx: .01, step: 1 / 30),
            ),
            const SizedBox(width: 4),
            Expanded(
              child: NumField(id: 'pi_c_${sel}v', label: 'Value', unit: '%', min: 0, max: 100),
            ),
            const SizedBox(width: 4),
            const Expanded(
              child: NumField(id: 'pi_c_off', label: 'Offset', unit: '°'),
            ),
          ],
        ),
      ],
    );
  }
}

class _PiTl {
  _PiTl(this.w);
  final double w;
  static const dur = 2.0, lab = 36.0, ruler = 14.0, laneH = 24.0, gap = 4.0, h = ruler + laneH * 2 + gap;
  double tx(double t) => lab + 5 + t / dur * (w - lab - 10);
  double xt(double x) => ((x - lab - 5) / (w - lab - 10) * dur).clamp(0.0, dur);
  double top(int i) => ruler + i * (laneH + gap);
  double vy(int i, double v) => top(i) + 3 + (1 - v / 100) * (laneH - 6);
  double yv(int i, double y) => ((1 - (y - top(i) - 3) / (laneH - 6)) * 100).clamp(0.0, 100.0);
  Offset key(String k, Map<String, double> v) => Offset(tx(v['${k}t']!), vy(k[0] == 's' ? 0 : 1, v['${k}v']!));
}

/// Start and End as two lanes of value over time: drag a key (time sideways, value up and down), press elsewhere to move the playhead.
/// contract: hovering reports the time under the pointer (the stage previews it); nothing runs between pointer events.
class _PiTimeline extends StatefulWidget {
  const _PiTimeline({required this.onPeek});
  final ValueChanged<double?> onPeek;
  @override
  State<_PiTimeline> createState() => _PiTimelineState();
}

class _PiTimelineState extends State<_PiTimeline> {
  String? _drag, _hot;
  bool _scrub = false;
  double? _hover;

  String? _pick(_PiTl g, Map<String, double> v, Offset p) {
    String? best;
    var bd = 9.0;
    for (final k in _piCKeys) {
      final d = (g.key(k, v) - p).distance;
      if (d < bd) {
        bd = d;
        best = k;
      }
    }
    return best;
  }

  void _end(Doc doc) {
    if (_drag == null && !_scrub) return;
    doc.endGesture();
    setState(() {
      _drag = null;
      _scrub = false;
    });
    widget.onPeek(_hover);
  }

  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, on = !x.cfg.locked;
    return LayoutBuilder(
      builder: (context, box) {
        final g = _PiTl(box.maxWidth), v = _piCVals(doc), t = doc.get('pi_c_t') ?? 0;
        double snap(double s) => (s * 30).roundToDouble() / 30;
        void move(Offset p) {
          final k = _drag;
          if (k != null) {
            final other = v['${k[0]}${k[1] == '0' ? 1 : 0}t']!;
            final tt = (k[1] == '0' ? math.min(snap(g.xt(p.dx)), other - 1 / 30) : math.max(snap(g.xt(p.dx)), other + 1 / 30)).clamp(0.0, _PiTl.dur);
            doc.setMany({'pi_c_${k}t': tt, 'pi_c_${k}v': g.yv(k[0] == 's' ? 0 : 1, p.dy).roundToDouble()});
            widget.onPeek(tt);
          } else if (_scrub) {
            doc.set('pi_c_t', snap(g.xt(p.dx)));
          }
        }

        return MouseRegion(
          cursor: !on ? SystemMouseCursors.basic : (_drag != null || _hot != null ? SystemMouseCursors.move : SystemMouseCursors.precise),
          onHover: (e) {
            final ht = e.localPosition.dx >= _PiTl.lab ? g.xt(e.localPosition.dx) : null;
            setState(() {
              _hot = on ? _pick(g, v, e.localPosition) : null;
              _hover = ht;
            });
            widget.onPeek(ht);
          },
          onExit: (_) {
            setState(() {
              _hot = null;
              _hover = null;
            });
            if (_drag == null && !_scrub) widget.onPeek(null);
          },
          child: Listener(
            behavior: HitTestBehavior.opaque,
            onPointerDown: (e) {
              if (!on || e.buttons != kPrimaryButton) return;
              final k = _pick(g, v, e.localPosition);
              if (k == null && e.localPosition.dx < _PiTl.lab) return;
              if (k != null) doc.str('pi_c_sel', k);
              doc.beginGesture();
              setState(() {
                _drag = k;
                _scrub = k == null;
              });
              if (k == null) move(e.localPosition);
            },
            onPointerMove: (e) => move(e.localPosition),
            onPointerUp: (_) => _end(doc),
            onPointerCancel: (_) => _end(doc),
            child: SizedBox(
              height: _PiTl.h,
              child: Stack(
                children: [
                  Positioned.fill(
                    child: CustomPaint(painter: _PiTlPaint(g, v, (doc.get('pi_c_ease') ?? 0).round(), doc.s2['pi_c_sel'] ?? '', t, _hover, _drag ?? _hot)),
                  ),
                  for (final k in _piCKeys)
                    Positioned(
                      key: ValueKey('h:pi_c_$k'),
                      left: g.key(k, v).dx - 6,
                      top: g.key(k, v).dy - 6,
                      width: 12,
                      height: 12,
                      child: const IgnorePointer(child: SizedBox()),
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

class _PiTlPaint extends CustomPainter {
  _PiTlPaint(this.g, this.v, this.ease, this.sel, this.t, this.hover, this.hot);
  final _PiTl g;
  final Map<String, double> v;
  final int ease;
  final String sel;
  final double t;
  final double? hover;
  final String? hot;

  void _text(Canvas c, String s, Offset at, Color col, {bool centre = false}) {
    final tp = TextPainter(
      text: TextSpan(text: s, style: T.micro(col)),
      textDirection: TextDirection.ltr,
    )..layout();
    tp.paint(c, at - Offset(centre ? tp.width / 2 : 0, tp.height / 2));
  }

  @override
  void paint(Canvas c, Size s) {
    final tick = Paint()
      ..color = Grey.g38
      ..strokeWidth = 1;
    for (var q = 0; q <= 8; q++) {
      final x = g.tx(q * .25);
      c.drawLine(Offset(x, _PiTl.ruler - (q % 4 == 0 ? 6 : 3)), Offset(x, _PiTl.ruler - 1), tick);
      if (q % 4 == 0) _text(c, '${q ~/ 4}s', Offset((x + (q == 0 ? 8 : (q == 8 ? -8 : 0))).toDouble(), 4), Grey.g56, centre: true);
    }
    for (final (i, lane, name) in const [(0, 's', 'Start'), (1, 'e', 'End')]) {
      final top = g.top(i);
      c.drawRRect(
        RRect.fromRectAndRadius(Rect.fromLTWH(_PiTl.lab, top, s.width - _PiTl.lab, _PiTl.laneH), const Radius.circular(4)),
        Paint()..color = Grey.g10,
      );
      _text(c, name, Offset(0, top + _PiTl.laneH / 2), Grey.g63);
      final curve = Path(), fillP = Path()..moveTo(g.tx(0), top + _PiTl.laneH);
      for (var k = 0; k <= 60; k++) {
        final q = Offset(g.tx(k / 30), g.vy(i, _piLane(v, lane, k / 30, ease)));
        k == 0 ? curve.moveTo(q.dx, q.dy) : curve.lineTo(q.dx, q.dy);
        fillP.lineTo(q.dx, q.dy);
      }
      fillP
        ..lineTo(g.tx(_PiTl.dur), top + _PiTl.laneH)
        ..close();
      c.drawPath(fillP, Paint()..color = Pop.tonePath.withValues(alpha: .12));
      c.drawPath(
        curve,
        Paint()
          ..color = Pop.tonePath.withValues(alpha: .9)
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1.4,
      );
    }
    final ph = g.tx(t);
    c.drawLine(Offset(ph, 0), Offset(ph, s.height), Paint()..color = Pop.accentInk);
    c.drawPath(Path()..addPolygon([Offset(ph - 4, 0), Offset(ph + 4, 0), Offset(ph, 5)], true), Paint()..color = Pop.accentInk);
    final hv = hover;
    if (hv != null && (g.tx(hv) - ph).abs() > 1) {
      final hx = g.tx(hv);
      c.drawLine(Offset(hx, _PiTl.ruler), Offset(hx, s.height), Paint()..color = Grey.g63);
      final tp = TextPainter(
        text: TextSpan(text: '${hv.toStringAsFixed(2)}s', style: T.micro(Grey.g95)),
        textDirection: TextDirection.ltr,
      )..layout();
      final r = Rect.fromLTWH((hx - tp.width / 2 - 3).clamp(_PiTl.lab, s.width - tp.width - 6), 0, tp.width + 6, 12);
      c.drawRRect(RRect.fromRectAndRadius(r, const Radius.circular(3)), Paint()..color = Grey.g26);
      tp.paint(c, r.topLeft + Offset(3, (12 - tp.height) / 2));
    }
    for (final k in _piCKeys) {
      final q = g.key(k, v), r = k == hot ? 6.0 : 4.5;
      final d = Path()..addPolygon([q + Offset(0, -r), q + Offset(r, 0), q + Offset(0, r), q + Offset(-r, 0)], true);
      c.drawPath(d, Paint()..color = Pop.keyDot);
      c.drawPath(
        d,
        Paint()
          ..color = k == sel ? Grey.g100 : Grey.g07
          ..style = PaintingStyle.stroke
          ..strokeWidth = k == sel ? 1.6 : 1.2,
      );
    }
  }

  @override
  bool shouldRepaint(_PiTlPaint o) => true;
}
