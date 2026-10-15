// Where everything sits on a canvas of [size] for the layer values [values], seen from the front as in 2D.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_direct_transform.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart';

/// Where everything sits on a canvas of [size] for the layer values [values], seen from the front as in 2D.
/// Every space edits the same values with the same handles; outside 2D nearer layers draw larger, and 3D shows Depth as thickness.
class TransformLayout {
  TransformLayout(this.size, this.values, {this.space = Space.d2});
  final Size size;
  final Map<String, double> values;
  final Space space;

  bool get deep => space != Space.d2;
  bool get solid => space == Space.d3;

  /// Every value the canvas edits; any of them mixed makes it inert.
  List<String> get ids => ['pos.x', 'pos.y', 'scale.x', 'scale.y', 'rot.z', 'anchor.x', 'anchor.y', 'pos.z', 'rot.x', 'rot.y', 'depth'];

  double value(String id) => values[id] ?? 0;
  double get angle => value('rot.z') * math.pi / 180;
  double get tiltX => value('rot.x') * math.pi / 180;
  double get tiltY => value('rot.y') * math.pi / 180;
  double get z => value('pos.z');
  Rect get frame {
    final w = math.min(size.width - 24, (size.height - 24) * 1920 / 1080);
    return Rect.fromCenter(center: size.center(Offset.zero), width: w, height: w * 1080 / 1920);
  }

  double get compositionFactor => frame.width / 1920;

  // camera: a composition-wide distance, so Z -1000 reads clearly bigger without leaving the frame
  static const _camera = 2400.0;

  /// Held to the camera (2D), Z orders the layer but never changes its size.
  double depthScale(double zc) => deep ? _camera / math.max(300, _camera + zc) : 1;
  Offset _project(Offset flat, double zc) => frame.center + (flat - frame.center) * depthScale(zc);

  Offset get half => Offset(DirectSurface.objectWidth / 2 * value('scale.x') / 100, DirectSurface.objectHeight / 2 * value('scale.y') / 100);
  Offset get anchor => Offset((value('anchor.x') / 50 - 1) * half.dx, (value('anchor.y') / 50 - 1) * half.dy);
  Offset get flatPivot => frame.topLeft + Offset(value('pos.x'), value('pos.y')) * compositionFactor;
  Offset get pivot => _project(flatPivot, z);

  /// A point of the layer, in canvas units at scale around the layer centre, pushed [back] units away from the viewer.
  Offset world(Offset p, [double back = 0]) {
    var (x, y, d) = (p.dx - anchor.dx, p.dy - anchor.dy, back);
    (y, d) = (y * math.cos(tiltX) - d * math.sin(tiltX), y * math.sin(tiltX) + d * math.cos(tiltX));
    (x, d) = (x * math.cos(tiltY) + d * math.sin(tiltY), -x * math.sin(tiltY) + d * math.cos(tiltY));
    return _project(flatPivot + rotate(Offset(x, y), angle), z + d / compositionFactor);
  }

  Offset get center => world(Offset.zero);
  Offset local(Offset p) => rotate((p - pivot) / depthScale(z), -angle) + anchor;

  static const _handlePoints = [Offset(-1, -1), Offset(1, -1), Offset(1, 1), Offset(-1, 1), Offset(-1, 0), Offset(1, 0), Offset(0, -1), Offset(0, 1)];
  List<Offset> get handles => [for (final p in _handlePoints) world(Offset(p.dx * half.dx, p.dy * half.dy))];
  List<Offset> get corners => handles.take(4).toList();
  Offset get stemBase => world(Offset(0, -half.dy.abs()));
  Offset get knob => world(Offset(0, -half.dy.abs() - DirectSurface.stem));

