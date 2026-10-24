// Part of inspector_idea_paths.dart (cut there by size, as it was).
part of 'inspector_idea_paths.dart';

/// An instance's More: order, duplicate, remove.
class _PiMore extends StatelessWidget {
  const _PiMore(this.lane, this.inst);
  final String lane, inst;
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, o = _piOrder(doc, lane), i = o.indexOf(inst);
    void move(int by) => _piSetOrder(
      doc,
      lane,
      o
        ..removeAt(i)
        ..insert(i + by, inst),
    );
    void picked(String s) => lane == 'a' ? doc.str('pi_a_sel', s) : doc.flag('pi_b_$s.open', true);
    final items = <(String, bool, VoidCallback, bool)>[
      ('Move earlier', i > 0, () => move(-1), false),
      ('Move later', i >= 0 && i < o.length - 1, () => move(1), false),
      ('Duplicate', i >= 0, () => picked(_piAdd(doc, lane, _piKind(inst), at: i + 1, copy: inst)), false),
      ('Remove', i >= 0, () => _piSetOrder(doc, lane, o..remove(inst)), true),
    ];
    return Float(
      right: true,
      width: 160,
      need: items.length * 22 + 8,
      anchor: (open, toggle) => Hov(
        onTap: x.cfg.locked ? null : toggle,
        builder: (_, h) => Padding(
          padding: const EdgeInsets.symmetric(horizontal: 6),
          child: Text('More', key: ValueKey('pi_${lane}_$inst.more'), style: T.label(open || h ? Grey.g95 : Grey.g56)),
        ),
      ),
      menu: (close) => Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          for (final (s, ok, go, danger) in items)
            Hov(
              cursor: ok ? SystemMouseCursors.click : SystemMouseCursors.basic,
              onTap: ok
                  ? () {
                      close();
                      go();
                    }
                  : null,
              builder: (_, h) => Container(
                height: 22,
                padding: const EdgeInsets.symmetric(horizontal: 8),
                decoration: BoxDecoration(color: h && ok ? Grey.g26 : null, borderRadius: BorderRadius.circular(5)),
                alignment: Alignment.centerLeft,
                child: Row(
                  children: [
                    if (danger) const ErrMark(size: 10, gap: 5),
                    Text(s, style: T.name(!ok ? Grey.g56 : (danger ? Role.error : Grey.g91))),
                  ],
                ),
              ),
            ),
        ],
      ),
    );
  }
}

/// The stack's last row: every kind in a floating two-column list; a pick is a new instance at the bottom (a kind already there is fine).
class _PiAdd extends StatelessWidget {
  const _PiAdd(this.lane, {this.card = false});
  final String lane;

  /// Drawn as its own card row (B) rather than a row inside the stack's card (A).
  final bool card;
  static const _itemH = 22.0;
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, kinds = _piKinds.keys.toList(), rows = (kinds.length / 2).ceil();
    return Float(
      need: rows * _itemH + 8,
      anchor: (open, toggle) => Hov(
        key: ValueKey('pi_${lane}_add'),
        onTap: x.cfg.locked ? null : toggle,
        builder: (_, h) => Container(
          height: Pop.row,
          padding: EdgeInsets.symmetric(horizontal: card ? Pop.inset : 6),
          margin: EdgeInsets.only(bottom: card ? Pop.gap : 0),
          decoration: BoxDecoration(color: h || open ? Grey.g15 : (card ? Pop.card : null), borderRadius: card ? null : BorderRadius.circular(6)),
          child: Row(
            children: [
              SizedBox(
                width: 22,
                child: Text('+', style: T.name(h || open ? Grey.g95 : Grey.g63).copyWith(fontWeight: FontWeight.w700)),
              ),
              Text('Add path operation', style: T.name(x.cfg.locked ? Grey.g44 : (h || open ? Grey.g95 : Grey.g76))),
              const Spacer(),
              if (open) Text('Close', style: T.label(Grey.g56)),
            ],
          ),
        ),
      ),
      menu: (close) => Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          for (var r = 0; r < rows; r++)
            Row(
              children: [
                for (final k in kinds.skip(r * 2).take(2))
                  Expanded(
                    child: Hov(
                      key: ValueKey('pi_${lane}_add_$k'),
                      onTap: () {
                        close();
                        final s = _piAdd(doc, lane, k);
                        lane == 'a' ? doc.str('pi_a_sel', s) : doc.flag('pi_b_$s.open', true);
                      },
                      builder: (_, h) => Container(
                        height: _itemH,
                        padding: const EdgeInsets.symmetric(horizontal: 6),
                        decoration: BoxDecoration(color: h ? Grey.g26 : null, borderRadius: BorderRadius.circular(5)),
                        child: Row(
                          children: [
                            OpGlyph(k, hot: h),
                            const SizedBox(width: 6),
                            Expanded(
                              child: Text(_piKinds[k]!.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(h ? Grey.g95 : Grey.g76)),
                            ),
                          ],
                        ),
                      ),
                    ),
                  ),
                if (r * 2 + 1 >= kinds.length) const Spacer(),
              ],
            ),
        ],
      ),
    );
  }
}

