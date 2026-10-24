// Part of panel_browser_b.dart (cut there by size, as it was).
part of 'panel_browser_b.dart';

class _Ghost extends StatelessWidget {
  const _Ghost(this.text, {this.thumb});
  final String text;
  final Widget? thumb;
  @override
  Widget build(BuildContext context) => Container(
        height: 26,
        padding: const EdgeInsets.fromLTRB(3, 3, 8, 3),
        decoration: BoxDecoration(color: N.g20, borderRadius: BorderRadius.circular(5), border: Border.all(color: N.g44), boxShadow: [BoxShadow(color: N.g00.withValues(alpha: .5), blurRadius: 16, offset: const Offset(0, 6))]),
        child: Row(mainAxisSize: MainAxisSize.min, children: [
          if (thumb != null) ...[SizedBox(width: 26, height: 20, child: ClipRRect(borderRadius: BorderRadius.circular(3), child: thumb)), const SizedBox(width: 6)],
          Text(text, style: T.name(N.g95)),
        ]),
      );
}

/// What just happened (up to two lines). The keys that work here are a hover tooltip only (never text in the panel). Without a message or an action it takes no room.
class _Foot extends StatelessWidget {
  const _Foot({this.msg, this.hints = const [], this.trailing});
  final String? msg;
  final Widget? trailing;
  final List<(String, String)> hints;
  @override
  Widget build(BuildContext context) {
    if (msg == null && trailing == null) return const SizedBox.shrink();
    final bar = Container(
      padding: const EdgeInsets.fromLTRB(_px, 8, _px, 8),
      decoration: const BoxDecoration(color: N.g13, border: Border(top: BorderSide(color: N.g20))),
      child: Row(children: [if (msg != null && (msg!.startsWith('Added') || msg!.startsWith('Saved'))) const OkMark(size: 10, gap: 6), Expanded(child: Text(msg ?? '', maxLines: 2, overflow: TextOverflow.ellipsis, style: T.label(N.g91).copyWith(height: 1.25))), ...?(trailing == null ? null : [const SizedBox(width: 8), trailing!])]),
    );
    return hints.isEmpty ? bar : Tooltip(message: hints.map((h) => '${h.$1} ${h.$2}').join('  ·  '), waitDuration: const Duration(milliseconds: 500), child: bar);
  }
}

/// A read-only parameter row, grey only (a Fam hue would claim a relation).
class _Param extends StatelessWidget {
  const _Param(this.label, this.v, {this.hot = false, this.grip = false, this.changed = false});
  final String label;
  final double v;
  final bool hot, grip, changed;
  @override
  Widget build(BuildContext context) => Container(
        height: 20,
        decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(4), border: Border.all(color: N.g20)),
        child: Stack(children: [
          Positioned.fill(child: Align(alignment: Alignment.centerLeft, child: FractionallySizedBox(widthFactor: v.clamp(0.0, 1.0), heightFactor: 1, child: DecoratedBox(decoration: BoxDecoration(color: hot ? N.g38 : N.g26, borderRadius: BorderRadius.circular(3)))))),
          if (grip) Positioned.fill(child: Align(alignment: Alignment.centerLeft, child: FractionallySizedBox(widthFactor: v.clamp(0.0, 1.0), heightFactor: 1, child: Align(alignment: Alignment.centerRight, child: Padding(padding: const EdgeInsets.only(right: 3), child: SizedBox(width: 5, height: 8, child: CustomPaint(painter: _GripP(hot ? N.g95 : N.g76)))))))),
          Positioned.fill(child: Padding(padding: const EdgeInsets.symmetric(horizontal: 8), child: Row(children: [
            Expanded(child: Text(label, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g91).copyWith(decoration: changed ? TextDecoration.underline : null, decorationColor: N.g76, decorationThickness: 1))),
            Text((v * 40).toStringAsFixed(1), style: _num(N.g95)),
          ]))),
        ]),
      );
}

/// Two 1 px grip lines, 5 x 8: the mark of a field that is dragged.
class _GripP extends CustomPainter {
  const _GripP(this.c);
  final Color c;
  @override
  void paint(Canvas canvas, Size size) {
    final p = Paint()..color = c..strokeWidth = 1;
    canvas.drawLine(const Offset(1, 0), Offset(1, size.height), p);
    canvas.drawLine(Offset(size.width - 1, 0), Offset(size.width - 1, size.height), p);
  }

