// The stage's artwork: a pop poster frame drawn from Ws.layers at one frame, plus the geometry the stage needs to select it.
// contract: everything here is a pure function of an [ArtScene]; no clock, no randomness at paint time, so a frame always looks the same.
import 'dart:math' as math;

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';

import 'ws.dart';

/// The poster's own inks: flat, saturated, opaque. They belong to the picture, never to the chrome.
abstract final class ArtInk {
  static const blue = Color(0xFF2A52E4), deep = Color(0xFF1D3BBE), sky = Color(0xFF4C79FF), navy = Color(0xFF0E1A4A);
  static const cream = Color(0xFFF6EFDD), orange = Color(0xFFFF7A3C), pink = Color(0xFFF06BAA), lime = Color(0xFFCBE057), ink = Color(0xFF15170D);
}

/// One layer as the picture needs it: a frozen copy, so a scene can be compared with the last one.
@immutable
class ArtLayer {
  ArtLayer(WsLayer l) : id = l.id, name = l.name, kind = l.kind, inF = l.inF, outF = l.outF, keys = List.unmodifiable([...l.keys]..sort());
  final String id, name;
  final WsKind kind;
  final int inF, outF;
  final List<int> keys;
  bool live(int f) => f >= inF && f <= outF;
  @override
  bool operator ==(Object other) =>
      other is ArtLayer && other.id == id && other.name == name && other.kind == kind && other.inF == inF && other.outF == outF && listEquals(other.keys, keys);
  @override
  int get hashCode => Object.hash(id, inF, outF, Object.hashAll(keys));
}

/// What the stage draws: the frame, the layers, and whether the camera layer frames the view.
@immutable
class ArtScene {
  final String title;

  /// The guides, in composition px: the document's own.
  final List<double> guidesX, guidesY;
  ArtScene.of(Ws ws, {required this.camera})
    : frame = ws.frame,
      title = ws.document == null ? 'CODA_main' : ws.document!.replaceFirst(RegExp(r'\.motolii$'), ''), // (joined: the document's own name)
      guidesX = [for (final (a, at) in ws.guides) if (a == 'x') at],
      guidesY = [for (final (a, at) in ws.guides) if (a == 'y') at],
      layers = List.unmodifiable([for (final l in ws.layers) ArtLayer(l)]),
      rev = ws.rev,
      boxes = Map.unmodifiable({
              for (final l in ws.layers)
                if (ws.inside!.frame(int.parse(l.id)) case final f?) l.id: ArtBox(Offset(f[0], f[1]), Size(f[2], f[3]), rot: f[4] * math.pi / 180, anchor: Offset(f[5], f[6])),
            });

  /// Each thing's frame on the picture as the inside sees it at this moment.
  final Map<String, ArtBox> boxes;

  /// How many times the joined document has been read again: a change at the same frame is another scene.
  final int rev;
  final int frame;
  final bool camera;
  final List<ArtLayer> layers;

  ArtLayer? layer(String? id) => layers.where((l) => l.id == id).firstOrNull;
  @override
  bool operator ==(Object other) =>
      other is ArtScene && other.frame == frame && other.rev == rev && other.camera == camera && listEquals(other.layers, layers);
  @override
  int get hashCode => Object.hash(frame, camera, Object.hashAll(layers));
}

/// A layer's box in document px: centre, unscaled size, scale, rotation and its anchor.
class ArtBox {
  const ArtBox(this.c, this.size, {this.scale = 1, this.rot = 0, Offset? anchor}) : anchor = anchor ?? c;
  final Offset c, anchor;
  final Size size;
  final double scale, rot;
  Offset _local(double x, double y) => c + _rot(Offset(x * size.width / 2 * scale, y * size.height / 2 * scale), rot);
  List<Offset> get corners => [_local(-1, -1), _local(1, -1), _local(1, 1), _local(-1, 1)];
  Offset get top => _local(0, -1);
  bool contains(Offset p, [double pad = 0]) {
    final q = _rot(p - c, -rot);
    return q.dx.abs() <= size.width / 2 * scale + pad && q.dy.abs() <= size.height / 2 * scale + pad;
  }
}

Offset _rot(Offset p, double a) => Offset(p.dx * math.cos(a) - p.dy * math.sin(a), p.dx * math.sin(a) + p.dy * math.cos(a));


/// The stage's picture in document px: its size, the things' frames, and what a point is on.
abstract final class WsArt {
  static Size comp = const Size(1920, 1080); // (a document's own)
  static Offset get centre => comp.center(Offset.zero);

  /// Document px to the px the viewer sees, and back (the stage's eye is the inside's: no camera of the window's own).
  static Offset toView(ArtScene s, Offset p) => p;
  static Offset toDoc(ArtScene s, Offset p) => p;

  /// Where the camera frames, in document px: the whole picture.
  static ArtBox cameraBox(ArtScene s) => ArtBox(centre, comp);

  /// The box the stage frames when [id] is selected, or null when the layer has no picture at this frame (the inside
  /// gives a frame only to what it shows: whether it is on is its, not counted again here in frames).
  static ArtBox? boxOf(ArtScene s, String? id) => s.layer(id) == null ? null : s.boxes[id];

  /// The topmost layer under [p] (document px), or null. The background is locked: a click on it means "nothing".
  static String? hit(ArtScene s, Offset p) {
    for (final l in s.layers) {   // (the thing in front, of those whose real frame the point is in; the first row is the nearest thing)
      if (s.boxes[l.id]?.contains(p) ?? false) return l.id;
    }
    return null;
  }
}
