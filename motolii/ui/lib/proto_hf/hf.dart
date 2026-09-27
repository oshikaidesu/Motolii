import 'dart:io';
import 'dart:math' as math;
import 'package:flutter/widgets.dart';
import '../hf/glyphs.dart';
import 'hg_item.dart';
import 'ref.dart';
import '../hf/shell/top.dart' as shell_top show top;
import '../hf/shell/top.dart' show TopModel;
import '../hf/shell/stage.dart' as shell_stage show stage;
import '../hf/shell/stage.dart' show StageModel;

Color shade(Color c, double t) => Color.lerp(c, const Color(0xFF000000), t)!;
const _stageArt = String.fromEnvironment('PROTO_STAGE', defaultValue: '/Users/member_ottoto/rust_ae/Motolii/motolii/ui/lib/proto/stage_hf.png');

List<RI> all() => [Rc(0, 0, 1536, 1024, fill: H.window), ...top(), ...browser(), ...stage(), ...inspector(), ...timeline()];

// ------------------------------------------------------------------ TOP
// The face is hf/shell/top.dart; the fixture is the reference's own values.
final _fixtureReadouts = ValueNotifier(const ['120.00', '4 / 4', '00:02:13']);
List<RI> top() => shell_top.top(TopModel(readouts: _fixtureReadouts));