  @override
  bool shouldRepaint(_GripP o) => o.c != c;
}

class _Tabs extends StatelessWidget {
  const _Tabs(this.i, {this.onChanged});
  final int i;
  final ValueChanged<int>? onChanged;
  @override
  Widget build(BuildContext context) => LayoutBuilder(builder: (c, box) => Segmented(items: box.maxWidth < 250 ? const ['Obj', 'Rel', 'Fx', 'Media'] : _tabs, index: i, expand: true, onChanged: onChanged));
}

/// The undo count: nothing at 0, otherwise the undo glyph and the number (hover says what it is).
Widget _undoMark(int n) => n <= 0
    ? const SizedBox.shrink()
    : Tooltip(message: 'Undo', waitDuration: const Duration(milliseconds: 500), child: Row(mainAxisSize: MainAxisSize.min, children: [const tp.Glyph(tp.G.undo, size: 12, color: N.g76), const SizedBox(width: 3), Text('$n', style: _num())]));

Widget _t(String s, TextStyle st, {int lines = 1}) => Text(s, maxLines: lines, overflow: TextOverflow.ellipsis, style: st);

double _tw(String s, TextStyle st) => (TextPainter(text: TextSpan(text: s, style: st), maxLines: 1, textDirection: TextDirection.ltr)..layout()).width;

/// The one truncation rule for every tile/row name in this file: a tail "…"; only a counter or extension suffix ("dusk_ridge_00…2.png") is kept.
String _fitMid(String s, TextStyle st, double maxW) {
  if (!maxW.isFinite || _tw(s, st) <= maxW - .5) return s;
  var tail = RegExp(r'\.\w{2,4}$').hasMatch(s) ? 8 : (RegExp(r'\d$').hasMatch(s) ? 3 : 0);
  tail = math.min(tail, s.length ~/ 3);
  final t = s.substring(s.length - tail);
  var lo = 0, hi = s.length - tail;
  while (lo < hi) {
    final m = (lo + hi + 1) >> 1;
    if (_tw('${s.substring(0, m)}…$t', st) <= maxW - .5) {
      lo = m;
    } else {
      hi = m - 1;
    }
  }
  return '${s.substring(0, lo)}…$t';
}

/// A one-line name with the one truncation rule ([_fitMid]).
class _Mid extends StatelessWidget {
  const _Mid(this.text, this.style);
  final String text;
  final TextStyle style;
  @override
  Widget build(BuildContext context) => LayoutBuilder(builder: (c, box) => Text(_fitMid(text, style, box.maxWidth), maxLines: 1, softWrap: false, overflow: TextOverflow.clip, style: style));
}

/// English name then Japanese name on one line. `whole`: the Japanese part is shown only when all of it fits (a cut "カラー化の…" tells nothing); otherwise the English name alone, with the one truncation rule.
class _NameJp extends StatelessWidget {
  const _NameJp(this.en, this.jp, this.enStyle, this.jpStyle, {this.whole = false});
  final String en, jp;
  final TextStyle enStyle, jpStyle;
  final bool whole;
  @override
  Widget build(BuildContext context) {
    final both = TextSpan(children: [TextSpan(text: en, style: enStyle), TextSpan(text: '  $jp', style: jpStyle)]);
    if (!whole) return Text.rich(both, maxLines: 1, overflow: TextOverflow.ellipsis);
    return LayoutBuilder(builder: (c, box) => _tw(en, enStyle) + _tw('  $jp', jpStyle) <= box.maxWidth - .5 ? Text.rich(both, maxLines: 1, softWrap: false, overflow: TextOverflow.clip) : Text(_fitMid(en, enStyle, box.maxWidth), maxLines: 1, softWrap: false, overflow: TextOverflow.clip, style: enStyle));
  }
}

/// The Inspector-region header of a layer (B9-b, B9-c): a plain glyph (never a plate that reads as a button), the kind, the name.
Widget _layerHead() => Row(children: [
      const nav.Glyph(nav.G.text, size: 14, color: N.g76),
      const SizedBox(width: 8),
      Expanded(child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [_t('Text layer', T.label(N.g76)), const SizedBox(height: 3), _t('タイトル — Intro v3', T.title())])),
    ]);

// ================================================================ shared: the effect library (real names, Japanese and English)

enum _Sel { none, shape, text, key, many }

