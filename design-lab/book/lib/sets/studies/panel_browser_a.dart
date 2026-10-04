// Panel drafts A (Browser): find (B3), narrowing and showing a filter is on (B2/B4), preview (B5/B6), recents and shelves (B10), similar (B12-b).
// Each use case is ONE PANEL at real size (Browser 320, narrow dock 232) with real-looking content. Local state only; no cross-panel session.
// The one-line story (option id, habit status, contract) sits OUTSIDE the panel frame. Source: research/options-browser.md, experience-first.md section C.
import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import '../../parts/controls.dart';
import '../../parts/glyphs.dart';
import '../../tokens.dart';
import '../panels/media_parts.dart';

// ================================================================================ data

enum Fx { none, blur, glow, tint, warp, glitch, echo, pixel, grain, wipe, pop }

class It {
  It(this.name, this.jp, this.place, this.cat, this.fx, this.tags, {this.uses = 0, this.recent = -1, this.applies = 'layer', this.asset, this.long = false})
      : hay = '$name $jp $cat ${tags.join(' ')} ${_kindOf(place)}'.toLowerCase();
  final String name, jp, place, cat, applies, hay;
  final Fx fx;
  final List<String> tags;
  final int uses, recent; // recent = minutes since last use, -1 never
  final Asset? asset;
  final bool long;
  String get kind => _kindOf(place);
  String get source => place == 'Mine' ? 'Mine' : (place == 'Project' ? 'Project' : 'Built-in');
  bool get fav => uses >= 15;
  int get seed => name.codeUnits.fold<int>(0, (a, b) => a + b) % 31;
}

String _kindOf(String place) => place == 'Effects' ? 'Effect' : (place == 'Project' ? 'Media' : 'Preset');

It _e(String n, String jp, String cat, Fx fx, List<String> t, {int u = 0, int r = -1, String ap = 'layer'}) => It(n, jp, 'Effects', cat, fx, t, uses: u, recent: r, applies: ap);
It _p(String n, String jp, String cat, Fx fx, List<String> t, {int u = 0, int r = -1, String ap = 'layer', bool long = false}) => It(n, jp, 'Presets', cat, fx, t, uses: u, recent: r, applies: ap, long: long);
It _m(String n, String jp, Fx fx, List<String> t, {int u = 0, int r = -1}) => It(n, jp, 'Mine', 'Mine', fx, t, uses: u, recent: r);

final List<It> kItems = [
  _e('Gaussian Blur', 'ガウスぼかし', 'Blur', Fx.blur, ['soft'], u: 41, r: 3),
  _e('Directional Blur (Motion Trail, 8-sample, GPU)', '方向ぼかし(モーショントレイル)', 'Blur', Fx.blur, ['soft', 'moving'], u: 6),
  _e('Radial Blur', '放射ぼかし', 'Blur', Fx.blur, ['moving']),
  _e('Lens Blur', 'レンズぼかし', 'Blur', Fx.blur, ['soft'], u: 9),
  _e('Bokeh Blur', 'ボケ', 'Blur', Fx.blur, ['soft', 'bright']),
  _e('Tilt Shift', 'ティルトシフト', 'Blur', Fx.blur, ['soft']),
  _e('Hue / Saturation', '色相・彩度', 'Color', Fx.tint, ['grade'], u: 29, r: 12),
  _e('Tint', '色調', 'Color', Fx.tint, ['grade'], u: 8),
  _e('Curves', 'カーブ', 'Color', Fx.tint, ['grade'], u: 22, r: 95, ap: 'any'),
  _e('Levels', 'レベル補正', 'Color', Fx.tint, ['grade']),
  _e('Duotone', 'デュオトーン', 'Color', Fx.tint, ['grade', 'retro']),
  _e('Color Balance', 'カラーバランス', 'Color', Fx.tint, ['grade'], u: 4),
  _e('Apply LUT (Cinematic Teal & Orange 3D)', 'LUT 適用(シネマティック)', 'Color', Fx.tint, ['grade'], u: 3),
  _e('Chromatic Aberration', '色収差', 'Distort', Fx.glitch, ['glitch', 'retro'], u: 5),
  _e('Glow', 'グロー', 'Glow', Fx.glow, ['bright', 'soft'], u: 33, r: 1),
  _e('Bloom', 'ブルーム', 'Glow', Fx.glow, ['bright', 'soft'], u: 11),
  _e('Halation (Film Stock 500T)', 'ハレーション', 'Glow', Fx.glow, ['bright', 'retro']),
  _e('Light Rays', '光線', 'Glow', Fx.glow, ['bright']),
  _e('Lens Flare', 'レンズフレア', 'Glow', Fx.glow, ['bright'], u: 2),
  _e('Edge Glow', '輪郭グロー', 'Glow', Fx.glow, ['bright']),
  _e('Wave Warp', '波形ワープ', 'Distort', Fx.warp, ['moving'], u: 7),
  _e('Twirl', '渦巻き', 'Distort', Fx.warp, ['moving']),
  _e('Bulge', '膨張', 'Distort', Fx.warp, []),
  _e('Displacement Map', 'ディスプレイスメント', 'Distort', Fx.warp, ['moving']),
  _e('Ripple', '波紋', 'Distort', Fx.warp, ['moving', 'soft']),
  _e('Turbulent Displace', '乱流ディスプレイス', 'Distort', Fx.warp, ['moving']),
  _e('Pixelate', 'モザイク', 'Stylize', Fx.pixel, ['retro'], u: 3),
  _e('Halftone', 'ハーフトーン', 'Stylize', Fx.pixel, ['retro']),
  _e('Posterize', 'ポスタライズ', 'Stylize', Fx.pixel, ['retro']),
  _e('Glitch RGB Split', 'グリッチ', 'Stylize', Fx.glitch, ['glitch'], u: 17, r: 25),
  _e('VHS Wobble', 'VHS 揺れ', 'Stylize', Fx.glitch, ['glitch', 'retro'], u: 6),
  _e('Film Grain', 'フィルムグレイン', 'Stylize', Fx.grain, ['retro'], u: 12, r: 70),
  _e('Scanlines', '走査線', 'Stylize', Fx.pixel, ['retro']),
  _e('Echo', 'エコー', 'Time', Fx.echo, ['moving'], u: 4, r: 240),
  _e('Posterize Time', 'コマ落とし', 'Time', Fx.echo, ['retro', 'moving']),
  _e('Time Displacement', '時間差', 'Time', Fx.echo, ['moving']),
  _e('Gradient Ramp', 'グラデーション', 'Generate', Fx.tint, ['grade'], u: 10),
  _e('Fractal Noise', 'フラクタルノイズ', 'Generate', Fx.grain, ['moving']),
  _e('Checkerboard', '市松模様', 'Generate', Fx.pixel, ['retro']),
  _e('Grid', 'グリッド', 'Generate', Fx.pixel, []),
  _e('Radio Waves', '電波', 'Generate', Fx.warp, ['moving']),
  _e('Luma Key', 'ルマキー', 'Matte', Fx.wipe, [], ap: 'layer'),
  _e('Chroma Key (Green Screen)', 'クロマキー', 'Matte', Fx.wipe, [], u: 2),
  _e('Matte Choker', 'マット境界', 'Matte', Fx.wipe, []),
  _p('Title In: Soft Rise', 'タイトル: やわらか上昇', 'Title', Fx.pop, ['soft', 'title'], u: 14, r: 40),
  _p('Typewriter', 'タイプライター', 'Title', Fx.wipe, ['title', 'retro'], u: 5, r: 300, long: true),
  _p('Pop In Overshoot', 'ポップイン', 'Title', Fx.pop, ['moving', 'title'], u: 19, r: 8),
  _p('Wipe Reveal L→R', 'ワイプ表示', 'Title', Fx.wipe, ['title'], u: 9),
  _p('Shake Impact', '衝撃シェイク', 'Motion', Fx.glitch, ['moving'], u: 3),
  _p('Glitch Flicker (Loop)', 'グリッチ点滅(ループ)', 'Motion', Fx.glitch, ['glitch', 'loop']),
  _p('Neon Sign Flicker', 'ネオン点滅', 'Motion', Fx.glow, ['bright', 'retro', 'loop'], u: 8),
  _p('Lower Third Slide', 'ローワーサード', 'Title', Fx.wipe, ['title'], u: 6, long: true),
  _p('Bounce Drop', 'バウンスドロップ', 'Motion', Fx.pop, ['moving'], u: 1),
  _p('Kinetic Zoom Punch', 'キネティックズーム', 'Motion', Fx.pop, ['moving']),
  _p('Ease Out Back (Soft)', 'イーズ: 戻りやわらか', 'Ease', Fx.pop, ['soft', 'moving'], u: 21, ap: 'key'),
  _p('Elastic Snap', 'イーズ: 弾む', 'Ease', Fx.pop, ['moving'], u: 4, ap: 'key'),
  _p('Hold then Release', 'イーズ: 溜めて解放', 'Ease', Fx.pop, ['moving'], ap: 'key'),
  _m('My Soft Bloom 01', '自作: やわらかブルーム', Fx.glow, ['bright', 'soft'], u: 16),
  _m('夜のネオン (client A)', '自作: クライアント A', Fx.glow, ['bright', 'retro'], u: 7),
  _m('Title Pop v3', '自作: タイトルポップ', Fx.pop, ['title', 'moving'], u: 15),
  for (final a in sampleAssets) It(a.name, '${a.dims} · ${a.size}', 'Project', a.kind.label, Fx.none, a.tags, asset: a, uses: a.seed > 10 ? 1 : 0),
];

const _places = ['Effects', 'Presets', 'Project', 'Mine'];

List<It> _match(Iterable<It> src, String q) {
  final t = _tokens(q);
  final r = src.where((i) => t.every(i.hay.contains)).toList();
  if (t.isEmpty) return r;
  // name-first ranking: a word of the name starts with the query, then the name contains it, then only tags/category do. Stable inside a rank.
  int score(It i) {
    final n = i.name.toLowerCase(), words = n.split(RegExp(r'[^a-z0-9\u3040-\u30ff\u4e00-\u9fff]+'));
    if (t.every((x) => words.any((w) => w.startsWith(x)))) return 0;
    if (t.every(n.contains)) return 1;
    return 2;
  }

  final idx = {for (var k = 0; k < r.length; k++) r[k]: k};
  r.sort((a, b) => score(a) != score(b) ? score(a).compareTo(score(b)) : idx[a]!.compareTo(idx[b]!));
  return r;
}

List<String> _tokens(String q) => q.toLowerCase().split(RegExp(r'\s+')).where((s) => s.isNotEmpty).toList();

int _lev(String a, String b) {
  var prev = List<int>.generate(b.length + 1, (i) => i);
  for (var i = 1; i <= a.length; i++) {
    final cur = <int>[i];
    for (var j = 1; j <= b.length; j++) {
      cur.add(math.min(math.min(cur[j - 1] + 1, prev[j] + 1), prev[j - 1] + (a[i - 1] == b[j - 1] ? 0 : 1)));
    }
    prev = cur;
  }
  return prev.last;
}

/// Words of item names within a small edit distance of the query's first word (for "did you mean").
List<String> _suggest(String q) {
  final w = _tokens(q).isEmpty ? '' : _tokens(q).first;
  if (w.length < 3) return const [];
  final best = <String, int>{};
  for (final i in kItems) {
    for (final x in i.name.toLowerCase().split(RegExp(r'[^a-z0-9]+'))) {
      if (x.length < 3 || x == w) continue;
      final d = _lev(w, x);
      if (d <= (w.length >= 4 ? 2 : 1)) best[x] = d;
    }
  }
  final out = best.entries.toList()..sort((a, b) => a.value.compareTo(b.value));
  return [for (final e in out.take(3)) e.key];
}

String _ago(int min) => min < 60 ? '$min min ago' : (min < 1440 ? '${min ~/ 60} h ago' : '${min ~/ 1440} d ago');

// ================================================================================ previews (a tiny stand-in for "what the effect does")

class _Fx extends StatelessWidget {
  const _Fx(this.fx, this.t, {this.base, this.seed = 3});
  final Fx fx;
  final double t;
  final Widget? base;
  final int seed;

  @override
  Widget build(BuildContext context) {
    final s = (math.sin(t * math.pi * 2 - math.pi / 2) + 1) / 2; // 0 at t=0, 1 at t=.5
    Widget art() => base ?? Art(sampleAssets[0]);
    final Widget w;
    switch (fx) {
      case Fx.none:
        w = art();
      case Fx.blur:
        w = ImageFiltered(imageFilter: ui.ImageFilter.blur(sigmaX: .01 + s * 4, sigmaY: .01 + s * 4), child: art());
      case Fx.glow:
        w = Stack(fit: StackFit.expand, children: [
          art(),
          DecoratedBox(decoration: BoxDecoration(gradient: RadialGradient(center: const Alignment(.3, -.25), radius: .95, colors: [N.g100.withValues(alpha: .75 * s), N.g100.withValues(alpha: 0)]))),
        ]);
      case Fx.tint:
        w = ColorFiltered(colorFilter: ColorFilter.mode(Fam.scatter.c.withValues(alpha: .08 + .5 * s), BlendMode.color), child: art());
      case Fx.warp:
        w = Transform(alignment: Alignment.center, transform: Matrix4.skewX(.3 * (s - .5)) * Matrix4.diagonal3Values(1 + .1 * s, 1, 1), child: art());
      case Fx.glitch:
        final q = (t * 8).floor(), off = ((q % 3) - 1) * 5.0;
        w = Stack(fit: StackFit.expand, children: [
          art(),
          Transform.translate(offset: Offset(off, 0), child: Opacity(opacity: .5, child: ColorFiltered(colorFilter: ColorFilter.mode(Fam.scatter.c, BlendMode.srcATop), child: art()))),
          Align(alignment: Alignment(0, -.7 + (q % 5) * .35), child: FractionallySizedBox(heightFactor: .12, widthFactor: 1, child: Transform.translate(offset: Offset(-off * 3, 0), child: ColoredBox(color: N.g100.withValues(alpha: .35))))),
        ]);
      case Fx.echo:
        w = Stack(fit: StackFit.expand, children: [
          for (var i = 2; i >= 0; i--) Opacity(opacity: i == 0 ? 1 : .35 - i * .1, child: FractionalTranslation(translation: Offset(i * .09 * s, 0), child: art())),
        ]);
      case Fx.pixel:
        w = Stack(fit: StackFit.expand, children: [
          ImageFiltered(imageFilter: ui.ImageFilter.blur(sigmaX: 1.2, sigmaY: 1.2), child: art()),
          CustomPaint(painter: _GridPaint(5 + s * 5)),
        ]);
      case Fx.grain:
        w = Stack(fit: StackFit.expand, children: [art(), CustomPaint(painter: _NoisePaint((t * 12).floor() + seed))]);
      case Fx.wipe:
        w = Stack(fit: StackFit.expand, children: [const ColoredBox(color: N.g13), ClipRect(clipper: _FracClip(.04 + .96 * s), child: art())]);
      case Fx.pop:
        w = Stack(fit: StackFit.expand, children: [
          const ColoredBox(color: N.g13),
          Center(child: Opacity(opacity: math.min(1, t * 5), child: Transform.scale(scale: .55 + .45 * Curves.easeOutBack.transform(t), child: FractionallySizedBox(widthFactor: .72, heightFactor: .72, child: art())))),
        ]);
    }
    return ClipRect(child: w);
  }
}

class _FracClip extends CustomClipper<Rect> {
  const _FracClip(this.f);
  final double f;
  @override
  Rect getClip(Size s) => Rect.fromLTWH(0, 0, s.width * f, s.height);
  @override
  bool shouldReclip(_FracClip o) => o.f != f;
}

class _GridPaint extends CustomPainter {
  const _GridPaint(this.step);
  final double step;
  @override
  void paint(Canvas c, Size s) {
    final p = Paint()..color = N.g00.withValues(alpha: .35)..strokeWidth = 1;
    for (var x = 0.0; x < s.width; x += step) {
      c.drawLine(Offset(x, 0), Offset(x, s.height), p);
    }
    for (var y = 0.0; y < s.height; y += step) {
      c.drawLine(Offset(0, y), Offset(s.width, y), p);
    }
  }

  @override
  bool shouldRepaint(_GridPaint o) => o.step != step;
}

class _NoisePaint extends CustomPainter {
  const _NoisePaint(this.seed);
  final int seed;
  @override
  void paint(Canvas c, Size s) {
    final r = math.Random(seed), p = Paint();
    final n = (s.width * s.height / 30).clamp(30, 400).toInt();
    for (var i = 0; i < n; i++) {
      p.color = (r.nextBool() ? N.g100 : N.g00).withValues(alpha: .3);
      c.drawRect(Rect.fromLTWH(r.nextDouble() * s.width, r.nextDouble() * s.height, 1.5, 1.5), p);
    }
  }

  @override
  bool shouldRepaint(_NoisePaint o) => o.seed != seed;
}

/// What an item looks like in a list: one soft mid-grey tile with a recognisable shape per effect family (never a near-black blur, never a loud hue).
/// blur = concentric soft rings, glow = a halo, warp = a wave, stylize (pixel) = blocks, colour (tint) = a gradient chip; the item's seed nudges the size so siblings are not clones.
class _Thumb extends StatelessWidget {
  const _Thumb(this.it, {this.w = 36, this.h = 22});
  final It it;
  final double w, h;
  @override
  Widget build(BuildContext context) => SizedBox(
        width: w,
        height: h,
        child: ClipRRect(
          borderRadius: BorderRadius.circular(3),
          child: it.asset != null
              ? Stack(fit: StackFit.expand, children: [
                  ColorFiltered(colorFilter: const ColorFilter.matrix(<double>[.33, .33, .33, 0, 0, .33, .33, .33, 0, 0, .33, .33, .33, 0, 0, 0, 0, 0, 1, 0]), child: Art(it.asset!)),
                  const ColoredBox(color: Color(0x66808080)),
                ])
              : CustomPaint(painter: _TilePaint(it.fx, it.seed, _variant(it))),
        ),
      );
}

/// Which shape of a family an item gets: the named blur and glow effects each have their own (aperture or halo), everything else follows its seed.
int _variant(It it) {
  const blur = ['Gaussian Blur', 'Directional Blur', 'Radial Blur', 'Lens Blur', 'Bokeh Blur', 'Tilt Shift'];
  const glow = ['Glow', 'Bloom', 'Halation', 'Light Rays', 'Lens Flare'];
  final key = it.name.split(' (').first;
  if (it.fx == Fx.blur && blur.contains(key)) return blur.indexOf(key);
  if (it.fx == Fx.glow && glow.contains(key)) return glow.indexOf(key);
  return it.seed;
}

class _TilePaint extends CustomPainter {
  const _TilePaint(this.fx, [this.seed = 0, this.v = 0]);
  final Fx fx;
  final int seed, v;
  @override
  void paint(Canvas c, Size s) {
    c.drawRect(Offset.zero & s, Paint()..shader = ui.Gradient.linear(Offset.zero, Offset(0, s.height), const [Color(0xFF6A6A6A), Color(0xFF585858)]));
    final ink = Paint()..color = N.g91.withValues(alpha: .8), soft = Paint()..color = N.g91.withValues(alpha: .55);
    final k = .9 + (seed % 5) * .05; // .9 .. 1.1
    final m = s.center(Offset.zero), r = s.height * .3 * k;
    switch (fx) {
      case Fx.blur:
        // Soft rings in the shape of the aperture: 0 soft round (Gaussian), 1 a horizontal streak (Directional), 2 a hexagon (Radial), 3 a rounded square with an inner ring (Lens), 4 a pentagon with a bright rim, 5 round with a sharp band across.
        final shape = v % 6, sides = const [0, 0, 6, 0, 5, 0][shape];
        for (var i = 3; i >= 1; i--) {
          final rr = r * (.55 + i * .5), paint = Paint()..color = N.g91.withValues(alpha: .16 + (4 - i) * .14)..maskFilter = const MaskFilter.blur(BlurStyle.normal, 2.2);
          if (shape == 1) {
            c.drawOval(Rect.fromCenter(center: m, width: rr * 3.4, height: rr * .7), paint);
          } else if (shape == 3) {
            c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: m, width: rr * 1.7, height: rr * 1.7), Radius.circular(rr * .4)), paint);
          } else if (sides == 0) {
            c.drawCircle(m, rr, paint);
          } else {
            final path = Path();
            for (var k = 0; k < sides; k++) {
              final a = -math.pi / 2 + k * 2 * math.pi / sides + (sides == 4 ? math.pi / 4 : 0);
              final pt = m + Offset(math.cos(a), math.sin(a)) * rr * (sides == 4 ? .9 : 1);
              k == 0 ? path.moveTo(pt.dx, pt.dy) : path.lineTo(pt.dx, pt.dy);
            }
            c.drawPath(path..close(), paint);
          }
        }
        if (shape == 3) c.drawCircle(m, r * .8, Paint()..color = N.g100.withValues(alpha: .7)..style = PaintingStyle.stroke..strokeWidth = 1);
        if (shape == 4) c.drawCircle(m, r * 1.55, Paint()..color = N.g100.withValues(alpha: .5)..style = PaintingStyle.stroke..strokeWidth = 1);
        if (shape == 5) c.drawRect(Rect.fromLTWH(0, m.dy - 1.5, s.width, 3), Paint()..color = N.g100.withValues(alpha: .55));
      case Fx.glow:
        // Halo variants: 0 halo and a core (Glow), 1 a wide soft ellipse with a ring and no core (Bloom), 2 a thick blurred ring round a dark centre (Halation), 3 rays (Light Rays), 4 two spots (Lens Flare).
        final g = v % 5, halo = s.height * const [.62, .8, .6, .55, .6][g];
        if (g != 1 && g != 2) c.drawCircle(m, halo, Paint()..shader = ui.Gradient.radial(m, halo, [N.g100.withValues(alpha: .85), N.g100.withValues(alpha: .0)]));
        switch (g) {
          case 0:
            c.drawCircle(m, r * .45, Paint()..color = N.g100.withValues(alpha: .95));
          case 1:
            // Bloom: a wide soft ellipse with a thin ring, no hard core (Glow has the core).
            final oval = Rect.fromCenter(center: m, width: s.width * .86, height: s.height * .66);
            c.drawOval(oval, Paint()..color = N.g100.withValues(alpha: .42)..maskFilter = const MaskFilter.blur(BlurStyle.normal, 4));
            c.drawOval(oval.deflate(2), Paint()..color = N.g100.withValues(alpha: .6)..style = PaintingStyle.stroke..strokeWidth = 1);
          case 2:
            // Halation: a thick blurred ring around a dark centre.
            c.drawCircle(m, r * 1.3, Paint()..color = N.g100.withValues(alpha: .75)..style = PaintingStyle.stroke..strokeWidth = 3.2..maskFilter = const MaskFilter.blur(BlurStyle.normal, 1.6));
            c.drawCircle(m, r * .5, Paint()..color = const Color(0xFF4A4A4A));
          case 3:
            for (var k = 0; k < 8; k++) {
              final a = k * math.pi / 4;
              c.drawLine(m + Offset(math.cos(a), math.sin(a)) * r * .5, m + Offset(math.cos(a), math.sin(a)) * r * 1.5, Paint()..color = N.g100.withValues(alpha: .8)..strokeWidth = 1.2);
            }
            c.drawCircle(m, r * .3, Paint()..color = N.g100.withValues(alpha: .95));
          default:
            c.drawCircle(m + Offset(-s.width * .12, 0), r * .4, Paint()..color = N.g100.withValues(alpha: .95));
            c.drawCircle(m + Offset(s.width * .16, s.height * .1), r * .22, Paint()..color = N.g100.withValues(alpha: .8));
        }
      case Fx.tint:
        final chip = RRect.fromRectAndRadius(Rect.fromLTWH(s.width * .2, s.height * .24, s.width * .6, s.height * .52), const Radius.circular(2));
        c.drawRRect(chip, Paint()..shader = ui.Gradient.linear(chip.outerRect.centerLeft, chip.outerRect.centerRight, [N.g100.withValues(alpha: .15), N.g100.withValues(alpha: .9)]));
        c.drawRRect(chip, Paint()..color = N.g91.withValues(alpha: .35)..style = PaintingStyle.stroke..strokeWidth = 1);
      case Fx.warp:
        for (var j = -1; j <= 1; j += 2) {
          final path = Path()..moveTo(3, m.dy + j * 4);
          for (var x = 3.0; x <= s.width - 3; x += 1) {
            path.lineTo(x, m.dy + j * 4 + math.sin(x / s.width * math.pi * 3 + seed * .3) * s.height * .2);
          }
          c.drawPath(path, (j < 0 ? ink : soft)..style = PaintingStyle.stroke..strokeWidth = 1.8);
        }
      case Fx.pixel:
        for (var i = 0; i < 4; i++) {
          for (var j = 0; j < 2; j++) {
            c.drawRect(Rect.fromLTWH(s.width * .2 + i * s.width * .15 + .5, s.height * .22 + j * s.height * .28 + .5, s.width * .15 - 1, s.height * .28 - 1), (i + j).isEven ? ink : soft);
          }
        }
      case Fx.glitch:
        c.drawRect(Rect.fromLTWH(s.width * .2, s.height * .25, s.width * .5, s.height * .18), ink);
        c.drawRect(Rect.fromLTWH(s.width * .32, s.height * .55, s.width * .5, s.height * .18), soft);
      case Fx.grain:
        final rnd = math.Random(5);
        for (var i = 0; i < 26; i++) {
          c.drawRect(Rect.fromLTWH(rnd.nextDouble() * s.width, rnd.nextDouble() * s.height, 1.5, 1.5), soft);
        }
      case Fx.echo:
        for (var i = 2; i >= 0; i--) {
          c.drawCircle(Offset(m.dx - 6 + i * 6, m.dy), r * .8, i == 0 ? ink : Paint()..color = N.g91.withValues(alpha: .3));
        }
      case Fx.wipe:
        c.drawRect(Rect.fromLTWH(0, 0, s.width * .55, s.height), Paint()..color = N.g91.withValues(alpha: .45));
        c.drawRect(Rect.fromLTWH(s.width * .55, 0, 1.5, s.height), ink);
      case Fx.pop:
        c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: m, width: s.width * .5, height: s.height * .5), const Radius.circular(2)), ink);
      case Fx.none:
        c.drawCircle(m, r, ink);
    }
  }

  @override
  bool shouldRepaint(_TilePaint o) => o.fx != fx || o.seed != seed || o.v != v;
}

/// A loop clock: one controller, a builder that gets t in 0..1. Not running = held at [rest].
class _Loop extends StatefulWidget {
  const _Loop({required this.builder, this.running = true, this.rest = .0});
  final Widget Function(BuildContext, double t) builder;
  final bool running;
  final double rest;
  @override
  State<_Loop> createState() => _LoopState();
}

class _LoopState extends State<_Loop> with SingleTickerProviderStateMixin {
  late final AnimationController _c = AnimationController(vsync: this, duration: const Duration(milliseconds: 1200));
  @override
  void initState() {
    super.initState();
    if (widget.running) _c.repeat();
  }

  @override
  void didUpdateWidget(_Loop o) {
    super.didUpdateWidget(o);
    if (widget.running && !_c.isAnimating) {
      _c.repeat();
    } else if (!widget.running && _c.isAnimating) {
      _c.stop();
    }
  }

  @override
  void dispose() {
    _c.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => AnimatedBuilder(animation: _c, builder: (context, _) => widget.builder(context, widget.running ? _c.value : widget.rest));
}

// ================================================================================ small parts

const _clear = Color(0x00000000);

/// A count or a time in the mono face: 10 px, the one numeric style of these panels.
TextStyle _num(Color c) => T.value(c).copyWith(fontSize: 10);

/// Hover shell. [force] pins the hover look (the Parts sheets show it beside the rest look); null = follow the pointer.
class _H extends StatefulWidget {
  const _H({required this.builder, this.onTap, this.onDouble, this.onSecondary, this.cursor = SystemMouseCursors.click, this.force});
  final Widget Function(BuildContext, bool hover) builder;
  final VoidCallback? onTap, onDouble, onSecondary;
  final MouseCursor cursor;
  final bool? force;
  @override
  State<_H> createState() => _HState();
}

class _HState extends State<_H> {
  bool _h = false;
  @override
  Widget build(BuildContext context) => MouseRegion(
        cursor: widget.cursor,
        onEnter: (_) => setState(() => _h = true),
        onExit: (_) => setState(() => _h = false),
        child: GestureDetector(behavior: HitTestBehavior.opaque, onTap: widget.onTap, onDoubleTap: widget.onDouble, onSecondaryTap: widget.onSecondary, child: widget.builder(context, widget.force ?? _h)),
      );
}

/// Hover, keyboard focus and pressed in one shell for the small buttons (Clear, +N hidden tags, the active chip). hover / focus / pressed pin a state (Parts); [onTap] null = disabled: no state shows, arrow cursor.
/// The look every one of them shares: focus = a 1 px g63 edge, pressed = g20 fill and a 1 px accent line at 50% on the foot.
class _Ctl extends StatefulWidget {
  const _Ctl({required this.builder, this.onTap, this.hover, this.focus, this.pressed});
  final Widget Function(BuildContext, bool hover, bool focus, bool pressed) builder;
  final VoidCallback? onTap;
  final bool? hover, focus, pressed;
  @override
  State<_Ctl> createState() => _CtlState();
}

class _CtlState extends State<_Ctl> {
  bool _h = false, _f = false, _p = false;
  @override
  Widget build(BuildContext context) {
    final off = widget.onTap == null;
    return FocusableActionDetector(
      enabled: !off,
      onShowHoverHighlight: (v) => setState(() => _h = v),
      onShowFocusHighlight: (v) => setState(() => _f = v),
      mouseCursor: off ? SystemMouseCursors.basic : SystemMouseCursors.click,
      actions: {ActivateIntent: CallbackAction<ActivateIntent>(onInvoke: (_) {
        widget.onTap?.call();
        return null;
      })},
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: widget.onTap,
        onTapDown: off ? null : (_) => setState(() => _p = true),
        onTapUp: (_) => setState(() => _p = false),
        onTapCancel: () => setState(() => _p = false),
        child: widget.builder(context, !off && (widget.hover ?? _h), !off && (widget.focus ?? _f), !off && (widget.pressed ?? _p)),
      ),
    );
  }
}

/// The shared button body: [fill], an optional 1 px [edge] drawn over it, a pressed accent line on the foot.
Widget _ctlBody({required double height, required EdgeInsets pad, required Color fill, required Color edge, required bool pressed, required Widget child}) => ClipRRect(
      borderRadius: BorderRadius.circular(4),
      child: AnimatedContainer(
        duration: Mo.dur,
        curve: Mo.ease,
        height: height,
        padding: pad,
        color: fill,
        foregroundDecoration: BoxDecoration(borderRadius: BorderRadius.circular(4), border: Border.all(color: edge)),
        child: Stack(alignment: Alignment.center, children: [
          child,
          Positioned(left: 0, right: 0, bottom: 0, height: 1, child: ColoredBox(color: pressed ? Role.selected.withValues(alpha: .5) : _clear)),
        ]),
      ),
    );

/// A key mark: 16 px tall, 6 px sides, g20 (g26 when it is the live one).
class _Kbd extends StatelessWidget {
  const _Kbd(this.text, {this.on = false});
  final String text;
  final bool on;
  @override
  Widget build(BuildContext context) => Container(
        height: 16,
        constraints: const BoxConstraints(minWidth: 18),
        padding: const EdgeInsets.symmetric(horizontal: 6),
        decoration: BoxDecoration(color: on ? N.g26 : N.g20, borderRadius: BorderRadius.circular(3)),
        child: Center(widthFactor: 1, child: Text(text, style: _num(on ? N.g95 : N.g76))),
      );
}

/// A word that acts: g76 at rest, g95 and underlined on hover.
class _Word extends StatelessWidget {
  const _Word(this.text, {this.onTap, this.style, this.hover, this.rule = false});
  final String text;
  final VoidCallback? onTap;
  final TextStyle? style;
  final bool? hover;

  /// A dotted g44 underline at rest: the word is a control (it drops a filter), not a caption.
  final bool rule;
  @override
  Widget build(BuildContext context) => _H(
        onTap: onTap,
        force: hover,
        builder: (context, h) => Text(text, maxLines: 1, overflow: TextOverflow.ellipsis, style: (style ?? T.label(N.g76)).copyWith(color: h ? N.g95 : (style?.color ?? N.g76), decoration: h || rule ? TextDecoration.underline : TextDecoration.none, decorationStyle: h ? TextDecorationStyle.solid : TextDecorationStyle.dotted, decorationColor: h ? N.g95 : N.g44)),
      );
}

/// The one remove mark: a 14 px multiplication sign centred in a [w] x 20 target (g63 at rest, g95 on hover). In a search field w = 18, the width of the "/" key mark, so both share one centre and one right inset.
class _Cross extends StatelessWidget {
  const _Cross({this.onTap, this.hover, this.w = 16});
  final VoidCallback? onTap;
  final bool? hover;
  final double w;
  @override
  Widget build(BuildContext context) => _H(onTap: onTap, force: hover, builder: (context, h) => Container(width: w, height: 20, alignment: Alignment.center, child: Text('×', style: T.name(h ? N.g95 : N.g63).copyWith(fontSize: 14, height: 1))));
}

/// A 2 px selection tick inset in the left gutter (x 2 to 4, never on the panel edge); everything else in a row starts at x 10.
class _GutterTick extends StatelessWidget {
  const _GutterTick(this.on, {this.height = 16});
  final bool on;
  final double height;
  @override
  Widget build(BuildContext context) => Positioned(left: 2, top: 0, bottom: 0, width: 2, child: Center(child: AnimatedContainer(duration: Mo.dur, curve: Mo.ease, width: 2, height: height, decoration: BoxDecoration(color: on ? N.g95 : _clear, borderRadius: BorderRadius.circular(1)))));
}

class _Sec extends StatelessWidget {
  const _Sec(this.title, {this.right, this.pad = const EdgeInsets.fromLTRB(10, 10, 10, 4)});
  final String title;
  final Widget? right;
  final EdgeInsets pad;
  @override
  Widget build(BuildContext context) => Padding(
        padding: pad,
        child: Row(crossAxisAlignment: CrossAxisAlignment.center, children: [
          Expanded(child: Text(title.toUpperCase(), maxLines: 1, overflow: TextOverflow.ellipsis, style: T.micro(N.g76).copyWith(letterSpacing: .8))),
          ?right,
        ]),
      );
}

