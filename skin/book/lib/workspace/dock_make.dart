// The making shelves: Colors (pop palettes, a swatch field, saved gradients) and Create (tiles for every new layer, shape and copier).
import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:auto_size_text/auto_size_text.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:material_color_utilities/material_color_utilities.dart';

import '../sets/inspector/inspector_parts.dart' show Pop;
import '../sets/panels/media_parts.dart' show MiniSlider;
import '../tokens.dart';
import 'desk_kit.dart' show DeskChip;
import 'dock_glyphs.dart';
import 'dock_parts.dart';
import '../theme.dart' show ColorTheme;
import 'colour_wheel.dart';
import 'ws_menu.dart';
import 'create_preview.dart';
import 'ws.dart';
import 'two_fingers.dart';
part 'dock_make_2.dart';
part 'dock_make_3.dart';

// ---------------------------------------------------------------- colors

class _Pal {
  const _Pal(this.id, this.name, this.colors, {this.saved = false, this.reference, this.shares});
  final String id, name;
  final List<Color> colors;
  final bool saved;

  /// The photo the colours were taken from, kept beside them (a copy, so it is there whenever the palette is looked at),
  /// and how much of the picture each colour is (the strip is drawn by it: the picture's composition).
  final String? reference;
  final List<double>? shares;
}

/// A gradient: its colours down the thing (a straight ramp, linear: round or straight is the inspector's), and how they
/// are mixed between, kept with it (the user: a gradient's way of mixing is kept in its preset as it is): rgb, oklab (as
/// an eye mixes, no muddy middle between blue and yellow), hue (round the hue), steps (each held till the next). The
/// inside mixes them (gradient_blend, CSS Color 4's color-interpolation-method).
class _Gradient {
  const _Gradient(this.name, this.colors, {this.blend = 'oklab', this.saved = false, this.at});
  final String name;
  final List<Color> colors;
  final List<double>? at;   // (where each colour stands, a share of the way; null: evenly)
  final String blend;
  final bool saved;
}

const _blends = {'rgb': 'RGB', 'oklab': 'OKLab', 'hue': 'Hue', 'steps': 'Steps'};

/// How the gradient coloured now is mixed: the picked thing's gradient_blend, or the ink's ('rgb' unsaid).
String _blendNow(DockCtx c) {
  final t = int.tryParse(c.ws.selected ?? ''), w = t != null && (c.ws.aim ?? 'fill') == 'fill' ? c.ws.inside!.word(t, 'gradient_blend') : c.ws.inkBlend;
  return w.isEmpty ? 'rgb' : w;
}

/// How a gradient looks on the shelf, mixed as the inside will mix it (OKLab drawn by CAM16, the eye's mix Flutter has).
(List<Color>, List<double>?) _previewOf(List<Color> cs, String blend, [List<double>? at]) => at != null && blend == 'rgb' ? (cs, at) : switch (blend) {
  'steps' => ([for (final c in cs) ...[c, c]], [for (var i = 0; i < cs.length; i++) ...[i / cs.length, (i + 1) / cs.length]]),
  'hue' => ([for (var i = 0; i < cs.length - 1; i++) for (var k = 0; k < 6; k++) HSVColor.lerp(HSVColor.fromColor(cs[i]), HSVColor.fromColor(cs[i + 1]), k / 6)!.toColor(), cs.last], null),
  'oklab' => (_stopsOf(cs, true), null),
  _ => (cs, null),
};

/// A gradient put on what is coloured: its colours, and how they are mixed (the picked thing's gradient_blend, or the
/// ink's, for the next thing drawn); one step.
void _putGradient(DockCtx c, List<Color> cs, String blend, [List<double>? at]) {
  final m = c.ws.inside!, t = int.tryParse(c.ws.selected ?? '');
  final word = at != null && at.length == cs.length ? _wordOfStops([for (var i = 0; i < cs.length; i++) (cs[i], at[i])]) : [for (final k in cs) _hex(k).substring(1).toLowerCase()].join(' ');
  if (t == null || (c.ws.aim ?? 'fill') != 'fill') {
    c.ws.inkBlend = blend;
    return _fill(c, word);
  }
  m.later('together', number: 1);
  _fill(c, word);
  m.later('say', thing: t, name: 'gradient_blend', word: blend);
  c.ws.after(m.later('together'));
}

