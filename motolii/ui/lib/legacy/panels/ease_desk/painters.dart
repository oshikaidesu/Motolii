part of '../ease_desk.dart';

class EaseCurvePainter extends CustomPainter {
  final EditorTheme colors;

  const EaseCurvePainter({
    this.colors = EditorTheme.chromatic,
    required this.shape,
    this.handles = false,
    this.selected = false,
    this.free = false,
    this.playhead,
    this.ghost = false,
    this.marks = const [],
  });
  final Map<String, dynamic> shape;
  final bool handles, free, selected;
  final double? playhead;

  /// ゴーストモード: 差し色を反転し、鎖の各ゴーストの位置を曲線上に打つ。
  final bool ghost;
  final List<double> marks;
  bool get expanded => free || shape['overshoots'] == true;
  double get lo =>
      handles ? (expanded ? -.5 : 0) : (shape['overshoots'] == true ? -.5 : 0);
  double get hi =>
      handles ? (expanded ? 2.2 : 1) : (shape['overshoots'] == true ? 2.2 : 1);
  Rect plot(Size size) => (Offset.zero & size).deflate(handles ? 18 : 4);
  Offset toPixel(Offset p, Size size) {
    final r = plot(size);
    return Offset(
      r.left + p.dx * r.width,
      r.top + (hi - p.dy) / (hi - lo) * r.height,
    );
  }

  Offset toCurve(Offset p, Size size) {
    final r = plot(size);
    return Offset(
      ((p.dx - r.left) / r.width).clamp(-.2, 1.2),
      hi - (p.dy - r.top) / r.height * (hi - lo),
    );
  }