/// The foot line: what just happened (one fact, no instruction) and, on the right, the keys that belong to this panel as key marks.
class _Status extends StatelessWidget {
  const _Status(this.text, {this.keys = const []});
  final String text;
  final List<String> keys;
  @override
  Widget build(BuildContext context) => Container(
        height: 26,
        padding: const EdgeInsets.symmetric(horizontal: 10),
        decoration: const BoxDecoration(color: N.g07, border: Border(top: BorderSide(color: N.rowLine))),
        child: Row(children: [
          Expanded(child: Text(text, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g76))),
          for (final k in keys) Padding(padding: const EdgeInsets.only(left: 4), child: _Kbd(k)),
        ]),
      );
}

/// A note that stops something: g10 or g13 ground, a 2 px danger bar on the left, 8 px all round.
class _Notice extends StatelessWidget {
  const _Notice(this.text, {this.fill = N.g10});
  final String text;
  final Color fill;
  @override
  Widget build(BuildContext context) => Container(
        padding: const EdgeInsets.all(8),
        decoration: BoxDecoration(color: fill, borderRadius: BorderRadius.circular(4), border: Border(left: BorderSide(color: Role.error, width: 2))),
        child: Row(crossAxisAlignment: CrossAxisAlignment.start, children: [const ErrMark(size: 10, gap: 6), Expanded(child: Text(text, style: T.label(N.g91).copyWith(height: 1.35)))]),
      );
}

/// The quiet text button of these panels, 24 px: g13 at rest, g20 on hover, a g63 edge when focused by keyboard; onTap null = disabled (g10 ground, g63 word, no hover). [primary] is one grey step up. hover / focus pin a state (Parts).
class _Btn extends StatelessWidget {
  const _Btn(this.text, {this.onTap, this.primary = false, this.hover, this.focus});
  final String text;
  final VoidCallback? onTap;
  final bool primary;
  final bool? hover, focus;
  @override
  Widget build(BuildContext context) {
    final off = onTap == null;
    return Press(
      onTap: onTap,
      cursor: off ? SystemMouseCursors.basic : SystemMouseCursors.click,
      builder: (context, h, f, _) {
        final hv = !off && (hover ?? h), fc = !off && (focus ?? f);
        // Disabled = no fill, a dashed outline instead of a solid edge, a g63 word (shape as well as tone).
        return CustomPaint(
          foregroundPainter: off ? const DashedBox(N.g44, radius: 4) : null,
          child: AnimatedContainer(
            duration: Mo.dur,
            curve: Mo.ease,
            height: 24,
            padding: const EdgeInsets.symmetric(horizontal: 12),
            decoration: BoxDecoration(color: off ? N.g10 : (primary ? (hv ? N.g26 : N.g20) : (hv ? N.g20 : N.g13)), borderRadius: BorderRadius.circular(4), border: Border.all(color: off ? _clear : (fc ? N.g63 : (primary ? N.g26 : N.g20)))),
            child: Center(widthFactor: 1, child: Text(text, style: T.label(off ? N.g63 : N.g91).copyWith(fontWeight: FontWeight.w500))),
          ),
        );
      },
    );
  }
}

/// Name with the matched query tokens brighter.
class _Hl extends StatelessWidget {
  const _Hl(this.text, this.tokens, {this.on = false}) : sub = false;

  /// The second line (Japanese name and category): matched letters are bold there too, so a match that lives in the category still shows why the row is listed.
  const _Hl.sub(this.text, this.tokens) : on = false, sub = true;
  final String text;
  final List<String> tokens;
  final bool on, sub;
  @override
  Widget build(BuildContext context) {
    final base = sub ? T.label(N.g76) : T.name(on ? N.g95 : N.g91), hit = sub ? T.label(N.g91).copyWith(fontWeight: FontWeight.w700) : T.name(N.g100).copyWith(fontWeight: FontWeight.w700);
    if (tokens.isEmpty) return Text(text, maxLines: 1, overflow: TextOverflow.ellipsis, style: sub ? T.label(N.g76) : T.name(on ? N.g95 : N.g91));
    final low = text.toLowerCase(), mark = List<bool>.filled(text.length, false);
    for (final t in tokens) {
      var i = low.indexOf(t);
      while (i >= 0) {
        for (var k = i; k < i + t.length && k < mark.length; k++) {
          mark[k] = true;
        }
        i = low.indexOf(t, i + t.length);
      }
    }
    final spans = <TextSpan>[];
    var k = 0;
    while (k < text.length) {
      var e = k;
      while (e < text.length && mark[e] == mark[k]) {
        e++;
      }
      spans.add(TextSpan(text: text.substring(k, e), style: mark[k] ? hit : base));
      k = e;
    }
    return Text.rich(TextSpan(children: spans), maxLines: 1, overflow: TextOverflow.ellipsis);
  }
}

class _Empty extends StatelessWidget {
  const _Empty(this.title, this.lines, {this.actions = const []});
  final String title;
  final List<String> lines;
  final List<Widget> actions;
  @override
  Widget build(BuildContext context) => _Fit(
        natural: null,
        child: Center(
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 20),
          child: Flex(direction: Axis.vertical, mainAxisSize: MainAxisSize.min, children: [
            const Glyph(G.effect, size: 26, color: N.g56),
            const SizedBox(height: 12),
            if (title.contains('match')) Row(mainAxisSize: MainAxisSize.min, children: [ErrMark(gap: 6, color: Role.warning), Flexible(child: Text(title, textAlign: TextAlign.center, style: T.name(N.g91).copyWith(height: 1.3)))]) else Text(title, textAlign: TextAlign.center, style: T.name(N.g91).copyWith(height: 1.3)),
            const SizedBox(height: 8),
            for (final l in lines) Padding(padding: const EdgeInsets.only(bottom: 4), child: Text(l, textAlign: TextAlign.center, style: T.label(N.g76).copyWith(height: 1.35))),
            if (actions.isNotEmpty) ...[const SizedBox(height: 8), Wrap(alignment: WrapAlignment.center, spacing: 8, runSpacing: 8, children: actions)],
          ]),
        ),
      ),
      );
}

/// A list row for an item, 34 px. One left rhythm: the 2 px tick lives in the gutter (x 2 to 4), thumbnail / name / everything else starts at x 10. Selected = g20 + tick; hover = g15.
/// A filter that already fixes a fact must not be repeated by its rows: [showKind] false drops the right-hand kind label, [showCat] false drops the category from the second line.
/// A media row never repeats its media kind ("Video") in the second line: the thumbnail badge and the right-hand label say it.
class _ItemRow extends StatelessWidget {
  const _ItemRow(this.it, {this.selected = false, this.tokens = const [], this.onTap, this.onUse, this.right, this.hover, this.showKind = true, this.showCat = true});
  final It it;
  final bool selected, showKind, showCat;
  final List<String> tokens;
  final VoidCallback? onTap, onUse;
  final Widget? right;
  final bool? hover;
  @override
  Widget build(BuildContext context) {
    final sub = (it.cat == it.place || !showCat || it.asset != null) ? it.jp : '${it.jp} · ${it.cat}';
    final tail = right ?? (showKind && _QuietKind.of(context) != it.kind ? Text(it.kind, style: T.micro(N.g76)) : null);
    return _H(
      onTap: onTap,
      onDouble: onUse,
      force: hover,
      builder: (context, hover) => AnimatedContainer(
        duration: Mo.dur,
        curve: Mo.ease,
        height: 34,
        color: selected ? N.g20 : (hover ? N.g15 : _clear),
        child: Stack(fit: StackFit.expand, children: [
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 10),
            child: Row(crossAxisAlignment: CrossAxisAlignment.center, children: [
              _Thumb(it, w: 34, h: 22),
              const SizedBox(width: 8),
              Expanded(
                child: Flex(direction: Axis.vertical, mainAxisAlignment: MainAxisAlignment.center, crossAxisAlignment: CrossAxisAlignment.start, children: [
                  _Hl(it.name, tokens, on: selected),
                  const SizedBox(height: 4),
                  _Hl.sub(sub, tokens),
                ]),
              ),
              if (tail != null) ...[const SizedBox(width: 8), tail],
            ]),
          ),
          _GutterTick(selected),
        ]),
      ),
    );
  }
}

/// The foot of a result list (g76, 26 px, hairline above). It says exactly one of two things: "End of results", or "N more (where)" with N = only what lies OUTSIDE the count line's scope, so it can never repeat the count line's numbers.
/// A list whose count line already says it ("8 of 72", "72 items") gets no foot at all (see [_Results.foot]).
class _EndLine extends StatelessWidget {
  const _EndLine(this.more, {this.where = 'in other places'});
  final int more;
  final String where;
  @override
  Widget build(BuildContext context) => Container(
        height: 26,
        padding: const EdgeInsets.symmetric(horizontal: 10),
        alignment: Alignment.centerLeft,
        decoration: const BoxDecoration(border: Border(top: BorderSide(color: N.rowLine))),
        child: Text(more == 0 ? 'End of results' : '$more more $where', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g76)),
      );
}

/// What a panel frame tells its body: whether the frame is only as tall as its content ("Fit list"), where the body is (so the chrome around the list can be measured), and where to report the height the content needs.
class _FitScope extends InheritedWidget {
  const _FitScope({required this.fit, required this.bodyKey, required this.report, required super.child});
  final bool fit;
  final GlobalKey bodyKey;
  final void Function(double? need) report;
  static _FitScope? of(BuildContext c) => c.dependOnInheritedWidgetOfExactType<_FitScope>();
  @override
  bool updateShouldNotify(_FitScope o) => o.fit != fit;
}

/// The one flexible region of a panel (it sits where an Expanded child sits). Outside a fitting frame it is its child. In "Fit list" mode it takes only [natural] px (top-aligned) and reports frame height = everything around it + [natural];
/// natural null = the content wants the whole frame (an empty state): the frame keeps its default height.
class _Fit extends StatelessWidget {
  const _Fit({required this.natural, required this.child});
  final double? natural;
  final Widget child;
  @override
  Widget build(BuildContext context) {
    final s = _FitScope.of(context);
    if (s == null || !s.fit) return child;
    return LayoutBuilder(builder: (context, box) {
      final e = box.maxHeight, n = natural;
      WidgetsBinding.instance.addPostFrameCallback((_) {
        final bodyH = (s.bodyKey.currentContext?.findRenderObject() as RenderBox?)?.size.height;
        if (bodyH != null) s.report(n == null ? null : bodyH - e + n);
      });
      if (n == null) return child;
      return Align(alignment: Alignment.topCenter, child: SizedBox(height: math.min(n, e), child: child));
    });
  }
}

/// The results area. Fit list (default): the list is exactly its rows and the end line sits directly under the last row. 700 tall: the list is top-aligned and the end line is pinned at the foot; the free space stays empty (no hint block).
/// [listH] is the height the list takes (default: [rows] x 34). [foot] false = no foot (the count line above already states what is shown and what is not); the frame then ends right under the last row.
class _Results extends StatelessWidget {
  const _Results({required this.list, required this.rows, required this.more, this.where = 'in other places', this.listH, this.foot = true});
  final Widget list;
  final int rows, more;
  final String where;
  final double? listH;
  final bool foot;
  @override
  Widget build(BuildContext context) => _Fit(
        natural: (listH ?? rows * 34.0) + (foot ? 26 : 0),
        child: Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          Expanded(child: Align(alignment: Alignment.topCenter, child: list)),
          if (foot) _EndLine(more, where: where),
        ]),
      );
}

/// The one kind a list is mostly made of: more than 80% of at least 5 rows share it. Those rows drop the kind word; the exceptions keep it and the count line says it once ("mostly effects"). null = mixed.
String? _dominant(List<It> r, String Function(It) of) {
  if (r.length < 5) return null;
  final n = <String, int>{};
  for (final i in r) {
    n[of(i)] = (n[of(i)] ?? 0) + 1;
  }
  final top = n.entries.reduce((a, b) => a.value >= b.value ? a : b);
  return top.value * 5 > r.length * 4 ? top.key : null;
}

/// "mostly effects" for a count line, from the kind of the rows ([_dominant] on [It.kind]); null when mixed.
String? _mostly(List<It> r) {
  final k = _dominant(r, (i) => i.kind);
  return k == null ? null : 'mostly ${k == 'Media' ? 'media' : '${k.toLowerCase()}s'}';
}

/// Tells the rows below which kind is said once in the count line, so they leave their own kind word off (only the exceptions show it).
class _QuietKind extends InheritedWidget {
  const _QuietKind(this.kind, {required super.child});
  final String? kind;
  static String? of(BuildContext c) => c.dependOnInheritedWidgetOfExactType<_QuietKind>()?.kind;
  @override
  bool updateShouldNotify(_QuietKind o) => o.kind != kind;
}

/// Where a list has nothing yet: an empty slot (28 px, dashed g38 outline) with one short word in g63, not a sentence.
class _HintRow extends StatelessWidget {
  const _HintRow(this.text);
  final String text;
  @override
  Widget build(BuildContext context) => Padding(
        padding: const EdgeInsets.fromLTRB(10, 4, 10, 4),
        child: CustomPaint(
          foregroundPainter: const DashedBox(N.g38, radius: 4),
          child: Container(height: 28, alignment: Alignment.centerLeft, padding: const EdgeInsets.symmetric(horizontal: 8), child: Text(text, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g63))),
        ),
      );
}

// ---- the search field (EditableText; Esc and arrows are the owner's, through the focus node). Edge: rest g20, hover g38, focus g63.

class _Search extends StatelessWidget {
  const _Search({required this.controller, required this.node, this.hint = 'Search effects, presets, media', this.onChanged, this.slash = true, this.autofocus = false, this.trailing, this.focusLook, this.hoverLook, this.clearHover});
  final TextEditingController controller;
  final FocusNode node;
  final String hint;
  final ValueChanged<String>? onChanged;
  final bool slash, autofocus;
  final Widget? trailing;

  /// Pins the focused look (Parts sheets); null = follow the focus node.
  final bool? focusLook;

  /// Pins the hover look (Parts sheets): edge g38 between rest g20 and focus g63; null = follow the pointer.
  final bool? hoverLook;

  /// Pins the hover look of the clear mark (Parts sheets).
  final bool? clearHover;

  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: Listenable.merge([controller, node]),
        builder: (context, _) {
          final f = focusLook ?? node.hasFocus, empty = controller.text.isEmpty;
          return _H(
            onTap: node.requestFocus,
            cursor: SystemMouseCursors.text,
            force: hoverLook,
            builder: (context, hv) => AnimatedContainer(
              duration: Mo.dur,
              curve: Mo.ease,
              height: 28,
              padding: const EdgeInsets.symmetric(horizontal: 8),
              decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(4), border: Border.all(color: f ? N.g63 : (hv ? N.g38 : N.g20))),
              child: Row(crossAxisAlignment: CrossAxisAlignment.center, children: [
                Expanded(
                  child: LayoutBuilder(builder: (context, box) {
                    // Typed text: at rest it ends in an ellipsis like the hint; while editing the field scrolls, so the right edge fades instead.
                    final over = !empty && (TextPainter(text: TextSpan(text: controller.text, style: T.name(N.g95)), maxLines: 1, textDirection: TextDirection.ltr)..layout()).width > box.maxWidth;
                    final rest = over && !f;
                    final editable = EditableText(
                      controller: controller,
                      focusNode: node,
                      autofocus: autofocus,
                      style: T.name(N.g95),
                      cursorColor: N.g95,
                      backgroundCursorColor: N.g44,
                      selectionColor: Role.selected.withValues(alpha: .35),
                      cursorWidth: 1,
                      maxLines: 1,
                      onChanged: onChanged,
                    );
                    return Stack(alignment: Alignment.centerLeft, children: [
                      if (empty) IgnorePointer(child: Text(hint, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(N.g76))),
                      if (rest) IgnorePointer(child: Text(controller.text, maxLines: 1, softWrap: false, overflow: TextOverflow.ellipsis, style: T.name(N.g95))),
                      Offstage(
                        offstage: rest,
                        child: over
                            ? ShaderMask(
                                blendMode: BlendMode.dstIn,
                                shaderCallback: (r) => ui.Gradient.linear(Offset(r.width - 16, 0), Offset(r.width, 0), const [Color(0xFFFFFFFF), Color(0x00FFFFFF)]),
                                child: editable,
                              )
                            : editable,
                      ),
                    ]);
                  }),
                ),
                if (trailing != null) ...[const SizedBox(width: 8), trailing!],
                if (!empty)
                  _Cross(w: 18, hover: clearHover, onTap: () {
                    controller.clear();
                    onChanged?.call('');
                    node.requestFocus();
                  })
                else if (slash && !f) ...[const SizedBox(width: 8), const SizedBox(width: 18, child: _Kbd('/'))],
              ]),
            ),
          );
        },
      );
}

/// One tab word, a choice control (DESIGN.md selected state): 22 px tall, 8 px sides; rest g63 text, hover g15, selected = g20 pill + g95 text + a 1 px accent underline. Used by the tab trough and the menu's family bar.
class _TabCell extends StatelessWidget {
  const _TabCell(this.text, {this.on = false, this.hover = false});
  final String text;
  final bool on, hover;
  @override
  Widget build(BuildContext context) => ClipRRect(
        borderRadius: BorderRadius.circular(4),
        child: AnimatedContainer(
          duration: Mo.dur,
          curve: Mo.ease,
          height: 22,
          padding: const EdgeInsets.symmetric(horizontal: 8),
          color: on ? N.g20 : (hover ? N.g15 : _clear),
          child: Stack(alignment: Alignment.center, children: [
            Text(text, maxLines: 1, softWrap: false, style: T.micro(on ? N.g95 : N.g76)),
            Positioned(left: 0, right: 0, bottom: 0, height: 1, child: ColoredBox(color: on ? Role.selected : _clear)),
          ]),
        ),
      );
}

/// Esc: first press clears the text, second leaves the field. Returns null when the key is not Esc.
KeyEventResult _esc(KeyEvent e, TextEditingController c, FocusNode n, void Function(String) say, {VoidCallback? cleared}) {
  if (e is! KeyDownEvent || e.logicalKey != LogicalKeyboardKey.escape) return KeyEventResult.ignored;
  if (c.text.isNotEmpty) {
    c.clear();
    cleared?.call();
    say('Cleared');
  } else {
    n.unfocus();
    say('Left the field');
  }
  return KeyEventResult.handled;
}

bool _isDown(KeyEvent e, LogicalKeyboardKey k) => (e is KeyDownEvent || e is KeyRepeatEvent) && e.logicalKey == k;

// ================================================================================ stage (caption outside, panel inside)

enum Hab { habit, addition, departs }

class _Caption extends StatelessWidget {
  const _Caption(this.id, this.title, this.hab, this.contract);
  final String id, title, contract;
  final Hab hab;
  @override
  Widget build(BuildContext context) {
    final (word, jp) = switch (hab) { Hab.habit => ('HABIT', '手癖'), Hab.addition => ('ADDITION', '追加'), Hab.departs => ('DEPARTS', '手癖から外れる') };
    return Flex(direction: Axis.vertical, mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
      Row(crossAxisAlignment: CrossAxisAlignment.center, children: [
        Text(id, style: T.value(N.g95).copyWith(fontWeight: FontWeight.w700)),
        const SizedBox(width: 8),
        Flexible(child: Text(title, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.title())),
        const SizedBox(width: 8),
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 4),
          decoration: BoxDecoration(color: N.g15, borderRadius: BorderRadius.circular(4), border: Border(bottom: BorderSide(color: hab == Hab.departs ? Role.selected : N.g26))),
          child: Text('$word $jp', style: T.micro(hab == Hab.departs ? N.g95 : N.g76)),
        ),
      ]),
      const SizedBox(height: 6),
      Text(contract, style: T.label(N.g76).copyWith(height: 1.35)),
    ]);
  }
}

class _Stage extends StatefulWidget {
  const _Stage({super.key, required this.id, required this.title, required this.hab, required this.contract, required this.width, required this.body, this.fit = true});
  final String id, title, contract;
  final Hab hab;
  final double width;
  final Widget body;

  /// "Fit list": the frame is as tall as its content (no floor, max 700). false = the 700 tall stress frame.
  final bool fit;
  @override
  State<_Stage> createState() => _StageState();
}

class _StageState extends State<_Stage> {
  final _bodyKey = GlobalKey();
  double? _need;

  void _report(double? n) {
    if (!mounted) return;
    final same = n == null ? _need == null : (_need != null && (n - _need!).abs() < .5);
    if (!same) setState(() => _need = n);
  }

  @override
  Widget build(BuildContext context) => ColoredBox(
        color: N.g07,
        child: LayoutBuilder(builder: (context, box) {
          final avail = box.maxHeight.isFinite ? box.maxHeight : 840.0;
          final legacy = (avail - 24 - 78 - 10).clamp(440.0, 700.0);
          // Fit: the body reports what its list needs (+2 for the frame edge). A body with nothing to fit (an empty state, tiles) keeps the default height.
          final h = !widget.fit ? 700.0 : (_need == null ? legacy : (_need! + 2).clamp(0.0, 700.0));
          return Center(
            child: SingleChildScrollView(
              padding: const EdgeInsets.all(12),
              child: Flex(direction: Axis.vertical, mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
                SizedBox(width: math.max(widget.width, 560), child: _Caption(widget.id, widget.title, widget.hab, widget.contract)),
                const SizedBox(height: 10),
                SizedBox(
                  width: widget.width,
                  height: h,
                  child: DecoratedBox(
                    decoration: BoxDecoration(color: N.g10, border: Border.all(color: N.g20)),
                    child: ClipRect(child: _FitScope(fit: widget.fit, bodyKey: _bodyKey, report: _report, child: KeyedSubtree(key: _bodyKey, child: widget.body))),
                  ),
                ),
              ]),
            ),
          );
        }),
      );
}

/// One use case = one panel draft. Two knobs are on every one: Panel width (Browser 320 / narrow dock 232) and Panel height (Fit list = the frame is as tall as its content, no floor / 700 tall = the stress frame).
WidgetbookUseCase _uc(String id, String title, Hab hab, String contract, Widget Function(BuildContext c, double w) build, {bool narrow = false}) => WidgetbookUseCase(
      name: '$id $title',
      builder: (c) {
        final w = c.knobs.object.dropdown<double>(label: 'Panel width', options: const [320.0, 232.0], initialOption: narrow ? 232.0 : 320.0, labelBuilder: (v) => v > 300 ? 'Browser 320' : 'Narrow dock 232');
        final fit = c.knobs.object.dropdown<String>(label: 'Panel height', options: const ['Fit list (default)', '700 tall'], initialOption: 'Fit list (default)', labelBuilder: (v) => v) != '700 tall';
        return _Stage(key: ValueKey('$id $title'), id: id, title: title, hab: hab, contract: contract, width: w, fit: fit, body: build(c, w));
      },
    );


// ================================================================================ shared list + tabs

void _revealAt(ScrollController sc, double top, double bot) {
  WidgetsBinding.instance.addPostFrameCallback((_) {
    if (!sc.hasClients) return;
    final vp = sc.position.viewportDimension;
    if (top < sc.offset) {
      sc.jumpTo(top);
    } else if (bot > sc.offset + vp) {
      sc.jumpTo(bot - vp);
    }
  });
}

/// A flat list of section headings and selectable rows (variable height). Selection is counted over rows only.
class _Ent {
  const _Ent.head(this.title, {this.right}) : row = null, h = 26;
  const _Ent.row(this.row, {this.h = 34}) : title = null, right = null;
  final String? title, right;
  final Widget Function(bool sel)? row;
  final double h;
  bool get isHead => row == null;
}

double _entH(List<_Ent> es) => es.fold<double>(0, (a, e) => a + e.h);

int _rowCount(List<_Ent> e) => e.where((x) => !x.isHead).length;

(double, double) _entSpan(List<_Ent> es, int rowIdx) {
  var y = 0.0, n = 0;
  for (final e in es) {
    if (!e.isHead) {
      if (n == rowIdx) return (y, y + e.h);
      n++;
    }
    y += e.h;
  }
  return (y, y);
}

class _FlatList extends StatelessWidget {
  const _FlatList({required this.entries, required this.sel, required this.sc, this.fade = false});
  final bool fade;
  final List<_Ent> entries;
  final int sel;
  final ScrollController sc;
  @override
  Widget build(BuildContext context) {
    var n = 0;
    final idx = [for (final e in entries) e.isHead ? -1 : n++];
    final list = ListView.builder(
      controller: sc,
      padding: EdgeInsets.zero,
      itemCount: entries.length + 1,
      itemBuilder: (context, i) {
        if (i == entries.length) return const SizedBox(height: 12); // air, so the last row is never cut by the footer
        final e = entries[i];
        return SizedBox(height: e.h, child: e.isHead ? _Sec(e.title!, right: e.right == null ? null : Text(e.right!, style: T.label(N.g76))) : e.row!(idx[i] == sel));
      },
    );
    return fade ? _FadeBottom(child: list) : list;
  }
}

/// Tabs for a place: words in a g07 trough, scrolls instead of shrinking the text. Tapping the selected one again clears it (-1 = all places). [hoverIndex] pins one tab's hover look (Parts).
class _Tabs extends StatelessWidget {
  const _Tabs(this.items, this.index, this.onChanged, {this.toggle = true, this.hoverIndex, this.chevron = false});
  final bool chevron;
  final List<String> items;
  final int index;
  final ValueChanged<int> onChanged;
  final bool toggle;
  final int? hoverIndex;
  @override
  Widget build(BuildContext context) => Container(
        height: 26,
        width: double.infinity,
        padding: const EdgeInsets.all(2),
        decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(6)),
        child: _FadeScroll(
          index: index,
          chevron: chevron,
          children: [
            for (var i = 0; i < items.length; i++)
              _H(
                onTap: () => onChanged(toggle && i == index ? -1 : i),
                force: hoverIndex == null ? null : hoverIndex == i,
                builder: (context, h) => _TabCell(items[i], on: i == index, hover: h),
              ),
          ],
        ),
      );
}

/// A row of cells that scrolls sideways, never cuts a word without saying so: a 16 px fade on the side that has more, and the chosen cell is kept in view.
class _FadeScroll extends StatefulWidget {
  const _FadeScroll({required this.children, this.index = -1, this.pad = EdgeInsets.zero, this.chevron = false});
  final bool chevron;
  final List<Widget> children;
  final int index;
  final EdgeInsets pad;
  @override
  State<_FadeScroll> createState() => _FadeScrollState();
}

class _FadeScrollState extends State<_FadeScroll> {
  final _sc = ScrollController();
  late final List<GlobalKey> _keys = List.generate(widget.children.length, (_) => GlobalKey());

  @override
  void initState() {
    super.initState();
    _sc.addListener(() => setState(() {}));
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) {
        setState(() {});
        _show(false);
      }
    });
  }

  @override
  void didUpdateWidget(_FadeScroll o) {
    super.didUpdateWidget(o);
    if (o.index != widget.index) WidgetsBinding.instance.addPostFrameCallback((_) => _show(true));
  }

  void _show(bool animate) {
    final i = widget.index;
    if (!mounted || i < 0 || i >= _keys.length) return;
    final c = _keys[i].currentContext;
    if (c != null) Scrollable.ensureVisible(c, alignment: .5, duration: animate ? Mo.dur : Duration.zero, curve: Mo.ease);
  }

  @override
  void dispose() {
    _sc.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final has = _sc.hasClients && _sc.position.hasContentDimensions;
    final more = has && _sc.position.maxScrollExtent > 1, left = more && _sc.offset > 1, right = more && _sc.offset < _sc.position.maxScrollExtent - 1;
    final row = SingleChildScrollView(
      controller: _sc,
      scrollDirection: Axis.horizontal,
      physics: const ClampingScrollPhysics(),
      padding: widget.pad,
      child: Row(mainAxisSize: MainAxisSize.min, children: [
        for (var i = 0; i < widget.children.length; i++) ...[
          if (i > 0) const SizedBox(width: 2),
          KeyedSubtree(key: _keys[i], child: widget.children[i]),
        ],
      ]),
    );
    if (!left && !right) return row;
    final masked = ShaderMask(
      blendMode: BlendMode.dstIn,
      shaderCallback: (r) {
        final f = math.min(.5, (widget.chevron ? 28 : 16) / r.width);
        return ui.Gradient.linear(Offset.zero, Offset(r.width, 0), [Color(left ? 0x00FFFFFF : 0xFFFFFFFF), const Color(0xFFFFFFFF), const Color(0xFFFFFFFF), Color(right ? 0x00FFFFFF : 0xFFFFFFFF)], [0, f, 1 - f, 1]);
      },
      child: row,
    );
    if (!widget.chevron) return masked;
    // A › (or ‹) on the side that has more, so the cut is a cue and not a clipped word.
    return Stack(children: [
      Positioned.fill(child: masked),
      if (right) Positioned(right: 4, top: 0, bottom: 0, child: IgnorePointer(child: Center(child: Text('›', style: T.name(N.g76).copyWith(fontSize: 14, height: 1))))),
      if (left) Positioned(left: 4, top: 0, bottom: 0, child: IgnorePointer(child: Center(child: Text('‹', style: T.name(N.g76).copyWith(fontSize: 14, height: 1))))),
    ]);
  }
}

/// A scrolling list that says there is more below: a 20 px fade at the foot while the list can still scroll down, so a row is never cut without a cue.
class _FadeBottom extends StatefulWidget {
  const _FadeBottom({required this.child, this.top = false});
  final Widget child;
  /// Also fade the top edge while the list is scrolled away from its start (never at rest).
  final bool top;
  @override
  State<_FadeBottom> createState() => _FadeBottomState();
}

class _FadeBottomState extends State<_FadeBottom> {
  bool _more = false, _before = false;
  bool _on(ScrollMetrics m) {
    if (m.axis != Axis.vertical) return false;
    final more = m.extentAfter > 1, before = widget.top && m.extentBefore > 1;
    if ((more != _more || before != _before) && mounted) setState(() { _more = more; _before = before; });
    return false;
  }

  @override
  Widget build(BuildContext context) => NotificationListener<ScrollMetricsNotification>(
        onNotification: (n) => _on(n.metrics),
        child: NotificationListener<ScrollNotification>(
          onNotification: (n) => _on(n.metrics),
          child: ShaderMask(
            blendMode: BlendMode.dstIn,
            shaderCallback: (r) {
              final f = math.min(.5, 20 / r.height), t = math.min(.25, 12 / r.height);
              return ui.Gradient.linear(Offset.zero, Offset(0, r.height), [Color(_before ? 0x00FFFFFF : 0xFFFFFFFF), const Color(0xFFFFFFFF), const Color(0xFFFFFFFF), Color(_more ? 0x00FFFFFF : 0xFFFFFFFF)], [0, _before ? t : 0, 1 - f, 1]);
            },
            child: widget.child,
          ),
        ),
      );
}

/// The kind most of these rows share (always one, unlike [_dominant]): the rows of that kind leave the kind word off and only the exceptions say it.
String? _majority(Iterable<It> r) {
  final n = <String, int>{};
  for (final i in r) {
    n[i.kind] = (n[i.kind] ?? 0) + 1;
  }
  if (n.isEmpty) return null;
  return n.entries.reduce((a, b) => a.value >= b.value ? a : b).key;
}

Widget _pad(Widget c, {double l = 10, double t = 0, double r = 10, double b = 0}) => Padding(padding: EdgeInsets.fromLTRB(l, t, r, b), child: SizedBox(width: double.infinity, child: c));

// ================================================================================ B3-a  Find: unified query (always-on field)

class _FindA extends StatefulWidget {
  const _FindA({super.key, this.query = '', this.grouped = false, this.tab = -1, this.clearHover});
  final String query;
  final bool grouped;
  final int tab;

  /// Pins the hover look of the field's clear mark (a use case that shows it).
  final bool? clearHover;
  @override
  State<_FindA> createState() => _FindAState();
}

class _FindAState extends State<_FindA> {
  late final _c = TextEditingController(text: widget.query);
  late final _n = FocusNode(onKeyEvent: _key);
  late final _root = FocusNode(onKeyEvent: _rootKey);
  final _sc = ScrollController();
  late int _tab = widget.tab;
  int _sel = 0;
  String _say = 'Layer 3 · Title';

  Iterable<It> get _pool => _tab < 0 ? kItems : kItems.where((i) => i.place == _places[_tab]);
  List<It> get _res => _match(_pool, _c.text);

  /// One place that is more than 80% of an all-places result is said once in the count line ("mostly effects"); only the exceptions keep their place word.
  String? _dom(List<It> r) => widget.grouped || _tab >= 0 ? null : _dominant(r, (i) => i.place);

  List<_Ent> _entries() {
    final r = _res, toks = _tokens(_c.text), dom = _dom(r);
    final out = <_Ent>[];
    var k = 0;
    _Ent row(It it) {
      final i = k++;
      // Grouped under a place heading the place is already said, so no label on the right. Across places the place stays on the right and a media row's second line never repeats its kind.
      return _Ent.row((sel) => _ItemRow(it, selected: sel, tokens: toks, right: widget.grouped || it.place == dom ? const SizedBox.shrink() : Text(it.place, style: T.micro(N.g76)), onTap: () => setState(() => _sel = i), onUse: () => _use(it)));
    }

    if (widget.grouped) {
      for (final p in _places) {
        final g = r.where((i) => i.place == p).toList();
        if (g.isEmpty) continue;
        out.add(_Ent.head(p, right: '${g.length}'));
        for (final it in g) {
          out.add(row(it));
        }
      }
    } else {
      for (final it in r) {
        out.add(row(it));
      }
    }
    return out;
  }

  void _use(It it) => setState(() => _say = 'Applied “${it.name}” to Layer 3');

