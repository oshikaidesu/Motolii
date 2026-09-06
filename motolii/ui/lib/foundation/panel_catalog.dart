import 'package:flutter/material.dart';

class PanelSpec {
  const PanelSpec(
    this.name,
    this.category,
    this.icon, {
    this.minWidth = 260,
    this.minHeight = 160,
    this.drawer = false,
  });
  final String name, category;
  final IconData icon;
  final double minWidth, minHeight;
  final bool drawer;
}

const deskHostSpec = PanelSpec(
  'Desk',
  'Host',
  Icons.all_inbox_outlined,
  minWidth: 240,
  minHeight: 200,
);
const panelCatalog = [
  PanelSpec(
    'Stage',
    'Work',
    Icons.movie_outlined,
    minWidth: 720,
    minHeight: 280,
  ),
  PanelSpec(
    'Timeline',
    'Work',
    Icons.view_timeline_outlined,
    minWidth: 720,
    minHeight: 180,
  ),
  PanelSpec('Inspector', 'Work', Icons.tune, minWidth: 300),
  PanelSpec('Create', 'Browse', Icons.add_box_outlined),
  PanelSpec('Media', 'Browse', Icons.perm_media_outlined),
  PanelSpec('Effects', 'Browse', Icons.auto_fix_high_outlined),
  PanelSpec('Colors', 'Browse', Icons.palette_outlined),
  PanelSpec('Depth', 'Adjust', Icons.scatter_plot_outlined, drawer: true),
  PanelSpec('Ease', 'Adjust', Icons.timeline, drawer: true),
  PanelSpec('Blend', 'Adjust', Icons.layers_outlined, drawer: true),
  PanelSpec('Notes', 'Note', Icons.dashboard_outlined),
  PanelSpec('Web', 'Note', Icons.language),
  PanelSpec('History', 'Session', Icons.history, drawer: true),
];
PanelSpec? panelSpec(String name) =>
    panelCatalog.where((p) => p.name == name).firstOrNull;
