// The Browser seat of the reference: four panels sharing one seat under a tab strip. What they list comes from a
// [BrowserModel]: proto_hf hands it fixture catalogues, production hands it the session's shelves.
import 'package:flutter/widgets.dart';

import '../colors/shelf.dart';
import 'parts.dart';
import 'create/shelf.dart';
import '../effects/shelf.dart';
import '../fonts/shelf.dart';
import 'things.dart';
import '../session/editor_session.dart';

const browserTabs = [tabCreate, tabEffects, tabColors, tabFonts, tabMedia];

class BrowserModel {
  const BrowserModel({
    required this.catalog,
    required this.tab,
    this.user,
    this.scene,
    this.colors,
    this.gradients,
    this.onColor,
    this.onGradient,
    this.colorEditor,
    this.currentColor,
    this.onColorMenu,
    this.usedFonts = const {},
    this.onGradientMenu,
    this.fonts = fontsBase,
    this.fontGroups,
    this.selectedFont,
    this.onFont,
    this.onFontCreate,
    this.onFontFavorite,
    this.fontController,
    this.media,
    this.tabs = browserTabs,
    this.onTab,
  });
  final Catalog catalog;
  final int tab;
  final UserViews? user;
  final EffectScene? scene;

  /// Null: the panel's own palette.
  final List<Sw>? colors;
  final List<Map<String, dynamic>>? gradients;
  final ValueChanged<Sw>? onColor;
  final ValueChanged<Map<String, dynamic>>? onGradient;
  final Widget Function(double wheel)? colorEditor;
  final Color? currentColor;
  final void Function(Sw swatch, Offset at)? onColorMenu;
  final Set<String> usedFonts;
  final void Function(Map<String, dynamic> gradient, Offset at)? onGradientMenu;
  final List<FontItem> fonts;
  final List<List<String>>? fontGroups;
  final String? selectedFont;
  final ValueChanged<FontItem>? onFont, onFontCreate;
  final ValueChanged<FontItem>? onFontFavorite;
  final EditorSession? fontController;
  final Widget? media;
  final List<TabSpec> tabs;
  final ValueChanged<int>? onTab;
}

/// All five tabs: the Browser's own seat (its Leaf strip). One tab: a panel sitting in a Dock seat, whose Leaf strip is
/// the seat's — so the body comes without a strip of its own.
Widget browserFace(BrowserModel m) => m.tabs.length == 1 ? _body(m) : Leaf(tabs: m.tabs, active: m.tab, onTab: m.onTab, body: _body(m));

Widget _body(BrowserModel m) => KeyedSubtree(
    key: ValueKey('browser-tab-${m.tab}'),
    child: switch (m.tab) {
      0 => CreatePanel(catalog: m.catalog, user: m.user, scene: m.scene),
      1 => EffectsPanel(m.scene, catalog: m.catalog, user: m.user),
      2 => ColorsPanel(
        items: m.colors,
        gradients: m.gradients,
        onSwatch: m.onColor,
        onGradient: m.onGradient,
        editor: m.colorEditor,
        current: m.currentColor,
        onSwatchMenu: m.onColorMenu,
        onGradientMenu: m.onGradientMenu,
        controller: m.fontController,
      ),
      3 => FontsPanel(
        items: m.fonts,
        groups: m.fontGroups,
        user: m.user,
        selectedFamily: m.selectedFont,
        onSelect: m.onFont,
        onCreate: m.onFontCreate,
        onFavorite: m.onFontFavorite,
        controller: m.fontController,
        used: m.usedFonts,
      ),
      _ => m.media ?? const SizedBox.shrink(),
    },
);