  KeyEventResult _key(FocusNode n, KeyEvent e) {
    final es = _entries(), cnt = _rowCount(es);
    if (_isDown(e, LogicalKeyboardKey.arrowDown) || _isDown(e, LogicalKeyboardKey.arrowUp)) {
      setState(() => _sel = cnt == 0 ? 0 : (_sel + (e.logicalKey == LogicalKeyboardKey.arrowDown ? 1 : -1)).clamp(0, cnt - 1));
      final (t, b) = _entSpan(es, _sel);
      _revealAt(_sc, t, b);
      return KeyEventResult.handled;
    }
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.enter && cnt > 0) {
      final it = _res[_sel.clamp(0, _res.length - 1)];
      _use(it);
      return KeyEventResult.handled;
    }
    return _esc(e, _c, n, (s) => setState(() => _say = s), cleared: () => _sel = 0);
  }

  KeyEventResult _rootKey(FocusNode n, KeyEvent e) {
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.slash) {
      _n.requestFocus();
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  @override
  void dispose() {
    _c.dispose();
    _n.dispose();
    _root.dispose();
    _sc.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Focus(
        focusNode: _root,
        autofocus: true,
        child: ListenableBuilder(
          listenable: _c,
          builder: (context, _) {
            final r = _res, es = _entries(), empty = r.isEmpty;
            final sg = empty ? _suggest(_c.text) : const <String>[];
            final dom = _dom(r), outside = _tab >= 0 ? _match(kItems, _c.text).length - r.length : 0;
            return Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
              _pad(_Tabs(_places, _tab, (i) => setState(() {
                    _tab = i;
                    _sel = 0;
                  })), t: 10),
              const SizedBox(height: 8),
              _pad(_Search(controller: _c, node: _n, clearHover: widget.clearHover, onChanged: (_) => setState(() => _sel = 0))),
              _pad(
                Row(children: [
                  Expanded(child: Text(empty ? 'No results' : '${r.length} of ${_pool.length}  ·  ${_tab < 0 ? 'no tab on: all places' : _places[_tab]}${dom == null ? '' : '  ·  mostly ${dom.toLowerCase()}'}', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g76))),
                ]),
                t: 8,
                b: 6,
              ),
              Expanded(
                child: empty
                    ? _Empty('Nothing matches “${_c.text}”', [
                        _tab < 0 ? 'Searched all 4 places, ${kItems.length} items.' : 'Searched only ${_places[_tab]}.',
                      ], actions: [
                        _Btn('Clear search', onTap: () => setState(() => _c.clear())),
                        if (_tab >= 0) _Btn('Search all places', onTap: () => setState(() => _tab = -1)),
                        for (final s in sg) _Btn('Did you mean “$s”?', onTap: () => setState(() => _c.text = s)),
                      ])
                    : _Results(
                        list: _FlatList(entries: es, sel: _sel, sc: _sc),
                        rows: r.length,
                        listH: _entH(es),
                        more: outside,
                        where: 'in other places',
                        foot: outside > 0,
                      ),
              ),
              _Status(_say),
            ]);
          },
        ),
      );
}

// ================================================================================ B3-b  Tab menu (type-ahead at the cursor)

const _fams = ['All', 'Blur', 'Color', 'Glow', 'Distort', 'Stylize', 'Time', 'Title', 'Ease'];

class _TabMenu extends StatefulWidget {
  const _TabMenu({super.key, required this.w, this.target = 0, this.text = ''});
  final double w;
  final int target; // 0 layer, 1 none, 2 key
  final String text;
  @override
  State<_TabMenu> createState() => _TabMenuState();
}

class _TabMenuState extends State<_TabMenu> {
  late final _c = TextEditingController(text: widget.text);
  late final _n = FocusNode(onKeyEvent: _key);
  late final _root = FocusNode(onKeyEvent: _rootKey);
  bool _open = true;
  int _fam = 0, _sel = 0;
  String? _done;

  String get _tgt => const ['Layer 3 · Title', '', 'Key · Position X'][widget.target];

  List<It> get _cands {
    var p = kItems.where((i) => i.place != 'Project' && (widget.target == 1 || (widget.target == 2 ? i.applies != 'layer' : i.applies != 'key')));
    if (_fam > 0) p = p.where((i) => i.cat == _fams[_fam]);
    final r = _match(p, _c.text);
    if (_c.text.trim().isEmpty) {
      r.sort((a, b) {
        final ra = a.recent < 0 ? 1 << 20 : a.recent, rb = b.recent < 0 ? 1 << 20 : b.recent;
        return ra != rb ? ra.compareTo(rb) : b.uses.compareTo(a.uses);
      });
    }
    return r;
  }

  void _close(String msg) {
    setState(() {
      _open = false;
      _done = msg;
    });
    _root.requestFocus();
  }

  KeyEventResult _key(FocusNode n, KeyEvent e) {
    final cs = _cands;
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.tab) {
      setState(() {
        _fam = (_fam + 1) % _fams.length;
        _sel = 0;
      });
      return KeyEventResult.handled;
    }
    if (_isDown(e, LogicalKeyboardKey.arrowDown) || _isDown(e, LogicalKeyboardKey.arrowUp)) {
      setState(() => _sel = cs.isEmpty ? 0 : (_sel + (e.logicalKey == LogicalKeyboardKey.arrowDown ? 1 : -1)).clamp(0, math.min(cs.length, 8) - 1));
      return KeyEventResult.handled;
    }
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.enter) {
      if (widget.target == 1) {
        setState(() => _done = null);
        return KeyEventResult.handled; // stays open: the empty-target message is already visible
      }
      if (cs.isNotEmpty) _close('Applied “${cs[_sel.clamp(0, cs.length - 1)].name}” to $_tgt');
      return KeyEventResult.handled;
    }
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape) {
      _close('Closed, nothing written');
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  KeyEventResult _rootKey(FocusNode n, KeyEvent e) {
    if (!_open && e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.tab) {
      setState(() {
        _open = true;
        _c.clear();
        _fam = 0;
        _sel = 0;
      });
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  @override
  void dispose() {
    _c.dispose();
    _n.dispose();
    _root.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Focus(
        focusNode: _root,
        autofocus: false,
        child: Stack(fit: StackFit.expand, children: [
          const Art(Asset('', Kind.image, 3)),
          const ColoredBox(color: Color(0x73131313)),
          // the stage fragment behind the menu: the selected layer with its name
          Positioned(left: 28, right: 28, top: 28, height: 104, child: DecoratedBox(decoration: BoxDecoration(border: Border.all(color: widget.target == 1 ? N.g44 : Role.selected), borderRadius: BorderRadius.circular(2)), child: Stack(children: [
            Align(alignment: Alignment.topLeft, child: Container(padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 4), color: widget.target == 1 ? N.g26 : Role.selected, child: Text(widget.target == 1 ? 'nothing selected' : (widget.target == 2 ? 'Position X · key at 00:12' : 'Layer 3 · Title'), style: T.micro(N.g100)))),
            if (widget.target != 2) Center(child: Text('光の速さで', style: T.title().copyWith(fontSize: 22))),
          ]))),
          // The menu scrolls inside the frame (never a cut row); the closed card keeps the Tab key mark where the menu was.
          if (_open) Positioned(left: 12, right: 12, top: 144, bottom: 34, child: LayoutBuilder(builder: (context, box) => Align(alignment: Alignment.topCenter, child: SingleChildScrollView(child: _menu(box.maxHeight))))) else Positioned(left: 12, right: 12, top: 152, child: _closedCard()),
          Positioned(left: 0, right: 0, bottom: 0, child: _Status('', keys: _open ? const ['Esc'] : const [])),
        ]),
      );

  Widget _closedCard() => _ClosedCard(_done ?? 'Closed');

  /// [avail] is the height the frame leaves for the card: the rows shown are only as many as fit whole (28 px each), and the "+N more" line says the rest, so no row is ever cut.
  Widget _menu(double avail) {
    final noTarget = widget.target == 1;
    final cs = _cands, empty = _c.text.trim().isEmpty;
    final fit = ((avail - 146 - (noTarget ? 44 : 0)) / 28).floor().clamp(3, 8);
    final shown = cs.take(fit).toList();
    return Container(
      decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(6), border: Border.all(color: N.g26), boxShadow: const [BoxShadow(color: Color(0x80000000), blurRadius: 18, offset: Offset(0, 6))]),
      child: Flex(direction: Axis.vertical, mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
        Padding(padding: const EdgeInsets.fromLTRB(10, 10, 10, 6), child: _Search(controller: _c, node: _n, autofocus: true, slash: false, hint: noTarget ? 'Add effect (no target)' : 'Add effect to ${widget.target == 2 ? 'Position X' : 'Layer 3'}', onChanged: (_) => setState(() => _sel = 0), trailing: const _Kbd('Tab'))),
        SizedBox(
          height: 26,
          child: _FadeScroll(
            index: _fam,
            chevron: true,
            pad: const EdgeInsets.symmetric(horizontal: 10),
            children: [
              for (var i = 0; i < _fams.length; i++)
                _H(
                  onTap: () => setState(() {
                    _fam = i;
                    _sel = 0;
                    _n.requestFocus();
                  }),
                  builder: (context, h) => Center(child: _TabCell(_fams[i], on: i == _fam, hover: h)),
                ),
            ],
          ),
        ),
        if (noTarget)
          const Padding(padding: EdgeInsets.fromLTRB(10, 2, 10, 4), child: _Notice(_noTargetText)),
        _Sec(empty ? 'Recent' : (cs.isEmpty ? 'No match' : 'Matches · ${cs.length}'), pad: const EdgeInsets.fromLTRB(10, 6, 10, 2)),
        if (shown.isEmpty) Padding(padding: const EdgeInsets.fromLTRB(10, 4, 10, 10), child: Text('Nothing in ${_fams[_fam]} matches “${_c.text}”', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g76).copyWith(height: 1.35))),
        for (var i = 0; i < shown.length; i++)
          _H(
            onTap: () => setState(() => _sel = i),
            onDouble: () => _close('Applied “${shown[i].name}” to $_tgt'),
            builder: (context, h) => _MenuRow(shown[i].name, tokens: _tokens(_c.text), selected: i == _sel, hover: h, tail: i == _sel && !noTarget ? null : (empty && shown[i].recent >= 0 ? _ago(shown[i].recent) : (_fam > 0 ? '' : shown[i].cat))),
          ),
        if (cs.length > shown.length) _MoreLine('+${cs.length - shown.length} more'),
        const SizedBox(height: 6),
      ]),
    );
  }
}

const _noTargetText = 'No target selected';

/// A menu row, 28 px: tail = a quiet word on the right, '' = nothing (the family tab already says it), null = the Enter key mark (the row Enter would use).
class _MenuRow extends StatelessWidget {
  const _MenuRow(this.name, {this.tokens = const [], this.selected = false, this.hover = false, this.tail});
  final String name;
  final List<String> tokens;
  final bool selected, hover;
  final String? tail;
  @override
  Widget build(BuildContext context) => Container(
        height: 28,
        color: selected ? N.g20 : (hover ? N.g15 : null),
        child: Stack(fit: StackFit.expand, children: [
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 10),
            child: Row(children: [
              Expanded(child: _Hl(name, tokens, on: selected)),
              const SizedBox(width: 8),
              if (tail == null) const _Kbd('Enter') else if (tail!.isNotEmpty) Text(tail!, style: T.label(N.g76)),
            ]),
          ),
          _GutterTick(selected, height: 12),
        ]),
      );
}

/// What stays where the menu was after it closes: the Tab key mark and one line of what happened.
class _ClosedCard extends StatelessWidget {
  const _ClosedCard(this.text);
  final String text;
  @override
  Widget build(BuildContext context) => Container(
        padding: const EdgeInsets.all(12),
        decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(6), border: Border.all(color: N.g26)),
        child: Row(children: [const _Kbd('Tab'), const SizedBox(width: 8), Expanded(child: Text(text, style: T.name(N.g91).copyWith(height: 1.3)))]),
      );
}

// ================================================================================ B3-c  Prefix: one field, the first character picks the area

const _actions = [
  ('Apply to selection', '選択に適用', 'Enter'),
  ('Reset all effects on Layer 3', 'Layer 3 のエフェクトをリセット', 'Cmd Shift R'),
  ('Add keyframe at playhead', '現在位置にキーを打つ', 'K'),
  ('Align to composition center', 'コンポジションの中央に整列', 'Cmd Alt C'),
  ('Open Export…', '書き出し…', 'Cmd E'),
  ('Toggle Inspector', 'Inspector の表示切替', 'Cmd 2'),
  ('Duplicate layer', 'レイヤーを複製', 'Cmd D'),
  ('Solo layer', 'ソロ', 'S'),
  ('Pre-compose…', 'プリコンポーズ…', 'Cmd Shift C'),
  ('Undo', '取り消し', 'Cmd Z'),
];

const _layers = [
  ('Title', 'Text'), ('キャッチコピー', 'Text'), ('BG Sky', 'Image'), ('Logo mark', 'Shape'), ('Subtitle (EN)', 'Text'), ('字幕 02', 'Text'), ('Particles A', 'Shape'),
  ('Particles B', 'Shape'), ('Light leak 03', 'Video'), ('Camera 1', 'Camera'), ('Null: Rig', 'Null'), ('city_pass.mov', 'Video'), ('Shape 14', 'Shape'), ('Lower third bar', 'Shape'),
  ('Lower third name', 'Text'), ('Vignette', 'Shape'), ('Adjustment: Grade', 'Adjust'), ('Wind pad', 'Audio'), ('Kick loop', 'Audio'), ('Sphere tour', 'Video'),
];

Map<String, int> _tagCounts() {
  final m = <String, int>{};
  for (final i in kItems) {
    for (final t in i.tags) {
      m[t] = (m[t] ?? 0) + 1;
    }
  }
  return m;
}

/// The line that closes a capped section: how many are hidden, as "+N more" and nothing else (24 px, text at the same x 10 as the rows' names).
class _MoreLine extends StatelessWidget {
  const _MoreLine(this.text);
  final String text;
  @override
  Widget build(BuildContext context) => Padding(padding: const EdgeInsets.only(left: 10, top: 4), child: Text(text, style: T.label(N.g76)));
}

/// The line under the prefix field: which area the first mark chose, and the three marks (the live one lit).
class _AreaLine extends StatelessWidget {
  const _AreaLine(this.area, this.mark);
  final String area, mark;
  @override
  Widget build(BuildContext context) => Row(children: [
        Text('Area', style: T.label(N.g76)),
        const SizedBox(width: 8),
        Text(area, style: T.name(N.g95)),
        const Spacer(),
        for (final m in const ['#', '>', '@']) Padding(padding: const EdgeInsets.only(left: 4), child: _Kbd(m, on: mark == m)),
      ]);
}

class _Prefix extends StatefulWidget {
  const _Prefix({super.key, this.query = ''});
  final String query;
  @override
  State<_Prefix> createState() => _PrefixState();
}

class _PrefixState extends State<_Prefix> {
  late final _c = TextEditingController(text: widget.query);
  late final _n = FocusNode(onKeyEvent: _key);
  final _sc = ScrollController();
  int _sel = 0;
  String _say = 'Layer 3 · Title';

  String get _q => _c.text;
  String get _mark => _q.isNotEmpty && '#>@'.contains(_q[0]) ? _q[0] : '';
  String get _rest => _mark.isEmpty ? _q : _q.substring(1).trim();

  List<_Ent> _entries() {
    final toks = _tokens(_rest), out = <_Ent>[];
    var k = 0;
    _Ent tag(MapEntry<String, int> e) {
      final i = k++;
      return _Ent.row((s) => _PlainRow(title: '#${e.key}', sub: 'tag', right: '${e.value}', count: true, selected: s, onTap: () => setState(() => _sel = i), toks: toks));
    }

    _Ent act((String, String, String) a) {
      final i = k++;
      return _Ent.row((s) => _PlainRow(title: a.$1, sub: a.$2, right: a.$3, kbd: true, selected: s, onTap: () => setState(() => _sel = i), toks: toks));
    }

    _Ent lay((String, String) l) {
      final i = k++;
      return _Ent.row((s) => _PlainRow(title: l.$1, sub: 'Layer', right: l.$2, selected: s, onTap: () => setState(() => _sel = i), toks: toks));
    }

    _Ent item(It it) {
      final i = k++;
      return _Ent.row((s) => _ItemRow(it, selected: s, tokens: toks, showKind: false, onTap: () => setState(() => _sel = i)));
    }

    final tags = _tagCounts().entries.where((e) => toks.every(e.key.contains)).toList()..sort((a, b) => b.value.compareTo(a.value));
    final acts = _actions.where((a) => toks.every((t) => '${a.$1} ${a.$2}'.toLowerCase().contains(t))).toList();
    final lays = _layers.where((l) => toks.every((t) => '${l.$1} ${l.$2}'.toLowerCase().contains(t))).toList();
    final items = _match(kItems, _rest);

    void section(String name, List<_Ent> rows, int total, int cap) {
      if (rows.isEmpty) return;
      out.add(_Ent.head(name, right: '$total'));
      out.addAll(rows.take(cap));
      if (total > cap) out.add(_Ent.row((_) => _MoreLine('+${total - cap} more'), h: 24));
    }

    if (_q.isEmpty) {
      out.add(const _Ent.head('Marks'));
      for (final m in const [('#', 'tag', '#soft'), ('>', 'action', '>reset'), ('@', 'layer', '@title')]) {
        final i = k++;
        out.add(_Ent.row((s) => _PlainRow(title: '${m.$1}  ${m.$2}', sub: m.$3, right: '›', selected: s, onTap: () => setState(() {
              _sel = i;
              _c.value = TextEditingValue(text: m.$1, selection: const TextSelection.collapsed(offset: 1));
              _n.requestFocus();
            }))));
      }
    }
    switch (_mark) {
      case '#':
        section('Tags', [for (final e in tags) tag(e)], tags.length, 99);
      case '>':
        section('Actions', [for (final a in acts) act(a)], acts.length, 99);
      case '@':
        section('Layers', [for (final l in lays) lay(l)], lays.length, 99);
      default:
        final it = [for (final i in items.take(4)) item(i)];
        section('Effects, presets, media', it, items.length, 4);
        section('Tags', [for (final e in tags.take(2)) tag(e)], tags.length, 2);
        section('Actions', [for (final a in acts.take(2)) act(a)], acts.length, 2);
        section('Layers', [for (final l in lays.take(2)) lay(l)], lays.length, 2);
    }
    return out;
  }

  KeyEventResult _key(FocusNode n, KeyEvent e) {
    final es = _entries(), cnt = _rowCount(es);
    if (_isDown(e, LogicalKeyboardKey.arrowDown) || _isDown(e, LogicalKeyboardKey.arrowUp)) {
      setState(() => _sel = cnt == 0 ? 0 : (_sel + (e.logicalKey == LogicalKeyboardKey.arrowDown ? 1 : -1)).clamp(0, cnt - 1));
      final (t, b) = _entSpan(es, _sel);
      _revealAt(_sc, t, b);
      return KeyEventResult.handled;
    }
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.enter && cnt > 0) {
      setState(() => _say = 'Row ${_sel + 1} used');
      return KeyEventResult.handled;
    }
    return _esc(e, _c, n, (s) => setState(() => _say = s));
  }

  String _area() => switch (_mark) { '#' => 'tags', '>' => 'actions', '@' => 'layers', _ => 'all areas' };

  @override
  void dispose() {
    _c.dispose();
    _n.dispose();
    _sc.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: _c,
        builder: (context, _) {
          final es = _entries();
          return Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            _pad(_Search(controller: _c, node: _n, hint: 'Search, or use # > @', onChanged: (_) => setState(() => _sel = 0)), t: 10),
            _pad(
              _AreaLine(_area(), _mark),
              t: 8,
              b: 4,
            ),
            Expanded(
              child: es.isEmpty
                  ? _Empty('Nothing in ${_area()} for “$_rest”', const [], actions: [_Btn('Search all areas', onTap: () => setState(() => _c.text = _rest))])
                  : _Fit(natural: _entH(es), child: _FlatList(entries: es, sel: _sel, sc: _sc)),
            ),
            _Status(_say),
          ]);
        },
      );
}

class _PlainRow extends StatelessWidget {
  const _PlainRow({required this.title, required this.sub, required this.right, required this.selected, this.onTap, this.toks = const [], this.kbd = false, this.count = false, this.hover});
  final String title, sub, right;

  /// Three kinds of right-hand word, three looks: [kbd] a key mark (the shortcut), [count] the mono number, else a quiet kind word.
  final bool selected, kbd, count;
  final bool? hover;
  final VoidCallback? onTap;
  final List<String> toks;
  @override
  Widget build(BuildContext context) => _H(
        onTap: onTap,
        force: hover,
        builder: (context, hover) => AnimatedContainer(
          duration: Mo.dur,
          curve: Mo.ease,
          color: selected ? N.g20 : (hover ? N.g15 : _clear),
          child: Stack(fit: StackFit.expand, children: [
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 10),
              child: Row(crossAxisAlignment: CrossAxisAlignment.center, children: [
                Expanded(
                  child: Flex(direction: Axis.vertical, mainAxisAlignment: MainAxisAlignment.center, crossAxisAlignment: CrossAxisAlignment.start, children: [
                    _Hl(title, toks, on: selected),
                    const SizedBox(height: 4),
                    Text(sub, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g76)),
                  ]),
                ),
                const SizedBox(width: 8),
                if (kbd) _Kbd(right) else Text(right, style: count ? _num(N.g76) : T.micro(N.g63)),
              ]),
            ),
            _GutterTick(selected),
          ]),
        ),
      );
}

// ================================================================================ B3-d  Empty query: recent, then what fits the selection

/// The quiet reason a row is listed (g63, at most 92 px, ellipsis).
class _Why extends StatelessWidget {
  const _Why(this.text);
  final String text;
  @override
  Widget build(BuildContext context) => ConstrainedBox(constraints: const BoxConstraints(maxWidth: 92), child: Text(text, maxLines: 1, overflow: TextOverflow.ellipsis, textAlign: TextAlign.right, style: T.label(N.g76)));
}

class _Freq extends StatefulWidget {
  const _Freq({super.key, required this.sel, required this.history});
  final int sel; // 0 none, 1 layer (Text), 2 key
  final bool history;
  @override
  State<_Freq> createState() => _FreqState();
}

class _FreqState extends State<_Freq> {
  final _c = TextEditingController();
  late final _n = FocusNode(onKeyEvent: _key);
  final _sc = ScrollController();
  int _sel = 0;
  late String _say = widget.sel == 0 ? 'No selection' : (widget.sel == 1 ? 'Layer 3 · Title' : 'Key · Position X');

  List<It> get _all => kItems.where((i) => i.place != 'Project').toList();

  (List<It>, List<It>, List<It>) _groups() {
    bool ok(It i) => widget.sel == 0 || (widget.sel == 2 ? i.applies != 'layer' : i.applies != 'key');
    final recent = widget.history ? (_all.where((i) => i.recent >= 0 && ok(i)).toList()..sort((a, b) => a.recent.compareTo(b.recent))).take(5).toList() : <It>[];
    var fit = <It>[];
    if (widget.sel == 1) {
      fit = _all.where((i) => i.applies != 'key' && !recent.contains(i)).toList()..sort((a, b) => ((b.tags.contains('title') ? 100 : 0) + b.uses).compareTo((a.tags.contains('title') ? 100 : 0) + a.uses));
      fit = fit.take(5).toList();
    } else if (widget.sel == 2) {
      fit = _all.where((i) => i.applies != 'layer' && !recent.contains(i)).toList();
    }
    final rest = _all.where((i) => !recent.contains(i) && !fit.contains(i) && ok(i)).toList()..sort((a, b) => widget.history ? b.uses.compareTo(a.uses) : 0);
    return (recent, fit, rest);
  }

  String _fitWhy(It i) => widget.sel == 2 ? 'curve for keys' : (i.tags.contains('title') ? 'made for text' : 'used on ${i.uses ~/ 4 + 1} text layers');

  List<_Ent> _entries() {
    final out = <_Ent>[];
    var k = 0;
    _Ent row(It it, String why) {
      final i = k++;
      return _Ent.row((s) => _ItemRow(it, selected: s, tokens: _tokens(_c.text), right: why.isEmpty ? null : _Why(why), onTap: () => setState(() => _sel = i), onUse: () => setState(() => _say = 'Applied “${it.name}”')));
    }

    if (_c.text.trim().isNotEmpty) {
      final r = _match(_all, _c.text);
      out.add(_Ent.head('Results', right: '${r.length}'));
      for (final it in r) {
        out.add(row(it, ''));
      }
      return out;
    }
    final (recent, fit, rest) = _groups();
    out.add(_Ent.head('Recent', right: widget.history ? 'all projects' : null));
    if (recent.isEmpty) {
      out.add(_Ent.row((s) => const _HintRow('None yet'), h: 36));
    }
    for (final it in recent) {
      out.add(row(it, _ago(it.recent)));
    }
    if (widget.sel == 0) {
      out.add(const _Ent.head('Fits your selection'));
      out.add(_Ent.row((s) => const _HintRow('No selection'), h: 36));
    } else {
      out.add(_Ent.head('Fits ${widget.sel == 1 ? 'Layer 3 · Title (Text)' : 'Key · Position X'}'));
      for (final it in fit) {
        out.add(row(it, _fitWhy(it)));
      }
    }
    out.add(_Ent.head('All others', right: '${rest.length}'));
    for (final it in rest) {
      out.add(row(it, widget.history && it.uses > 0 ? '${it.uses}×' : ''));
    }
    return out;
  }

  KeyEventResult _key(FocusNode n, KeyEvent e) {
    final es = _entries(), cnt = _rowCount(es);
    if (_isDown(e, LogicalKeyboardKey.arrowDown) || _isDown(e, LogicalKeyboardKey.arrowUp)) {
      setState(() => _sel = cnt == 0 ? 0 : (_sel + (e.logicalKey == LogicalKeyboardKey.arrowDown ? 1 : -1)).clamp(0, cnt - 1));
      final (t, b) = _entSpan(es, _sel);
      _revealAt(_sc, t, b);
      return KeyEventResult.handled;
    }
    return _esc(e, _c, n, (s) => setState(() => _say = s));
  }

  @override
  void dispose() {
    _c.dispose();
    _n.dispose();
    _sc.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: _c,
        builder: (context, _) => Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          _pad(_Search(controller: _c, node: _n, hint: 'Add effect…', onChanged: (_) => setState(() => _sel = 0)), t: 10, b: 4),
          Expanded(child: _Fit(natural: _entH(_entries()), child: _QuietKind(_majority(_all), child: _FlatList(entries: _entries(), sel: _sel, sc: _sc, fade: true)))),
          _Status(_say),
        ]),
      );
}

// ================================================================================ B2/B4  narrowing, and showing that a filter is on

const _groups = <String, List<String>>{
  'Kind': ['Effect', 'Preset', 'Media'],
  'Category': ['Blur', 'Color', 'Glow', 'Distort', 'Stylize', 'Time', 'Generate', 'Title', 'Motion', 'Ease'],
  'Character': ['Soft', 'Glitch', 'Retro', 'Bright', 'Grade', 'Moving', 'Loop'],
  'Source': ['Built-in', 'Mine', 'Project'],
};

bool _in(It i, String g, String v) => switch (g) {
      'Kind' => i.kind == v,
      'Category' => i.cat == v,
      'Source' => i.source == v,
      _ => i.tags.contains(v.toLowerCase()),
    };

typedef _Sel = Map<String, Set<String>>;

bool _pass(It i, _Sel s, {String? skip}) => s.entries.every((e) => e.key == skip || e.value.isEmpty || e.value.any((v) => _in(i, e.key, v)));

int _nFilters(_Sel s) => s.values.fold(0, (a, b) => a + b.length);

/// A filter that fixes a fact says it once: with exactly one Kind chosen the rows drop their kind label, with exactly one Category chosen their second line drops the category. (Two kinds chosen: the label tells them apart, so it stays.)
bool _kindFixed(Set<String> kinds) => kinds.length == 1;

Widget _maybeFade(bool on, Widget w) => on ? _FadeBottom(child: w) : w;

/// A tiny list builder for the filter panels: rows are 34 px, top-aligned. The count line above ("8 of 72") already says what is hidden, so there is no foot ([quiet]); a panel whose count line shows only the shown number passes quiet false and gets "N more hidden by the filters".
/// When one kind is more than 80% of the rows, the rows leave it off and only the exceptions say it.
Widget _filtered(List<It> r, {required int hidden, required Widget Function(It, int) row, ScrollController? sc, bool quiet = true, bool fade = false, bool useMajority = false}) => _QuietKind(
      useMajority ? _majority(r) : _dominant(r, (i) => i.kind),
      child: _Results(
        list: _maybeFade(fade, ListView.builder(controller: sc, padding: EdgeInsets.zero, itemExtent: 34, itemCount: r.length, itemBuilder: (context, i) => row(r[i], i))),
        rows: r.length,
        more: hidden,
        where: 'hidden by the filters',
        foot: !quiet && hidden > 0,
      ),
    );

/// A filter chip, 22 px, on the panel plate g10: rest g15 (visibly above the plate), hover g20, pressed g20 + the accent line at 50%, on g26 + a 1 px accent line, on + hover g26 + the line + a g63 edge,
/// keyboard focus a 1 px g63 edge. Count 0 = disabled (never shown at rest: the group folds these into one "+N hidden tags" word; when shown, no fill, g63 word, a dashed g56 hairline, arrow cursor).
/// A long word never widens the group: it takes the room it is given and ends in an ellipsis. hover / focus / pressed pin a state (Parts).
class _ChipBtn extends StatefulWidget {
  const _ChipBtn(this.text, {this.count, this.on = false, this.onTap, this.hover, this.focus, this.pressed});
  final String text;
  final int? count;
  final bool on;
  final VoidCallback? onTap;
  final bool? hover, focus, pressed;
  @override
  State<_ChipBtn> createState() => _ChipBtnState();
}

class _ChipBtnState extends State<_ChipBtn> {
  bool _h = false, _f = false, _p = false;
  @override
  Widget build(BuildContext context) {
    final w = widget, zero = w.count == 0 && !w.on;
    final hv = !zero && (w.hover ?? _h), fc = !zero && (w.focus ?? _f), pr = !zero && (w.pressed ?? _p);
    final fill = zero ? _clear : (w.on ? N.g26 : (pr || hv ? N.g20 : N.g15));
    final line = w.on ? Role.selected : (pr ? Role.selected.withValues(alpha: .5) : _clear);
    final edge = fc ? N.g63 : (w.on && hv ? N.g63 : _clear);
    final body = ClipRRect(
      borderRadius: BorderRadius.circular(4),
      child: AnimatedContainer(
        duration: Mo.dur,
        curve: Mo.ease,
        height: 22,
        padding: const EdgeInsets.symmetric(horizontal: 8),
        foregroundDecoration: BoxDecoration(borderRadius: BorderRadius.circular(4), border: Border.all(color: edge)),
        color: fill,
        child: Stack(alignment: Alignment.center, children: [
          Row(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.center, children: [
            Flexible(child: Text(w.text, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.micro(w.on ? N.g95 : (zero ? N.g63 : N.g91)))),
            if (w.count != null) ...[const SizedBox(width: 4), Text('${w.count}', style: _num(w.on ? N.g95 : N.g76))],
          ]),
          Positioned(left: 0, right: 0, bottom: 0, height: 1, child: ColoredBox(color: line)),
        ]),
      ),
    );
    return FocusableActionDetector(
      enabled: !zero,
      onShowHoverHighlight: (v) => setState(() => _h = v),
      onShowFocusHighlight: (v) => setState(() => _f = v),
      mouseCursor: zero ? SystemMouseCursors.basic : SystemMouseCursors.click,
      actions: {ActivateIntent: CallbackAction<ActivateIntent>(onInvoke: (_) {
        w.onTap?.call();
        return null;
      })},
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: zero ? null : w.onTap,
        onTapDown: zero ? null : (_) => setState(() => _p = true),
        onTapUp: (_) => setState(() => _p = false),
        onTapCancel: () => setState(() => _p = false),
        child: zero ? CustomPaint(foregroundPainter: const DashedBox(N.g56, radius: 4), child: body) : body,
      ),
    );
  }
}

/// Clear, as a button: g76 word and a g26 hairline, g20 fill on hover, pressed = g20 + the accent line at 50%, keyboard focus a 1 px g63 edge; with nothing on it is dimmed (g63 word, g20 hairline, no hover, arrow cursor). hover / focus / pressed pin a state (Parts).
class _ClearBtn extends StatelessWidget {
  const _ClearBtn({this.onTap, this.hover, this.focus, this.pressed});
  final VoidCallback? onTap;
  final bool? hover, focus, pressed;
  @override
  Widget build(BuildContext context) {
    final off = onTap == null;
    return _Ctl(
      onTap: onTap,
      hover: off ? false : hover,
      focus: focus,
      pressed: pressed,
      builder: (context, h, f, p) => _ctlBody(
        height: 20,
        pad: const EdgeInsets.symmetric(horizontal: 8),
        fill: h || p ? N.g20 : _clear,
        edge: f ? N.g63 : (off ? N.g20 : N.g26),
        pressed: p,
        child: Center(widthFactor: 1, child: Text('Clear', style: T.label(off ? N.g63 : N.g76))),
      ),
    );
  }
}

/// The results band: how many filters are on, how many results, one Clear button.
class _Band extends StatelessWidget {
  const _Band({required this.filters, required this.shown, required this.total, required this.onClear, this.extra, this.hoverClear, this.focusClear, this.pressClear, this.showFilters = true, this.showCount = true});
  final int filters, shown, total;

  /// false = the chips / slot / tab already say how many filters are on, so the band does not count them again; [showCount] false = shown and total are not restated either (only [extra]).
  final bool showFilters, showCount;
  final VoidCallback onClear;
  final bool? hoverClear, focusClear, pressClear;
  final String? extra;
  @override
  Widget build(BuildContext context) => Container(
        height: 28,
        padding: const EdgeInsets.symmetric(horizontal: 10),
        decoration: const BoxDecoration(color: N.g13, border: Border.symmetric(horizontal: BorderSide(color: N.rowLine))),
        child: Row(children: [
          if (filters == 0)
            Expanded(child: Text('$total items', style: T.label(N.g76)))
          else
            Expanded(child: _bandText()),
          const SizedBox(width: 8),
          _ClearBtn(onTap: filters > 0 ? onClear : null, hover: hoverClear, focus: focusClear, pressed: pressClear),
        ]),
      );