const _cats = ['Blur & Sharpen', 'Color', 'Distort', 'Stylize', 'Time', 'Generate', 'Transition', 'Text', 'Ease (key)'];
const _catShort = ['ぼかし', 'カラー', 'ゆがみ', '装飾', '時間', '生成', 'ワイプ', 'テキスト', 'ease'];
const _catJp = ['ぼかし・シャープ', 'カラー', 'ゆがみ', 'スタイライズ', '時間', '生成', 'トランジション', 'テキスト', 'イージング'];

const _raw = <(String, String, int)>[
  ('Gaussian Blur', 'ガウスぼかし', 0), ('Fast Box Blur', 'ボックスぼかし(高速)', 0), ('Directional Blur', '方向ぼかし', 0), ('Radial Blur', '放射ぼかし', 0), ('Lens Blur', 'レンズぼかし', 0), ('Sharpen', 'シャープ', 0), ('Unsharp Mask', 'アンシャープマスク', 0),
  ('Hue/Saturation', '色相・彩度', 1), ('Levels', 'レベル補正', 1), ('Curves', 'トーンカーブ', 1), ('Tint', '色合い', 1), ('Fill', '塗り', 1), ('Gradient Map', 'グラデーションマップ', 1), ('Exposure', '露光量', 1), ('Vibrance', '自然な彩度', 1), ('Color Balance', 'カラーバランス', 1),
  ('Wave Warp', '波形ワープ', 2), ('Twirl', '渦巻き', 2), ('Bulge', '膨張', 2), ('Turbulent Displace', 'タービュレント ディスプレイス', 2), ('Ripple', '波紋', 2), ('Corner Pin', 'コーナーピン', 2), ('Polar Coordinates', '極座標', 2), ('Mesh Warp', 'メッシュワープ', 2),
  ('Glow', 'グロー', 3), ('Drop Shadow', 'ドロップシャドウ', 3), ('Mosaic', 'モザイク', 3), ('Halftone', 'ハーフトーン', 3), ('Posterize', 'ポスタライズ', 3), ('Find Edges', '輪郭検出', 3), ('Vignette', 'ビネット', 3),
  ('Chromatic Aberration with Lens Distortion', '色収差+レンズ歪み(広角レンズ向け)', 3), ('Glitch RGB Split', 'グリッチ RGB ずらし', 3), ('Film Grain', 'フィルムグレイン', 3), ('Scanlines', '走査線', 3),
  ('Echo', 'エコー', 4), ('Posterize Time', 'コマ落とし', 4), ('Time Displacement', 'タイムディスプレイスメント', 4), ('Frame Blend', 'フレームブレンド', 4),
  ('Fractal Noise', 'フラクタルノイズ', 5), ('Gradient Ramp', 'グラデーションランプ', 5), ('Grid', 'グリッド', 5), ('Lens Flare', 'レンズフレア', 5), ('Light Sweep', '光の走査', 5), ('4-Color Gradient', '4色グラデーション', 5),
  ('Linear Wipe', 'リニアワイプ', 6), ('Radial Wipe', '時計ワイプ', 6), ('Venetian Blinds', 'ブラインド', 6), ('Iris Wipe', 'アイリスワイプ', 6), ('Block Dissolve', 'ブロックディゾルブ', 6),
  ('Typewriter', '文字送り', 7), ('Per-character Fade', '文字ごとフェード', 7), ('Tracking Reveal', '字間アニメ', 7), ('Text Scramble', '文字スクランブル', 7),
  ('Ease Out Expo', '急停止(Expo)', 8), ('Ease In-Out Cubic', 'なめらか(Cubic)', 8), ('Overshoot', '行き過ぎ', 8), ('Bounce', 'バウンス', 8), ('Elastic', 'ゴム', 8), ('Anticipate', '予備動作', 8), ('Hold', '保持', 8), ('Snap', 'スナップ', 8), ('Slow-Mo Ease', 'スローモーション', 8),
];

class _Fx {
  const _Fx(this.en, this.jp, this.cat, this.seed);
  final String en, jp;
  final int cat, seed;
  bool get isKey => cat == 8;
  bool get isText => cat == 7;
}

final List<_Fx> _fxs = [for (final (i, e) in _raw.indexed) _Fx(e.$1, e.$2, e.$3, i)];
final List<_Fx> _layerFx = _fxs.where((f) => !f.isKey && !f.isText).toList();

bool _fits(_Fx f, _Sel s) => switch (s) {
      _Sel.none => true,
      _Sel.key => f.isKey,
      _Sel.text => !f.isKey,
      _Sel.shape || _Sel.many => !f.isKey && !f.isText,
    };

