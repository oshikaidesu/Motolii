part of 'panel_inspector_b.dart';

// ---- content: real-looking effects and properties ---------------------------------------------------------------------

enum _K { num, toggle, choice, color }

/// The colours a colour property can take (content, not chrome).
const _swatch = [N.g95, Color(0xFFE974AB), Color(0xFF7DD5B1), Color(0xFF4781E5), Color(0xFFEFCB4E), Color(0xFFF69260), Color(0xFFA889E9)];

class _P {
  const _P(this.id, this.en, this.jp, this.k, this.def, {this.min = 0, this.max = 100, this.unit = '', this.dec = 0, this.per = 1, this.opts = const [], this.major = false, this.group = 'Basic|基本'});
  final String id, en, jp, unit, group;
  final _K k;
  final double def, min, max, per;
  final int dec;
  final List<String> opts;
  final bool major;
  double get step => dec == 0 ? 1 : math.pow(10, -dec).toDouble();
  String grp(int lang) => lang == 2 ? group.split('|').last : group.split('|').first;
}

class _Fx {
  const _Fx(this.id, this.en, this.jp, this.params, {this.jpFx = false});
  final String id, en, jp;
  final List<_P> params;
  final bool jpFx;
  int get majorCount => params.where((p) => p.major).length;
}

String _nm(_P p, int lang, {bool jp = false}) => lang == 1 ? p.en : (lang == 2 ? p.jp : (jp ? p.jp : p.en));
String _fxName(_Fx f, int lang) => lang == 1 ? f.en : (lang == 2 ? f.jp : (f.jpFx ? f.jp : f.en));

bool _match(_P p, String q) {
  final s = q.toLowerCase();
  return p.en.toLowerCase().contains(s) || p.jp.toLowerCase().contains(s);
}

const _bl = ['Normal', 'Add', 'Multiply', 'Screen', 'Overlay', 'Soft Light'];

const _fractal = _Fx('fn', 'Fractal Noise', 'フラクタルノイズ', [
  _P('fn.type', 'Fractal Type', 'フラクタルの種類', _K.choice, 0, max: 3, opts: ['Basic', 'Turbulent Smooth', 'Dynamic', 'Max'], major: true),
  _P('fn.noise', 'Noise Type', 'ノイズの種類', _K.choice, 2, max: 3, opts: ['Block', 'Linear', 'Soft Linear', 'Spline'], major: true),
  _P('fn.invert', 'Invert', '反転', _K.toggle, 0, max: 1),
  _P('fn.contrast', 'Contrast', 'コントラスト', _K.num, 100, max: 400, unit: '%', per: .5, major: true),
  _P('fn.bright', 'Brightness', '明るさ', _K.num, 0, min: -200, max: 200, unit: '%'),
  _P('fn.overflow', 'Overflow', 'オーバーフロー', _K.choice, 0, max: 3, opts: ['Clip', 'Soft Clamp', 'Back Reflect', 'Allow HDR Results']),
  _P('fn.scale', 'Scale', 'スケール', _K.num, 100, min: 10, max: 10000, unit: '%', major: true, group: 'Transform|変形'),
  _P('fn.uni', 'Uniform Scaling', '縦横比を固定', _K.toggle, 1, max: 1, group: 'Transform|変形'),
  _P('fn.sw', 'Scale Width', 'スケールの幅', _K.num, 100, min: 10, max: 10000, unit: '%', group: 'Transform|変形'),
  _P('fn.sh', 'Scale Height', 'スケールの高さ', _K.num, 100, min: 10, max: 10000, unit: '%', group: 'Transform|変形'),
  _P('fn.ox', 'Offset Turbulence X', '乱流オフセット X', _K.num, 960, min: -4000, max: 4000, unit: 'px', group: 'Transform|変形'),
  _P('fn.oy', 'Offset Turbulence Y', '乱流オフセット Y', _K.num, 540, min: -4000, max: 4000, unit: 'px', group: 'Transform|変形'),
  _P('fn.rot', 'Rotation', '回転', _K.num, 0, min: -360, max: 360, unit: '°', dec: 1, group: 'Transform|変形'),
  _P('fn.persp', 'Perspective Offset', '遠近オフセット', _K.toggle, 0, max: 1, group: 'Transform|変形'),
  _P('fn.anchor', 'Transform Anchor Point Offset (Scaled by Layer Distance)', '変形アンカーポイントのオフセット(レイヤー距離でスケール)', _K.num, 0, min: -1000, max: 1000, unit: 'px', group: 'Transform|変形'),
  _P('fn.cx', 'Complexity', '複雑度', _K.num, 6, min: 1, max: 20, dec: 1, major: true, group: 'Complexity|複雑度'),
  _P('fn.si', 'Sub Influence', 'サブ影響度', _K.num, 70, unit: '%', group: 'Complexity|複雑度'),
  _P('fn.ss', 'Sub Scaling', 'サブスケール', _K.num, 56, min: 10, max: 400, unit: '%', group: 'Complexity|複雑度'),
  _P('fn.sr', 'Sub Rotation', 'サブ回転', _K.num, 0, min: -360, max: 360, unit: '°', dec: 1, group: 'Complexity|複雑度'),
  _P('fn.sox', 'Sub Offset X', 'サブオフセット X', _K.num, 0, min: -4000, max: 4000, unit: 'px', group: 'Complexity|複雑度'),
  _P('fn.soy', 'Sub Offset Y', 'サブオフセット Y', _K.num, 0, min: -4000, max: 4000, unit: 'px', group: 'Complexity|複雑度'),
  _P('fn.cs', 'Center Subscale', 'サブスケールの中心', _K.toggle, 0, max: 1, group: 'Complexity|複雑度'),
  _P('fn.evo', 'Evolution', '展開', _K.num, 0, min: -3600, max: 3600, unit: '°', dec: 1, per: .5, major: true, group: 'Evolution|展開'),
  _P('fn.cyc', 'Cycle Evolution', '展開のサイクル', _K.toggle, 0, max: 1, group: 'Evolution|展開'),
  _P('fn.cyclen', 'Cycle', 'サイクル', _K.num, 1, min: 1, max: 100, unit: 'rev', group: 'Evolution|展開'),
  _P('fn.seed', 'Random Seed', 'ランダムシード', _K.num, 0, max: 99999, group: 'Evolution|展開'),
  _P('fn.opacity', 'Opacity', '不透明度', _K.num, 100, unit: '%', major: true, group: 'Output|出力'),
  _P('fn.blend', 'Blending Mode', '描画モード', _K.choice, 0, max: 5, opts: _bl, group: 'Output|出力'),
]);