  Widget _bandText() {
    if (showFilters) {
      return Text.rich(TextSpan(children: [TextSpan(text: '$filters ${filters == 1 ? 'filter' : 'filters'}', style: T.name(N.g95)), TextSpan(text: '  ·  ${shown == 0 ? 'no results' : '$shown of $total'}${extra == null ? '' : '  ·  $extra'}', style: T.label(N.g76))]), maxLines: 1, overflow: TextOverflow.ellipsis);
    }
    final parts = [if (showCount) (shown == 0 ? 'no results' : '$shown of $total'), ?extra];
    return Text(parts.join('  ·  '), maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(N.g95));
  }
}

/// An active filter, as the same filled chip object B2 draws: 22 px, rest g15, hover g20, pressed g20 + the accent line at 50%, keyboard focus a 1 px g63 edge; g91 word and an x mark inside the chip (g63, g95 when the chip is hot). The whole chip drops the filter. hover / focus / pressed pin a state (Parts).
class _ActiveChip extends StatelessWidget {
  const _ActiveChip(this.text, {this.onTap, this.hover, this.focus, this.pressed});
  final String text;
  final VoidCallback? onTap;
  final bool? hover, focus, pressed;
  @override
  Widget build(BuildContext context) => _Ctl(
        onTap: onTap ?? () {},
        hover: hover,
        focus: focus,
        pressed: pressed,
        builder: (context, h, f, p) => _ctlBody(
          height: 22,
          pad: const EdgeInsets.only(left: 8, right: 2),
          fill: h || p ? N.g20 : N.g15,
          edge: f ? N.g63 : _clear,
          pressed: p,
          child: Row(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.center, children: [
            Flexible(child: Text(text, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.micro(N.g91))),
            const SizedBox(width: 2),
            SizedBox(width: 16, height: 20, child: Center(child: Text('×', style: T.name(h || p ? N.g95 : N.g63).copyWith(fontSize: 14, height: 1)))),
          ]),
        ),
      );
}

/// The tail of a tag group: "+N hidden tags" (grammatical: "+1 hidden tag") or "Hide N", as a quiet text button on the chip line: g76 word, g20 fill on hover, pressed g20 + the accent line at 50%, keyboard focus a 1 px g63 edge, a 10 px chevron (down = shows them, up = hides). hover / focus / pressed pin a state (Parts).
class _HiddenToggle extends StatelessWidget {
  const _HiddenToggle(this.n, {this.open = false, this.onTap, this.hover, this.focus, this.pressed});
  final int n;
  final bool open;
  final VoidCallback? onTap;
  final bool? hover, focus, pressed;
  @override
  Widget build(BuildContext context) => _Ctl(
        onTap: onTap ?? () {},
        hover: hover,
        focus: focus,
        pressed: pressed,
        builder: (context, h, f, p) => _ctlBody(
          height: 22,
          pad: const EdgeInsets.only(left: 6, right: 4),
          fill: h || p ? N.g20 : _clear,
          edge: f ? N.g63 : _clear,
          pressed: p,
          child: Row(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.center, children: [
            Text(open ? 'Hide $n' : '+$n hidden ${n == 1 ? 'tag' : 'tags'}', maxLines: 1, softWrap: false, style: T.micro(N.g76)),
            const SizedBox(width: 3),
            CustomPaint(size: const Size(10, 10), painter: _ChevronPaint(open, N.g76)),
          ]),
        ),
      );
}

class _ChevronPaint extends CustomPainter {
  const _ChevronPaint(this.up, this.color);
  final bool up;
  final Color color;
  @override
  void paint(Canvas c, Size s) {
    final y0 = up ? 6.5 : 3.5, y1 = up ? 3.5 : 6.5;
    c.drawPath(Path()..moveTo(2, y0)..lineTo(5, y1)..lineTo(8, y0), Paint()..color = color..style = PaintingStyle.stroke..strokeWidth = 1.3..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round);
  }

  @override
  bool shouldRepaint(_ChevronPaint o) => o.up != up || o.color != color;
}

/// One filter group: its name on the left (64 px, g63) and its chips; every chip starts at x 74 (10 + 64), the label at x 10.
/// Chips with count 0 are not drawn at rest: one trailing text button "+N hidden tags" (a chevron, g76, g20 on hover) follows the last chip on the same line when it fits, and shows them (dashed, g56); it then reads "Hide N".
/// Folded = one line ("any" or the choice) and a Show word. [hoverIndex] pins one chip's hover look; [zeroOpen] pins whether the hidden ones are shown (null = the group's own state).
class _TagGroup extends StatefulWidget {
  const _TagGroup({required this.label, required this.values, required this.on, this.counts, this.open = true, this.canHide = false, this.onTap, this.onShow, this.onHide, this.hoverIndex, this.zeroOpen, this.hoverToggle});
  final String label;
  final List<String> values;
  final Set<String> on;
  final Map<String, int>? counts;
  final bool open, canHide;
  final ValueChanged<String>? onTap;
  final VoidCallback? onShow, onHide;
  final int? hoverIndex;
  final bool? zeroOpen, hoverToggle;
  @override
  State<_TagGroup> createState() => _TagGroupState();
}

class _TagGroupState extends State<_TagGroup> {
  bool _zero = false;
  @override
  Widget build(BuildContext context) {
    final w = widget, showZero = w.zeroOpen ?? _zero;
    bool isZero(String v) => w.counts != null && w.counts![v] == 0 && !w.on.contains(v);
    final live = [for (final v in w.values) if (!isZero(v)) v], dead = [for (final v in w.values) if (isZero(v)) v];
    Widget chip(String v, int i) => _ChipBtn(v, count: w.counts?[v], on: w.on.contains(v), hover: w.hoverIndex == null ? null : w.hoverIndex == i, onTap: () => w.onTap?.call(v));
    return Padding(
      padding: const EdgeInsets.fromLTRB(10, 0, 10, 6),
      child: Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
        SizedBox(width: 64, height: 22, child: Align(alignment: Alignment.centerLeft, child: Text(w.label, style: T.label(N.g76)))),
        Expanded(
          child: w.open
              ? Wrap(spacing: 4, runSpacing: 4, children: () {
                  // Chips pack as many to a line as the width allows; the toggle is the next item after the last chip, so it sits right after it and wraps only when the width is out.
                  final chips = [for (final v in live) chip(v, w.values.indexOf(v)), if (showZero) for (final v in dead) chip(v, w.values.indexOf(v))];
                  if (dead.isEmpty) return chips;
                  return [...chips, _HiddenToggle(dead.length, open: showZero, hover: w.hoverToggle, onTap: () => setState(() => _zero = !_zero))];
                }())
              : SizedBox(height: 22, child: Row(children: [
                  Expanded(child: Text(w.on.isEmpty ? 'any' : w.on.join(', '), maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(w.on.isEmpty ? N.g76 : N.g95))),
                  const SizedBox(width: 8),
                  _Word('Show ${w.values.length}', onTap: w.onShow),
                ])),
        ),
        if (w.open && w.canHide && w.on.isEmpty) Padding(padding: const EdgeInsets.only(left: 8), child: SizedBox(height: 22, child: Center(child: _Word('Hide', onTap: w.onHide)))),
      ]),
    );
  }
}

class _TagBands extends StatefulWidget {
  const _TagBands({super.key, this.fold = false, this.add = false, this.start = const {}});
  final bool fold, add;
  final Map<String, List<String>> start;
  @override
  State<_TagBands> createState() => _TagBandsState();
}

class _TagBandsState extends State<_TagBands> {
  final _c = TextEditingController();
  late final _n = FocusNode(onKeyEvent: (n, e) => _esc(e, _c, n, (_) {}));
  late final _Sel _s = {for (final g in _groups.keys) g: <String>{...(widget.start[g] ?? const [])}};
  late final Set<String> _open = {if (!widget.fold) ..._groups.keys else 'Kind', for (final e in widget.start.entries) if (e.value.isNotEmpty) e.key};
  int _pick = 0;

  bool get _addMode => widget.add || HardwareKeyboard.instance.isMetaPressed || HardwareKeyboard.instance.isControlPressed || HardwareKeyboard.instance.isShiftPressed;

  List<It> get _res => _match(kItems.where((i) => _pass(i, _s)), _c.text);

  int _count(String g, String v) => _match(kItems.where((i) => _pass(i, _s, skip: g) && _in(i, g, v)), _c.text).length;

  void _tap(String g, String v) => setState(() {
        final cur = _s[g]!;
        if (cur.contains(v)) {
          cur.remove(v);
        } else {
          if (!_addMode) cur.clear();
          cur.add(v);
        }
        _open.add(g);
      });

  @override
  void dispose() {
    _c.dispose();
    _n.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: _c,
        builder: (context, _) {
          final r = _res, n = _nFilters(_s) + (_c.text.trim().isEmpty ? 0 : 1);
                    Widget group(String g) => _TagGroup(
                label: g,
                values: _groups[g]!,
                on: _s[g]!,
                counts: {for (final v in _groups[g]!) v: _count(g, v)},
                open: _open.contains(g),
                canHide: widget.fold,
                onTap: (v) => _tap(g, v),
                onShow: () => setState(() => _open.add(g)),
                onHide: () => setState(() => _open.remove(g)),
              );

          final worst = <String>[];
          if (r.isEmpty && n > 0) {
            for (final e in _s.entries) {
              for (final v in e.value) {
                final t = {for (final k in _s.keys) k: {..._s[k]!}}..[e.key]!.remove(v);
                final k = _match(kItems.where((i) => _pass(i, t)), _c.text).length;
                worst.add('$k\t${e.key}: $v');
              }
            }
            worst.sort((a, b) => int.parse(b.split('\t').first).compareTo(int.parse(a.split('\t').first)));
            worst.removeWhere((w) => w.startsWith('0\t'));
          }
          return Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            _pad(_Search(controller: _c, node: _n, onChanged: (_) => setState(() {})), t: 10, b: 8),
            for (final g in _groups.keys) group(g),
            _Band(filters: n, shown: r.length, total: kItems.length, extra: _kindFixed(_s['Kind']!) ? null : _mostly(r), onClear: () => setState(() {
                  for (final s in _s.values) {
                    s.clear();
                  }
                  _c.clear();
                })),
            Expanded(
              child: r.isEmpty
                  ? _Empty('No item has all $n filters', const [], actions: [
                      for (final w in worst.take(2)) _Btn('Remove ${w.split('\t').last} → ${w.split('\t').first}', onTap: () => setState(() {
                            final p = w.split('\t').last.split(': ');
                            _s[p[0]]!.remove(p[1]);
                          })),
                      _Btn('Clear $n filters', onTap: () => setState(() {
                            for (final s in _s.values) {
                              s.clear();
                            }
                            _c.clear();
                          })),
                    ])
                  : _filtered(r, hidden: kItems.length - r.length, row: (it, i) => _ItemRow(it, selected: i == _pick, tokens: _tokens(_c.text), showKind: !_kindFixed(_s['Kind']!), showCat: !_kindFixed(_s['Category']!), onTap: () => setState(() => _pick = i))),
            ),
            _Status(_addMode ? 'Add' : 'Only', keys: const ['Cmd', 'Shift']),
          ]);
        },
      );
}

// ---- B4 alone: the band and what it tells

class _BandDemo extends StatefulWidget {
  const _BandDemo({super.key, required this.n, required this.text, this.hoverClear, this.hoverX});
  final int n;
  final String text;
  final bool? hoverClear, hoverX;
  @override
  State<_BandDemo> createState() => _BandDemoState();
}

class _BandDemoState extends State<_BandDemo> {
  static const _all = [('Kind', 'Effect'), ('Category', 'Glow'), ('Character', 'Soft'), ('Source', 'Built-in')];
  late List<(String, String)> _on = _all.take(widget.n).toList();
  late String _text = widget.text;

  List<It> get _res {
    final s = <String, Set<String>>{for (final g in _groups.keys) g: {}};
    for (final f in _on) {
      s[f.$1]!.add(f.$2);
    }
    return _match(kItems.where((i) => _pass(i, s)), _text);
  }

  @override
  Widget build(BuildContext context) {
    final r = _res, n = _on.length + (_text.isEmpty ? 0 : 1);
    return Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      _pad(
        Container(
          height: 28,
          padding: const EdgeInsets.symmetric(horizontal: 8),
          decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(4), border: Border.all(color: N.g20)),
          child: Row(children: [
            Expanded(child: Text(_text.isEmpty ? 'Search effects, presets, media' : _text, style: T.name(_text.isEmpty ? N.g76 : N.g95))),
            if (_text.isNotEmpty) _Cross(hover: widget.hoverX, onTap: () => setState(() => _text = '')),
          ]),
        ),
        t: 10,
        b: 8,
      ),
      _Band(filters: n, shown: r.length, total: kItems.length, extra: _on.any((f) => f.$1 == 'Kind') ? null : _mostly(r), hoverClear: widget.hoverClear, onClear: () => setState(() {
            _on = [];
            _text = '';
          })),
      if (n > 0)
        Padding(
          padding: const EdgeInsets.fromLTRB(10, 8, 10, 4),
          child: Wrap(spacing: 4, runSpacing: 4, children: [
            for (final f in _on) _ActiveChip(f.$2, onTap: () => setState(() => _on = [..._on]..remove(f))),
            if (_text.isNotEmpty) _ActiveChip('“$_text”', onTap: () => setState(() => _text = '')),
          ]),
        ),
      Expanded(
        child: r.isEmpty
            ? _Empty('Nothing matches $n filters', const [], actions: [_Btn('Clear all', onTap: () => setState(() {
                    _on = [];
                    _text = '';
                  }))])
            : _filtered(r, hidden: kItems.length - r.length, row: (it, i) => _ItemRow(it, tokens: _tokens(_text), showKind: !_on.any((f) => f.$1 == 'Kind'), showCat: !_on.any((f) => f.$1 == 'Category'))),
      ),
    ]);
  }
}

// ================================================================================ B2-b  Chips inside the field

const _qkeys = <String, List<String>>{
  'kind': ['effect', 'preset', 'media'],
  'cat': ['blur', 'color', 'glow', 'distort', 'stylize', 'time', 'generate', 'title', 'motion', 'ease'],
  'tag': ['soft', 'glitch', 'retro', 'bright', 'grade', 'moving', 'loop'],
  'in': ['effects', 'presets', 'project', 'mine'],
  'fav': ['yes'],
};

class _Q {
  const _Q(this.key, this.val);
  final String key, val;
  String get raw => '$key:$val';
  bool test(It i) => switch (key) {
        'kind' => i.kind.toLowerCase() == val,
        'cat' => i.cat.toLowerCase() == val,
        'tag' => i.tags.contains(val),
        'in' => i.place.toLowerCase() == val,
        _ => i.fav,
      };
}

_Q? _parseQ(String t) {
  final p = t.toLowerCase().split(':');
  if (p.length != 2 || !_qkeys.containsKey(p[0]) || !(_qkeys[p[0]]!.contains(p[1]))) return null;
  return _Q(p[0], p[1]);
}

class _ChipField extends StatefulWidget {
  const _ChipField({super.key, this.chips = 0, this.text = ''});
  final int chips;
  final String text;
  @override
  State<_ChipField> createState() => _ChipFieldState();
}

class _ChipFieldState extends State<_ChipField> {
  static const _seed = ['kind:effect', 'tag:soft', 'cat:glow', 'in:effects', 'tag:bright', 'fav:yes', 'tag:moving', 'tag:retro'];
  late final _c = TextEditingController(text: widget.text);
  late final _n = FocusNode(onKeyEvent: _key);
  late final List<_Q> _chips = [for (final s in _seed.take(widget.chips)) _parseQ(s)!];
  String _say = '';

  @override
  void initState() {
    super.initState();
    _n.addListener(() => setState(() {}));
    _c.addListener(_watch);
  }

  void _watch() {
    final t = _c.text;
    if (t.endsWith(' ') && t.trim().isNotEmpty) {
      final parts = t.trim().split(RegExp(r'\s+'));
      final q = _parseQ(parts.last);
      if (q != null) {
        _chips.add(q);
        final rest = parts.sublist(0, parts.length - 1).join(' ');
        _c.value = TextEditingValue(text: rest.isEmpty ? '' : '$rest ', selection: TextSelection.collapsed(offset: rest.isEmpty ? 0 : rest.length + 1));
        _say = 'Added ${q.raw}';
      }
    }
    setState(() {});
  }

  String get _tok => _c.text.split(' ').last;

  /// (label, insertion) pairs for the token being typed.
  List<(String, String)> _suggest2() {
    final t = _tok.toLowerCase();
    if (t.isEmpty) return const [];
    if (!t.contains(':')) return [for (final k in _qkeys.keys) if (k.startsWith(t) && k != t) ('$k:', '$k:')];
    final p = t.split(':');
    final vals = _qkeys[p[0]];
    if (vals == null || p.length > 2) return const [];
    return [for (final v in vals) if (v.startsWith(p[1])) ('${p[0]}:$v', '${p[0]}:$v ')];
  }

  void _insert(String s) {
    final words = _c.text.split(' ')..removeLast();
    final t = [...words, s].join(' ');
    _c.value = TextEditingValue(text: t, selection: TextSelection.collapsed(offset: t.length));
    _n.requestFocus();
  }

  KeyEventResult _key(FocusNode n, KeyEvent e) {
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.backspace && _c.text.isEmpty && _chips.isNotEmpty) {
      final q = _chips.removeLast();
      _c.value = TextEditingValue(text: q.raw, selection: TextSelection.collapsed(offset: q.raw.length));
      setState(() => _say = 'Editing ${q.raw}');
      return KeyEventResult.handled;
    }
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.enter) {
      final s = _suggest2();
      if (s.isNotEmpty) _insert(s.first.$2);
      return KeyEventResult.handled;
    }
    return _esc(e, _c, n, (s) => setState(() => _say = s));
  }

  List<It> get _res {
    final free = _c.text.split(' ').where((w) => w.isNotEmpty && _parseQ(w) == null).join(' ');
    return _match(kItems.where((i) => _chips.every((q) => q.test(i))), free);
  }

  @override
  void dispose() {
    _c.dispose();
    _n.dispose();
    super.dispose();
  }

  Widget _chip(_Q q) => _FilterChip(q.key, q.val, onRemove: () => setState(() => _chips.remove(q)));

  @override
  Widget build(BuildContext context) {
    final f = _n.hasFocus, collapse = !f && _chips.length > 3;
    final shownChips = collapse ? _chips.take(2).toList() : _chips;
    final r = _res, sg = _suggest2();
    final bad = _tok.contains(':') && _qkeys.containsKey(_tok.split(':').first.toLowerCase()) && sg.isEmpty && _tok.split(':').length == 2 && _tok.split(':')[1].isNotEmpty && !_tok.endsWith(' ');
    final tp = TextPainter(text: TextSpan(text: _c.text, style: T.name(N.g95)), textDirection: TextDirection.ltr)..layout();
    return Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      LayoutBuilder(builder: (context, box) {
        final tw = _c.text.isEmpty && _chips.isEmpty ? box.maxWidth - 28 : (tp.width + 16).clamp(90.0, box.maxWidth - 36);
        return _pad(
          GestureDetector(
            behavior: HitTestBehavior.opaque,
            onTap: _n.requestFocus,
            child: _FieldShell(
              focused: f,
              children: [
                for (final q in shownChips) _chip(q),
                if (collapse) _MoreChip(_chips.length - 2),
                SizedBox(
                  width: tw,
                  height: 20,
                  child: Stack(alignment: Alignment.centerLeft, children: [
                    if (_c.text.isEmpty && _chips.isEmpty) IgnorePointer(child: Text('Search, or kind: tag: cat:', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(N.g76))),
                    EditableText(controller: _c, focusNode: _n, style: T.name(N.g95), cursorColor: N.g95, backgroundCursorColor: N.g44, selectionColor: Role.selected.withValues(alpha: .35), cursorWidth: 1, maxLines: 1),
                  ]),
                ),
              ],
            ),
          ),
          t: 10,
        );
      }),
      if (_c.text.isEmpty && _chips.isEmpty)
        Padding(
          padding: const EdgeInsets.fromLTRB(10, 8, 10, 0),
          child: Wrap(spacing: 4, runSpacing: 4, crossAxisAlignment: WrapCrossAlignment.center, children: [
            for (final k in const ['kind:', 'cat:', 'tag:', 'in:', 'fav:yes']) _H(onTap: () => _insert(k), builder: (context, h) => _HintChip(k, hover: h)),
          ]),
        ),
      if (sg.isNotEmpty)
        Padding(
          padding: const EdgeInsets.fromLTRB(10, 4, 10, 0),
          child: _SugBox([
            for (var i = 0; i < math.min(sg.length, 5); i++) _H(onTap: () => _insert(sg[i].$2), builder: (context, h) => _SugRow(sg[i].$1, first: i == 0, hover: h)),
          ]),
        ),
      if (bad)
        Padding(
          padding: const EdgeInsets.fromLTRB(10, 6, 10, 0),
          child: _Notice('“${_tok.split(':')[1]}” is not a ${_tok.split(':')[0]}  ·  ${_qkeys[_tok.split(':')[0].toLowerCase()]!.take(3).join(', ')}', fill: N.g13),
        ),
      const SizedBox(height: 8),
      _Band(filters: _chips.length, shown: r.length, total: kItems.length, showFilters: false, extra: _chips.any((q) => q.key == 'kind') ? null : _mostly(r), onClear: () => setState(() {
            _chips.clear();
            _c.clear();
          })),
      Expanded(
        child: r.isEmpty
            ? _Empty('No match for these chips', const [], actions: [if (_chips.isNotEmpty) _Btn('Remove ${_chips.last.raw}', onTap: () => setState(() => _chips.removeLast())), _Btn('Clear', onTap: () => setState(() {
                      _chips.clear();
                      _c.clear();
                    }))])
            : _filtered(r, fade: true, hidden: kItems.length - r.length, row: (it, i) => _ItemRow(it, tokens: _tokens(_c.text), showKind: _chips.where((q) => q.key == 'kind').length != 1, showCat: _chips.where((q) => q.key == 'cat').length != 1)),
      ),
      _Status(_say, keys: const ['Space', 'Bksp']),
    ]);
  }
}

/// The chip in a field or a filter row: key g63, value g95 bold, a 16 px cross. 20 px tall.
class _FilterChip extends StatelessWidget {
  const _FilterChip(this.k, this.v, {this.onRemove, this.hoverCross});
  final String k, v;
  final VoidCallback? onRemove;
  final bool? hoverCross;
  @override
  Widget build(BuildContext context) => Container(
        height: 20,
        padding: const EdgeInsets.only(left: 6),
        decoration: BoxDecoration(color: N.g20, borderRadius: BorderRadius.circular(4)),
        child: Row(mainAxisSize: MainAxisSize.min, children: [
          Text('$k:', style: T.label(N.g76)),
          const SizedBox(width: 2),
          Flexible(child: Text(v, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g95).copyWith(fontWeight: FontWeight.w600))),
          _Cross(onTap: onRemove, hover: hoverCross),
        ]),
      );
}

/// What a crowded field shows instead of the chips it folded.
class _MoreChip extends StatelessWidget {
  const _MoreChip(this.n);
  final int n;
  @override
  Widget build(BuildContext context) => Container(height: 20, padding: const EdgeInsets.symmetric(horizontal: 6), decoration: BoxDecoration(color: N.g15, borderRadius: BorderRadius.circular(4)), child: Center(widthFactor: 1, child: Text('+$n more', style: T.label(N.g76))));
}

/// A key the empty field offers: an outlined chip (g13, 1 px g20), so it reads as "tap to try", not as a filter already on.
class _HintChip extends StatelessWidget {
  const _HintChip(this.text, {this.hover = false});
  final String text;
  final bool hover;
  @override
  Widget build(BuildContext context) => Container(height: 22, padding: const EdgeInsets.symmetric(horizontal: 8), decoration: BoxDecoration(color: hover ? N.g20 : N.g13, borderRadius: BorderRadius.circular(4), border: Border.all(color: N.g20)), child: Center(widthFactor: 1, child: Text(text, style: T.label(N.g76))));
}

/// One line of the completion list under the chip field: 24 px, mono; the first is the one Enter takes.
class _SugRow extends StatelessWidget {
  const _SugRow(this.text, {this.first = false, this.hover = false});
  final String text;
  final bool first, hover;
  @override
  Widget build(BuildContext context) => Container(
        height: 24,
        padding: const EdgeInsets.symmetric(horizontal: 8),
        alignment: Alignment.centerLeft,
        color: first ? N.g20 : (hover ? N.g15 : null),
        child: Row(children: [Expanded(child: Text(text, style: T.value(first ? N.g95 : N.g76))), if (first) const _Kbd('Enter')]),
      );
}

/// The completion list: a g13 box with a g26 edge around [_SugRow]s.
class _SugBox extends StatelessWidget {
  const _SugBox(this.children);
  final List<Widget> children;
  @override
  Widget build(BuildContext context) => Container(
        decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(4), border: Border.all(color: N.g26)),
        child: Flex(direction: Axis.vertical, mainAxisSize: MainAxisSize.min, children: children),
      );
}

/// The field that holds chips and the text being typed: g07, 1 px g20 (g63 when focused), 4 px inside.
class _FieldShell extends StatelessWidget {
  const _FieldShell({required this.focused, required this.children});
  final bool focused;
  final List<Widget> children;
  @override
  Widget build(BuildContext context) => AnimatedContainer(
        duration: Mo.dur,
        curve: Mo.ease,
        constraints: const BoxConstraints(minHeight: 30),
        padding: const EdgeInsets.all(4),
        decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(4), border: Border.all(color: focused ? N.g63 : N.g20)),
        child: Wrap(spacing: 4, runSpacing: 4, crossAxisAlignment: WrapCrossAlignment.center, children: children),
      );
}

// ================================================================================ B2-c  The result heading says the filter as a sentence

/// The result heading as a sentence: each word is a filter and drops itself when clicked; the tail says how many remain.
class _SentenceHead extends StatelessWidget {
  const _SentenceHead({required this.words, required this.count, required this.total, this.taps, this.hoverIndex, this.note});
  final List<String> words;
  final String? note;
  final List<VoidCallback>? taps;
  final int count, total;
  final int? hoverIndex;
  @override
  Widget build(BuildContext context) => Container(
        width: double.infinity,
        padding: const EdgeInsets.all(12),
        decoration: const BoxDecoration(color: N.g13, border: Border.symmetric(horizontal: BorderSide(color: N.rowLine))),
        child: words.isEmpty
            ? Text('All $total items', style: T.title())
            : Wrap(spacing: 8, runSpacing: 4, crossAxisAlignment: WrapCrossAlignment.center, children: [
                for (var i = 0; i < words.length; i++) ...[
                  if (i > 0) Text('·', style: T.title(N.g63)),
                  _Word(words[i], onTap: taps?[i], style: T.title(N.g95), rule: true, hover: hoverIndex == null ? null : hoverIndex == i),
                ],
                Text(count == 0 ? '— nothing' : '— $count of $total${note == null ? '' : ' · $note'}', style: T.name(N.g76)),
              ]),
      );
}

class _Sentence extends StatefulWidget {
  const _Sentence({super.key, this.kind = 0, this.chr = 0, this.fav = false, this.text = ''});
  final int kind, chr;
  final bool fav;
  final String text;
  @override
  State<_Sentence> createState() => _SentenceState();
}

class _SentenceState extends State<_Sentence> {
  static const _kinds = ['Any', 'Effect', 'Preset', 'Media'], _chars = ['Any', 'Soft', 'Glitch', 'Retro', 'Bright', 'Grade'];
  late int _k = widget.kind, _ch = widget.chr;
  late bool _fav = widget.fav;
  late final _c = TextEditingController(text: widget.text);
  late final _n = FocusNode(onKeyEvent: (n, e) => _esc(e, _c, n, (_) {}));

  List<It> _by({bool k = true, bool ch = true, bool fav = true, bool tx = true}) => _match(kItems.where((i) => (!k || _k == 0 || i.kind == _kinds[_k]) && (!ch || _ch == 0 || i.tags.contains(_chars[_ch].toLowerCase())) && (!fav || !_fav || i.fav)), tx ? _c.text : '');

  @override
  void dispose() {
    _c.dispose();
    _n.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: _c,
        builder: (context, _) {
          final r = _by(), q = _c.text.trim();
          final words = <(String, VoidCallback)>[
            if (_ch > 0) (_chars[_ch], () => setState(() => _ch = 0)),
            if (_k > 0) (_kinds[_k] == 'Media' ? 'Media' : '${_kinds[_k]}s', () => setState(() => _k = 0)),
            if (_fav) ('Favorites', () => setState(() => _fav = false)),
            if (q.isNotEmpty) ('“$q”', _c.clear),
          ];
          final fixes = <(String, int, VoidCallback)>[];
          if (r.isEmpty) {
            if (_ch > 0) fixes.add((_chars[_ch], _by(ch: false).length, () => setState(() => _ch = 0)));
            if (_k > 0) fixes.add((_kinds[_k] == 'Media' ? 'Media' : '${_kinds[_k]}s', _by(k: false).length, () => setState(() => _k = 0)));
            if (_fav) fixes.add(('Favorites', _by(fav: false).length, () => setState(() => _fav = false)));
            if (q.isNotEmpty) fixes.add(('“$q”', _by(tx: false).length, _c.clear));
            fixes.sort((a, b) => b.$2.compareTo(a.$2));
          }
          return Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            _pad(_Search(controller: _c, node: _n, onChanged: (_) => setState(() {})), t: 10, b: 8),
            _pad(_Tabs(_kinds, _k, (i) => setState(() => _k = i < 0 ? 0 : i), toggle: false, chevron: true), b: 6),
            _pad(_Tabs(_chars, _ch, (i) => setState(() => _ch = i < 0 ? 0 : i), toggle: false, chevron: true), b: 8),
            _pad(Row(children: [PillSwitch(on: _fav, onChanged: (v) => setState(() => _fav = v)), const SizedBox(width: 8), Text('Favorites only', style: T.label(N.g76))]), b: 8),
            _SentenceHead(words: [for (final w in words) w.$1], taps: [for (final w in words) w.$2], count: r.length, total: kItems.length, note: _k == 0 ? _mostly(r) : null),
            Expanded(
              child: r.isEmpty
                  ? _Empty('Nothing fits', const [], actions: [for (final f in fixes.take(3)) _Btn('Drop ${f.$1} → ${f.$2}', onTap: f.$3)])
                  : _filtered(r, fade: true, hidden: kItems.length - r.length, row: (it, i) => _ItemRow(it, tokens: _tokens(_c.text), showKind: _k == 0)),
            ),
          ]);
        },
      );
}

// ================================================================================ B2-d  A saved filter becomes a place

/// A place or a saved filter, 26 px: name and live count; on hover a saved filter swaps the count for its Delete word.
class _PlaceRow extends StatelessWidget {
  const _PlaceRow(this.label, this.count, {this.on = false, this.hover = false, this.right});
  final String label;
  final int count;
  final bool on, hover;
  final Widget? right;
  @override
  Widget build(BuildContext context) => Container(
        height: 26,
        color: on ? N.g20 : (hover ? N.g15 : null),
        child: Stack(fit: StackFit.expand, children: [
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 10),
            child: Row(children: [
              Expanded(child: Text(label, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(on ? N.g95 : N.g91))),
              const SizedBox(width: 8),
              if (right != null && hover) right! else Text('$count', style: _num(N.g76)),
            ]),
          ),
          _GutterTick(on, height: 12),
        ]),
      );
}

class _SavedF {
  _SavedF(this.name, this.spec);
  final String name;
  final Set<String> spec; // 'cat:Blur', 'tag:soft', 'kind:Effect'
}

bool _specPass(It i, Set<String> s) => s.every((x) {
      final p = x.split(':');
      return switch (p[0]) { 'cat' => i.cat == p[1], 'tag' => i.tags.contains(p[1]), _ => i.kind == p[1] };
    });

class _Saved extends StatefulWidget {
  const _Saved({super.key, this.extra = 0});
  final int extra;
  @override
  State<_Saved> createState() => _SavedState();
}

