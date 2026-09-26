// Visual-state sheet. Every state here except "idle" is DERIVED - REQUIRES VALIDATION.
// flutter run -d macos -t lib/proto_hf/main_states.dart [--dart-define=PROTO_SHOT=/path.png]
import 'dart:io';
import 'dart:ui' as ui;
import 'package:flutter/rendering.dart';
import 'package:flutter/widgets.dart';
import 'glyphs.dart';
import 'ref.dart';

const _shot = String.fromEnvironment('PROTO_SHOT');
final _root = GlobalKey();
enum St { idle, hover, focus, pressed, disabled, selected }
const _names = ['idle', 'hover', 'focus', 'pressed', 'disabled', 'selected'];
double cx(St s) => 470.0 + 170.0 * s.index;

Color lift(Color c, int d) => Color.fromARGB(255, (c.r * 255 + d).round().clamp(0, 255), (c.g * 255 + d).round().clamp(0, 255), (c.b * 255 + d).round().clamp(0, 255));
Color mix(Color a, Color b, double t) => Color.lerp(a, b, t)!;
const dis = 0.38;
Rc ring(double x, double y, double w, double h, double r) => Rc(x - 3, y - 3, w + 6, h + 6, border: const Color(0xB3E9E9EC), bw: 1.5, r: r + 3);

