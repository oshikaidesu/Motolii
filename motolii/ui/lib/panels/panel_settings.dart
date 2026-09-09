import 'package:flutter/material.dart';

import '../session/editor_session.dart';
import '../foundation/panel_catalog.dart';
import 'browser.dart';
import '../foundation/panel_controls.dart';
import '../foundation/theme.dart';
import '../foundation/metrics.dart';

class PanelSettings extends StatelessWidget {
  const PanelSettings({super.key, required this.controller});
  final EditorSession controller;
  @override
  Widget build(BuildContext context) => AnimatedBuilder(
    animation: Listenable.merge([controller.panePlaces, controller.deskWork]),
    builder: (context, _) => ListView(
      padding: const EdgeInsets.all(EditorMetrics.s8),
      children: [
        const Text(
          'Browser',
          style: TextStyle(
            fontWeight: FontWeight.w600,
            fontSize: EditorMetrics.s12,
          ),
        ),
        Row(
          children: [
            const Text(
              'Tile size',
              style: TextStyle(fontSize: EditorMetrics.font),
            ),
            Expanded(
              child: EditorZoomBar(
                keyPrefix: 'settings:browserTile',
                base: BrowserSize.base,
                min: BrowserSize.min,
                max: BrowserSize.max,
                value: BrowserSize.tile(controller),
                onChanged: (v) => controller.storeDesk('browserTile', v),
              ),
            ),
          ],
        ),
        const SizedBox(height: EditorMetrics.s6),
        const Text(
          'Panels',
          style: TextStyle(
            fontWeight: FontWeight.w600,
            fontSize: EditorMetrics.s12,
          ),
        ),
        const SizedBox(height: EditorMetrics.s6),
        for (final spec in panelCatalog)
          SizedBox(
            height: EditorMetrics.s34,
            child: Row(
              children: [
                Icon(spec.icon, size: EditorMetrics.s19),
                const SizedBox(width: EditorMetrics.s8),
                Expanded(
                  child: Text(
                    spec.name,
                    style: const TextStyle(fontSize: EditorMetrics.font),
                  ),
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
                    child: EditorIconButton(
                      key: ValueKey('placement:${spec.name}:$place'),
                      constraints: const BoxConstraints.tightFor(
                        width: EditorMetrics.s32,
                        height: EditorMetrics.tall,
                      ),
                      padding: EdgeInsets.zero,
                      iconSize: EditorMetrics.s17,
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