String _why(_Fx f, _Sel s) => f.isKey ? 'Key only' : (s == _Sel.key ? 'Layer only' : 'Text only');
String _selName(_Sel s) => switch (s) {
      _Sel.none => 'Nothing selected',
      _Sel.shape => 'Layer 4 · Logo Mark (Shape)',
      _Sel.text => 'Layer 3 · タイトル (Text)',
      _Sel.key => '2 keys · Position X',
      _Sel.many => '5 layers (mixed)',
    };

(String, String) _params(_Fx f) => switch (f.cat) {
      0 => ('Blurriness', 'Repeat Edge'),
      1 => ('Amount', 'Channel'),
      2 => ('Amount', 'Size'),
      3 => ('Intensity', 'Radius'),
      4 => ('Offset', 'Count'),
      5 => ('Scale', 'Seed'),
      _ => ('Completion', 'Feather'),
    };


double _ease(String n, double t) {
  double bounce(double x) {
    const n1 = 7.5625, d1 = 2.75;
    if (x < 1 / d1) return n1 * x * x;
    if (x < 2 / d1) return n1 * (x -= 1.5 / d1) * x + .75;
    if (x < 2.5 / d1) return n1 * (x -= 2.25 / d1) * x + .9375;
    return n1 * (x -= 2.625 / d1) * x + .984375;
  }

  switch (n) {
    case 'Ease Out Expo':
      return t >= 1 ? 1 : 1 - math.pow(2, -10 * t).toDouble();
    case 'Ease In-Out Cubic':
      return t < .5 ? 4 * t * t * t : 1 - math.pow(-2 * t + 2, 3) / 2;
    case 'Overshoot':
      const c1 = 1.70158, c3 = c1 + 1;
      return 1 + c3 * math.pow(t - 1, 3) + c1 * math.pow(t - 1, 2);
    case 'Bounce':
      return bounce(t);
    case 'Elastic':
      return t <= 0 ? 0 : (t >= 1 ? 1 : math.pow(2, -10 * t) * math.sin((t * 10 - .75) * (2 * math.pi / 3)) + 1);
    case 'Anticipate':
      const c = 1.70158 * 1.2;
      return (c + 1) * t * t * t - c * t * t;
    case 'Hold':
      return t < 1 ? 0 : 1;
    case 'Snap':
      return t < .25 ? 0 : (t < .3 ? (t - .25) / .05 : 1);
    default:
      return t * t * (3 - 2 * t) * .6 + t * .4 * t;
  }
}

class _CurveP extends CustomPainter {
  const _CurveP(this.name);
  final String name;
  @override
  void paint(Canvas c, Size s) {
    c.drawRect(Offset.zero & s, Paint()..color = N.g07);
    final g = Paint()..color = N.g20..strokeWidth = 1;
    for (var i = 1; i < 4; i++) {
      c.drawLine(Offset(s.width * i / 4, 0), Offset(s.width * i / 4, s.height), g);
      c.drawLine(Offset(0, s.height * i / 4), Offset(s.width, s.height * i / 4), g);
    }
    final p = Path();
    const n = 48;
    for (var i = 0; i <= n; i++) {
      final t = i / n, y = _ease(name, t).clamp(-.4, 1.4);
      final o = Offset(s.width * (.1 + .8 * t), s.height * (.82 - .64 * y));
      i == 0 ? p.moveTo(o.dx, o.dy) : p.lineTo(o.dx, o.dy);
    }
    c.drawPath(p, Paint()..color = N.g95..style = PaintingStyle.stroke..strokeWidth = 1.5..strokeJoin = StrokeJoin.round);
  }

  @override
  bool shouldRepaint(_CurveP o) => o.name != name;
}


// ================================================================ thumbnails: grey shapes, one shape per kind

/// The shape a thumbnail draws: one per effect family and one per media kind, so a row tells its kind without colour.
enum _Sh { blur, color, distort, stylize, time, generate, wipe, text, image, video, audio, model, pano }

_Sh _shOfFx(int cat) => const [_Sh.blur, _Sh.color, _Sh.distort, _Sh.stylize, _Sh.time, _Sh.generate, _Sh.wipe, _Sh.text][cat.clamp(0, 7)];
_Sh _shOfKind(Kind k) => switch (k) { Kind.image => _Sh.image, Kind.video => _Sh.video, Kind.audio => _Sh.audio, Kind.model => _Sh.model, Kind.pano => _Sh.pano };

