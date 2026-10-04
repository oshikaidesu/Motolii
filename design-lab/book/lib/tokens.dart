// Every value here is read from ../DESIGN.md (the tag in the comment says where it came from). A part names a token; it never picks a number.
import 'package:flutter/widgets.dart';

/// The one motion of the book: 120 ms, ease-out, started the moment the input lands. Every state change (hover, select, drop) uses this and nothing else.
abstract final class Mo {
  static const dur = Duration(milliseconds: 120);
  static const ease = Curves.easeOut;
}

abstract final class N {
  static const g00 = Color(0xFF000000), g07 = Color(0xFF131313), g10 = Color(0xFF191919), g13 = Color(0xFF202020), g15 = Color(0xFF262626);
  static const g20 = Color(0xFF343434), g26 = Color(0xFF424242), g38 = Color(0xFF616161), g44 = Color(0xFF717171), g56 = Color(0xFF8E8E8E);
  static const g63 = Color(0xFFA1A1A1), g76 = Color(0xFFC1C1C1), g91 = Color(0xFFE7E7E7), g95 = Color(0xFFF2F2F2), g100 = Color(0xFFFFFFFF);
  // [decided] the timeline's two row bands and its row line
  static const bandA = Color(0xFF1E1E1E), bandB = Color(0xFF2A2A2A), rowLine = Color(0xFF161616);
  static const glaze9 = Color(0x17FFFFFF), glaze15 = Color(0x29FFFFFF);
}

/// The window's colour theme. Themed parts (the Workspace and the Inspector library) read [Grey], never [N]; [N] stays the dark
/// ramp of the older studies, which are dark only.
enum Shade { dark, light }

/// The grey ramp that follows [Grey.shade], with [N]'s names. Each step keeps its role in both shades — ground (the gutter),
/// body, card, well, raised, lines, quiet text, text, strong text — only the lightness turns around: in light, cards are the
/// brightest and text the darkest. Dark is exactly [N].
abstract final class Grey {
  static final shade = ValueNotifier<Shade>(Shade.dark);
  static bool get light => shade.value == Shade.light;
  static Color _p(Color dark, Color light) => Grey.light ? light : dark;

  static Color get g00 => _p(N.g00, const Color(0xFFCFD0D4));
  static Color get g07 => _p(N.g07, const Color(0xFFF2F2F4));
  static Color get g10 => _p(N.g10, const Color(0xFFECECEF));
  static Color get g13 => _p(N.g13, const Color(0xFFFFFFFF));
  static Color get g15 => _p(N.g15, const Color(0xFFEAEAED));
  static Color get g20 => _p(N.g20, const Color(0xFFDEDFE3));
  static Color get g26 => _p(N.g26, const Color(0xFFD2D3D8));
  static Color get g38 => _p(N.g38, const Color(0xFFB4B5BC));
  static Color get g44 => _p(N.g44, const Color(0xFF9C9DA5));
  static Color get g56 => _p(N.g56, const Color(0xFF7A7B84));
  static Color get g63 => _p(N.g63, const Color(0xFF696A73));
  static Color get g76 => _p(N.g76, const Color(0xFF4A4B53));
  static Color get g91 => _p(N.g91, const Color(0xFF22232A));
  static Color get g95 => _p(N.g95, const Color(0xFF121318));
  static Color get g100 => _p(N.g100, const Color(0xFF000000));
  static Color get bandA => _p(N.bandA, const Color(0xFFFAFAFB));
  static Color get bandB => _p(N.bandB, const Color(0xFFF0F0F3));
  static Color get rowLine => _p(N.rowLine, const Color(0xFFE3E4E8));
  static Color get glaze9 => _p(N.glaze9, const Color(0x0F000000));
  static Color get glaze15 => _p(N.glaze15, const Color(0x1A000000));
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
  /// [open] Error and destructive actions. Delegates to [Role.error] (Grey keeps #E5584B).
  static Color get danger => Role.error;
  static const playhead = Color(0xFF6EA6DB), play = Color(0xFF7BCC9E), record = Color(0xFFF03C8A);

