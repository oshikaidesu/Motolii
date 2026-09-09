import 'dart:math' as math;

import 'package:flutter/material.dart';

import '../session/editor_session.dart';
import '../foundation/theme.dart';
import '../foundation/panel_controls.dart';
import '../foundation/metrics.dart';

class DepthDesk extends StatefulWidget {
  const DepthDesk({super.key, required this.controller});
  final EditorSession controller;
  @override
  State<DepthDesk> createState() => _DepthDeskState();
}

class _DepthDeskState extends State<DepthDesk> {
  EditorSession get c => widget.controller;
  double get _range =>
      (c.deskWork.value['depthRange'] as num? ?? 2400).toDouble();
  set _range(double value) {
    c.storeDesk('depthRange', value);
  }

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
                const Spacer(),
                panelButton(
                  '−',
                  () => setState(
                    () => _range = (_range * 1.3).clamp(100, 100000),
                  ),
                ),
                panelButton(
                  '+',
                  () => setState(
                    () => _range = (_range / 1.3).clamp(100, 100000),
                  ),
                ),
                panelButton(
                  'Fit',
                  () => setState(
                    () => _range = items.fold<double>(
                      1200,
                      (v, i) => math.max(
                        v,
                        math.max(
                              (i['point'][0] as num).abs(),
                              (i['point'][1] as num).abs(),
                            ) *
                            1.3,
                      ),
                    ),
                  ),
                ),
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
                    (2 * _range);
                Offset point(Map<String, dynamic> i) =>
                    origin +
                    Offset(
                          (i['point'][0] as num).toDouble(),
                          -(i['point'][1] as num).toDouble(),
                        ) *
                        scale;
                return ClipRect(
                  child: Listener(
                    behavior: HitTestBehavior.opaque,
                    onPointerDown: (e) {
                      if (_finishing || _grab != null) return;
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
                              scale,
                              _range,
                              (data['halfFov'] as num? ?? .9).toDouble(),
                            ),
                          ),
                        ),
                        for (final item in items)
                          Positioned(
                            left: point(item).dx - EditorMetrics.micro,
                            top: point(item).dy - EditorMetrics.micro,
                            child: Tooltip(
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
  _DepthGrid(this.origin, this.scale, this.range, this.fov);
  final Offset origin;
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
    for (final sign in [-1, 1])
      canvas.drawLine(
        origin,
        origin + Offset(sign * range * fov, -range) * scale,
        view,
      );
    canvas.drawRect(
      Rect.fromCenter(
        center: origin,
        width: EditorMetrics.s14,
        height: EditorMetrics.dense,
      ),
      Paint()..color = const Color(0xff8ed9e6),
    );
    final arrow = Path()
      ..moveTo(origin.dx - 4, origin.dy - 7)
      ..lineTo(origin.dx, origin.dy - 12)
      ..lineTo(origin.dx + 4, origin.dy - 7)
      ..close();
    canvas.drawPath(arrow, view);
  }

  @override
  bool shouldRepaint(covariant _DepthGrid old) =>
      origin != old.origin ||
      scale != old.scale ||
      fov != old.fov ||
      range != old.range;
}
