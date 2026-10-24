// The stage's viewport: the composition centred on a dark surround, rulers and guides, overlays, the selection, the zoom HUD and the navigator.
// contract: paints only when the scene, the selection, the hover or the view changes; the pointer readout repaints the rulers alone.
import 'dart:math' as math;

import 'package:flutter/foundation.dart' show listEquals;
import 'package:flutter/gestures.dart';
import 'package:flutter/scheduler.dart' show SchedulerBinding;
import 'package:flutter/services.dart' show HardwareKeyboard;
import 'package:flutter/widgets.dart';

import '../tokens.dart';
import 'stage_art.dart';
import '../motolii.dart' show Motolii, MotoliiReads;
import '../sets/panels/timeline_parts_parts.dart' show TpGlyph, TpGlyphView;
import 'relate.dart';
import 'ws.dart';
part 'stage_view_2.dart';
part 'stage_view_3.dart';
part 'stage_view_4.dart';

/// Zoom steps of the header chip: fit, then fixed scales.
const stageZooms = ['Fit', '50%', '100%'];
const _fixed = [0.0, .5, 1.0];

/// Viewport sizes: the ruler band, the margin a fit leaves around the composition, the HUD pill and the navigator.
abstract final class _V {
  static double get ruler => Shape.z(16.0);
  static double get fitMargin => Shape.s(28.0);
  static double get hud => Shape.z(22.0);
  static double get mapW => Shape.z(136.0);
  static double get pad => Shape.s(8.0);
  static double get handle => Shape.z(7.0);
  static double get stem => Shape.z(18.0);
  static double get tagH => Shape.z(16.0);
  static Color get surround => Grey.g10;
  static Color get rulerGround => Grey.g13;
  static Color get rulerSpan => Grey.g15;
  static Color get tick => Grey.g38;
  static Color get guide => WsT.toneAudio;
  static const overlay = Color(0x40FFFFFF), safe = Color(0x99F6EFDD);
}


/// Composition px to screen px: the composition's top-left at [o], [s] screen px per composition px, inside [area].
@immutable
class _View {
  const _View(this.o, this.s, this.area);
  final Offset o;
  final double s;
  final Rect area;
  Rect get comp => o & WsArt.comp * s;
  Offset toScreen(Offset p) => o + p * s;
  Offset toComp(Offset p) => (p - o) / s;
  @override
  bool operator ==(Object x) => x is _View && x.o == o && x.s == s && x.area == area;
  @override
  int get hashCode => Object.hash(o, s, area);
}

class StageView extends StatefulWidget {
  const StageView({
    super.key,
    required this.scene,
    required this.selected,
    required this.zoom,
    required this.grid,
    required this.safe,
    required this.cursor,
    this.onPoke,
    this.texture,
    this.make,
    this.axis = 0,
    this.fromCentre = false,
    this.snap = false,
    this.pivots = false,
    this.group = false,
    this.mask = false,
  });

  /// Joined: the Move tool's way (0 free, 1 along x only, 2 along y only), and whether an ellipse is drawn out from its middle.
  final int axis;
  final bool fromCentre;

  /// Joined: a thing carried sticks to the lines of the others, of the picture and of the guides (snap); every thing's
  /// point it turns about is shown (pivots); a thing taken hold of is the one it goes with, outermost (group); what is
  /// drawn cuts the picked thing (mask: it lies just behind it, and that one is clipped by it).
  final bool snap, pivots, group, mask;

  /// Joined: what the tool in hand makes where the hand is drawn across the stage (rect, circle, line, text; null:
  /// the hand takes hold of things).
  final String? make;

  /// The inside's picture of the stage (a Flutter texture). null: the stand-in art is painted.
  final int? texture;
  final ArtScene scene;
  final String? selected;
  final int zoom;
  final bool grid, safe;

  /// The pointer in composition px while it is over the viewport; the HUD and the rulers read it.
  final ValueNotifier<Offset?> cursor;
  final VoidCallback? onPoke;
  @override
  State<StageView> createState() => _StageViewState();
}

/// The picked thing's handles where the stage draws them now, by name, in the stage's own coordinates: what a hand
/// (or a test, as tldraw's and Excalidraw's do) presses to take hold of each.
abstract interface class StageHandles {
  Map<String, Offset> get handles;
}

/// The selection's handles where they are drawn, by name: its four corners, the knob it turns by, the point it turns
/// about and its middle (the body). The painter draws them from here; [StageHandles] hands them out from here.
Map<String, Offset> _handles(Offset Function(Offset doc) scr, ArtBox b, {required bool knob}) {
  final top = scr(b.top), dir = top - scr(b.c), n = dir.distance < 1 ? const Offset(0, -1) : dir / dir.distance;
  return {
    for (final (i, p) in b.corners.indexed) 'corner $i': scr(p),
    if (knob) 'turn': top + n * _V.stem,
    'anchor': scr(b.anchor),
    'body': scr(b.c),
  };
}