  /// The accent / mode colour. Delegates to [Role.mode] (Grey keeps #7A87E3).
  static Color get mode => Role.mode;
}

/// Colour palettes of research/color-ud.md: Grey is today's look; A is CUDO-derived, B is Okabe-Ito-derived.
enum Palette { grey, a, b }

/// Colour-vision simulation (Machado 2009, severity 1.0) applied by the lab chrome around a use case. It is a lens, never part of the product.
enum Cvd { none, deut, prot, trit }

/// The ONE place of role colours. A part reads `Role.xxx` and never types a hue. Policy P1: colour only on state marks, small chips,
/// 1 px edges / underlines and the changed value text; no large fills, no stroke over 2 px, at most 2 role colours per row; a non-colour cue is always kept.
abstract final class Role {
  static final palette = ValueNotifier<Palette>(Palette.grey);
  static final cvd = ValueNotifier<Cvd>(Cvd.none);

  /// Fires when either knob moves.
  static final Listenable changes = Listenable.merge([palette, cvd]);

  static Color _pick(Color grey, Color a, Color b) => switch (palette.value) {
    Palette.grey => grey,
    Palette.a => a,
    Palette.b => b,
  };
  static bool get grey => palette.value == Palette.grey;

  static Color get changed => _pick(N.g95, const Color(0xFF66CCFF), const Color(0xFF56B4E9));
  static Color get key => _pick(N.g95, const Color(0xFFFAF500), const Color(0xFFF0E442));
  static Color get linked => _pick(N.g95, const Color(0xFFC77DD8), const Color(0xFFCC79A7));

  /// A relation's own family colour when one is known (it is already CVD-checked); Grey and unknown family fall back to [linked].
  static Color linkedFor(Fam? f) => f == null ? linked : f.c;
  static Color get selected => _pick(const Color(0xFF7A87E3), const Color(0xFF5C8DFF), const Color(0xFF4A8FE0));

  /// A still row with no key yet (hollow diamond): neutral cool grey-blue in A and B, today's grey in Grey. Never the mode hue.
  static Color get still => _pick(N.g95, const Color(0xFF8A98B8), const Color(0xFF8A98B8));

  /// 1 px dark outline of a key diamond in the coloured palettes (the yellow key must stay visible on the yellow Face bar). Grey keeps no outline.
  static Color? get keyEdge => grey ? null : N.g07.withValues(alpha: .6);
  static Color get mode => _pick(const Color(0xFF7A87E3), const Color(0xFFAE9CFF), const Color(0xFF8F9BEA));
  static Color get ok => _pick(const Color(0xFF7BCC9E), const Color(0xFF3DBE82), const Color(0xFF1FB98C));
  static Color get warning => _pick(const Color(0xFFF69260), const Color(0xFFFF9900), const Color(0xFFE69F00));
  static Color get error => _pick(const Color(0xFFE5584B), const Color(0xFFFF4B3A), const Color(0xFFE5484D));

  /// Grey returns the colour the part has today ([today]); a palette returns the role colour. One call keeps Grey pixel-identical.
  static Color of(Color today, Color role) => grey ? today : role;
  static Color get info => N.g63;
  static Color get disabled => N.g44;
}

abstract final class T {
  static const sans = 'Inter', mono = 'Menlo';

  /// No text is dimmer than g56 nor smaller than 10 px (precedents: Adobe Spectrum's smallest UI text is 10 px; critique 2026-10-02: 87 of 126 use cases had 9 px / g44 captions).
  /// Only DIM text on a dark ground is lifted (luminance between g26 and g56); in the light shade, text paler than Grey.g56 is darkened to it. Dark INK (g00..g20, the text on a light fill such as the playhead head) is left alone: lifting it to g56 made it unreadable (critique pass 3).
  /// Test tool, 'Words' addon: when true every T.* text is fully transparent (layout unchanged), so a reviewer can ask "can I tell what this does without reading?".
  static final hidden = ValueNotifier<bool>(false);

