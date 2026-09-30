// surface-file: reference-frame fixtures, every number is a coordinate of the 1536x1024 reference, not chrome spacing
// The reference frame's ruler: every seat of the 1536x1024 reference is placed in its own coordinates, by these.
// The top bar draws Motolii Live's chrome with them.
import 'package:flutter/widgets.dart';

import '../../theme/glyphs.dart';
import '../../theme/neutral.dart';

abstract class RI {
  Widget build(double ox, double oy);
}

class RF extends StatelessWidget {
  const RF(this.items, {super.key, this.ox = 0, this.oy = 0});
  final List<RI> items;
  final double ox, oy;
  @override
  Widget build(BuildContext context) => Stack(clipBehavior: Clip.hardEdge, children: [for (final i in items) i.build(ox, oy)]);
}

class Rc extends RI {
  Rc(this.x, this.y, this.w, this.h, {this.fill, this.border, this.bw = 1, this.r = 0, this.borders, this.child});
  final double x, y, w, h, bw, r;
  final Color? fill, border;
  final Border? borders;
  final Widget? child;
  @override
  Widget build(double ox, double oy) => Positioned(
        left: x - ox, top: y - oy, width: w, height: h,
        child: DecoratedBox(
          decoration: BoxDecoration(color: fill, border: borders ?? (border == null ? null : Border.all(color: border!, width: bw)), borderRadius: r > 0 ? BorderRadius.circular(r) : null),
          child: child,
        ),
      );
}

class Ln extends RI {
  Ln(this.x, this.y, this.w, this.h, this.color);
  final double x, y, w, h;
  final Color color;
  @override
  Widget build(double ox, double oy) => Positioned(left: x - ox, top: y - oy, width: w, height: h, child: ColoredBox(color: color));
}

class Pt extends RI {
  Pt(this.painter);
  final CustomPainter painter;
  @override
  Widget build(double ox, double oy) => Positioned(left: -ox, top: -oy, width: 1536, height: 1024, child: CustomPaint(painter: painter));
}

/// Any widget at a reference rectangle.
class Wd extends RI {
  Wd(this.x, this.y, this.w, this.h, this.child);
  final double x, y, w, h;
  final Widget child;
  @override
  Widget build(double ox, double oy) => Positioned(left: x - ox, top: y - oy, width: w, height: h, child: child);
}

enum Al { left, center, right }

// Text placed by alphabetic baseline; width-fitted by letter-spacing when [w] is given.
class Tx extends RI {
  Tx(this.x, this.base, this.text, this.style, {this.w, this.al = Al.left, this.fit = true});
  final double x, base;
  final String text;
  final TextStyle style;
  final double? w;
  final Al al;

  /// Fit the reference's own words to [w] (shrink, then tune tracking). A name that comes from data (a layer's)
  /// is not fitted: it keeps its natural spacing and ends in an ellipsis at [w].
  final bool fit;
  @override
  Widget build(double ox, double oy) {
    if (!fit && w != null) {
      return Positioned(
        left: x - 1 - ox, top: 0, width: w! + 1.5,
        child: Baseline(baseline: base - oy, baselineType: TextBaseline.alphabetic, child: Text(text, softWrap: false, maxLines: 1, overflow: TextOverflow.ellipsis, style: style)),
      );
    }
    var st = style;
    var tp = TextPainter(text: TextSpan(text: text, style: st), textDirection: TextDirection.ltr)..layout();
    if (w != null && text.length > 1) {
      // Inter is wider than the reference face: shrink first (down to 82%), then tune tracking.
      if (tp.width > (w! + 1.5) * 1.03) {
        final f = ((w! + 1.5) / tp.width).clamp(0.82, 1.0);
        st = st.copyWith(fontSize: st.fontSize! * f, letterSpacing: (st.letterSpacing ?? 0) * f);
        tp = TextPainter(text: TextSpan(text: text, style: st), textDirection: TextDirection.ltr)..layout();
      }
      final ls = ((st.letterSpacing ?? 0) + (w! + 1.5 - tp.width) / text.length).clamp(-0.7, 2.2);
      st = st.copyWith(letterSpacing: ls);
      tp = TextPainter(text: TextSpan(text: text, style: st), textDirection: TextDirection.ltr)..layout();
    }
    final left = switch (al) { Al.left => x - 1, Al.center => x - tp.width / 2, Al.right => x - tp.width };
    return Positioned(
      left: left - ox, top: 0,
      child: Baseline(baseline: base - oy, baselineType: TextBaseline.alphabetic, child: Text(text, softWrap: false, style: st)),
    );
  }
}

/// A glyph centred on a reference point.
class Hg extends RI {
  Hg(this.cx, this.cy, this.size, this.g, this.color, {this.bg = N.g13, this.a = 1});
  final double cx, cy, size, a;
  final HG g;
  final Color color, bg;
  @override
  Widget build(double ox, double oy) => Positioned(
        left: cx - size / 2 - ox, top: cy - size / 2 - oy, width: size, height: size,
        child: CustomPaint(painter: HgPainter(g, color, bg, a)),
      );
}
