// Paper: one type scale (four roles) and one row rhythm (three heights) applied across the Timeline, Inspector, Browser and
// top bar, drawn at real pixels, to see whether they read as one tool at a 75 % density. Icons are stand-ins.
import 'package:flutter/widgets.dart';
import 'package:motolii_ui/theme/neutral.dart';

// ---- the roles (px at the new 100 %) and the rhythm
const rowStd = 20.0, rowTight = 18.0, rowHead = 24.0;

TextStyle _inter(double s, Color c, FontWeight w, double ls) => TextStyle(fontFamily: 'Inter', fontSize: s, color: c, fontWeight: w, letterSpacing: ls, height: 1, decoration: TextDecoration.none);
TextStyle name([Color c = N.g91]) => _inter(11, c, FontWeight.w500, .05);
TextStyle value([Color c = N.g91]) => TextStyle(fontFamily: 'Menlo', fontSize: 11, color: c, height: 1, decoration: TextDecoration.none);
TextStyle label([Color c = N.g69]) => _inter(10, c, FontWeight.w400, .12);
TextStyle micro([Color c = N.g76]) => _inter(9.5, c, FontWeight.w500, .15);

Widget _dot(Color c, [double d = 8]) => Container(width: d, height: d, decoration: BoxDecoration(color: c, borderRadius: BorderRadius.circular(2)));

Widget _panel(String title, double w, Widget body, {double? h}) => Container(
      width: w,
      height: h,
      color: N.g10,
      child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        Container(height: rowHead, padding: const EdgeInsets.symmetric(horizontal: 8), alignment: Alignment.centerLeft, decoration: const BoxDecoration(border: Border(bottom: BorderSide(color: N.g20, width: .5))), child: Text(title, style: name())),
        Expanded(child: body),
      ]),
    );

Widget timeline(double w) {
  const rows = [('Camera', N.g44, .0, .0), ('Text', Color(0xFF7BCBA3), .0, .55), ('Rounded Rectangle', Color(0xFF5596E9), .0, .55), ('Night sky', Color(0xFFA282E8), .0, .55)];
  return _panel('Timeline    Graph    Console', w, Column(children: [
    Container(height: rowTight, padding: const EdgeInsets.only(left: 160), alignment: Alignment.centerLeft, child: Row(children: [for (final t in const ['00:00', '00:01', '00:02', '00:03', '00:04', '00:05', '00:06']) SizedBox(width: (w - 170) / 7, child: Text(t, style: micro(N.g56)))])),
    for (final r in rows)
      SizedBox(height: rowTight, child: Row(children: [
        const SizedBox(width: 8),
        _dot(r.$2),
        const SizedBox(width: 6),
        SizedBox(width: 108, child: Text(r.$1, maxLines: 1, style: name())),
        SizedBox(width: 30, child: Row(children: [_dot(N.g38, 5), const SizedBox(width: 3), _dot(N.g38, 5), const SizedBox(width: 3), _dot(N.g38, 5)])),
        Expanded(child: Container(height: 12, margin: const EdgeInsets.only(right: 60), decoration: BoxDecoration(color: r.$2.withValues(alpha: .85), borderRadius: BorderRadius.circular(2)))),
      ])),
  ]), h: rowHead + rowTight * 5 + 6);
}

Widget _field(String v, double w) => Container(width: w, height: rowStd - 2, padding: const EdgeInsets.symmetric(horizontal: 6), alignment: Alignment.centerLeft, decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(3)), child: Text(v, style: value()));

Widget inspector(double w) => _panel('Night sky', w, Padding(
      padding: const EdgeInsets.fromLTRB(8, 6, 8, 6),
      child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
        Text('TRANSFORM', style: micro(N.g56)),
        const SizedBox(height: 4),
        for (final r in const [('Position', '960.00', '540.00', '0.000'), ('Scale', '100', '100', '100'), ('Rotation', '0', '0', '0')])
          SizedBox(height: rowStd, child: Row(children: [
            SizedBox(width: 58, child: Text(r.$1, style: label())),
            _field(r.$2, 62), const SizedBox(width: 4), _field(r.$3, 62), const SizedBox(width: 4), _field(r.$4, 62),
          ])),
        SizedBox(height: rowStd, child: Row(children: [SizedBox(width: 58, child: Text('Opacity', style: label())), _field('100', 190), const SizedBox(width: 4), Text('%', style: label())])),
        const SizedBox(height: 6),
        Text('WORLD', style: micro(N.g56)),
        const SizedBox(height: 4),
        SizedBox(height: rowStd, child: Row(children: [SizedBox(width: 58, child: Text('Blend', style: label())), Text('Normal', style: name())])),
        SizedBox(height: rowStd, child: Row(children: [Text('Clip to below', style: label()), const SizedBox(width: 10), Text('Environment', style: micro(N.g100))])),
      ]),
    ), h: rowHead + 6 * 2 + 4 * 2 + rowStd * 7 + 22);

Widget browser(double w) {
  const cells = [('Text', N.g76), ('Rectangle', Color(0xFF5596E9)), ('Rounded', Color(0xFFF27AB6)), ('Ellipse', Color(0xFFF27AB6)), ('Star', Color(0xFFF0D455)), ('Polygon', Color(0xFFA282E8)), ('Line', N.g76), ('Arrow', N.g63)];
  return _panel('All   Text   Shapes   Paths   Tools', w, Padding(
    padding: const EdgeInsets.all(8),
    child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
      Text('PRIMITIVES', style: micro(N.g56)),
      const SizedBox(height: 4),
      Wrap(spacing: 4, runSpacing: 4, children: [
        for (final c in cells)
          Container(width: (w - 16 - 12) / 4, height: 44, decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(4)), child: Column(mainAxisAlignment: MainAxisAlignment.center, children: [_dot(c.$2, 14), const SizedBox(height: 5), Text(c.$1, style: label(N.g82))])),
      ]),
    ]),
  ), h: rowHead + 8 * 2 + 12 + 4 + 44 * 2 + 4 + 4);
}

Widget topbar(double w) => Container(
      width: w,
      height: 30,
      color: N.g10,
      padding: const EdgeInsets.symmetric(horizontal: 8),
      child: Row(children: [
        Text('Motolii', style: _inter(12, N.g95, FontWeight.w600, 0)),
        const SizedBox(width: 14),
        for (final c in const [Color(0xFF7BCC9E), N.g33, Color(0xFFF03C8A)]) Padding(padding: const EdgeInsets.only(right: 6), child: _dot(c, 14)),
        const SizedBox(width: 8),
        Text('30.00', style: value()),
        const SizedBox(width: 10),
        Text('1800', style: value(N.g63)),
        const SizedBox(width: 10),
        Text('00:00:00', style: value()),
        const Spacer(),
        for (final t in const ['EDIT', 'PLAY', 'EXPORT'])
          Container(margin: const EdgeInsets.only(left: 4), padding: const EdgeInsets.symmetric(horizontal: 8), height: 20, alignment: Alignment.center, decoration: BoxDecoration(color: t == 'EDIT' ? const Color(0xFF6982D1) : null, borderRadius: BorderRadius.circular(3)), child: Text(t, style: micro(t == 'EDIT' ? N.g100 : N.g63))),
      ]),
    );

/// The whole sheet: the four surfaces at real pixels, one rhythm.
Widget densitySpecimen() => ColoredBox(
      color: N.g00,
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
          topbar(730),
          const SizedBox(height: 10),
          Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
            timeline(420),
            const SizedBox(width: 10),
            inspector(300),
          ]),
          const SizedBox(height: 10),
          browser(270),
        ]),
      ),
    );
