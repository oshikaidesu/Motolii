import 'package:flutter/widgets.dart';

import '../../foundation/theme.dart';
import '../../session/editor_session.dart';

/// What a shelf may ask of the Browser panel that holds it. The panel owns
/// the frame — tabs, search, rail, grid, selection, keys — and every shelf
/// speaks to it only through this.
abstract class BrowserHost {
  EditorSession get controller;
  BuildContext get context;
  bool get mounted;

  /// Whether the native side offers an operation right now.
  bool has(String op);
  String id(Map<String, dynamic> item);

  /// The rows on screen after search and category, in order.
  List<Map<String, dynamic>> get visible;

  /// The ids picked on this shelf.
  Set<String> get selectedIds;

  /// Tile scale against the default tile: marks and type follow it.
  double get tileScale;

  /// The caption band under a tile at this scale: one definition, for the
  /// grid that reserves it and the tile that draws in it.
  double get captionHeight;

  /// One tile's width at the shelf's current size.
  double get tileWidth;

  /// Grid (0), List (1) or Thumbnails (2).
  int get viewMode;

  /// The shelf in front and its name.
  BrowserShelf get shelf;
  String get tab;

  void select(Map<String, dynamic> item);
  Future<void> apply(Map<String, dynamic> item);
  void menu(Map<String, dynamic> item, Offset point);

  /// Redraw the panel; the shelf changed something it shows.
  void refresh([VoidCallback? change]);

  /// List again: the shelf's items changed.
  void relist();

  /// Drop the pick on this shelf.
  void clearSelection();

  /// Put a shelf on a rail category; the front shelf lists again.
  void showCategory(String shelf, String category);
}

/// How one shelf lays its tiles when it does not want the standard grid.
class ShelfLayout {
  const ShelfLayout({
    required this.column,
    required this.extent,
    required this.gap,
    required this.padding,
    required this.ground,
  });
  final double column;
  final double extent;
  final double gap;
  final double padding;
  final Color ground;
}

/// One filter group (Live 12's Filter View): a name and the tags an item may
/// carry in it. An item's own tags are facts about it; the user's tags live in
/// one more group the frame adds.
class FilterGroup {
  const FilterGroup(
    this.name,
    this.tags, {
    this.kind = FilterKind.declared,
    this.unit = '',
  });
  final String name;

  /// Declared: the tags. Values: unused (the frame collects the values the
  /// items actually have). Range: seed ranges as `min-max` (either end may
  /// be empty), shown until the user keeps their own.
  final List<String> tags;
  final FilterKind kind;
  final String unit;
}

/// How a group gets its tags: declared by the shelf, collected from the
/// values the items actually carry (a frame rate, a count), or ranges the
/// user cuts on a continuous fact (a duration).
enum FilterKind { declared, actual, range }

/// One tab of the Browser: what it lists, how a tile looks, what applying
/// does. Adding a tab is adding a shelf to [browserShelves]; the panel never
/// names a tab itself.
abstract class BrowserShelf {
  String get name;

  /// Rail entries; the first is the whole shelf.
  List<String> rails(BrowserHost host);
  List<Map<String, dynamic>> items(BrowserHost host);
  String classification(BrowserHost host, Map<String, dynamic> item);

  /// Whether the rail entry keeps [item]; the default is a category match.
  bool passes(BrowserHost host, Map<String, dynamic> item, String chosen) =>
      chosen == rails(host).first || classification(host, item) == chosen;

  /// The rail entry pressed; the default narrows the list to it.
  void rail(BrowserHost host, String entry) => host.showCategory(name, entry);

  /// The rail entry in force.
  String chosen(BrowserHost host, String category) => category;

  /// Shift and ⌘ extend the pick.
  bool get multiSelect => false;

  /// Tiles are the picture alone: a click applies, no caption, no ground.
  bool get bare => false;

  /// A bare tile applies on a click; an item whose apply makes something new
  /// (a layer) asks for a double-click instead, so a stray click adds nothing.
  bool doubleClick(BrowserHost host, Map<String, dynamic> item) => false;

  /// Grid / List / Thumbnails beside the search field.
  bool get showViews => true;

  /// The view the shelf opens in when the desk says nothing.
  int get defaultView => 0;

  ShelfLayout? layout(BrowserHost host, double width, double tile) => null;

  /// Signals the panel's document slice derives; a change redraws the shelf.
  List<Object?> derived(EditorSession controller) => const [];

  /// Desk keys the shelf reads; a write to any other leaves it still.
  List<String> get deskKeys => const [];

  /// The filter groups this shelf's items carry (empty: no Filter View).
  List<FilterGroup> groups(BrowserHost host) => const [];

  /// The item's own tags, as `group/tag` pairs are not needed: a tag name is
  /// unique within its group and the frame asks group by group.
  Set<String> tagsOf(BrowserHost host, Map<String, dynamic> item) => const {};

  /// A values group's tag for the item (null: none), a range group's number.
  String? valueOf(BrowserHost host, Map<String, dynamic> item, String group) =>
      null;
  double? numberOf(BrowserHost host, Map<String, dynamic> item, String group) =>
      null;

  /// Called when the shelf comes to the front.
  void enter(BrowserHost host) {}
  void dispose() {}

  /// Controls beside the search field.
  List<Widget> tools(BrowserHost host) => const [];

  /// A row under the search field, the panel's whole width (a path).
  Widget? header(BrowserHost host) => null;

  /// Above the grid, beside the rail (an editor).
  Widget? editor(BrowserHost host) => null;

  bool supported(BrowserHost host, Map<String, dynamic> item);

  /// The colour that stands for the item: its family or its id.
  String identity(BrowserHost host, Map<String, dynamic> item) => host.id(item);

  /// The badge on the caption's right; empty for none.
  String format(BrowserHost host, Map<String, dynamic> item) => '';
  Widget preview(BrowserHost host, Map<String, dynamic> item, Color identity);
  Future<void> apply(BrowserHost host, Map<String, dynamic> item);
  String applyLabel(BrowserHost host, Map<String, dynamic> item) => 'Apply';

  /// Quiet rows at the top of the menu: what the tile is.
  List<String> facts(BrowserHost host, Map<String, dynamic> item) => [
    if (item['detail'] != null) '${item['detail']}',
  ];

  /// Menu rows after Apply; [act] receives the value chosen.
  List<EditorMenuItem<String>> menu(
    BrowserHost host,
    Map<String, dynamic> item,
  ) => const [];
  Future<void> act(
    BrowserHost host,
    String action,
    Map<String, dynamic> item,
  ) async {}

  /// Wraps the tile for dragging out; the default is no drag.
  Widget draggable(BrowserHost host, Map<String, dynamic> item, Widget tile) =>
      tile;

  /// Over the whole panel (a drop hint).
  Widget? overlay(BrowserHost host) => null;

  /// Delete on this shelf.
  void delete(BrowserHost host, Map<String, dynamic> item) {}
}