class _SavedState extends State<_Saved> {
  final List<_SavedF> _saved = [
    _SavedF('Soft blurs', {'cat:Blur', 'tag:soft'}),
    _SavedF('Retro looks', {'tag:retro'}),
    _SavedF('Title presets', {'cat:Title'}),
    _SavedF('Bright & loop', {'tag:bright', 'tag:loop'}),
  ];
  final List<It> _extra = [];
  Set<String> _spec = {};
  _SavedF? _active;
  int _place = -1;
  bool _naming = false, _more = false;
  final _name = TextEditingController();
  late final _nn = FocusNode(onKeyEvent: (n, e) {
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape) {
      setState(() => _naming = false);
      return KeyEventResult.handled;
    }
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.enter) {
      _commit();
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  });
  String _say = '“Soft blurs” = cat:Blur + tag:soft';

  @override
  void initState() {
    super.initState();
    _active = _saved.first;
    _spec = {..._saved.first.spec};
    for (var i = 0; i < widget.extra; i++) {
      _saved.add(_SavedF(const ['Grade only', 'Glow family', 'Moving type', 'Glitch + retro', 'Ease curves', 'Everything soft', 'Used 10+ times', 'Client A look', 'Loops', 'Bokeh set', 'Time effects', 'Distortions'][i % 12], {i.isEven ? 'tag:soft' : 'cat:Glow'}));
    }
  }

  List<It> get _pool => [...kItems, ..._extra];
  List<It> get _res => _pool.where((i) => (_place < 0 || i.place == _places[_place]) && _specPass(i, _spec)).toList();

  void _commit() {
    final n = _name.text.trim();
    if (n.isEmpty || _spec.isEmpty) return;
    final s = _SavedF(n, {..._spec});
    setState(() {
      _saved.add(s);
      _active = s;
      _naming = false;
      _say = 'Saved “$n”';
    });
  }

  @override
  void dispose() {
    _name.dispose();
    _nn.dispose();
    super.dispose();
  }

  Widget _chipRow(String label, String prefix, List<String> vals) {
    String key(String v) => '$prefix:${prefix == 'tag' ? v.toLowerCase() : v}';
    return _TagGroup(
      label: label,
      values: vals,
      on: {for (final v in vals) if (_spec.contains(key(v))) v},
      onTap: (v) => setState(() {
        final k = key(v);
        _spec.contains(k) ? _spec.remove(k) : _spec.add(k);
        _active = null;
      }),
    );
  }

  @override
  Widget build(BuildContext context) {
    final r = _res;
    final shownSaved = !_more && _saved.length > 4 ? _saved.take(3).toList() : _saved;
    Widget place(String label, int count, bool on, VoidCallback tap, {Widget? right}) => _H(onTap: tap, builder: (context, h) => _PlaceRow(label, count, on: on, hover: h, right: right));
    return Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      const _Sec('Places'),
      for (var i = 0; i < _places.length; i++) place(_places[i], _pool.where((x) => x.place == _places[i]).length, _place == i && _active == null && _spec.isEmpty, () => setState(() {
            _place = _place == i ? -1 : i;
            _spec = {};
            _active = null;
          })),
      _Sec('Saved filters', right: Text('${_saved.length}', style: T.label(N.g76))),
      SizedBox(
        height: math.min(shownSaved.length * 26.0, 130),
        child: ListView(padding: EdgeInsets.zero, children: [
      for (final s in shownSaved)
        place(s.name, _pool.where((x) => _specPass(x, s.spec)).length, identical(_active, s), () => setState(() {
              _active = s;
              _spec = {...s.spec};
              _place = -1;
              _say = '“${s.name}” = ${s.spec.join(' + ')}';
            }), right: _Word('Delete', onTap: () => setState(() {
              _saved.remove(s);
              if (identical(_active, s)) _active = null;
              _say = 'Deleted “${s.name}”';
            }))),
        ]),
      ),
      if (!_more && _saved.length > 4) Padding(padding: const EdgeInsets.fromLTRB(10, 4, 10, 0), child: Row(children: [Flexible(child: _Word('Show ${_saved.length - 3} more saved filters', onTap: () => setState(() => _more = true)))])),
      if (_more && _saved.length > 4) Padding(padding: const EdgeInsets.fromLTRB(10, 4, 10, 0), child: Row(children: [Flexible(child: _Word('Fold to 3', onTap: () => setState(() => _more = false)))])),
      const _Sec('Filter'),
      _chipRow('Category', 'cat', const ['Blur', 'Glow', 'Color', 'Title']),
      _chipRow('Character', 'tag', const ['Soft', 'Retro', 'Bright', 'Loop']),
      Padding(
        padding: const EdgeInsets.fromLTRB(10, 2, 10, 6),
        child: _naming
            ? SizedBox(height: 28, child: _Search(controller: _name, node: _nn, autofocus: true, slash: false, hint: 'Filter name', trailing: const _Kbd('Esc')))
            : Row(children: [
                Expanded(
                  child: Wrap(spacing: 6, runSpacing: 4, children: [
                    _Btn('Save as place…', onTap: _spec.isEmpty ? null : () => setState(() {
                          _naming = true;
                          _name.clear();
                        })),
                    _Btn('Simulate import', onTap: () => setState(() {
                          _extra.add(It('Soft Rain Blur ${_extra.length + 1}', 'やわらか雨ぼかし', 'Effects', 'Blur', Fx.blur, const ['soft']));
                          _say = 'Added “Soft Rain Blur ${_extra.length}”';
                        })),
                  ]),
                ),
              ]),
      ),
      Container(height: 1, color: N.rowLine),
      Expanded(
        child: r.isEmpty
            ? _Empty('Nothing in this filter yet', const [], actions: [_Btn('Show everything', onTap: () => setState(() {
                    _spec = {};
                    _active = null;
                    _place = -1;
                  }))])
            : _filtered(r, quiet: false, fade: true, hidden: _pool.length - r.length, row: (it, i) => _ItemRow(it, showKind: !_spec.any((x) => x.startsWith('kind:')), showCat: _spec.where((x) => x.startsWith('cat:')).length != 1)),
      ),
      _Status(_say),
    ]);
  }
}

// ================================================================================ B5/B6  preview before applying

Widget _ownBase({bool sample = false}) => sample
    ? Stack(fit: StackFit.expand, children: [const Art(Asset('', Kind.image, 9, style: ArtStyle.gradient)), Center(child: Text('SAMPLE', style: T.micro(N.g76).copyWith(letterSpacing: 8)))])
    : Stack(fit: StackFit.expand, children: [
        const Art(Asset('', Kind.image, 3)),
        Center(child: Flex(direction: Axis.vertical, mainAxisSize: MainAxisSize.min, children: [
          Text('光の速さで', style: T.title().copyWith(fontSize: 22, height: 1.1)),
          const SizedBox(height: 6),
          Text('LIGHTSPEED', style: T.micro(N.g95).copyWith(letterSpacing: 4)),
        ])),
      ]);

const _scrim = Color(0x99000000);

/// A label laid over a picture: 16 px tall, 6 px sides, one line, ellipsis when the picture is narrow.
Widget _pill(String s, {Color fill = _scrim, Color ink = N.g95}) => Container(
      height: 16,
      padding: const EdgeInsets.symmetric(horizontal: 6),
      decoration: BoxDecoration(color: fill, borderRadius: BorderRadius.circular(3)),
      child: Center(widthFactor: 1, child: Text(s, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.micro(ink).copyWith(letterSpacing: .4))),
    );

/// Two labels on one edge of a picture, 6 px in; the right one gives way (ellipsis) before it touches the left one.
Widget _edge(Widget left, Widget? right, {bool top = true}) => Positioned(left: 6, right: 6, top: top ? 6 : null, bottom: top ? null : 6, child: Row(children: [Flexible(flex: 3, child: Align(alignment: Alignment.centerLeft, child: left)), const SizedBox(width: 6), Expanded(flex: 2, child: Align(alignment: Alignment.centerRight, child: right ?? const SizedBox()))]));

// ---- B5-a  every tile is a small screening

class _MovingTiles extends StatefulWidget {
  const _MovingTiles({super.key, required this.count, required this.reduce, required this.w});
  final int count;
  final bool reduce;
  final double w;
  @override
  State<_MovingTiles> createState() => _MovingTilesState();
}

class _MovingTilesState extends State<_MovingTiles> {
  final _sc = ScrollController();
  static const _gap = 8.0;
  List<It> get _pool => kItems.where((i) => i.asset == null).toList();

  It _at(int i) {
    final p = _pool, b = p[(i * 5) % p.length];
    return i < p.length ? b : It('${b.name} v${i ~/ p.length + 1}', b.jp, b.place, b.cat, b.fx, b.tags);
  }

  @override
  void dispose() {
    _sc.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final tw = (widget.w - 20 - _gap) / 2, th = tw * .62 + 42, rowH = th + _gap;
    return Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      AnimatedBuilder(
        animation: _sc,
        builder: (context, _) {
          final vp = _sc.hasClients ? _sc.position.viewportDimension : 500.0, off = _sc.hasClients ? _sc.offset : 0.0;
          final first = (off / rowH).floor(), last = ((off + vp) / rowH).floor();
          final vis = math.min(widget.count, (last - first + 1) * 2);
          // Right of the title, in words: how many tiles move now, or "held" (reduce motion). No mark on the tiles.
          return _pad(
            Row(children: [
              Expanded(child: Text('Effects and presets', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.title())),
              Text(widget.reduce ? 'held' : '$vis playing', style: widget.reduce ? T.label(N.g63) : T.label(N.g76)),
            ]),
            t: 12,
            b: 8,
          );
        },
      ),
      Expanded(
        child: _Loop(
          running: !widget.reduce,
          rest: .55,
          builder: (context, t) => _FadeBottom(child: GridView.builder(
            controller: _sc,
            padding: const EdgeInsets.fromLTRB(10, 0, 10, 10),
            gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(crossAxisCount: 2, mainAxisSpacing: _gap, crossAxisSpacing: _gap, mainAxisExtent: th),
            itemCount: widget.count,
            itemBuilder: (context, i) {
              final it = _at(i);
              return _H(builder: (context, h) => _TileCard(it, t: t, w: tw, hover: h));
            },
          )),
        ),
      ),
    ]);
  }
}

/// One screening tile: the moving picture (g20 edge, g56 on hover), the name, then category and kind.
class _TileCard extends StatelessWidget {
  const _TileCard(this.it, {required this.t, required this.w, this.hover = false});
  final It it;
  final double t, w;
  final bool hover;
  @override
  Widget build(BuildContext context) => SizedBox(
        width: w,
        child: Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.start, children: [
          Container(
            height: w * .62,
            decoration: BoxDecoration(borderRadius: BorderRadius.circular(6), border: Border.all(color: hover ? N.g56 : N.g20)),
            child: ClipRRect(borderRadius: BorderRadius.circular(5), child: Stack(fit: StackFit.expand, children: [_Fx(it.fx, t, seed: it.seed)])),
          ),
          const SizedBox(height: 6),
          Text(it.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name()),
          const SizedBox(height: 4),
          Text('${it.cat} · ${it.kind}', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g76)),
        ]),
      );
}

// ---- B5-b / B6  hover to play, x to scrub

/// The one picture area of a tile: still at rest, a loop while hovered; timed items scrub by pointer x.
class _HoverThumb extends StatefulWidget {
  const _HoverThumb(this.it, {required this.w, required this.h, this.onInfo, this.selected = false, this.holdHover = false, this.holdX});
  final It it;
  final double w, h;
  final bool selected;

  /// Parts sheets: start with the pointer already on the tile (and at x = holdX), loop held at its middle.
  final bool holdHover;
  final double? holdX;
  final ValueChanged<String?>? onInfo;
  @override
  State<_HoverThumb> createState() => _HoverThumbState();
}

class _HoverThumbState extends State<_HoverThumb> with SingleTickerProviderStateMixin {
  late final AnimationController _c = AnimationController(vsync: this, duration: const Duration(milliseconds: 1200));
  late bool _hover = widget.holdHover;
  late double? _x = widget.holdX;

  @override
  void initState() {
    super.initState();
    if (_hover && !_scrubbed) _c.value = .5;
  }

  double get _dur => widget.it.asset != null ? widget.it.asset!.dur : (widget.it.long ? 2.4 : 0);
  bool get _scrubbed => _dur > 0;

  @override
  void dispose() {
    _c.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final it = widget.it;
    final scrub = _scrubbed && _hover && _x != null ? (_x! / widget.w).clamp(0.0, 1.0) : null;
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) {
        setState(() => _hover = true);
        if (!_scrubbed) _c.repeat();
      },
      onHover: (e) {
        setState(() => _x = e.localPosition.dx);
        if (_scrubbed) widget.onInfo?.call('${it.name} at ${_clock2((e.localPosition.dx / widget.w).clamp(0.0, 1.0) * _dur)} of ${_clock2(_dur)}');
      },
      onExit: (_) {
        setState(() {
          _hover = false;
          _x = null;
        });
        _c.stop();
        _c.value = 0;
        widget.onInfo?.call(null);
      },
      child: Container(
        width: widget.w,
        height: widget.h,
        decoration: BoxDecoration(borderRadius: BorderRadius.circular(6), border: Border.all(color: widget.selected ? Role.selected : (_hover ? N.g56 : N.g20))),
        child: ClipRRect(
          borderRadius: BorderRadius.circular(5),
          child: Stack(fit: StackFit.expand, children: [
            if (it.asset != null)
              Art(scrub == null ? it.asset! : Asset(it.asset!.name, it.asset!.kind, it.asset!.seed + 1 + (scrub * 6).floor(), style: it.asset!.style, dur: it.asset!.dur, aspect: it.asset!.aspect))
            else
              AnimatedBuilder(animation: _c, builder: (context, _) => _Fx(it.fx, scrub ?? (_hover ? _c.value : .6), seed: it.seed)),
            if (it.asset != null && it.asset!.kind != Kind.image) Positioned(left: 4, top: 4, child: TypeBadge(it.asset!.kind)),
            if (_hover && !_scrubbed) Positioned(left: 4, top: 4, child: _pill('Playing')),
            if (_scrubbed) Positioned(right: 4, bottom: 4, child: TimePill(scrub == null ? _clock2(_dur) : _clock2(scrub * _dur))),
            // A 2 px rail along the foot of a scrubbable picture: it fills to the pointer, so the picture reads as a strip of time before anyone touches it.
            if (_scrubbed) Positioned(left: 0, right: 0, bottom: 0, height: 2, child: Stack(children: [const ColoredBox(color: Color(0x66000000)), Align(alignment: Alignment.centerLeft, child: FractionallySizedBox(widthFactor: scrub ?? 0, child: const ColoredBox(color: N.g95)))])),
            if (scrub != null) Positioned(left: (scrub * widget.w).clamp(0.0, widget.w - 1), top: 0, bottom: 0, width: 1, child: const ColoredBox(color: N.g100)),
            if (widget.selected) Positioned(right: 0, top: 0, child: CustomPaint(size: const Size(8, 8), painter: _CornerPaint(Role.selected))),
          ]),
        ),
      ),
    );
  }
}

/// An 8 px triangle in the top right corner: the shape that says "selected" beside the accent edge.
class _CornerPaint extends CustomPainter {
  const _CornerPaint(this.color, {this.bottomLeft = false});
  final Color color;
  final bool bottomLeft;
  @override
  void paint(Canvas c, Size s) => c.drawPath(bottomLeft ? (Path()..moveTo(0, 0)..lineTo(0, s.height)..lineTo(s.width, s.height)..close()) : (Path()..moveTo(0, 0)..lineTo(s.width, 0)..lineTo(s.width, s.height)..close()), Paint()..color = color);
  @override
  bool shouldRepaint(_CornerPaint o) => o.color != color || o.bottomLeft != bottomLeft;
}

String _clock2(double s) => s < 60 ? '${s.toStringAsFixed(1)}s' : clock(s);

/// A tile of the hover grid: the thumbnail (hover plays, x scrubs), the name (selects), then kind or category and length.
class _HoverTile extends StatelessWidget {
  const _HoverTile(this.it, {required this.w, this.selected = false, this.onInfo, this.onSelect});
  final It it;
  final double w;
  final bool selected;
  final ValueChanged<String?>? onInfo;
  final VoidCallback? onSelect;
  @override
  Widget build(BuildContext context) => SizedBox(
        width: w,
        child: Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.start, children: [
          _HoverThumb(it, w: w, h: w * .62, selected: selected, onInfo: onInfo),
          const SizedBox(height: 6),
          _H(onTap: onSelect, builder: (context, h) => Text(it.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(selected ? N.g95 : N.g91).copyWith(decoration: selected ? TextDecoration.underline : TextDecoration.none, decorationColor: Role.selected))),
          const SizedBox(height: 4),
          Text(it.asset != null ? (it.asset!.kind == Kind.image ? 'Image · ${it.asset!.dims}' : it.asset!.dims) : '${it.cat} · ${it.kind}', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g76)),
        ]),
      );
}

/// A row of the hover list, 50 px: the thumbnail is the hover target (60 or 76 wide), the name selects.
class _HoverRow extends StatelessWidget {
  const _HoverRow(this.it, {required this.tw, this.selected = false, this.onInfo, this.onSelect, this.holdHover = false, this.holdX});
  final It it;
  final double tw;
  final bool selected, holdHover;
  final double? holdX;
  final ValueChanged<String?>? onInfo;
  final VoidCallback? onSelect;
  @override
  Widget build(BuildContext context) {
    final meta = it.asset != null ? it.asset!.dims : (it.long ? '2.4s' : 'loop');
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
      child: Row(children: [
        _HoverThumb(it, w: tw, h: 42, selected: selected, onInfo: onInfo, holdHover: holdHover, holdX: holdX),
        const SizedBox(width: 8),
        Expanded(
          child: _H(
            onTap: onSelect,
            builder: (context, h) => Flex(direction: Axis.vertical, mainAxisAlignment: MainAxisAlignment.center, crossAxisAlignment: CrossAxisAlignment.start, children: [
              Text(it.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(selected ? N.g95 : N.g91).copyWith(decoration: selected ? TextDecoration.underline : TextDecoration.none, decorationColor: Role.selected)),
              const SizedBox(height: 4),
              Text(it.asset != null && it.asset!.kind != Kind.image ? meta : (it.asset != null ? 'Image · $meta' : '${it.cat} · $meta'), maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g76)),
            ]),
          ),
        ),
      ]),
    );
  }
}

class _HoverTiles extends StatefulWidget {
  const _HoverTiles({super.key, required this.w, this.rows = false});
  final double w;
  final bool rows;
  @override
  State<_HoverTiles> createState() => _HoverTilesState();
}

class _HoverTilesState extends State<_HoverTiles> {
  String? _info;
  int _sel = -1;

  List<It> get _items {
    It n(String s) => kItems.firstWhere((i) => i.name == s);
    return [n('Glow'), n('Gaussian Blur'), n('lens_bloom.mp4'), n('Glitch RGB Split'), n('Typewriter'), n('city_pass.mov'), n('Pop In Overshoot'), n('sunrise.jpg'), n('sphere_tour.mp4'), n('Wave Warp'), n('wind_pad.wav'), n('Film Grain'), n('Lower Third Slide'), n('dusk_ridge.jpg')];
  }

  @override
  Widget build(BuildContext context) {
    final its = _items;
    final String idle = _sel >= 0 ? its[_sel].name : '${its.length} items';
    return Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      _pad(_Tabs(_places, 0, (_) {}, toggle: false), t: 10, b: 8),
      Expanded(
        child: widget.rows
            ? _Fit(natural: its.length * 50.0, child: _FadeBottom(child: ListView.builder(
                padding: EdgeInsets.zero,
                itemExtent: 50,
                itemCount: its.length,
                itemBuilder: (context, i) {
                  final tw = widget.w < 300 ? 60.0 : 76.0;
                  return _HoverRow(its[i], tw: tw, selected: _sel == i, onInfo: (s) => setState(() => _info = s), onSelect: () => setState(() => _sel = i));
                })))
            : LayoutBuilder(builder: (context, box) {
                final tw = (widget.w - 20 - 8) / 2;
                return _FadeBottom(child: SingleChildScrollView(
                  padding: const EdgeInsets.fromLTRB(10, 0, 10, 10),
                  child: Wrap(spacing: 8, runSpacing: 10, children: [
                    for (var i = 0; i < its.length; i++)
                      _HoverTile(its[i], w: tw, selected: _sel == i, onInfo: (s) => setState(() => _info = s), onSelect: () => setState(() => _sel = i)),
                  ]),
                ));
              }),
      ),
      _Status(_info ?? idle),
    ]);
  }

}

/// B6: long clips, one big tile each; x maps to time.
class _ScrubClips extends StatefulWidget {
  const _ScrubClips({super.key, required this.w});
  final double w;
  @override
  State<_ScrubClips> createState() => _ScrubClipsState();
}

class _ScrubClipsState extends State<_ScrubClips> {
  String? _info;
  @override
  Widget build(BuildContext context) {
    final names = ['lens_bloom.mp4', 'city_pass.mov', 'sphere_tour.mp4', 'Lower Third Slide', 'kick_loop.wav'];
    final tw = widget.w - 20;
    return Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      _pad(Text('Clips', style: T.title()), t: 12, b: 2),
      const SizedBox(height: 8),
      Expanded(
        child: ListView(padding: const EdgeInsets.fromLTRB(10, 0, 10, 10), children: [
          for (final n in names) ...[
            _HoverThumb(kItems.firstWhere((i) => i.name == n), w: tw, h: tw * .5, onInfo: (s) => setState(() => _info = s)),
            const SizedBox(height: 6),
            Row(children: [
              Expanded(child: Text(n, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name())),
              Text(() {
                final a = kItems.firstWhere((i) => i.name == n);
                return a.asset != null ? a.asset!.dims : '2.4s · preset';
              }(), style: _num(N.g76)),
            ]),
            const SizedBox(height: 12),
          ],
        ]),
      ),
      _Status(_info ?? '${names.length} clips'),
    ]);
  }
}

// ---- B5-c  preview on your own frame

/// Your own frame in the Browser: a 1 px edge (g20, the mode colour while previewing), the picture, the layer label on the left and the state label on the right.
class _OwnFrameView extends StatelessWidget {
  const _OwnFrameView({required this.w, required this.h, required this.label, required this.previewing, required this.picture, this.applied, this.noTarget = false});
  final double w, h;
  final String label;
  final bool previewing, noTarget;
  final String? applied;
  final Widget picture;
  @override
  Widget build(BuildContext context) => SizedBox(
        width: w,
        height: h,
        child: CustomPaint(
          foregroundPainter: previewing ? DashedBox(Role.selected, radius: 0) : null,
          child: DecoratedBox(
          decoration: BoxDecoration(border: Border.all(color: previewing ? _clear : N.g20)),
          child: ClipRect(
            child: Stack(fit: StackFit.expand, children: [
              picture,
              _edge(_pill(label), previewing ? _pill('PREVIEW', fill: N.g20) : (applied != null ? _pill('APPLIED · $applied', fill: N.g26) : null)),
              if (previewing) const Positioned(right: 6, bottom: 6, child: Row(mainAxisSize: MainAxisSize.min, children: [_Kbd('Enter'), SizedBox(width: 4), _Kbd('Esc')])),
            ]),
          ),
        ),
        ),
      );
}

/// Tried and Written under the frame: the written count brightens (g95) once it is not zero; right = the Down key mark to start (the Enter and Esc marks sit on the preview frame), nothing when hover starts it.
class _Counters extends StatelessWidget {
  const _Counters(this.tried, this.written, this.hint);
  final int tried, written;
  final String? hint;
  @override
  Widget build(BuildContext context) => Row(children: [
        Text('Tried ', style: T.label(N.g76)),
        Text('$tried', style: T.value(N.g95)),
        const SizedBox(width: 12),
        Text('Written ', style: T.label(N.g76)),
        Text('$written', style: T.value(written > 0 ? N.g95 : N.g76)),
        const Spacer(),
        if (hint != null && hint!.isNotEmpty) _Kbd(hint!),
      ]);
}

class _OwnFrame extends StatefulWidget {
  const _OwnFrame({super.key, required this.w, required this.hover, required this.noTarget});
  final double w;
  final bool hover, noTarget;
  @override
  State<_OwnFrame> createState() => _OwnFrameState();
}

class _OwnFrameState extends State<_OwnFrame> {
  late final _root = FocusNode(onKeyEvent: _key);
  final _sc = ScrollController();
  List<It> get _pool => kItems.where((i) => i.place != 'Project' && i.applies != 'key').toList();
  int _sel = 0, _tries = 0, _writes = 0;
  It? _pv;
  final List<It> _applied = [];
  String _say = 'Layer 3 · Title';

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) _peek(_pool.indexWhere((i) => i.name == 'Glow'));
    });
  }

  void _peek(int i) {
    final c = _pool.length;
    setState(() {
      _sel = i.clamp(0, c - 1);
      _pv = _pool[_sel];
      _tries++;
      _say = '“${_pv!.name}”';
    });
    _revealAt(_sc, _sel * 34.0, _sel * 34.0 + 38);
  }

  void _commit() {
    if (_pv == null) return;
    if (widget.noTarget) {
      setState(() => _say = 'No target selected');
      return;
    }
    setState(() {
      _applied.add(_pv!);
      _writes++;
      _say = 'Applied “${_pv!.name}”';
      _pv = null;
    });
  }

  KeyEventResult _key(FocusNode n, KeyEvent e) {
    if (_isDown(e, LogicalKeyboardKey.arrowDown)) {
      _peek(_pv == null && _tries == 0 ? 0 : _sel + 1);
      return KeyEventResult.handled;
    }
    if (_isDown(e, LogicalKeyboardKey.arrowUp)) {
      _peek(_sel - 1);
      return KeyEventResult.handled;
    }
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.enter) {
      _commit();
      return KeyEventResult.handled;
    }
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape) {
      setState(() {
        _pv = null;
        _say = 'Dropped';
      });
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  @override
  void dispose() {
    _root.dispose();
    _sc.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final fw = widget.w - 20, fh = fw * 9 / 16, pool = _pool;
    final label = widget.noTarget ? 'Sample frame (no selection)' : 'Layer 3 · Title';
    return Focus(
      focusNode: _root,
      autofocus: true,
      onFocusChange: (f) {
        if (!f && !widget.hover && mounted) setState(() => _pv = null);
      },
      child: MouseRegion(
        onExit: (_) {
          if (widget.hover) setState(() => _pv = null);
        },
        child: GestureDetector(
          behavior: HitTestBehavior.translucent,
          onTap: _root.requestFocus,
          child: Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            _pad(
              _OwnFrameView(
                w: fw,
                h: fh,
                label: label,
                previewing: _pv != null,
                applied: _applied.isNotEmpty ? _applied.last.name : null,
                noTarget: widget.noTarget,
                picture: _Loop(running: _pv != null, builder: (context, t) => _Fx(_pv?.fx ?? (_applied.isEmpty ? Fx.none : _applied.last.fx), _pv != null ? t : .6, base: _ownBase(sample: widget.noTarget))),
              ),
              t: 10,
            ),
            _pad(
              _Counters(_tries, _writes, _pv != null ? null : (widget.hover ? '' : 'Down')),
              t: 8,
              b: 8,
            ),
            Container(height: 1, color: N.rowLine),
            Expanded(
              child: _Fit(natural: pool.length * 34.0 + 4, child: _QuietKind(_majority(pool), child: _FadeBottom(top: true, child: ListView.builder(
                controller: _sc,
                padding: const EdgeInsets.only(top: 4),
                itemExtent: 34,
                itemCount: pool.length,
                itemBuilder: (context, i) => MouseRegion(
                  onEnter: (_) {
                    if (widget.hover) _peek(i);
                  },
                  child: _ItemRow(pool[i], selected: i == _sel && (_pv != null || _tries > 0), onTap: () {
                    _peek(i);
                    _root.requestFocus();
                  }, onUse: () {
                    _peek(i);
                    _commit();
                  }),
                ),
              )))),
            ),
            _Status(_say),
          ]),
        ),
      ),
    );
  }
}

enum _Phase { preview, applied, dropped }

/// The band under a frame that says which state it is in: a word and a shape. PREVIEW = dashed g56 edge + the Esc key mark, APPLIED = g20 ground, the tick + the Cmd Z key mark, DROPPED = g63 word struck through.
class _PhaseBanner extends StatelessWidget {
  const _PhaseBanner(this.phase);
  final _Phase phase;
  @override
  Widget build(BuildContext context) {
    final (head, tail) = switch (phase) {
      _Phase.preview => ('PREVIEW', '0 writes'),
      _Phase.applied => ('APPLIED', '1 write'),
      _Phase.dropped => ('DROPPED', '0 writes'),
    };
    final dim = phase == _Phase.dropped;
    return Container(
      height: 28,
      decoration: BoxDecoration(color: phase == _Phase.applied ? N.g20 : N.g10),
      child: CustomPaint(
        foregroundPainter: phase == _Phase.preview ? const DashedBox(N.g56, radius: 0) : null,
        child: Stack(fit: StackFit.expand, children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(10, 0, 8, 0),
            child: Row(children: [
              Text(head, style: T.micro(dim ? N.g63 : N.g95).copyWith(letterSpacing: .6, decoration: dim ? TextDecoration.lineThrough : TextDecoration.none, decorationColor: N.g63)),
              const SizedBox(width: 8),
              Expanded(child: Text(tail, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g76))),
              if (phase == _Phase.preview) const _Kbd('Esc'),
              if (phase == _Phase.applied) const _Kbd('Cmd Z'),
            ]),
          ),
          _GutterTick(phase == _Phase.applied, height: 12),
        ]),
      ),
    );
  }
}

/// B5-c, the three states side by side: previewing, applied, dropped. Every state is a word and a grey step.
class _Phases extends StatelessWidget {
  const _Phases({required this.w, required this.fx});
  final double w;
  final It fx;
  @override
  Widget build(BuildContext context) {
    final fw = w - 20, fh = fw * 9 / 16 * (w < 300 ? .9 : .62);
    Widget frame(Widget child, Widget banner) => Padding(
          padding: const EdgeInsets.fromLTRB(10, 0, 10, 12),
          child: Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.stretch, children: [SizedBox(width: fw, height: fh, child: ClipRect(child: child)), banner]),
        );
    return ListView(padding: const EdgeInsets.only(top: 12), children: [
      frame(_Loop(builder: (context, t) => _Fx(fx.fx, t, base: _ownBase())), const _PhaseBanner(_Phase.preview)),
      frame(_Fx(fx.fx, .6, base: _ownBase()), const _PhaseBanner(_Phase.applied)),
      frame(_ownBase(), const _PhaseBanner(_Phase.dropped)),
    ]);
  }
}

// ---- B5-d  a fixed big preview surface

/// The fixed preview surface: a 1 px g20 edge, the picture, whose frame it is + Hide on top, Playing or Paused + the item's name below.
class _SurfaceFrame extends StatelessWidget {
  const _SurfaceFrame({required this.w, required this.h, required this.picture, required this.sample, required this.playing, required this.name, this.onHide, this.hoverHide});
  final double w, h;
  final Widget picture;
  final bool sample, playing;
  final String name;
  final VoidCallback? onHide;
  final bool? hoverHide;
  @override
  Widget build(BuildContext context) => SizedBox(
        width: w,
        height: h,
        child: DecoratedBox(
          decoration: BoxDecoration(border: Border.all(color: N.g20)),
          child: ClipRect(
            child: Stack(fit: StackFit.expand, children: [
              picture,
              _edge(_pill(sample ? 'Sample frame' : 'Your frame · Layer 3'), _H(onTap: onHide, force: hoverHide, builder: (context, h) => _pill('Hide', fill: h ? N.g26 : _scrim))),
              _edge(_pill(playing ? 'Playing' : 'Paused'), _pill(name, fill: N.g20), top: false),
            ]),
          ),
        ),
      );
}

/// What is left of the preview surface when it is hidden: one 26 px line and a Show word.
class _ClosedBar extends StatelessWidget {
  const _ClosedBar({this.onShow, this.hoverShow});
  final VoidCallback? onShow;
  final bool? hoverShow;
  @override
  Widget build(BuildContext context) => Container(
        height: 26,
        padding: const EdgeInsets.symmetric(horizontal: 8),
        decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(4)),
        child: Row(children: [
          Expanded(child: Text('Preview hidden', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g76))),
          const SizedBox(width: 8),
          _Word('Show', onTap: onShow, hover: hoverShow),
        ]),
      );
}

class _BigPreview extends StatefulWidget {
  const _BigPreview({super.key, required this.w, this.open = true, this.sample = false});
  final double w;
  final bool open, sample;
  @override
  State<_BigPreview> createState() => _BigPreviewState();
}

class _BigPreviewState extends State<_BigPreview> {
  late final _root = FocusNode(onKeyEvent: _key);
  final _sc = ScrollController(initialScrollOffset: 14 * 34.0 - 68);
  late bool _open = widget.open;
  bool _play = true;
  int _sel = 14;
  List<It> get _pool => kItems.where((i) => i.place != 'Project').toList();

  KeyEventResult _key(FocusNode n, KeyEvent e) {
    if (_isDown(e, LogicalKeyboardKey.arrowDown) || _isDown(e, LogicalKeyboardKey.arrowUp)) {
      setState(() => _sel = (_sel + (e.logicalKey == LogicalKeyboardKey.arrowDown ? 1 : -1)).clamp(0, _pool.length - 1));
      _revealAt(_sc, _sel * 34.0, _sel * 34.0 + 34);
      return KeyEventResult.handled;
    }
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.space) {
      setState(() => _play = !_play);
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  @override
  void dispose() {
    _root.dispose();
    _sc.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final fw = widget.w - 20, fh = fw * 9 / 16, it = _pool[_sel];
    return Focus(
      focusNode: _root,
      autofocus: true,
      child: GestureDetector(
        behavior: HitTestBehavior.translucent,
        onTap: _root.requestFocus,
        child: Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          AnimatedSize(
            duration: Mo.dur,
            curve: Mo.ease,
            alignment: Alignment.topCenter,
            child: _open
                ? Padding(
                    padding: const EdgeInsets.fromLTRB(10, 10, 10, 0),
                    child: _SurfaceFrame(
                      w: fw,
                      h: fh,
                      sample: widget.sample,
                      playing: _play,
                      name: it.name,
                      onHide: () => setState(() => _open = false),
                      picture: _Loop(running: _play, rest: .6, builder: (context, t) => _Fx(it.fx, t, base: _ownBase(sample: widget.sample))),
                    ),
                  )
                : SizedBox(width: widget.w, child: Padding(padding: const EdgeInsets.fromLTRB(10, 10, 10, 0), child: _ClosedBar(onShow: () => setState(() => _open = true)))),
          ),
          _pad(
            Text('${_sel + 1} of ${_pool.length}', maxLines: 1, overflow: TextOverflow.ellipsis, style: _num(N.g76)),
            t: 8,
            b: 8,
          ),
          Container(height: 1, color: N.rowLine),
          Expanded(
            child: _Fit(natural: _pool.length * 34.0, child: _QuietKind(_majority(_pool), child: _FadeBottom(child: ListView.builder(
              controller: _sc,
              padding: EdgeInsets.zero,
              itemExtent: 34,
              itemCount: _pool.length,
              itemBuilder: (context, i) => _ItemRow(_pool[i], selected: i == _sel, onTap: () {
                setState(() => _sel = i);
                _root.requestFocus();
              }),
            )))),
          ),
        ]),
      ),
    );
  }
}

// ================================================================================ B10  favourites, recents, shelves

/// Rows with arrow navigation over the flat list; the item behind each row is kept for the keys.
mixin _RowKeys<T extends StatefulWidget> on State<T> {
  final sc = ScrollController();
  int sel = 0;
  List<_Ent> entries = const [];
  List<It?> rowItems = const [];

  void moveSel(int d) {
    final cnt = _rowCount(entries);
    if (cnt == 0) return;
    setState(() => sel = (sel + d).clamp(0, cnt - 1));
    final (t, b) = _entSpan(entries, sel);
    _revealAt(sc, t, b);
  }

  It? get selItem => sel >= 0 && sel < rowItems.length ? rowItems[sel] : null;
}

/// The right end of a starred-list row: a quiet word (when it was used, how often) and the star.
class _StarTail extends StatelessWidget {
  const _StarTail(this.tail, {required this.on, this.onToggle});
  final String? tail;
  final bool on;
  final VoidCallback? onToggle;
  @override
  Widget build(BuildContext context) => Row(mainAxisSize: MainAxisSize.min, children: [
        if (tail != null) Padding(padding: const EdgeInsets.only(right: 4), child: Text(tail!, style: T.label(N.g76))),
        StarButton(on: on, onChanged: (_) => onToggle?.call()),
      ]);
}

