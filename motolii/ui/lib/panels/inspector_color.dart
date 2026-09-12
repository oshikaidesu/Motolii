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
    // The colour itself is the result: a wide swatch leads, its word sits
    // above in the smallest type, and the hex follows for typing.
    return Row(
      children: [
        SizedBox(
          width: EditorMetrics.s48,
          child: Text(
            '${color['label']}',
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
            style: const TextStyle(
              fontSize: EditorMetrics.micro,
              color: EditorTheme.muted,
            ),
          ),
        ),
        EditorTooltip(
          message: 'Choose ${color['label'] ?? 'color'}',
          child: InkWell(
            onTap:
                layer['locked'] == true || !panelCan(controller, 'focusColor')
                ? null
                : () => controller.focusColor({
                      'layer': layer['id'],
                      'slot': color['slot'],
                    }),
            child: Container(
              width: EditorMetrics.s70,
              height: EditorMetrics.control,
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
        const SizedBox(width: EditorMetrics.s6),
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
