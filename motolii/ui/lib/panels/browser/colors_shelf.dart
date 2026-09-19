import 'dart:io';
import 'dart:math' as math;
import 'dart:ui'
    as ui
    show instantiateImageCodec, ImageByteFormat, ViewFocusEvent, ViewFocusState;

import 'package:flutter/widgets.dart';
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
import '../../foundation/glyphs.dart';
import '../../foundation/leaves.dart';

/// Colours: the wheel edits the selected layer's slot; the tiles are the
/// palette in use, saved swatches and starters, and a click applies one.
class ColorsShelf extends BrowserShelf {
  @override
  String get name => 'Colors';

  /// The wheel's colour while nothing is focused.
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
    return [target, _targetLayer(c, target)?['name'], c.activeLayer?['fill']];
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

  /// What the swatch is: solid or gradient, how its stops are blended, how
  /// many, and where it came from — all read off the swatch itself.
  @override
  List<FilterGroup> groups(BrowserHost host) => const [
    FilterGroup('Kind', ['Solid', 'Gradient']),
    FilterGroup('Blend', [], kind: FilterKind.actual),
    FilterGroup('Stops', [], kind: FilterKind.actual),
    FilterGroup('Source', ['Used here', 'Saved', 'Starter']),
  ];

  @override
  Set<String> tagsOf(BrowserHost host, Map<String, dynamic> item) => {
    stopsOf(item).length > 1 ? 'Gradient' : 'Solid',
    classification(host, item),
  };

  @override
  String? valueOf(BrowserHost host, Map<String, dynamic> item, String group) {
    final stops = stopsOf(item).length;
    if (stops < 2) return null;
    return switch (group) {
      'Blend' =>
        const {
              'rgb': 'RGB',
              'linear_rgb': 'Linear',
              'oklab': 'Oklab',
              'oklch_short': 'Oklch short',
              'oklch_long': 'Oklch long',
              'steps': 'Steps',
            }['${item['blend'] ?? 'oklab'}'] ??
            '${item['blend']}',
      'Stops' => '$stops',
      _ => null,
    };
  }

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
      gradientBox(host.controller, stopsOf(item), blend: item['blend']);

