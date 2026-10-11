// The desk's Ease tool: the curve between the picked key and the next one, its two handles, presets as thumbnails, and the numbers.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../sets/inspector/inspector_parts.dart' show Hov;
import '../tokens.dart';
import 'desk_kit.dart';
import 'ws.dart';

enum EaseKind { bezier, bounce, hold }

/// One ease: a kind, and the two bezier handles (kept even while another kind shows, so grabbing a handle returns to bezier).
class Ease {
  const Ease(this.kind, this.p1, this.p2, [this.name]);
  final EaseKind kind;
  final Offset p1, p2;
  final String? name;

  double at(double x) => switch (kind) {
    EaseKind.bezier => bezierY(p1, p2, x),
    EaseKind.bounce => _bounce(x),
    EaseKind.hold => x < 1 ? 0 : 1,
  };

  Ease withHandles(Offset a, Offset b) => Ease(EaseKind.bezier, a, b);
  Ease reversed() => Ease(kind, Offset(1 - p2.dx, 1 - p2.dy), Offset(1 - p1.dx, 1 - p1.dy), name);

  @override
  bool operator ==(Object other) => other is Ease && other.kind == kind && other.p1 == p1 && other.p2 == p2;
  @override
  int get hashCode => Object.hash(kind, p1, p2);
}

const easePresets = [
  Ease(EaseKind.bezier, Offset(0, 0), Offset(1, 1), 'Linear'),
  Ease(EaseKind.bezier, Offset(.25, .1), Offset(.25, 1), 'Ease'),
  Ease(EaseKind.bezier, Offset(.42, 0), Offset(1, 1), 'In'),
  Ease(EaseKind.bezier, Offset(0, 0), Offset(.58, 1), 'Out'),
  Ease(EaseKind.bezier, Offset(.42, 0), Offset(.58, 1), 'In-out'),
  Ease(EaseKind.bezier, Offset(.34, 1.56), Offset(.64, 1), 'Back'),
  Ease(EaseKind.bounce, Offset(.25, .1), Offset(.25, 1), 'Bounce'),
  Ease(EaseKind.hold, Offset(.25, .1), Offset(.25, 1), 'Hold'),
];

String? easeName(Ease e) => easePresets.where((p) => p == e).firstOrNull?.name;

double bezierY(Offset p1, Offset p2, double x) {
  double bx(double u) => 3 * (1 - u) * (1 - u) * u * p1.dx + 3 * (1 - u) * u * u * p2.dx + u * u * u;
  var lo = 0.0, hi = 1.0;
  for (var i = 0; i < 24; i++) {
    final m = (lo + hi) / 2;
    if (bx(m) < x) {
      lo = m;
    } else {
      hi = m;
    }
  }
  final u = (lo + hi) / 2;
  return 3 * (1 - u) * (1 - u) * u * p1.dy + 3 * (1 - u) * u * u * p2.dy + u * u * u;
}

double _bounce(double x) {
  const n = 7.5625, d = 2.75;
  if (x < 1 / d) return n * x * x;
  if (x < 2 / d) return n * (x -= 1.5 / d) * x + .75;
  if (x < 2.5 / d) return n * (x -= 2.25 / d) * x + .9375;
  return n * (x -= 2.625 / d) * x + .984375;
}

/// The stretch of time the ease shapes: the first picked key and the layer's next key.
typedef EaseSpan = ({WsLayer layer, int from, int to, int picked});

EaseSpan? easeSpan(Ws ws) {
  if (ws.keys.isEmpty) return null;
  final picked = ws.keys.toList()..sort((a, b) => a.frame.compareTo(b.frame));
  final k = picked.first, layer = ws.layers.where((l) => l.id == k.layer).firstOrNull;
  if (layer == null) return null;
  final keys = [...layer.keys]..sort();
  final next = keys.where((f) => f > k.frame).firstOrNull, prev = keys.where((f) => f < k.frame).lastOrNull;
  return next != null
      ? (layer: layer, from: k.frame, to: next, picked: picked.length)
      : (layer: layer, from: prev ?? math.max(0, k.frame - 30), to: k.frame, picked: picked.length);
}

