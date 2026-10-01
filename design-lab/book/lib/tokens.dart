// Every value here is read from ../DESIGN.md (the tag in the comment says where it came from). A part names a token; it never picks a number.
import 'package:flutter/widgets.dart';

abstract final class N {
  static const g00 = Color(0xFF000000), g07 = Color(0xFF131313), g10 = Color(0xFF191919), g13 = Color(0xFF202020), g15 = Color(0xFF262626);
  static const g20 = Color(0xFF343434), g26 = Color(0xFF424242), g38 = Color(0xFF616161), g44 = Color(0xFF717171), g56 = Color(0xFF8E8E8E);
  static const g63 = Color(0xFFA1A1A1), g76 = Color(0xFFC1C1C1), g91 = Color(0xFFE7E7E7), g95 = Color(0xFFF2F2F2), g100 = Color(0xFFFFFFFF);
  // [decided] the timeline's two row bands and its row line
  static const bandA = Color(0xFF1E1E1E), bandB = Color(0xFF2A2A2A), rowLine = Color(0xFF161616);
  static const glaze9 = Color(0x17FFFFFF), glaze15 = Color(0x29FFFFFF);
}

/// A relation family: the colour a relation carries everywhere. [code] identity.dart
class Fam {
  const Fam(this.name, this.jp, this.c);
  final String name, jp;
  final Color c;
  static const scatter = Fam('Scatter', '散らす', Color(0xFFE974AB));
  static const along = Fam('Along Path', 'パスに沿って', Color(0xFF7DD5B1));
  static const stagger = Fam('Stagger', 'ずらす', Color(0xFF4781E5));
  static const face = Fam('Face', '向かせる', Color(0xFFEFCB4E));
  static const follow = Fam('Follow', '追従', Color(0xFFF69260));
  static const attach = Fam('Attach', 'くっつける', Color(0xFFA889E9));
  static const all = [scatter, along, stagger, face, follow, attach];
}

abstract final class C {
  /// [open] Error and destructive actions. The record pink (#F03C8A) is too close to the Scatter family (#E974AB); one warm red, used only for danger.
  static const danger = Color(0xFFE5584B);
  static const playhead = Color(0xFF6EA6DB), play = Color(0xFF7BCC9E), record = Color(0xFFF03C8A), mode = Color(0xFF7A87E3);
}

abstract final class T {
  static const sans = 'Inter', mono = 'Menlo';

  /// No text is dimmer than g56 nor smaller than 10 px (precedents: Adobe Spectrum's smallest UI text is 10 px; critique 2026-10-02: 87 of 126 use cases had 9 px / g44 captions).
  /// Only DIM text on a dark ground is lifted (luminance between g26 and g56). Dark INK (g00..g20, the text on a light fill such as the playhead head) is left alone: lifting it to g56 made it unreadable (critique pass 3).
  static Color _floor(Color c) => c.a < .5 || c.computeLuminance() >= N.g56.computeLuminance() || c.computeLuminance() < .04 ? c : N.g56;
  static TextStyle name([Color c = N.g91]) => TextStyle(fontFamily: sans, fontFamilyFallback: const ['.AppleSystemUIFont'], fontSize: 11, fontWeight: FontWeight.w500, color: _floor(c), height: 1, letterSpacing: .05, decoration: TextDecoration.none);
  static TextStyle label([Color c = N.g63]) => TextStyle(fontFamily: sans, fontFamilyFallback: const ['.AppleSystemUIFont'], fontSize: 10, color: _floor(c), height: 1, letterSpacing: .1, decoration: TextDecoration.none);
  static TextStyle micro([Color c = N.g63]) => TextStyle(fontFamily: sans, fontFamilyFallback: const ['.AppleSystemUIFont'], fontSize: 10, fontWeight: FontWeight.w500, color: _floor(c), height: 1, letterSpacing: .15, decoration: TextDecoration.none);
  static TextStyle value([Color c = N.g91]) => TextStyle(fontFamily: mono, fontSize: 11, color: _floor(c), height: 1, decoration: TextDecoration.none);
  static TextStyle title([Color c = N.g95]) => TextStyle(fontFamily: sans, fontFamilyFallback: const ['.AppleSystemUIFont'], fontSize: 13, fontWeight: FontWeight.w600, color: _floor(c), height: 1.1, decoration: TextDecoration.none);
}

/// [decided] the timeline's proportions: every size of a row is a fraction of the row.
abstract final class TL {
  static const height = 23.0, label = 180.0, ruler = 20.0;
  static const bar = height * .87, radius = height * .09, key = height * .33;
}
