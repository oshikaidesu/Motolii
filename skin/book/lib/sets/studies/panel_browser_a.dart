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
part 'panel_browser_a_2.dart';
part 'panel_browser_a_3.dart';
part 'panel_browser_a_4.dart';
part 'panel_browser_a_5.dart';
part 'panel_browser_a_6.dart';
part 'panel_browser_a_7.dart';
part 'panel_browser_a_8.dart';
part 'panel_browser_a_9.dart';
part 'panel_browser_a_10.dart';
part 'panel_browser_a_11.dart';
part 'panel_browser_a_12.dart';

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
