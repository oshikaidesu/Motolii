// Colour that means something: the semantic families (a property's hue), the operational colours (play, record, guide,
// the relation red) and the two text builders. The greys are N (neutral.dart); sizes and surface levels are Surface (metrics.dart).
import 'package:flutter/widgets.dart';
import 'neutral.dart';

abstract final class H {
  // ---- Semantic families: one hue, area-dependent variants.
  static const scatter = Fam(Color(0xFFF27AB6), t: Color(0xFFE974AB), n: Color(0xFFDD6F9F));
  static const stagger = Fam(Color(0xFF5596E9), t: Color(0xFF4781E5), n: Color(0xFF4880E5));
  static const along = Fam(Color(0xFF7BCBA3), t: Color(0xFF7DD5B1), n: Color(0xFF73CEAB));
  static const face = Fam(Color(0xFFF0D455), t: Color(0xFFEFCB4E), n: Color(0xFFE3C748));
  static const follow = Fam(Color(0xFFF69260));
  static const attach = Fam(Color(0xFFA282E8), t: Color(0xFFA889E9), n: Color(0xFFA086E2));
  // Operational
  static const relation = Color(0xFFFF4D3D), warn = Color(0xFFF08A3C), wave = Color(0xFF3B6D5F), textSelection = Color(0x552F6BFF), guide = Color(0xFFB0E3EF);
  static const play = Color(0xFF7BCC9E), record = Color(0xFFF03C8A), mode = Color(0xFF7A87E3), toggleOn = Color(0xFF6982D1), playhead = Color(0xFF6EA6DB);

  static const sans = 'Inter';
  static const mono = 'Menlo';
  static TextStyle s(double size, {Color color = N.g95, FontWeight w = FontWeight.w400, double ls = 0}) =>
      TextStyle(fontFamily: sans, fontSize: size, color: color, fontWeight: w, letterSpacing: ls, height: 1);
  static TextStyle m(double size, {Color color = N.g95, double ls = 0}) =>
      TextStyle(fontFamily: mono, fontSize: size, color: color, letterSpacing: ls, height: 1);
}

class Fam {
  const Fam(this.b, {Color? t, Color? n}) : _t = t, _n = n;
  final Color b; // Browser: large surface
  final Color? _t, _n;
  Color get i => _v(b, 1.16, .96); // Inspector: small accents may be stronger
  Color get t => _t ?? b; // Timeline body: sampled from the reference, not derived
  Color get n => _n ?? b; // Timeline name icon: sampled
  Color get dim => Color.lerp(N.g13, b, .30)!; // off / dim
  static Color _v(Color c, double sat, double lit) {
    final h = HSLColor.fromColor(c);
    return h.withSaturation((h.saturation * sat).clamp(0, 1)).withLightness((h.lightness * lit).clamp(0, 1)).toColor();
  }
}