/// A soft mid-grey tile with light-grey ink (never a near-black smear, never a hue). The seed nudges the size so siblings are not clones.
class _Ink extends CustomPainter {
  const _Ink(this.sh, this.seed);
  final _Sh sh;
  final int seed;
  @override
  void paint(Canvas c, Size s) {
    c.drawRect(Offset.zero & s, Paint()..shader = ui.Gradient.linear(Offset.zero, Offset(0, s.height), const [Color(0xFF6A6A6A), Color(0xFF585858)]));
    final w = s.width, h = s.height, u = h, m = Offset(w / 2, h / 2), k = .9 + (seed % 5) * .05, v = seed % 3;
    final ink = Paint()..color = N.g91.withValues(alpha: .8), soft = Paint()..color = N.g91.withValues(alpha: .45);
    final line = Paint()..color = N.g91.withValues(alpha: .85)..style = PaintingStyle.stroke..strokeWidth = math.max(1, u / 14)..strokeJoin = StrokeJoin.round..strokeCap = StrokeCap.round;
    switch (sh) {
      case _Sh.blur:
        if (v == 0) {
          for (var i = 2; i >= 0; i--) {
            c.drawCircle(m, u * (.14 + .11 * i) * k, Paint()..color = N.g91.withValues(alpha: [.8, .45, .22][i]));
          }
        } else if (v == 1) {
          for (var i = 0; i < 5; i++) {
            final y = h * (.26 + i * .12), a = [.25, .5, .85, .5, .25][i];
            c.drawLine(Offset(w * (.2 + (i % 2) * .06), y), Offset(w * (.8 - (i % 2) * .06), y), line..color = N.g91.withValues(alpha: a));
          }
          line.color = N.g91.withValues(alpha: .85);
        } else {
          for (var i = 2; i >= 0; i--) {
            c.drawCircle(m, u * (.12 + .12 * i) * k, line..color = N.g91.withValues(alpha: [.9, .55, .3][i]));
          }
          line.color = N.g91.withValues(alpha: .85);
        }
      case _Sh.color:
        final r = RRect.fromRectAndRadius(Rect.fromLTWH(w * .14, h * .26, w * .72, h * .48), Radius.circular(u * .08));
        final shader = v == 0
            ? ui.Gradient.linear(Offset(w * .14, 0), Offset(w * .86, 0), [N.g95.withValues(alpha: .9), N.g38.withValues(alpha: .9)])
            : (v == 1 ? ui.Gradient.linear(Offset(0, h * .26), Offset(0, h * .74), [N.g38.withValues(alpha: .9), N.g95.withValues(alpha: .9)]) : ui.Gradient.linear(Offset(w * .14, h * .26), Offset(w * .86, h * .74), [N.g95.withValues(alpha: .9), N.g38.withValues(alpha: .9), N.g95.withValues(alpha: .9)], const [0, .5, 1]));
        c.drawRRect(r, Paint()..shader = shader);
      case _Sh.distort:
        final p = Path();
        for (var i = 0; i <= 20; i++) {
          final t = i / 20, x = w * (.12 + .76 * t), y = v == 1 ? m.dy + (((t * 4 * k) % 1) < .5 ? -1 : 1) * u * .16 * (1 - ((t * 8 * k) % 1) * .0) : m.dy + math.sin(t * math.pi * 2 * k) * u * .2;
          i == 0 ? p.moveTo(x, y) : p.lineTo(x, y);
        }
        c.drawPath(p, line);
        if (v == 2) c.drawPath(p.shift(Offset(0, u * .2)), line..color = N.g91.withValues(alpha: .45));
        line.color = N.g91.withValues(alpha: .85);
      case _Sh.stylize:
        final q = u * .17;
        for (var i = 0; i < 3; i++) {
          for (var j = 0; j < 3; j++) {
            final o = Offset(m.dx + (i - 1) * q * 1.4, m.dy + (j - 1) * q * 1.4);
            if (v == 0) {
              c.drawRect(Rect.fromCenter(center: o, width: q, height: q), (i + j + seed) % 3 == 0 ? ink : soft);
            } else if (v == 1) {
              c.drawCircle(o, q * (.25 + .2 * ((i + j) % 3)), ink);
            } else {
              c.drawRect(Rect.fromCenter(center: o, width: q * (i == 1 ? 1 : .45), height: q * (j == 1 ? 1 : .45)), soft);
            }
          }
        }
      case _Sh.time:
        for (var i = 2; i >= 0; i--) {
          c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(w * .2 + i * u * .13, h * .24 + i * u * .06, w * .42, h * .42), Radius.circular(u * .06)), Paint()..color = N.g91.withValues(alpha: [.85, .5, .25][i]));
        }
      case _Sh.generate:
        final r = math.Random(seed * 17 + 3);
        for (var i = 0; i < 16; i++) {
          c.drawCircle(Offset(w * (.14 + .72 * r.nextDouble()), h * (.16 + .68 * r.nextDouble())), u * (.04 + .04 * r.nextDouble()), i.isEven ? ink : soft);
        }
      case _Sh.wipe:
        c.drawPath(Path()..moveTo(0, 0)..lineTo(w * (.5 + (seed % 3) * .05), 0)..lineTo(w * (.36 + (seed % 3) * .05), h)..lineTo(0, h)..close(), Paint()..color = N.g91.withValues(alpha: .7));
      case _Sh.text:
        for (var i = 0; i < 3; i++) {
          c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(w * .18, h * (.28 + i * .2), w * [.64, .5, .36][i] * k, u * .1), Radius.circular(u * .05)), i == 0 ? ink : soft);
        }
      case _Sh.image:
        c.drawPath(Path()..moveTo(w * .08, h * .82)..lineTo(w * .38, h * .36)..lineTo(w * .62, h * .82)..close(), ink);
        c.drawPath(Path()..moveTo(w * .46, h * .82)..lineTo(w * .7, h * .5)..lineTo(w * .94, h * .82)..close(), soft);
        c.drawCircle(Offset(w * .74, h * .26), u * .1 * k, ink);
      case _Sh.video:
        c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(w * .16, h * .2, w * .68, h * .6), Radius.circular(u * .08)), line);
        c.drawPath(Path()..moveTo(m.dx - u * .1, m.dy - u * .15)..lineTo(m.dx + u * .17, m.dy)..lineTo(m.dx - u * .1, m.dy + u * .15)..close(), ink);
      case _Sh.audio:
        for (var i = 0; i < 9; i++) {
          final a = u * (.1 + .3 * (.5 + .5 * math.sin(i * 1.3 + seed))) * k;
          c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: Offset(w * (.16 + .085 * i), m.dy), width: math.max(1.5, w * .04), height: a * 2), const Radius.circular(1)), ink);
        }
      case _Sh.model:
        final r = u * .34 * k;
        final pts = [for (var i = 0; i < 6; i++) m + Offset(math.cos(math.pi / 3 * i - math.pi / 2) * r, math.sin(math.pi / 3 * i - math.pi / 2) * r)];
        c.drawPath(Path()..addPolygon(pts, true), line);
        for (var i = 1; i < 6; i += 2) {
          c.drawLine(m, pts[i], line);
        }
      case _Sh.pano:
        c.drawOval(Rect.fromLTWH(w * .1, h * .24, w * .8, h * .52), line);
        c.drawLine(Offset(w * .1, m.dy), Offset(w * .9, m.dy), line);
        c.drawCircle(Offset(w * .66, h * .38), u * .06 * k, ink);
    }
  }

  @override
  bool shouldRepaint(_Ink o) => o.sh != sh || o.seed != seed;
}

