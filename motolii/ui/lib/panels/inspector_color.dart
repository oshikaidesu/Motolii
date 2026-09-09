part of 'inspector.dart';

/// The swatch targets the existing Colors panel; the hex edits the same slot.
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
    final rgba = (color['rgba'] as List? ?? [0, 0, 0, 1])
        .map((v) => (v as num).toDouble())
        .toList();
    final hex = rgba
        .take(3)
        .map(
          (v) =>
              (v.clamp(0, 1) * 255).round().toRadixString(16).padLeft(2, '0'),
        )
        .join();
    return Row(
      children: [
        SizedBox(
          width: EditorMetrics.s70,
          child: Text(
            '${color['label']}',
            style: const TextStyle(fontSize: EditorMetrics.font),
          ),
        ),
        Tooltip(
          message: 'Choose ${color['label'] ?? 'color'}',
          child: InkWell(
            onTap:
                layer['locked'] == true || !panelCan(controller, 'focusColor')
                ? null
                : () async {
                    await controller.command('focusColor', {
                      'layer': layer['id'],
                      'slot': color['slot'],
                    });
                    controller.browserTab.value = 'Colors';
                    await controller.placePanel('Colors', 'show');
                  },
            child: Container(
              width: EditorMetrics.s32,
              height: EditorMetrics.row,
              decoration: BoxDecoration(
                color: Color.fromARGB(
                  (rgba[3].clamp(0, 1) * 255).round(),
                  (rgba[0].clamp(0, 1) * 255).round(),
                  (rgba[1].clamp(0, 1) * 255).round(),
                  (rgba[2].clamp(0, 1) * 255).round(),
                ),
                border: Border.all(color: EditorTheme.border),
              ),
            ),
          ),
        ),
        Expanded(
          child: EditorDraftField(
            value: '#$hex',
            label: '${color['label']} hex',
            enabled:
                layer['locked'] != true && panelCan(controller, 'setColor'),
            validator: (v) =>
                _parseHex(v) == null ? 'Use three or six hex digits' : null,
            onCommit: (v) => controller.command('setColor', {
              'layer': layer['id'],
              'slot': color['slot'],
              'rgba': [..._parseHex(v)!, rgba[3]],
            }),
          ),
        ),
      ],
    );
  }
}