const _starterPals = <_Pal>[
  _Pal('pop', 'Pop Poster', [Color(0xFFB3C66B), Color(0xFF4781E5), Color(0xFFF08A5D), Color(0xFFE974AB), Color(0xFFEFCB4E), Color(0xFF15170D)]),
  _Pal('riso', 'Riso Print', [Color(0xFFFF48B0), Color(0xFF0078BF), Color(0xFFFFE800), Color(0xFF00A95C), Color(0xFFF15060), Color(0xFFF2EDE4)]),
  _Pal('coda', 'Coda Title', [Color(0xFFD9D2C3), Color(0xFFF08A5D), Color(0xFF7FB8B0), Color(0xFF3B3631), Color(0xFFF2C94C)]),
  _Pal('market', 'Night Market', [Color(0xFF1B1F3B), Color(0xFF53354A), Color(0xFFE84545), Color(0xFFFFB740), Color(0xFF2B9EB3)]),
  _Pal('candy', 'Candy Shop', [Color(0xFFFFB5C2), Color(0xFFB5EAD7), Color(0xFFC7CEEA), Color(0xFFFFDAC1), Color(0xFFE2F0CB), Color(0xFF9ADCFF)]),
  _Pal('signal', 'Signal', [Color(0xFFFF3B30), Color(0xFFFF9500), Color(0xFFFFCC00), Color(0xFF34C759), Color(0xFF007AFF), Color(0xFFAF52DE)]),
];

const _starterGrads = <_Gradient>[
  _Gradient('Lime dusk', [Color(0xFFB3C66B), Color(0xFF4781E5)]),
  _Gradient('Hot pink', [Color(0xFFE974AB), Color(0xFFF08A5D), Color(0xFFEFCB4E)]),
  _Gradient('Deep sea', [Color(0xFF15170D), Color(0xFF2B9EB3), Color(0xFF7DD5B1)]),
];

// What the maker has kept: ~/.motolii/colors.json {"palettes": [{"name": "", "colors": ["#rrggbb"]}], "gradients": [{"name": "", "colors": [], "blend": "oklab"}], "colours": [], "tile": 52}.
final _minePals = <_Pal>[];
/// One's own colours, kept one by one (CLIP STUDIO PAINT's colour set: Add Color puts the drawing colour in, a tile
/// pressed draws with it, a tile's menu replaces or deletes it).
final _mineColours = <Color>[];
final _mineGrads = <_Gradient>[];
bool _mineRead = false;
final _mineFile = File('${Platform.environment['HOME']}/.motolii/colors.json');

Color? _colourOf(Object? hex) {
  final v = int.tryParse('$hex'.replaceFirst('#', ''), radix: 16);
  return v == null ? null : Color(0xFF000000 | v);
}

void _readMine() {
  if (_mineRead) return;
  _mineRead = true;
  try {
    final told = jsonDecode(_mineFile.readAsStringSync()) as Map;
    for (final (i, p) in ((told['palettes'] as List?) ?? const []).indexed) {
      final cs = [for (final h in (p['colors'] as List)) ?_colourOf(h)];
      final sh = (p['shares'] as List?)?.map((v) => (v as num).toDouble()).toList();
      if (cs.isNotEmpty) _minePals.add(_Pal('mine$i', '${p['name']}', cs, saved: true, reference: p['reference'] as String?, shares: sh != null && sh.length == cs.length ? sh : null));
    }
    _mineColours.addAll([for (final h in (told['colours'] as List?) ?? const []) ?_colourOf(h)]);
    if (told['tile'] case final num v) _tile = v.toDouble().clamp(_tileLeast, _tileMost);
    for (final g in (told['gradients'] as List?) ?? const []) {
      final cs = [for (final h in (g['colors'] as List)) ?_colourOf(h)];
      final at = (g['at'] as List?)?.map((v) => (v as num).toDouble()).toList();
      if (cs.length > 1) _mineGrads.add(_Gradient('${g['name']}', cs, blend: g['blend'] as String? ?? (g['oklab'] == false ? 'rgb' : 'oklab'), saved: true, at: at != null && at.length == cs.length ? at : null));
    }
  } catch (_) {}   // (no file: only the starters)
}

