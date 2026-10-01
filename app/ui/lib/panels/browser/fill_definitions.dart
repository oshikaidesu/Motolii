import 'package:flutter/widgets.dart';

import '../../foundation/leaves.dart';
import '../../foundation/metrics.dart';
import '../../foundation/theme.dart';
import '../../session/editor_session.dart';
import '../native_visual_sample.dart';
import 'color_values.dart';

/// The fill's definitions, written hard into the document: its kind and how
/// the colour travels between stops. One press applies; the tiles are the
/// current fill drawn each way, so the choice is seen before it is made.
class FillDefinitions extends StatelessWidget {
  const FillDefinitions({required this.controller, required this.fill});
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
                color: on
                    ? EditorTheme.of(context).accent
                    : EditorTheme.of(context).line,
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
            style: TextStyle(
              fontSize: EditorMetrics.micro,
              color: EditorTheme.of(context).muted,
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
                  ? ColoredBox(color: colorOf(stops.first))
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