  /// The ink of a text: the floor, or nothing while the Words addon says Hidden. Direct TextStyles in the lab use this too.
  static Color ink(Color c) => hidden.value ? const Color(0x00000000) : _floor(c);
  static Color _floor(Color c) {
    if (c.a < .5) return c;
    final l = c.computeLuminance();
    if (Grey.light) return l <= Grey.g56.computeLuminance() || l > .6 ? c : Grey.g56;
    return l >= N.g56.computeLuminance() || l < .04 ? c : N.g56;
  }

  static TextStyle name([Color? c]) => TextStyle(
    fontFamily: sans,
    fontFamilyFallback: const ['.AppleSystemUIFont'],
    fontSize: 11,
    fontWeight: FontWeight.w500,
    color: ink(c ?? Grey.g91),
    height: 1,
    letterSpacing: .05,
    decoration: TextDecoration.none,
  );
  static TextStyle label([Color? c]) => TextStyle(
    fontFamily: sans,
    fontFamilyFallback: const ['.AppleSystemUIFont'],
    fontSize: 10,
    color: ink(c ?? Grey.g63),
    height: 1,
    letterSpacing: .1,
    decoration: TextDecoration.none,
  );
  static TextStyle micro([Color? c]) => TextStyle(
    fontFamily: sans,
    fontFamilyFallback: const ['.AppleSystemUIFont'],
    fontSize: 10,
    fontWeight: FontWeight.w500,
    color: ink(c ?? Grey.g63),
    height: 1,
    letterSpacing: .15,
    decoration: TextDecoration.none,
  );
  static TextStyle value([Color? c]) => TextStyle(fontFamily: mono, fontSize: 11, color: ink(c ?? Grey.g91), height: 1, decoration: TextDecoration.none);
  static TextStyle title([Color? c]) => TextStyle(
    fontFamily: sans,
    fontFamilyFallback: const ['.AppleSystemUIFont'],
    fontSize: 13,
    fontWeight: FontWeight.w600,
    color: ink(c ?? Grey.g95),
    height: 1.1,
    decoration: TextDecoration.none,
  );
}

/// [decided] the timeline's proportions: every size of a row is a fraction of the row.
abstract final class TL {
  static const height = 23.0, label = 180.0, ruler = 20.0;
  static const bar = height * .87, radius = height * .09, key = height * .33;
}

/// [proposal] research/pop-magazine.md section 7: one knob (0..3) drives every seasoning parameter. Dose 0 is today's calm tone; each step only adds.
/// Widgets of the dose study read this class and nothing else for a seasoning number; the plain control group never reads it.
class Dose {
  const Dose._(this.level);
  factory Dose.at(int l) => all[l.clamp(0, 3)];
  final int level;
  static const all = [Dose._(0), Dose._(1), Dose._(2), Dose._(3)];

  X _by<X>(List<X> v) => v[level];

  // Section 7 table, row by row.
  int get loudMax => _by(const [0, 3, 6, 10]);
  double get selAlpha => _by(const [1.0, .5, .75, 1.0]); // hairline accent alpha of the selection frame
  bool get selHue => level >= 1; // dose 0 keeps the draft: a 1px accent underline (controls) or outline (tiles)
  double get innerAlpha => _by(const [0.0, 0.0, 0.0, .2]); // 1px inner highlight under the frame
  double get cut => _by(const [0.0, 0.0, 3.0, 5.0]); // chamfer on selected chip / tile / badge
  double get skewDeg => _by(const [0.0, 0.0, -6.0, -9.0]); // non-editable labels only
  bool get kickerHeader => level >= 1;
  bool get kickerItems => level >= 2;
  bool get kickerSelected => level >= 3;
  int get badgesMax => _by(const [1, 1, 2, 3]);
  bool get badgeLoud => level >= 2; // below this a badge is a plain grey tag
  double get halftoneAlpha => _by(const [0.0, 0.0, .06, .09]);
  double get halftonePitch => _by(const [0.0, 0.0, 4.0, 3.5]);
  bool get offsetShadow => level >= 3;
  double get shadowHueShift => 30; // degrees, Splatoon rule
  int get selMs => _by(const [0, 90, 120, 160]);
  Duration get selDur => Duration(milliseconds: selMs);
  double get satPctMax => _by(const [0.0, .5, 2.0, 4.0]);
  double selLabelPx(bool selected) => selected ? _by(const [11.0, 11.0, 12.0, 13.0]) : 11;
  bool get padHueTicks => level >= 2;
  bool get padCornerTicks => level >= 2;
  bool get padHueCursor => level >= 3;
  double get padCut => _by(const [0.0, 0.0, 0.0, 3.0]);
  int get detentFlashMs => level >= 3 ? 50 : 0;
  bool get hoverDot => level >= 1;
  bool get leadLine => level >= 2;
  bool get headerCut => level >= 3;
  bool get halftone => level >= 2;
  bool get headerNumber => level >= 2;
  double get chipCut => cut;

