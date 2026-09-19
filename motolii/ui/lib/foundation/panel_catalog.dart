import 'package:flutter/widgets.dart';

import 'glyphs.dart';

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
  Glyph.all_inbox_outlined,
  minWidth: 240,
  minHeight: 200,
  width: Extent.any,
  height: Extent.fixed(200),
);
const panelCatalog = [
  PanelSpec(
    'Stage',
    'Work',
    Glyph.movie_outlined,
    minWidth: 720,
    minHeight: 280,
    width: Extent.fill,
    height: Extent.fill,
  ),
  PanelSpec(
    'Camera',
    'Work',
    Glyph.videocam_outlined,
    minWidth: 720,
    minHeight: 280,
    width: Extent.fill,
    height: Extent.fill,
  ),
  PanelSpec(
    'Timeline',
    'Work',
    Glyph.view_timeline_outlined,
    minWidth: 720,
    minHeight: 180,
    width: Extent.fill,
    height: Extent.fill,
  ),
  PanelSpec(
    'Inspector',
    'Work',
    Glyph.tune,
    minWidth: 300,
    width: Extent.fixed(300),
  ),
  PanelSpec('Create', 'Browse', Glyph.add_box_outlined),
  PanelSpec('Media', 'Browse', Glyph.perm_media_outlined),
  PanelSpec('Effects', 'Browse', Glyph.auto_fix_high_outlined),
  PanelSpec('Fonts', 'Browse', Glyph.font_download_outlined),
  PanelSpec('Colors', 'Browse', Glyph.palette_outlined),
  PanelSpec('Files', 'Browse', Glyph.folder_outlined),
  PanelSpec(
    'Depth',
    'Adjust',
    Glyph.scatter_plot_outlined,
    width: Extent.fixed(300),
    drawer: true,
  ),
  PanelSpec(
    'Ease',
    'Adjust',
    Glyph.timeline,
    width: Extent.fixed(300),
    drawer: true,
  ),
  PanelSpec(
    'Blend',
    'Adjust',
    Glyph.layers_outlined,
    width: Extent.fixed(300),
    drawer: true,
  ),
  PanelSpec(
    'Notes',
    'Note',
    Glyph.dashboard_outlined,
    width: Extent.fill,
    height: Extent.fill,
  ),
  PanelSpec(
    'Web',
    'Note',
    Glyph.language,
    width: Extent.fill,
    height: Extent.fill,
  ),
  PanelSpec(
    'History',
    'Session',
    Glyph.history,
    width: Extent.fixed(300),
    drawer: true,
  ),
];
PanelSpec? panelSpec(String name) =>
    panelCatalog.where((p) => p.name == name).firstOrNull;