  /// Tilt knobs: below the layer tilts it about X, left of it about Y.
  Offset tiltBase(int axis) => axis == 1 ? world(Offset(0, half.dy.abs())) : world(Offset(-half.dx.abs(), 0));
  Offset tiltKnob(int axis) => axis == 1 ? world(Offset(0, half.dy.abs() + DirectSurface.tiltStem)) : world(Offset(-half.dx.abs() - DirectSurface.tiltStem, 0));

  /// The Z rail: near (negative Z) at the top, far at the bottom.
  double get railX => size.width - DirectSurface.rail;
  double get railHalf => size.height / 2 - 16;
  Offset get liftKnob => Offset(railX, size.height / 2 + (z / DirectSurface.zReach).clamp(-1, 1) * railHalf);

  bool body(Offset p) => (Path()..addPolygon(corners, true)).contains(p);

  /// The zone under [p] (with the handle index for scale and rotation), front to back.
  (TransformZone, int)? pick(Offset p) {
    if ((p - knob).distance <= 8) return (TransformZone.rotate, 0);
    if ((p - liftKnob).distance <= 8) return (TransformZone.lift, 0);
    for (final axis in const [1, 2]) {
      if ((p - tiltKnob(axis)).distance <= 7) {
        return (TransformZone.rotate, axis);
      }
    }
    if ((p - pivot).distance <= 7) return (TransformZone.pivot, 0);
    for (final (i, h) in handles.indexed) {
      if ((p - h).distance <= 7) return (TransformZone.scale, i);
    }
    if (body(p)) return (TransformZone.move, 0);
    return null;
  }

  /// New values for a drag of [zone] from [last] to [now]; [down] is where it started on this layout.
  Map<String, double> drag(
    TransformZone zone,
    int index, {
    required Offset down,
    required Offset last,
    required Offset now,
    required Doc doc,
    required double mult,
  }) {
    double cur(String id) => doc.get(id)!;
    final step = now - last;
    switch (zone) {
      case TransformZone.move:
        final delta = step / (compositionFactor * depthScale(z)) * mult;
        return {'pos.x': cur('pos.x') + delta.dx, 'pos.y': cur('pos.y') + delta.dy};
      case TransformZone.lift:
        return {'pos.z': cur('pos.z') + step.dy / railHalf * DirectSurface.zReach * mult};
      case TransformZone.rotate when index == 1:
        return {'rot.x': cur('rot.x') - step.dy * .8 * mult};
      case TransformZone.rotate when index == 2:
        return {'rot.y': cur('rot.y') + step.dx * .8 * mult};
      case TransformZone.rotate:
        final turn = _wrapped((now - pivot).direction - (last - pivot).direction);
        return {'rot.z': cur('rot.z') + turn * 180 / math.pi * mult};
      case TransformZone.scale:
        final from = rotate(down - pivot, -angle), to = rotate(now - pivot, -angle);
        double ratio(double a, double b) => b.abs() < 1 ? 1 : math.max(.01, a / b);
        var fx = index < 6 ? ratio(to.dx, from.dx) : 1.0;
        var fy = index < 4 || index >= 6 ? ratio(to.dy, from.dy) : 1.0;
        if (doc.link) {
          fx = fy = index < 4 ? math.max(.01, to.distance / math.max(1, from.distance)) : (index < 6 ? fx : fy);
        }
        fx = math.pow(fx, mult).toDouble();
        fy = math.pow(fy, mult).toDouble();
        return {'scale.x': value('scale.x') * fx, 'scale.y': value('scale.y') * fy, if (doc.link) 'scale.z': value('scale.z') * fx};
      case TransformZone.pivot:
        final delta = local(now) - local(last);
        double anchor(String id, double change, double half) => (cur(id) + change / (2 * half) * 100 * mult).clamp(0, 100).toDouble();
        return {'anchor.x': anchor('anchor.x', delta.dx, half.dx), 'anchor.y': anchor('anchor.y', delta.dy, half.dy)};
    }
  }
}

double _wrapped(double a) => math.atan2(math.sin(a), math.cos(a));