/// The eases the user shaped, per document and per span, so leaving the desk and coming back keeps them.
final _eases = Expando<Map<String, Ease>>();
String _spanId(EaseSpan? s) => s == null ? 'new' : '${s.layer.id}:${s.from}';

class DeskEase extends StatefulWidget {
  const DeskEase({super.key});
  @override
  State<DeskEase> createState() => _DeskEaseState();
}

class _DeskEaseState extends State<DeskEase> {
  @override
  Widget build(BuildContext context) {
    final ws = WsScope.of(context), span = easeSpan(ws), store = _eases[ws] ??= {}, id = _spanId(span);
    final ease = store[id] ?? easePresets[1];
    void set(Ease e) => setState(() => store[id] = e);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        _SpanStrip(span: span, onReverse: () => set(ease.reversed())),
        const SizedBox(height: WsT.gutter),
        Expanded(
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Expanded(
                child: ColoredBox(
                  color: WsT.card,
                  child: _EaseGraph(ease: ease, span: span, frame: ws.frame, onChanged: set),
                ),
              ),
              const SizedBox(width: WsT.gutter),
              SizedBox(
                width: 128,
                child: _EaseNumbers(ease: ease, span: span, onChanged: set),
              ),
            ],
          ),
        ),
        const SizedBox(height: WsT.gutter),
        _Presets(ease: ease, onPick: set),
      ],
    );
  }
}

/// Which keys: the layer (in its hue), from and to frames, the span's length, and Reverse.
class _SpanStrip extends StatelessWidget {
  const _SpanStrip({required this.span, required this.onReverse});
  final EaseSpan? span;
  final VoidCallback onReverse;
  @override
  Widget build(BuildContext context) {
    final s = span, ws = WsScope.of(context);
    final frames = s == null ? 0 : s.to - s.from;
    return DeskStrip(
      left: [
        if (s == null)
          Text('No keys picked · shapes new keys', style: T.label(Grey.g63))
        else ...[
          DeskLayerChip(s.layer.name, wsTone(s.layer.kind)),
          const SizedBox(width: 8),
          _KeyMark(),
          const SizedBox(width: 4),
          Text('${s.from}', style: T.value(Grey.g91)),
          const SizedBox(width: 6),
          CustomPaint(size: const Size(14, 8), painter: _Arrow(Grey.g56)),
          const SizedBox(width: 6),
          _KeyMark(),
          const SizedBox(width: 4),
          Text('${s.to}', style: T.value(Grey.g91)),
          const SizedBox(width: 8),
          Text('$frames f · ${(frames / Ws.fps).toStringAsFixed(2)} s${s.picked > 1 ? ' · ${s.picked} keys' : ''}', style: T.label(Grey.g56)),
        ],
      ],
      right: [
        if (s != null) ...[DeskChip(label: ws.timecode(s.from)), const SizedBox(width: WsT.gap)],
        DeskIconButton(size: 18, glyph: (ink) => _ReverseGlyph(ink), onTap: onReverse),
      ],
    );
  }
}

class _KeyMark extends StatelessWidget {
  @override
  Widget build(BuildContext context) => Transform.rotate(
    angle: math.pi / 4,
    child: Container(width: 6, height: 6, color: WsT.keyDot),
  );
}

class _Arrow extends CustomPainter {
  const _Arrow(this.ink);
  final Color ink;
  @override
  void paint(Canvas c, Size s) {
    final p = Paint()
      ..color = ink
      ..strokeWidth = 1.2
      ..style = PaintingStyle.stroke
      ..strokeCap = StrokeCap.round;
    final y = s.height / 2;
    c.drawLine(Offset(0, y), Offset(s.width, y), p);
    c.drawPath(
      Path()
        ..moveTo(s.width - 3.5, y - 3)
        ..lineTo(s.width, y)
        ..lineTo(s.width - 3.5, y + 3),
      p,
    );
  }

  @override
  bool shouldRepaint(_Arrow o) => o.ink != ink;
}

