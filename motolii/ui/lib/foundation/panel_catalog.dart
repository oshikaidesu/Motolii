import 'package:flutter/material.dart';

/// How much of a dock axis a panel asks for: a fixed number of pixels, all
/// that is left (`fill`), or whatever its neighbours decide (`any`).
class Extent {
  const Extent.fixed(this.px);
  const Extent._(this.px);
  static const fill = Extent._(double.infinity);
  static const any = Extent._(0);
  final double px;
  bool get isFill => px == double.infinity;
  bool get isFixed => px > 0 && !isFill;
}

class PanelSpec {
  const PanelSpec(
    this.name,
    this.category,
    this.icon, {
    this.minWidth = 260,
    this.minHeight = 160,
    this.width = const Extent.fixed(260),
    this.height = Extent.any,
    this.drawer = false,
  });
  final String name, category;
  final IconData icon;
  final double minWidth, minHeight;

  /// Docked size along each axis; detached windows use [minWidth]/[minHeight].
  final Extent width, height;
  final bool drawer;
  Extent extent(Axis axis) => axis == Axis.horizontal ? width : height;
}

const deskHostSpec = PanelSpec(
  'Desk',
  'Host',
  Icons.all_inbox_outlined,
  minWidth: 240,
  minHeight: 200,
  width: Extent.any,
  height: Extent.fixed(200),
);
const panelCatalog = [
  PanelSpec(
    'Stage',
    'Work',
    Icons.movie_outlined,
    minWidth: 720,
    minHeight: 280,
    width: Extent.fill,
    height: Extent.fill,
  ),
  PanelSpec(
    'Timeline',
    'Work',
    Icons.view_timeline_outlined,
    minWidth: 720,
    minHeight: 180,
    width: Extent.fill,
    height: Extent.fill,
  ),
  PanelSpec(
    'Inspector',
    'Work',
    Icons.tune,
    minWidth: 300,
    width: Extent.fixed(300),
  ),
  PanelSpec('Create', 'Browse', Icons.add_box_outlined),
  PanelSpec('Media', 'Browse', Icons.perm_media_outlined),
  PanelSpec('Effects', 'Browse', Icons.auto_fix_high_outlined),
  PanelSpec('Fonts', 'Browse', Icons.font_download_outlined),
  PanelSpec('Colors', 'Browse', Icons.palette_outlined),
  PanelSpec('Files', 'Browse', Icons.folder_outlined),
  PanelSpec(
    'Depth',
    'Adjust',
    Icons.scatter_plot_outlined,
    width: Extent.fixed(300),
    drawer: true,
  ),
  PanelSpec(
    'Ease',
    'Adjust',
    Icons.timeline,
    width: Extent.fixed(300),
    drawer: true,
  ),
  PanelSpec(
    'Blend',
    'Adjust',
    Icons.layers_outlined,
    width: Extent.fixed(300),
    drawer: true,
  ),
  PanelSpec(
    'Notes',
    'Note',
    Icons.dashboard_outlined,
    width: Extent.fill,
    height: Extent.fill,
  ),
  PanelSpec(
    'Web',
    'Note',
    Icons.language,
    width: Extent.fill,
    height: Extent.fill,
  ),
  PanelSpec(
    'History',
    'Session',
    Icons.history,
    width: Extent.fixed(300),
    drawer: true,
  ),
];
PanelSpec? panelSpec(String name) =>
    panelCatalog.where((p) => p.name == name).firstOrNull;