const _glow = _Fx('glow', 'Glow', 'グロー', [
  _P('gl.th', 'Glow Threshold', 'グローしきい値', _K.num, 60, unit: '%', major: true),
  _P('gl.r', 'Glow Radius', 'グロー半径', _K.num, 25, max: 500, unit: 'px', major: true),
  _P('gl.i', 'Glow Intensity', 'グロー強度', _K.num, 1.4, max: 10, dec: 2, per: .02, major: true),
  _P('gl.cmp', 'Composite Original', '元画像を合成', _K.choice, 0, max: 2, opts: ['On Top', 'Behind', 'None']),
  _P('gl.op', 'Glow Operation', 'グローの演算', _K.choice, 0, max: 5, opts: _bl),
  _P('gl.cols', 'Glow Colors', 'グローカラー', _K.choice, 0, max: 2, opts: ['Original Colors', 'A & B Colors', 'Arbitrary Map']),
  _P('gl.ca', 'Color A', 'カラー A', _K.color, 4),
  _P('gl.cb', 'Color B', 'カラー B', _K.color, 1),
  _P('gl.loop', 'Color Looping', 'カラーループ', _K.choice, 1, max: 3, opts: ['None', 'Sawtooth A>B', 'Sawtooth B>A', 'Triangle']),
  _P('gl.loops', 'Color Loops', 'カラーループ数', _K.num, 1, min: 1, max: 10, dec: 1),
  _P('gl.dim', 'Glow Dimensions', 'グローの方向', _K.choice, 0, max: 2, opts: ['Both axes', 'Horizontal', 'Vertical']),
]);

const _hue = _Fx('hue', 'Hue/Saturation', '色相/彩度', jpFx: true, [
  _P('hs.ch', 'Channel Control', 'チャンネル制御', _K.choice, 0, max: 3, opts: ['Master', 'Reds', 'Yellows', 'Greens'], major: true),
  _P('hs.hue', 'Master Hue', 'マスター色相', _K.num, 0, min: -180, max: 180, unit: '°', dec: 1, major: true),
  _P('hs.sat', 'Master Saturation', 'マスター彩度', _K.num, 0, min: -100, max: 100, unit: '%', major: true),
  _P('hs.lit', 'Master Lightness', 'マスター輝度', _K.num, 0, min: -100, max: 100, unit: '%', major: true),
  _P('hs.col', 'Colorize', 'カラー化', _K.toggle, 0, max: 1, major: true),
  _P('hs.ch2', 'Colorize Hue', 'カラー化の色相', _K.num, 0, min: -180, max: 180, unit: '°', dec: 1),
  _P('hs.cs', 'Colorize Saturation', 'カラー化の彩度', _K.num, 25, min: 0, max: 100, unit: '%'),
  _P('hs.cl', 'Colorize Lightness', 'カラー化の輝度', _K.num, 0, min: -100, max: 100, unit: '%'),
]);

const _blur = _Fx('blur', 'Gaussian Blur', 'ガウスブラー', [
  _P('gb.r', 'Blurriness', 'ブラーの量', _K.num, 12, max: 500, unit: 'px', dec: 1, per: .25, major: true),
  _P('gb.dim', 'Blur Dimensions', 'ブラーの方向', _K.choice, 0, max: 2, opts: ['Both axes', 'Horizontal', 'Vertical'], major: true),
  _P('gb.edge', 'Repeat Edge Pixels', 'エッジピクセルを繰り返す', _K.toggle, 1, max: 1),
]);

