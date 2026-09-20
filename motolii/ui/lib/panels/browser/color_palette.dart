import 'dart:math' as math;
import 'dart:ui' as ui show instantiateImageCodec, ImageByteFormat;

import 'package:flutter/foundation.dart' show Uint8List;

/// The few colours a picture is mostly made of: decode small, drop
/// transparent pixels, k-means in RGB, brightest first.
Future<List<List<double>>> paletteOf(Uint8List bytes, {int count = 6}) async {
  final codec = await ui.instantiateImageCodec(bytes, targetWidth: 64);
  final image = (await codec.getNextFrame()).image;
  final data = await image.toByteData(format: ui.ImageByteFormat.rawRgba);
  image.dispose();
  if (data == null) return [];
  final px = <List<double>>[];
  for (var i = 0; i + 3 < data.lengthInBytes; i += 4) {
    if (data.getUint8(i + 3) < 128) continue;
    px.add([
      data.getUint8(i) / 255,
      data.getUint8(i + 1) / 255,
      data.getUint8(i + 2) / 255,
    ]);
  }
  if (px.isEmpty) return [];
  double luma(List<double> c) => .299 * c[0] + .587 * c[1] + .114 * c[2];
  px.sort((a, b) => luma(a).compareTo(luma(b)));
  final k = math.min(count, px.length);
  var centers = [
    for (var i = 0; i < k; i++) px[(px.length - 1) * i ~/ math.max(1, k - 1)],
  ];
  for (var round = 0; round < 8; round++) {
    final sums = List.generate(k, (_) => [0.0, 0.0, 0.0, 0.0]);
    for (final p in px) {
      var best = 0;
      var bestD = double.infinity;
      for (var i = 0; i < k; i++) {
        final c = centers[i];
        final d =
            (p[0] - c[0]) * (p[0] - c[0]) +
            (p[1] - c[1]) * (p[1] - c[1]) +
            (p[2] - c[2]) * (p[2] - c[2]);
        if (d < bestD) {
          bestD = d;
          best = i;
        }
      }
      final s = sums[best];
      s[0] += p[0];
      s[1] += p[1];
      s[2] += p[2];
      s[3] += 1;
    }
    centers = [
      for (var i = 0; i < k; i++)
        sums[i][3] == 0
            ? centers[i]
            : [
                sums[i][0] / sums[i][3],
                sums[i][1] / sums[i][3],
                sums[i][2] / sums[i][3],
              ],
    ];
  }
  // Edge blends and stray pixels do not count as a colour of the picture.
  final weights = List.filled(k, 0);
  for (final p in px) {
    var best = 0;
    var bestD = double.infinity;
    for (var i = 0; i < k; i++) {
      final c = centers[i];
      final d =
          (p[0] - c[0]) * (p[0] - c[0]) +
          (p[1] - c[1]) * (p[1] - c[1]) +
          (p[2] - c[2]) * (p[2] - c[2]);
      if (d < bestD) {
        bestD = d;
        best = i;
      }
    }
    weights[best]++;
  }
  final unique = <String, List<double>>{
    for (final (i, c) in centers.indexed)
      if (weights[i] * 20 >= px.length)
        c.map((v) => (v * 255).round()).join(','): [...c, 1.0],
  };
  return unique.values.toList()..sort((a, b) => luma(b).compareTo(luma(a)));
}