class _ReverseGlyph extends CustomPainter {
  const _ReverseGlyph(this.ink);
  final Color ink;
  @override
  void paint(Canvas c, Size s) {
    final p = Paint()
      ..color = ink
      ..strokeWidth = 1.3
      ..style = PaintingStyle.stroke
      ..strokeCap = StrokeCap.round
      ..strokeJoin = StrokeJoin.round;
    final u = s.width / 18;
    c.drawLine(Offset(5 * u, 7 * u), Offset(13 * u, 7 * u), p);
    c.drawPath(
      Path()
        ..moveTo(11 * u, 5 * u)
        ..lineTo(13 * u, 7 * u)
        ..lineTo(11 * u, 9 * u),
      p,
    );
    c.drawLine(Offset(5 * u, 11.5 * u), Offset(13 * u, 11.5 * u), p);
    c.drawPath(
      Path()
        ..moveTo(7 * u, 9.5 * u)
        ..lineTo(5 * u, 11.5 * u)
        ..lineTo(7 * u, 13.5 * u),
      p,
    );
  }

  @override
  bool shouldRepaint(_ReverseGlyph o) => o.ink != ink;
}

/// The graph: value up, time across. Handles are dragged; only a drag or hover repaints it.
class _EaseGraph extends StatefulWidget {
  const _EaseGraph({required this.ease, required this.span, required this.frame, required this.onChanged});
  final Ease ease;
  final EaseSpan? span;
  final int frame;
  final ValueChanged<Ease> onChanged;
  @override
  State<_EaseGraph> createState() => _EaseGraphState();
}

class _EaseGraphState extends State<_EaseGraph> {
  int? _drag, _hot;
  static const _ymin = -.42, _ymax = 1.62;

  static Rect _box(Size s) {
    final r = const EdgeInsets.fromLTRB(30, 10, 14, 22).deflateRect(Offset.zero & s);
    return Rect.fromLTRB(r.left, r.top, math.max(r.left + 1, r.right), math.max(r.top + 1, r.bottom));
  }

  static Offset _px(Rect g, Offset v) => Offset(g.left + v.dx * g.width, g.bottom - (v.dy - _ymin) / (_ymax - _ymin) * g.height);
  static Offset _val(Rect g, Offset p) =>
      Offset(((p.dx - g.left) / g.width).clamp(0.0, 1.0), (_ymin + (g.bottom - p.dy) / g.height * (_ymax - _ymin)).clamp(_ymin + .02, _ymax - .02));

  int? _near(Rect g, Offset p) {
    final d1 = (_px(g, widget.ease.p1) - p).distance, d2 = (_px(g, widget.ease.p2) - p).distance;
    return math.min(d1, d2) > 14 ? null : (d1 <= d2 ? 0 : 1);
  }

  void _move(Rect g, Offset p) {
    final v = _val(g, p), e = widget.ease;
    widget.onChanged(_drag == 0 ? e.withHandles(v, e.p2) : e.withHandles(e.p1, v));
  }

  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, box) {
      final g = _box(box.biggest);
      return MouseRegion(
        cursor: _hot != null || _drag != null ? SystemMouseCursors.grab : MouseCursor.defer,
        onHover: (e) {
          final n = _near(g, e.localPosition);
          if (n != _hot) setState(() => _hot = n);
        },
        onExit: (_) {
          if (_hot != null) setState(() => _hot = null);
        },
        child: GestureDetector(
          behavior: HitTestBehavior.opaque,
          onPanDown: (d) {
            _drag = _near(g, d.localPosition);
            if (_drag != null) setState(() {});
          },
          onPanUpdate: (d) {
            if (_drag != null) _move(g, d.localPosition);
          },
          onPanEnd: (_) => setState(() => _drag = null),
          onPanCancel: () => setState(() => _drag = null),
          child: CustomPaint(
            size: box.biggest,
            painter: _GraphPainter(
              ease: widget.ease,
              from: widget.span?.from,
              to: widget.span?.to,
              frame: widget.frame,
              active: _drag ?? _hot,
              dragging: _drag != null,
            ),
          ),
        ),
      );
    },
  );
}

class _GraphPainter extends CustomPainter {
  const _GraphPainter({required this.ease, required this.from, required this.to, required this.frame, required this.active, required this.dragging});
  final Ease ease;
  final int? from, to, active;
  final int frame;
  final bool dragging;