  @override
  Future<void> apply(BrowserHost host, Map<String, dynamic> item) async {
    final c = host.controller;
    final colors = stopsOf(item);
    if (colors.length > 1) {
      final target = EditorSession.map(c.state['colorTarget']);
      final fill = EditorSession.map(c.activeLayer?['fill']);
      final slot = target['slot'] ?? fill['slot'];
      if (host.has('setGradient') && slot != null) {
        await c.command('setGradient', {
          'slot': slot,
          'stops': colors,
          if (item['blend'] != null) 'blend': item['blend'],
        });
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
        // The wheel takes the asked size, but never more than the panel.
        LayoutBuilder(
          builder: (context, box) {
            final side = math
                .min(wheelSize(c), box.maxWidth - EditorMetrics.s6 * 2)
                .clamp(EditorMetrics.s96, EditorMetrics.s200);
            return Padding(
              padding: const EdgeInsets.all(EditorMetrics.s6),
              child: _ColorPicker(
                controller: c,
                target: target,
                enabled: host.has('setColor'),
                size: side,
                unbound: picked ?? const [1, 0, 0, 1],
                onPick: (rgba) => host.refresh(() => picked = rgba),
              ),
            );
          },
        ),
        if (c.activeLayer?['fill'] is Map && host.has('setGradient'))
          _FillDefinitions(
            controller: c,
            fill: EditorSession.map(c.activeLayer!['fill']),
          ),
        Padding(
          padding: const EdgeInsets.fromLTRB(
            EditorMetrics.s6,
            0,
            EditorMetrics.s6,
            EditorMetrics.s4,
          ),
          child: EditorButton(
            'Save current color',
            target == null ? null : () => saveCurrent(host),
            key: const ValueKey('browser:save-current-color'),
          ),
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

  Future<void> saveCurrent(BrowserHost host) async {
    final fill = EditorSession.map(host.controller.activeLayer?['fill']);
    final target = _colorTarget(host.controller);
    final colors = fill.isNotEmpty
        ? [
            for (final stop in EditorSession.maps(fill['stops']))
              rgbaOf(stop['rgba']),
          ]
        : target == null
        ? <List<double>>[]
        : [rgbaOf(target['rgba'])];
    if (colors.isEmpty) return;
    await host.controller.storeDesk('swatches', [
      ...saved(host.controller),
      {
        'stops': colors,
        if (colors.length > 1) 'blend': fill['blend'] ?? 'oklab',
      },
    ]);
    if (host.mounted) host.showCategory('Colors', 'Saved');
  }

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
Widget gradientBox(
  EditorSession controller,
  List<List<double>> stops, {
  String? blend,
}) => stops.length > 1
    ? NativeVisualSample(
        controller: controller,
        request: {
          'kind': 'gradient',
          'type': 'linear',
          'stops': stops,
          if (blend != null) 'blend': blend,
        },
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
  if (target['slot'] == 'Background') return 'Composition · Background';
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
  final fill = EditorSession.map(layer['fill']);
  final stops = EditorSession.maps(fill['stops']);
  if (stops.isNotEmpty) {
    return {
      'layer': layer['id'],
      'slot': stops.first['slot'] ?? fill['slot'],
      'rgba': stops.first['rgba'],
      'label': 'Fill',
    };
  }
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

class _ColorPickerState extends State<_ColorPicker>
    with WidgetsBindingObserver {
  List<double>? draft;
  String? dragPart;
  double? rememberedHue;
  bool previewUsed = false, ending = false, gestureCancelled = false;
  final pickerFocus = FocusNode();
  late final queue = EditorPreviewQueue<Map<String, dynamic>>(
    (patch) => widget.controller.command('previewColor', patch),
  );
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state != AppLifecycleState.resumed) cancel();
  }

  @override
  void didChangeViewFocus(ui.ViewFocusEvent event) {
    if (event.state == ui.ViewFocusState.unfocused) cancel();
  }

  bool get canPreview =>
      widget.controller.supports('previewColor') && widget.enabled;

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    if (previewUsed && !ending)
      queue.finish(true, () => widget.controller.command('cancelPreview'));
    pickerFocus.dispose();
    super.dispose();
  }

  @override
  void didUpdateWidget(covariant _ColorPicker oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (!sameValue(oldWidget.target?['slot'], widget.target?['slot']) ||
        oldWidget.target?['layer'] != widget.target?['layer']) {
      cancel();
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
    if (ending || gestureCancelled || !widget.enabled) return;
    pickerFocus.requestFocus();
    final v = value;
    var hsv = HSVColor.fromColor(_color(v));
    if (dragPart == null) {
      if (hsv.saturation > 1 / 255) rememberedHue = hsv.hue;
      hsv = hsv.withHue(rememberedHue ?? hsv.hue);
      dragPart = wheel.hitsInner(p, hsv) ? 'sv' : 'hue';
    } else if (dragPart == 'sv') {
      // White and black have no hue. Keep the hue that this gesture began
      // with instead of deriving 0° (red) from an achromatic RGB snapshot.
      hsv = hsv.withHue(rememberedHue ?? hsv.hue);
    }
    final updated = dragPart == 'sv'
        ? wheel.pickInner(p, hsv)
        : hsv.withHue(wheel.hueAt(p));
    rememberedHue = updated.hue;
    final c = updated.toColor();
    final next = [c.r, c.g, c.b, v[3]];
    setState(() => draft = next);
    if (widget.target == null) {
      widget.onPick(next);
    } else if (canPreview) {
      previewUsed = true;
      queue.add({
        if (widget.target!['layer'] != null) 'layer': widget.target!['layer'],
        'slot': widget.target!['slot'],
        'rgba': next,
      });
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
                if (target['layer'] != null) 'layer': target['layer'],
                'slot': target['slot'],
                'rgba': next,
              }),
      );
    } finally {
      ending = false;
      if (mounted) setState(() {});
    }
  }

  Future<void> cancel() async {
    if (ending) return;
    gestureCancelled = true;
    final used = previewUsed;
    previewUsed = false;
    setState(() {
      draft = null;
      dragPart = null;
    });
    if (!used) return;
    ending = true;
    try {
      await queue.finish(
        true,
        () => widget.controller.command('cancelPreview'),
      );
    } finally {
      ending = false;
    }
  }

  @override
  Widget build(BuildContext context) {
    final color = _color(value);
    final hsv = HSVColor.fromColor(color);
    if (dragPart == null && hsv.saturation > 1 / 255) {
      rememberedHue = hsv.hue;
    }
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
                  onPanStart: (e) {
                    gestureCancelled = false;
                    sample(e.localPosition);
                  },
                  onPanUpdate: (e) => sample(e.localPosition),
                  onPanEnd: (_) => commit(),
                  onPanCancel: cancel,
                  onTapUp: (e) {
                    gestureCancelled = false;
                    sample(e.localPosition);
                    commit();
                  },
                  child: SizedBox(
                    width: wheel.side,
                    height: wheel.side,
                    child: CustomPaint(
                      painter: ColorWheelPainter(
                        color,
                        wheel,
                        hue: rememberedHue,
                      ),
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
                      const SizedBox(width: EditorMetrics.s4),
                      Expanded(
                        child: EditorDraftField(
                          key: ValueKey(
                            'browser:hex:${widget.target?['layer']}:${widget.target?['slot']}',
                          ),
                          value: hexOf(color),
                          label: 'hex',
                          enabled: widget.enabled,
                          validator: (v) => parseHex(v) == null
                              ? 'Use three or six hex digits'
                              : null,
                          onCommit: (v) async {
                            final c = parseHex(v)!;
                            final rgba = [c.r, c.g, c.b, value[3]];
                            if (widget.target == null) {
                              widget.onPick(rgba);
                              return;
                            }
                            await widget.controller.command('setColor', {
                              if (widget.target!['layer'] != null)
                                'layer': widget.target!['layer'],
                              'slot': widget.target!['slot'],
                              'rgba': rgba,
                            });
                          },
                        ),
                      ),
                      const SizedBox(width: EditorMetrics.s4),
                      ValueListenableBuilder<bool>(
                        valueListenable: widget.controller.eyedropper,
                        builder: (context, on, _) => EditorTooltip(
                          message: on
                              ? 'Click the Stage to pick a colour · Esc cancels'
                              : 'Pick a colour from the Stage',
                          child: EditorPress(
                            key: const ValueKey('browser:eyedropper'),
                            onTap: () =>
                                widget.controller.eyedropper.value = !on,
                            child: Container(
                              padding: const EdgeInsets.all(EditorMetrics.s3),
                              decoration: BoxDecoration(
                                color: on ? EditorTheme.hover : null,
                                borderRadius: BorderRadius.circular(
                                  EditorMetrics.s3,
                                ),
                              ),
                              child: Icon(
                                Glyph.colorize,
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
                        child: EditorPress(
                          key: const ValueKey('browser:color-shape'),
                          onTap: () => widget.controller.storeDesk(
                            'colorShape',
                            shape == 'square' ? 'triangle' : 'square',
                          ),
                          child: Padding(
                            padding: const EdgeInsets.symmetric(
                              horizontal: EditorMetrics.s3,
                            ),
                            child: Text(
                              shape == 'square' ? 'Square' : 'Triangle',
                              style: const TextStyle(
                                fontSize: EditorMetrics.micro,
                                color: EditorTheme.muted,
                              ),
                            ),
                          ),
                        ),
                      ),
                    ],
                  ),
                ),
                SizedBox(
                  width: wheel.side,
                  height: EditorMetrics.s14,
                  child: ValueListenableBuilder<bool>(
                    valueListenable: widget.controller.eyedropper,
                    builder: (context, on, _) => Text(
                      on ? 'PICK STAGE · ESC TO CANCEL' : 'HEX · 3 OR 6 DIGITS',
                      style: TextStyle(
                        fontSize: EditorMetrics.micro,
                        color: on ? EditorTheme.accent : EditorTheme.muted,
                      ),
                    ),
                  ),
                ),
                // The alpha, only where the slot carries one (text, params).
                if (widget.target?['alpha'] == true)
                  SizedBox(
                    width: wheel.side,
                    height: EditorMetrics.s23,
                    child: Row(
                      children: [
                        const Icon(
                          Glyph.opacity,
                          size: EditorMetrics.s14,
                          color: EditorTheme.muted,
                        ),
                        Expanded(
                          child: EditorSlider(
                            value: value[3],
                            onChanged: !widget.enabled
                                ? null
                                : (a) {
                                    final v = value;
                                    final next = [v[0], v[1], v[2], a];
                                    setState(() => draft = next);
                                    if (canPreview) {
                                      previewUsed = true;
                                      queue.add({
                                        if (widget.target!['layer'] != null)
                                          'layer': widget.target!['layer'],
                                        'slot': widget.target!['slot'],
                                        'rgba': next,
                                      });
                                    }
                                  },
                            onChangeEnd: (_) => commit(),
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

/// The fill's definitions, written hard into the document: its kind and how
/// the colour travels between stops. One press applies; the tiles are the
/// current fill drawn each way, so the choice is seen before it is made.
class _FillDefinitions extends StatelessWidget {
  const _FillDefinitions({required this.controller, required this.fill});
  final EditorSession controller;
  final Map<String, dynamic> fill;
  static const kinds = ['solid', 'linear', 'radial', 'angular', 'diamond'];
  static const blends = {
    'rgb': 'RGB',
    'linear_rgb': 'Linear',
    'oklab': 'Oklab',
    'oklch_short': 'Oklch',
    'oklch_long': 'Oklch long',
    'steps': 'Steps',
  };

  @override
  Widget build(BuildContext context) {
    final stops = [
      for (final s in EditorSession.maps(fill['stops'])) rgbaOf(s['rgba']),
    ];
    if (stops.isEmpty) return const SizedBox.shrink();
    final sample = stops.length > 1 ? stops : [stops.first, stops.first];
    Widget tile(
      String key,
      String tip,
      bool on,
      Widget picture,
      VoidCallback press,
    ) => Expanded(
      child: EditorTooltip(
        message: tip,
        child: EditorPress(
          key: ValueKey(key),
          onTap: press,
          child: Container(
            height: EditorMetrics.s16,
            clipBehavior: Clip.antiAlias,
            decoration: BoxDecoration(
              border: Border.all(
                color: on ? EditorTheme.accent : EditorTheme.line,
                width: on ? EditorMetrics.s2 : 1,
              ),
              borderRadius: BorderRadius.circular(EditorMetrics.s2),
            ),
            child: picture,
          ),
        ),
      ),
    );
    Widget row(String label, String current, List<Widget> tiles) => Padding(
      padding: const EdgeInsets.fromLTRB(
        EditorMetrics.s6,
        0,
        EditorMetrics.s6,
        EditorMetrics.s4,
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text(
            '$label · $current'.toUpperCase(),
            style: const TextStyle(
              fontSize: EditorMetrics.micro,
              color: EditorTheme.muted,
            ),
          ),
          const SizedBox(height: EditorMetrics.s2),
          Row(
            children: [
              for (final (i, t) in tiles.indexed) ...[
                if (i > 0) const SizedBox(width: EditorMetrics.s3),
                t,
              ],
            ],
          ),
        ],
      ),
    );
    final kind = fill['kind'] ?? 'solid';
    final blend = fill['blend'] ?? 'oklab';
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        row('Fill type', kind, [
          for (final k in kinds)
            tile(
              'browser:fill-kind:$k',
              '${k[0].toUpperCase()}${k.substring(1)} fill',
              kind == k,
              k == 'solid'
                  ? ColoredBox(color: _color(stops.first))
                  : NativeVisualSample(
                      controller: controller,
                      request: {
                        'kind': 'gradient',
                        'type': k,
                        'stops': sample,
                        'blend': blend,
                        'angle': fill['angle'] ?? 0,
                      },
                      fit: BoxFit.fill,
                    ),
              () => k == 'solid'
                  ? controller.command('setFillMode', {
                      'slot': fill['slot'],
                      'gradient': false,
                    })
                  : controller.command('setGradient', {
                      'slot': fill['slot'],
                      'kind': k,
                    }),
            ),
        ]),
        if (kind != 'solid')
          row('Blend', blends[blend] ?? blend, [
            for (final b in blends.entries)
              tile(
                'browser:fill-blend:${b.key}',
                'Colours travel ${b.value}',
                blend == b.key,
                NativeVisualSample(
                  controller: controller,
                  request: {
                    'kind': 'gradient',
                    'type': 'linear',
                    'stops': sample,
                    'blend': b.key,
                  },
                  fit: BoxFit.fill,
                ),
                () => controller.command('setGradient', {
                  'slot': fill['slot'],
                  'blend': b.key,
                }),
              ),
          ]),
      ],
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
