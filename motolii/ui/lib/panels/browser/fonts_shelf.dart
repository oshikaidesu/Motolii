import 'package:flutter/material.dart';

import '../../foundation/metrics.dart';
import '../../foundation/panel_controls.dart';
import '../../foundation/theme.dart';
import '../../session/editor_session.dart';
import 'shelf.dart';

/// Fonts: the editor at the top says which characters of the selected text
/// layer are being dressed (a script, a case, or all of it) and how they sit
/// (size, alignment); the tiles are the machine's families drawn with that
/// layer's own words, and a click dresses those characters. Discrete choices
/// live here; the Inspector keeps the numbers that take keys.
class FontsShelf extends BrowserShelf {
  @override
  String get name => 'Fonts';
  @override
  bool get showViews => false;

  /// A click dresses the layer; the name rides above the specimen.
  @override
  bool get bare => true;

  static const scopes = [
    MapEntry('all', 'All text'),
    MapEntry('hiragana', 'Hiragana'),
    MapEntry('katakana', 'Katakana'),
    MapEntry('han', 'Kanji'),
    MapEntry('latin', 'Latin'),
    MapEntry('upper', 'Uppercase'),
    MapEntry('lower', 'Lowercase'),
  ];

  @override
  List<Object?> derived(EditorSession c) {
    final layer = c.activeLayer;
    final text = EditorSession.map(layer?['text']);
    return [
      layer?['id'],
      layer?['kind'],
      layer?['locked'],
      layer?['name'],
      text['fontFamily'],
      text['content'],
      _justify(layer)?['value'],
    ];
  }

  @override
  List<String> rails(BrowserHost host) => const ['All', 'Used here'];

  @override
  List<Map<String, dynamic>> items(BrowserHost host) {
    final c = host.controller;
    final used = {
      for (final layer in c.layers)
        if (EditorSession.map(layer['text'])['fontFamily'] is String)
          EditorSession.map(layer['text'])['fontFamily'],
    };
    final names =
        (c.state['fontFamilies'] as List? ?? const [])
            .whereType<String>()
            .toList()
          ..sort((a, b) {
            final private = (a.startsWith('.') ? 1 : 0).compareTo(
              b.startsWith('.') ? 1 : 0,
            );
            return private != 0
                ? private
                : a.toLowerCase().compareTo(b.toLowerCase());
          });
    return [
      for (final name in names)
        {'id': 'font:$name', 'name': name, 'used': used.contains(name)},
    ];
  }

  @override
  String classification(BrowserHost host, Map<String, dynamic> item) =>
      item['used'] == true ? 'Used here' : 'All';

  /// One family per row: the name, then one line of the layer's words in it.
  @override
  ShelfLayout layout(BrowserHost host, double width, double tile) =>
      ShelfLayout(
        column: width,
        extent: EditorMetrics.s48,
        gap: 0,
        padding: 0,
        ground: EditorTheme.app,
      );

  Map<String, dynamic>? _layer(EditorSession c) {
    final layer = c.activeLayer;
    return layer != null && layer['kind'] == 'Text' ? layer : null;
  }

  bool _enabled(BrowserHost host) {
    final layer = _layer(host.controller);
    return layer != null && layer['locked'] != true && host.has('setFont');
  }

  /// With a text layer the row dresses it; with none it is a shortcut: a
  /// new text layer in that face.
  /// Making a layer is a double-click; dressing one is a click.
  @override
  bool doubleClick(BrowserHost host, Map<String, dynamic> item) =>
      _layer(host.controller) == null;

  @override
  bool supported(BrowserHost host, Map<String, dynamic> item) =>
      _enabled(host) || (_layer(host.controller) == null && host.has('create'));

