import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart' show ScrollCacheExtent;

import '../foundation/metrics.dart';
import '../foundation/theme.dart';
import '../session/editor_session.dart';
import 'native_visual_sample.dart';

class FontBrowser extends StatefulWidget {
  const FontBrowser({super.key, required this.controller});
  final EditorSession controller;
  @override
  State<FontBrowser> createState() => _FontBrowserState();
}

class _FontBrowserState extends State<FontBrowser> {
  String query = '';
  @override
  Widget build(BuildContext context) => AnimatedBuilder(
    animation: widget.controller.slice(
      'fonts',
      const ['fontFamilies', 'path', 'capabilities'],
      derived: () => fontReading(widget.controller),
    ),
    builder: (context, _) {
      final c = widget.controller;
      final layer = c.activeLayer;
      final text = EditorSession.map(layer?['text']);
      final enabled =
          layer != null &&
          layer['kind'] == 'Text' &&
          layer['locked'] != true &&
          c.supports('setFont');
      final family = text['fontFamily'];
      final names =
          (c.state['fontFamilies'] as List? ?? [])
              .whereType<String>()
              .where((name) => name.toLowerCase().contains(query.toLowerCase()))
              .toList()
            ..sort((a, b) {
              final private = (a.startsWith('.') ? 1 : 0).compareTo(
                b.startsWith('.') ? 1 : 0,
              );
              return private != 0
                  ? private
                  : a.toLowerCase().compareTo(b.toLowerCase());
            });
      return ColoredBox(
        color: EditorTheme.app,
        child: Column(
          children: [
            Padding(
              padding: const EdgeInsets.all(EditorMetrics.s6),
              child: TextField(
                decoration: const InputDecoration(
                  hintText: 'Search fonts',
                  isDense: true,
                ),
                onChanged: (value) => setState(() => query = value),
              ),
            ),
            Padding(
              padding: const EdgeInsets.all(EditorMetrics.s6),
              child: Text(
                enabled
                    ? '${layer['name']} · ${text['content'] ?? ''}'
                    : 'Select an unlocked Text layer',
                maxLines: 2,
                overflow: TextOverflow.ellipsis,
              ),
            ),
            Expanded(
              child: ListView.builder(
                itemCount: names.length,
                itemExtent: EditorMetrics.s85,
                scrollCacheExtent: const ScrollCacheExtent.pixels(0),
                itemBuilder: (context, index) {
                  final name = names[index];
                  return Tooltip(
                    message: name,
                    child: InkWell(
                      key: ValueKey('font:$name'),
                      onTap: !enabled
                          ? null
                          : () => c.command('setFont', {
                              'layer': layer['id'],
                              'family': name,
                              if (c.textStyleTarget.value?['layer'] ==
                                  layer['id'])
                                ...c.textStyleTarget.value!,
                            }),
                      child: Container(
                        padding: const EdgeInsets.symmetric(
                          horizontal: EditorMetrics.s8,
                        ),
                        decoration: BoxDecoration(
                          border: Border(
                            left: BorderSide(
                              width: EditorMetrics.s3,
                              color: family == name
                                  ? EditorTheme.accent
                                  : Colors.transparent,
                            ),
                            bottom: BorderSide(color: EditorTheme.line),
                          ),
                        ),
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.stretch,
                          children: [
                            const SizedBox(height: EditorMetrics.s6),
                            Text(
                              name,
                              maxLines: 1,
                              overflow: TextOverflow.ellipsis,
                              style: TextStyle(
                                fontSize: EditorMetrics.dense,
                                color: family == name
                                    ? EditorTheme.ink
                                    : EditorTheme.muted,
                              ),
                            ),
                            Expanded(
                              child: layer == null
                                  ? const SizedBox()
                                  : NativeVisualSample(
                                      controller: c,
                                      request: {
                                        'kind': 'font',
                                        'layer': layer['id'],
                                        'family': name,
                                        'sampleKey': fontSampleKey(text),
                                        'path': c.state['path'],
                                      },
                                    ),
                            ),
                            const SizedBox(height: EditorMetrics.s4),
                          ],
                        ),
                      ),
                    ),
                  );
                },
              ),
            ),
          ],
        ),
      );
    },
  );
}

/// What a specimen actually depends on. The renderer overrides the family,
/// the fill and the stroke, drops the wrap and the justification, and scales
/// every style so the first one lands at a fixed size; only the first line's
/// opening words are drawn.
Map<String, dynamic> fontSampleKey(Map<String, dynamic> text) {
  final styles = EditorSession.maps(text['styles']);
  final first = (styles.firstOrNull?['size'] as num?)?.toDouble() ?? 1;
  final base = first < 1 ? 1.0 : first;
  double? share(dynamic value) => value is num ? value / base : null;
  return {
    'content': '${text['content'] ?? ''}'
        .split('\n')
        .first
        .split(RegExp(r'\s+'))
        .where((word) => word.isNotEmpty)
        .take(8)
        .join(' '),
    'runs': text['runs'],
    'styles': [
      for (final style in styles)
        {
          'style': EditorSession.map(style['font'])['style'],
          'size': share(style['size']),
          'line_height': share(style['line_height']),
          'tracking': style['tracking'],
          'axes': style['axes'],
          'features': style['features'],
        },
    ],
  };
}

/// The shelf follows the family in force, the words it draws, and whether the
/// selected layer can take a font at all.
Object? fontReading(EditorSession c) {
  final layer = c.activeLayer;
  if (layer == null) return null;
  final text = EditorSession.map(layer['text']);
  return [
    layer['id'],
    layer['kind'],
    layer['locked'],
    layer['name'],
    text['content'],
    text['fontFamily'],
    fontSampleKey(text),
  ];
}