void _keepMine() {
  try {
    _mineFile.parent.createSync(recursive: true);
    _mineFile.writeAsStringSync(jsonEncode({
      'palettes': [for (final p in _minePals) {'name': p.name, 'colors': [for (final c in p.colors) _hex(c)], 'reference': p.reference, 'shares': p.shares}],
      'gradients': [for (final g in _mineGrads) {'name': g.name, 'colors': [for (final c in g.colors) _hex(c)], 'blend': g.blend, 'at': g.at}],
      'colours': [for (final c in _mineColours) _hex(c)],
      'tile': _tile,
    }));
  } catch (_) {}
}

List<_Pal> get _pals => [..._minePals, ..._starterPals];
List<_Gradient> get _grads => [..._mineGrads, ..._starterGrads];

String _hex(Color c) {
  String h(double v) => (v * 255).round().toRadixString(16).padLeft(2, '0').toUpperCase();
  return '#${h(c.r)}${h(c.g)}${h(c.b)}';
}

// ---- the mix of a gradient: CAM16-UCS (Google's, material_color_utilities), the space an eye mixes in, so the middle
// between two colours is what it expects. The ramp the document draws mixes plainly, so the middle is given as more
// stops, laid in that space (a few between each pair).
List<Color> _stopsOf(List<Color> cs, bool smooth, {int per = 5}) {
  if (!smooth || cs.length < 2) return cs;
  final out = <Color>[];
  for (var i = 0; i < cs.length - 1; i++) {
    final a = Cam16.fromInt(cs[i].toARGB32()), b = Cam16.fromInt(cs[i + 1].toARGB32());
    for (var k = 0; k < per; k++) {
      final t = k / per;
      out.add(k == 0 ? cs[i] : Color(Cam16.fromUcs(a.jstar + (b.jstar - a.jstar) * t, a.astar + (b.astar - a.astar) * t, a.bstar + (b.bstar - a.bstar) * t).viewed(ViewingConditions.sRgb)));
    }
  }
  return out..add(cs.last);
}

// The colour on the picker, and the ink, edge or value it last followed.
HSVColor _wheel = const HSVColor.fromAHSV(1, 0, .8, .9);
String _wheelFrom = '';

