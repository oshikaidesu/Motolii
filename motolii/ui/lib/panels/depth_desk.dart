import 'dart:math' as math;

import 'package:flutter/material.dart';

import '../session/editor_session.dart';
import '../foundation/theme.dart';
import '../foundation/metrics.dart';

class DepthDesk extends StatefulWidget {
  const DepthDesk({super.key, required this.controller});
  final EditorSession controller;
  @override
  State<DepthDesk> createState() => _DepthDeskState();
}

class _DepthDeskState extends State<DepthDesk> {
  EditorSession get c => widget.controller;

  /// 目盛りは中身から。注視点を原点に、カメラと全層が常に外の輪の内側へ入る。
  static double _fit(Iterable<List> points) => points.fold<double>(
    600,
    (v, p) => math.max(v, math.max(p[0].abs(), p[1].abs()) * 1.25),
  );

  Map<String, dynamic>? _grab;
  Offset? _start;
  Future<void> _tail = Future.value();
  List<Map<String, dynamic>>? _pending;
  bool _sending = false, _finishing = false;
  Future<void> _drain() async {
    if (_sending) return;
    _sending = true;
    try {
      while (_pending != null) {
        final edits = _pending!;
        _pending = null;
        await c.command('previewProperties', {'edits': edits});
      }
    } finally {
      _sending = false;
    }
  }

  Future<void> _finish(bool cancel) async {
    if (_grab == null) return;
    _grab = null;
    _finishing = true;
    if (cancel) _pending = null;
    await _tail;
    await c.command(cancel ? 'cancelPreview' : 'commitPreview');
    _finishing = false;
  }

  @override
  void dispose() {
    _finish(true);
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => AnimatedBuilder(
    animation: c.slice('depth', const [
      'depthLayout',
      'selectedId',
      'selectedIds',
      'capabilities',
    ]),
    builder: (context, _) {
      final data = EditorSession.map(c.state['depthLayout']);
      final items = EditorSession.maps(data['items']);
      final camera = EditorSession.map(data['camera']);
      final cameraPoint = camera['point'] is List
          ? camera['point'] as List
          : const [0.0, -1000.0];
      final target = camera['target'] is num
          ? c.layers.cast<Map<String, dynamic>?>().firstWhere(
              (l) => l?['id'] == camera['target'],
              orElse: () => null,
            )
          : null;
      final range = _fit([
        for (final i in items) i['point'] as List,
        cameraPoint,
      ]);
      return Column(
        children: [
          SizedBox(
            height: EditorMetrics.control,
            child: Row(
              children: [
                const SizedBox(width: EditorMetrics.s6),
                const Icon(
                  Icons.videocam_outlined,
                  size: EditorMetrics.s15,
                  color: EditorTheme.muted,
                ),
                if (target != null) ...[
                  const SizedBox(width: EditorMetrics.s6),
                  const Icon(
                    Icons.gps_fixed,
                    size: EditorMetrics.s15,
                    color: EditorTheme.muted,
                  ),
                  const SizedBox(width: EditorMetrics.s4),
                  Text(
                    '${target['name']}',
                    style: const TextStyle(
                      fontSize: EditorMetrics.dense,
                      color: EditorTheme.muted,
                    ),
                  ),
                ],
                const Spacer(),
              ],
            ),
          ),
          Expanded(
            child: LayoutBuilder(
              builder: (context, box) {
                final size = box.biggest;
                final origin = Offset(size.width / 2, size.height / 2);
                final scale =
                    math.max(1, math.min(size.width, size.height) - 36) /
                    (2 * range);
                Offset place(List p) =>
                    origin +
                    Offset(
                          (p[0] as num).toDouble(),
                          -(p[1] as num).toDouble(),
                        ) *
                        scale;
                Offset point(Map<String, dynamic> i) =>
                    place(i['point'] as List);
                final eye = place(cameraPoint);
                return ClipRect(
                  child: Listener(
                    behavior: HitTestBehavior.opaque,
                    onPointerDown: (e) {
                      if (_finishing || _grab != null) return;
                      if (camera['layer'] is num &&
                          (eye - e.localPosition).distance < 13) {
                        c.command('select', {
                          'ids': [camera['layer']],
                        });
                        _grab = {'camera': camera};
                        _start = e.localPosition;
                        return;
                      }
                      Map<String, dynamic>? hit;
                      for (final i in items.reversed) {
                        if ((point(i) - e.localPosition).distance < 13) {
                          hit = i;
                          break;
                        }
                      }
                      if (hit == null) {
                        c.command('select', {'ids': []});
                        return;
                      }
                      c.command('select', {
                        'ids': [hit['id']],
                      });
                      if (hit['locked'] == true) return;
                      _grab = Map.of(hit);
                      _start = e.localPosition;
                    },
                    onPointerMove: (e) {
                      final grab = _grab;
                      if (grab == null) return;
                      if (grab['camera'] is Map) {
                        // eye を注視点のまわりで動かす: 向きが yaw、平面距離 ÷ cos(pitch) が距離。
                        final cam = grab['camera'] as Map;
                        final orbit = cam['orbit'] as List;
                        final pitch = (orbit[0] as num).toDouble();
                        final p = (e.localPosition - origin) / scale;
                        final x = p.dx, z = -p.dy;
                        final yaw = math.atan2(-x, -z) * 180 / math.pi;
                        final flat = math.sqrt(x * x + z * z);
                        final cos = math.max(
                          0.05,
                          math.cos(pitch * math.pi / 180).abs(),
                        );
                        final base = (cam['baseDistance'] as num).toDouble();
                        _pending = [
                          {
                            'layer': cam['layer'],
                            'property': 'camera.orbit',
                            'value': [pitch, yaw],
                          },
                          {
                            'layer': cam['layer'],
                            'property': 'camera.distance',
                            'value': (flat / cos / base).clamp(0.01, 100),
                          },
                        ];
                        if (!_sending) _tail = _drain();
                        return;
                      }
                      final delta = (e.localPosition - _start!) / scale;
                      final x = grab['inverseX'] as List,
                          z = grab['inverseZ'] as List,
                          local = grab['local'] as List;
                      final next = List.generate(
                        3,
                        (i) =>
                            (local[i] as num).toDouble() +
                            (x[i] as num) * delta.dx -
                            (z[i] as num) * delta.dy,
                      );
                      _pending = [
                        {
                          'layer': grab['id'],
                          'property': 'position',
                          'value': [next[0], next[1]],
                        },
                        {
                          'layer': grab['id'],
                          'property': 'position.z',
                          'value': next[2],
                        },
                      ];
                      if (!_sending) _tail = _drain();
                    },
                    onPointerUp: (_) => _finish(false),
                    onPointerCancel: (_) => _finish(true),
                    child: Stack(
                      children: [
                        Positioned.fill(
                          child: CustomPaint(
                            painter: _DepthGrid(
                              origin,
                              eye,
                              scale,
                              range,
                              (data['halfFov'] as num? ?? .9).toDouble(),
                            ),
                          ),
                        ),
                        for (final item in items)
                          Positioned(
                            left: point(item).dx - EditorMetrics.micro,
                            top: point(item).dy - EditorMetrics.micro,
                            child: EditorTooltip(
                              message: '${item['name']}',
                              child: Container(
                                width: EditorMetrics.s18,
                                height: EditorMetrics.s18,
                                decoration: BoxDecoration(
                                  color: EditorTheme.layerColor(item['id']),
                                  shape: BoxShape.circle,
                                  border: Border.all(
                                    color: c.selectedIds.contains(item['id'])
                                        ? EditorTheme.ink
                                        : EditorTheme.line,
                                    width: c.selectedIds.contains(item['id'])
                                        ? EditorMetrics.s2
                                        : 1,
                                  ),
                                ),
                                child: Center(
                                  child: Icon(
                                    Icons.circle,
                                    size: EditorMetrics.s4,
                                    color: c.selectedIds.contains(item['id'])
                                        ? EditorTheme.ink
                                        : EditorTheme.line,
                                  ),
                                ),
                              ),
                            ),
                          ),
                        for (final item in items.where(
                          (item) => c.selectedIds.contains(item['id']),
                        ))
                          Positioned(
                            left: point(item).dx + EditorMetrics.s12,
                            top: point(item).dy - EditorMetrics.s7,
                            child: IgnorePointer(
                              child: SizedBox(
                                width: EditorMetrics.s76,
                                child: Text(
                                  '${item['name']}',
                                  maxLines: 1,
                                  overflow: TextOverflow.ellipsis,
                                  style: const TextStyle(
                                    fontSize: EditorMetrics.dense,
                                    color: EditorTheme.ink,
                                  ),
                                ),
                              ),
                            ),
                          ),
                      ],
                    ),
                  ),
                );
              },
            ),
          ),
        ],
      );
    },
  );
}

class _DepthGrid extends CustomPainter {
  _DepthGrid(this.origin, this.eye, this.scale, this.range, this.fov);

