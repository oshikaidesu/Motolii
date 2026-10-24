// A recipe's motion, for its tile on the Create shelf: the inside makes it on a document of its own (another isolate,
// so the window never waits) and says where each thing and copy is, moment by moment, in the round (motolii_path3);
// the tile draws those places as points, seen a little from above and turning, and plays them round and round. Nothing
// is kept on disk: the pictures are the inside's own reckoning, drawn again whenever the shelf is shown.
import 'dart:async';
import 'dart:io';
import 'dart:isolate';
import 'dart:math' as math;
import 'dart:typed_data';
import 'dart:ui' show PointMode;

import 'package:flutter/widgets.dart';
import 'package:vector_math/vector_math_64.dart' show Matrix4, Vector3;

import '../motolii.dart';

/// Each moving part of a recipe: its colour, and per copy its places (n moments, x y z size).
typedef RecipeMotion = List<({int colour, List<Float32List> copies})>;

const _moments = 24, _seconds = 3.0, _mostCopies = 160;

abstract final class RecipeMotions {
  static final _all = <String, RecipeMotion>{};
  static Future<void>? _asked;
  static final ready = ValueNotifier<int>(0);

  static RecipeMotion? of(String recipe) => _all[recipe];

  /// Asked once: every recipe made on a document of its own, its motion read, the document shut.
  static void ask(String libDir, List<String> recipes) {
    _asked ??= Isolate.run(() => _reckon(libDir, recipes)).then((got) {
      _all.addAll(got);
      ready.value++;
    });
  }

  static Map<String, RecipeMotion> _reckon(String dir, List<String> recipes) {
    final folder = Directory.systemTemp.createTempSync('motolii_previews');
    final m = Motolii.open('${folder.path}/previews.motolii', wide: 1080, high: 1080, dir: dir);
    final out = <String, RecipeMotion>{};
    if (m == null) return out;
    for (final r in recipes) {
      final before = m.things.toSet();
      if (m.make(r) == 0) continue;
      final parts = <({int colour, List<Float32List> copies})>[];
      for (final t in m.things.where((t) => !before.contains(t))) {
        if (const {'behaviour', 'camera', 'light'}.contains(m.word(t, 'kind'))) continue;
        final n = math.max(1, (m.value(t, 'copies', 0) ?? 1).round());
        final step = math.max(1, (n / _mostCopies).ceil());
        final copies = <Float32List>[];
        for (var c = 0; c < n; c += step) {
          final p = m.path3(t, copy: c, to: _seconds, n: _moments);
          if (p != null) copies.add(Float32List.fromList(p));
        }
        final fill = m.word(t, 'fill').split(' ').first;
        parts.add((colour: 0xFF000000 | (int.tryParse(fill, radix: 16) ?? 0xD9D2C3), copies: copies));
      }
      out[r] = parts;
    }
    m.close();
    try {
      folder.deleteSync(recursive: true);
    } catch (_) {}
    return out;
  }
}

final _clock = ValueNotifier<int>(0);
Timer? _ticking;

/// A recipe's motion drawn in its tile, playing by itself.
class RecipeMotionView extends StatelessWidget {
  const RecipeMotionView(this.recipe, {super.key});
  final String recipe;
  @override
  Widget build(BuildContext context) {
    _ticking ??= Timer.periodic(const Duration(milliseconds: 1000 * 3 ~/ 24), (_) => _clock.value++);
    return ValueListenableBuilder<int>(
      valueListenable: RecipeMotions.ready,
      builder: (context, _, _) {
        final motion = RecipeMotions.of(recipe);
        if (motion == null || motion.isEmpty) return const SizedBox.shrink();
        return ValueListenableBuilder<int>(
          valueListenable: _clock,
          builder: (context, k, _) => CustomPaint(painter: _MotionPainter(motion, k % _moments, k)),
        );
      },
    );
  }
}

class _MotionPainter extends CustomPainter {
  _MotionPainter(this.motion, this.at, this.tick);
  final RecipeMotion motion;
  final int at, tick;

  @override
  void paint(Canvas canvas, Size s) {
    // (seen a little from above, turning slowly: what is deep reads as deep; Flutter's own Matrix4 does the seeing)
    final look = Matrix4.identity()
      ..rotateX(-0.35)
      ..rotateY(tick * 0.01);
    final points = <(Offset, double, int)>[];
    var lo = const Offset(double.infinity, double.infinity), hi = const Offset(-double.infinity, -double.infinity);
    for (final part in motion) {
      for (final c in part.copies) {
        for (var i = 0; i < c.length ~/ 4; i++) {   // (the box is of every moment: the picture holds still while it moves)
          final v = look.transform3(Vector3(c[4 * i], c[4 * i + 1], c[4 * i + 2]));
          final o = Offset(v.x, -v.y);
          lo = Offset(math.min(lo.dx, o.dx), math.min(lo.dy, o.dy));
          hi = Offset(math.max(hi.dx, o.dx), math.max(hi.dy, o.dy));
          if (i == at) points.add((o, c[4 * i + 3], part.colour));
        }
      }
    }
    if (points.isEmpty) return;
    final span = math.max(hi.dx - lo.dx, hi.dy - lo.dy), pad = 6.0;
    final k = span <= 0 ? 1.0 : (math.min(s.width, s.height) - 2 * pad) / span;
    final mid = (lo + hi) / 2, centre = Offset(s.width / 2, s.height / 2);
    final paint = Paint()..strokeCap = StrokeCap.round;
    for (final (o, size, colour) in points) {
      paint
        ..color = Color(colour)
        ..strokeWidth = (1.6 + size.abs() * 1.2).clamp(1.2, 5.0);
      canvas.drawPoints(PointMode.points, [centre + (o - mid) * k], paint);
    }
  }

  @override
  bool shouldRepaint(_MotionPainter o) => o.at != at || o.tick != tick || !identical(o.motion, motion);
}
