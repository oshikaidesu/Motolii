import 'dart:convert';
import 'dart:io';
import 'dart:math' as math;
import 'dart:typed_data';
import 'dart:ui' as ui show instantiateImageCodec, ImageByteFormat;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../../foundation/metrics.dart';
import '../../foundation/theme.dart';
import '../../session/editor_session.dart';
import '../gradient_inspector.dart';
import '../native_visual_sample.dart';
import 'color_wheel.dart';
import 'parts.dart';
import 'shelf.dart';

/// Colours: the wheel edits the selected layer's slot; the tiles are the
/// palette in use, saved swatches and starters, and a click applies one.
class ColorsShelf extends BrowserShelf {
  @override
  String get name => 'Colors';

  /// Colours stacked under the wheel; one is a solid, more make a gradient.
  List<List<double>> stops = [];

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
    return [target, _targetFill(c, target), _targetLayer(c, target)?['name']];
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
        if (_targetFill(c, target) case final fill?)
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: EditorMetrics.s6),
            child: GradientInspector(
              key: ValueKey('colors-fill:${target!['layer']}'),
              controller: c,
              layer: _targetLayer(c, target)!,
              fill: fill,
            ),
          ),
        _ColorPicker(
          controller: c,
          target: target,
          enabled: host.has('setColor'),
          size: wheelSize(c),
          stops: stops,
          onStops: (next) => host.refresh(() => stops = next),
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
  final what = slot.containsKey('ShapeStroke') || slot.containsKey('TextStroke')
      ? 'Stroke'
      : slot.keys.any((k) => k.startsWith('ShapeGradient'))
      ? 'Fill · stop'
      : 'Fill';
  final name = '${layer?['name'] ?? ''}'.trim();
  return name.isEmpty ? what : '$name · $what';
}

/// The shape fill the target belongs to, when the target is a fill or one of
/// its stops; null for strokes and text, which have no gradient to edit.
Map<String, dynamic>? _targetFill(
  EditorSession controller,
  Map<String, dynamic>? target,
) {
  final layer = _targetLayer(controller, target);
  if (layer == null || layer['fill'] is! Map) return null;
  final slot = EditorSession.map(target!['slot']);
  if (slot.containsKey('ShapeStroke')) return null;
  return Map<String, dynamic>.from(layer['fill'] as Map);
}

Map<String, dynamic>? _colorTarget(EditorSession controller) {
  if (controller.state['colorTarget'] is Map)
    return Map<String, dynamic>.from(controller.state['colorTarget']);
  final layer = controller.activeLayer;
  final colors = layer?['colors'] as List? ?? [];
  if (layer == null || colors.isEmpty) return null;
  return {
    ...Map<String, dynamic>.from(colors.first as Map),
    'layer': layer['id'],
  };
}

class _ColorPicker extends StatefulWidget {
  const _ColorPicker({
    required this.controller,
    required this.target,
    required this.enabled,
    required this.size,
    required this.stops,
    required this.onStops,
  });
  final EditorSession controller;
  final Map<String, dynamic>? target;
  final bool enabled;

  /// Wheel side the panel asks for; the width may still shrink it.
  final double size;
  final List<List<double>> stops;
  final ValueChanged<List<List<double>>> onStops;
  @override
  State<_ColorPicker> createState() => _ColorPickerState();
}

class _ColorPickerState extends State<_ColorPicker> {
  List<double> unbound = [1, 0, 0, 1];
  List<double>? draft;
  String? dragPart;
  final hex = TextEditingController();
  final hexFocus = FocusNode();
  final pickerFocus = FocusNode();
  Future<void>? previewFlight;
  Map<String, dynamic>? queuedPreview;
  bool previewUsed = false;
  bool get canPreview =>
      (widget.controller.state['capabilities'] as List? ?? []).contains(
        'previewColor',
      );

  void preview() {
    if (widget.target == null ||
        draft == null ||
        !canPreview ||
        !widget.enabled)
      return;
    previewUsed = true;
    queuedPreview = {
      'layer': widget.target!['layer'],
      'slot': widget.target!['slot'],
      'rgba': List<double>.of(draft!),
    };
    previewFlight ??= pumpPreview();
  }

  Future<void> pumpPreview() async {
    try {
      while (queuedPreview != null) {
        final request = queuedPreview!;
        queuedPreview = null;
        await widget.controller.command('previewColor', request);
      }
    } finally {
      previewFlight = null;
    }
  }

  @override
  void dispose() {
    queuedPreview = null;
    if (previewUsed) widget.controller.cancelPreview();
    hex.dispose();
    hexFocus.dispose();
    pickerFocus.dispose();
    super.dispose();
  }

