import 'dart:math' as math;

import 'package:flutter/material.dart';

import '../../foundation/metrics.dart';
import '../../session/editor_session.dart';
import 'shelf.dart';

/// Create: text, shapes, paths, helpers and the bundled 3D bodies; a
/// double-click adds one to the composition.
class CreateShelf extends BrowserShelf {
  @override
  String get name => 'Create';

  @override
  List<String> rails(BrowserHost host) => const [
    'All',
    'Text',
    'Shapes',
    'Paths',
    '3D',
    'Helpers',
  ];

  @override
  List<Map<String, dynamic>> items(BrowserHost host) => [
    {'id': 'text', 'name': 'Text', 'detail': 'Adds a text layer', 'glyph': 'T'},
    // 形は AE の shape ツールの並び。矩形だけ角丸を効果として最初から積む。
    for (final (id, name, glyph) in [
      ('rectangle', 'Rectangle', '■'),
      ('roundedRectangle', 'Rounded Rectangle', '▢'),
      ('ellipse', 'Ellipse', '●'),
      ('star', 'Star', '★'),
      ('polygon', 'Polygon', '⬟'),
    ])
      {'id': id, 'name': name, 'detail': 'Adds a shape layer', 'glyph': glyph},
    {
      'id': 'null',
      'name': 'Null',
      'detail': 'Adds an empty layer to parent others to',
      'glyph': '✛',
    },
    {'id': 'camera', 'name': 'Camera', 'detail': 'Adds a camera layer'},
    {
      'id': 'stage',
      'name': 'Stage',
      'detail': 'Widens the working area around the frame',
      'glyph': '⬚',
    },
    {
      'id': 'line',
      'name': 'Line',
      'detail': 'Adds a straight stroked path',
      'glyph': '─',
    },
    {
      'id': 'bezier',
      'name': 'Bezier',
      'detail': 'Adds a path layer',
      'glyph': '〜',
    },
    // 同梱の基本形は Rust の表そのまま。名前が線画を選ぶ(cube ⇒ 箱)。
    for (final p in EditorSession.maps(host.controller.state['primitives']))
      {...p, 'detail': 'Adds a 3D ${'${p['name']}'.toLowerCase()}'},
  ];

  @override
  String classification(BrowserHost host, Map<String, dynamic> item) =>
      switch (host.id(item)) {
        'text' => 'Text',
        'rectangle' ||
        'roundedRectangle' ||
        'ellipse' ||
        'star' ||
        'polygon' => 'Shapes',
        'bezier' || 'line' => 'Paths',
        'null' => 'Helpers',
        'camera' || 'stage' => '3D',
        _ => shapeOf(host.id(item)) != null ? '3D' : 'Other',
      };

  @override
  bool supported(BrowserHost host, Map<String, dynamic> item) =>
      host.has('create');

  @override
  Widget preview(BrowserHost host, Map<String, dynamic> item, Color identity) {
    final shape = shapeOf(host.id(item));
    return Center(
      child: host.id(item) == 'camera'
          ? Icon(
              Icons.videocam_outlined,
              size: EditorMetrics.bar,
              color: identity,
            )
          : shape != null
          ? SizedBox(
              width: EditorMetrics.bar,
              height: EditorMetrics.bar,
              child: CustomPaint(painter: ShapeMark(shape, identity)),
            )
          : Text(
              '${item['glyph'] ?? 'ƒ'}',
              style: TextStyle(fontSize: EditorMetrics.s23, color: identity),
            ),
    );
  }

  @override
  Future<void> apply(BrowserHost host, Map<String, dynamic> item) async {
    if (host.has('create'))
      await host.controller.command('create', {'kind': host.id(item)});
  }
}

/// The bodies a mesh file is usually named after. A file whose name says none
/// of them keeps the generic 3D icon.
enum Shape { sphere, torus, cube, cylinder, cone, pyramid, plane }

