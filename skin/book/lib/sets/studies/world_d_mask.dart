// World D 24: Mask feather. A lattice of light seen through a window cut in a plane; the window's edge is soft and its size can swell.
// A Feather (blue): the soft edge (two dashed contours bound the blur). B Expansion (green): the dotted contour is the window as cut; the lit
// window grows or shrinks away from it. C Opacity (white): the plane's corner marks and how much light the plane lets through.
// D Invert (orange): the small ring in the window; filled = the window is the plane and the plane is the window.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../../tokens.dart';
import 'world_d_kit.dart';

const maskSpecs = [
  WdSpec('feather', 'Feather', 0, 200, 0, unit: 'px', digits: 0, amp: .12),
  WdSpec('exp', 'Expansion', -100, 100, 0, unit: 'px', digits: 0, amp: .16),
  WdSpec('op', 'Opacity', 0, 100, 100, unit: '%', digits: 0, amp: .2),
  WdSpec('inv', 'Invert', 0, 1, 0, digits: 0, range: 'off/on', defText: 'off', driven: false),
];

class MaskMini extends WdMini {
  MaskMini(super.doc);
  double _acc = 0, _moved = 0;
  bool _invAt = false;

  @override
  List<List<String>> get zoneIds => const [['feather'], ['exp'], ['op'], ['inv']];

  @override
  List<WdR> readout() => [
        (doc.show('feather'), doc.changed('feather')),
        (doc.show('exp'), doc.changed('exp')),
        (doc.show('op'), doc.changed('op')),
        (doc.base('inv') > .5 ? 'inv' : '', doc.changed('inv')),
      ];

  double _kf(Rect a) => a.shortestSide * .002;
  double _ke(Rect a) => a.shortestSide * .0016;

  RRect _hole(Rect a, [double grow = 0]) {
    final r = Rect.fromCenter(center: a.center, width: a.width * .56, height: a.height * .66);
    return RRect.fromRectAndRadius(r.inflate(math.max(grow, -r.shortestSide / 2 + 2)), const Radius.circular(6));
  }

  @override
  int? zoneAt(Offset p, Size s) {
    final a = art(s), h = _hole(a);
    if (wdFat(Rect.fromCenter(center: a.center, width: 8, height: 8)).contains(p)) return 3;
    if (h.contains(p)) return 1;
    if (h.outerRect.inflate(24).contains(p)) return 0;
    if (a.inflate(10).contains(p)) return 2;
    return null;
  }

  @override
  void down(int z, Offset p, Size s) {
    _acc = 0;
    _moved = 0;
    _invAt = doc.base('inv') > .5;
  }

  @override
  void move(int z, Offset p, Offset d, bool fine, Size s) {
    final k = fine ? .1 : 1.0, a = art(s);
    final r = p - a.center, u = r.distance < 1 ? const Offset(1, 0) : r / r.distance;
    final radial = d.dx * u.dx + d.dy * u.dy;
    switch (z) {
      case 0:
        doc.set('feather', doc.base('feather') + radial * k / _kf(a));
      case 1:
        doc.set('exp', doc.base('exp') + radial * k / _ke(a));
      case 2:
        doc.set('op', doc.base('op') - d.dy * k * 100 / (a.height * .7));
      case 3:
        _moved += d.distance;
        _acc += d.dx;
        if (_moved > 6 && _acc > 10) doc.set('inv', 1);
        if (_moved > 6 && _acc < -10) doc.set('inv', 0);
    }
  }

  @override
  void up(int z) {
    if (z == 3 && _moved <= 6) doc.set('inv', _invAt ? 0 : 1);
  }

  @override
  void paint(Canvas c, Size s) {
    final a = art(s);
    final breathe = math.sin(doc.time * 1.3);
    final feather = doc.eff('feather'), exp = doc.eff('exp'), op = doc.eff('op') / 100, inv = doc.base('inv') > .5;
    final sigma = _kf(a) * feather * .5 + .6 + breathe * .5;
    final grow = exp * _ke(a) + breathe * .8;

    // the light, seen only through the mask
    c.saveLayer(a.inflate(1), Paint());
    final dot = wdFill(N.g76);
    for (var x = a.left + 4; x < a.right; x += 7) {
      for (var y = a.top + 4; y < a.bottom; y += 7) {
        c.drawCircle(Offset(x, y), 1.0, dot);
      }
    }
    c.saveLayer(a.inflate(1), Paint()..blendMode = BlendMode.dstIn);
    final holePaint = Paint()..maskFilter = MaskFilter.blur(BlurStyle.normal, sigma);
    if (!inv) {
      c.drawRect(a.inflate(1), Paint()..color = Color.fromARGB((255 * (1 - op)).round(), 255, 255, 255));
      holePaint
        ..color = Color.fromARGB((255 * op).round(), 255, 255, 255)
        ..blendMode = BlendMode.plus;
    } else {
      c.drawRect(a.inflate(1), Paint()..color = const Color(0xFFFFFFFF));
      holePaint
        ..color = Color.fromARGB((255 * op).round(), 255, 255, 255)
        ..blendMode = BlendMode.dstOut;
    }
    c.drawRRect(_hole(a, grow), holePaint);
    c.restore();
    c.restore();

    // B: the window as cut (dotted); A: the extent of the soft edge (dashed, solid when the edge is hard)
    wdDashPath(c, Path()..addRRect(_hole(a)), wdStroke(zc(1)), on: 1.2, off: 3);
    final fw = sigma * 1.4;
    if (fw < 2.5) {
      c.drawRRect(_hole(a, grow), wdStroke(zc(0)));
    } else {
      wdDashPath(c, Path()..addRRect(_hole(a, grow + fw)), wdStroke(zc(0)), on: 3, off: 3);
      wdDashPath(c, Path()..addRRect(_hole(a, math.max(-20, grow - fw))), wdStroke(zc(0)), on: 3, off: 3);
    }
    // C: the plane's corner marks
    final cm = wdStroke(zc(2));
    for (final sx in const [-1, 1]) {
      for (final sy in const [-1, 1]) {
        final o = Offset(sx < 0 ? a.left + 2 : a.right - 2, sy < 0 ? a.top + 2 : a.bottom - 2);
        c.drawLine(o, o + Offset(-sx * 7.0, 0), cm);
        c.drawLine(o, o + Offset(0, -sy * 7.0), cm);
      }
    }
    // D: the ring
    inv ? c.drawCircle(a.center, 3, wdFill(zc(3))) : c.drawCircle(a.center, 3, wdStroke(zc(3)));

    final h = _hole(a);
    hint(c, h.outerRect.inflate(24), 0);
    hint(c, h.outerRect, 1);
    hint(c, wdFat(Rect.fromCenter(center: a.center, width: 8, height: 8)), 3);
  }
}
