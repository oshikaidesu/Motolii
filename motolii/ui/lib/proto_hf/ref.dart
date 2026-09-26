// Handoff v1 harness: everything is placed in 1536x1024 reference coordinates.
// Throwaway prototype. Not wired to anything.
import 'package:flutter/widgets.dart';
import '../proto/icons.dart';

abstract final class H {
  // ---- Neutral palette: normalised from the reference (see handoff section 15).
  static const window = Color(0xFF191919); // ground: window, panels, gutters
  static const raised = Color(0xFF202020); // tiles, keys, inputs, rows, cards, row ground A
  static const raisedHi = Color(0xFF262626); // hover, value wells, row ground B
  static const sel = Color(0xFF2F3034); // selected tab / preset / open tab
  static const selHi = Color(0xFF3B3D42); // selected inside a bar (subtab, tool)
  static const rule = Color(0xFF343434); // outlines: panel edges, control borders
  static const rule2 = Color(0xFF2A2A2A); // inner dividers
  static const track = Color(0xFF2C2C2C); // slider track
  static const text = Color(0xFFF5F5F5); // primary: names, active labels
  static const text2 = Color(0xFFD2D2D2); // secondary: items, inactive tabs
  static const text3 = Color(0xFF9E9E9F); // tertiary: annotation, sub-labels
  static const ink = Color(0xFF14171A); // glyph and label on identity surfaces
  static const gutter = window;
  static Color selAt() => sel;

  // ---- Semantic families: one hue, area-dependent variants.
  static const scatter = Fam(Color(0xFFF27AB6), t: Color(0xFFE274AA), n: Color(0xFFC76295));
  static const stagger = Fam(Color(0xFF5596E9), t: Color(0xFF4C7AAC), n: Color(0xFF477FBD));
  static const along = Fam(Color(0xFF7BCBA3), t: Color(0xFF77B68C), n: Color(0xFF517F68));
  static const face = Fam(Color(0xFFF0D455), t: Color(0xFFCEBA54), n: Color(0xFFE3C748));
  static const follow = Fam(Color(0xFFF69260));
  static const attach = Fam(Color(0xFFA282E8), t: Color(0xFFA889E9), n: Color(0xFFA086E2));
  // Transform is a sibling operation with a neutral, subdued identity (not a property, not pink).
  static const neutralT = Color(0xFF8C7A88), neutralN = Color(0xFF77717C);
  // Operational
  static const play = Color(0xFF7BCC9E), record = Color(0xFFF03C8A), mode = Color(0xFF7A87E3), toggleOn = Color(0xFF6982D1), toggleOff = Color(0xFF999BA0), playhead = Color(0xFF6EA6DB);

  static const sans = 'Inter';
  static const mono = 'Menlo';
  static TextStyle s(double size, {Color color = text, FontWeight w = FontWeight.w400, double ls = 0}) =>
      TextStyle(fontFamily: sans, fontSize: size, color: color, fontWeight: w, letterSpacing: ls, height: 1);
  static TextStyle m(double size, {Color color = text, double ls = 0}) =>
      TextStyle(fontFamily: mono, fontSize: size, color: color, letterSpacing: ls, height: 1);
}

class Fam {
  const Fam(this.b, {Color? t, Color? n}) : _t = t, _n = n;
  final Color b; // Browser: large surface
  final Color? _t, _n;
  Color get i => _v(b, 1.16, .96); // Inspector: small accents may be stronger
  Color get t => _t ?? b; // Timeline body: sampled from the reference, not derived
  Color get n => _n ?? b; // Timeline name icon: sampled
  Color get dim => Color.lerp(H.raised, b, .30)!; // off / dim
  static Color _v(Color c, double sat, double lit) {
    final h = HSLColor.fromColor(c);
    return h.withSaturation((h.saturation * sat).clamp(0, 1)).withLightness((h.lightness * lit).clamp(0, 1)).toColor();
  }
}

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

class Gl extends RI {
  Gl(this.cx, this.cy, this.size, this.g, this.color, {this.stroke = 1.5});
  final double cx, cy, size, stroke;
  final Glyph g;
  final Color color;
  @override
  Widget build(double ox, double oy) => Positioned(left: cx - size / 2 - ox, top: cy - size / 2 - oy, width: size, height: size, child: Icon1(g, size: size, color: color, stroke: stroke));
}

class Pt extends RI {
  Pt(this.painter);
  final CustomPainter painter;
  @override
  Widget build(double ox, double oy) => Positioned(left: -ox, top: -oy, width: 1536, height: 1024, child: CustomPaint(painter: painter));
}

enum Al { left, center, right }

// Text placed by alphabetic baseline; width-fitted by letter-spacing when [w] is given.
class Tx extends RI {
  Tx(this.x, this.base, this.text, this.style, {this.w, this.al = Al.left});
  final double x, base;
  final String text;
  final TextStyle style;
  final double? w;
  final Al al;
  @override
  Widget build(double ox, double oy) {
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