  @override
  void paint(Canvas c, Size s) {
    final g = _EaseGraphState._box(s);
    Offset px(Offset v) => _EaseGraphState._px(g, v);
    final a = px(Offset.zero), b = px(const Offset(1, 1));
    final fill = Paint();
    // The unit band (0..100 %) is a well; overshoot above and below stays on the card.
    c.drawRect(Rect.fromLTRB(g.left, b.dy, g.right, a.dy), fill..color = Grey.g10);
    final grid = Paint()..strokeWidth = 1;
    for (var i = 0; i <= 8; i++) {
      final x = (g.left + g.width * i / 8).floorToDouble() + .5;
      c.drawLine(Offset(x, g.top), Offset(x, g.bottom), grid..color = i % 4 == 0 ? Grey.g26 : Grey.g15);
    }
    for (var i = -2; i <= 6; i++) {
      final y = px(Offset(0, i / 4)).dy.floorToDouble() + .5;
      if (y < g.top || y > g.bottom) continue;
      c.drawLine(Offset(g.left, y), Offset(g.right, y), grid..color = i == 0 || i == 4 ? Grey.g38 : Grey.g15);
    }
    final lab = T.label(Grey.g56).copyWith(fontSize: 10);
    deskText(c, '100', Offset(g.left - 5, b.dy), lab, ax: 1, ay: .5);
    deskText(c, '50', Offset(g.left - 5, px(const Offset(0, .5)).dy), lab, ax: 1, ay: .5);
    deskText(c, '0', Offset(g.left - 5, a.dy), lab, ax: 1, ay: .5);
    // Frame ticks under the box.
    if (from != null && to != null) {
      final span = to! - from!;
      final step = span <= 12 ? 2 : (span <= 30 ? 6 : (span <= 60 ? 12 : 24));
      for (var f = from!; f <= to!; f++) {
        if ((f - from!) % step != 0 && f != to) continue;
        final x = g.left + g.width * (f - from!) / span;
        c.drawLine(Offset(x, g.bottom), Offset(x, g.bottom + 3), grid..color = Grey.g38);
        if (f == from || f == to || (to! - f) >= step * .6) {
          deskText(c, '$f', Offset(x, g.bottom + 6), T.value(Grey.g63).copyWith(fontSize: 9.5), ax: f == from ? 0 : (f == to ? 1 : .5));
        }
      }
    } else {
      deskText(c, 'start', Offset(g.left, g.bottom + 6), lab);
      deskText(c, 'end', Offset(g.right, g.bottom + 6), lab, ax: 1);
    }
    final bez = ease.kind == EaseKind.bezier;
    final stroke = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1;
    // Handle stems.
    c.drawLine(a, px(ease.p1), stroke..color = bez ? Grey.g63 : Grey.g26);
    c.drawLine(b, px(ease.p2), stroke);
    // The curve.
    final path = Path()..moveTo(a.dx, a.dy);
    const n = 96;
    for (var i = 1; i <= n; i++) {
      final x = i / n;
      final o = px(Offset(x, ease.at(x)));
      if (ease.kind == EaseKind.hold && i == n) path.lineTo(o.dx, a.dy);
      path.lineTo(o.dx, o.dy);
    }
    c.drawPath(
      path,
      stroke
        ..strokeWidth = 2.2
        ..strokeCap = StrokeCap.round
        ..strokeJoin = StrokeJoin.round
        ..color = WsT.accentInk,
    );
    // The playhead, when it is inside the span.
    if (from != null && to != null && frame >= from! && frame <= to! && to! > from!) {
      final t = (frame - from!) / (to! - from!), p = px(Offset(t, ease.at(t)));
      c.drawLine(Offset(p.dx, g.top), Offset(p.dx, g.bottom), Paint()..color = Grey.g44);
      c.drawCircle(p, 3.5, fill..color = Grey.g95);
      c.drawCircle(
        p,
        3.5,
        Paint()
          ..color = Grey.g07
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1.2,
      );
    }
    // Keys at both ends.
    for (final k in [a, b]) {
      c.save();
      c.translate(k.dx, k.dy);
      c.rotate(math.pi / 4);
      c.drawRect(const Rect.fromLTWH(-4, -4, 8, 8), fill..color = WsT.keyDot);
      c.drawRect(
        const Rect.fromLTWH(-4, -4, 8, 8),
        Paint()
          ..color = Grey.g07
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1.2,
      );
      c.restore();
    }
    // Handles.
    for (var i = 0; i < 2; i++) {
      final o = px(i == 0 ? ease.p1 : ease.p2), on = active == i;
      final r = on ? 5.0 : 4.0;
      c.drawCircle(o, r, fill..color = !bez ? Grey.g38 : (on ? WsT.accent : Grey.g95));
      c.drawCircle(
        o,
        r,
        Paint()
          ..color = Grey.g07
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1.5,
      );
      if (dragging && on) {
        final v = i == 0 ? ease.p1 : ease.p2;
        deskText(
          c,
          '${v.dx.toStringAsFixed(2)}, ${v.dy.toStringAsFixed(2)}',
          o + Offset(i == 0 ? 9 : -9, -9),
          T.value(Grey.g95).copyWith(fontSize: 10),
          ax: i == 0 ? 0 : 1,
          ay: 1,
        );
      }
    }
    // Name of the shape, top-left inside the box.
    final name = easeName(ease) ?? 'Custom';
    deskText(c, name.toUpperCase(), Offset(g.left + 6, g.top + 5), T.micro(Grey.g76).copyWith(fontWeight: FontWeight.w700, letterSpacing: 1));
  }