// -------------------------------------------------------------- BROWSER
List<RI> browser() {
  final items = <RI>[
    Ln(10, 62, 1, 953, H.rule), Ln(333, 62, 1, 953, H.rule), Ln(10, 62, 324, 1, H.rule), Ln(10, 1014, 324, 1, H.rule),
    Rc(11, 71, 78, 37, fill: H.sel, r: 2),
    Tx(23, 93, 'OBJECTS', H.s(12.5, w: FontWeight.w500, ls: 0.7, color: H.text), w: 52),
    Tx(100, 93, 'RELATIONS', H.s(12.5, w: FontWeight.w500, ls: 0.7, color: const Color(0xFFC0C2C2)), w: 59),
    Tx(182, 93, 'EFFECTS', H.s(12.5, w: FontWeight.w500, ls: 0.7, color: const Color(0xFFC0C2C2)), w: 45),
    Tx(250, 93, 'MEDIA', H.s(12.5, w: FontWeight.w500, ls: 0.7, color: const Color(0xFFC0C2C2)), w: 33),
    Rc(296, 71, 36, 37, fill: H.raised, border: H.rule, r: 3),
    Hg(314, 89, 20, HG.search, const Color(0xFFBAB9B9)),
    Ln(11, 108, 322, 1, H.rule2),
  ];
  TextStyle lab() => H.s(12.5, w: FontWeight.w500, ls: 0.7, color: H.text2);
  items.addAll([
    Tx(20, 132, 'PRIMITIVES', lab(), w: 73), Tx(20, 261, 'GENERATORS', lab(), w: 77), Tx(20, 390, 'RELATIONS', lab(), w: 67),
    Tx(20, 587, 'EFFECTS', lab(), w: 51), Tx(20, 783, 'PRESETS', lab(), w: 52),
  ]);
  // Primitive / Generator: 4 columns, centred, neutral.
  void neutral4(double top, List<(String, HG)> t) {
    for (var i = 0; i < 4; i++) {
      final x = 14.0 + 80 * i;
      items.add(Rc(x, top, 75, 85, fill: H.raised, border: H.rule, r: 4));
      items.add(Hg(x + 37.5, top + 32.5, t[i].$2 == HG.shape ? 26 : ((t[i].$2 == HG.image || t[i].$2 == HG.camera) ? 27 : 28), t[i].$2, const Color(0xFFB2B2B4), bg: H.raised));
      items.add(Tx(x + 37.5, top + 69, t[i].$1, H.s(12, color: const Color(0xFFBFC0C2)), al: Al.center, w: t[i].$1 == 'Camera' ? 42 : null));
    }
  }
  neutral4(145, [('Text', HG.text), ('Shape', HG.shape), ('Image', HG.image), ('Camera', HG.camera)]);
  neutral4(275, [('Repeater', HG.repeater), ('Grid', HG.grid), ('Circle', HG.circle), ('Spiral', HG.spiral)]);
  // Relation: 3 columns, coloured, left aligned.
  const rx = [15.0, 121.0, 226.0];
  final rel = [
    [('Scatter', HG.scatter, H.scatter.b), ('Along Path', HG.alongPath, H.along.b), ('Stagger', HG.stagger, H.stagger.b)],
    [('Face', HG.face, H.face.b), ('Follow', HG.follow, H.follow.b), ('Attach', HG.attach, H.attach.b)],
  ];
  for (var r = 0; r < 2; r++) {
    final top = r == 0 ? 402.0 : 488.0;
    for (var c = 0; c < 3; c++) {
      final (n, g, col) = rel[r][c];
      items.add(Rc(rx[c], top, c == 2 ? 100 : 98, 80, fill: col, border: shade(col, .14), r: 4));
      items.add(Hg(rx[c] + 28, top + 28.5, 29, g, const Color(0xFF14171A), bg: rel[r][c].$3));
      items.add(Tx(rx[c] + 14, top + 66, n, H.s(12.5, w: FontWeight.w600, color: const Color(0xFF14171A))));
    }
  }
  // Effects: 3 columns, neutral, left aligned.
  final eff = [
    [('Blur', HG.blur), ('Glow', HG.glow), ('Color', HG.color)],
    [('Composite', HG.composite), ('Distort', HG.distort), ('Stylize', HG.stylize)],
  ];
  for (var r = 0; r < 2; r++) {
    final top = r == 0 ? 597.0 : 680.0;
    for (var c = 0; c < 3; c++) {
      items.add(Rc(rx[c], top, 97, 78, fill: H.raised, border: H.rule, r: 4));
      items.add(Hg(rx[c] + 28.5, top + 28, eff[r][c].$2 == HG.glow ? 26 : 28, eff[r][c].$2, const Color(0xFFB2B2B4), bg: H.raised));
      items.add(Tx(rx[c] + 14, top + 63, eff[r][c].$1, H.s(12, color: const Color(0xFFBFC0C2))));
    }
  }
  // Presets
  items.add(Rc(12, 794, 316, 30, fill: const Color(0xFF292929), border: H.rule, r: 3));
  const pn = ['Jewel Field', 'Neon Tunnel', 'Photo Scatter', 'VJ Loop', 'Title Minimal'];
  const pb = [813.0, 842.0, 871.0, 900.0, 928.0];
  for (var i = 0; i < 5; i++) {
    items.add(Hg(30.5, pb[i] - 4.3, 14, HG.preset, const Color(0xFF85868A), bg: i == 0 ? const Color(0xFF292929) : H.window));
    items.add(Tx(54, pb[i], pn[i], H.s(12, color: i == 0 ? const Color(0xFFD2D2D2) : const Color(0xFFC6C7C8)), w: const [64.0, 71.0, 82.0, 47.0, 75.0][i]));
  }
  items.addAll([
    Ln(11, 943, 322, 1, H.rule2),
    Hg(31, 965, 22, HG.plus, const Color(0xFFCECFCF)),
    Tx(54, 969, 'New Preset', H.s(12.5, color: H.text2), w: 66),
    Ln(11, 986, 322, 1, H.rule2),
    Tx(18, 1008, 'v0.5.0', H.m(12, color: H.text2), w: 45),
  ]);
  return items;
}

// ---------------------------------------------------------------- STAGE
// The seat is hf/shell/stage.dart; under the tab row is the reference's mock Stage.
List<RI> stage() => shell_stage.stage(StageModel(body: _mockStage()));

List<RI> _mockStage() {
  const tools = [HG.arrow, HG.move, HG.rect, HG.ellipse, HG.pen, HG.type, HG.crop];
  const dd = Color(0xFF1D1D1D);
  const ddB = Color(0xFF3E3E3D);
  return [
    Wd(345, 101, 777, 537, Image.file(File(_stageArt), fit: BoxFit.fill, filterQuality: FilterQuality.medium)),
    Ln(345, 637, 777, 1, H.rule2),
    // tool column, over the work
    Rc(354, 111, 42, 304, fill: H.raised, border: H.rule, r: 2),
    Rc(355, 112, 40, 43, fill: H.selHi),
    for (var i = 1; i < 7; i++) Ln(355, 111 + 43.4 * i, 40, 1, H.rule2),
    for (var i = 0; i < 7; i++) Hg(375, 111 + 43.4 * i + 21.7, 23, tools[i], i == 0 ? const Color(0xFFEEEEF0) : const Color(0xFFD8D8DA), bg: const Color(0xFF222223)),
    // bottom bar
    Rc(354, 649, 117, 33, fill: dd, border: ddB, r: 3),
    Tx(371, 670, '100%', H.s(13, color: H.text), w: 31),
    Hg(453, 665, 15, HG.chevronDown, const Color(0xFFDDDDDD)),
    Rc(489, 649, 38, 33, fill: dd, border: ddB, r: 3), Hg(508, 665, 20, HG.fit, const Color(0xFFD0D0D0)),
    Rc(534, 649, 37, 33, fill: dd, border: ddB, r: 3), Hg(552, 665, 20, HG.fit, const Color(0xFFD0D0D0)),
    Rc(894, 649, 144, 33, fill: dd, border: ddB, r: 3),
    Tx(905, 670, 'Camera View', H.s(13, color: H.text), w: 80),
    Hg(1020, 665, 15, HG.chevronDown, const Color(0xFFDDDDDD)),
    Rc(1057, 649, 37, 33, fill: dd, border: ddB, r: 3), Hg(1075.5, 665, 20, HG.corners, const Color(0xFFD0D0D0)),
  ];
}

