part of 'inspector.dart';

/// The slot is the edit target; the colour control only supplies a value.
class EditorColorRow extends StatelessWidget {
  const EditorColorRow({
    required this.controller,
    required this.layer,
    required this.color,
  });
  final EditorSession controller;
  final Map<String, dynamic> layer, color;
  @override
  Widget build(BuildContext context) {
    final rgba = (color['rgba'] as List? ?? [0, 0, 0, 1]).cast<num>();
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        SizedBox(
          width: EditorMetrics.s48,
          child: Text(
            '${color['label']}',
            style: const TextStyle(
              fontSize: EditorMetrics.micro,
              color: EditorTheme.muted,
            ),
          ),
        ),
        Expanded(
          child: EditorColorField(
            key: ValueKey('color:${layer['id']}:${color['slot']}'),
            value: Color.from(
              red: rgba[0].toDouble(),
              green: rgba[1].toDouble(),
              blue: rgba[2].toDouble(),
              alpha: rgba[3].toDouble(),
            ),
            label: '${color['label']}',
            allowAlpha: panelMap(color['slot']).containsKey('TextFill'),
            enabled:
                layer['locked'] != true && panelCan(controller, 'previewColor'),
            onPreview: (v) => controller.command('previewColor', {
              'layer': layer['id'],
              'slot': color['slot'],
              'rgba': [v.r, v.g, v.b, v.a],
            }),
            onFinish: () => controller.command('commitPreview'),
            onCancel: () => controller.command('cancelPreview'),
          ),
        ),
      ],
    );
  }
}
