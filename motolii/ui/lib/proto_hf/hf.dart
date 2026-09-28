import 'dart:io';
import 'dart:math' as math;
import 'package:flutter/widgets.dart';
import '../hf/glyphs.dart';
import 'hg_item.dart';
import 'ref.dart';
import '../hf/shell/top.dart' as shell_top show top;
import '../hf/shell/top.dart' show TopModel;
import 'stage.dart' as shell_stage show stage;
import 'stage.dart' show StageModel, StageBodyModel, stageBody;
import '../hf/shell/timeline.dart' as shell_timeline show timeline;
import '../hf/shell/timeline.dart' show TimelineModel, TlRow, TlKind, tlX0;

Color shade(Color c, double t) => Color.lerp(c, const Color(0xFF000000), t)!;
const _stageArt = String.fromEnvironment('PROTO_STAGE', defaultValue: 'lib/proto/stage_hf.png');

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
List<RI> stage() => shell_stage.stage(StageModel(body: stageBody(StageBodyModel(picture: Image.file(File(_stageArt), fit: BoxFit.fill, filterQuality: FilterQuality.medium)))));

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
// The face is hf/shell/timeline.dart; the fixture is the reference's own rows.
List<RI> timeline() {
  final wave = <double>[];
  var sd = 11;
  for (var x = 596.0; x < 1488; x += 2) {
    sd = (sd * 1103515245 + 12345) & 0x7fffffff;
    final env = 0.45 + 0.55 * math.sin((x - 594) / 896 * math.pi);
    wave.add(((sd % 100) / 100 * 7 + 1.2) * env);
  }
  return shell_timeline.timeline(TimelineModel(
    ruler: [for (var i = 0; i <= 10; i++) '00:${i.toString().padLeft(2, '0')}'],
    playhead: 736.75,
    rows: [
      const TlRow('Jewel Field', TlKind.group, nameW: 68, open: true),
      TlRow('Transform', TlKind.item, chip: H.neutralN, nameW: 55, body: (625, 856, H.neutralT), keys: const [632, 738, 850]),
      TlRow('Scatter', TlKind.item, chip: H.scatter.n, nameW: 41, body: (621, 1317, H.scatter.t), keys: const [770, 988]),
      TlRow('Stagger', TlKind.item, chip: H.stagger.n, body: (607, 1201, H.stagger.t), keys: const [617, 778, 867, 1195]),
      TlRow('Along Path', TlKind.item, chip: H.along.n, body: (tlX0, 1281, H.along.t), keys: const [632, 816]),
      TlRow('Rotation', TlKind.item, chip: H.face.n, body: (608, 928, H.face.t), keys: const [617, 781, 906]),
      TlRow('Scale Variation', TlKind.item, chip: H.attach.n, body: (682, 1143, H.attach.t), keys: const [690]),
      const TlRow('Camera', TlKind.camera, nameW: 43, open: false, keys: [706]),
      TlRow('Audio', TlKind.audio, nameW: 33, wave: wave),
    ],
  ));
}
