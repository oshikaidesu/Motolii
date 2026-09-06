import 'package:flutter/material.dart';

import '../session/editor_session.dart';
import '../foundation/theme.dart';
import '../foundation/metrics.dart';

class CompositionControls extends StatelessWidget {
  const CompositionControls({super.key, required this.controller});
  final EditorSession controller;
  @override
  Widget build(BuildContext context) =>
      ValueListenableBuilder<Map<String, dynamic>>(
        valueListenable: controller.document,
        builder: (_, s, __) => Padding(
          padding: const EdgeInsets.all(EditorMetrics.s8),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              const Text('COMPOSITION'),
              Wrap(
                children: [
                  for (final preset in [
                    ('16:9', 1920, 1080),
                    ('9:16', 1080, 1920),
                    ('1:1', 1080, 1080),
                    ('4K', 3840, 2160),
                  ])
                    EditorButton(
                      preset.$1,
                      () => controller.command('composition', {
                        'width': preset.$2,
                        'height': preset.$3,
                      }),
                    ),
                ],
              ),
              for (final key in ['width', 'height', 'durationFrames'])
                Row(
                  children: [
                    SizedBox(width: EditorMetrics.s90, child: Text(key)),
                    Expanded(
                      child: TextFormField(
                        key: ValueKey('$key-${s[key]}'),
                        initialValue: '${s[key]}',
                        style: const TextStyle(fontSize: EditorMetrics.font),
                        onFieldSubmitted: (value) {
                          final parsed = int.tryParse(value);
                          if (parsed == null || parsed < 1) {
                            controller.error.value = 'Enter a positive integer';
                            return;
                          }
                          controller.command('composition', {key: parsed});
                        },
                      ),
                    ),
                  ],
                ),
              Wrap(
                children: [
                  for (final fps in [
                    ('23.976', 24000, 1001),
                    ('24', 24, 1),
                    ('25', 25, 1),
                    ('29.97', 30000, 1001),
                    ('30', 30, 1),
                    ('59.94', 60000, 1001),
                    ('60', 60, 1),
                  ])
                    EditorButton(
                      fps.$1,
                      () => controller.command('composition', {
                        'fpsNum': fps.$2,
                        'fpsDen': fps.$3,
                      }),
                      selected:
                          (s['fps'] as num? ?? 0).toDouble() == fps.$2 / fps.$3,
                    ),
                ],
              ),
            ],
          ),
        ),
      );
}
