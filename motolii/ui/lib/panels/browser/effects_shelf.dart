import 'package:flutter/material.dart';

import '../../app/editor_window.dart' show effectsNotice;
import '../../foundation/metrics.dart';
import '../../foundation/theme.dart';
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

  /// 棚の頭の理由は世代を動かさずに変わる(壊れた保存)。理由が変われば描き直す。
  @override
  List<Object?> derived(EditorSession c) => [effectsNotice(c.state)];

  /// 棚を読み直す口と、断った効果の理由。保存すれば見張りが読み直すが、
  /// 手でも押せる(ISF Editor の Reload)。理由は 1 行、全文は tooltip。
  @override
  Widget? header(BrowserHost host) {
    if (!host.has('reloadEffects')) return null;
    final notice = effectsNotice(host.controller.state);
    return Container(
      key: const ValueKey('browser:effects:header'),
      height: EditorMetrics.control,
      padding: const EdgeInsets.symmetric(horizontal: EditorMetrics.s4),
      decoration: const BoxDecoration(
        border: Border(bottom: BorderSide(color: EditorTheme.line)),
      ),
      child: Row(
        children: [
          EditorTooltip(
            message: 'Reload effects',
            child: IconButton(
              key: const ValueKey('browser:effects:reload'),
              iconSize: EditorMetrics.s14,
              color: EditorTheme.muted,
              onPressed: () => host.controller.command('reloadEffects'),
              icon: const Icon(Icons.refresh),
            ),
          ),
          const SizedBox(width: EditorMetrics.s4),
          Expanded(
            child: EditorTooltip(
              message: notice,
              child: Text(
                notice,
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                style: const TextStyle(color: EditorTheme.error),
              ),
            ),
          ),
        ],
      ),
    );
  }

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
      NativeVisualSample(
        key: ValueKey('browser:effect:${item['id']}'),
        controller: host.controller,
        request: {
          'kind': 'effect',
          'id': item['id'],
          'generation': item['generation'],
        },
        fit: BoxFit.cover,
        placeholder: Center(
          child: Text(
            '${item['glyph'] ?? 'ƒ'}',
            style: TextStyle(fontSize: EditorMetrics.s23, color: identity),
          ),
        ),
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
