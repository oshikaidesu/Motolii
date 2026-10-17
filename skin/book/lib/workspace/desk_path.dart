// The desk's Path tool: the picked thing's way through its time, as the stage places it (motolii_path: its keys, wave and
// all), over the picture's frame; its keys as dots, where it is now as a ring. And whether it turns the way it is going
// (the inside's orient: After Effects' Auto-Orient).
import 'dart:ui' as ui;

import 'package:flutter/widgets.dart';

import '../tokens.dart';
import 'desk_kit.dart';
import 'stage_art.dart' show WsArt;
import 'ws.dart';

class DeskPath extends StatelessWidget {
  const DeskPath({super.key});
  static (int, int, List<Offset>, List<Offset>)? _kept;

  @override
  Widget build(BuildContext context) {
    final ws = WsScope.of(context), m = ws.inside, t = int.tryParse(ws.selected ?? '');
    final live = m != null && t != null && m.word(t, 'kind') != 'camera' && m.word(t, 'kind') != 'light';
    var way = const <Offset>[], keys = const <Offset>[];
    Offset? now;
    var orient = false, canTurn = false;
    if (live) {
      final from = m.value(t, 'start', 0) ?? 0, lasts = m.lasts(t), to = lasts > 0 ? from + lasts : m.seconds;
      Offset seen(double x, double y) => WsArt.centre + Offset(x * m.point, -y * m.point);   // (units of the stage to points of the picture)
      List<Offset> at(double a, double b, int n) {
        final p = m.path(t, from: a, to: b, n: n) ?? const <double>[];
        return [for (var i = 0; i + 3 < p.length; i += 4) seen(p[i], p[i + 1])];
      }

      if (_kept?.$1 != t || _kept?.$2 != ws.rev) {   // (asked again only when the document has changed: not each frame it plays)
        _kept = (t, ws.rev, [...at(from, to, ((to - from) * Ws.fps).round().clamp(60, 7200)), ...at(to, to + 1e-4, 1)],   // (a point a frame, to its very end)
             [for (final k in {...m.keyMoments(t, 'x'), ...m.keyMoments(t, 'y')}) ...at(k, k + 1e-4, 1)]);
      }
      (_, _, way, keys) = _kept!;
      final s = ws.frame / Ws.fps;
      now = s < from || s > to ? null : at(s, s + 1e-4, 1).firstOrNull;   // (outside its time: nowhere)
      orient = (m.value(t, 'orient', 0) ?? 0) > .5;
      canTurn = m.range(t, 'orient') != null;
    }
    return ListView(
      padding: const EdgeInsets.all(WsT.gap),
      children: [
        DeskStrip(
          left: [if (ws.layer case final l?) DeskLayerChip(l.name, wsTone(l.kind))],
          right: [
            if (canTurn)
            DeskIconButton(
              key: const ValueKey('path.orient'),
              size: 28,
              on: orient,
              glyph: (ink) => _Orient(ink),
              onTap: () {
                if (live && m.set(t, 'orient', orient ? 0 : 1)) ws.reread();
              },
            ),
          ],
        ),
        const SizedBox(height: 8),
        AspectRatio(aspectRatio: WsArt.comp.aspectRatio, child: CustomPaint(painter: _Way(way, keys, now))),
      ],
    );
  }
}

/// The picture's frame, the way through it, its keys and where it is now.
class _Way extends CustomPainter {
  _Way(this.way, this.keys, this.now);
  final List<Offset> way, keys;
  final Offset? now;
  @override
  void paint(Canvas c, Size s) {
    // (the frame and the whole way both in view, with room round them: a way may leave the picture)
    var all = Offset.zero & WsArt.comp;
    for (final p in way) {
      all = all.expandToInclude(Rect.fromCenter(center: p, width: 0, height: 0));
    }
    all = all.inflate(all.longestSide * .06);
    final k = (s.width / all.width) < (s.height / all.height) ? s.width / all.width : s.height / all.height;
    final o = Offset(s.width - all.width * k, s.height - all.height * k) / 2 - all.topLeft * k;
    Offset to(Offset p) => o + p * k;
    c.clipRRect(RRect.fromRectAndRadius(Offset.zero & s, const Radius.circular(5)));
    c.drawRRect(RRect.fromRectAndRadius(Offset.zero & s, const Radius.circular(5)), Paint()..color = WsT.well);
    c.drawRect(to(Offset.zero) & WsArt.comp * k, Paint()
      ..color = Grey.g38
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1);
    if (way.length > 1) {
      c.drawPoints(ui.PointMode.polygon, [for (final p in way) to(p)], Paint()
        ..color = WsT.accent.withValues(alpha: .8)
        ..strokeWidth = 1.4);
    }
    for (final p in keys) {
      c.drawRect(Rect.fromCenter(center: to(p), width: 5, height: 5), Paint()..color = Grey.g91);
    }
    if (now case final p?) {
      c.drawCircle(to(p), 4.5, Paint()
        ..color = WsT.accent
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.6);
    }
  }

  @override
  bool shouldRepaint(_Way o) => true;
}

/// Turned the way it goes: an arrow riding a curve.
class _Orient extends CustomPainter {
  _Orient(this.ink);
  final Color ink;
  @override
  void paint(Canvas c, Size s) {
    final line = Paint()
      ..color = ink
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.4
      ..strokeCap = StrokeCap.round;
    c.drawPath(Path()
      ..moveTo(s.width * .18, s.height * .72)
      ..quadraticBezierTo(s.width * .45, s.height * .2, s.width * .82, s.height * .42), line);
    c.drawPath(Path()
      ..moveTo(s.width * .62, s.height * .28)
      ..lineTo(s.width * .82, s.height * .42)
      ..lineTo(s.width * .6, s.height * .52), line);
  }

  @override
  bool shouldRepaint(_Orient o) => o.ink != ink;
}
