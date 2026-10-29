// The stage's viewport: the composition centred on a dark surround, rulers and guides, overlays, the selection, the zoom HUD and the navigator.
// contract: paints only when the scene, the selection, the hover or the view changes; the pointer readout repaints the rulers alone.
import 'dart:math' as math;
import 'dart:ui' show PointMode;

import 'package:flutter/foundation.dart' show listEquals;
import 'package:flutter/gestures.dart';
import 'package:flutter/scheduler.dart' show SchedulerBinding;
import 'package:flutter/services.dart' show HardwareKeyboard;
import 'package:flutter/widgets.dart';

import '../tokens.dart';
import 'stage_art.dart';
import 'two_fingers.dart';
import '../motolii.dart' show Motolii, MotoliiDocument, MotoliiMoment, MotoliiWrites, MotoliiQuestions, MotoliiShelves, MotoliiWatch;
import '../sets/panels/timeline_parts_parts.dart' show TpGlyph, TpGlyphView;
import '../sets/inspector/inspector_material.dart' show textToType;
import 'relate.dart';
import 'package:perfect_freehand/perfect_freehand.dart' show getStrokeOutlinePoints, getStrokePoints, PointVector, StrokeOptions;
import 'package:perfect_freehand/src/get_stroke_radius.dart' show getStrokeRadius;   // (how wide its pressure makes a point: kept private by the package, read as it is)
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
    this.brushSize = 16,
    this.brushTool = 1,
  });

  /// Joined: how wide the Brush draws, in points of the picture (perfect-freehand's size).
  final double brushSize;
  final int brushTool;   // (AbsentMotion's: 0 a pen, 1 a brush, 2 a fill: doc_brush)

  /// Joined: the Move tool's way (0 free, 1 along x only, 2 along y only), and whether an ellipse is drawn out from its middle.
  final int axis;
  final bool fromCentre;

  /// Joined: a thing carried sticks to the lines of the others, of the picture and of the guides (snap); every thing's
  /// point it turns about is shown (pivots); a thing taken hold of is the one it goes with, outermost (group); what is
  /// drawn cuts the picked thing (mask: it lies just behind it, and that one is clipped by it).
  final bool snap, pivots, group, mask;

  /// Joined: what the tool in hand makes where the hand is drawn across the stage (rect, circle, line, text, brush; null:
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
/// A shape's own too (Alight Motion's, on the canvas): its corners' roundness, in from its first corner along its
/// diagonal as far as they are round (round: a share of its shorter side); a polygon's or star's points, out from the
/// middle of its right side.
/// A photo's crop (its left, top, right and bottom cut in, shares of its width or height): a bar on each side where it is cut.
Map<String, Offset> _handles(Offset Function(Offset doc) scr, ArtBox b, {required bool knob, String form = '', double round = 0, List<double> crop = const []}) {
  final top = scr(b.top), dir = top - scr(b.c), n = dir.distance < 1 ? const Offset(0, -1) : dir / dir.distance;
  final c = [for (final p in b.corners) scr(p)];
  return {
    for (final (i, p) in c.indexed) 'corner $i': p,
    if (knob) 'turn': top + n * _V.stem,
    'anchor': scr(b.anchor),
    'body': scr(b.c),
    if (const ['rect', 'polygon', 'star', 'squircle', 'chamfer', 'cog'].contains(form)) 'round': _shapeGrip(c).roundAt(round),
    if (const ['polygon', 'star', 'cog'].contains(form)) 'points': (c[1] + c[2]) / 2 + _shapeGrip(c).u * 18,
    if (crop.length == 4) ...{
      'crop 0': (c[0] + c[3]) / 2 + (c[1] - c[0]) * crop[0],
      'crop 1': (c[0] + c[1]) / 2 + (c[3] - c[0]) * crop[1],
      'crop 2': (c[1] + c[2]) / 2 - (c[1] - c[0]) * crop[2],
      'crop 3': (c[3] + c[2]) / 2 - (c[3] - c[0]) * crop[3],
    },
  };
}

/// Where a shape's own handles are reckoned from: its first corner, the way along its top and down its left side, and
/// its shorter side, in points of the window.
({Offset c0, Offset u, Offset w, double short}) _gripOf(List<Offset> c) {
  final top = c[1] - c[0], side = c[3] - c[0];
  return (c0: c[0], u: top.distance < 1e-6 ? const Offset(1, 0) : top / top.distance, w: side.distance < 1e-6 ? const Offset(0, 1) : side / side.distance, short: math.max(1.0, math.min(top.distance, side.distance)));
}

extension on ({Offset c0, Offset u, Offset w, double short}) {
  /// The roundness handle: in along the diagonal as far as the corners are round, never on the corner itself.
  Offset roundAt(double round) => c0 + (u + w) * math.max(round.clamp(0.0, .5) * short, 12.0);
  /// How round a hand at `at` makes the corners: how far in along the diagonal, a share of the shorter side.
  double roundOf(Offset at) => (((at - c0).dx * (u + w).dx + (at - c0).dy * (u + w).dy) / 2 / short).clamp(0.0, .5);
}
({Offset c0, Offset u, Offset w, double short}) _shapeGrip(List<Offset> c) => _gripOf(c);