/// An op's numbers, small and exact, under whatever does it by hand.
class _PiFields extends StatelessWidget {
  const _PiFields(this.lane, this.inst);
  final String lane, inst;
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc, k = 'pi_${lane}_$inst', g = _piG(doc, lane, inst), op = _piKind(inst);
    NumField f(String p, String l, {String u = '', double? min, double? max, int dec = 0, double perPx = 1, bool bi = false, bool whole = false}) => NumField(
      id: '${k}_$p',
      label: l,
      unit: u,
      min: min,
      max: max,
      decimals: dec,
      perPx: perPx,
      bipolar: bi,
      onSet: whole ? (v) => doc.set('${k}_$p', v.roundToDouble()) : null,
    );
    Widget seg(String p, List<String> o) => Dis(
      child: Segmented(items: o, index: g(p).round().clamp(0, o.length - 1), expand: true, onChanged: (i) => doc.set('${k}_$p', i.toDouble())),
    );
    final (List<Widget> cells, Widget? choice) = switch (op) {
      'round' => ([f('radius', 'Radius', u: 'px', min: 0, dec: 1, perPx: .5)], null),
      'pucker' => ([f('amount', 'Amount', u: '%', min: -100, max: 100, bi: true)], null),
      'zigzag' => ([f('amp', 'Size', u: 'px', dec: 1, perPx: .25), f('ridges', 'Ridges', min: 0, max: 60, perPx: .1, whole: true)], seg('pt', points)),
      'offset' => (
        [f('amount', 'Amount', u: 'px', dec: 1, perPx: .25), if (g('join') == 0) f('miter', 'Miter', min: 1, dec: 1, perPx: .05)],
        seg('join', joins),
      ),
      'twist' => ([f('angle', 'Angle', u: '°'), f('cx', 'X', u: 'px'), f('cy', 'Y', u: 'px')], null),
      'wiggle' => (
        [f('size', 'Size', min: 0, dec: 1, perPx: .25), f('detail', 'Detail', min: 1, max: 100, perPx: .25, whole: true), f('phase', 'Phase', u: '°')],
        seg('pt', points),
      ),
      'trim' => ([f('start', 'Start', u: '%', min: 0, max: 100), f('end', 'End', u: '%', min: 0, max: 100), f('offset', 'Offset', u: '°')], null),
      _ => (const <Widget>[], null),
    };
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Row(
          children: [
            for (final (i, c) in cells.indexed) ...[if (i > 0) const SizedBox(width: 4), Expanded(child: c)],
          ],
        ),
        if (choice != null) ...[const SizedBox(height: 4), choice],
        if (op == 'wiggle') AdvancedFold('$k.adv', summary: 'Seed ${brief(g('seed'), 0)}', [f('seed', 'Seed', min: 0, max: 9999, perPx: .1, whole: true)]),
      ],
    );
  }
}

/// Brief chips that drop whole ones rather than clip.
class _PiChips extends StatelessWidget {
  const _PiChips(this.chips, {this.dim = false});
  final List<String> chips;
  final bool dim;
  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, box) {
      var used = 0.0;
      final fit = <String>[];
      for (final t in chips) {
        final w = BriefChip.width(t) + (fit.isEmpty ? 0 : 4);
        if (used + w > box.maxWidth) break;
        used += w;
        fit.add(t);
      }
      return Row(
        children: [
          for (final (i, t) in fit.indexed) ...[if (i > 0) const SizedBox(width: 4), BriefChip(t, dim: dim)],
        ],
      );
    },
  );
}