class _StarRecent extends StatefulWidget {
  const _StarRecent({super.key, this.empty = false});
  final bool empty;
  @override
  State<_StarRecent> createState() => _StarRecentState();
}

class _StarRecentState extends State<_StarRecent> with _RowKeys<_StarRecent> {
  late final _root = FocusNode(onKeyEvent: _key);
  late final Map<String, int> _uses = {for (final i in kItems) i.name: widget.empty ? 0 : i.uses};
  late final List<String> _recent = widget.empty ? [] : (kItems.where((i) => i.recent >= 0 && i.place != 'Project').toList()..sort((a, b) => a.recent.compareTo(b.recent))).map((i) => i.name).toList();
  late final Set<String> _stars = widget.empty ? {} : {'Glow', 'Gaussian Blur', 'Pop In Overshoot', 'Ease Out Back (Soft)'};
  String _say = '';

  List<It> get _pool => kItems.where((i) => i.place != 'Project').toList();
  It _by(String n) => kItems.firstWhere((i) => i.name == n);

  void _build() {
    final out = <_Ent>[], items = <It?>[];
    void row(It it, {String? tail}) {
      final i = items.length;
      items.add(it);
      out.add(_Ent.row((s) => _ItemRow(it, selected: s, onTap: () => setState(() => sel = i), onUse: () => _use(it), right: _StarTail(tail, on: _stars.contains(it.name), onToggle: () => _toggle(it)))));
    }

    void hint(String s) => out.add(_Ent.row((_) => _HintRow(s), h: 36));
    final stars = _pool.where((i) => _stars.contains(i.name)).toList();
    out.add(_Ent.head('Starred', right: '${stars.length}'));
    stars.isEmpty ? hint('No stars yet') : stars.forEach(row);
    final rec = _recent.take(6).map(_by).toList();
    out.add(_Ent.head('Recent', right: '${rec.length}'));
    rec.isEmpty ? hint('None yet') : [for (final it in rec) row(it, tail: it.recent >= 0 && _recent.first != it.name ? _ago(it.recent) : 'just now')];
    final freq = _pool.where((i) => (_uses[i.name] ?? 0) >= 5).toList()..sort((a, b) => _uses[b.name]!.compareTo(_uses[a.name]!));
    out.add(_Ent.head('Frequent', right: '${math.min(5, freq.length)}'));
    freq.isEmpty ? hint('None yet') : [for (final it in freq.take(5)) row(it, tail: '${_uses[it.name]}×')];
    out.add(_Ent.head('All', right: '${_pool.length}'));
    for (final it in _pool) {
      row(it);
    }
    entries = out;
    rowItems = items;
  }

  void _toggle(It it) => setState(() {
        if (!_stars.remove(it.name)) _stars.add(it.name);
        _say = _stars.contains(it.name) ? 'Starred “${it.name}”.' : 'Unstarred “${it.name}”.';
      });

  void _use(It it) => setState(() {
        _uses[it.name] = (_uses[it.name] ?? 0) + 1;
        _recent.remove(it.name);
        _recent.insert(0, it.name);
        _say = 'Used “${it.name}” (${_uses[it.name]}×)';
      });

  KeyEventResult _key(FocusNode n, KeyEvent e) {
    if (_isDown(e, LogicalKeyboardKey.arrowDown) || _isDown(e, LogicalKeyboardKey.arrowUp)) {
      moveSel(e.logicalKey == LogicalKeyboardKey.arrowDown ? 1 : -1);
      return KeyEventResult.handled;
    }
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.keyF && selItem != null) {
      _toggle(selItem!);
      return KeyEventResult.handled;
    }
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.enter && selItem != null) {
      _use(selItem!);
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  @override
  void dispose() {
    _root.dispose();
    sc.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    _build();
    return Focus(
      focusNode: _root,
      autofocus: true,
      child: GestureDetector(
        behavior: HitTestBehavior.translucent,
        onTap: _root.requestFocus,
        child: Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          _pad(_Tabs(_places, 0, (_) {}, toggle: false), t: 10, b: 4),
          Expanded(child: _Fit(natural: _entH(entries), child: _QuietKind(_kindOf(_places[0]), child: _FlatList(entries: entries, sel: sel, sc: sc, fade: true)))),
          _Status(_say, keys: const ['F', 'Enter']),
        ]),
      ),
    );
  }
}

/// One line of the used shelf, 30 px: thumb, name, ×count when folded. [flash] 1..0 is the g26 glow of a row that was just used.
class _UsedRow extends StatelessWidget {
  const _UsedRow(this.it, this.count, {this.flash = 0});
  final It it;
  final int count;
  final double flash;
  @override
  Widget build(BuildContext context) => Container(
        height: 30,
        padding: const EdgeInsets.symmetric(horizontal: 10),
        color: Color.lerp(_clear, N.g26, flash),
        child: Row(children: [
          _Thumb(it, w: 26, h: 18),
          const SizedBox(width: 8),
          Expanded(child: Text(it.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name())),
          if (count > 1) ...[const SizedBox(width: 8), Text('×$count', style: _num(N.g76))],
        ]),
      );
}

class _UsedStack extends StatefulWidget {
  const _UsedStack({super.key, required this.shelf});
  final int shelf;
  @override
  State<_UsedStack> createState() => _UsedStackState();
}

class _UsedStackState extends State<_UsedStack> {
  static const _cap = 12;
  late final _root = FocusNode(onKeyEvent: _key);
  final _sc = ScrollController();
  late final List<(String, int)> _stack = [for (final e in const ['Glow', 'Gaussian Blur', 'Pop In Overshoot', 'Hue / Saturation', 'Glitch RGB Split', 'Title In: Soft Rise', 'Film Grain', 'Curves', 'Echo', 'Typewriter', 'Bloom', 'Wave Warp'].indexed.take(widget.shelf)) (e.$2, const [4, 1, 3, 1, 2, 1, 1, 2, 1, 1, 1, 1][e.$1])];
  int _sel = 0, _stamp = 0;
  String _say = '';
  List<It> get _pool => kItems.where((i) => i.place != 'Project').toList();
  It _by(String n) => kItems.firstWhere((i) => i.name == n);

  void _use(It it) => setState(() {
        final at = _stack.indexWhere((e) => e.$1 == it.name);
        var cnt = 1;
        if (at >= 0) {
          cnt = _stack[at].$2 + 1;
          _stack.removeAt(at);
        }
        _stack.insert(0, (it.name, cnt));
        String? dropped;
        if (_stack.length > _cap) dropped = _stack.removeLast().$1;
        _stamp++;
        _say = at >= 0 ? '“${it.name}” ×$cnt' : (dropped != null ? 'Full: dropped “$dropped”' : 'Added “${it.name}”');
      });

  KeyEventResult _key(FocusNode n, KeyEvent e) {
    if (_isDown(e, LogicalKeyboardKey.arrowDown) || _isDown(e, LogicalKeyboardKey.arrowUp)) {
      setState(() => _sel = (_sel + (e.logicalKey == LogicalKeyboardKey.arrowDown ? 1 : -1)).clamp(0, _pool.length - 1));
      _revealAt(_sc, _sel * 34.0, _sel * 34.0 + 34);
      return KeyEventResult.handled;
    }
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.enter) {
      _use(_pool[_sel]);
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  @override
  void dispose() {
    _root.dispose();
    _sc.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Focus(
        focusNode: _root,
        autofocus: true,
        child: GestureDetector(
          behavior: HitTestBehavior.translucent,
          onTap: _root.requestFocus,
          child: Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            _Sec('Used', right: Text('${_stack.length} of $_cap', style: T.label(N.g76))),
            SizedBox(
              height: math.min(_stack.length, 6) * 30.0 + (_stack.isEmpty ? 40 : 0),
              child: _stack.isEmpty
                  ? const Align(alignment: Alignment.topCenter, child: _HintRow('Nothing used yet'))
                  : ListView.builder(
                      padding: EdgeInsets.zero,
                      itemExtent: 30,
                      itemCount: _stack.length,
                      itemBuilder: (context, i) {
                        final it = _by(_stack[i].$1);
                        return TweenAnimationBuilder<double>(
                          key: ValueKey(i == 0 ? 'top$_stamp' : 'row${_stack[i].$1}'),
                          tween: Tween(begin: i == 0 && _stamp > 0 ? 1.0 : 0.0, end: 0.0),
                          duration: const Duration(milliseconds: 600),
                          curve: Curves.easeOut,
                          builder: (context, v, _) => _UsedRow(it, _stack[i].$2, flash: v),
                        );
                      },
                    ),
            ),
            Container(height: 1, margin: const EdgeInsets.only(top: 4), color: N.rowLine),
            _Sec('Effects and presets', right: Text('${_pool.length}', style: T.label(N.g76))),
            Expanded(
              child: _Fit(natural: _pool.length * 34.0, child: _QuietKind(_majority(_pool), child: _FadeBottom(child: ListView.builder(
                controller: _sc,
                padding: EdgeInsets.zero,
                itemExtent: 34,
                itemCount: _pool.length,
                itemBuilder: (context, i) => _ItemRow(_pool[i], selected: i == _sel, onTap: () {
                  setState(() => _sel = i);
                  _root.requestFocus();
                }, onUse: () => _use(_pool[i])),
              )))),
            ),
            _Status(_say, keys: const ['Enter']),
          ]),
        ),
      );
}

/// One collection slot, 24 px: its number key mark, its name and how many it holds. An unnamed slot is drawn as an empty slot: a dashed g26 outline and the word "empty" in g63.
class _SlotRow extends StatelessWidget {
  const _SlotRow(this.n, this.name, this.count, {this.on = false, this.hover = false});
  final int n, count;
  final String name;
  final bool on, hover;
  @override
  Widget build(BuildContext context) => CustomPaint(
        foregroundPainter: name.isEmpty ? const DashedBox(N.g26, radius: 3) : null,
        child: Container(
        height: 24,
        color: on ? N.g20 : (hover ? N.g15 : null),
        child: Stack(fit: StackFit.expand, children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(10, 0, 8, 0),
            child: Row(children: [
              _Kbd('$n', on: on),
              const SizedBox(width: 8),
              Expanded(child: Text(name.isEmpty ? 'empty' : name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(name.isEmpty ? N.g76 : (on ? N.g95 : N.g91)))),
              if (name.isNotEmpty) Text('$count', style: _num(N.g76)),
            ]),
          ),
          _GutterTick(on, height: 12),
        ]),
        ),
      );
}

/// The collections a row belongs to, as the same grey number plates the slot buttons carry (the slot being shown is not repeated); more than 3 ends in a +.
class _SlotBadges extends StatelessWidget {
  const _SlotBadges(this.slots, {this.skip = 0});
  final List<int> slots;
  final int skip;
  @override
  Widget build(BuildContext context) {
    final m = [for (final n in slots) if (n != skip) n];
    return Row(mainAxisSize: MainAxisSize.min, children: [
      for (final n in m.take(3)) Padding(padding: const EdgeInsets.only(left: 2), child: _Kbd('$n')),
      if (m.length > 3) Padding(padding: const EdgeInsets.only(left: 2), child: Text('+', style: _num(N.g76))),
    ]);
  }
}

class _Collections extends StatefulWidget {
  const _Collections({super.key, required this.w});
  final double w;
  @override
  State<_Collections> createState() => _CollectionsState();
}

class _CollectionsState extends State<_Collections> {
  late final _root = FocusNode(onKeyEvent: _key);
  final _sc = ScrollController();
  final List<String> _names = ['Titles', 'Glows', 'Client A', 'Retro', 'Soft blurs', '', '', '', ''];
  final Map<String, Set<int>> _mem = {
    'Title In: Soft Rise': {1},
    'Pop In Overshoot': {1},
    'Glow': {2},
    'Bloom': {2, 5},
    'Neon Sign Flicker': {2, 3},
    '夜のネオン (client A)': {3},
    'Film Grain': {4},
    'Halation (Film Stock 500T)': {2, 4},
    'Gaussian Blur': {5},
    'Lens Blur': {5},
  };
  int _slot = 1, _sel = 0;
  String _say = '';

  List<It> get _pool => kItems.where((i) => i.place != 'Project' && (_slot < 0 || (_mem[i.name] ?? {}).contains(_slot + 1))).toList();
  int _n(int s) => kItems.where((i) => (_mem[i.name] ?? {}).contains(s)).length;

  KeyEventResult _key(FocusNode n, KeyEvent e) {
    final p = _pool;
    if (_isDown(e, LogicalKeyboardKey.arrowDown) || _isDown(e, LogicalKeyboardKey.arrowUp)) {
      setState(() => _sel = p.isEmpty ? 0 : (_sel + (e.logicalKey == LogicalKeyboardKey.arrowDown ? 1 : -1)).clamp(0, p.length - 1));
      _revealAt(_sc, _sel * 34.0, _sel * 34.0 + 34);
      return KeyEventResult.handled;
    }
    if (e is! KeyDownEvent || p.isEmpty) return KeyEventResult.ignored;
    final it = p[_sel.clamp(0, p.length - 1)];
    final k = e.logicalKey;
    final digits = [LogicalKeyboardKey.digit0, LogicalKeyboardKey.digit1, LogicalKeyboardKey.digit2, LogicalKeyboardKey.digit3, LogicalKeyboardKey.digit4, LogicalKeyboardKey.digit5, LogicalKeyboardKey.digit6, LogicalKeyboardKey.digit7, LogicalKeyboardKey.digit8, LogicalKeyboardKey.digit9];
    final d = digits.indexOf(k);
    if (d < 0) return KeyEventResult.ignored;
    setState(() {
      if (d == 0) {
        _mem.remove(it.name);
        _say = 'Took “${it.name}” out of all';
      } else {
        if (_names[d - 1].isEmpty) _names[d - 1] = 'Collection $d';
        final s = _mem.putIfAbsent(it.name, () => {});
        s.contains(d) ? s.remove(d) : s.add(d);
        _say = s.contains(d) ? '“${it.name}” → $d ${_names[d - 1]}' : '“${it.name}” left $d ${_names[d - 1]}';
      }
    });
    return KeyEventResult.handled;
  }

  @override
  void dispose() {
    _root.dispose();
    _sc.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final p = _pool, two = widget.w > 300, cw = two ? (widget.w - 4) / 2 : widget.w;
    return Focus(
      focusNode: _root,
      autofocus: true,
      child: GestureDetector(
        behavior: HitTestBehavior.translucent,
        onTap: _root.requestFocus,
        child: Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          const _Sec('Collections'),
          Padding(
            padding: EdgeInsets.zero,
            child: Wrap(spacing: 4, children: [
              for (var i = 0; i < 9; i++)
                SizedBox(
                  width: cw,
                  child: _H(
                    onTap: () => setState(() {
                      if (_names[i].isEmpty) {
                        _names[i] = 'Collection ${i + 1}';
                        _say = 'Named slot ${i + 1}';
                      } else {
                        _slot = _slot == i ? -1 : i;
                        _sel = 0;
                      }
                    }),
                    builder: (context, h) => _SlotRow(i + 1, _names[i], _n(i + 1), on: _slot == i, hover: h),
                  ),
                ),
            ]),
          ),
          const SizedBox(height: 8),
          _Band(filters: _slot < 0 ? 0 : 1, shown: p.length, total: kItems.where((i) => i.place != 'Project').length, showFilters: false, onClear: () => setState(() => _slot = -1)),
          Expanded(
            child: p.isEmpty
                ? _Empty('${_names[_slot]} is empty', const [], actions: [_Btn('Show everything', onTap: () => setState(() => _slot = -1))])
                : _Results(
                    rows: p.length,
                    more: kItems.where((i) => i.place != 'Project').length - p.length,
                    where: 'outside this collection',
                    foot: false,
                    list: _FadeBottom(child: ListView.builder(
                      controller: _sc,
                      padding: EdgeInsets.zero,
                      itemExtent: 34,
                      itemCount: p.length,
                      itemBuilder: (context, i) {
                        final m = ((_mem[p[i].name] ?? {}).toList()..sort());
                        return _ItemRow(p[i], selected: i == _sel, onTap: () {
                          setState(() => _sel = i);
                          _root.requestFocus();
                        }, right: _SlotBadges(m, skip: _slot + 1));
                      },
                    )),
                  ),
          ),
          _Status(_say, keys: const ['1-9', '0']),
        ]),
      ),
    );
  }
}

// ================================================================================ B12-b  Similar to this

final List<Asset> _lib = [
  ...sampleAssets,
  for (var i = 0; i < 24; i++) Asset('frame_${(i + 1).toString().padLeft(2, '0')}.png', Kind.image, 30 + i * 7, style: ArtStyle.values[i % 3], aspect: .7),
];

List<double> _vec(Asset a) {
  final r = math.Random(a.name.codeUnits.fold<int>(7, (s, c) => s * 31 + c) & 0xFFFF);
  return [for (var i = 0; i < 8; i++) r.nextDouble()];
}

/// A tile of the similar view: picture (g20 edge, g56 on hover, mode when selected), name, and the closeness bar with its number.
class _SimTile extends StatelessWidget {
  const _SimTile(this.a, {required this.w, this.score, this.selected = false, this.hover = false, this.source = false, this.onSimilar, this.simHover});
  final Asset a;
  final double w;
  final double? score;
  final bool selected, hover, source;
  final VoidCallback? onSimilar;
  final bool? simHover;
  @override
  Widget build(BuildContext context) => SizedBox(
        width: w,
        child: Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.start, children: [
          Container(
            height: w * .7,
            decoration: BoxDecoration(borderRadius: BorderRadius.circular(6), border: Border.all(color: selected ? Role.selected : (hover ? N.g56 : N.g20))),
            child: ClipRRect(
              borderRadius: BorderRadius.circular(5),
              child: Stack(fit: StackFit.expand, children: [
                Art(a),
                if (source) Positioned(left: 6, top: 6, child: _pill('Source', fill: N.g26)),
                if (selected) Positioned(left: 0, bottom: 0, child: CustomPaint(size: const Size(8, 8), painter: _CornerPaint(Role.selected, bottomLeft: true))),
                if (hover && !source) Positioned(right: 6, top: 6, child: _H(onTap: onSimilar, force: simHover, builder: (context, hh) => _pill('Similar', fill: hh ? N.g26 : N.g20))),
              ]),
            ),
          ),
          const SizedBox(height: 6),
          Text(a.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(selected ? N.g95 : N.g91).copyWith(decoration: selected ? TextDecoration.underline : TextDecoration.none, decorationColor: Role.selected)),
          if (score != null) ...[
            const SizedBox(height: 4),
            Row(children: [
              Expanded(child: Container(height: 2, alignment: Alignment.centerLeft, color: N.g20, child: FractionallySizedBox(widthFactor: score!.clamp(0.0, 1.0), child: Container(color: N.g76)))),
              const SizedBox(width: 8),
              Text(score!.toStringAsFixed(2), style: _num(N.g76)),
            ]),
          ],
        ]),
      );
}

class _Similar extends StatefulWidget {
  const _Similar({super.key, required this.w});
  final double w;
  @override
  State<_Similar> createState() => _SimilarState();
}

class _SimilarState extends State<_Similar> {
  late final _root = FocusNode(onKeyEvent: (n, e) {
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.keyY && _sel != null) {
      _find(_sel!);
      return KeyEventResult.handled;
    }
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape && _from != null) {
      setState(() => _from = null);
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  });
  Asset? _from = sampleAssets[0], _sel = sampleAssets[0];
  String _say = '';

  void _find(Asset a) => setState(() {
        _from = a;
        _sel = a;
        _say = '';
      });

  List<(Asset, double)> get _list {
    if (_from == null) return [for (final a in _lib) (a, 0.0)];
    final v = _vec(_from!);
    double d(Asset a) {
      final u = _vec(a);
      var s = 0.0;
      for (var i = 0; i < 8; i++) {
        s += (u[i] - v[i]) * (u[i] - v[i]);
      }
      return math.sqrt(s);
    }

    final all = [for (final a in _lib) (a, d(a))];
    final mx = all.map((e) => e.$2).reduce(math.max);
    final out = [for (final e in all) (e.$1, 1 - e.$2 / mx)];
    out.sort((a, b) => b.$2.compareTo(a.$2));
    return out;
  }

  @override
  void dispose() {
    _root.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final cols = widget.w > 300 ? 3 : 2, tw = (widget.w - 20 - (cols - 1) * 6) / cols, l = _list;
    return Focus(
      focusNode: _root,
      autofocus: true,
      child: GestureDetector(
        behavior: HitTestBehavior.translucent,
        onTap: _root.requestFocus,
        child: Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          _pad(_Tabs(const ['All', 'Image', 'Video', 'Audio', '3D'], 0, (_) {}, toggle: false), t: 10, b: 8),
          if (_from != null)
            _pad(
              Align(
                alignment: Alignment.centerLeft,
                child: _FilterChip('similar', _from!.name, onRemove: () => setState(() => _from = null)),
              ),
              b: 8,
            ),
          _Band(filters: _from == null ? 0 : 1, shown: l.length, total: _lib.length, showFilters: false, showCount: _from == null, extra: _from == null ? null : 'by similarity', onClear: () => setState(() => _from = null)),
          Expanded(
            child: _FadeBottom(child: SingleChildScrollView(
              padding: const EdgeInsets.fromLTRB(10, 10, 10, 10),
              child: Wrap(spacing: 6, runSpacing: 10, children: [
                for (final e in l)
                  _H(
                    onTap: () => setState(() => _sel = e.$1),
                    onDouble: () => _find(e.$1),
                    onSecondary: () => _find(e.$1),
                    builder: (context, h) => _SimTile(e.$1, w: tw, score: _from == null ? null : e.$2, selected: identical(_sel, e.$1), hover: h, source: _from != null && identical(e.$1, _from), onSimilar: () => _find(e.$1)),
                  ),
              ]),
            )),
          ),
          _Status(_say, keys: const ['Y', 'Esc']),
        ]),
      ),
    );
  }
}

// ================================================================================ the set

// ================================================================================ Parts @200%
// Each Parts sheet draws the small parts of one option group at twice their size, every state side by side, a 10 px caption under each.
// Vector text and lines are scaled by the canvas (FittedBox), not rasterised: nothing is blurred, so a 1 px line is a 2 px line and a padding is exactly doubled.
// Cell ground is g10 (the panel's own); the sheet is g00 so the cell's edge shows. Hover / focus are pinned through each part's own parameter, never faked.

class _PC {
  const _PC(this.caption, this.w, this.child, {this.h, this.ground = N.g10, this.pad = 0});
  final String caption;
  final double w;
  final double? h;
  final Widget child;
  final Color ground;
  final double pad;
}

class _Zoom extends StatelessWidget {
  const _Zoom(this.c);
  final _PC c;
  static const k = 2.0;
  @override
  Widget build(BuildContext context) {
    final w = c.w + c.pad * 2;
    return SizedBox(
      width: w * k,
      child: Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.start, children: [
        Text(c.caption, softWrap: true, style: T.label(N.g76).copyWith(height: 1.3)),
        const SizedBox(height: 4),
        ColoredBox(
          color: c.ground,
          child: SizedBox(
            width: w * k,
            child: FittedBox(fit: BoxFit.fitWidth, alignment: Alignment.topLeft, child: SizedBox(width: w, height: c.h == null ? null : c.h! + c.pad * 2, child: Padding(padding: EdgeInsets.all(c.pad), child: c.child))),
          ),
        ),
      ]),
    );
  }
}

class _Parts extends StatefulWidget {
  const _Parts(this.id, this.title, this.note, this.sections);
  final String id, title, note;
  final List<(String, List<_PC>)> sections;
  @override
  State<_Parts> createState() => _PartsState();
}

class _PartsState extends State<_Parts> {
  final _v = ScrollController(), _h = ScrollController();
  late final List<GlobalKey> _keys = [for (final _ in widget.sections) GlobalKey()];
  @override
  void dispose() {
    _v.dispose();
    _h.dispose();
    super.dispose();
  }

  void _go(int i) {
    final c = _keys[i].currentContext;
    if (c != null) Scrollable.ensureVisible(c, duration: Mo.dur, curve: Mo.ease, alignment: 0, alignmentPolicy: ScrollPositionAlignmentPolicy.explicit);
  }

  /// Every lower row must be reachable without a pointer wheel: PageUp / PageDown / Home / End / arrows scroll the sheet, and the jump bar at the top goes to a section.
  KeyEventResult _key(FocusNode n, KeyEvent e) {
    if (!(e is KeyDownEvent || e is KeyRepeatEvent) || !_v.hasClients) return KeyEventResult.ignored;
    final vp = _v.position.viewportDimension, k = e.logicalKey;
    double? to;
    if (k == LogicalKeyboardKey.pageDown || k == LogicalKeyboardKey.space) to = _v.offset + vp * .9;
    if (k == LogicalKeyboardKey.pageUp) to = _v.offset - vp * .9;
    if (k == LogicalKeyboardKey.arrowDown) to = _v.offset + 80;
    if (k == LogicalKeyboardKey.arrowUp) to = _v.offset - 80;
    if (k == LogicalKeyboardKey.home) to = 0;
    if (k == LogicalKeyboardKey.end) to = _v.position.maxScrollExtent;
    if (to == null) return KeyEventResult.ignored;
    _v.jumpTo(to.clamp(0.0, _v.position.maxScrollExtent));
    return KeyEventResult.handled;
  }

  @override
  Widget build(BuildContext context) => ColoredBox(
        color: N.g00,
        child: Focus(
          autofocus: true,
          onKeyEvent: _key,
          child: LayoutBuilder(builder: (context, box) {
          // The sheet is as wide as the viewport, or as the widest cell if that is wider: then it scrolls sideways instead of being cut.
          final widest = [for (final s in widget.sections) for (final c in s.$2) (c.w + c.pad * 2) * _Zoom.k].fold<double>(0, math.max);
          final inner = math.max(box.maxWidth - 32 - 12, widest);
          return RawScrollbar(
            controller: _v,
            thumbVisibility: true,
            thumbColor: N.g38,
            radius: const Radius.circular(3),
            thickness: 6,
            child: RawScrollbar(
              controller: _h,
              thumbVisibility: true,
              thumbColor: N.g38,
              radius: const Radius.circular(3),
              thickness: 6,
              notificationPredicate: (n) => n.depth == 1,
              child: SingleChildScrollView(
                controller: _v,
                child: SingleChildScrollView(
                  controller: _h,
                  scrollDirection: Axis.horizontal,
                  child: Padding(
                    padding: const EdgeInsets.fromLTRB(16, 16, 28, 28),
                    child: SizedBox(
                      width: inner,
                      child: Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.start, children: [
                        Row(crossAxisAlignment: CrossAxisAlignment.center, children: [
                          Text(widget.id, style: T.value(N.g95).copyWith(fontWeight: FontWeight.w700)),
                          const SizedBox(width: 8),
                          Text(widget.title, style: T.title()),
                          const SizedBox(width: 8),
                          Text('@200%', style: T.micro(N.g76)),
                        ]),
                        const SizedBox(height: 8),
                        Text(widget.note, style: T.label(N.g76).copyWith(height: 1.35)),
                        const SizedBox(height: 10),
                        Wrap(spacing: 12, runSpacing: 6, crossAxisAlignment: WrapCrossAlignment.center, children: [
                          Text('JUMP TO', style: T.micro(N.g76).copyWith(letterSpacing: .8)),
                          for (var i = 0; i < widget.sections.length; i++) _Word(widget.sections[i].$1, style: T.label(N.g76), onTap: () => _go(i)),
                          Text('(PageDown, End and the arrows scroll)', style: T.label(N.g76)),
                        ]),
                        for (var i = 0; i < widget.sections.length; i++) ...[
                          const SizedBox(height: 24),
                          KeyedSubtree(key: _keys[i], child: Text(widget.sections[i].$1.toUpperCase(), style: T.micro(N.g76).copyWith(letterSpacing: .8))),
                          const SizedBox(height: 8),
                          Wrap(spacing: 16, runSpacing: 16, children: [for (final c in widget.sections[i].$2) _Zoom(c)]),
                        ],
                        const SizedBox(height: 24),
                        Text('END OF SHEET', style: T.micro(N.g76).copyWith(letterSpacing: .8)),
                      ]),
                    ),
                  ),
                ),
              ),
            ),
          );
        }),
        ),
      );
}

WidgetbookUseCase _parts(String name, Widget Function() build) => WidgetbookUseCase(name: 'Parts @200% $name', builder: (_) => build());

It _by(String n) => kItems.firstWhere((i) => i.name == n);
Widget _left(Widget w) => Align(alignment: Alignment.centerLeft, child: w);

/// A search field with its own text and focus node; [focus] pins the focused look.
class _SearchPart extends StatefulWidget {
  const _SearchPart({this.text = '', this.focus = false, this.hover, this.clearHover, this.hint = 'Search effects, presets, media', this.slash = true, this.trailing});
  final String text, hint;
  final bool focus, slash;
  final bool? hover, clearHover;
  final Widget? trailing;
  @override
  State<_SearchPart> createState() => _SearchPartState();
}

class _SearchPartState extends State<_SearchPart> {
  late final _c = TextEditingController(text: widget.text);
  final _n = FocusNode();
  @override
  void dispose() {
    _c.dispose();
    _n.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => _Search(controller: _c, node: _n, hint: widget.hint, slash: widget.slash, trailing: widget.trailing, focusLook: widget.focus, hoverLook: widget.hover, clearHover: widget.clearHover);
}

const _statusLong = 'Applied “Directional Blur (Motion Trail, 8-sample, GPU)” to Layer 3';

// ---- B3 Find
Widget _pFindChrome() {
  // One plate width per row of the sheet: 300 fields, 212 narrow, 72 marks, 64 cells, 320 lines.
  _PC field(String cap, Widget w, {double width = 300}) => _PC(cap, width, w, h: 28, pad: 8);
  _PC mark(String cap, Widget w, {double h = 16}) => _PC(cap, 72, _left(w), h: h, pad: 8);
  _PC cell(String cap, Widget w) => _PC(cap, 64, _left(w), h: 22, pad: 8, ground: N.g07);
  _PC line(String cap, Widget w, {double? h}) => _PC(cap, 320, w, h: h);
  Widget count(String t) => Padding(padding: const EdgeInsets.fromLTRB(10, 8, 10, 6), child: Text(t, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g76)));
  const longText = 'Directional Blur motion trail 8-sample GPU soft';
  return _Parts('B3', 'Find: field, tabs, key marks, status', 'The chrome every find option shares. Field 300 x 28 in a 320 panel; tabs are a 26 px trough with 22 px cells; key marks 16 px. Field edge: rest g20, hover g38, focus g63. Not designed: a disabled field (find is always on).', [
    ('Search field: states', [
      field('rest, empty: hint + "/" key', const _SearchPart()),
      field('hover, empty: g38 edge', const _SearchPart(hover: true)),
      field('focused, empty: g63 edge, no "/" key', const _SearchPart(focus: true)),
      field('typed, resting: clear mark', const _SearchPart(text: 'bl')),
      field('typed, hover: g38 edge', const _SearchPart(text: 'bl', hover: true)),
      field('typed, clear mark hovered: x g95', const _SearchPart(text: 'bl', clearHover: true)),
      field('typed, focused', const _SearchPart(text: 'bl', focus: true)),
      field('Tab menu variant: Tab key mark', const _SearchPart(focus: true, slash: false, hint: 'Add effect to Layer 3', trailing: _Kbd('Tab'))),
    ]),
    ('Search field: long text', [
      field('long text, resting: ellipsis', const _SearchPart(text: longText)),
      field('long text, focused: fades at the edge', const _SearchPart(text: longText, focus: true)),
    ]),
    ('Search field: narrow dock 232 (field 212)', [
      field('narrow dock 232: field 212, hint', const _SearchPart(), width: 212),
      field('narrow dock 232: typed, resting', const _SearchPart(text: 'bl'), width: 212),
      field('narrow dock 232: long text, resting', const _SearchPart(text: longText), width: 212),
      field('narrow dock 232: long text, focused', const _SearchPart(text: longText, focus: true), width: 212),
    ]),
    ('Key marks and the clear mark (one centre, one right inset)', [
      _PC('"/" and the clear mark in the same 18 px slot: same centre, same 8 px right inset', 300, Column(crossAxisAlignment: CrossAxisAlignment.end, mainAxisSize: MainAxisSize.min, children: [
        Container(height: 20, padding: const EdgeInsets.only(right: 8), alignment: Alignment.centerRight, color: N.g07, child: const SizedBox(width: 18, child: _Kbd('/'))),
        Container(height: 20, padding: const EdgeInsets.only(right: 8), alignment: Alignment.centerRight, color: N.g07, child: const _Cross(w: 18)),
      ]), pad: 8),
      mark('"/" rest', const _Kbd('/')),
      mark('Tab', const _Kbd('Tab')),
      mark('Enter', const _Kbd('Enter')),
      mark('Esc', const _Kbd('Esc')),
      mark('live mark (g26)', const _Kbd('#', on: true)),
      mark('clear mark rest (g63)', const _Cross(w: 18), h: 20),
      mark('clear mark hover (g95)', const _Cross(w: 18, hover: true), h: 20),
    ]),
    ('Tabs', [
      _PC('nothing selected = all places: four cells at rest (no tab is added for All)', 300, _Tabs(_places, -1, (_) {}, toggle: false), h: 26, pad: 8),
      _PC('first selected: g20 pill + g95 + 1 px accent line', 300, _Tabs(_places, 0, (_) {}, toggle: false), h: 26, pad: 8),
      _PC('selected 1st, hover on 2nd (g15)', 300, _Tabs(_places, 0, (_) {}, toggle: false, hoverIndex: 1), h: 26, pad: 8),
      _PC('narrow 212: scrolls, text never shrinks', 212, _Tabs(_places, 0, (_) {}, toggle: false), h: 26, pad: 8),
      cell('one cell: rest', const _TabCell('Effects')),
      cell('one cell: hover', const _TabCell('Effects', hover: true)),
      cell('one cell: selected (g20 pill, g95, 1 px accent line)', const _TabCell('Effects', on: true)),
      cell('one cell: selected + hover', const _TabCell('Effects', on: true, hover: true)),
    ]),
    ('Count line, heading, status', [
      line('count line, no tab on', count('12 of 82  ·  no tab on: all places')),
      line('count line, one tab on', count('9 of 31  ·  Presets')),
      line('foot, results outside the scope: "N more in other places" (only what the count line does not show)', const _EndLine(22), h: 26),
      line('foot, nothing outside and the count line does not say so: "End of results" (otherwise no foot)', const _EndLine(0), h: 26),
      line('count line, all places, one place is most rows: said once', count('72 of 72  ·  no tab on: all places  ·  mostly effects')),
      line('count line, no result', count('No results')),
      line('section heading + count', _Sec('Effects', right: Text('12', style: T.label(N.g76)))),
      line('status, short', const _Status('Layer 3 · Title', keys: ['Esc']), h: 26),
      line('status, long: one line, ellipsis', const _Status(_statusLong), h: 26),
      _PC('status, narrow 232', 232, const _Status('Layer 3 · Title', keys: ['Esc']), h: 26),
    ]),
    ('Empty result', [
      line('no match: what was searched + ways out', _Empty('Nothing matches “glwo”', const ['Searched only Presets.'], actions: [_Btn('Clear search', onTap: () {}), _Btn('Search all places', onTap: () {}), _Btn('Did you mean “glow”?', onTap: () {})]), h: 220),
    ]),
  ]);
}