/// Joined: the picked thing's fill told to the document (a colour as six digits, or a material's name). Not done: the foot says why.
void _fill(DockCtx c, String word) {
  final m = c.ws.inside!, t = int.tryParse(c.ws.selected ?? '');
  // (a colour given to a thing is also what the next thing drawn comes in: its fill the ink, its outline the edge, as
  //  CLIP STUDIO PAINT has one drawing colour that every tool draws in, its text too, the last one chosen; glass fills
  //  a solid alone, so it is not kept)
  void keep(String name) {
    if (word == 'glass') return;
    if (name == 'fill') c.ws.ink = word;
    if (name == 'stroke') c.ws.edge = word;
  }
  if (t == null && (c.ws.aim == 'fill' || c.ws.aim == 'stroke')) { // (the tool strip's ink or edge aimed it, nothing picked: what a thing drawn next comes in, stage.dart _Swatches)
    c.set(() {
      word == 'glass' ? c.mem.trouble['fill'] = 'Glass fills a solid thing: pick one' : c.mem.trouble.remove('fill');
      keep(c.ws.aim!);
    });
    c.ws.reread();
    return;
  }
  if (c.ws.aim case final aim?) { // (a card's colour link aimed it: said to that value, of the picked thing or of the stage)
    c.ws.after(m.later('say', thing: t ?? 0, name: aim, word: word), (done, why) => c.set(() {
      if (done && t != null) keep(aim);
      done ? c.mem.trouble.remove('fill') : c.mem.trouble['fill'] = why;
    }));   // (without waiting)
    return;
  }
  if (t == null) { // (nothing picked: the drawing tools' ink, what a thing drawn next is filled with)
    c.set(() => word == 'glass' ? c.mem.trouble['fill'] = 'Glass fills a solid thing: pick one' : (c.ws.ink = word, c.mem.trouble.remove('fill')));
    c.ws.reread();
    return;
  }
  c.ws.after(m.later('say', thing: t, name: 'fill', word: word), (done, why) => c.set(() {
    if (done) keep('fill');
    done ? c.mem.trouble.remove('fill') : c.mem.trouble['fill'] = why;
  }));   // (without waiting)
}

/// The colours of the things, each by the thing's name: what the document's things are filled with (a ramp by its first).
Map<String, Color> usedColours(Ws ws) {
  final m = ws.inside;
  if (m == null) return const {};
  return {
    for (final l in ws.layers)
      if (m.word(int.parse(l.id), 'fill').split(' ').first.split('@').first case final f when f.length == 6)
        if (int.tryParse(f, radix: 16) case final v?) l.name: Color(0xFF000000 | v),
  };
}

/// How many of the Create shelf's tiles make or lay out something of the document, of all it shows.
String createFact(Ws ws) {
  final recipes = ws.inside!.recipes.length;
  return '${_asked.length + _laid.length + _worn.length + recipes} ready'; // (the shelf shows only these)
}

/// Joined: the inside's recipes, pictures in one call (as Cavalry's tutorials build them), each a tile.
(String, List<_Make>) _recipesOf(Ws ws) => ('Recipes', [for (final (name, what) in ws.inside!.recipes) _Make(name, DockG.copies, Pop.tonePlace, '', what, recipe: true)]);

/// The elements kept (Edit › Save to My Elements), put in at the playhead linked to their file: Alight Motion's + ›
/// Object / Element › Element, AbsentMotion's ElementPicker (ElementUi.kt).
(String, List<_Make>) _elementsOf(Ws ws) => ('Elements', [for (final (name, _) in ws.inside!.elements) _Make(name, DockG.nullObj, Grey.g76, '', 'Element, linked to its file')]);

/// A palette taken from a photo: the picture shrunk, its pixels sorted into colours by Google's quantizer
/// (material_color_utilities, as Material You takes a palette from a wallpaper), each with how many pixels it has
/// (how much of the picture it is). The photo itself is copied beside the palette and stays there.
Future<_Pal?> _paletteFrom(String path, {int count = 6}) async {
  try {
    final codec = await ui.instantiateImageCodec(await File(path).readAsBytes(), targetWidth: 96);
    final img = (await codec.getNextFrame()).image;
    final data = await img.toByteData(format: ui.ImageByteFormat.rawRgba);
    if (data == null) return null;
    final pixels = <int>[
      for (var i = 0; i + 3 < data.lengthInBytes; i += 4)
        if (data.getUint8(i + 3) > 200) 0xFF000000 | (data.getUint8(i) << 16) | (data.getUint8(i + 1) << 8) | data.getUint8(i + 2),
    ];
    if (pixels.isEmpty) return null;
    final found = (await QuantizerCelebi().quantize(pixels, count)).colorToCount.entries.toList()..sort((a, b) => b.value.compareTo(a.value));
    final total = found.fold<int>(0, (s, e) => s + e.value);
    final kept = Directory('${Platform.environment['HOME']}/.motolii/references')..createSync(recursive: true);
    final name = path.split('/').last, copy = '${kept.path}/${DateTime.now().millisecondsSinceEpoch}_$name';
    File(path).copySync(copy);
    return _Pal('mine${_minePals.length}', name.replaceFirst(RegExp(r'\.[A-Za-z0-9]+$'), ''), [for (final e in found) Color(e.key)], saved: true, reference: copy, shares: [for (final e in found) e.value / total]);
  } catch (_) {
    return null;
  }
}