// ------------------------------------------------------------ INSPECTOR
List<RI> inspector() {
  final items = <RI>[
    Ln(1135, 62, 1, 631, H.rule), Ln(1521, 62, 1, 631, H.rule), Ln(1135, 62, 387, 1, H.rule), Ln(1135, 692, 387, 1, H.rule),
    Rc(1135, 71, 107, 37, fill: H.sel, r: 2),
    Tx(1155, 93, 'INSPECTOR', H.s(12.5, w: FontWeight.w500, ls: 0.7, color: H.text), w: 68),
    Tx(1265, 93, 'PROJECT', H.s(12.5, w: FontWeight.w500, ls: 0.7, color: const Color(0xFFC0C2C2)), w: 52),
    Tx(1367, 93, 'LOOK', H.s(12.5, w: FontWeight.w500, ls: 0.7, color: const Color(0xFFC0C2C2)), w: 30),
    Rc(1483, 72, 38, 35, fill: H.raised, border: H.rule, r: 3),
    Ln(1135, 108, 386, 1, H.rule),
    // header
    Tx(1192, 131, 'Group', H.s(12.5, color: H.text2), w: 35),
    Rc(1148, 134, 28, 27, fill: H.scatter.i, r: 3),
    Rc(1190, 140, 260, 29, fill: H.raised, border: H.rule, r: 3),
    Tx(1198, 159, 'Jewel Field', H.s(15, w: FontWeight.w500, color: H.text), w: 80),
    Rc(1467, 132, 39, 36, fill: H.raised, border: H.rule, r: 3),
    Hg(1486, 150, 23, HG.lock, const Color(0xFFC8C8C8), bg: const Color(0xFF222222)),
    // mode tabs
    Rc(1134, 182, 387, 36, fill: H.raised, border: H.rule, r: 3),
    Rc(1237, 183, 102, 34, fill: H.selHi),
    Ln(1430, 183, 1, 34, H.rule2),
    Tx(1155, 204, 'Transform', H.s(12.5, color: const Color(0xFFB4B5B5)), w: 60),
    Tx(1259, 204, 'Relations', H.s(12.5, w: FontWeight.w500, color: H.text), w: 58),
    Tx(1362, 204, 'Effects', H.s(12.5, color: const Color(0xFFB4B5B5)), w: 42),
    Tx(1452, 204, 'Material', H.s(12.5, color: const Color(0xFFB4B5B5)), w: 47),
    // Scatter card
    Rc(1134, 225, 388, 290, fill: H.raised, border: H.rule, r: 2),
    Rc(1147, 240, 17, 17, fill: H.scatter.i, r: 8.5),
    Tx(1181, 254, 'Scatter', H.s(21, w: FontWeight.w500, ls: 0.4, color: H.text), w: 67),
    Tx(1181, 274, 'Scatter points to make spread.', H.s(12, color: H.text3), w: 162),
    Rc(1440, 242, 37, 20, fill: H.toggleOn, r: 10), Rc(1459, 244, 16, 16, fill: const Color(0xFFFCFCFE), r: 8),
    Ln(1135, 284, 386, 1, H.rule),
    Ln(1342, 285, 1, 229, H.rule2),
    Pt(_InspectorPaint()),
    Tx(1354, 317, 'Density', H.s(12.5, color: H.text2), w: 42),
    Tx(1354, 366, 'Spread', H.s(12.5, color: H.text2), w: 38),
    Tx(1354, 410, 'Falloff', H.s(12.5, color: H.text2), w: 36),
    Tx(1354, 468, 'Shape', H.s(12.5, color: H.text2), w: 34),
  ];
  for (final (yy, v) in [(307.0, '0.72'), (355.5, '0.48'), (399.0, '0.36')]) {
    items.add(Rc(1467, yy, 36, 19, fill: H.raisedHi, r: 3));
    items.add(Tx(1498, yy + 10.5, v, H.m(12, color: H.text2), al: Al.right, w: 25));
  }
  // Shape selector
  items.addAll([
    Rc(1354, 476, 27, 26, fill: H.scatter.b, r: 3),
    Hg(1367.5, 489, 19, HG.circle, const Color(0xFF14171A)),
    Hg(1399, 489, 20, HG.rect, const Color(0xFFE6E6E6)),
    Hg(1430, 489, 21, HG.triangle, const Color(0xFFE6E6E6)),
    Hg(1460, 489, 20, HG.cross, const Color(0xFFE6E6E6)),
    Hg(1491, 489, 21, HG.star, const Color(0xFFE6E6E6)),
  ]);
  // Folded rows
  final folded = [(529.0, 'Stagger', H.stagger.i, true), (569.5, 'Along Path', H.along.i, true), (610.0, 'Face Target', H.face.i, false)];
  for (final (top, name, col, on) in folded) {
    items.addAll([
      Rc(1134, top, 388, 38, fill: H.raised, border: H.rule, r: 2),
      Rc(1149, top + 10, 18, 18, fill: col, r: 9),
      Tx(1181, top + 24.5, name, w: const {'Stagger': 60.0, 'Along Path': 84.0, 'Face Target': 91.0}[name], H.s(16, w: FontWeight.w500, ls: 0.3, color: H.text)),
      Rc(1440, top + 9, 37, 20, fill: on ? H.toggleOn : H.toggleOff, r: 10),
      Rc(on ? 1459 : 1442, top + 11, 16, 16, fill: const Color(0xFFFCFCFE), r: 8),
    ]);
  }
  items.addAll([
    Rc(1134, 654, 388, 38, fill: H.raised, border: H.rule, r: 2),
    Hg(1158, 673.5, 24, HG.plus, const Color(0xFFE6E7E7)),
    Tx(1181, 676.5, 'Add Relation', H.s(13, w: FontWeight.w400, color: H.text2), w: 78),
    Pt(_KebabPaint()),
  ]);
  return items;
}

