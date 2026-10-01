// Handoff v1 harness: everything is placed in 1536x1024 reference coordinates.
// The ruler itself lives in hf/shell/place.dart, shared with the production faces.
import 'package:flutter/widgets.dart';
import '../proto/icons.dart';
import '../hf/shell/place.dart' show RI;

export '../hf/shell/place.dart' show H, Fam, RI, RF, Rc, Ln, Pt, Wd, Al, Tx;

class Gl extends RI {
  Gl(this.cx, this.cy, this.size, this.g, this.color, {this.stroke = 1.5});
  final double cx, cy, size, stroke;
  final Glyph g;
  final Color color;
  @override
  Widget build(double ox, double oy) => Positioned(left: cx - size / 2 - ox, top: cy - size / 2 - oy, width: size, height: size, child: Icon1(g, size: size, color: color, stroke: stroke));
}