const _turb = _Fx('turb', 'Turbulent Displace', 'タービュレントディスプレイス', [
  _P('td.type', 'Displacement', '変位', _K.choice, 0, max: 3, opts: ['Turbulent', 'Bulge', 'Twist', 'Turbulent Smoother'], major: true),
  _P('td.amt', 'Amount', '量', _K.num, 50, max: 500, unit: 'px', major: true),
  _P('td.size', 'Size', 'サイズ', _K.num, 80, max: 500, unit: 'px', major: true),
  _P('td.off', 'Turbulence Offset', '乱流オフセット', _K.num, 0, min: -2000, max: 2000, unit: 'px'),
  _P('td.cx', 'Complexity', '複雑度', _K.num, 1, min: 1, max: 10, dec: 1),
  _P('td.evo', 'Evolution', '展開', _K.num, 0, min: -3600, max: 3600, unit: '°', dec: 1),
]);

const _echo = _Fx('echo', 'Echo', 'エコー', [
  _P('ec.t', 'Echo Time', 'エコー時間', _K.num, -.033, min: -1, max: 1, unit: 's', dec: 3, per: .005, major: true),
  _P('ec.n', 'Number Of Echoes', 'エコーの数', _K.num, 8, min: 1, max: 100, major: true),
  _P('ec.s', 'Starting Intensity', '開始強度', _K.num, 1, max: 1, dec: 2, per: .01),
  _P('ec.d', 'Decay', '減衰', _K.num, .6, max: 1, dec: 2, per: .01),
  _P('ec.op', 'Echo Operator', 'エコーの演算', _K.choice, 0, max: 3, opts: ['Add', 'Maximum', 'Screen', 'Composite In Back']),
]);

const _vig = _Fx('vig', 'Optics Compensation', 'オプティクス補正', [
  _P('oc.fov', 'Field Of View (FOV)', '視野 (FOV)', _K.num, 40, max: 180, unit: '°', dec: 1, major: true),
  _P('oc.rev', 'Reverse Lens Distortion', 'レンズ歪みを反転', _K.toggle, 0, max: 1),
  _P('oc.fv', 'FOV Orientation', '視野の方向', _K.choice, 0, max: 2, opts: ['Horizontal', 'Vertical', 'Diagonal']),
  _P('oc.vc', 'View Center', 'ビューの中心', _K.num, 0, min: -2000, max: 2000, unit: 'px'),
]);

/// What the stack panels show: a real-looking pile of effects, some off, one broken.
const _stackLib = <_Fx>[_glow, _fractal, _blur, _hue, _turb, _echo, _vig];

/// Extra names to scale a stack to 20 (a real project piles up effects; the names are real built-ins).
const _extraNames = [
  ('Curves', 'カーブ'), ('Levels', 'レベル補正'), ('Tint', '色合い'), ('Fill', '塗り'), ('Drop Shadow', 'ドロップシャドウ'), ('Directional Blur', '方向ブラー'), ('Radial Wipe', 'ラジアルワイプ'),
  ('CC Particle World', 'CC Particle World'), ('Posterize Time', 'ポスタリゼーション時間'), ('Mosaic', 'モザイク'), ('Lens Flare', 'レンズフレア'), ('Vignette (custom, large)', 'ビネット(カスタム・大)'), ('Sharpen', 'シャープ'),
];

// ---- state: values of a property list ------------------------------------------------------------------------------------

class _Vals extends ChangeNotifier {
  final Map<String, double> v = {};
  final List<String> touched = [];
  double of(_P p) => v[p.id] ?? p.def;
  bool changed(_P p) => (of(p) - p.def).abs() > 1e-9;
  void set(_P p, double x, {bool touch = true}) {
    v[p.id] = x.clamp(p.min, p.max).toDouble();
    if (touch) {
      touched
        ..remove(p.id)
        ..insert(0, p.id);
    }
    notifyListeners();
  }

  void reset(_P p) {
    v.remove(p.id);
    notifyListeners();
  }

  void clear() {
    v.clear();
    touched.clear();
    notifyListeners();
  }
}

/// A bag of local state for a use case. A knob that must act as an initial value goes through [sync].
class _St extends ChangeNotifier {
  final Map<String, Object?> _d = {};
  V get<V>(String k, V def) => _d.containsKey(k) ? _d[k] as V : def;
  void put(String k, Object? v) {
    _d[k] = v;
    notifyListeners();
  }

  void quiet(String k, Object? v) => _d[k] = v;

  /// True when [v] differs from what this key last saw (also the first time): the caller then applies the knob without notifying.
  bool sync(String k, Object? v) {
    final key = 'sync.$k';
    if (_d.containsKey(key) && _d[key] == v) {
      return false;
    }
    _d[key] = v;
    return true;
  }
}

String _f(double v, int dec) {
  var s = v.toStringAsFixed(dec);
  if (s.startsWith('-') && double.parse(s) == 0) {
    s = s.substring(1);
  }
  return s;
}

/// A single English / Japanese / mixed knob that every property panel reads.
int _lang(BuildContext c) => c.knobs.object.dropdown<int>(label: 'Names', options: const [0, 1, 2], initialOption: 0, labelBuilder: (v) => const ['Mixed (default)', 'English', '日本語'][v]);
