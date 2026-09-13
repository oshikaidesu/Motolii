import 'dart:io';
import 'dart:math' as math;
import 'dart:typed_data';
import 'dart:ui' as ui show instantiateImageCodec, ImageByteFormat;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../../foundation/metrics.dart';
import '../../foundation/theme.dart';
import '../../session/editor_session.dart';
import '../native_visual_sample.dart';
import '../../foundation/color_field.dart';
import '../../foundation/panel_controls.dart';
import 'color_wheel.dart';
import 'parts.dart';
import 'shelf.dart';

/// Colours: the wheel edits the selected layer's slot; the tiles are the
/// palette in use, saved swatches and starters, and a click applies one.
class ColorsShelf extends BrowserShelf {
  @override
  String get name => 'Colors';

  /// Colours stacked beside the wheel; one is a solid, more make a gradient.
  List<List<double>> stops = [];

  /// The wheel's colour while nothing is focused: what the stops are made of.
  List<double>? picked;

  /// The wheel's size is the picker's height; the grip under the picker
  /// drags it.
  static const double wheelDefault = 160;
  double? wheelDrag;
  double wheelSize(EditorSession c) =>
      (wheelDrag ??
              (c.deskWork.value['browserWheel'] as num? ?? wheelDefault)
                  .toDouble())
          .clamp(EditorMetrics.thumb, EditorMetrics.s200);

  @override
  bool get bare => true;
  @override
  bool get showViews => false;
  @override
  List<String> get deskKeys => const ['browserWheel', 'colorShape', 'swatches'];

  @override
  List<Object?> derived(EditorSession c) {
    final target = _colorTarget(c);
    // The fill being edited: its kind, stops and direction redraw the editor
    // at the top of the panel; the layer's name titles it.
    return [target, _targetLayer(c, target)?['name']];
  }

  @override
  List<String> rails(BrowserHost host) => const [
    'All',
    'Saved',
    'Used here',
    'Starter',
  ];

  @override
  List<Map<String, dynamic>> items(BrowserHost host) => [
    ...EditorSession.maps(host.controller.state['palette']),
    for (final (i, s) in saved(host.controller).indexed)
      {...s, 'id': 'saved:$i', 'saved': true},
  ];

  @override
  String classification(BrowserHost host, Map<String, dynamic> item) =>
      item['saved'] == true
      ? 'Saved'
      : item['used'] == true
      ? 'Used here'
      : 'Starter';

  @override
  ShelfLayout layout(BrowserHost host, double width, double tile) =>
      ShelfLayout(
        column: tile * .6,
        extent: tile * .55,
        gap: EditorMetrics.s4,
        padding: EditorMetrics.s6,
        ground: EditorTheme.panel,
      );

  @override
  bool supported(BrowserHost host, Map<String, dynamic> item) =>
      (stopsOf(item).length > 1
          ? host.has('setGradient')
          : host.has('applyPalette')) &&
      host.controller.selectedIds.isNotEmpty;

  @override
  Widget preview(BrowserHost host, Map<String, dynamic> item, Color identity) =>
      gradientBox(host.controller, stopsOf(item));

  @override
  Future<void> apply(BrowserHost host, Map<String, dynamic> item) async {
    final c = host.controller;
    final colors = stopsOf(item);
    if (colors.length > 1) {
      host.refresh(() => stops = colors);
      final target = EditorSession.map(c.state['colorTarget']);
      final fill = EditorSession.map(c.activeLayer?['fill']);
      final slot = target['slot'] ?? fill['slot'];
      if (host.has('setGradient') && slot != null) {
        await c.command('setGradient', {'slot': slot, 'stops': colors});
      }
    } else if (host.has('applyPalette')) {
      await c.command('applyPalette', {'rgba': colors.single});
    }
  }

  @override
  List<Widget> tools(BrowserHost host) => [
    shelfAction('From image', () => _paletteFromFile(host)),
  ];