  /// Element classes allowed to carry energy, with the dose that first allows each (section 7, row 2; the header class from 2 because the table gives halftone at 2).
  static const classMin = {'selection': 1, 'kicker': 1, 'badge': 2, 'pad': 2, 'header': 2};

  // Fixed geometry of the study (section 4, 5, 6): not seasoning, but named once so no widget types a number.
  static const hair = 1.0, hairIdle = .18, crossAlpha = .30, leadAlpha = .22, frameAlpha = .26;
  static const chipW = 24.0, chipH = 20.0, pictoPx = 14.0, plateW = 28.0, plateH = 22.0, thumbW = 28.0, thumbH = 20.0;
  static const curvePx = 1.0, curveSelPx = 1.5, dotPx = 3.0, dotMs = 900;
  static const gap = 4.0, gapL = 8.0, gapXL = 16.0, radius = 4.0;
  static const kickerPx = 10.0, kickerTrack = 2.0, kickerSlot = 14.0, kickerCap = 10.0;
  static const badgeH = 15.0, badgePadX = 5.0;
  static const tileW = 132.0, tileH = 104.0, tilePad = 6.0, tileThumbH = 40.0, tileNameCell = 14.0;
  static const headerW = 300.0, headerH = 42.0, stripH = 8.0;
  static const padSize = 128.0, railW = 24.0, dialR = 64.0, scaleBox = 128.0, scaleBase = 64.0, handle = 5.0, hit = 24.0, cursorR = 3.0;
  static const tickMinor = 3.0, tickMid = 5.0, tickMajor = 8.0, cornerTick = 4.0, diamond = 5.0;
  static const padRange = 960.0, zRange = 100.0; // readout units: comp px
  static const minContrast = 4.5;

  /// The three axis hues come from families (no new hue): X Scatter, Y Along Path, Z Stagger; rotation takes the accent.
  static Color get hueX => Fam.scatter.c;
  static Color get hueY => Fam.along.c;
  static Color get hueZ => Fam.stagger.c;
  static Color get hueR => C.mode;

  static Color shifted(Color c, double deg) {
    final h = HSLColor.fromColor(c);
    return h.withHue((h.hue + deg) % 360).toColor();
  }
}

/// Hands-on GUI instruments (sets/inspector_gui_set.dart): the sizes of the spatial pad, dial, face and layout diagram, in one place.
/// [decided] 280 px inspector column = 12 px gutter + 256 px content. The rest is [open]: judged in the real window.
abstract final class Gui {
  static const content = 256.0, radius = 4.0;
  static const padH = 120.0, rail = 26.0, cell = 80.0, faceH = 150.0, layoutH = 128.0;
  static const hit = 12.0, fine = .1, dot = 3.0, handle = 5.0;
  static const span = 2400.0, zSpan = 2400.0; // composition px across the pad's width / the Z rail's height
  static const comp = Size(1920, 1080);
  static const bodyW = 22.0, bodyH = 13.0; // the body at 100 % on the pad, half extents
  static const orbitR = 36.0, ringR = 54.0;
  static const layK = .45, layCw = 30.0, layCh = 20.0, layN = 6;
  static const field = 22.0, tile = 36.0;