/// A media thumbnail: a grey tile whose shape says the kind (mountain = image, frame = video, bars = audio, cube = 3D, horizon = 360).
class _Art extends StatelessWidget {
  const _Art(this.asset);
  final Asset asset;
  @override
  Widget build(BuildContext context) => CustomPaint(painter: _Ink(_shOfKind(asset.kind), asset.seed), size: Size.infinite);
}

class _Thumb extends StatelessWidget {
  const _Thumb(this.fx);
  final _Fx fx;
  @override
  Widget build(BuildContext context) => fx.isKey ? CustomPaint(painter: _CurveP(fx.en), size: Size.infinite) : CustomPaint(painter: _Ink(_shOfFx(fx.cat), fx.seed), size: Size.infinite);
}

/// The artist's own frame with the effect laid over it: a stand-in for "preview on my frame" (B5-c); the lab fakes the look per family, in grey.
class _Shot extends StatelessWidget {
  const _Shot(this.fx);
  final _Fx? fx;
  static const _base = _Art(Asset('shot', Kind.image, 3));
  @override
  Widget build(BuildContext context) {
    final f = fx;
    Widget body;
    if (f == null) {
      body = _base;
    } else if (f.cat == 0) {
      body = ImageFiltered(imageFilter: ui.ImageFilter.blur(sigmaX: 3, sigmaY: 3), child: _base);
    } else if (f.cat == 1) {
      body = Stack(fit: StackFit.expand, children: [_base, ColoredBox(color: N.g100.withValues(alpha: .1 + (f.seed % 3) * .05))]);
    } else {
      body = Stack(fit: StackFit.expand, children: [_base, Opacity(opacity: .45, child: _Thumb(f))]);
    }
    return ClipRRect(borderRadius: BorderRadius.circular(4), child: body);
  }
}

