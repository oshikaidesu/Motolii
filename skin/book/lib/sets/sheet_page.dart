// A sheet to judge the shapes on, before they go into the window. One figure is large. The rest is type and a rule.
import 'dart:io';
import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import '../theme.dart';
import '../tokens.dart';
import 'collage_cut.dart';

const _paper = Color(0xFFF3F1EC);
const _ink = Color(0xFF1A1A1A);
const _quiet = Color(0xFF8A8478);
const _red = Color(0xFFD23C2E);
const _green = Color(0xFF1F6B4A);
const _blue = Color(0xFF2A4F8F);
const _rule = Color(0xFFD4CEC3);

WidgetbookComponent sheetSet() => WidgetbookComponent(
  name: 'Sheet',
  useCases: [WidgetbookUseCase(name: 'Paper', builder: (_) => const CollageCut())],
);

class SheetPaper extends StatefulWidget {
  const SheetPaper({super.key});

  @override
  State<SheetPaper> createState() => _SheetPaperState();
}

class _SheetPaperState extends State<SheetPaper> {
  static const _zoom = ['Fit', '50%', '100%'];
  static const _flags = ['Snap', 'Safe', 'Grid', 'Camera'];
  static const _eases = ['Linear', 'Ease', 'In', 'Out', 'In-out', 'Back', 'Hold', 'Bounce', 'Elastic', 'Cyclic', 'Random', 'Steps', 'ElasticSteps', 'Snappy'];
  static const _tracks = [(1, 'swinging.json'), (2, 'Camera'), (7, 'Cube')];

  var fit = 0;
  var ease = 0;
  var track = 2;
  var red = 'fit';
  final on = <String>{'Snap'};

  @override
  void initState() {
    super.initState();
    // The book wears whatever skin the window last saved. This page is the sheet, so it reads that file itself.
    final path = '${Directory.current.path}/skins/sheet/theme.xml';
    ColorTheme.load(File(path).existsSync() ? path : '/Users/motolii_maker/motolii/skin/book/skins/sheet/theme.xml');
  }

  TextStyle _sans(double size, Color color, {FontWeight weight = FontWeight.w500}) => TextStyle(
    fontFamily: T.sans,
    fontSize: size,
    height: 1,
    letterSpacing: size > 28 ? -1.4 : 0,
    fontWeight: weight,
    color: color,
    decoration: TextDecoration.none,
  );

  TextStyle _mono(double size, Color color) => TextStyle(fontFamily: T.mono, fontSize: size, height: 1, fontWeight: FontWeight.w400, color: color, decoration: TextDecoration.none);

  void _pickFit(int i) => setState(() {
    fit = i;
    red = 'fit';
  });