  /// What the wheel edits, in words, then the fill's editor and the wheel.
  @override
  Widget editor(BrowserHost host) {
    final c = host.controller;
    final target = _colorTarget(c);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        if (target != null)
          Padding(
            padding: const EdgeInsets.symmetric(
              horizontal: EditorMetrics.s6,
              vertical: EditorMetrics.s3,
            ),
            child: Text(
              _targetTitle(c, target),
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: const TextStyle(
                fontSize: EditorMetrics.dense,
                color: EditorTheme.ink,
              ),
            ),
          ),
        // The wheel takes the asked size, but never more than the panel
        // leaves beside the stops bar.
        LayoutBuilder(
          builder: (context, box) {
            final side = math
                .min(
                  wheelSize(c),
                  box.maxWidth - _StopsBar.width - EditorMetrics.s6 * 3,
                )
                .clamp(EditorMetrics.s96, EditorMetrics.s200);
            return Padding(
              padding: const EdgeInsets.all(EditorMetrics.s6),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  _ColorPicker(
                    controller: c,
                    target: target,
                    enabled: host.has('setColor'),
                    size: side,
                    unbound: picked ?? const [1, 0, 0, 1],
                    onPick: (rgba) => host.refresh(() => picked = rgba),
                  ),
                  const SizedBox(width: EditorMetrics.s6),
                  _StopsBar(
                    controller: c,
                    height: side,
                    current: picked ?? rgbaOf(target?['rgba']),
                    stops: stops,
                    onStops: (next) => host.refresh(() => stops = next),
                    onPick: (rgba) => target == null
                        ? host.refresh(() => picked = rgba)
                        : c.command('setColor', {
                            'layer': target['layer'],
                            'slot': target['slot'],
                            'rgba': rgba,
                          }),
                  ),
                ],
              ),
            );
          },
        ),
        shelfGrip(
          key: const ValueKey('browser:picker-grip'),
          vertical: true,
          onDrag: (d) => host.refresh(() => wheelDrag = wheelSize(c) + d),
          onEnd: () {
            final size = wheelSize(c);
            wheelDrag = null;
            c.storeDesk('browserWheel', size);
          },
        ),
      ],
    );
  }

  @override
  List<EditorMenuItem<String>> menu(
    BrowserHost host,
    Map<String, dynamic> item,
  ) => [
    if (item['saved'] == true)
      const EditorMenuItem<String>(
        value: 'forget',
        child: Text('Forget swatch'),
      ),
  ];

  @override
  Future<void> act(
    BrowserHost host,
    String action,
    Map<String, dynamic> item,
  ) async {
    if (action != 'forget') return;
    final kept = saved(host.controller)
      ..removeAt(int.parse('${item['id']}'.split(':').last));
    await host.controller.storeDesk('swatches', kept);
  }

  static const _imageExtensions = [
    'png',
    'jpg',
    'jpeg',
    'webp',
    'bmp',
    'gif',
    'tif',
    'tiff',
  ];
  Future<void> _paletteFromFile(BrowserHost host) async {
    final picked = await host.controller.native('pickImport', {
      'extensions': _imageExtensions,
    });
    if (picked is! List) return;
    for (final path in picked.whereType<String>()) {
      await savePalette(host, await File(path).readAsBytes());
    }
  }

  /// The picture's main colours become saved solids, shown at once.
  static Future<void> savePalette(BrowserHost host, Uint8List bytes) async {
    final colors = await paletteOf(bytes);
    if (colors.isEmpty) {
      host.controller.error.value = 'No colours found in that image';
      return;
    }
    await host.controller.storeDesk('swatches', [
      ...saved(host.controller),
      for (final c in colors)
        {
          'stops': [c],
        },
    ]);
    if (host.mounted) host.showCategory('Colors', 'Saved');
  }

  static List<Map<String, dynamic>> saved(EditorSession c) =>
      EditorSession.maps(c.deskWork.value['swatches']);
  static List<List<double>> stopsOf(Map<String, dynamic> item) => [
    for (final s in item['stops'] as List? ?? [item['rgba']]) rgbaOf(s),
  ];
}

List<double> rgbaOf(dynamic raw) {
  final values = (raw as List? ?? [1, 0, 0, 1])
      .map((v) => (v as num).toDouble())
      .toList();
  if (values.length < 4) values.add(1);
  final scale = values.any((v) => v > 1) ? 255.0 : 1.0;
  return values.take(4).map((v) => (v / scale).clamp(0.0, 1.0)).toList();
}

Color _color(List<double> rgba) =>
    Color.from(alpha: rgba[3], red: rgba[0], green: rgba[1], blue: rgba[2]);

/// A solid, or the same strip a saved gradient was made from.
Widget gradientBox(EditorSession controller, List<List<double>> stops) =>
    stops.length > 1
    ? NativeVisualSample(
        controller: controller,
        request: {'kind': 'gradient', 'type': 'linear', 'stops': stops},
        fit: BoxFit.fill,
      )
    : ColoredBox(color: _color(stops.single));