// ---- the stage: a path view with handles ----------------------------------------------------------------------------
class _PiView {
  _PiView(Size s, Rect m) : k = math.min(s.width / m.width, s.height / m.height), c = s.center(Offset.zero), m = m.center;
  final double k;
  final Offset c, m;
  Offset on(Offset p) => c + (p - m) * k;
  Offset back(Offset p) => m + (p - c) / k;
}

Path _piPath(List<_PiC> cs, _PiView v) {
  final p = Path();
  for (final c in cs) {
    final f = c.flat;
    if (f.length > 1) p.addPolygon([for (final q in f) v.on(q)], c.closed);
  }
  return p;
}

/// A handle in model space: where it sits, what a drag to a model point writes, an optional guide line from [from].
class _PiH {
  const _PiH(this.id, this.at, this.drag, {this.tip, this.from, this.shape = 0, Color? color}) : _color = color;
  final Color? _color;
  Color get color => _color ?? Grey.g95;
  final String id;
  final Offset at;
  final void Function(Offset m) drag;
  final String Function()? tip;
  final Offset? from;

  /// 0 dot, 1 diamond, 2 ring, 3 tab.
  final int shape;
}

class _PiInk {
  const _PiInk(this.cs, this.color, this.w, {this.side = 0});
  final List<_PiC> cs;
  final Color color;
  final double w;

  /// -1 drawn left of the wipe only, 1 right of it only.
  final int side;
}

/// contract: repaints only when rebuilt (a value changed) or on pointer input; one drag = one undo step.
class _PiStage extends StatefulWidget {
  const _PiStage({
    super.key,
    required this.inks,
    required this.bounds,
    this.handles = const [],
    this.height = 150,
    this.marks = const [],
    this.rings = const [],
    this.wipe,
    this.overlay,
    this.note,
  });
  final List<_PiInk> inks;
  final List<_PiH> handles;
  final double height;
  final Rect bounds;
  final List<(Offset, Color, double)> marks;
  final List<(Offset, double)> rings;
  final double? wipe;
  final Widget? overlay;
  final String? note;
  @override
  State<_PiStage> createState() => _PiStageState();
}

class _PiStageState extends State<_PiStage> {
  String? _hot, _drag;
  _PiH? _find(String? id) => id == null ? null : widget.handles.where((h) => h.id == id).firstOrNull;
  String? _pick(_PiView v, Offset p) {
    String? best;
    var bd = 11.0;
    for (final h in widget.handles) {
      final d = (v.on(h.at) - p).distance;
      if (d < bd) {
        bd = d;
        best = h.id;
      }
    }
    return best;
  }

