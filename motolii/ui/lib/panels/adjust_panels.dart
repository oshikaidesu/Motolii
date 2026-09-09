import 'dart:convert';

import 'package:flutter/material.dart';

import '../session/editor_session.dart';
import '../session/read_model.dart';
import '../foundation/panel_controls.dart';
import '../foundation/metrics.dart';

class BlendPanel extends StatefulWidget {
  const BlendPanel({super.key, required this.controller});
  final EditorSession controller;
  @override
  State<BlendPanel> createState() => BlendPanelState();
}

class BlendPanelState extends State<BlendPanel> {
  Map<String, dynamic> _previews = {};
  String _target = '';
  bool _working = false;
  EditorSession get controller => widget.controller;
  static const modes = [
    'Normal',
    'Add',
    'Multiply',
    'Screen',
    'Overlay',
    'Darken',
    'Lighten',
    'ColorDodge',
    'ColorBurn',
    'HardLight',
    'SoftLight',
    'Difference',
    'Exclusion',
    'Hue',
    'Saturation',
    'Color',
    'Luminosity',
  ];
  @override
  Widget build(BuildContext context) {
    final layer = controller.activeLayer;
    final source =
        layer ?? (controller.layers.isEmpty ? null : controller.layers.first);
    final livePreviews = panelMap(source?['blendPreviews']);
    if (livePreviews.isNotEmpty) _previews = livePreviews;
    if (_previews.isEmpty)
      _previews = panelMap(controller.deskWork.value['blendPreviews']);
    final target = jsonEncode(controller.selectedIds);
    if (_target != target) {
      _target = target;
      _working = false;
    }
    final mode = layer != null && !_working
        ? '${layer['blendMode'] ?? 'Normal'}'
        : controller.deskWork.value['blend'] as String? ?? 'Normal';
    final saved = List<String>.from(
      controller.deskWork.value['blends'] as List? ?? [],
    );
    void choose(String value) => setState(() {
      _working = true;
      controller.storeDesk('blendPreviews', _previews);
      controller.storeDesk('blend', value);
    });
    return ListView(
      padding: const EdgeInsets.all(EditorMetrics.s6),
      children: [
        Wrap(
          spacing: EditorMetrics.s4,
          children: [
            panelButton(
              'Load selection',
              layer == null
                  ? null
                  : () => choose('${layer['blendMode'] ?? 'Normal'}'),
            ),
            panelButton(
              'Apply',
              layer == null || layer['kind'] == 'Camera'
                  ? null
                  : () => controller.command('setAttrs', {
                      'layers': controller.selectedIds,
                      'patch': {'blendMode': mode},
                    }),
            ),
            panelButton(
              'Save preset',
              () => setState(() {
                controller.storeDesk('blends', {...saved, mode}.toList());
              }),
            ),
          ],
        ),
        const SizedBox(height: EditorMetrics.s5),
        LayoutBuilder(
          builder: (context, box) => Wrap(
            spacing: EditorMetrics.s3,
            runSpacing: EditorMetrics.s3,
            children: [
              for (final item in modes)
                SizedBox(
                  width: (box.maxWidth - 6) / 3,
                  height: EditorMetrics.s44,
                  child: Tooltip(
                    message: item,
                    child: InkWell(
                      onTap: () => choose(item),
                      child: Column(
                        children: [
                          Expanded(
                            child: Row(
                              children: [
                                for (final rgba
                                    in (_previews[item] as List? ?? const []))
                                  Expanded(
                                    child: ColoredBox(
                                      color: Color.fromARGB(
                                        255,
                                        ((rgba[0] as num).clamp(0, 1) * 255)
                                            .round(),
                                        ((rgba[1] as num).clamp(0, 1) * 255)
                                            .round(),
                                        ((rgba[2] as num).clamp(0, 1) * 255)
                                            .round(),
                                      ),
                                      child: const SizedBox.expand(),
                                    ),
                                  ),
                              ],
                            ),
                          ),
                          panelButton(
                            item,
                            () => choose(item),
                            selected: item == mode,
                          ),
                        ],
                      ),
                    ),
                  ),
                ),
            ],
          ),
        ),
        if (saved.isNotEmpty) const SizedBox(height: EditorMetrics.s6),
        for (final item in saved)
          Row(
            children: [
              Expanded(
                child: panelButton(
                  item,
                  () => choose(item),
                  selected: item == mode,
                ),
              ),
              panelButton(
                '×',
                () => setState(() {
                  controller.storeDesk('blends', [...saved]..remove(item));
                }),
              ),
            ],
          ),
      ],
    );
  }
}