/// The layer a colour target points at, from the status rows.
Map<String, dynamic>? _targetLayer(
  EditorSession controller,
  Map<String, dynamic>? target,
) {
  if (target == null) return null;
  for (final row in controller.state['layers'] as List? ?? const []) {
    if (row is Map && row['id'] == target['layer'])
      return Map<String, dynamic>.from(row);
  }
  return null;
}

/// "Layer name · Fill" / "· Stroke": what the wheel is editing.
String _targetTitle(EditorSession controller, Map<String, dynamic> target) {
  final layer = _targetLayer(controller, target);
  final slot = EditorSession.map(target['slot']);
  final what = slot.containsKey('ShapeStroke')
      ? 'Stroke'
      : slot.keys.any((k) => k.startsWith('ShapeGradient'))
      ? 'Fill · stop'
      : 'Fill';
  final name = '${layer?['name'] ?? ''}'.trim();
  return name.isEmpty ? what : '$name · $what';
}

Map<String, dynamic>? _colorTarget(EditorSession controller) {
  if (controller.state['colorTarget'] is Map)
    return Map<String, dynamic>.from(controller.state['colorTarget']);
  final layer = controller.activeLayer;
  if (layer == null) return null;
  for (final row in EditorSession.maps(layer['properties'])) {
    if (row['kind'] == 'color' && row['slot'] != null)
      return {
        'layer': layer['id'],
        'slot': row['slot'],
        'rgba': row['value'],
        'label': row['label'],
      };
  }
  return null;
}

/// The wheel: hue on the ring, saturation and value inside. It edits the
/// focused slot as a draft while the pointer is down and writes once on
/// release; with nothing focused it only feeds the stops bar.
class _ColorPicker extends StatefulWidget {
  const _ColorPicker({
    required this.controller,
    required this.target,
    required this.enabled,
    required this.size,
    required this.unbound,
    required this.onPick,
  });
  final EditorSession controller;
  final Map<String, dynamic>? target;
  final bool enabled;

  /// The wheel's colour while nothing is focused; the shelf keeps it.
  final List<double> unbound;

  /// The wheel's side; the shelf has already fitted it to the panel.
  final double size;
  final ValueChanged<List<double>> onPick;
  @override
  State<_ColorPicker> createState() => _ColorPickerState();
}

class _ColorPickerState extends State<_ColorPicker> {
  List<double>? draft;
  String? dragPart;
  bool previewUsed = false, ending = false;
  final pickerFocus = FocusNode();
  late final queue = EditorPreviewQueue<List<double>>(
    (rgba) => widget.controller.command('previewColor', {
      'layer': widget.target!['layer'],
      'slot': widget.target!['slot'],
      'rgba': rgba,
    }),
  );
  bool get canPreview =>
      widget.controller.supports('previewColor') && widget.enabled;

  @override
  void dispose() {
    if (previewUsed && !ending) widget.controller.cancelPreview();
    pickerFocus.dispose();
    super.dispose();
  }

