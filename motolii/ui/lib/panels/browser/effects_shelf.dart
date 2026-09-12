import 'package:flutter/material.dart';

import '../../foundation/metrics.dart';
import '../../session/editor_session.dart';
import '../native_visual_sample.dart';
import 'shelf.dart';

/// Effects: the shelf of effects the engine knows. A tile is the effect's
/// snapshot picture (VST3's plug-in snapshot); the glyph stays when it ships
/// none. Applying puts the picked effects on the selected layers.
class EffectsShelf extends BrowserShelf {
  @override
  String get name => 'Effects';
  @override
  bool get multiSelect => true;

  @override
  List<String> rails(BrowserHost host) => const [
    'All',
    'Blur',
    'Light',
    'Color',
    'Stylize',
    'Distort',
    '3D',
    'Path',
    'Place',
    'Other',
  ];

  @override
  List<Map<String, dynamic>> items(BrowserHost host) =>
      EditorSession.maps(host.controller.state['catalog']);

  @override
  String classification(BrowserHost host, Map<String, dynamic> item) =>
      const <String, String>{
        'motolii.blur': 'Blur',
        'motolii.isf_bloom': 'Light',
        'motolii.glow': 'Light',
        'motolii.radiance': 'Light',
        'motolii.gain': 'Color',
        'motolii.gradient': 'Color',
        'motolii.tri_led': 'Stylize',
        'motolii.repeat': 'Place',
      }[host.id(item)] ??
      // shader を持たない棚の札は native の stage が族(Path = 形の層の輪郭)。
      switch (item['stage']) {
        'Warp' => 'Distort',
        'Field' || 'Surface' || 'Clip' || 'Solid' => '3D',
        'Path' => 'Path',
        _ => 'Other',
      };

  @override
  bool supported(BrowserHost host, Map<String, dynamic> item) =>
      host.has('applyEffect') &&
      host.controller.selectedIds.isNotEmpty &&
      (item['stage'] != 'Path' || _shapeSelected(host.controller.state));

  static bool _shapeSelected(Map<String, dynamic> state) {
    final ids = (state['selectedIds'] as List? ?? const []).toSet();
    return EditorSession.maps(state['layers'])
        .any((l) => ids.contains(l['id']) && l['kind'] == 'Shape');
  }

  @override
  Widget preview(BrowserHost host, Map<String, dynamic> item, Color identity) =>
      Stack(
        fit: StackFit.expand,
        children: [
          Center(
            child: Text(
              '${item['glyph'] ?? 'ƒ'}',
              style: TextStyle(fontSize: EditorMetrics.s23, color: identity),
            ),
          ),
          NativeVisualSample(
            key: ValueKey('browser:effect:${item['id']}'),
            controller: host.controller,
            request: {
              'kind': 'effect',
              'id': item['id'],
              'generation': item['generation'],
            },
            fit: BoxFit.cover,
          ),
        ],
      );

  @override
  Future<void> apply(BrowserHost host, Map<String, dynamic> item) async {
    if (!host.has('applyEffect')) return;
    final chosen = host.visible
        .where((row) => host.selectedIds.contains(host.id(row)))
        .map((row) => row['id'])
        .toList();
    await host.controller.command('applyEffect', {
      'pluginIds': chosen.isEmpty ? [item['id']] : chosen,
    });
  }
}