List<RI> sheet() {
  final it = <RI>[
    Rc(0, 0, 1500, 800, fill: H.window),
    Tx(30, 40, 'VISUAL STATE SHEET', H.s(15, w: FontWeight.w600, ls: 1.0)),
    Tx(30, 62, 'idle = measured from the reference.  All other states: DERIVED - REQUIRES VALIDATION.', H.s(12, color: const Color(0xFFB0B0B0))),
    for (final s in St.values) Tx(cx(s), 96, _names[s.index].toUpperCase(), H.s(11, w: FontWeight.w600, ls: 1.2, color: const Color(0xFF9A9A9A)), w: null),
  ];
  const ink = Color(0xFF14171A);
  void label(double y, String t, String rule) {
    it.add(Ln(30, y - 22, 1440, 1, const Color(0xFF2A2A2A)));
    it.add(Tx(30, y + 4, t, H.s(12.5, w: FontWeight.w600)));
    final parts = rule.split(' | ');
    final half = (parts.length + 1) ~/ 2;
    it.add(Tx(30, y + 19, parts.sublist(0, half).join('  |  '), H.s(10, color: const Color(0xFF8A8A8A))));
    if (parts.length > half) it.add(Tx(30, y + 32, parts.sublist(half).join('  |  '), H.s(10, color: const Color(0xFF8A8A8A))));
  }

  // ---- neutral tile
  var y = 130.0;
  label(y + 26, 'Neutral tile', 'hover +7 surface, +8 border | pressed -5 | focus ring | disabled 38%');
  for (final s in [St.idle, St.hover, St.focus, St.pressed, St.disabled]) {
    final f = switch (s) { St.hover => 0xFF272727, St.pressed => 0xFF1A1A1A, _ => 0xFF202020 };
    final b = s == St.hover ? const Color(0xFF3A3A3A) : const Color(0xFF313131);
    final a = s == St.disabled ? dis : 1.0;
    if (s == St.focus) it.add(ring(cx(s), y, 76, 60, 4));
    it.add(Rc(cx(s), y, 76, 60, fill: Color(f), border: b, r: 4));
    it.add(Hg(cx(s) + 38, y + 24, 26, HG.shape, const Color(0xFFB3B3B5), a: a));
    it.add(Tx(cx(s) + 38, y + 50, 'Shape', H.s(12.5, color: Color.fromRGBO(208, 209, 210, a)), al: Al.center));
  }
  // ---- relation tile
  y = 230;
  label(y + 26, 'Relation tile', 'hover lighten 7% | pressed darken 10% | focus ring | disabled: 30% of family colour');
  for (final s in [St.idle, St.hover, St.focus, St.pressed, St.disabled]) {
    var f = H.scatter.b;
    if (s == St.hover) f = mix(f, const Color(0xFFFFFFFF), .07);
    if (s == St.pressed) f = mix(f, const Color(0xFF000000), .10);
    if (s == St.disabled) f = mix(H.raised, H.scatter.b, .30);
    if (s == St.focus) it.add(ring(cx(s), y, 98, 60, 4));
    it.add(Rc(cx(s), y, 98, 60, fill: f, border: shade2(f), r: 4));
    it.add(Hg(cx(s) + 28, y + 24, 26, HG.scatter, s == St.disabled ? const Color(0x6614171A) : ink, bg: f));
    it.add(Tx(cx(s) + 14, y + 50, 'Scatter', H.s(12.5, w: FontWeight.w600, color: s == St.disabled ? const Color(0x6614171A) : ink)));
  }
  // ---- tab
  y = 310;
  label(y + 22, 'Tab', 'hover: surface #232323 | selected = measured | pressed selected -4 | disabled 38%');
  for (final s in [St.idle, St.hover, St.focus, St.pressed, St.disabled, St.selected]) {
    var f = const Color(0x00000000);
    if (s == St.hover) f = const Color(0xFF232323);
    if (s == St.selected) f = H.sel;
    if (s == St.pressed) f = Color.lerp(H.sel, const Color(0xFF000000), .12)!;
    if (s == St.focus) it.add(ring(cx(s), y, 78, 37, 2));
    it.add(Rc(cx(s), y, 78, 37, fill: f, r: 2));
    it.add(Tx(cx(s) + 39, y + 23, 'OBJECTS', H.s(12, w: FontWeight.w500, ls: .7, color: s == St.disabled ? const Color(0x61DDE0E0) : (s == St.selected || s == St.pressed ? const Color(0xFFFDFDFD) : const Color(0xFFDDE0E0))), al: Al.center));
  }
  // ---- transport / icon keys
  y = 378;
  label(y + 22, 'Key (transport, icon)', 'hover +8 | pressed -8 and no shadow | focus ring | disabled 38%');
  for (final s in [St.idle, St.hover, St.focus, St.pressed, St.disabled]) {
    var f = const Color(0xFF1E1E1E);
    if (s == St.hover) f = const Color(0xFF262626);
    if (s == St.pressed) f = const Color(0xFF171717);
    if (s == St.focus) it.add(ring(cx(s), y, 40, 37, 3));
    it.add(Rc(cx(s), y, 40, 37, fill: f, border: const Color(0xFF343636), r: 3));
    it.add(Hg(cx(s) + 20, y + 18.5, 21, HG.folder, const Color(0xFFCCCECE), a: s == St.disabled ? dis : 1));
    var pf = H.play;
    if (s == St.hover) pf = mix(pf, const Color(0xFFFFFFFF), .07);
    if (s == St.pressed) pf = mix(pf, const Color(0xFF000000), .12);
    if (s == St.disabled) pf = mix(H.window, H.play, .35);
    if (s == St.focus) it.add(ring(cx(s) + 60, y, 41, 37, 3));
    it.add(Rc(cx(s) + 60, y, 41, 37, fill: pf, border: pf, r: 3));
    it.add(Hg(cx(s) + 80.5, y + 18.5, 18, HG.play, const Color(0xFF040709), a: s == St.disabled ? .5 : 1));
  }
  // ---- toggle
  y = 448;
  label(y + 18, 'Toggle', 'on/off measured | hover knob +2 | pressed knob widens 20 | focus ring | disabled 38%');
  for (final s in [St.idle, St.hover, St.focus, St.pressed, St.disabled, St.selected]) {
    final on = s != St.selected;
    final a = s == St.disabled ? dis : 1.0;
    if (s == St.focus) it.add(ring(cx(s), y, 37, 20, 10));
    final w = s == St.pressed ? 20.0 : 16.0;
    it.add(Rc(cx(s), y, 37, 20, fill: (on ? H.toggleOn : H.toggleOff).withValues(alpha: a), r: 10));
    it.add(Rc(on ? cx(s) + 35 - w - 0 : cx(s) + 3, y + 2, w, 16, fill: Color.fromRGBO(252, 252, 254, a), r: 8));
    it.add(Tx(cx(s) + 46, y + 14, on ? 'on' : 'off', H.s(11, color: const Color(0xFF8A8A8A))));
  }
  // ---- slider
  y = 500;
  label(y + 14, 'Slider', 'hover: thumb 12 | active: thumb 12 + ring | focus ring on thumb | disabled 35%');
  for (final s in [St.idle, St.hover, St.focus, St.pressed, St.disabled]) {
    final a = s == St.disabled ? .35 : 1.0;
    it.add(Rc(cx(s), y + 8, 147, 5, fill: const Color(0xFF2C2C2C), r: 3));
    it.add(Rc(cx(s), y + 8, 83, 5, fill: H.scatter.i.withValues(alpha: a), r: 3));
    final d = (s == St.hover || s == St.pressed) ? 12.0 : 10.0;
    if (s == St.pressed) it.add(Rc(cx(s) + 83 - 10, y + 10.5 - 10, 20, 20, fill: H.scatter.i.withValues(alpha: .22), r: 10));
    if (s == St.focus) it.add(Rc(cx(s) + 83 - 9, y + 10.5 - 9, 18, 18, border: const Color(0xB3E9E9EC), bw: 1.5, r: 9));
    it.add(Rc(cx(s) + 83 - d / 2, y + 10.5 - d / 2, d, d, fill: H.scatter.i.withValues(alpha: a), r: d / 2));
  }
  // ---- folded row
  y = 548;
  label(y + 22, 'Folded relation row', 'hover #262626 | pressed #1D1D1D | focus ring | disabled 40% content, dot stays');
  for (final s in [St.idle, St.hover, St.focus, St.pressed, St.disabled]) {
    var f = const Color(0xFF212120);
    if (s == St.hover) f = const Color(0xFF262626);
    if (s == St.pressed) f = const Color(0xFF1D1D1D);
    final a = s == St.disabled ? .4 : 1.0;
    if (s == St.focus) it.add(ring(cx(s), y, 160, 38, 0));
    it.add(Rc(cx(s), y, 160, 38, fill: f, border: const Color(0xFF303030)));
    it.add(Rc(cx(s) + 15, y + 10, 18, 18, fill: H.stagger.i.withValues(alpha: a), r: 9));
    it.add(Tx(cx(s) + 47, y + 24.5, 'Stagger', H.s(16, w: FontWeight.w600, ls: .2, color: Color.fromRGBO(247, 248, 248, a))));
    it.add(Rc(cx(s) + 116, y + 9, 37, 20, fill: H.toggleOn.withValues(alpha: a), r: 10));
    it.add(Rc(cx(s) + 116 + 19, y + 11, 16, 16, fill: Color.fromRGBO(252, 252, 254, a), r: 8));
  }
  // ---- preset row
  y = 606;
  label(y + 14, 'Preset row', 'hover #202020 | selected measured | pressed selected -3 | focus ring');
  for (final s in [St.idle, St.hover, St.focus, St.pressed, St.disabled, St.selected]) {
    var f = const Color(0x00000000);
    Color? b;
    if (s == St.hover || s == St.focus) f = const Color(0xFF202020);
    if (s == St.selected) { f = const Color(0xFF292929); b = const Color(0xFF404041); }
    if (s == St.pressed) { f = const Color(0xFF242424); b = const Color(0xFF3A3A3B); }
    if (s == St.focus) it.add(ring(cx(s), y, 160, 30, 3));
    it.add(Rc(cx(s), y, 160, 30, fill: f, border: b, r: 3));
    final a = s == St.disabled ? .38 : 1.0;
    it.add(Hg(cx(s) + 15, y + 15, 17, HG.preset, Color.fromRGBO(158, 159, 161, a), bg: f.a == 0 ? H.window : f));
    it.add(Tx(cx(s) + 30, y + 19, 'Jewel Field', H.s(12.5, color: Color.fromRGBO(218, 220, 221, a))));
  }
  // ---- timeline row + bar
  y = 662;
  label(y + 30, 'Timeline row + bar', 'hover row +6 | selected (no selection in reference): row +10 and 3px identity edge | disabled: bar 35%');
  for (final s in [St.idle, St.hover, St.focus, St.pressed, St.disabled, St.selected]) {
    var f = const Color(0xFF2B2B2B);
    if (s == St.hover) f = const Color(0xFF313131);
    if (s == St.selected) f = const Color(0xFF353535);
    it.add(Rc(cx(s), y, 160, 21, fill: f));
    if (s == St.selected) it.add(Rc(cx(s), y, 3, 21, fill: H.scatter.n));
    if (s == St.focus) it.add(Rc(cx(s) - 1, y - 1, 162, 23, border: const Color(0xB3E9E9EC), bw: 1.5));
    final bc = s == St.disabled ? mix(f, H.scatter.t, .35) : (s == St.hover ? mix(H.scatter.t, const Color(0xFFFFFFFF), .08) : H.scatter.t);
    it.add(Rc(cx(s) + 20, y + 2, 130, 18, fill: bc, border: s == St.selected ? const Color(0xCCF0F0F0) : shade2(bc), r: 3));
  }
  // ---- keyframe
  y = 730;
  label(y + 10, 'Keyframe', 'idle = the one reference appearance | hover, pressed larger | selected: filled white (derived) | disabled 35%');
  for (final s in [St.idle, St.hover, St.focus, St.pressed, St.disabled, St.selected]) {
    it.add(Rc(cx(s), y - 6, 60, 22, fill: H.scatter.t, r: 3));
    it.add(Pt(_KeyP(cx(s) + 30, y + 5, s)));
  }
  return it;
}