  @override
  void didUpdateWidget(covariant _ColorPicker oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.target?['slot'] != widget.target?['slot'] ||
        oldWidget.target?['layer'] != widget.target?['layer']) {
      draft = null;
      dragPart = null;
      if (previewUsed) {
        previewUsed = false;
        widget.controller.cancelPreview();
      }
    }
  }

  List<double> get value =>
      draft ??
      (widget.target == null ? widget.unbound : rgbaOf(widget.target!['rgba']));

  /// Set by the last layout; sampling reads the same geometry the paint used.
  ColorWheel wheel = ColorWheel(EditorMetrics.thumb, 'square');
  String get shape =>
      widget.controller.deskWork.value['colorShape'] as String? ?? 'square';

  void sample(Offset p) {
    pickerFocus.requestFocus();
    final v = value;
    final hsv = HSVColor.fromColor(_color(v));
    dragPart ??= wheel.hitsInner(p, hsv) ? 'sv' : 'hue';
    final updated = dragPart == 'sv'
        ? wheel.pickInner(p, hsv)
        : hsv.withHue(wheel.hueAt(p));
    final c = updated.toColor();
    final next = [c.r, c.g, c.b, v[3]];
    setState(() => draft = next);
    if (widget.target == null) {
      widget.onPick(next);
    } else if (canPreview) {
      previewUsed = true;
      queue.add(next);
    }
  }

  Future<void> commit() async {
    final target = widget.target;
    final next = draft;
    draft = null;
    dragPart = null;
    final used = previewUsed;
    previewUsed = false;
    if (next == null) return;
    if (target == null) {
      setState(() {});
      return;
    }
    if (!widget.enabled) return;
    ending = true;
    try {
      await queue.finish(
        false,
        () => used
            ? widget.controller.command('commitPreview')
            : widget.controller.command('setColor', {
                'layer': target['layer'],
                'slot': target['slot'],
                'rgba': next,
              }),
      );
    } finally {
      ending = false;
      if (mounted) setState(() {});
    }
  }

  void cancel() {
    if (previewUsed) {
      previewUsed = false;
      queue.finish(true, () async => widget.controller.cancelPreview());
    }
    setState(() {
      draft = null;
      dragPart = null;
    });
  }

  @override
  Widget build(BuildContext context) {
    final color = _color(value);
    return CallbackShortcuts(
      bindings: {
        const SingleActivator(LogicalKeyboardKey.escape): () {
          widget.controller.eyedropper.value = false;
          cancel();
        },
      },
      child: Focus(
        focusNode: pickerFocus,
        child: Builder(
          builder: (context) {
            wheel = ColorWheel(widget.size, shape);
            return Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                GestureDetector(
                  onPanStart: (e) => sample(e.localPosition),
                  onPanUpdate: (e) => sample(e.localPosition),
                  onPanEnd: (_) => commit(),
                  onPanCancel: cancel,
                  onTapUp: (e) {
                    sample(e.localPosition);
                    commit();
                  },
                  child: SizedBox(
                    width: wheel.side,
                    height: wheel.side,
                    child: CustomPaint(
                      painter: ColorWheelPainter(color, wheel),
                    ),
                  ),
                ),
                Container(
                  width: wheel.side,
                  height: EditorMetrics.s23,
                  margin: const EdgeInsets.only(top: EditorMetrics.s6),
                  padding: const EdgeInsets.symmetric(
                    horizontal: EditorMetrics.s5,
                  ),
                  decoration: BoxDecoration(
                    color: EditorTheme.line,
                    borderRadius: BorderRadius.circular(EditorMetrics.s5),
                  ),
                  child: Row(
                    children: [
                      Swatch(color: color, size: EditorMetrics.s14),
                      const Spacer(),
                      ValueListenableBuilder<bool>(
                        valueListenable: widget.controller.eyedropper,
                        builder: (context, on, _) => EditorTooltip(
                          message: on
                              ? 'Click the Stage to pick a colour · Esc cancels'
                              : 'Pick a colour from the Stage',
                          child: InkWell(
                            key: const ValueKey('browser:eyedropper'),
                            onTap: () =>
                                widget.controller.eyedropper.value = !on,
                            child: Padding(
                              padding: const EdgeInsets.only(
                                right: EditorMetrics.s6,
                              ),
                              child: Icon(
                                Icons.colorize,
                                size: EditorMetrics.s14,
                                color: on
                                    ? EditorTheme.accent
                                    : EditorTheme.muted,
                              ),
                            ),
                          ),
                        ),
                      ),
                      EditorTooltip(
                        message: shape == 'square'
                            ? 'Switch to triangle'
                            : 'Switch to square',
                        child: InkWell(
                          key: const ValueKey('browser:color-shape'),
                          onTap: () => widget.controller.storeDesk(
                            'colorShape',
                            shape == 'square' ? 'triangle' : 'square',
                          ),
                          child: Icon(
                            shape == 'square'
                                ? Icons.change_history
                                : Icons.crop_square,
                            size: EditorMetrics.s14,
                            color: EditorTheme.muted,
                          ),
                        ),
                      ),
                    ],
                  ),
                ),
              ],
            );
          },
        ),
      ),
    );
  }
}

/// Stacked colours beside the wheel: tap one to pick it, right-click to
/// drop it, `+` adds the current colour, Save keeps the strip as a swatch.
/// This authors the catalogue; it never edits a layer by itself.
class _StopsBar extends StatelessWidget {
  const _StopsBar({
    required this.controller,
    required this.height,
    required this.current,
    required this.stops,
    required this.onStops,
    required this.onPick,
  });
  final EditorSession controller;
  final double height;
  final List<double> current;
  final List<List<double>> stops;
  final ValueChanged<List<List<double>>> onStops;
  final ValueChanged<List<double>> onPick;
  static const double width = EditorMetrics.row;