  void _end(Doc doc) {
    if (_drag == null) return;
    doc.endGesture();
    setState(() => _drag = null);
  }

  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, on = !x.cfg.locked;
    return LayoutBuilder(
      builder: (context, box) {
        final v = _PiView(Size(box.maxWidth, widget.height), widget.bounds), tip = _find(_drag ?? _hot)?.tip?.call() ?? widget.note;
        return MouseRegion(
          cursor: !on
              ? SystemMouseCursors.basic
              : (_drag != null ? SystemMouseCursors.grabbing : (_hot != null ? SystemMouseCursors.grab : SystemMouseCursors.basic)),
          onHover: (e) {
            final h = on ? _pick(v, e.localPosition) : null;
            if (h != _hot) setState(() => _hot = h);
          },
          onExit: (_) {
            if (_hot != null) setState(() => _hot = null);
          },
          child: Listener(
            behavior: HitTestBehavior.opaque,
            onPointerDown: (e) {
              if (!on || e.buttons != kPrimaryButton) return;
              final h = _pick(v, e.localPosition);
              if (h == null) return;
              doc.beginGesture();
              setState(() => _drag = h);
            },
            onPointerMove: (e) => _find(_drag)?.drag(v.back(e.localPosition)),
            onPointerUp: (_) => _end(doc),
            onPointerCancel: (_) => _end(doc),
            child: SizedBox(
              height: widget.height,
              child: Stack(
                children: [
                  Positioned.fill(child: CustomPaint(painter: _PiStagePaint(widget, v, _drag ?? _hot))),
                  for (final h in widget.handles)
                    Positioned(
                      key: ValueKey('h:${h.id}'),
                      left: v.on(h.at).dx - 6,
                      top: v.on(h.at).dy - 6,
                      width: 12,
                      height: 12,
                      child: const IgnorePointer(child: SizedBox()),
                    ),
                  if (widget.overlay != null) Positioned(top: 6, right: 6, child: widget.overlay!),
                  if (tip != null)
                    Positioned(
                      left: 8,
                      bottom: 6,
                      child: IgnorePointer(child: Text(tip, style: T.label(_drag != null ? Grey.g95 : Grey.g63))),
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

class _PiStagePaint extends CustomPainter {
  _PiStagePaint(this.st, this.v, this.hot);
  final _PiStage st;
  final _PiView v;
  final String? hot;
  @override
  void paint(Canvas c, Size s) {
    c.drawRect(Offset.zero & s, Paint()..color = Grey.g10);
    final b = st.bounds, grid = Paint()..color = Grey.g20;
    for (var gx = (b.left / 20).ceil() * 20.0; gx <= b.right; gx += 20) {
      for (var gy = (b.top / 20).ceil() * 20.0; gy <= b.bottom; gy += 20) {
        c.drawCircle(v.on(Offset(gx, gy)), .8, grid);
      }
    }
    final w = st.wipe == null ? null : v.on(Offset(st.wipe!, 0)).dx;
    for (final k in st.inks) {
      c.save();
      if (w != null && k.side != 0) c.clipRect(k.side < 0 ? Rect.fromLTRB(0, 0, w, s.height) : Rect.fromLTRB(w, 0, s.width, s.height));
      c.drawPath(
        _piPath(k.cs, v),
        Paint()
          ..color = k.color
          ..style = PaintingStyle.stroke
          ..strokeWidth = k.w
          ..strokeCap = StrokeCap.round
          ..strokeJoin = StrokeJoin.round,
      );
      c.restore();
    }
    final line = Paint()
      ..color = Grey.g44
      ..strokeWidth = 1
      ..style = PaintingStyle.stroke;
    if (w != null) {
      c.drawLine(Offset(w, 0), Offset(w, s.height), line);
      for (final (t, right) in const [('BEFORE', false), ('AFTER', true)]) {
        final tp = TextPainter(
          text: TextSpan(text: t, style: T.micro(Grey.g44)),
          textDirection: TextDirection.ltr,
        )..layout();
        tp.paint(c, Offset(right ? s.width - tp.width - 8 : 8, 7));
      }
    }
    for (final (a, r) in st.rings) {
      c.drawCircle(v.on(a), r * v.k, line..color = Grey.g26);
    }
    for (final (a, col, r) in st.marks) {
      c.drawCircle(v.on(a), r, Paint()..color = col);
    }
    final edge = Paint()
      ..color = Grey.g07
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.5;
    for (final h in st.handles) {
      final q = v.on(h.at), big = h.id == hot, r = big ? 5.5 : 4.5, fill = Paint()..color = big ? Pop.accent : h.color;
      if (h.from != null) c.drawLine(v.on(h.from!), q, line..color = Grey.g56);
      switch (h.shape) {
        case 1:
          final d = Path()..addPolygon([q + Offset(0, -r - 1), q + Offset(r + 1, 0), q + Offset(0, r + 1), q + Offset(-r - 1, 0)], true);
          c.drawPath(d, fill);
          c.drawPath(d, edge);
        case 2:
          c.drawCircle(q, r, edge..strokeWidth = 4);
          c.drawCircle(
            q,
            r,
            Paint()
              ..color = fill.color
              ..style = PaintingStyle.stroke
              ..strokeWidth = 2,
          );
          edge.strokeWidth = 1.5;
        case 3:
          final rr = RRect.fromRectAndRadius(Rect.fromCenter(center: q, width: 10, height: 14), const Radius.circular(3));
          c.drawRRect(rr, fill);
          c.drawRRect(rr, edge);
        default:
          c.drawCircle(q, r, fill);
          c.drawCircle(q, r, edge);
      }
    }
  }

  @override
  bool shouldRepaint(_PiStagePaint o) => true;
}

class _PiThumb extends StatelessWidget {
  const _PiThumb(this.cs, this.ink, {this.w, this.h});
  final List<_PiC> cs;
  final Color ink;
  final double? w, h;
  @override
  Widget build(BuildContext context) =>
      CustomPaint(size: Size(w ?? 0, h ?? 0), painter: _PiThumbPaint(cs, ink), child: w == null ? const SizedBox.expand() : null);
}