Widget _pRows() {
  final g = _by('Gaussian Blur'), long = _by('Directional Blur (Motion Trail, 8-sample, GPU)'), jp = _by('夜のネオン (client A)'), pre = _by('Typewriter'), media = _by('lens_bloom.mp4'), lut = _by('Apply LUT (Cinematic Teal & Orange 3D)');
  _PC row(String cap, Widget w, {double width = 320}) => _PC(cap, width, w, h: 34);
  return _Parts('B3', 'Find: item rows', 'The row every list shares, 34 px. Two left edges on purpose: the 2 px selected tick sits in the gutter (x 2 to 4) and the content edge stays at 10 px, so the tick never pushes text. Thumbnail and name start at x 10. Name 11 px, second line 10 px g63, kind or place on the right. A filter that already fixes a fact is not repeated by its rows.', [
    ('States', [
      row('rest', _ItemRow(g)),
      row('hover: g15', _ItemRow(g, hover: true)),
      row('selected: g20 + 2 px tick', _ItemRow(g, selected: true)),
      row('query "bl" matched, rest', _ItemRow(g, tokens: const ['bl'])),
      row('query "bl" matched, selected', _ItemRow(g, tokens: const ['bl'], selected: true)),
    ]),
    ('Content', [
      row('long English name: ellipsis', _ItemRow(long)),
      row('long name, selected', _ItemRow(long, selected: true)),
      row('Japanese-led name', _ItemRow(jp)),
      row('long name with units', _ItemRow(lut)),
      row('preset', _ItemRow(pre)),
      row('media (asset art)', _ItemRow(media)),
      row('mine: place on the right, not repeated below', _ItemRow(_by('My Soft Bloom 01'), right: Text('Mine', style: T.micro(N.g76)))),
      row('grouped under a heading: no label on the right', _ItemRow(g, right: const SizedBox.shrink())),
      row('place on the right (across places)', _ItemRow(g, right: Text(g.place, style: T.micro(N.g76)))),
    ]),
    ('A filter already says it: the row does not repeat it', [
      row('no filter: kind on the right, category in line 2', _ItemRow(g)),
      row('Kind = Effect on: no kind label on the right', _ItemRow(g, showKind: false)),
      row('Category = Blur on: line 2 has no category', _ItemRow(g, showCat: false)),
      row('Kind and Category on: nothing repeated', _ItemRow(g, showKind: false, showCat: false)),
      row('mixed list, media: place on the right, line 2 has no "Video"', _ItemRow(media, right: Text(media.place, style: T.micro(N.g76)))),
      row('mixed list, effect: place on the right, category kept', _ItemRow(g, right: Text(g.place, style: T.micro(N.g76)))),
    ]),
    ('Thumbnails: one shape per family', [
      row('blur: concentric soft rings', _ItemRow(_by('Radial Blur'), showKind: false)),
      row('glow: a halo', _ItemRow(_by('Light Rays'), showKind: false)),
      row('warp: a wave', _ItemRow(_by('Twirl'), showKind: false)),
      row('stylize: blocks', _ItemRow(_by('Halftone'), showKind: false)),
      row('colour: a gradient chip', _ItemRow(_by('Levels'), showKind: false)),
      row('glitch / echo / grain / wipe / pop', _ItemRow(_by('VHS Wobble'), showKind: false)),
    ]),
    ('Narrow dock 232', [
      row('rest', _ItemRow(long), width: 232),
      row('selected', _ItemRow(long, selected: true), width: 232),
      row('Japanese-led, hover', _ItemRow(jp, hover: true), width: 232),
    ]),
  ]);
}

// ---- B3 Tab menu
Widget _pMenu() {
  final g = _by('Glow'), long = _by('Directional Blur (Motion Trail, 8-sample, GPU)');
  _PC mrow(String cap, Widget w) => _PC(cap, 296, w, h: 28, ground: N.g13);
  return _Parts('B3-b', 'Tab menu: pieces', 'The small menu that opens at the cursor: g13, 1 px g26 edge, 6 px radius. Rows 28 px, family bar 26 px. One left rhythm: the tick in the gutter, field, tabs, names and headings all start at x 10.', [
    ('Family bar', [
      _PC('rest: g63 word', 72, const Center(child: _TabCell('Distort')), h: 26, ground: N.g13),
      _PC('hover: g15', 72, const Center(child: _TabCell('Distort', hover: true)), h: 26, ground: N.g13),
      _PC('selected: g20 pill + g95 + 1 px accent line', 72, const Center(child: _TabCell('Distort', on: true)), h: 26, ground: N.g13),
    ]),
    ('Menu rows', [
      mrow('rest: family on the right', _MenuRow(g.name, tail: g.cat)),
      mrow('hover: g15', _MenuRow(g.name, tail: g.cat, hover: true)),
      mrow('selected: g20 + Enter key mark', _MenuRow(g.name, selected: true)),
      mrow('empty input: last used, time on the right', _MenuRow(g.name, tail: _ago(3))),
      mrow('query "gl" matched, rest', _MenuRow(g.name, tokens: const ['gl'], tail: g.cat)),
      mrow('query "gl" matched, selected', _MenuRow(g.name, tokens: const ['gl'], selected: true)),
      mrow('long name: ellipsis, tail kept', _MenuRow(long.name, tail: _ago(240))),
      mrow('selected without a target: tail, no Enter', _MenuRow(g.name, selected: true, tail: g.cat)),
    ]),
    ('Heading, notice, empty line', [
      _PC('heading: last used', 296, const _Sec('Last used first', pad: EdgeInsets.fromLTRB(10, 6, 10, 2)), ground: N.g13),
      _PC('heading: matches', 296, const _Sec('Matches · 4', pad: EdgeInsets.fromLTRB(10, 6, 10, 2)), ground: N.g13),
      _PC('no target: danger bar, g10 on g13', 276, const _Notice(_noTargetText), ground: N.g13, pad: 10),
      _PC('nothing in this family', 296, Padding(padding: const EdgeInsets.fromLTRB(10, 4, 10, 10), child: Text('Nothing in Glow matches “zz”', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g76))), ground: N.g13),
    ]),
    ('Input and the closed card', [
      _PC('input: Tab key mark', 276, const _SearchPart(text: 'gl', focus: true, slash: false, trailing: _Kbd('Tab')), h: 28, pad: 10, ground: N.g13),
      _PC('input, no target', 276, const _SearchPart(slash: false, hint: 'Add effect (no target)', trailing: _Kbd('Tab')), h: 28, pad: 10, ground: N.g13),
      _PC('closed card: after Enter', 296, const _ClosedCard('Applied “Glow” to Layer 3 · Title')),
      _PC('closed card: after Esc', 296, const _ClosedCard('Closed, nothing written')),
    ]),
  ]);
}

// ---- B3 Prefix
Widget _pPrefix() {
  _PC row(String cap, Widget w, {double width = 320}) => _PC(cap, width, w, h: 34);
  return _Parts('B3-c', 'Prefix: rows, area line, marks', 'Rows of tags, actions and layers share the item row geometry: tick, name 11 px, second line 10 px, count or key on the right.', [
    ('Rows', [
      row('tag row', const _PlainRow(title: '#soft', sub: 'tag', right: '12', count: true, selected: false)),
      row('tag row, hover', const _PlainRow(title: '#soft', sub: 'tag', right: '12', count: true, selected: false, hover: true)),
      row('tag row, selected', const _PlainRow(title: '#soft', sub: 'tag', right: '12', count: true, selected: true)),
      row('action: mono key on the right', const _PlainRow(title: 'Add keyframe at playhead', sub: '現在位置にキーを打つ', right: 'K', selected: false, kbd: true)),
      row('action: long name and long key', const _PlainRow(title: 'Reset all effects on Layer 3', sub: 'Layer 3 のエフェクトをリセット', right: 'Cmd Shift R', selected: false, kbd: true)),
      row('action: query "re" matched', const _PlainRow(title: 'Reset all effects on Layer 3', sub: 'Layer 3 のエフェクトをリセット', right: 'Cmd Shift R', selected: false, kbd: true, toks: ['re'])),
      row('layer row', const _PlainRow(title: 'キャッチコピー', sub: 'Layer', right: 'Text', selected: false)),
      row('mark row: the example and the chevron', const _PlainRow(title: '#  tag', sub: '#soft', right: '›', selected: false)),
      row('narrow 232, long action', const _PlainRow(title: 'Align to composition center', sub: 'コンポジションの中央に整列', right: 'Cmd Alt C', selected: false, kbd: true), width: 232),
    ]),
    ('Area line and heading', [
      _PC('no mark: all areas', 300, const _AreaLine('all areas', ''), pad: 8),
      _PC('# lit', 300, const _AreaLine('tags', '#'), pad: 8),
      _PC('> lit', 300, const _AreaLine('actions', '>'), pad: 8),
      _PC('@ lit', 300, const _AreaLine('layers', '@'), pad: 8),
      _PC('section heading + count', 320, _Sec('Tags', right: Text('12', style: T.label(N.g76)))),
      _PC('capped section: what is hidden + the key', 320, const _MoreLine('+10 more'), h: 24),
    ]),
    ('Field and empty', [
      _PC('field hint, 300', 300, const _SearchPart(hint: 'Search, or use # > @'), h: 28, pad: 8),
      _PC('field hint, narrow 212', 212, const _SearchPart(hint: 'Search, or use # > @'), h: 28, pad: 8),
      _PC('typed mark', 300, const _SearchPart(text: '#so', focus: true), h: 28, pad: 8),
      _PC('nothing in this area', 320, _Empty('Nothing in tags for “zz”', const [], actions: [_Btn('Search all areas', onTap: () {})]), h: 180),
    ]),
  ]);
}

// ---- B3 Empty query
Widget _pFreq() {
  final g = _by('Gaussian Blur'), t = _by('Title In: Soft Rise');
  _PC row(String cap, Widget w, {double width = 320}) => _PC(cap, width, w, h: 34);
  return _Parts('B3-d', 'Empty query: headings, hints, reasons', 'Four sections answer an empty field. The reason a row is there is a g63 word on its right, at most 92 px.', [
    ('Headings', [
      _PC('Recent + where it comes from', 320, _Sec('Recent', right: Text('by you, all projects', style: T.label(N.g76))), h: 26),
      _PC('Fits + the selection', 320, const _Sec('Fits Layer 3 · Title (Text)'), h: 26),
      _PC('All others + count', 320, _Sec('All others', right: Text('64', style: T.label(N.g76))), h: 26),
    ]),
    ('Hints instead of empty boxes', [
      _PC('first run: Recent', 320, const _HintRow('None yet'), h: 36),
      _PC('no selection: Fits', 320, const _HintRow('No selection'), h: 36),
      _PC('narrow 232', 232, const _HintRow('No selection'), h: 36),
    ]),
    ('Reasons on the right of a row', [
      row('recent: time', _ItemRow(g, right: const _Why('3 min ago'))),
      row('fits: why', _ItemRow(t, right: const _Why('made for text'))),
      row('fits: why, long: ellipsis at 92', _ItemRow(t, right: const _Why('used on 12 text layers today'))),
      row('others: use count', _ItemRow(g, right: const _Why('41×'))),
      row('selected, with a reason', _ItemRow(t, selected: true, right: const _Why('made for text'))),
      row('hover, with a reason', _ItemRow(t, hover: true, right: const _Why('made for text'))),
      row('key selected: curve for keys', _ItemRow(_by('Ease Out Back (Soft)'), right: const _Why('curve for keys'))),
    ]),
  ]);
}

// ---- B2-a Tag bands
Widget _pTags() {
  _PC chip(String cap, Widget c, {double w = 104}) => _PC(cap, w, _left(c), h: 22, pad: 8);
  _PC grey(String cap, Color c) => _PC(cap, 72, Container(height: 22, decoration: BoxDecoration(color: c, border: Border.all(color: N.g20), borderRadius: BorderRadius.circular(4))), h: 22, pad: 8);
  const kind = ['Effect', 'Preset', 'Media'], kc = {'Effect': 52, 'Preset': 15, 'Media': 15};
  const cat = ['Blur', 'Color', 'Glow', 'Distort', 'Stylize', 'Time', 'Generate', 'Title', 'Motion', 'Ease'];
  const cc = {'Blur': 6, 'Color': 7, 'Glow': 6, 'Distort': 7, 'Stylize': 6, 'Time': 3, 'Generate': 5, 'Title': 5, 'Motion': 5, 'Ease': 0};
  Widget grp(String label, List<String> v, Set<String> on, {Map<String, int>? counts, bool open = true, bool hide = false, int? hover, bool? zero, bool? hoverToggle}) => _TagGroup(label: label, values: v, on: on, counts: counts, open: open, canHide: hide, hoverIndex: hover, zeroOpen: zero, hoverToggle: hoverToggle, onTap: (_) {});
  return _Parts('B2-a', 'Tag bands: chips and groups', 'A chip is 22 px, on the panel plate g10. Rest g15 (visibly above the plate), hover g20, on g26 + a 1 px accent line, pressed g20 + the accent line at 50%, keyboard focus a 1 px g63 edge. Count 0 is not drawn at rest: the group folds all of them into one "+N hidden tags" text button on the chip line (g76, g20 on hover, a 10 px chevron; "+1 hidden tag" for one); shown, they are g56 with a dashed g56 hairline and no fill, so they never read as live chips. Plates fit their content (the cell width is the plate width). A group row is 64 px of name (g63) and the chips; label at x 10, chips from x 74. List rows keep two left edges on purpose: the selected tick sits in the gutter (x 2 to 4), the content edge stays at 10 px.', [
    ('The greys (cell ground is the plate)', [
      grey('plate g10 (panel body)', N.g10),
      grey('rest g15', N.g15),
      grey('hover g20', N.g20),
      grey('on g26 (+ accent line)', N.g26),
      grey('label / count text g63', N.g13),
    ]),
    ('Chip states', [
      chip('rest: g15 on the g10 plate', const _ChipBtn('Blur', count: 6)),
      chip('hover: g20', const _ChipBtn('Blur', count: 6, hover: true)),
      chip('pressed: g20 + accent line 50%', const _ChipBtn('Blur', count: 6, pressed: true)),
      chip('keyboard focus: 1 px g63 edge', const _ChipBtn('Blur', count: 6, focus: true)),
      chip('on: g26 + 1 px accent line', const _ChipBtn('Blur', count: 6, on: true)),
      chip('on + hover: g26 + line + g63 edge', const _ChipBtn('Blur', count: 6, on: true, hover: true)),
      chip('no count (saved filter)', const _ChipBtn('Soft')),
      chip('no count, on', const _ChipBtn('Soft', on: true)),
      chip('two-digit count', const _ChipBtn('Stylize', count: 18), w: 120),
      chip('two-digit count, on', const _ChipBtn('Stylize', count: 18, on: true), w: 120),
      chip('long word, hover', const _ChipBtn('Directional', count: 112, hover: true), w: 136),
      chip('zero, when shown: g63 word, dashed g56 hairline, no fill', const _ChipBtn('Blur', count: 0), w: 120),
      chip('zero + hover: no change', const _ChipBtn('Blur', count: 0, hover: true), w: 120),
    ]),
    ('A long chip in a narrow group (ends in an ellipsis, never widens the row)', [
      _PC('232 dock: group 142 wide, chip takes what it has', 232, grp('Source', const ['Directional Blur Motion Trail 8-sample GPU', 'Mine'], {}, counts: const {'Directional Blur Motion Trail 8-sample GPU': 4, 'Mine': 3})),
      _PC('232 dock: the same long chip, on', 232, grp('Source', const ['Directional Blur Motion Trail 8-sample GPU', 'Mine'], {'Directional Blur Motion Trail 8-sample GPU'}, counts: const {'Directional Blur Motion Trail 8-sample GPU': 4, 'Mine': 3})),
    ]),
    ('Group rows, 320 wide', [
      _PC('open, nothing chosen', 320, grp('Kind', kind, {}, counts: kc)),
      _PC('open, one on, one hover', 320, grp('Kind', kind, {'Effect'}, counts: kc, hover: 1)),
      _PC('open, chips pack per line (Distort stays on line 1 when it fits); the zero chip folded: "+1 hidden tag", right after the last chip', 320, grp('Category', cat, {'Blur'}, counts: cc)),
      _PC('narrow 232: the row is full, chips pack to the width, the button follows the last chip and wraps only when the width is out', 232, grp('Category', cat, {}, counts: cc)),
      _PC('toggle hovered: g20 fill under "+1 hidden tag"', 320, grp('Category', cat, {'Blur'}, counts: cc, hoverToggle: true)),
      _PC('same group, hidden ones shown: dashed, g56, then "Hide 1"', 320, grp('Category', cat, {'Blur'}, counts: cc, zero: true)),
      _PC('many zeros (Media filtered): 8 dead chips collapse into one word', 320, grp('Category', cat, {}, counts: const {'Blur': 0, 'Color': 0, 'Glow': 6, 'Distort': 0, 'Stylize': 0, 'Time': 0, 'Generate': 0, 'Title': 0, 'Motion': 0, 'Ease': 0})),
      _PC('same, expanded', 320, grp('Category', cat, {}, counts: const {'Blur': 0, 'Color': 0, 'Glow': 6, 'Distort': 0, 'Stylize': 0, 'Time': 0, 'Generate': 0, 'Title': 0, 'Motion': 0, 'Ease': 0}, zero: true)),
      _PC('folded, nothing chosen', 232, grp('Category', const ['Blur', 'Color', 'Glow'], {}, open: false)),
      _PC('folded, a choice stays visible', 232, grp('Category', const ['Blur', 'Color', 'Glow'], {'Blur', 'Glow'}, open: false)),
      _PC('open in a dock: Hide word', 232, grp('Kind', kind, {}, hide: true)),
    ]),
    ('Words that act', [
      _PC('Show N: rest', 96, _left(const _Word('Show 10')), h: 14, pad: 8),
      _PC('Show N: hover, underlined', 96, _left(const _Word('Show 10', hover: true)), h: 14, pad: 8),
      _PC('+N hidden tags: rest (g76, chevron)', 120, _left(const _HiddenToggle(8)), h: 22, pad: 8),
      _PC('+N hidden tags: hover (g20 fill)', 120, _left(const _HiddenToggle(8, hover: true)), h: 22, pad: 8),
      _PC('+N hidden tags: pressed (g20 + accent line 50%)', 120, _left(const _HiddenToggle(8, pressed: true)), h: 22, pad: 8),
      _PC('+N hidden tags: keyboard focus (1 px g63 edge)', 120, _left(const _HiddenToggle(8, focus: true)), h: 22, pad: 8),
      _PC('+1 hidden tag: singular', 120, _left(const _HiddenToggle(1)), h: 22, pad: 8),
      _PC('open: Hide N, chevron up', 96, _left(const _HiddenToggle(8, open: true)), h: 22, pad: 8),
      _PC('status line, rest', 320, const _Status('Only', keys: ['Cmd', 'Shift']), h: 26),
      _PC('status line, add mode', 320, const _Status('Add', keys: ['Cmd', 'Shift']), h: 26),
    ]),
    ('End of a short list (always shown, directly under the last row)', [
      _PC('more results exist: only the cue, the count lives in the band', 320, const _EndLine(64, where: 'hidden by the filters'), h: 26),
      _PC('nothing more and the band does not say it: End of results', 320, const _EndLine(0, where: 'hidden by the filters'), h: 26),
    ]),
  ]);
}

// ---- B4 band
Widget _pBand() {
  _PC b(String cap, Widget w, {double width = 320}) => _PC(cap, width, w, h: 28);
  return _Parts('B4', 'Results band: states', 'One 28 px band under the filters: g13, 1 px row line above and below. The filter count is the brightest word; Clear is one word.', [
    ('Band', [
      b('no filter: quiet, Clear dimmed (g63, g20 hairline, no hover)', _Band(filters: 0, shown: 82, total: 82, onClear: () {})),
      b('no filter, pointer over Clear: no change (dimmed, arrow)', _Band(filters: 0, shown: 82, total: 82, onClear: () {}, hoverClear: true)),
      b('one filter: Clear at rest (g76, g26 hairline)', _Band(filters: 1, shown: 52, total: 82, onClear: () {})),
      b('three filters', _Band(filters: 3, shown: 6, total: 82, onClear: () {})),
      b('three filters, Clear hover (g20 fill)', _Band(filters: 3, shown: 6, total: 82, onClear: () {}, hoverClear: true)),
      b('three filters, Clear pressed (g20 + accent line 50%)', _Band(filters: 3, shown: 6, total: 82, onClear: () {}, pressClear: true)),
      b('three filters, Clear keyboard focus (1 px g63 edge)', _Band(filters: 3, shown: 6, total: 82, onClear: () {}, focusClear: true)),
      b('search text on: the text alone counts as one filter', _Band(filters: 1, shown: 9, total: 82, extra: 'text “glow”', onClear: () {})),
      b('narrow 232: the count text gives way, Clear stays', _Band(filters: 3, shown: 12, total: 82, extra: 'in 1 Soft blurs', onClear: () {}), width: 232),
      b('zero results: says so', _Band(filters: 4, shown: 0, total: 82, onClear: () {})),
      b('with a note: in 2 Glows', _Band(filters: 1, shown: 9, total: 64, extra: 'in 2 Glows', onClear: () {})),
      b('by similarity', _Band(filters: 1, shown: 40, total: 40, extra: 'by similarity', onClear: () {})),
    ]),
    ('Active filters: the same filled chip as B2, and the search box that carries x', [
      _PC('active chip: rest (g15, x inside)', 96, _left(const _ActiveChip('Effect')), h: 22, pad: 8),
      _PC('active chip: hover (g20, x g95)', 96, _left(const _ActiveChip('Effect', hover: true)), h: 22, pad: 8),
      _PC('active chip: pressed (g20 + accent line 50%)', 96, _left(const _ActiveChip('Effect', pressed: true)), h: 22, pad: 8),
      _PC('active chip: keyboard focus (1 px g63 edge)', 96, _left(const _ActiveChip('Effect', focus: true)), h: 22, pad: 8),
      _PC('active chip for the search text', 110, _left(const _ActiveChip('“glow”')), h: 22, pad: 8),
      _PC('search box with text: x', 300, const _SearchPart(text: 'glow'), h: 28, pad: 8),
      _PC('search box, clear mark hovered: x g95', 300, const _SearchPart(text: 'glow', clearHover: true), h: 28, pad: 8),
      _PC('end line, more results exist', 320, const _EndLine(70, where: 'hidden by the filters'), h: 26),
      _PC('end line, nothing more: End of results', 320, const _EndLine(0, where: 'hidden by the filters'), h: 26),
    ]),
  ]);
}

// ---- B2-b Chips
Widget _pChips() {
  _PC chip(String cap, Widget c, {double w = 144}) => _PC(cap, w, _left(c), h: 20, pad: 8);
  Widget field(List<Widget> kids, {bool focused = false}) => _FieldShell(focused: focused, children: kids);
  Widget typed(String t, {double w = 90}) => SizedBox(width: w, height: 20, child: Align(alignment: Alignment.centerLeft, child: Text(t, style: T.name(N.g95))));
  Widget hint(String t, {double w = 250}) => SizedBox(width: w, height: 20, child: Align(alignment: Alignment.centerLeft, child: Text(t, style: T.name(N.g76))));
  return _Parts('B2-b', 'Chips: parts', 'A chip is 20 px: g20, key g63, value g95 semibold, a 16 x 20 cross. The field is g07 with a 1 px g20 edge (g63 focused), 4 px inside.', [
    ('Chip', [
      chip('rest', const _FilterChip('kind', 'effect')),
      chip('cross hover: g95', const _FilterChip('kind', 'effect', hoverCross: true)),
      chip('longer value', const _FilterChip('cat', 'generate')),
      chip('similar chip: long file name, clipped', const _FilterChip('similar', 'dusk_ridge_golden_hour_v2.jpg'), w: 160),
      chip('fold chip: +N more (g15)', const _MoreChip(4), w: 80),
      _PC('hint chip: rest', 72, _left(const _HintChip('kind:')), h: 22, pad: 8),
      _PC('hint chip: hover', 72, _left(const _HintChip('kind:', hover: true)), h: 22, pad: 8),
      _PC('hint chip: longest', 88, _left(const _HintChip('fav:yes')), h: 22, pad: 8),
    ]),
    ('Field, 300 wide', [
      _PC('empty, resting: hint', 300, field([hint('Search, or kind: tag: cat:')]), pad: 8),
      _PC('empty, focused', 300, field([hint('Search, or kind: tag: cat:')], focused: true), pad: 8),
      _PC('two chips, resting', 300, field(const [_FilterChip('kind', 'effect'), _FilterChip('tag', 'soft')]), pad: 8),
      _PC('two chips, focused', 300, field([const _FilterChip('kind', 'effect'), const _FilterChip('tag', 'soft'), typed('bl')], focused: true), pad: 8),
      _PC('six chips wrap (narrow 212, focused)', 212, field(const [_FilterChip('kind', 'effect'), _FilterChip('tag', 'soft'), _FilterChip('cat', 'glow'), _FilterChip('in', 'effects'), _FilterChip('tag', 'bright'), _FilterChip('fav', 'yes')], focused: true), pad: 8),
      _PC('six chips folded when not in use', 212, field(const [_FilterChip('kind', 'effect'), _FilterChip('tag', 'soft'), _MoreChip(4)]), pad: 8),
    ]),
    ('Completion list and the typo note', [
      _PC('keys: first row is the one Enter takes', 300, const _SugBox([_SugRow('kind:', first: true), _SugRow('cat:'), _SugRow('tag:')]), pad: 8),
      _PC('values, one row hover (g15)', 300, const _SugBox([_SugRow('tag:soft', first: true), _SugRow('tag:glitch', hover: true), _SugRow('tag:retro')]), pad: 8),
      _PC('unknown value: why and the valid words', 300, const _Notice('“vidoe” is not a kind. Try effect, preset, media.', fill: N.g13), pad: 8),
      _PC('"Try" label and keys', 300, Row(children: [Padding(padding: const EdgeInsets.only(right: 4), child: Text('Try', style: T.label(N.g76))), const _HintChip('kind:'), const SizedBox(width: 4), const _HintChip('cat:'), const SizedBox(width: 4), const _HintChip('tag:')]), pad: 8),
    ]),
  ]);
}

// ---- B2-c Sentence
Widget _pSentence() => _Parts('B2-c', 'Sentence heading: states', 'The heading is 13 px semibold on g13 with a row line above and below, 12 px inside. Each word is a filter; hover = g95 + underline; the tail says how many remain.', [
      ('Heading, 320', [
        _PC('no filter', 320, const _SentenceHead(words: [], count: 82, total: 82)),
        _PC('one word', 320, const _SentenceHead(words: ['Soft'], count: 22, total: 82)),
        _PC('three words, rest', 320, const _SentenceHead(words: ['Soft', 'Effects', 'Favorites'], count: 3, total: 82)),
        _PC('three words, 2nd hovered', 320, const _SentenceHead(words: ['Soft', 'Effects', 'Favorites'], count: 3, total: 82, hoverIndex: 1)),
        _PC('search text as a word', 320, const _SentenceHead(words: ['Soft', 'Presets', '“glow”'], count: 0, total: 82)),
        _PC('nothing fits', 320, const _SentenceHead(words: ['Media', 'Soft'], count: 0, total: 82)),
        _PC('narrow 232: wraps, never cut', 232, const _SentenceHead(words: ['Soft', 'Effects', 'Favorites', '“glow”'], count: 2, total: 82)),
      ]),
      ('Around it', [
        _PC('Favorites only: switch off, word to its right', 300, Row(children: [PillSwitch(on: false, onChanged: (_) {}), const SizedBox(width: 8), Text('Favorites only', style: T.label(N.g76))]), pad: 8),
        _PC('Favorites only: switch on', 300, Row(children: [PillSwitch(on: true, onChanged: (_) {}), const SizedBox(width: 8), Text('Favorites only', style: T.label(N.g76))]), pad: 8),
        _PC('kind tabs: Effect selected', 300, _Tabs(const ['Any', 'Effect', 'Preset', 'Media'], 1, (_) {}, toggle: false), h: 26, pad: 8),
        _PC('character tabs: Soft, Retro hover', 300, _Tabs(const ['Any', 'Soft', 'Glitch', 'Retro', 'Bright', 'Grade'], 1, (_) {}, toggle: false, hoverIndex: 3), h: 26, pad: 8),
      ]),
    ]);

// ---- B2-d Saved filters
Widget _pSaved() {
  _PC row(String cap, Widget w, {double width = 320}) => _PC(cap, width, w, h: 26);
  return _Parts('B2-d', 'Saved filters: places and buttons', 'A place or a saved filter is a 26 px row: tick, name, live count. On hover a saved filter swaps its count for Delete.', [
    ('Place rows', [
      row('place, rest', const _PlaceRow('Effects', 52)),
      row('place, hover', const _PlaceRow('Effects', 52, hover: true)),
      row('place, selected', const _PlaceRow('Effects', 52, on: true)),
      row('saved filter, rest: count', const _PlaceRow('Soft blurs', 9)),
      row('saved filter, hover: Delete replaces the count', _PlaceRow('Soft blurs', 9, hover: true, right: const _Word('Delete'))),
      row('saved filter, selected', const _PlaceRow('Soft blurs', 9, on: true)),
      row('long name: ellipsis, count kept', const _PlaceRow('Everything soft and bright, client A look', 14)),
      row('count 0', const _PlaceRow('Bokeh set', 0)),
      row('narrow 232, hover', _PlaceRow('Everything soft and bright', 14, hover: true, right: const _Word('Delete')), width: 232),
    ]),
    ('Headings and folding', [
      _PC('Places', 320, const _Sec('Places'), h: 24),
      _PC('Saved filters + count', 320, _Sec('Saved filters', right: Text('12', style: T.label(N.g76))), h: 24),
      _PC('fold word: rest', 240, const Padding(padding: EdgeInsets.fromLTRB(10, 4, 10, 0), child: _Word('Show 9 more saved filters')), h: 20),
      _PC('fold word: hover', 240, const Padding(padding: EdgeInsets.fromLTRB(10, 4, 10, 0), child: _Word('Show 9 more saved filters', hover: true)), h: 20),
    ]),
    ('Filter chips and buttons', [
      _PC('Category group, one on', 320, _TagGroup(label: 'Category', values: const ['Blur', 'Glow', 'Color', 'Title'], on: const {'Glow'}, onTap: (_) {})),
      _PC('Character group, hover on 3rd', 320, _TagGroup(label: 'Character', values: const ['Soft', 'Retro', 'Bright', 'Loop'], on: const {}, hoverIndex: 2, onTap: (_) {})),
      _PC('button: rest', 120, _left(_Btn('Save as place…', onTap: () {})), h: 24, pad: 8),
      _PC('button: hover', 120, _left(_Btn('Save as place…', onTap: () {}, hover: true)), h: 24, pad: 8),
      _PC('button: keyboard focus (g63 edge)', 120, _left(_Btn('Save as place…', onTap: () {}, focus: true)), h: 24, pad: 8),
      _PC('button: disabled (nothing to save)', 120, _left(const _Btn('Save as place…')), h: 24, pad: 8),
      _PC('button: primary', 120, _left(_Btn('Show everything', onTap: () {}, primary: true)), h: 24, pad: 8),
      _PC('name field: Esc key mark', 300, const _SearchPart(focus: true, slash: false, hint: 'Name this filter, Enter to save', trailing: _Kbd('Esc')), h: 28, pad: 8),
    ]),
  ]);
}

// ---- B5-a Moving tiles
Widget _pTiles() {
  final long = _by('Directional Blur (Motion Trail, 8-sample, GPU)');
  _PC tile(String cap, It it, {double w = 146, bool hover = false, double t = .55}) => _PC(cap, w, _TileCard(it, t: t, w: w, hover: hover), pad: 8);
  return _Parts('B5-a', 'Moving tiles: card and stand-in pictures', 'A tile is a 1 px g20 picture edge (g56 on hover), 6 px, the name 11 px, 4 px, then category and kind 10 px g63. Pictures are held at t = .55 here.', [
    ('Tile, two across in a 320 panel (146 wide)', [
      tile('rest', _by('Glow')),
      tile('hover: edge g56', _by('Glow'), hover: true),
      tile('long English name: ellipsis', long),
      tile('Japanese-led name', _by('夜のネオン (client A)')),
      tile('preset', _by('Typewriter')),
      tile('media-less stand-in: grain', _by('Film Grain')),
    ]),
    ('Tile, two across in a 232 dock (102 wide)', [
      tile('rest', _by('Glow'), w: 102),
      tile('hover', _by('Glow'), w: 102, hover: true),
      tile('long name', long, w: 102),
    ]),
    ('The eleven stand-in pictures at t = .55 (what each effect family looks like)', [
      for (final f in Fx.values) _PC(f.name, 102, SizedBox(width: 102, height: 63, child: ClipRRect(borderRadius: BorderRadius.circular(5), child: _Fx(f, .55))), h: 63, pad: 8),
    ]),
  ]);
}