  @override
  Widget build(BuildContext context) {
    Widget small(IconData icon, String tip, VoidCallback? press, {Key? key}) =>
        EditorTooltip(
          message: tip,
          child: InkWell(
            key: key,
            onTap: press,
            child: SizedBox(
              width: width,
              height: EditorMetrics.row,
              child: Icon(
                icon,
                size: EditorMetrics.s14,
                color: press == null
                    ? EditorTheme.muted.withValues(alpha: .45)
                    : EditorTheme.muted,
              ),
            ),
          ),
        );
    return SizedBox(
      width: width,
      height: height,
      child: Column(
        children: [
          small(
            Icons.add,
            'Add this colour as a stop',
            () => onStops([...stops, current]),
            key: const ValueKey('browser:stop-add'),
          ),
          Expanded(
            child: stops.isEmpty
                ? const SizedBox()
                : ClipRRect(
                    borderRadius: BorderRadius.circular(EditorMetrics.s5),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.stretch,
                      children: [
                        for (final (i, stop) in stops.indexed)
                          Expanded(
                            child: GestureDetector(
                              key: ValueKey('browser:stop:$i'),
                              onTap: () => onPick(List.of(stop)),
                              onSecondaryTap: () =>
                                  onStops([...stops]..removeAt(i)),
                              child: ColoredBox(color: _color(stop)),
                            ),
                          ),
                      ],
                    ),
                  ),
          ),
          small(
            Icons.bookmark_add_outlined,
            stops.length > 1 ? 'Save gradient' : 'Save colour',
            stops.isEmpty
                ? null
                : () {
                    controller.storeDesk('swatches', [
                      ...ColorsShelf.saved(controller),
                      {'stops': stops},
                    ]);
                    onStops([]);
                  },
            key: const ValueKey('browser:stop-save'),
          ),
        ],
      ),
    );
  }
}

/// The few colours a picture is mostly made of: decode small, drop
/// transparent pixels, k-means in RGB, brightest first.
Future<List<List<double>>> paletteOf(Uint8List bytes, {int count = 6}) async {
  final codec = await ui.instantiateImageCodec(bytes, targetWidth: 64);
  final image = (await codec.getNextFrame()).image;
  final data = await image.toByteData(format: ui.ImageByteFormat.rawRgba);
  image.dispose();
  if (data == null) return [];
  final px = <List<double>>[];
  for (var i = 0; i + 3 < data.lengthInBytes; i += 4) {
    if (data.getUint8(i + 3) < 128) continue;
    px.add([
      data.getUint8(i) / 255,
      data.getUint8(i + 1) / 255,
      data.getUint8(i + 2) / 255,
    ]);
  }
  if (px.isEmpty) return [];
  double luma(List<double> c) => .299 * c[0] + .587 * c[1] + .114 * c[2];
  px.sort((a, b) => luma(a).compareTo(luma(b)));
  final k = math.min(count, px.length);
  var centers = [
    for (var i = 0; i < k; i++) px[(px.length - 1) * i ~/ math.max(1, k - 1)],
  ];
  for (var round = 0; round < 8; round++) {
    final sums = List.generate(k, (_) => [0.0, 0.0, 0.0, 0.0]);
    for (final p in px) {
      var best = 0;
      var bestD = double.infinity;
      for (var i = 0; i < k; i++) {
        final c = centers[i];
        final d =
            (p[0] - c[0]) * (p[0] - c[0]) +
            (p[1] - c[1]) * (p[1] - c[1]) +
            (p[2] - c[2]) * (p[2] - c[2]);
        if (d < bestD) {
          bestD = d;
          best = i;
        }
      }
      final s = sums[best];
      s[0] += p[0];
      s[1] += p[1];
      s[2] += p[2];
      s[3] += 1;
    }
    centers = [
      for (var i = 0; i < k; i++)
        sums[i][3] == 0
            ? centers[i]
            : [
                sums[i][0] / sums[i][3],
                sums[i][1] / sums[i][3],
                sums[i][2] / sums[i][3],
              ],
    ];
  }
  // Edge blends and stray pixels do not count as a colour of the picture.
  final weights = List.filled(k, 0);
  for (final p in px) {
    var best = 0;
    var bestD = double.infinity;
    for (var i = 0; i < k; i++) {
      final c = centers[i];
      final d =
          (p[0] - c[0]) * (p[0] - c[0]) +
          (p[1] - c[1]) * (p[1] - c[1]) +
          (p[2] - c[2]) * (p[2] - c[2]);
      if (d < bestD) {
        bestD = d;
        best = i;
      }
    }
    weights[best]++;
  }
  final unique = <String, List<double>>{
    for (final (i, c) in centers.indexed)
      if (weights[i] * 20 >= px.length)
        c.map((v) => (v * 255).round()).join(','): [...c, 1.0],
  };
  return unique.values.toList()..sort((a, b) => luma(b).compareTo(luma(a)));
}