class _KebabPaint extends CustomPainter {
  @override
  void paint(Canvas cv, Size s) {
    final k = Paint()..color = const Color(0xFFD6D6D6);
    for (final y in [548.0, 588.5, 629.0]) {
      for (final d in [-6.0, 0.0, 6.0]) { cv.drawCircle(Offset(1498, y + d), 1.7, k); }
    }
  }
  @override
  bool shouldRepaint(_KebabPaint o) => false;
}

class _InspectorPaint extends CustomPainter {
  @override
  void paint(Canvas cv, Size s) {
    // kebabs
    final k = Paint()..color = const Color(0xFFD6D6D6);
    for (final y in [251.0]) {
      for (final d in [-6.0, 0.0, 6.0]) { cv.drawCircle(Offset(1498, y + d), 1.7, k); }
    }
    for (final d in [-9.0, 0.0, 9.0]) { cv.drawCircle(Offset(1502 - 1 + d, 89.5), 1.6, Paint()..color = const Color(0xFFE8E8E8)); }
    // visualizer
    const c = Offset(1240, 402);
    final dash = Paint()..color = const Color(0xFF5F5F5F)..style = PaintingStyle.stroke..strokeWidth = 1;
    for (final m in (Path()..addOval(Rect.fromCircle(center: c, radius: 93))).computeMetrics()) {
      var d = 0.0;
      while (d < m.length) { cv.drawPath(m.extractPath(d, d + 4), dash); d += 8.5; }
    }
    final rnd = math.Random(9);
    final pink = Paint()..color = H.scatter.i;
    for (final (rad, n) in [(21.0, 6), (41.0, 10), (60.0, 13), (78.0, 15)]) {
      for (var i = 0; i < n; i++) {
        final a = i / n * math.pi * 2 + rad * .13 + rnd.nextDouble() * .18;
        final r = rad + (rnd.nextDouble() - .5) * 9;
        final o = c + Offset(math.cos(a) * r, math.sin(a) * r);
        final sz = 3.8 + rnd.nextDouble() * 3.8;
        cv.save();
        cv.translate(o.dx, o.dy);
        if (i % 3 == 0) {
          cv.drawCircle(Offset.zero, sz / 2, pink);
        } else {
          cv.rotate(math.pi / 4);
          cv.drawRect(Rect.fromCenter(center: Offset.zero, width: sz, height: sz), pink);
        }
        cv.restore();
      }
    }
    final cross = Paint()..color = const Color(0xFFF0F0F0)..strokeWidth = 1.3;
    cv.drawLine(c - const Offset(11, 0), c + const Offset(11, 0), cross);
    cv.drawLine(c - const Offset(0, 11), c + const Offset(0, 11), cross);
    const h = Offset(1295, 332.5);
    cv.drawCircle(h, 6.5, Paint()..color = H.scatter.i..style = PaintingStyle.stroke..strokeWidth = 3);
    cv.drawCircle(h, 2.2, Paint()..color = const Color(0xFF3A1F2E));
    // sliders (geometry measured; values do not map, see handoff 9)
    for (final (y, fillTo) in [(332.5, 1437.0), (381.5, 1418.0)]) {
      cv.drawRRect(RRect.fromRectAndRadius(Rect.fromLTRB(1354, y - 2.5, 1501, y + 2.5), const Radius.circular(3)), Paint()..color = const Color(0xFF2C2C2C));
      cv.drawRRect(RRect.fromRectAndRadius(Rect.fromLTRB(1354, y - 3.5, fillTo, y + 3.5), const Radius.circular(3.5)), Paint()..color = H.scatter.i);
      cv.drawCircle(Offset(fillTo, y), 4.6, Paint()..color = H.scatter.i);
    }
    // falloff curve
    final p = Path()..moveTo(1354, 449)..cubicTo(1380, 449, 1410, 421, 1437, 421)..cubicTo(1465, 421, 1480, 452, 1502, 452);
    cv.drawPath(p, Paint()..color = H.scatter.i..style = PaintingStyle.stroke..strokeWidth = 1.5);
    for (final o in const [Offset(1354, 449), Offset(1502, 452)]) { cv.drawCircle(o, 2, Paint()..color = const Color(0xFFE6E6E6)); }
    cv.drawCircle(const Offset(1395, 436), 3.2, Paint()..color = const Color(0xFF1F201F));
    cv.drawCircle(const Offset(1395, 436), 3.2, Paint()..color = const Color(0xFFE6E6E6)..style = PaintingStyle.stroke..strokeWidth = 1.2);
    cv.drawCircle(const Offset(1437, 421), 3.5, Paint()..color = H.scatter.i);
  }
  @override
  bool shouldRepaint(_InspectorPaint o) => false;
}

