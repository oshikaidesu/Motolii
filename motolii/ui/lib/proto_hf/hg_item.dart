// A glyph placed in the reference-coordinate harness (the painted 1536x1024 window). The glyph itself is hf/glyphs.dart.
import 'package:flutter/widgets.dart';
import '../hf/glyphs.dart';
import 'ref.dart';

class Hg extends RI {
  Hg(this.cx, this.cy, this.size, this.g, this.color, {this.bg = const Color(0xFF202020), this.a = 1});
  final double cx, cy, size, a;
  final HG g;
  final Color color, bg;
  @override
  Widget build(double ox, double oy) => Positioned(
        left: cx - size / 2 - ox, top: cy - size / 2 - oy, width: size, height: size,
        child: CustomPaint(painter: HgPainter(g, color, bg, a)),
      );
}
