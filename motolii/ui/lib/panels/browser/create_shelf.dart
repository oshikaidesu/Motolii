import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../../theme/editor_metrics.dart';
import '../../session/editor_session.dart';
import 'shelf.dart';
import '../../theme/material_icons.dart';

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
    'Copies',
  ];

  /// What can be created is the host's table (status `createKinds`); this only adds the mark a kind is drawn with.
  static const _glyphs = {
    'text': 'T',
    'rectangle': '■',
    'roundedRectangle': '▢',
    'ellipse': '●',
    'star': '★',
    'polygon': '⬟',
    'null': '✛',
    'particles': '⁘',
    'stage': '⬚',
    'line': '─',
    'bezier': '〜',
  };

  @override
  List<Map<String, dynamic>> items(BrowserHost host) => [
    for (final kind in EditorSession.maps(host.controller.state['createKinds']))
      {...kind, if (_glyphs['${kind['id']}'] != null) 'glyph': _glyphs['${kind['id']}']},
    // What the host adds to a layer rather than makes: applied the way it always was, stored where it always was.
    for (final capability in EditorSession.maps(host.controller.state['hostCapabilities']))
      {...capability, 'glyph': capability['id'] == 'motolii.mirror' ? '⧓' : '⁙'},
  ];

  bool _applied(Map<String, dynamic> item) => item['apply'] == 'applyEffect';

  @override
  String classification(BrowserHost host, Map<String, dynamic> item) =>
      '${item['rail'] ?? 'Other'}';

  @override
  bool supported(BrowserHost host, Map<String, dynamic> item) => _applied(item)
      ? host.has('applyEffect') && host.controller.selectedIds.isNotEmpty
      : host.has('create');

  @override
  Widget preview(BrowserHost host, Map<String, dynamic> item, Color identity) {
    final shape = shapeOf(host.id(item));
    return Center(
      child: host.id(item) == 'camera'
          ? Icon(
              Glyph.videocam_outlined,
              size: EditorMetrics.mark,
              color: identity,
            )
          : shape != null
          ? SizedBox(
              width: EditorMetrics.mark,
              height: EditorMetrics.mark,
              child: CustomPaint(painter: ShapeMark(shape, identity)),
            )
          : Text(
              '${item['glyph'] ?? 'ƒ'}',
              style: TextStyle(
                fontSize: EditorMetrics.mark,
                height: 1,
                fontWeight: FontWeight.w500,
                color: identity,
              ),
            ),
    );
  }

  @override
  Future<void> apply(BrowserHost host, Map<String, dynamic> item) async {
    if (_applied(item)) {
      if (host.has('applyEffect')) await host.controller.command('applyEffect', {'pluginIds': [item['id']]});
      return;
    }
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
      ..strokeWidth = 2
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