// ------------------------------------------------------------- TIMELINE
// Layers, bottom to top: panel ground, alternating row ground, hierarchy, ruler,
// major/minor time grid, temporal bodies, keyframes, playhead, waveform (a floor, not a body).
// Row height and bar height are separate tokens; the bar leaves a fixed negative space.
const _tlTop = 775.0, _pitch = 23.0, _rowH = 22.0, _barH = 18.0;
const _audioTop = 959.0, _audioH = 34.0;
const _x0 = 559.0, _unit = 92.9;
double rowTop(int i) => i < 8 ? _tlTop + _pitch * i : _audioTop;
double rowH(int i) => i < 8 ? _rowH : _audioH;
double rowCy(int i) => rowTop(i) + rowH(i) / 2;
double tx(double t) => _x0 + 0.5 + _unit * t; // time unit -> x

List<RI> timeline() {
  final items = <RI>[
    Ln(344, 703, 1178, 1, H.rule), Ln(344, 993, 1178, 1, H.rule), Ln(344, 703, 1, 291, H.rule), Ln(1521, 703, 1, 291, H.rule),
    Rc(345, 704, 103, 38, fill: H.sel),
    Tx(370, 727, 'Timeline', H.s(13, w: FontWeight.w500, color: H.text), w: 54),
    Tx(469, 727, 'Graph', H.s(13, color: H.text2), w: 34),
    Tx(546, 727, 'Console', H.s(13, color: H.text2), w: 46),
    Ln(345, 742, 1176, 1, H.rule),
    Pt(_TlPaint()),
    Ln(558, 743, 1, 250, H.rule),
  ];
  // ruler numerals: the same technical-readout family as the other numeric text
  for (var i = 0; i <= 10; i++) {
    items.add(Tx(tx(i.toDouble()) + 3, 762, '00:${i.toString().padLeft(2, '0')}', H.m(11, color: const Color(0xFFB0B0B2)), w: 33));
  }
  // hierarchy grid: c0 = disclosure / state, c1 = glyph, c2 = label.
  // Rows 1-6 are parallel operations of the group: one vertical metric, no per-row offset.
  const c0 = 363.0, c1 = 393.0, c2 = 412.0;
  final ico = [null, H.neutralN, H.scatter.n, H.stagger.n, H.along.n, H.face.n, H.attach.n];
  final names = ['Jewel Field', 'Transform', 'Scatter', 'Stagger', 'Along Path', 'Rotation', 'Scale Variation', 'Camera', 'Audio'];
  final fit = {'Transform': 55.0, 'Scatter': 41.0, 'Camera': 43.0, 'Audio': 33.0};
  for (var i = 0; i < 9; i++) {
    final cy = rowCy(i);
    final base = cy + 4.5;
    if (i == 0) {
      items.add(Tx(386, base, names[i], H.s(12.5, w: FontWeight.w600, color: H.text), w: 68));
      continue;
    }
    if (i <= 6) {
      items.add(Rc(c1 - 8, cy - 8, 16, 16, fill: ico[i]!, r: 3));
      items.add(Hg(c1, cy, 10, HG.diamond, const Color(0xFFF2F2F2)));
    } else if (i == 7) {
      items.add(Hg(c1, cy, 15, HG.power, H.text2));
    } else {
      items.add(Hg(c0, cy + 0.5, 16, HG.headphones, H.text2));
      items.add(Hg(c1, cy + 0.5, 16, HG.lock, H.text2, bg: H.raisedHi));
    }
    items.add(Tx(c2, base, names[i], H.s(12.5, color: H.text2), w: fit[names[i]]));
  }
  items.add(Pt(_TlMarks()));
  return items;
}