/// The photo picked on the Media shelf, if it is one (a file of the disk that Flutter reads).
String? _pickedPhoto(DockCtx c) {
  final id = c.mem.sel['Media'];
  return id != null && id.startsWith('/') && RegExp(r'\.(png|jpe?g|webp)$', caseSensitive: false).hasMatch(id) ? id : null;
}


/// The palette whose details the shelf's foot tells: the one the hand is on, else the one last pressed.
final _named = ValueNotifier<String?>(null);
String? _pressed;
bool _wheelHeld = false;   // (a hand on the wheel: it follows the hand)

/// The Colors shelf, laid out to stay small however much is kept in it, and to colour in one press: what is coloured,
/// in one line; the colour now, its square and hue full width (flutter_colorpicker's own, as Figma's picker) and its
/// code; then every palette as one small card of its colours in stripes, side by side in rows (Coolors', Adobe
/// Color's and Canva's palette cards: a press on a stripe takes that colour), and the gradients as small tiles. A
/// press colours at once, as in the best-selling editors.
class ColorsShelf extends StatelessWidget {
  const ColorsShelf(this.c, {super.key});
  final DockCtx c;

  /// The thing a colour goes to now and its value's name (null: the next thing drawn).
  (int?, String) _target() {
    final t = int.tryParse(c.ws.selected ?? ''), aim = c.ws.aim ?? 'fill';
    return (t ?? (aim == 'fill' || aim == 'stroke' ? null : 0), aim);
  }