  /// The specimen is Flutter's own text in that family — no trip to the
  /// machine, no picture: the words are the layer's first line, or the
  /// family's name when nothing is selected. A face the machine never listed
  /// is not on the shelf, so the name always resolves to an installed font.
  @override
  Widget preview(BrowserHost host, Map<String, dynamic> item, Color identity) {
    final c = host.controller;
    final layer = _layer(c);
    final text = EditorSession.map(layer?['text']);
    final name = '${item['name']}';
    final chosen = layer != null && text['fontFamily'] == name;
    final words = layer == null
        ? name
        : '${text['content'] ?? ''}'.split('\n').first.trim();
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: EditorMetrics.s8),
      decoration: BoxDecoration(
        border: Border(
          left: BorderSide(
            width: EditorMetrics.s3,
            color: chosen ? EditorTheme.accent : Colors.transparent,
          ),
          bottom: const BorderSide(color: EditorTheme.line),
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Text(
            name,
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
            style: TextStyle(
              fontSize: EditorMetrics.dense,
              color: chosen ? EditorTheme.ink : EditorTheme.muted,
            ),
          ),
          Text(
            words.isEmpty ? name : words,
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
            style: TextStyle(
              fontFamily: name,
              fontSize: EditorMetrics.s22,
              height: 1.1,
              color: EditorTheme.ink,
            ),
          ),
        ],
      ),
    );
  }

  /// The characters being dressed: the scope the editor chose for this layer,
  /// and the words as typed so far (the machine refuses a stale draft).
  Map<String, dynamic> _target(EditorSession c, Map<String, dynamic> layer) {
    final held = c.textStyleTarget.value;
    return held != null && held['layer'] == layer['id']
        ? Map<String, dynamic>.from(held)
        : {'layer': layer['id'], 'scope': 'all'};
  }

  @override
  Future<void> apply(BrowserHost host, Map<String, dynamic> item) async {
    final c = host.controller;
    final layer = _layer(c);
    if (layer == null) {
      if (host.has('create'))
        await c.command('create', {'kind': 'text', 'family': item['name']});
      return;
    }
    if (!_enabled(host)) return;
    await c.command('setFont', {
      ..._target(c, layer),
      'layer': layer['id'],
      'family': item['name'],
    });
  }

  Map<String, dynamic>? _justify(Map<String, dynamic>? layer) =>
      EditorSession.maps(layer?['properties'])
          .where((p) => p['id'] == 'text_justify')
          .firstOrNull;

  /// Which characters, how big, how aligned — the discrete half of a text's
  /// setting, one row each; the tiles below answer "in which face".
  @override
  Widget editor(BrowserHost host) {
    final c = host.controller;
    return ValueListenableBuilder<Map<String, dynamic>?>(
      valueListenable: c.textStyleTarget,
      builder: (context, _, _) {
        final layer = _layer(c);
        if (layer == null) {
          return const Padding(
            padding: EdgeInsets.all(EditorMetrics.s6),
            child: Text(
              'Double-click a face to add a text layer',
              style: TextStyle(color: EditorTheme.muted),
            ),
          );
        }
        final text = EditorSession.map(layer['text']);
        final target = _target(c, layer);
        final enabled = _enabled(host) && c.supports('styleText');
        final scope = '${target['scope'] ?? 'all'}';
        final styles = EditorSession.maps(text['styles']);
        final size = (styles.firstOrNull?['size'] as num?)?.toDouble() ?? 24;
        final justify = _justify(layer);
        Future<void> style(Map<String, dynamic> patch) =>
            c.command('styleText', {...target, 'layer': layer['id'], ...patch});
        return Padding(
          padding: const EdgeInsets.all(EditorMetrics.s6),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Text(
                '${layer['name']} · ${text['content'] ?? ''}',
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                style: const TextStyle(color: EditorTheme.muted),
              ),
              const SizedBox(height: EditorMetrics.s6),
              Row(
                children: [
                  Expanded(
                    child: EditorChoice<String>(
                      key: const ValueKey('fonts:scope'),
                      value: scope,
                      choices: scopes,
                      onChanged: !enabled
                          ? null
                          : (next) => c.textStyleTarget.value = {
                              ...target,
                              'layer': layer['id'],
                              'scope': next,
                            },
                    ),
                  ),
                  const SizedBox(width: EditorMetrics.s6),
                  SizedBox(
                    width: EditorMetrics.s96,
                    child: EditorNumericField(
                      key: const ValueKey('fonts:size'),
                      value: size,
                      label: 'Character size',
                      unit: 'px',
                      min: 1,
                      max: 1000,
                      enabled: enabled,
                      onPreview: (v) => style({'size': v, 'preview': true}),
                      onCommit: (v) => style({'size': v}),
                      onFinish: () => c.command('commitPreview'),
                      onCancel: () => c.command('cancelPreview'),
                    ),
                  ),
                  if (justify != null) ...[
                    const SizedBox(width: EditorMetrics.s6),
                    _Justify(
                      value: justify['value'],
                      enabled: enabled && host.has('setProperty'),
                      onPick: (value) => c.command('setProperty', {
                        'layer': layer['id'],
                        'property': 'text_justify',
                        'value': value,
                      }),
                    ),
                  ],
                ],
              ),
            ],
          ),
        );
      },
    );
  }
}

/// Justification is one control, not three spread across a card: three
/// squares of a well's height, side by side, the chosen one lit.
class _Justify extends StatelessWidget {
  const _Justify({
    required this.value,
    required this.enabled,
    required this.onPick,
  });
  final dynamic value;
  final bool enabled;
  final ValueChanged<int> onPick;
  @override
  Widget build(BuildContext context) => Container(
    height: EditorMetrics.row,
    decoration: BoxDecoration(
      color: EditorTheme.app,
      border: Border.all(color: EditorTheme.line),
    ),
    child: Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        for (final entry in const {
          0: (Icons.format_align_left, 'Align left'),
          2: (Icons.format_align_center, 'Align center'),
          1: (Icons.format_align_right, 'Align right'),
        }.entries)
          SizedBox(
            width: EditorMetrics.control,
            child: IconButton(
              key: ValueKey('fonts:justify:${entry.key}'),
              tooltip: entry.value.$2,
              isSelected: value == entry.key,
              icon: Icon(entry.value.$1, size: EditorMetrics.s14),
              color: value == entry.key
                  ? EditorTheme.accent
                  : EditorTheme.muted,
              onPressed: enabled ? () => onPick(entry.key) : null,
            ),
          ),
      ],
    ),
  );
}