  @override
  bool shouldRepaint(_GraphPainter o) => o.ease != ease || o.from != from || o.to != to || o.frame != frame || o.active != active || o.dragging != dragging;
}

/// The numbers: AE-style influence and speed for each end, then the four cubic-bezier numbers you can scrub.
class _EaseNumbers extends StatelessWidget {
  const _EaseNumbers({required this.ease, required this.span, required this.onChanged});
  final Ease ease;
  final EaseSpan? span;
  final ValueChanged<Ease> onChanged;
  @override
  Widget build(BuildContext context) {
    final bez = ease.kind == EaseKind.bezier, e = ease;
    String slope(Offset h, Offset k) => (h.dx - k.dx).abs() < 1e-3 ? '∞' : ((h.dy - k.dy) / (h.dx - k.dx)).abs().toStringAsFixed(2);
    Widget end(String title, double influence, String speed, {String? caption}) => Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      mainAxisSize: MainAxisSize.min,
      children: [
        DeskCaption(title, trailing: caption),
        const SizedBox(height: 4),
        Row(
          crossAxisAlignment: CrossAxisAlignment.end,
          children: [
            DeskReadout(value: bez ? (influence * 100).toStringAsFixed(0) : '—', unit: bez ? '%' : '', ink: bez ? Grey.g95 : Grey.g56),
            const Spacer(),
            Text(bez ? '$speed×' : '—', style: T.value(Grey.g76).copyWith(fontSize: 11)),
          ],
        ),
      ],
    );
    String f(double v) => v.toStringAsFixed(2);
    Widget pair(DeskScrub a, DeskScrub b) => Row(
      children: [
        Expanded(child: a),
        const SizedBox(width: 2),
        Expanded(child: b),
      ],
    );
    return ColoredBox(
      color: WsT.card,
      child: LayoutBuilder(
        builder: (context, box) => SingleChildScrollView(
          physics: const NeverScrollableScrollPhysics(),
          padding: const EdgeInsets.fromLTRB(WsT.inset, 7, WsT.inset, WsT.inset),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              end('Out', e.p1.dx, slope(e.p1, Offset.zero), caption: 'infl · speed'),
              const SizedBox(height: 10),
              end('In', 1 - e.p2.dx, slope(e.p2, const Offset(1, 1))),
              if (box.maxHeight >= 170) ...[
                const SizedBox(height: 10),
                const DeskCaption('Bezier'),
                const SizedBox(height: 4),
                pair(
                  DeskScrub(
                    label: 'x1',
                    value: e.p1.dx,
                    min: 0,
                    max: 1,
                    perPx: .01,
                    pad: 5,
                    format: f,
                    enabled: bez,
                    onChanged: (v) => onChanged(e.withHandles(Offset(v, e.p1.dy), e.p2)),
                  ),
                  DeskScrub(
                    label: 'y1',
                    value: e.p1.dy,
                    min: -.4,
                    max: 1.6,
                    perPx: .01,
                    pad: 5,
                    format: f,
                    enabled: bez,
                    onChanged: (v) => onChanged(e.withHandles(Offset(e.p1.dx, v), e.p2)),
                  ),
                ),
                const SizedBox(height: 2),
                pair(
                  DeskScrub(
                    label: 'x2',
                    value: e.p2.dx,
                    min: 0,
                    max: 1,
                    perPx: .01,
                    pad: 5,
                    format: f,
                    enabled: bez,
                    onChanged: (v) => onChanged(e.withHandles(e.p1, Offset(v, e.p2.dy))),
                  ),
                  DeskScrub(
                    label: 'y2',
                    value: e.p2.dy,
                    min: -.4,
                    max: 1.6,
                    perPx: .01,
                    pad: 5,
                    format: f,
                    enabled: bez,
                    onChanged: (v) => onChanged(e.withHandles(e.p1, Offset(e.p2.dx, v))),
                  ),
                ),
              ],
            ],
          ),
        ),
      ),
    );
  }
}