  @override
  void paint(Canvas canvas, Size size) {
    final grid = Paint()
      ..color = EditorInk.dark.easeInk.withValues(alpha: .15)
      ..strokeWidth = .5;
    Offset p(double x, double y) => toPixel(Offset(x, y), size);
    if (handles)
      for (var i = 0; i <= 4; i++) {
        canvas.drawLine(p(i / 4, 0), p(i / 4, 1), grid);
        canvas.drawLine(p(0, i / 4), p(1, i / 4), grid);
      }
    final samples = (shape['samples'] as List? ?? [])
        .whereType<List>()
        .toList();
    final path = Path();
    if (shape['kind'] == 'Hold') {
      final a = p(0, 0), b = p(1, 0), c = p(1, 1);
      path.moveTo(a.dx, a.dy);
      path.lineTo(b.dx, b.dy);
      path.lineTo(c.dx, c.dy);
    } else if (samples.isNotEmpty) {
      for (var i = 0; i < samples.length; i++) {
        final q = p(
          (samples[i][0] as num).toDouble(),
          (samples[i][1] as num).toDouble(),
        );
        if (i == 0) {
          path.moveTo(q.dx, q.dy);
        } else {
          path.lineTo(q.dx, q.dy);
        }
      }
    } else {
      final a = p(0, 0), b = p(1, 1);
      path.moveTo(a.dx, a.dy);
      path.lineTo(b.dx, b.dy);
    }
    canvas.save();
    canvas.clipRect(Offset.zero & size);
    canvas.drawPath(
      path,
      Paint()
        ..style = PaintingStyle.stroke
        ..color = handles || selected ? EditorInk.dark.easeInk : colors.ink
        ..strokeWidth = handles ? 2.5 : 1.5
        ..strokeCap = StrokeCap.round
        ..strokeJoin = StrokeJoin.round,
    );
    if (ghost) {
      for (final x in marks) {
        canvas.drawCircle(
          p(x, easeValueAt(shape, x)),
          3.5,
          Paint()..color = colors.accent,
        );
      }
    }
    if (handles && playhead != null) {
      final inside = playhead! >= 0 && playhead! <= 1;
      final x = playhead!.clamp(0.0, 1.0);
      canvas.drawLine(
        p(x, lo),
        p(x, hi),
        Paint()
          ..color = inside
              ? EditorInk.dark.easeTime
              : EditorInk.dark.easeTime.withValues(alpha: .4)
          ..strokeWidth = EditorMetrics.s2,
      );
      if (inside) {
        final current = p(x, easeValueAt(shape, x));
        canvas.drawCircle(
          current,
          EditorMetrics.s6,
          Paint()..color = EditorInk.dark.easePaper,
        );
        canvas.drawCircle(
          current,
          EditorMetrics.s4,
          Paint()..color = EditorInk.dark.easeTime,
        );
      } else {
        final tip = p(x, (lo + hi) / 2);
        final direction = playhead! < 0 ? -1.0 : 1.0;
        final arrow = Path()
          ..moveTo(tip.dx + direction * EditorMetrics.s8, tip.dy)
          ..lineTo(tip.dx, tip.dy - EditorMetrics.s4)
          ..lineTo(tip.dx, tip.dy + EditorMetrics.s4)
          ..close();
        canvas.drawPath(arrow, Paint()..color = EditorInk.dark.easeTime);
      }
    }
    if (handles) {
      final points = (shape['handles'] as List? ?? [])
          .whereType<List>()
          .map((v) => p((v[0] as num).toDouble(), (v[1] as num).toDouble()))
          .toList();
      if (shape['kind'] == 'Bezier' && points.length == 2) {
        canvas.drawLine(p(0, 0), points[0], grid);
        canvas.drawLine(p(1, 1), points[1], grid);
      }
      for (final q in points) {
        canvas.drawCircle(
          q,
          8,
          Paint()
            ..color = handles || selected
                ? EditorInk.dark.easeInk
                : colors.ink.withValues(alpha: .25),
        );
        canvas.drawCircle(q, 5, Paint()..color = EditorInk.dark.easeInk);
        canvas.drawCircle(q, 2, Paint()..color = EditorInk.dark.easePaper);
      }
    }
    canvas.restore();
    if (handles) {
      void label(String value, Offset position) {
        final text = TextPainter(
          text: TextSpan(
            text: value,
            style: TextStyle(
              fontFamily: EditorTheme.fontFamily,
              fontSize: EditorMetrics.micro,
              color: EditorInk.dark.easeInk,
            ),
          ),
          textDirection: TextDirection.ltr,
        )..layout();
        text.paint(canvas, position);
      }

      label('Value', const Offset(EditorMetrics.s4, EditorMetrics.s2));
      label(
        'Time',
        Offset(
          size.width / 2 - EditorMetrics.s12,
          size.height - EditorMetrics.s12,
        ),
      );
    }
  }

  @override
  bool shouldRepaint(covariant EaseCurvePainter old) =>
      colors != old.colors ||
      !sameValue(old.shape, shape) ||
      old.free != free ||
      old.handles != handles ||
      old.selected != selected ||
      old.playhead != playhead ||
      old.ghost != ghost ||
      !sameValue(old.marks, marks);
}

class EaseMotionPainter extends CustomPainter {
  final EditorTheme colors;

  const EaseMotionPainter({
    this.colors = EditorTheme.chromatic,
    required this.shape,
    required this.time,
    this.free = false,
  });
  final Map<String, dynamic> shape;
  final double time;
  final bool free;

  @override
  void paint(Canvas canvas, Size size) {
    final range = EaseCurvePainter(
      colors: colors,
      shape: shape,
      handles: true,
      free: free,
    );
    final half = EditorMetrics.s6;
    double x(double value) =>
        half +
        (value - range.lo) /
            (range.hi - range.lo) *
            math.max(0, size.width - half * 2);
    final y = size.height / 2;
    canvas.drawLine(
      Offset(x(0), y),
      Offset(x(1), y),
      Paint()..color = colors.border,
    );
    canvas.drawLine(
      Offset(x(1), y - EditorMetrics.s4),
      Offset(x(1), y + EditorMetrics.s4),
      Paint()..color = colors.muted,
    );
    canvas.drawCircle(
      Offset(x(easeValueAt(shape, time)), y),
      half,
      Paint()..color = EditorInk.dark.easePaper,
    );
  }

