import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../../foundation/theme.dart';
import 'layout.dart';

/// Timeline の絵の道具 — 使い回す塗り・線・縁取りと、組んだ文字の版。
///
/// paint() の中で Paint / Path を作らない(custom-canvas-and-gestures 規則 5)。
/// 塗りは 1 枚、線は 1 枚、縁取りは 1 枚を使い回し、呼ぶ度に安い属性だけ書き換える。
/// Canvas の draw* は呼んだ時の値を読むので、同じ物を続けて使っても絵は変わらない。
/// 両方の painter(TimelinePainter と const の ArrangementOverview)が読む —
/// paint() は UI thread の上で 1 本ずつしか走らない。

final Paint _fillPaint = Paint()..style = PaintingStyle.fill;
final Paint _linePaint = Paint()..style = PaintingStyle.fill;
final Paint _strokePaint = Paint()..style = PaintingStyle.stroke;
final Path scratchPath = Path();

Paint fillPaint(Color color) => _fillPaint..color = color;
Paint linePaint(Color color, [double width = 0]) => _linePaint
  ..color = color
  ..strokeWidth = width;
Paint strokePaint(Color color, [double width = 0]) => _strokePaint
  ..color = color
  ..strokeWidth = width;

/// Laid-out labels kept across paints: a row's name is the same text at the
/// same width on every frame of playback.
final _labels = <(String, Color, double, double, FontWeight), TextPainter>{};

/// 組み上げた文字の版を跨いで持つ: 行の名前は再生のどのコマでも同じ幅の同じ文字。
/// [rowAligned] の時は行の高さの真ん中へ寄せる(定規はそのまま置く)。
void paintText(
  Canvas canvas,
  String text,
  Offset at, {
  required Color color,
  double width = 200,
  double size = 11,
  bool centered = false,
  FontWeight weight = FontWeight.w500,
  bool rowAligned = true,
}) {
  if (_labels.length >= 512) _labels.clear();
  final p = _labels[(text, color, width, size, weight)] ??= TextPainter(
    text: TextSpan(
      text: text,
      style: TextStyle(
        color: color,
        fontSize: size,
        fontFamily: EditorTheme.fontFamily,
        fontWeight: weight,
      ),
    ),
    textDirection: TextDirection.ltr,
    maxLines: 1,
    ellipsis: '…',
  )..layout(maxWidth: math.max(0, width));
  final aligned = rowAligned
      ? Offset(
          at.dx,
          (at.dy / timelineRowHeight).floor() * timelineRowHeight +
              (timelineRowHeight - p.height) / 2,
        )
      : at;
  p.paint(
    canvas,
    centered ? Offset(at.dx + (width - p.width) / 2, aligned.dy) : aligned,
  );
}