  @override
  Widget build(BuildContext context) {
    final small = [for (var i = 0; i < _zoom.length; i++) if (i != fit) i];
    return ColoredBox(
      color: _paper,
      child: SingleChildScrollView(
        padding: const EdgeInsets.fromLTRB(64, 72, 80, 88),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              crossAxisAlignment: CrossAxisAlignment.end,
              children: [
                GestureDetector(
                  onTap: () => _pickFit(0),
                  child: CustomPaint(size: const Size(168, 168), painter: _Quarter(red == 'fit' ? _red : _ink)),
                ),
                const SizedBox(width: 28),
                Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      children: [
                        for (final i in small) ...[
                          _press(_zoom[i], () => _pickFit(i), _sans(16, _quiet)),
                          const SizedBox(width: 18),
                        ],
                      ],
                    ),
                    const SizedBox(height: 14),
                    _press(_zoom[fit], () => _pickFit(fit), _sans(64, _ink, weight: FontWeight.w700)),
                  ],
                ),
              ],
            ),
            const SizedBox(height: 72),
            Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                for (final name in _flags) ...[
                  _flag(name),
                  const SizedBox(width: 28),
                ],
              ],
            ),
            const SizedBox(height: 80),
            Row(
              crossAxisAlignment: CrossAxisAlignment.end,
              children: [
                _num('X', '-66216.9', _red),
                const SizedBox(width: 48),
                _num('Y', '540.0', _green),
                const SizedBox(width: 48),
                _num('Z', '0.0', _blue),
              ],
            ),
            const SizedBox(height: 72),
            SizedBox(
              width: 560,
              child: Wrap(
                spacing: 22,
                runSpacing: 16,
                crossAxisAlignment: WrapCrossAlignment.center,
                children: [
                  for (var i = 0; i < _eases.length; i++)
                    if (i == ease)
                      Row(
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          CustomPaint(size: const Size(22, 22), painter: _Quarter(red == 'ease' ? _red : _ink)),
                          const SizedBox(width: 8),
                          _press(_eases[i], () => setState(() { ease = i; red = 'ease'; }), _sans(22, _ink, weight: FontWeight.w700)),
                        ],
                      )
                    else
                      _press(_eases[i], () => setState(() { ease = i; red = 'ease'; }), _sans(15, _quiet)),
                ],
              ),
            ),
            const SizedBox(height: 80),
            for (var i = 0; i < _tracks.length; i++) ...[
              _track(i),
              if (i != _tracks.length - 1) const SizedBox(height: 18),
            ],
          ],
        ),
      ),
    );
  }

  Widget _press(String word, VoidCallback onTap, TextStyle style) => MouseRegion(
    cursor: SystemMouseCursors.click,
    child: GestureDetector(onTap: onTap, child: Text(word, style: style)),
  );

  Widget _flag(String name) {
    final lit = on.contains(name);
    final color = lit && red == name ? _red : _ink;
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      child: GestureDetector(
        onTap: () => setState(() {
          if (!on.add(name)) on.remove(name);
          red = on.contains(name) ? name : (red == name ? 'fit' : red);
        }),
        child: Column(
          children: [
            CustomPaint(size: const Size(56, 56), painter: _Disc(color, filled: lit)),
            const SizedBox(height: 10),
            Text(name, style: _sans(12, lit ? _ink : _quiet, weight: lit ? FontWeight.w600 : FontWeight.w400)),
          ],
        ),
      ),
    );
  }

  Widget _num(String axis, String value, Color color) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      Text(axis, style: _sans(13, color, weight: FontWeight.w700)),
      const SizedBox(height: 8),
      Text(value, style: _mono(32, _ink)),
    ],
  );

  Widget _track(int i) {
    final (n, name) = _tracks[i];
    final picked = track == i;
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      child: GestureDetector(
        onTap: () => setState(() {
          track = i;
          red = 'track';
        }),
        child: SizedBox(
          width: 460,
          height: 28,
          child: Row(
            children: [
              SizedBox(width: 22, child: Text('$n', style: _mono(13, _quiet))),
              const SizedBox(width: 12),
              SizedBox(width: 150, child: Text(name, style: _sans(15, picked ? _ink : _quiet, weight: picked ? FontWeight.w600 : FontWeight.w400))),
              Expanded(
                child: picked
                    ? ColoredBox(color: red == 'track' ? _red : _ink, child: const SizedBox(height: 16))
                    : const ColoredBox(color: _rule, child: SizedBox(height: 1)),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _Quarter extends CustomPainter {
  const _Quarter(this.color);
  final Color color;

  @override
  void paint(Canvas c, Size s) {
    c.drawArc(Rect.fromCircle(center: Offset(0, s.height), radius: s.shortestSide), -math.pi / 2, math.pi / 2, true, Paint()..color = color);
  }

  @override
  bool shouldRepaint(_Quarter o) => o.color != color;
}

class _Disc extends CustomPainter {
  const _Disc(this.color, {required this.filled});
  final Color color;
  final bool filled;

  @override
  void paint(Canvas c, Size s) {
    final p = Paint()..color = color;
    if (!filled) {
      p
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.5;
    }
    c.drawCircle(s.center(Offset.zero), s.shortestSide * (filled ? .46 : .42), p);
  }

  @override
  bool shouldRepaint(_Disc o) => o.color != color || o.filled != filled;
}
