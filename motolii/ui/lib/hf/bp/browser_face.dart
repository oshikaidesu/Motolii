// The Browser seat of the reference: four panels sharing one seat under a tab strip. What they list comes from a
// [BrowserModel]: proto_hf hands it fixture catalogues, production hands it the session's shelves.
import 'package:flutter/widgets.dart';
import 'colors.dart';
import 'common.dart';
import 'create.dart';
import 'effects.dart';
import 'fonts.dart';
import 'things.dart';

const browserTabs = [tabCreate, tabEffects, tabColors, tabFonts];

class BrowserModel {
  const BrowserModel({required this.catalog, required this.tab, this.user, this.scene, this.colors, this.fonts = fontsBase, this.onTab});
  final Catalog catalog;
  final int tab;
  final UserViews? user;
  final EffectScene? scene;

  /// Null: the panel's own palette.
  final List<Sw>? colors;
  final List<FontItem> fonts;
  final ValueChanged<int>? onTab;
}

Widget browserFace(BrowserModel m) => Leaf(
      tabs: browserTabs,
      active: m.tab,
      onTab: m.onTab,
      body: KeyedSubtree(
        key: ObjectKey(m.catalog),
        child: switch (m.tab) {
          0 => CreatePanel(catalog: m.catalog, user: m.user, scene: m.scene),
          1 => EffectsPanel(m.scene, catalog: m.catalog, user: m.user),
          2 => ColorsPanel(items: m.colors),
          _ => FontsPanel(items: m.fonts),
        },
      ),
    );