/// The presets, each a tiny curve; the one in use is lit in the accent.
class _Presets extends StatelessWidget {
  const _Presets({required this.ease, required this.onPick});
  final Ease ease;
  final ValueChanged<Ease> onPick;
  @override
  Widget build(BuildContext context) => Container(
    height: 50,
    color: WsT.card,
    padding: const EdgeInsets.all(5),
    child: Row(
      children: [
        for (final (i, p) in easePresets.indexed) ...[
          if (i > 0) const SizedBox(width: 3),
          Expanded(
            child: _PresetTile(p, on: p == ease, onTap: () => onPick(p)),
          ),
        ],
      ],
    ),
  );
}

class _PresetTile extends StatelessWidget {
  const _PresetTile(this.ease, {required this.on, required this.onTap});
  final Ease ease;
  final bool on;
  final VoidCallback onTap;
  @override
  Widget build(BuildContext context) => Hov(
    onTap: onTap,
    builder: (_, h) {
      final ink = on ? WsT.onAccent : (h ? Grey.g95 : Grey.g76);
      return Container(
        decoration: BoxDecoration(color: on ? WsT.accent : (h ? WsT.raised : WsT.well), borderRadius: BorderRadius.circular(WsT.radius)),
        padding: const EdgeInsets.fromLTRB(2, 5, 2, 4),
        child: Column(
          children: [
            Expanded(
              child: CustomPaint(size: Size.infinite, painter: _Thumb(ease, ink, on ? WsT.onAccent : Grey.g38)),
            ),
            const SizedBox(height: 3),
            Text(
              ease.name ?? '',
              maxLines: 1,
              softWrap: false,
              overflow: TextOverflow.fade,
              style: T.micro(ink).copyWith(fontSize: 9.5, fontWeight: on ? FontWeight.w700 : FontWeight.w500),
            ),
          ],
        ),
      );
    },
  );
}

class _Thumb extends CustomPainter {
  const _Thumb(this.ease, this.ink, this.base);
  final Ease ease;
  final Color ink, base;
  @override
  void paint(Canvas c, Size s) {
    const lo = -.25, hi = 1.6;
    final w = math.min(s.width, s.height * 1.6), r = Rect.fromCenter(center: s.center(Offset.zero), width: w, height: s.height);
    Offset px(double x, double y) => Offset(r.left + x * r.width, r.bottom - (y - lo) / (hi - lo) * r.height);
    c.drawLine(
      px(0, 0),
      px(1, 0),
      Paint()
        ..color = base
        ..strokeWidth = 1,
    );
    final path = Path()..moveTo(px(0, 0).dx, px(0, 0).dy);
    for (var i = 1; i <= 32; i++) {
      final x = i / 32, o = px(x, ease.at(x));
      if (ease.kind == EaseKind.hold && i == 32) path.lineTo(o.dx, px(0, 0).dy);
      path.lineTo(o.dx, o.dy);
    }
    c.drawPath(
      path,
      Paint()
        ..color = ink
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.5
        ..strokeCap = StrokeCap.round
        ..strokeJoin = StrokeJoin.round,
    );
  }

  @override
  bool shouldRepaint(_Thumb o) => o.ease != ease || o.ink != ink || o.base != base;
}