  @override
  Widget build(BuildContext context) {
    WsScope.of(context, {WsAspect.picked, WsAspect.document});   // (what it colours is said at its top: built again when a link or the tool strip aims it, or a thing is picked)
    _readMine();
    final m = c.ws.inside!, usedCount = <Color, int>{};
    for (final col in usedColours(c.ws).values) {
      usedCount[col] = (usedCount[col] ?? 0) + 1;   // (how many things are of it: how much of the picture, roughly)
    }
    final used = usedCount.keys.toList();
    final (t, name) = _target();
    final word = t == null ? (name == 'stroke' ? c.ws.edge : c.ws.ink) : m.word(t, name);
    final stops = _stopsIn(word), gradient = stops.length > 1, none = name == 'stroke' ? word.isEmpty : word == 'none';
    if (_stop >= stops.length) _stop = 0;
    // (the picker shows the colour of what is coloured now, or of the stop picked, as CLIP STUDIO PAINT's wheel shows
    //  the main or sub colour)
    if (word != _wheelFrom) {
      _wheelFrom = word;
      if (_wheelHeld) {
        // (a hand on the wheel: it keeps the hue the hand gave, which a grey written as hex would lose)
      } else if (gradient) {
        _wheel = HSVColor.fromColor(stops[_stop].$1.withValues(alpha: 1));
      } else if (_colourOf(word.split(' ').first.split('@').first) case final col?) {
        _wheel = HSVColor.fromColor(col);
      }
    }
    String hexOf(Color col) => _hex(col).substring(1).toLowerCase();
    final whom = t == null ? 'the next thing drawn' : t == 0 ? 'the stage' : (c.ws.layer?.name ?? 'the picked thing');
    final q = c.query.toLowerCase();
    final pals = [for (final p in _pals) if (q.isEmpty || p.name.toLowerCase().contains(q)) p];
    final grads = [for (final (i, g) in _grads.indexed) if (q.isEmpty || g.name.toLowerCase().contains(q)) (i, g)];
    final photo = _pickedPhoto(c);
    void to(HSVColor v) {   // (the square or the hue moved: the colour, or the picked stop's)
      _wheel = v;
      if (!gradient) return _fill(c, hexOf(v.toColor()));
      final next = [...stops];
      next[_stop] = (v.toColor().withValues(alpha: stops[_stop].$1.a), stops[_stop].$2);
      _fill(c, _wordOfStops(next));
    }
    // (what it is painted with: Alight Motion's four tabs, Figma's paint type; a press each. A colour to a gradient
    //  fades to clear, as Figma's first gradient does; a gradient to a colour keeps the stop picked)
    Widget kind(String label, bool on, VoidCallback go) => DeskChip(key: ValueKey('colour-kind-$label'), label: label, on: on, onTap: on ? null : go);
    // (two blocks, the user's: the top stays where it is, what is coloured, how, the wheel; only the colours below
    //  scroll, so + and the dice, at the head of them, are always at hand. The picker above, the kept colours under it:
    //  Procreate's Color Panel, its palette at the bottom of the Disc and Classic tabs; Figma's picker, the document's
    //  colours below the picker and its hex)
    final top = Padding(
      padding: EdgeInsets.fromLTRB(WsT.inset, Shape.s(6), WsT.inset, Shape.s(6)),
      child: Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      mainAxisSize: MainAxisSize.min,
      children: [
        Row(
          children: [
            Expanded(child: Text('${name == 'fill' ? 'Fill' : name == 'stroke' ? 'Stroke' : name[0].toUpperCase() + name.substring(1)} of $whom', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(Grey.g91).copyWith(fontWeight: FontWeight.w600))),
            SizedBox(width: Shape.s(6)),
            DeskChip(key: const ValueKey('colour-sample'), label: 'Sample', onTap: () => _sample(c)),   // (a colour from anywhere on the screen: the Mac's own eyedropper)
          ],
        ),
        if (c.mem.trouble['fill'] case final why?) Padding(padding: EdgeInsets.only(top: Shape.s(4)), child: Text(why, style: T.label(ColorTheme.of('Alert', const Color(0xFFE5484D))))),
        SizedBox(height: Shape.s(6)),
        Row(
          children: [
            kind('Solid', !gradient && !none, () => _fill(c, hexOf(gradient ? stops[_stop].$1 : _wheel.toColor()))),
            if (name == 'fill') ...[SizedBox(width: Shape.s(4)), kind('Gradient', gradient, () => _fill(c, _wordOfStops([(_wheel.toColor(), 0), (_wheel.toColor().withValues(alpha: 0), 1)])))],
            SizedBox(width: Shape.s(4)),
            kind('None', none, () => _fill(c, name == 'stroke' ? '' : 'none')),
            const Spacer(),
            if (gradient && stops.length > 2) DeskChip(key: const ValueKey('colour-stop-remove'), label: 'Remove stop', onTap: () => _fill(c, _wordOfStops([...stops]..removeAt(_stop)))),
          ],
        ),
        SizedBox(height: Shape.s(8)),
        if (gradient) ...[
          _Stops(
            stops,
            (next) => _fill(c, _wordOfStops(next)),
            (i) => c.set(() {
              _stop = i;
              _wheelFrom = '';   // (the picker shows the stop picked)
            }),
            onHand: (down) => down ? m.later('together', number: 1) : c.ws.after(m.later('together')),
          ),
          SizedBox(height: Shape.s(6)),
          Wrap(   // (how its colours are mixed between: CSS Color 4's color-interpolation-method, Blender's Constant; kept with it)
            spacing: Shape.s(4),
            children: [
              for (final MapEntry(key: b, value: label) in _blends.entries)
                DeskChip(key: ValueKey('colour-blend-$b'), label: label, on: _blendNow(c) == b, onTap: () {
                  final t = int.tryParse(c.ws.selected ?? '');
                  t != null && name == 'fill' ? c.ws.after(m.later('say', thing: t, name: 'gradient_blend', word: b)) : c.set(() => c.ws.inkBlend = b);
                }),
            ],
          ),
          SizedBox(height: Shape.s(8)),
        ],
        Center(   // (CLIP STUDIO PAINT's colour wheel: a hand on it, what it does till it lets go is one step)
          child: ConstrainedBox(
            constraints: BoxConstraints(maxWidth: Shape.z(170)),
            child: AspectRatio(
              aspectRatio: 1,
              child: WsColourWheel(
                key: const ValueKey('colour-square'),
                color: _wheel,
                onChanged: to,
                onStart: () {
                  _wheelHeld = true;
                  m.later('together', number: 1);
                },
                onEnd: () {
                  _wheelHeld = false;
                  c.ws.after(m.later('together'));
                },
              ),
            ),
          ),
        ),
        SizedBox(height: Shape.s(6)),
        Row(
          children: [
            Container(width: Shape.z(22), height: Shape.z(22), decoration: BoxDecoration(color: _wheel.toColor(), borderRadius: BorderRadius.all(Shape.r(4)), border: Border.all(color: Grey.g26))),
            SizedBox(width: Shape.s(8)),
            Text(_hex(_wheel.toColor()), style: T.value(Grey.g91).copyWith(fontWeight: FontWeight.w600)),
          ],
        ),
        SizedBox(height: Shape.s(10)),
        Row(
          children: [
            Expanded(child: Text('Colours', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(Grey.g76).copyWith(fontWeight: FontWeight.w600))),
            Text('size', style: T.label(Grey.g76)),
            SizedBox(width: Shape.s(4)),
            Flexible(   // (shorter in a narrow shelf, never pushing the row out)
              child: LayoutBuilder(
                builder: (context, b) => MiniSlider(   // (the boxes' size, any the maker likes; kept once the hand rests, not written every frame)
                  key: const ValueKey('colour-size'),
                  value: _tile,
                  min: _tileLeast,
                  max: _tileMost,
                  width: math.min(Shape.z(64), b.maxWidth),
                  onChanged: (v) {
                    c.set(() => _tile = v);
                    _tileKept?.cancel();
                    _tileKept = Timer(const Duration(milliseconds: 400), _keepMine);
                  },
                ),
              ),
            ),
            SizedBox(width: Shape.s(6)),
            DeskChip(key: const ValueKey('colour-add'), label: '+', onTap: () => c.set(() => _keepNow(c))),   // (the colour now kept as one's own, or the gradient whole: CLIP STUDIO PAINT's Add Color)
            SizedBox(width: Shape.s(4)),
            DeskChip(key: const ValueKey('colour-keep'), label: 'Keep', onTap: used.isEmpty ? null : () => c.set(() {   // (the document's colours kept as a palette of one's own)
              _minePals.insert(0, _Pal('mine${_minePals.length}', 'Used ${_minePals.length + 1}', used, saved: true));
              _keepMine();
            })),
            SizedBox(width: Shape.s(4)),
            DeskChip(key: const ValueKey('colour-photo'), label: 'Photo', onTap: photo == null ? null : () async {   // (taken from the photo picked in Media)
              final p = await _paletteFrom(photo);
              if (p != null) c.set(() {
                _minePals.insert(0, p);
                _keepMine();
              });
            }),
            SizedBox(width: Shape.s(6)),
            _Dice(() => c.set(() => _dice++)),
          ],
        ),
      ],
      ),
    );
    final list = SingleChildScrollView(
      padding: EdgeInsets.fromLTRB(WsT.inset, Shape.s(6), WsT.inset, WsT.inset),
      child: _Bento([   // (what was added last at the top: one's own colours, one's own gradients, newest first; the user, "追加はいつも上から"; no app found doing so: Procreate fills the next empty slot, Canva sorts by hue)
          if (q.isEmpty) for (final (i, col) in _mineColours.indexed) _Box('mine', i, 1, colour: col),
          for (final (i, g) in grads) if (g.saved) _Box('grads', i, 1, grad: g),
          if (q.isEmpty) for (final (i, col) in used.indexed) _Box('used', i, usedCount[col]! * used.length / usedCount.values.fold(0, (a, b) => a + b), colour: col),
          for (final p in pals)
            for (final (i, col) in p.colors.indexed) _Box(p.id, i, (p.shares?[i] ?? 1 / p.colors.length) * p.colors.length, colour: col),
          for (final (i, g) in grads) if (!g.saved) _Box('grads', i, 1, grad: g),
        ], c),
    );
    return Column(
      children: [
        top,
        Container(height: 1, color: WsT.line),
        Expanded(child: list),
        ValueListenableBuilder<String?>(   // (the details of the palette under the hand or last pressed: its photo, its name, whose it is)
          valueListenable: _named,
          builder: (context, id, _) {
            final p = _pals.where((p) => p.id == id).firstOrNull;
            if (p == null) return const SizedBox.shrink();
            final photo = p.reference != null && File(p.reference!).existsSync() ? p.reference : null;
            final whose = photo != null ? 'From ${photo.split('/').last.replaceFirst(RegExp(r'^[0-9]+_'), '')}' : p.saved ? 'Yours' : 'Starter';
            return DockFooter(
              key: const ValueKey('colour-details'),
              preview: photo != null ? Image.file(File(photo), fit: BoxFit.cover, cacheWidth: 96) : Row(crossAxisAlignment: CrossAxisAlignment.stretch, children: [for (final col in p.colors) Expanded(child: ColoredBox(color: col))]),
              title: p.name,
              meta: '$whose · ${p.colors.length} colours',
              action: 'Remove',
              done: !p.saved,
              doneLabel: 'Starter',
              onAction: () => c.set(() {   // (one's own taken off the shelf, its file kept in step)
                _minePals.remove(p);
                _named.value = _pressed = null;
                _keepMine();
              }),
            );
          },
        ),
      ],
    );
  }
}

