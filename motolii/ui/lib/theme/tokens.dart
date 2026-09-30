// The reference frame's ruler: every seat of the 1536x1024 reference is placed in its own coordinates, by these.
// proto_hf draws fixtures with them; the production faces draw Motolii Live with the same ones.
import 'package:flutter/widgets.dart';
import 'glyphs.dart';
import 'neutral.dart';

abstract final class H {
  // ---- Neutral palette: normalised from the reference (see handoff section 15).
  static const window = N.g10; // ground: window, panels, gutters
  static const raised = N.g13; // tiles, keys, inputs, rows, cards, row ground A
  static const raisedHi = N.g15; // hover, value wells, row ground B
  static const sel = N.g20; // selected tab / preset / open tab
  static const selHi = N.g26; // selected inside a bar (subtab, tool)
  static const rule = N.g20; // outlines: panel edges, control borders
  static const rule2 = N.g15; // inner dividers
  static const track = N.g15; // slider track
  static const text = N.g95; // primary: names, active labels
  static const text2 = N.g82; // secondary: items, inactive tabs
  static const text3 = N.g63; // tertiary: annotation, sub-labels
  static const ink = N.g10; // glyph and label on identity surfaces
  static const gutter = window;
  static Color selAt() => sel;

  // ---- Semantic families: one hue, area-dependent variants.
  static const scatter = Fam(Color(0xFFF27AB6), t: Color(0xFFE974AB), n: Color(0xFFDD6F9F));
  static const stagger = Fam(Color(0xFF5596E9), t: Color(0xFF4781E5), n: Color(0xFF4880E5));
  static const along = Fam(Color(0xFF7BCBA3), t: Color(0xFF7DD5B1), n: Color(0xFF73CEAB));
  static const face = Fam(Color(0xFFF0D455), t: Color(0xFFEFCB4E), n: Color(0xFFE3C748));
  static const follow = Fam(Color(0xFFF69260));
  static const attach = Fam(Color(0xFFA282E8), t: Color(0xFFA889E9), n: Color(0xFFA086E2));
  // Transform is a sibling operation with a neutral, subdued identity (not a property, not pink).
  static const neutralT = Color(0xFF8C7A88), neutralN = Color(0xFF77717C);
  // Operational
  static const relation = Color(0xFFFF4D3D), wave = Color(0xFF3B6D5F), textSelection = Color(0x552F6BFF), marquee = Color(0xFFFFBC53), guide = Color(0xFFB0E3EF);
  static const play = Color(0xFF7BCC9E), record = Color(0xFFF03C8A), mode = Color(0xFF7A87E3), toggleOn = Color(0xFF6982D1), toggleOff = N.g63, playhead = Color(0xFF6EA6DB);

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
