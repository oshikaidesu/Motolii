import 'package:flutter/material.dart';

import '../session/editor_session.dart';
import '../foundation/panel_catalog.dart';
import '../foundation/theme.dart';

class PanelSettings extends StatelessWidget {
  const PanelSettings({super.key, required this.controller});
  final EditorSession controller;
  @override
  Widget build(BuildContext context) => AnimatedBuilder(
    animation: controller.panePlaces,
    builder: (context, _) => ListView(
      padding: const EdgeInsets.all(8),
      children: [
        const Text(
          'Panels',
          style: TextStyle(fontWeight: FontWeight.w600, fontSize: 12),
        ),
        const SizedBox(height: 6),
        for (final spec in panelCatalog)
          SizedBox(
            height: 34,
            child: Row(
              children: [
                Icon(spec.icon, size: 19),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(spec.name, style: const TextStyle(fontSize: 11)),
                ),
                for (final place in [
                  if (spec.drawer) 'drawer',
                  'tab',
                  'window',
                  'hidden',
                ])
                  Tooltip(
                    message: switch (place) {
                      'drawer' => 'Desk',
                      'tab' => 'Tab',
                      'window' => 'Window',
                      _ => 'Hidden',
                    },
                    child: IconButton(
                      key: ValueKey('placement:${spec.name}:$place'),
                      constraints: const BoxConstraints.tightFor(
                        width: 32,
                        height: 30,
                      ),
                      padding: EdgeInsets.zero,
                      iconSize: 17,
                      isSelected:
                          (controller.panePlaces.value[spec.name] ??
                              (spec.drawer ? 'drawer' : 'hidden')) ==
                          place,
                      color:
                          (controller.panePlaces.value[spec.name] ??
                                  (spec.drawer ? 'drawer' : 'hidden')) ==
                              place
                          ? EditorTheme.accent
                          : EditorTheme.muted,
                      onPressed: () => controller.placePanel(spec.name, place),
                      icon: Icon(switch (place) {
                        'drawer' => Icons.all_inbox_outlined,
                        'tab' => Icons.tab,
                        'window' => Icons.open_in_new,
                        _ => Icons.visibility_off_outlined,
                      }),
                    ),
                  ),
              ],
            ),
          ),
      ],
    ),
  );
}