  @override
  bool shouldRepaint(covariant EaseMotionPainter old) =>
      colors != old.colors ||
      time != old.time ||
      free != old.free ||
      !sameValue(shape, old.shape);
}

/// どの区間を走っているか、を形で言う帯。選んだキー区間を時間軸のまま並べ、
/// 今のカーブが効いている区間だけを塗り、再生位置をその上に刺す。
/// グラフの縦線が「区間の中のどこ」なら、この帯は「どの区間」。
class EaseIntervalPainter extends CustomPainter {
  final EditorTheme colors;

  const EaseIntervalPainter({
    this.colors = EditorTheme.chromatic,
    required this.segments,
    required this.active,
    required this.frame,
  });
  final List<Map<String, dynamic>> segments;
  final int active;
  final int frame;

  double _start(Map<String, dynamic> s) => (s['frame'] as num).toDouble();
  double _end(Map<String, dynamic> s) => (s['end'] as num).toDouble();

  @override
  void paint(Canvas canvas, Size size) {
    final y = size.height / 2;
    final inset = EditorMetrics.s4;
    final width = math.max(1.0, size.width - inset * 2);
    final rail = Paint()
      ..color = colors.border
      ..strokeWidth = 1;
    if (segments.isEmpty) {
      canvas.drawLine(Offset(inset, y), Offset(inset + width, y), rail);
      return;
    }
    var lo = segments.map(_start).reduce(math.min);
    var hi = segments.map(_end).reduce(math.max);
    lo = math.min(lo, frame.toDouble());
    hi = math.max(hi, frame.toDouble());
    final pad = math.max(1.0, (hi - lo) * .04);
    lo -= pad;
    hi += pad;
    double x(double f) => inset + (f - lo) / (hi - lo) * width;
    canvas.drawLine(Offset(inset, y), Offset(inset + width, y), rail);
    final bar = EditorMetrics.s6;
    for (var i = 0; i < segments.length; i++) {
      final chosen = i == active;
      final rect = RRect.fromLTRBR(
        x(_start(segments[i])),
        y - bar / 2,
        math.max(x(_start(segments[i])) + 2, x(_end(segments[i]))),
        y + bar / 2,
        const Radius.circular(EditorMetrics.s2),
      );
      canvas.drawRRect(
        rect,
        Paint()
          ..color = chosen
              ? EditorInk.dark.easePaper
              : colors.muted.withValues(alpha: .35),
      );
      if (chosen) {
        canvas.drawRRect(
          rect,
          Paint()
            ..style = PaintingStyle.stroke
            ..strokeWidth = 1
            ..color = EditorInk.dark.easeInk,
        );
      }
      // 区間の両端 = キーフレーム。選ばれた区間だけ濃く。
      for (final f in [_start(segments[i]), _end(segments[i])]) {
        canvas.drawCircle(
          Offset(x(f), y),
          chosen ? 2.5 : 1.5,
          Paint()..color = chosen ? EditorInk.dark.easeInk : colors.border,
        );
      }
    }
    final head = x(frame.toDouble());
    canvas.drawLine(
      Offset(head, 0),
      Offset(head, size.height),
      Paint()
        ..color = EditorInk.dark.easeTime
        ..strokeWidth = EditorMetrics.s2,
    );
    final caret = Path()
      ..moveTo(head - EditorMetrics.s3, 0)
      ..lineTo(head + EditorMetrics.s3, 0)
      ..lineTo(head, EditorMetrics.s4)
      ..close();
    canvas.drawPath(caret, Paint()..color = EditorInk.dark.easeTime);
  }

  @override
  bool shouldRepaint(covariant EaseIntervalPainter old) =>
      colors != old.colors ||
      old.frame != frame ||
      old.active != active ||
      !identical(old.segments, segments) &&
          !sameValue(
            [
              for (final s in old.segments) [s['frame'], s['end']],
            ],
            [
              for (final s in segments) [s['frame'], s['end']],
            ],
          );
}