Color shade2(Color c) => mix(c, const Color(0xFF000000), .16);

class _KeyP extends CustomPainter {
  _KeyP(this.x, this.y, this.s);
  final double x, y;
  final St s;
  @override
  void paint(Canvas cv, Size sz) {
    // idle = the reference: a small diamond in the body's own pale tint. The rest is DERIVED.
    final body = H.scatter.t;
    Color w(double t) => Color.lerp(body, const Color(0xFFFFFFFF), t)!;
    final size = (s == St.hover || s == St.pressed) ? 7.2 : 6.2;
    final a = s == St.disabled ? .35 : 1.0;
    final fill = s == St.selected ? const Color(0xFFFFFFFF) : (s == St.pressed ? w(.8) : w(.55));
    final edge = s == St.selected ? const Color(0xFFFFFFFF) : w(.78);
    cv.save();
    cv.translate(x, y);
    cv.rotate(0.7853981);
    final r = Rect.fromCenter(center: Offset.zero, width: size, height: size);
    cv.drawRect(r, Paint()..color = fill.withValues(alpha: a));
    cv.drawRect(r, Paint()..color = edge.withValues(alpha: a)..style = PaintingStyle.stroke..strokeWidth = 1);
    cv.restore();
    if (s == St.focus) cv.drawCircle(Offset(x, y), 8.5, Paint()..color = const Color(0xB3E9E9EC)..style = PaintingStyle.stroke..strokeWidth = 1.3);
  }
  @override
  bool shouldRepaint(_KeyP o) => false;
}

void main() {
  runApp(WidgetsApp(
    color: H.window,
    debugShowCheckedModeBanner: false,
    builder: (_, __) => Align(
      alignment: Alignment.topLeft,
      child: OverflowBox(
        alignment: Alignment.topLeft, minWidth: 1500, maxWidth: 1500, minHeight: 800, maxHeight: 800,
        child: RepaintBoundary(key: _root, child: DefaultTextStyle(style: H.s(13), child: RF(sheet()))),
      ),
    ),
  ));
  if (_shot.isNotEmpty) {
    Future<void>.delayed(const Duration(seconds: 2), () async {
      final b = _root.currentContext!.findRenderObject()! as RenderRepaintBoundary;
      final img = await b.toImage(pixelRatio: 1.5);
      File(_shot).writeAsBytesSync((await img.toByteData(format: ui.ImageByteFormat.png))!.buffer.asUint8List());
      exit(0);
    });
  }
}
