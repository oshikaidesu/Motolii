// A recipe's motion, for its tile on the Create shelf: the inside makes it on a document of its own (another isolate,
// so the window never waits) and says where each thing and copy is, moment by moment, in the round (motolii_path3);
// the tile draws those places as points, seen a little from above and turning, and plays them round and round. Nothing
// is kept on disk: the pictures are the inside's own reckoning, drawn again whenever the shelf is shown.
import 'dart:async';
import 'dart:ffi' show Int32, Pointer, Int32Pointer;
import 'dart:io';
import 'dart:isolate';
import 'dart:math' as math;
import 'dart:typed_data';
import 'dart:ui' show PointMode;

import 'package:ffi/ffi.dart' show calloc;
import 'package:flutter/widgets.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:vector_math/vector_math_64.dart' show Matrix4, Vector3;

import '../motolii.dart';

/// Each moving part of a recipe: its colour, per copy its places (n moments, x y z size), and a path shape's outline
/// as its path says it ('' another kind), taken out even-odd or drawn by its edge alone as it says.
typedef RecipeMotion = List<({int colour, List<Float32List> copies, String outline, bool evenOdd, bool edge})>;

const _moments = 24, _seconds = 3.0, _mostCopies = 160;

abstract final class RecipeMotions {
  static final _all = <String, RecipeMotion>{};
  static bool _asked = false;
  static Pointer<Int32>? _halt;   // (said to the making: stop after the recipe it is on; read there between recipes)
  static Completer<void>? _ended;

  /// The making stopped once the recipe it is on is made, its document shut by itself, and waited for: a window shut,
  /// or a file of tests done, does not leave it opening documents while the program ends. Asked again, it begins again.
  static Future<void> stop() async {
    final halt = _halt, ended = _ended;
    if (halt == null || ended == null) return;
    halt.value = 1;
    await ended.future.timeout(const Duration(seconds: 30), onTimeout: () {});
    calloc.free(halt);
    _halt = null;
    _ended = null;
    _asked = false;
  }
  static final ready = ValueNotifier<int>(0);

  static RecipeMotion? of(String recipe) => _all[recipe];

  /// Asked once: every recipe made on a document of its own, its motion read, the document shut; each one's tile
  /// shown as soon as it is read (a recipe of many copies takes seconds: those after it do not wait for all).
  static void ask(String libDir, List<String> recipes) {
    if (_asked) return;
    _asked = true;
    final port = ReceivePort(), halt = _halt = calloc.allocate<Int32>(4)..value = 0, ended = _ended = Completer<void>();
    port.listen((got) {
      if (got case (final String r, final RecipeMotion motion)) {
        _all[r] = motion;
        ready.value++;
      } else {
        port.close();
        if (!ended.isCompleted) ended.complete();
      }
    });
    Isolate.spawn(_reckon, (libDir, recipes, port.sendPort, halt.address), onExit: port.sendPort);
  }

  static void _reckon((String, List<String>, SendPort, int) asked) {
    final (dir, recipes, tell, at) = asked;
    final halt = Pointer<Int32>.fromAddress(at);
    final folder = Directory.systemTemp.createTempSync('motolii_previews');
    final m = Motolii.open('${folder.path}/previews.motolii', wide: 1080, high: 1080, dir: dir);
    if (m == null) return tell.send(null);
    for (final r in recipes) {
      if (halt.value != 0) break;   // (stopped: its document shut below, as when all are made)
      final before = m.things.toSet();
      if (m.make(r) == 0) continue;
      final parts = <({int colour, List<Float32List> copies, String outline, bool evenOdd, bool edge})>[];
      for (final t in m.things.where((t) => !before.contains(t))) {
        if (const {'behaviour', 'camera', 'light'}.contains(m.word(t, 'kind'))) continue;
        final n = math.max(1, (m.value(t, 'copies', 0) ?? 1).round());
        final step = math.max(1, (n / _mostCopies).ceil());
        final copies = <Float32List>[];
        for (var c = 0; c < n; c += step) {
          final p = m.path3(t, copy: c, to: _seconds, n: _moments);
          if (p != null) copies.add(Float32List.fromList(p));
        }
        final fill = m.word(t, 'fill').split(' ').first, edge = fill == 'none';
        final outline = m.word(t, 'shape') == 'path' ? m.word(t, 'path') : '';
        parts.add((colour: 0xFF000000 | (int.tryParse(edge ? m.word(t, 'stroke') : fill, radix: 16) ?? 0xD9D2C3), copies: copies, outline: outline, evenOdd: m.word(t, 'fill_rule') == 'evenodd', edge: edge));
      }
      tell.send((r, parts));
    }
    m.close();
    try {
      folder.deleteSync(recursive: true);
    } catch (_) {}
    tell.send(null);
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
        if (motion case [final one] when one.outline.isNotEmpty && one.copies.length <= 1) return _Outline(one);   // (one path shape, still: its outline)
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

/// A path shape's outline in its tile, its own colour, fitted to the tile (its numbers are x and y in turn: the path
/// is said in M, L, C and Z) and drawn by flutter_svg.
class _Outline extends StatelessWidget {
  const _Outline(this.part);
  final ({int colour, List<Float32List> copies, String outline, bool evenOdd, bool edge}) part;
  @override
  Widget build(BuildContext context) {
    final n = [for (final m in RegExp(r'-?[0-9.]+(?:e-?[0-9]+)?').allMatches(part.outline)) double.parse(m.group(0)!)];
    if (n.length < 2) return const SizedBox.shrink();
    final xs = [for (var i = 0; i + 1 < n.length; i += 2) n[i]], ys = [for (var i = 1; i < n.length; i += 2) n[i]];
    final lo = (xs.reduce(math.min), ys.reduce(math.min)), side = math.max(xs.reduce(math.max) - lo.$1, ys.reduce(math.max) - lo.$2), pad = side * .12 + .05;
    final hex = (part.colour & 0xFFFFFF).toRadixString(16).padLeft(6, '0');
    final paint = part.edge ? 'fill="none" stroke="#$hex" stroke-width="${side * .08}" stroke-linecap="round"' : 'fill="#$hex"${part.evenOdd ? ' fill-rule="evenodd"' : ''}';
    return Padding(
      padding: const EdgeInsets.only(bottom: 14),   // (above its name)
      child: SvgPicture.string('<svg xmlns="http://www.w3.org/2000/svg" viewBox="${lo.$1 - pad} ${lo.$2 - pad} ${side + 2 * pad} ${side + 2 * pad}"><path d="${part.outline}" $paint/></svg>'),
    );
  }
}