// ---- B5-b / B6 hover and scrub
Widget _pHover() {
  final vid = _by('lens_bloom.mp4'), img = _by('dusk_ridge.jpg'), aud = _by('kick_loop.wav'), glow = _by('Glow'), tw = _by('Typewriter'), pano = _by('sphere_tour.mp4');
  _PC thumb(String cap, It it, {bool sel = false, bool hov = false, double? x, double w = 146}) => _PC(cap, w, _HoverThumb(it, w: w, h: w * .62, selected: sel, holdHover: hov, holdX: x), h: w * .62, pad: 8);
  return _Parts('B5-b', 'Hover and scrub: thumbnails', 'A thumbnail is a 1 px edge (g20 rest, g56 hover, mode selected), 6 px radius. Time pill and type badge sit 4 px in. A scrubbing line is 1 px white.', [
    ('Thumbnail states (146 x 91)', [
      thumb('still image, rest', img),
      thumb('video, rest: type badge + length', vid),
      thumb('video, hover at 0 %', vid, hov: true, x: 0),
      thumb('video, hover at 50 %: line + time', vid, hov: true, x: 73),
      thumb('video, hover at 100 %', vid, hov: true, x: 146),
      thumb('video, selected: mode edge', vid, sel: true),
      thumb('audio, rest', aud),
      thumb('360 clip, hover at 30 %', pano, hov: true, x: 44),
      thumb('preset effect, rest', glow),
      thumb('preset effect, hover: playing', glow, hov: true),
      thumb('timed preset, hover at 40 %', tw, hov: true, x: 58),
      thumb('selected still', img, sel: true),
    ]),
    ('Thumbnail in a 232 dock (102 wide)', [
      thumb('video, hover at 50 %', vid, hov: true, x: 51, w: 102),
      thumb('preset, hover', glow, hov: true, w: 102),
    ]),
    ('Badges over a picture (the 16 px rule)', [
      _PC('Playing label', 72, _left(_pill('Playing')), h: 16, pad: 8, ground: N.g26),
      _PC('Source label', 72, _left(_pill('Source', fill: N.g26)), h: 16, pad: 8),
      _PC('Similar label', 72, _left(_pill('Similar', fill: N.g20)), h: 16, pad: 8),
      _PC('type badge (media_parts)', 56, _left(const TypeBadge(Kind.video)), h: 16, pad: 8, ground: N.g26),
      _PC('time pill (media_parts)', 56, _left(const TimePill('12.0s')), h: 16, pad: 8, ground: N.g26),
    ]),
  ]);
}

Widget _pHoverRows() {
  final vid = _by('lens_bloom.mp4'), long = _by('Directional Blur (Motion Trail, 8-sample, GPU)');
  _PC row(String cap, Widget w, {double width = 320}) => _PC(cap, width, w, h: 50);
  return _Parts('B5-b / B6', 'Hover and scrub: rows, tiles, big clips', 'A row is 50 px: the thumbnail (76 wide, 60 in a dock) is the hover target; the name is what selects.', [
    ('Rows', [
      row('rest', _HoverRow(vid, tw: 76)),
      row('thumb hover at 40 %', _HoverRow(vid, tw: 76, holdHover: true, holdX: 30)),
      row('selected', _HoverRow(vid, tw: 76, selected: true)),
      row('long name: ellipsis', _HoverRow(long, tw: 76)),
      row('preset', _HoverRow(_by('Glow'), tw: 76)),
      row('narrow 232, hover', _HoverRow(vid, tw: 60, holdHover: true, holdX: 30), width: 232),
    ]),
    ('Tile captions (146)', [
      _PC('tile: rest', 146, _HoverTile(vid, w: 146), pad: 8),
      _PC('tile: selected: name g95', 146, _HoverTile(vid, w: 146, selected: true), pad: 8),
      _PC('tile: preset', 146, _HoverTile(_by('Glow'), w: 146), pad: 8),
      _PC('tile: long name', 146, _HoverTile(long, w: 146), pad: 8),
    ]),
    ('B6 big clip (300 x 150) and its caption line', [
      _PC('poster frame at rest', 300, _HoverThumb(vid, w: 300, h: 150), h: 150, pad: 8),
      _PC('scrub at 40 %', 300, _HoverThumb(vid, w: 300, h: 150, holdHover: true, holdX: 120), h: 150, pad: 8),
      _PC('caption: name and size (the length is on the picture)', 300, Row(children: [Expanded(child: Text(vid.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name())), Text('1920×1080', style: _num(N.g76))]), pad: 8),
    ]),
  ]);
}

// ---- B5-c Own frame
Widget _pOwn() {
  Widget frame(String label, {bool pv = false, String? applied, bool noTarget = false, double w = 300}) => _OwnFrameView(w: w, h: w * 9 / 16, label: label, previewing: pv, applied: applied, noTarget: noTarget, picture: _Fx(pv ? Fx.glow : (applied != null ? Fx.glow : Fx.none), .5, base: _ownBase(sample: noTarget)));
  return _Parts('B5-c', 'Own frame: overlays, banner, counters', 'The frame has a 1 px edge: g20, the mode colour only while previewing. Labels are 16 px, 6 px in; the right one gives way before it touches the left.', [
    ('Frame, 300 x 169', [
      _PC('nothing tried yet', 300, frame('Layer 3 · Title'), pad: 8),
      _PC('previewing: mode edge + PREVIEW label', 300, frame('Layer 3 · Title', pv: true), pad: 8),
      _PC('applied: APPLIED label + name', 300, frame('Layer 3 · Title', applied: 'Glow'), pad: 8),
      _PC('no selection: sample frame', 300, frame('Sample frame (no selection)', noTarget: true), pad: 8),
      _PC('narrow 212: labels ellipsize', 212, frame('Layer 3 · Title', pv: true, w: 212), pad: 8),
      _PC('applied, long name, 212', 212, frame('Layer 3 · Title', applied: 'Directional Blur (Motion Trail, 8-sample, GPU)', w: 212), pad: 8),
    ]),
    ('Phase banner, 300 x 28', [
      _PC('PREVIEW: dashed g56 edge', 300, const _PhaseBanner(_Phase.preview), h: 28, pad: 8),
      _PC('APPLIED: g20 ground + tick', 300, const _PhaseBanner(_Phase.applied), h: 28, pad: 8),
      _PC('DROPPED: g63 words', 300, const _PhaseBanner(_Phase.dropped), h: 28, pad: 8),
      _PC('PREVIEW, narrow 212', 212, const _PhaseBanner(_Phase.preview), h: 28, pad: 8),
    ]),
    ('Counters', [
      _PC('nothing yet', 300, const _Counters(0, 0, 'Down'), pad: 8),
      _PC('tried 3, hover trigger', 300, const _Counters(3, 0, ''), pad: 8),
      _PC('previewing: Enter and Esc key marks', 300, const _Counters(3, 1, null), pad: 8),
      _PC('written > 0: g95', 300, const _Counters(5, 2, 'Down'), pad: 8),
    ]),
  ]);
}

// ---- B5-d Preview surface
Widget _pSurface() {
  Widget surf(String name, {bool sample = false, bool playing = true, bool hide = false, double w = 300}) => _SurfaceFrame(w: w, h: w * 9 / 16, sample: sample, playing: playing, name: name, hoverHide: hide ? true : null, picture: _Fx(Fx.glow, .5, base: _ownBase(sample: sample)));
  return _Parts('B5-d', 'Preview surface: overlays and closed bar', 'The surface is 16:9 with a 1 px g20 edge. Top: whose frame, and Hide. Bottom: Playing or Paused, and the item name.', [
    ('Surface, 300 x 169', [
      _PC('your frame, playing', 300, surf('Glow'), pad: 8),
      _PC('your frame, paused', 300, surf('Glow', playing: false), pad: 8),
      _PC('sample frame', 300, surf('Glow', sample: true), pad: 8),
      _PC('Hide hovered: g26', 300, surf('Glow', hide: true), pad: 8),
      _PC('long item name', 300, surf('Directional Blur (Motion Trail, 8-sample, GPU)'), pad: 8),
      _PC('narrow 212, long name', 212, surf('Directional Blur (Motion Trail, 8-sample, GPU)', w: 212), pad: 8),
    ]),
    ('Closed bar, position line', [
      _PC('closed bar: rest', 300, const _ClosedBar(), h: 26, pad: 8),
      _PC('closed bar: Show hover', 300, const _ClosedBar(hoverShow: true), h: 26, pad: 8),
      _PC('closed bar, narrow 212: line clips, Show stays', 212, const _ClosedBar(), h: 26, pad: 8),
      _PC('position line', 300, Text('15 of 64', style: _num(N.g76)), pad: 8),
    ]),
  ]);
}

// ---- B10 star, used, collections
Widget _pStar() {
  final g = _by('Glow'), t = _by('Title In: Soft Rise');
  _PC row(String cap, Widget w, {double width = 320, double h = 34}) => _PC(cap, width, w, h: h);
  return _Parts('B10-b', 'Star and recent: rows and headings', 'The star is a 24 px target at the right end; the quiet word before it (when, how often) is 10 px g63. Star hover is the live use case only (its own part owns it).', [
    ('Rows', [
      row('starred', _ItemRow(g, right: _StarTail(null, on: true))),
      row('not starred', _ItemRow(g, right: _StarTail(null, on: false))),
      row('recent: time + star', _ItemRow(t, right: _StarTail(_ago(40), on: false))),
      row('recent: just now', _ItemRow(g, right: _StarTail('just now', on: true))),
      row('frequent: count', _ItemRow(g, right: _StarTail('41×', on: true))),
      row('selected', _ItemRow(t, selected: true, right: _StarTail(_ago(40), on: true))),
      row('hover', _ItemRow(t, hover: true, right: _StarTail(_ago(40), on: false))),
      row('long name, narrow 232', _ItemRow(_by('Directional Blur (Motion Trail, 8-sample, GPU)'), right: _StarTail('6×', on: false)), width: 232),
    ]),
    ('Headings and first-run hints', [
      row('Starred + count', _Sec('Starred', right: Text('4', style: T.label(N.g76))), h: 26),
      row('Recent + automatic', _Sec('Recent', right: Text('automatic', style: T.label(N.g76))), h: 26),
      row('first run: Starred', const _HintRow('No stars yet'), h: 36),
      row('first run: Recent', const _HintRow('None yet'), h: 36),
      row('first run: Frequent', const _HintRow('None yet'), h: 36),
    ]),
  ]);
}

Widget _pUsed() {
  final g = _by('Glow'), long = _by('Directional Blur (Motion Trail, 8-sample, GPU)');
  _PC row(String cap, Widget w, {double width = 320}) => _PC(cap, width, w, h: 30);
  return _Parts('B10-c', 'Used stack: rows and heading', 'A shelf row is 30 px: thumbnail 26 x 18, name, and x count when the same thing was folded in. A row just used glows g26 and fades in 600 ms.', [
    ('Rows', [
      row('single use', _UsedRow(g, 1)),
      row('folded: x3', _UsedRow(g, 3)),
      row('just used: g26 at full', _UsedRow(g, 2, flash: 1)),
      row('fading: halfway', _UsedRow(g, 2, flash: .5)),
      row('long name, count kept', _UsedRow(long, 12)),
      row('narrow 232', _UsedRow(long, 2), width: 232),
    ]),
    ('Heading and empty', [
      _PC('Used: 6 of 12', 320, _Sec('Used', right: Text('6 of 12', style: T.label(N.g76))), h: 24),
      _PC('Used: 12 of 12 (at the cap)', 320, _Sec('Used', right: Text('12 of 12', style: T.label(N.g76))), h: 24),
      _PC('empty shelf', 320, const _HintRow('Nothing used yet'), h: 40),
      _PC('status: folded', 320, const _Status('“Glow” ×4'), h: 26),
      _PC('status: full', 320, const _Status('Full: dropped “Typewriter”'), h: 26),
    ]),
  ]);
}

Widget _pCollections() {
  final g = _by('Glow');
  _PC slot(String cap, Widget w, {double width = 158}) => _PC(cap, width, w, h: 24);
  return _Parts('B10-a', 'Collections: slots and membership', 'A slot is 24 px: the tick in the gutter of its cell, then (x 10) its number as a key mark, its name, how many it holds. Two across in a 320 panel (158 each), one in a 232 dock.', [
    ('Slots, 158 wide', [
      slot('rest', const _SlotRow(1, 'Titles', 3)),
      slot('hover: g15', const _SlotRow(1, 'Titles', 3, hover: true)),
      slot('selected: g20, tick, lit key', const _SlotRow(1, 'Titles', 3, on: true)),
      slot('empty slot: dashed outline', const _SlotRow(6, '', 0)),
      slot('long name: ellipsis', const _SlotRow(5, 'Soft blurs for client A', 12)),
      slot('count 0', const _SlotRow(4, 'Retro', 0)),
      slot('count 41', const _SlotRow(2, 'Glows', 41)),
    ]),
    ('Slots, 232 wide (dock)', [
      slot('rest', const _SlotRow(3, 'Client A', 2), width: 232),
      slot('selected', const _SlotRow(3, 'Client A', 2, on: true), width: 232),
      slot('empty', const _SlotRow(7, '', 0), width: 232),
    ]),
    ('Heading, band, membership column', [
      _PC('Collections', 320, const _Sec('Collections'), h: 24),
      _PC('band: count only (the slot above is lit)', 320, _Band(filters: 1, shown: 4, total: 64, showFilters: false, onClear: () {}), h: 28),
      _PC('row: also in 1 more (number plate as on the slot)', 320, _ItemRow(g, right: const _SlotBadges([2, 3])), h: 34),
      _PC('row: also in 3', 320, _ItemRow(g, right: const _SlotBadges([2, 3, 5])), h: 34),
      _PC('row: more than 3', 320, _ItemRow(g, right: const _SlotBadges([1, 2, 3, 5])), h: 34),
      _PC('empty collection', 320, _Empty('Retro is empty', const [], actions: [_Btn('Show everything', onTap: () {})]), h: 170),
    ]),
  ]);
}

// ---- B12-b Similar
Widget _pSimilar() {
  final a = sampleAssets[0], v = sampleAssets[1];
  _PC tile(String cap, Widget t, {double w = 96}) => _PC(cap, w, t, pad: 8);
  return _Parts('B12-b', 'Similar: tile, chip, band', 'A tile is a 1 px picture edge (g20, g56 hover, mode selected), the name, and a 2 px closeness bar with its number in mono 10 px.', [
    ('Tile, three across in a 320 panel (96 wide)', [
      tile('all view: no bar', _SimTile(a, w: 96)),
      tile('rest with closeness', _SimTile(v, w: 96, score: .82)),
      tile('hover: Similar label', _SimTile(v, w: 96, score: .82, hover: true)),
      tile('hover, label hovered (g26)', _SimTile(v, w: 96, score: .82, hover: true, simHover: true)),
      tile('selected: mode edge', _SimTile(v, w: 96, score: .82, selected: true)),
      tile('source: first, 1.00', _SimTile(a, w: 96, score: 1, selected: true, source: true)),
      tile('low closeness', _SimTile(v, w: 96, score: .12)),
    ]),
    ('Tile, two across in a 232 dock (103 wide)', [
      tile('rest', _SimTile(v, w: 103, score: .64), w: 103),
      tile('hover', _SimTile(v, w: 103, score: .64, hover: true), w: 103),
    ]),
    ('Chip, band, tabs', [
      _PC('filter chip', 200, _left(const _FilterChip('similar', 'dusk_ridge.jpg')), h: 20, pad: 8),
      _PC('filter chip: cross hover', 200, _left(const _FilterChip('similar', 'dusk_ridge.jpg', hoverCross: true)), h: 20, pad: 8),
      _PC('filter chip: long name, 212', 212, _left(const _FilterChip('similar', 'dusk_ridge_golden_hour_v2_final.jpg')), h: 20, pad: 8),
      _PC('band: by similarity (the chip above says what from)', 320, _Band(filters: 1, shown: 38, total: 38, showFilters: false, showCount: false, extra: 'by similarity', onClear: () {}), h: 28),
      _PC('type tabs', 300, _Tabs(const ['All', 'Image', 'Video', 'Audio', '3D'], 0, (_) {}, toggle: false), h: 26, pad: 8),
      _PC('status', 320, const _Status('', keys: ['Y', 'Esc']), h: 26),
    ]),
  ]);
}

// ================================================================================ the sets

String _s(BuildContext c, String label, List<String> opts, String init) => c.knobs.object.dropdown<String>(label: label, options: opts, initialOption: init, labelBuilder: (s) => s.isEmpty ? '(empty)' : s);

/// One component per problem id, in the order to release them (simplest and most useful first). Every component opens with its Parts @200% sheet(s), then the whole-panel drafts.
List<WidgetbookComponent> panelBrowserASets() => [
      WidgetbookComponent(name: 'Browser B3 Find', useCases: [
        _parts('Find chrome', _pFindChrome),
        _parts('Find rows', _pRows),
        _uc('B3-a Find', 'unified query', Hab.habit, 'Field is always visible / each letter updates the list / several words = AND / Esc clears the text, a second Esc leaves the field.', (c, w) {
                final q = _s(c, 'Query', const ['', 'bl', 'glow soft', 'グロー', 'dir', 'zzz'], 'bl');
                return _FindA(key: ValueKey('a1$q$w'), query: q);
              }),
        _uc('B3-a Find', 'tab on', Hab.habit, 'One place chosen: its tab is the g20 pill with g95 text and the accent line / the count line names the place / results that exist elsewhere are cued in the footer / the tab again clears it.', (c, w) {
                final tab = _s(c, 'Place', const [..._places], 'Effects');
                return _FindA(key: ValueKey('a5$tab$w'), query: 'bl', tab: _places.indexOf(tab));
              }),
        _uc('B3-a Find', 'clear mark hovered', Hab.habit, 'With text in the field the x is the clear mark: g63 at rest, g95 under the pointer (pinned here); one click empties the field and keeps focus.', (c, w) => _FindA(key: ValueKey('a6$w'), query: 'bl', clearHover: true)),
        _uc('B3-a Find', 'more results exist', Hab.habit, 'With a place chosen, results in the other places are not hidden silently: the footer says how many more there are.', (c, w) => _FindA(key: ValueKey('a7$w'), query: 'soft', tab: 1)),
        _uc('B3-a Find', 'across places (D-4)', Hab.habit, 'One search spans all places and each result carries its place / choosing a place narrows it / the typed text stays when the place changes.', (c, w) {
                final tab = _s(c, 'Place', const ['All', ..._places], 'All');
                return _FindA(key: ValueKey('a2$tab$w'), query: 'soft', grouped: true, tab: tab == 'All' ? -1 : _places.indexOf(tab));
              }),
        _uc('B3-a Find', 'narrow dock, long names', Hab.habit, 'Same contract in a 232 px dock: names ellipsize, nothing is clipped mid-word, the field never shrinks.', (c, w) => _FindA(key: ValueKey('a3$w'), query: 'blur'), narrow: true),
        _uc('B3-a Find', 'no match, next step', Hab.habit, 'An empty result says what was searched and offers a way out: clear, widen to all places, or a spelling suggestion.', (c, w) {
                final q = _s(c, 'Query', const ['glwo', 'zzzz', 'blu'], 'glwo');
                final tab = _s(c, 'Place', const ['All', ..._places], 'Presets');
                return _FindA(key: ValueKey('a4$q$tab$w'), query: q, tab: tab == 'All' ? -1 : _places.indexOf(tab));
              }),
      ]),
      WidgetbookComponent(name: 'Browser B2 Tag bands', useCases: [
        _parts('Chips and groups', _pTags),
        _uc('B2-a Tag bands', 'Ableton type', Hab.habit, 'Groups cross as AND / the band counts the filters on / Clear removes all. The click rule is said once, in the status line under the panel.', (c, w) {
                final add = c.knobs.boolean(label: 'Add mode (as if Cmd is held)', initialValue: false);
                return _TagBands(key: ValueKey('e1$add$w'), add: add, start: const {'Kind': ['Effect'], 'Character': ['Soft']});
              }),
        _uc('B2-a Tag bands', 'groups fold (narrow dock)', Hab.habit, 'A group with nothing chosen folds to one line and a group with a choice shows it even when folded, so a filter is never hidden.', (c, w) => _TagBands(key: ValueKey('e2$w'), fold: true, start: const {'Category': ['Blur'], 'Character': ['Soft']}), narrow: true),
        _uc('B2-a Tag bands', 'zero results', Hab.habit, 'Narrowing never widens, so zero is reachable: the panel names what to remove and how many that returns, one click each.', (c, w) => _TagBands(key: ValueKey('e3$w'), start: const {'Kind': ['Media'], 'Category': ['Blur'], 'Character': ['Soft']})),
      ]),
      WidgetbookComponent(name: 'Browser B4 Results band', useCases: [
        _parts('Band states', _pBand),
        _uc('B4 Results band', 'n filters and Clear', Hab.addition, 'The band counts every filter that is on, search text included / zero says 0 and the way out / Clear is one click and dim when nothing is on.', (c, w) {
                final n = c.knobs.int.slider(label: 'Filters on', initialValue: 3, min: 0, max: 4);
                final t = _s(c, 'Search text', const ['', 'glow', 'blur', 'zzz'], '');
                return _BandDemo(key: ValueKey('f1$n$t$w'), n: n, text: t);
              }),
        _uc('B4 Results band', 'Clear and x hovered', Hab.addition, 'Clear is a button: g76 word, g26 hairline, g20 fill under the pointer (pinned here) / the search box x brightens to g95 / the footer cues the results the filters hide.', (c, w) => _BandDemo(key: ValueKey('f2$w'), n: 3, text: 'glow', hoverClear: true, hoverX: true)),
        _uc('B4 Results band', 'no filter, Clear dimmed', Hab.addition, 'With nothing on, Clear is dimmed (g63, no hover, arrow cursor) and the footer says end of results.', (c, w) => _BandDemo(key: ValueKey('f3$w'), n: 0, text: '', hoverClear: true)),
      ]),
      WidgetbookComponent(name: 'Browser B3 Tab menu', useCases: [
        _parts('Menu pieces', _pMenu),
        _uc('B3-b Tab menu', 'type-ahead at the cursor', Hab.addition, 'Tab opens a small search where the pointer is / Enter applies the first (or selected) to the selection / Esc writes nothing / Tab again cycles the family.', (c, w) {
                final t = _s(c, 'Target', const ['Layer', 'None', 'Key'], 'Layer');
                return _TabMenu(key: ValueKey('b1$t$w'), w: w, target: const ['Layer', 'None', 'Key'].indexOf(t), text: 'gl');
              }),
        _uc('B3-b Tab menu', 'empty input = last used first', Hab.addition, 'With nothing typed, the most recently used effect is on top, so Tab then Enter repeats it / Tab again moves to the next family.', (c, w) => _TabMenu(key: ValueKey('b2$w'), w: w)),
        _uc('B3-b Tab menu', 'no target selected', Hab.addition, 'Without a layer or key selected, Enter writes nothing and the menu says why in a line, not by going dim.', (c, w) => _TabMenu(key: ValueKey('b3$w'), w: w, target: 1, text: 'blur')),
        _uc('B3-b Tab menu', 'key selected: only curves', Hab.addition, 'The candidates follow the selection: with a key selected the list holds only what a key can take (ease presets, curves).', (c, w) => _TabMenu(key: ValueKey('b4$w'), w: w, target: 2)),
      ]),
      WidgetbookComponent(name: 'Browser B3 Prefix', useCases: [
        _parts('Rows, area line, marks', _pPrefix),
        _uc('B3-c Prefix', 'marks pick the area', Hab.addition, 'The first mark changes the area and its heading / no mark = every area under its own heading / the marks are listed under the field.', (c, w) {
                final q = _s(c, 'Query', const ['', 'lo', '#so', '>re', '@ti', '#', '@zz'], 'lo');
                return _Prefix(key: ValueKey('c1$q$w'), query: q);
              }),
        _uc('B3-c Prefix', 'discovery: empty field', Hab.addition, 'An empty field teaches the three marks (tap one to try it); the plain list stays below, so a newcomer who never types a mark loses nothing.', (c, w) => _Prefix(key: ValueKey('c2$w'))),
      ]),
      WidgetbookComponent(name: 'Browser B3 Empty query', useCases: [
        _parts('Headings, hints, reasons', _pFreq),
        _uc('B3-d Empty query', 'recent, then what fits', Hab.addition, 'Empty field = Recent, then what fits the selection, then the rest / each row says why it is there / one letter turns it into normal search.', (c, w) => _Freq(key: ValueKey('d1$w'), sel: 1, history: true)),
        _uc('B3-d Empty query', 'nothing selected', Hab.addition, 'With no selection the fit section does not guess: it says to select something, and Recent still works.', (c, w) => _Freq(key: ValueKey('d2$w'), sel: 0, history: true)),
        _uc('B3-d Empty query', 'a key selected', Hab.addition, 'The fit section follows the selection kind: for a key it lists curves and ease presets, not layer effects.', (c, w) => _Freq(key: ValueKey('d3$w'), sel: 2, history: true)),
        _uc('B3-d Empty query', 'first run, no history', Hab.addition, 'No history yet: Recent explains itself in a line instead of an empty box; Fit and the rest still answer.', (c, w) => _Freq(key: ValueKey('d4$w'), sel: 1, history: false)),
      ]),
      WidgetbookComponent(name: 'Browser B2 Chips', useCases: [
        _parts('Chips and field', _pChips),
        _uc('B2-b Chips', 'sentences become chips', Hab.addition, 'A filter typed as kind:effect becomes a chip you can see and remove / chip count = filter count / removing all chips returns the plain list.', (c, w) => _ChipField(key: ValueKey('g1$w'), chips: 2)),
        _uc('B2-b Chips', 'first use: the keys are offered', Hab.addition, 'An empty field offers the keys as tappable hints, and typing a key completes its values: the syntax is learnable without a manual.', (c, w) => _ChipField(key: ValueKey('g2$w')), narrow: false),
        _uc('B2-b Chips', 'many chips, narrow dock', Hab.addition, 'Chips wrap instead of scrolling away; when the field is not in use, extra chips fold to +N and the count in the band stays true.', (c, w) => _ChipField(key: ValueKey('g3$w'), chips: 6), narrow: true),
        _uc('B2-b Chips', 'a typo gets a reason', Hab.addition, 'An unknown value does not silently match nothing: the field says what it is not and lists the valid words.', (c, w) => _ChipField(key: ValueKey('g4$w'), chips: 1, text: 'kind:vidoe')),
      ]),
      WidgetbookComponent(name: 'Browser B2 Sentence', useCases: [
        _parts('Heading states', _pSentence),
        _uc('B2-c Sentence heading', 'the filter, said in words', Hab.departs, 'The heading names every filter in one sentence / clicking a word drops only that filter / with no filter it says All N.', (c, w) => _Sentence(key: ValueKey('h1$w'), kind: 1, chr: 1, fav: true)),
        _uc('B2-c Sentence heading', 'nothing fits', Hab.departs, 'When the sentence matches nothing it says so, and each word offers its own count if dropped.', (c, w) => _Sentence(key: ValueKey('h2$w'), kind: 3, chr: 1)),
        _uc('B2-c Sentence heading', 'wraps in a narrow dock', Hab.departs, 'The sentence wraps onto lines instead of truncating, because a cut-off filter is a hidden filter.', (c, w) => _Sentence(key: ValueKey('h3$w'), kind: 1, chr: 3, fav: true, text: 'glow'), narrow: true),
      ]),
      WidgetbookComponent(name: 'Browser B2 Saved filters', useCases: [
        _parts('Places and buttons', _pSaved),
        _uc('B2-d Saved filters', 'a filter becomes a place', Hab.addition, 'A saved filter is listed as a place with its live count / new matching items join by themselves / deleting it never deletes the items.', (c, w) => _Saved(key: ValueKey('i1$w'))),
        _uc('B2-d Saved filters', 'many saved, folded', Hab.addition, 'Past four saved filters the list folds to three plus Show N more, so places stay a short list however many you keep.', (c, w) => _Saved(key: ValueKey('i2$w'), extra: 8), narrow: true),
      ]),
      WidgetbookComponent(name: 'Browser B10 Collections', useCases: [
        _parts('Slots and membership', _pCollections),
        _uc('B10-a Collections', 'number keys 1 to 9', Hab.habit, '1 to 9 add the selected row to that collection, 0 removes it from all / a collection is one click and shows as a filter in the band / colours are not used.', (c, w) => _Collections(key: ValueKey('q1$w'), w: w)),
      ]),
      WidgetbookComponent(name: 'Browser B10 Star and recent', useCases: [
        _parts('Star rows and headings', _pStar),
        _uc('B10-b Star and auto recent', 'F stars, the rest fills itself', Hab.habit, 'F stars the selected row / Recent and Frequent come from use with nothing to maintain / a double-click or Enter counts as a use.', (c, w) => _StarRecent(key: ValueKey('o1$w'))),
        _uc('B10-b Star and auto recent', 'first run', Hab.habit, 'On first run each automatic section explains in a line when it fills, so empty never looks broken.', (c, w) => _StarRecent(key: ValueKey('o2$w'), empty: true)),
      ]),
      WidgetbookComponent(name: 'Browser B10 Used stack', useCases: [
        _parts('Used rows', _pUsed),
        _uc('B10-c Used stack', 'newest on top, no tidying', Hab.addition, 'Using something puts it first / the same thing folds into one row with a count / past 12 the oldest drops off.', (c, w) {
                final n = c.knobs.int.slider(label: 'Shelf size', initialValue: 6, min: 0, max: 12);
                return _UsedStack(key: ValueKey('p1$n$w'), shelf: n);
              }),
        _uc('B10-c Used stack', 'shelf at its cap', Hab.addition, 'At 12 a new use pushes the oldest off and says which one, so nothing disappears silently.', (c, w) => _UsedStack(key: ValueKey('p2$w'), shelf: 12), narrow: true),
      ]),
      WidgetbookComponent(name: 'Browser B12 Similar', useCases: [
        _parts('Tile, chip, band', _pSimilar),
        _uc('B12-b Similar', 'more like this', Hab.addition, 'Y or the Similar button re-sorts the same list by closeness and shows it as one removable filter / the source stays first / Clear returns all.', (c, w) => _Similar(key: ValueKey('r1$w'), w: w)),
      ]),
      WidgetbookComponent(name: 'Browser B5 Moving tiles', useCases: [
        _parts('Tile and stand-ins', _pTiles),
        _uc('B5-a Moving tiles', 'every tile screens itself', Hab.departs, 'A tile moves only while visible / every tile has the same period / reduce-motion holds one still frame.', (c, w) {
                final n = c.knobs.int.slider(label: 'Tiles', initialValue: 24, min: 4, max: 200);
                final r = c.knobs.boolean(label: 'Reduce motion', initialValue: false);
                return _MovingTiles(key: ValueKey('j1$n$r$w'), count: n, reduce: r, w: w);
              }),
        _uc('B5-a Moving tiles', '200 tiles, reduce motion', Hab.departs, 'With 200 tiles and reduce-motion on, nothing moves and the panel still shows what each does; the cost is one frame, not 200 clocks.', (c, w) => _MovingTiles(key: ValueKey('j2$w'), count: 200, reduce: true, w: w), narrow: true),
      ]),
      WidgetbookComponent(name: 'Browser B5 Hover and scrub', useCases: [
        _parts('Thumbnail states', _pHover),
        _parts('Rows, tiles, big clips', _pHoverRows),
        _uc('B5-b Hover plays', 'tiles', Hab.habit, 'Hover plays that tile only / leaving returns to its first picture / looking never writes, so skimming the shelf leaves no history.', (c, w) => _HoverTiles(key: ValueKey('k1$w'), w: w)),
        _uc('B5-b Hover plays', 'rows in a narrow dock', Hab.habit, 'The same contract in rows: the thumbnail is the hover target, x on it scrubs timed items, the name still selects.', (c, w) => _HoverTiles(key: ValueKey('k2$w'), w: w, rows: true), narrow: true),
        _uc('B6 Scrub', 'long clips and presets', Hab.habit, 'x position over a tile is the time / a time pill and a line show where you are / leaving returns to the poster frame.', (c, w) => _ScrubClips(key: ValueKey('l1$w'), w: w)),
      ]),
      WidgetbookComponent(name: 'Browser B5 Own frame', useCases: [
        _parts('Frame, banner, counters', _pOwn),
        _uc('B5-c Own frame', 'peek with the arrows', Hab.addition, 'Down or Up shows the next candidate on your own frame / nothing is written until Enter / Esc, leaving the panel or Cmd-Z puts it back.', (c, w) {
                final hov = _s(c, 'Trigger', const ['Arrows (D-1)', 'Hover'], 'Arrows (D-1)') == 'Hover';
                return _OwnFrame(key: ValueKey('m1$hov$w'), w: w, hover: hov, noTarget: false);
              }),
        _uc('B5-c Own frame', 'no selection: sample frame', Hab.addition, 'With nothing selected the frame is a sample and says so (D-9) / Enter writes nothing because there is no target.', (c, w) => _OwnFrame(key: ValueKey('m2$w'), w: w, hover: false, noTarget: true)),
        _uc('B5-c Own frame', 'previewing, applied, dropped', Hab.addition, 'The three states differ by word and grey step: dashed PREVIEW = not applied, solid APPLIED = one write, quiet DROPPED = back to the original.', (c, w) {
                final n = _s(c, 'Effect', const ['Glow', 'Gaussian Blur', 'Glitch RGB Split', 'Film Grain', 'Wave Warp'], 'Glow');
                return _Phases(w: w, fx: kItems.firstWhere((i) => i.name == n));
              }),
      ]),
      WidgetbookComponent(name: 'Browser B5 Preview surface', useCases: [
        _parts('Overlays and closed bar', _pSurface),
        _uc('B5-d Preview surface', 'a fixed big surface', Hab.habit, 'The surface follows the selection with no press / Space plays and pauses / Hide gives its height back and leaves nothing behind.', (c, w) {
                final s = c.knobs.boolean(label: 'Sample frame instead of yours', initialValue: false);
                return _BigPreview(key: ValueKey('n1$s$w'), w: w, sample: s);
              }),
        _uc('B5-d Preview surface', 'hidden', Hab.habit, 'With the surface hidden the list takes the whole height and a one-line bar offers it back.', (c, w) => _BigPreview(key: ValueKey('n2$w'), w: w, open: false)),
        _uc('B5-d Preview surface', 'narrow dock', Hab.habit, 'In a 232 px dock the surface scales with the width (16:9) and the list keeps at least ten rows.', (c, w) => _BigPreview(key: ValueKey('n3$w'), w: w), narrow: true),
      ]),
    ];