  /// 原点は注視点、eye はカメラ。frustum は eye から注視点へ向く。
  final Offset origin, eye;
  final double scale, range, fov;
  @override
  void paint(Canvas canvas, Size size) {
    canvas.drawRect(Offset.zero & size, Paint()..color = EditorTheme.panel);
    final line = Paint()
      ..color = EditorTheme.line
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1;
    for (final fraction in [.25, .5, .75, 1.0])
      canvas.drawCircle(origin, range * scale * fraction, line);
    canvas.drawLine(Offset(origin.dx, 0), Offset(origin.dx, size.height), line);
    canvas.drawLine(Offset(0, origin.dy), Offset(size.width, origin.dy), line);
    final view = Paint()
      ..color = const Color(0xff8ed9e6)
      ..strokeWidth = 1;
    final toTarget = origin - eye;
    final dir = toTarget.distance > 0
        ? toTarget / toTarget.distance
        : const Offset(0, -1);
    final side = Offset(-dir.dy, dir.dx);
    for (final sign in [-1, 1])
      canvas.drawLine(
        eye,
        eye + (dir + side * (sign * fov)) * range * scale,
        view,
      );
    canvas.save();
    canvas.translate(eye.dx, eye.dy);
    canvas.rotate(math.atan2(dir.dy, dir.dx) + math.pi / 2);
    canvas.drawRect(
      Rect.fromCenter(
        center: Offset.zero,
        width: EditorMetrics.s14,
        height: EditorMetrics.dense,
      ),
      Paint()..color = const Color(0xff8ed9e6),
    );
    final arrow = Path()
      ..moveTo(-4, -7)
      ..lineTo(0, -12)
      ..lineTo(4, -7)
      ..close();
    canvas.drawPath(arrow, view);
    canvas.restore();
    canvas.drawCircle(
      origin,
      EditorMetrics.s4,
      view..style = PaintingStyle.stroke,
    );
  }

  @override
  bool shouldRepaint(_DepthGrid old) =>
      origin != old.origin ||
      eye != old.eye ||
      scale != old.scale ||
      range != old.range ||
      fov != old.fov;
}