/// What is coloured now, as the document or the inks say it: the picked thing's value aimed at (its fill, unless a
/// link or the tool strip aimed another), else the stage's, else the ink or edge of the next thing drawn.
String _paintNow(DockCtx c) {
  final t = int.tryParse(c.ws.selected ?? ''), aim = c.ws.aim ?? 'fill', m = c.ws.inside!;
  if (t != null) return m.word(t, aim);
  if (aim == 'fill' || aim == 'stroke') return aim == 'stroke' ? c.ws.edge : c.ws.ink;
  return m.word(0, aim);
}

/// A colour taken from a swatch or the eyedropper: given to what is coloured; while that is a gradient, to the stop
/// picked (Figma's and Canva's swatches colour the stop being edited).
void _take(DockCtx c, Color col) {
  final stops = _stopsIn(_paintNow(c));
  if (stops.length < 2) return _fill(c, _hex(col).substring(1).toLowerCase());
  final next = [...stops];
  next[_stop.clamp(0, stops.length - 1)] = (col, stops[_stop.clamp(0, stops.length - 1)].$2);
  _wheelFrom = '';
  _fill(c, _wordOfStops(next));
}

/// The Mac's eyedropper (NSColorSampler, MainFlutterWindow.swift): the colour taken is given at once to what is
/// coloured, as Keynote's eyedropper gives it.
Future<void> _sample(DockCtx c) async {
  final hex = await const MethodChannel('motolii/sampler').invokeMethod<String>('sample');
  if (hex == null || int.tryParse(hex, radix: 16) == null) return;
  c.set(() => _wheel = HSVColor.fromColor(Color(0xFF000000 | int.parse(hex, radix: 16))));
  _take(c, Color(0xFF000000 | int.parse(hex, radix: 16)));
}