// ================================================================ shared: rows, tiles, grid, empty card

class _FxRow extends StatelessWidget {
  const _FxRow(this.fx, {this.sel = false, this.hover = false, this.tags = const [], this.whole = false});
  final _Fx fx;
  final bool sel, hover, whole;
  final List<Widget> tags;
  @override
  Widget build(BuildContext context) => _Line(
        height: 28,
        sel: sel,
        hover: hover,
        child: Row(children: [
          SizedBox(width: 22, height: 18, child: ClipRRect(borderRadius: BorderRadius.circular(3), child: _Thumb(fx))),
          const SizedBox(width: 8),
          Expanded(child: _NameJp(fx.en, fx.jp, T.name(sel ? N.g95 : N.g91), T.label(N.g76), whole: whole)),
          for (final t in tags) Padding(padding: const EdgeInsets.only(left: 4), child: t),
        ]),
      );
}

class _FxTile extends StatelessWidget {
  const _FxTile(this.fx, this.width, {this.sel = false, this.hover = false, this.dim = false, this.reason});
  final _Fx fx;
  final double width;
  final bool sel, hover, dim;
  final String? reason;
  @override
  Widget build(BuildContext context) {
    final h = width * .62;
    return SizedBox(
      width: width,
      child: Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
        Stack(children: [
          AnimatedContainer(
            duration: Mo.dur,
            curve: Mo.ease,
            width: width,
            height: h,
            foregroundDecoration: BoxDecoration(borderRadius: BorderRadius.circular(6), border: Border.all(color: sel ? _accent : (hover ? N.g56 : N.g20))),
            child: ClipRRect(borderRadius: BorderRadius.circular(6), child: Opacity(opacity: dim ? .35 : 1, child: _Thumb(fx))),
          ),
          if (reason != null) Positioned(left: 4, right: 4, bottom: 4, child: Center(child: Container(padding: const EdgeInsets.symmetric(horizontal: 5, vertical: 3), decoration: BoxDecoration(color: N.g10, borderRadius: BorderRadius.circular(3), border: Border.all(color: N.g26)), child: Text(reason!, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.micro(N.g76).copyWith(letterSpacing: .1))))),
        ]),
        const SizedBox(height: 5),
        _Mid(fx.en, T.name(sel ? N.g95 : (dim ? N.g63 : N.g91))),
        const SizedBox(height: 3),
        _t(fx.jp, T.label(N.g76)),
      ]),
    );
  }
}

/// Columns by width: the tile never goes under 92 px.
class _Grid extends StatelessWidget {
  const _Grid({required this.n, required this.item});
  final int n;
  final Widget Function(int i, double w) item;
  @override
  Widget build(BuildContext context) => LayoutBuilder(builder: (c, box) {
        final cols = math.max(2, ((box.maxWidth + 8) / 92).floor());
        final tw = (box.maxWidth - 8 * (cols - 1)) / cols;
        return Wrap(spacing: 8, runSpacing: 10, children: [for (var i = 0; i < n; i++) item(i, tw)]);
      });
}

/// A scrolling region that is cut by the frame: a 16 px fade into the panel ground at its foot says "there is more", so a half tile never looks like a bug.
Widget _fadeFoot(Widget child, {bool on = true}) => !on
    ? child
    : Stack(children: [
        child,
        Positioned(left: 0, right: 0, bottom: 0, height: 16, child: IgnorePointer(child: DecoratedBox(decoration: BoxDecoration(gradient: LinearGradient(begin: Alignment.topCenter, end: Alignment.bottomCenter, colors: [N.g10.withValues(alpha: 0), N.g10]))))),
      ]);