const _shapeWords = <String, Shape>{
  'sphere': Shape.sphere,
  'ball': Shape.sphere,
  'icosphere': Shape.sphere,
  'torus': Shape.torus,
  'donut': Shape.torus,
  'cube': Shape.cube,
  'box': Shape.cube,
  'cylinder': Shape.cylinder,
  'tube': Shape.cylinder,
  'cone': Shape.cone,
  'pyramid': Shape.pyramid,
  'tetra': Shape.pyramid,
  'plane': Shape.plane,
  'quad': Shape.plane,
};

Shape? shapeOf(String name) {
  final text = name.toLowerCase();
  for (final entry in _shapeWords.entries)
    if (text.contains(entry.key)) return entry.value;
  return null;
}

/// The outline of one body, drawn from the box it is given so the same mark
/// works at any tile size.
class ShapeMark extends CustomPainter {
  const ShapeMark(this.shape, this.color);
  final Shape shape;
  final Color color;

  @override
  void paint(Canvas canvas, Size size) {
    final stroke = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1
      ..color = color;
    final s = size.shortestSide;
    final c = Offset(size.width / 2, size.height / 2);
    Rect around(double rx, double ry, [double dy = 0]) => Rect.fromCenter(
      center: c.translate(0, dy * s),
      width: rx * s,
      height: ry * s,
    );
    switch (shape) {
      case Shape.sphere:
        canvas.drawCircle(c, s * .38, stroke);
        canvas.drawOval(around(.76, .3), stroke);
      case Shape.torus:
        canvas.drawOval(around(.86, .5), stroke);
        canvas.drawOval(around(.34, .2), stroke);
      case Shape.cube:
        final path = Path();
        for (var i = 0; i < 6; i++) {
          final a = math.pi / 6 + i * math.pi / 3;
          final p = c + Offset(math.cos(a), math.sin(a)) * s * .4;
          i == 0 ? path.moveTo(p.dx, p.dy) : path.lineTo(p.dx, p.dy);
        }
        canvas.drawPath(path..close(), stroke);
        for (final a in [math.pi / 2, math.pi * 7 / 6, math.pi * 11 / 6])
          canvas.drawLine(
            c,
            c + Offset(math.cos(a), math.sin(a)) * s * .4,
            stroke,
          );
      case Shape.cylinder:
        canvas.drawOval(around(.6, .22, -.28), stroke);
        canvas.drawOval(around(.6, .22, .28), stroke);
        for (final x in [-.3, .3])
          canvas.drawLine(
            c + Offset(x * s, -s * .28),
            c + Offset(x * s, s * .28),
            stroke,
          );
      case Shape.cone:
        canvas.drawOval(around(.7, .24, .3), stroke);
        for (final x in [-.35, .35])
          canvas.drawLine(
            c + Offset(0, -s * .38),
            c + Offset(x * s, s * .3),
            stroke,
          );
      case Shape.pyramid:
        final apex = c + Offset(0, -s * .38);
        final left = c + Offset(-s * .38, s * .3);
        final right = c + Offset(s * .38, s * .3);
        final back = c + Offset(s * .08, s * .12);
        canvas.drawPath(
          Path()
            ..moveTo(apex.dx, apex.dy)
            ..lineTo(left.dx, left.dy)
            ..lineTo(right.dx, right.dy)
            ..close(),
          stroke,
        );
        canvas.drawLine(apex, back, stroke);
        canvas.drawLine(left, back, stroke);
        canvas.drawLine(right, back, stroke);
      case Shape.plane:
        canvas.drawPath(
          Path()
            ..moveTo(c.dx - s * .2, c.dy - s * .22)
            ..lineTo(c.dx + s * .44, c.dy - s * .22)
            ..lineTo(c.dx + s * .2, c.dy + s * .22)
            ..lineTo(c.dx - s * .44, c.dy + s * .22)
            ..close(),
          stroke,
        );
    }
  }

  @override
  bool shouldRepaint(ShapeMark old) => old.shape != shape || old.color != color;
}