  @override
  void didUpdateWidget(covariant _ColorPicker oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (jsonEncode(oldWidget.target?['slot']) !=
            jsonEncode(widget.target?['slot']) ||
        oldWidget.target?['layer'] != widget.target?['layer']) {
      draft = null;
      dragPart = null;
      queuedPreview = null;
      if (previewUsed) {
        previewUsed = false;
        widget.controller.cancelPreview();
      }
    }
  }

  List<double> get value =>
      draft ??
      (widget.target == null ? unbound : rgbaOf(widget.target!['rgba']));

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
    setState(() => draft = [c.r, c.g, c.b, v[3]]);
    preview();
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
      setState(() => unbound = next);
      return;
    }
    await previewFlight;
    if (widget.enabled) {
      if (used) {
        await widget.controller.command('commitPreview');
      } else {
        await widget.controller.command('setColor', {
          'layer': target['layer'],
          'slot': target['slot'],
          'rgba': next,
        });
      }
    }
    if (mounted) setState(() {});
  }

  void cancel() {
    queuedPreview = null;
    if (previewUsed) {
      previewUsed = false;
      widget.controller.cancelPreview();
    }
    setState(() {
      draft = null;
      dragPart = null;
    });
  }

  static const double _stopsWidth = EditorMetrics.row;

  /// Stacked colours beside the wheel: tap one to pick it, right-click to
  /// drop it, `+` adds the current colour, Save keeps the strip as a swatch.
  Widget _stopsBar(List<double> current) {
    final stops = widget.stops;
    Widget small(IconData icon, String tip, VoidCallback? press, {Key? key}) =>
        EditorTooltip(
          message: tip,
          child: InkWell(
            key: key,
            onTap: press,
            child: SizedBox(
              width: _stopsWidth,
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
      width: _stopsWidth,
      height: wheel.side,
      child: Column(
        children: [
          small(
            Icons.add,
            'Add this colour as a stop',
            () => widget.onStops([...stops, current]),
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
                              onTap: () {
                                draft = List.of(stop);
                                commit();
                              },
                              onSecondaryTap: () =>
                                  widget.onStops([...stops]..removeAt(i)),
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
                    widget.controller.storeDesk('swatches', [
                      ...ColorsShelf.saved(widget.controller),
                      {'stops': stops},
                    ]);
                    widget.onStops([]);
                  },
            key: const ValueKey('browser:stop-save'),
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final v = value;
    final color = _color(v);
    final display =
        '#${(color.toARGB32() & 0xffffff).toRadixString(16).padLeft(6, '0')}';
    if (!hexFocus.hasFocus) hex.text = display;
    return CallbackShortcuts(
      bindings: {
        const SingleActivator(LogicalKeyboardKey.escape): () {
          widget.controller.eyedropper.value = false;
          cancel();
        },
      },
      child: Focus(
        focusNode: pickerFocus,
        child: Padding(
          padding: const EdgeInsets.all(EditorMetrics.s6),
          child: LayoutBuilder(
            builder: (context, constraints) {
              wheel = ColorWheel(
                math
                    .min(
                      widget.size,
                      constraints.maxWidth - _stopsWidth - EditorMetrics.s6,
                    )
                    .clamp(EditorMetrics.s96, EditorMetrics.s200),
                shape,
              );
              return Column(
                children: [
                  Row(
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
                      const SizedBox(width: EditorMetrics.s6),
                      _stopsBar(v),
                    ],
                  ),
                  Container(
                    width: wheel.side + EditorMetrics.s6 + _stopsWidth,
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
                        const SizedBox(width: EditorMetrics.s6),
                        Expanded(
                          child: TextField(
                            controller: hex,
                            focusNode: hexFocus,
                            readOnly: widget.target == null || !widget.enabled,
                            style: const TextStyle(
                              fontSize: EditorMetrics.font,
                              color: EditorTheme.ink,
                            ),
                            decoration: const InputDecoration(
                              isDense: true,
                              border: InputBorder.none,
                              contentPadding: EdgeInsets.zero,
                            ),
                            onSubmitted: (text) {
                              var raw = text.trim().replaceFirst('#', '');
                              if (raw.length == 3)
                                raw = raw.split('').map((s) => '$s$s').join();
                              final parsed = raw.length == 6
                                  ? int.tryParse(raw, radix: 16)
                                  : null;
                              if (parsed == null) {
                                widget.controller.error.value =
                                    'Enter a valid hex color';
                                return;
                              }
                              draft = [
                                ((parsed >> 16) & 255) / 255,
                                ((parsed >> 8) & 255) / 255,
                                (parsed & 255) / 255,
                                v[3],
                              ];
                              commit();
                            },
                          ),
                        ),
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
