import 'dart:io';
import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:flutter/services.dart';

import '../../foundation/metrics.dart';
import '../../foundation/theme.dart';
import '../../session/editor_session.dart';
import 'color_palette.dart';
import 'color_picker.dart';
import 'color_values.dart';
import 'fill_definitions.dart';
import 'parts.dart';
import 'shelf.dart';

export 'color_palette.dart';
export 'color_values.dart';

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
        ground: EditorTheme.of(host.context).panel,
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
              style: TextStyle(
                fontSize: EditorMetrics.dense,
                color: EditorTheme.of(host.context).ink,
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
              child: ColorPicker(
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
          FillDefinitions(
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
