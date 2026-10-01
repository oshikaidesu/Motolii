// The seam between the finished Browser bodies and a host that owns real items and actions.
// The bodies draw; the seat says what is picked, what a tile does, how it is drawn when the host has a better
// picture, and what the panel's keys do. With no seat in scope the bodies behave as they always did (fixtures).
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'things.dart';

abstract class BrowserSeat {
  /// A tile with its behaviour: pick, apply, menu, drag, tooltip, and the frame that says it is picked.
  Widget tile(BuildContext context, Thing thing, Widget tile);

  /// The host's own picture for a thing (a native snapshot), or null to draw the declared face.
  Widget? face(BuildContext context, Thing thing) => null;

  /// The host's own idea of the tiling (Colors' small swatches, Fonts' one row per family), or null for the default.
  ({double column, double extent, double gap, double padding})? tiling(BuildContext context, double width) => null;

  /// Tile size against the default: marks and type follow it.
  double get tileScale => 1;

  /// Controls the host's shelf puts beside search (Import, Reload, From image).
  Widget? tools(BuildContext context) => null;

  /// A row under the header that spans the panel (a folder path).
  Widget? header(BuildContext context) => null;

  /// The shelf's editor (the colour wheel, the font scope) in the place the body keeps for its instrument.
  Widget? editor(BuildContext context) => null;

  /// The keyboard of the panel: find, clear, arrows, Enter, Delete. [KeyEventResult.ignored] leaves the key alone.
  KeyEventResult key(FocusNode node, KeyEvent event) => KeyEventResult.ignored;

  /// A body reports what it shows, in order, and how many columns it lays them in, so the arrows can move.
  void shows(List<Thing> things, int columns) {}

  /// The header's overflow key: what the host keeps for the user (saved searches, recent).
  void more(BuildContext context, Offset at) {}

  /// The user's own views (favorites, recent, saved searches, collections) the class column lists.
  UserViews? get user => null;

  /// Something the seat wants the panel to hear (the pick moved).
  Listenable get changes;
}

class BrowserSeatScope extends InheritedWidget {
  const BrowserSeatScope({super.key, required this.seat, required super.child});
  final BrowserSeat seat;
  static BrowserSeat? of(BuildContext context) => context.getInheritedWidgetOfExactType<BrowserSeatScope>()?.seat;
  @override
  bool updateShouldNotify(BrowserSeatScope old) => old.seat != seat;
}

/// A tile as the seat wants it, when there is a seat.
Widget seated(BuildContext context, Thing thing, Widget tile) => BrowserSeatScope.of(context)?.tile(context, thing, tile) ?? tile;