  /// Number-vs-graphic weight, one knob with three steps [open]: 0 the graphic leads (readouts dim and small, instrument strokes bright),
  /// 1 balanced, 2 the numbers lead (readouts bright and larger, instrument strokes quiet). Moved in the real window.
  static const readoutInk = [N.g56, N.g76, N.g95];
  static const readoutSize = [10.0, 11.0, 12.0];
  static const graphicInk = [N.g95, N.g76, N.g63];
  static const gizmoH = 176.0, boost = 1.7, ringBand = 26.0, previewH = 72.0, anchorBox = Size(120, 84); // the unified canvas and the summoned instruments
  static const rowGap = 4.0; // gap between separate fields (Scale, Anchor, Position); stacked Blender-style fields use 1
  /// Decision B ([open], research/open-decisions.md): the colour of the X / Y / Z mark. Fam hues (now), a dedicated X red / Y green / Z blue, or none (letters only).
  static const rgb = [Color(0xFFD9605A), Color(0xFF8CBF5A), Color(0xFF5B8FD6)];
  static List<Color> get fam => [Fam.scatter.c, Fam.along.c, Fam.stagger.c];
}

/// The non-colour half of every error: a small "!" in a 1 px ring. Anything drawn in [Role.error] carries one (policy P1: never colour only).
class ErrMark extends StatelessWidget {
  const ErrMark({super.key, this.size = 11, this.gap = 0, this.lead = 0, this.color});
  final double size, gap, lead;
  final Color? color;

  /// Grey keeps today's look exactly, so the mark (and its gaps) is not there in Grey.
  @override
  Widget build(BuildContext context) => Role.grey
      ? const SizedBox.shrink()
      : Padding(
          padding: EdgeInsets.only(left: lead, right: gap),
          child: SizedBox.square(
            dimension: size,
            child: CustomPaint(painter: _ErrMarkPainter(color ?? Role.error)),
          ),
        );
}

class _ErrMarkPainter extends CustomPainter {
  const _ErrMarkPainter(this.c);
  final Color c;
  @override
  void paint(Canvas cv, Size s) {
    final m = s.center(Offset.zero), r = s.width / 2 - .5;
    final st = Paint()
      ..color = c
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1;
    cv.drawCircle(m, r, st);
    cv.drawLine(Offset(m.dx, m.dy - r * .52), Offset(m.dx, m.dy + r * .12), st);
    cv.drawCircle(Offset(m.dx, m.dy + r * .5), .6, Paint()..color = c);
  }

  @override
  bool shouldRepaint(_ErrMarkPainter o) => o.c != c;
}

/// The non-colour half of ok: a small check in a 1 px ring (not shown in Grey).
class OkMark extends StatelessWidget {
  const OkMark({super.key, this.size = 11, this.gap = 0});
  final double size, gap;
  @override
  Widget build(BuildContext context) => Role.grey
      ? const SizedBox.shrink()
      : Padding(
          padding: EdgeInsets.only(right: gap),
          child: SizedBox.square(
            dimension: size,
            child: CustomPaint(painter: _OkMarkPainter(Role.ok)),
          ),
        );
}

class _OkMarkPainter extends CustomPainter {
  const _OkMarkPainter(this.c);
  final Color c;
  @override
  void paint(Canvas cv, Size s) {
    final w = s.width,
        st = Paint()
          ..color = c
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1
          ..strokeJoin = StrokeJoin.round;
    cv.drawCircle(s.center(Offset.zero), w / 2 - .5, st);
    cv.drawPath(
      Path()
        ..moveTo(w * .28, w * .52)
        ..lineTo(w * .44, w * .67)
        ..lineTo(w * .72, w * .36),
      st,
    );
  }

  @override
  bool shouldRepaint(_OkMarkPainter o) => o.c != c;
}