class _TlMarks extends CustomPainter {
  @override
  void paint(Canvas cv, Size s) {
    final p = Paint()..color = H.text;
    // disclosure triangles sit on c0, optically centred on the row
    final g = rowCy(0);
    cv.drawPath(Path()..moveTo(358, g - 4)..lineTo(368.6, g - 4)..lineTo(363.3, g + 4)..close(), p);
    final c = rowCy(7);
    cv.drawPath(Path()..moveTo(359, c - 5.6)..lineTo(368, c + .1)..lineTo(359, c + 5.8)..close(), p);
  }
  @override
  bool shouldRepaint(_TlMarks o) => false;
}

class _TlPaint extends CustomPainter {
  @override
  void paint(Canvas cv, Size s) {
    // 1 panel ground is the window colour; 2 row ground alternates and runs continuously across both columns
    cv.drawRect(const Rect.fromLTRB(345, 743, 1521, 993), Paint()..color = H.window);
    cv.drawRect(const Rect.fromLTRB(_x0, 743, 1521, 774), Paint()..color = H.raised);
    cv.drawRect(const Rect.fromLTRB(345, 743, 558, 774), Paint()..color = H.raised);
    for (var i = 0; i < 9; i++) {
      final c = (i == 8) ? H.raised : (i.isEven ? H.raisedHi : H.raised);
      cv.drawRect(Rect.fromLTWH(345, rowTop(i), 1521 - 345, rowH(i)), Paint()..color = c);
    }
    cv.drawRect(const Rect.fromLTRB(345, 774, 1521, 775), Paint()..color = H.rule);
    // 3 time grid: major > minor > row gap. Major runs through the ruler as a tick.
    final major = Paint()..color = const Color(0x17FFFFFF)..strokeWidth = 1;
    final minor = Paint()..color = const Color(0x0AFFFFFF)..strokeWidth = 1;
    final tMaj = Paint()..color = const Color(0x47FFFFFF)..strokeWidth = 1;
    final tMin = Paint()..color = const Color(0x29FFFFFF)..strokeWidth = 1;
    for (var i = 0; i <= 10; i++) {
      final x = tx(i.toDouble());
      cv.drawLine(Offset(x, 766), Offset(x, 774), tMaj);
      cv.drawLine(Offset(x, 775), Offset(x, 993), major);
      for (var k = 1; k < 4; k++) {
        final xm = x + _unit * k / 4;
        cv.drawLine(Offset(xm, 770), Offset(xm, 774), tMin);
        cv.drawLine(Offset(xm, 775), Offset(xm, 993), minor);
      }
    }
    // 4 waveform: a quiet floor
    final cy = rowCy(8);
    cv.drawRRect(RRect.fromRectAndRadius(Rect.fromLTRB(594, cy - 11, 1490, cy + 11), const Radius.circular(2)), Paint()..color = const Color(0xFF1E2622));
    final wv = Paint()..color = const Color(0x803B6D5F)..strokeWidth = 1;
    var sd = 11;
    for (var x = 596.0; x < 1488; x += 2) {
      sd = (sd * 1103515245 + 12345) & 0x7fffffff;
      final env = 0.45 + 0.55 * math.sin((x - 594) / 896 * math.pi);
      final a = ((sd % 100) / 100 * 7 + 1.2) * env;
      cv.drawLine(Offset(x, cy - a), Offset(x, cy + a), wv);
    }
    // 5 temporal bodies: a flat body, one thin connector through the keys, small diamond keys on it.
    // One rule for every relation; a body with a single key has no connector.
    Color mixW(Color c, double t) => Color.lerp(c, const Color(0xFFFFFFFF), t)!;
    void node(double x, int r, Color body) {
      cv.save();
      cv.translate(x, rowCy(r));
      cv.rotate(math.pi / 4);
      final rr = Rect.fromCenter(center: Offset.zero, width: 4.8, height: 4.8);
      cv.drawRect(rr, Paint()..color = mixW(body, .32));
      cv.drawRect(rr, Paint()..color = mixW(body, .58)..style = PaintingStyle.stroke..strokeWidth = 1);
      cv.restore();
    }
    void bar(int r, double a, double b, Color c, {List<double> keys = const []}) {
      final top = rowTop(r) + (rowH(r) - _barH) / 2;
      final rc = Rect.fromLTRB(a, top, b, top + _barH);
      final clipped = a <= _x0;
      RRect rrect(Rect q, double rad) => RRect.fromRectAndCorners(q, topLeft: Radius.circular(clipped ? 0 : rad), bottomLeft: Radius.circular(clipped ? 0 : rad), topRight: Radius.circular(rad), bottomRight: Radius.circular(rad));
      cv.drawRRect(rrect(rc, 4), Paint()..color = c);
      cv.drawRRect(rrect(rc.deflate(.5), 3.6), Paint()..color = mixW(c, .13)..style = PaintingStyle.stroke..strokeWidth = 1);
      final ks = [...keys]..sort();
      if (ks.length > 1) cv.drawLine(Offset(ks.first, rowCy(r)), Offset(ks.last, rowCy(r)), Paint()..color = const Color(0x5C000000)..strokeWidth = 1);
      for (final k in ks) { node(k, r, c); }
    }
    bar(1, 625, 856, H.neutralT, keys: [632, 738, 850]);
    bar(2, 621, 1317, H.scatter.t, keys: [770, 988]);
    bar(3, 607, 1201, H.stagger.t, keys: [617, 778, 867, 1195]);
    bar(4, _x0, 1281, H.along.t, keys: [632, 816]);
    bar(5, 608, 928, H.face.t, keys: [617, 781, 906]);
    bar(6, 682, 1143, H.attach.t, keys: [690]);
    // a key with no body (Camera): the same diamond on the neutral row
    cv.save();
    cv.translate(706, rowCy(7));
    cv.rotate(math.pi / 4);
    final kr = Rect.fromCenter(center: Offset.zero, width: 4.8, height: 4.8);
    cv.drawRect(kr, Paint()..color = mixW(H.raisedHi, .25));
    cv.drawRect(kr, Paint()..color = mixW(H.raisedHi, .5)..style = PaintingStyle.stroke..strokeWidth = 1);
    cv.restore();
    // 6 playhead: ruler marker + thin line, above bodies and keys
    cv.drawRect(const Rect.fromLTWH(736.4, 748, 1.3, 245), Paint()..color = H.playhead);
    cv.drawPath(Path()..moveTo(731, 748)..lineTo(744, 748)..lineTo(744, 755)..lineTo(737.5, 762)..lineTo(731, 755)..close(), Paint()..color = H.playhead);
  }
  @override
  bool shouldRepaint(_TlPaint o) => false;
}
