// The making shelves: Colors (pop palettes, a swatch field, saved gradients) and Create (tiles for every new layer, shape and copier).
import 'dart:convert';
import 'dart:io';
import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:auto_size_text/auto_size_text.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_colorpicker/flutter_colorpicker.dart';
import 'package:material_color_utilities/material_color_utilities.dart';

import '../sets/inspector/inspector_parts.dart' show Pop;
import '../tokens.dart';
import 'dock_glyphs.dart';
import 'dock_parts.dart';
import 'dock_tags.dart';
import 'ws.dart';

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

/// A gradient: its colours down the thing (a straight ramp, linear: round or straight is the inspector's), and the
/// space it is mixed in. CAM16 is how an eye mixes (no muddy middle between blue and yellow); sRGB is a plain mix.
class _Gradient {
  const _Gradient(this.name, this.colors, {this.oklab = true, this.saved = false});
  final String name;
  final List<Color> colors;
  final bool oklab, saved;
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

// What the maker has kept: ~/.motolii/colors.json {"palettes": [{"name": "", "colors": ["#rrggbb"]}], "gradients": [{"name": "", "colors": [], "oklab": true}]}.
final _minePals = <_Pal>[];
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
    for (final g in (told['gradients'] as List?) ?? const []) {
      final cs = [for (final h in (g['colors'] as List)) ?_colourOf(h)];
      if (cs.length > 1) _mineGrads.add(_Gradient('${g['name']}', cs, oklab: g['oklab'] != false, saved: true));
    }
  } catch (_) {}   // (no file: only the starters)
}

void _keepMine() {
  try {
    _mineFile.parent.createSync(recursive: true);
    _mineFile.writeAsStringSync(jsonEncode({
      'palettes': [for (final p in _minePals) {'name': p.name, 'colors': [for (final c in p.colors) _hex(c)], 'reference': p.reference, 'shares': p.shares}],
      'gradients': [for (final g in _mineGrads) {'name': g.name, 'colors': [for (final c in g.colors) _hex(c)], 'oklab': g.oklab}],
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

/// The swatch field: twelve hues down four tones, plus a grey ramp.
final _field = [
  for (final l in const [.78, .64, .5, .34]) [for (var i = 0; i < 12; i++) HSLColor.fromAHSL(1, i * 30.0, .62, l).toColor()],
  [for (var i = 0; i < 12; i++) HSLColor.fromAHSL(1, 0, 0, i / 11).toColor()],
];

// What is being made on this shelf and not kept yet: the stops of a gradient, the colour on the wheel, the swatches a search found.
final _tray = <Color>[];
bool _trayOklab = true;
HSVColor _wheel = const HSVColor.fromAHSV(1, 0, .8, .9);
List<Color> _found = const [];

/// Joined: a fill that is not a colour. Glass is a material a solid is filled with, not a thing of its own.
final _glass = _Make('Glass', DockG.radial, WsT.toneShape, '', 'Thick glass material: bends and tints what lies behind it. Fills a solid; flat shapes keep a colour.');

/// Joined: the picked thing's fill told to the document (a colour as six digits, or a material's name). Not done: the foot says why.
void _fill(DockCtx c, String word) {
  final m = c.ws.inside!, t = int.tryParse(c.ws.selected ?? '');
  if (c.ws.aim case final aim?) { // (a card's colour link aimed it: said to that value, of the picked thing or of the stage)
    c.ws.after(m.later('say', thing: t ?? 0, name: aim, word: word), (done, why) => c.set(() => done ? c.mem.trouble.remove('fill') : c.mem.trouble['fill'] = why));   // (without waiting)
    return;
  }
  if (t == null) { // (nothing picked: the drawing tools' ink, what a thing drawn next is filled with)
    c.set(() => word == 'glass' ? c.mem.trouble['fill'] = 'Glass fills a solid thing: pick one' : (c.ws.ink = word, c.mem.trouble.remove('fill')));
    c.ws.reread();
    return;
  }
  c.ws.after(m.later('say', thing: t, name: 'fill', word: word), (done, why) => c.set(() => done ? c.mem.trouble.remove('fill') : c.mem.trouble['fill'] = why));   // (without waiting)
}

/// The colours of the things, each by the thing's name: what the document's things are filled with (a ramp by its first).
Map<String, Color> usedColours(Ws ws) {
  final m = ws.inside;
  if (m == null) return const {};
  return {
    for (final l in ws.layers)
      if (m.word(int.parse(l.id), 'fill').split(' ').first case final f when f.length == 6)
        if (int.tryParse(f, radix: 16) case final v?) l.name: Color(0xFF000000 | v),
  };
}

/// How many of the Create shelf's tiles make or lay out something of the document, of all it shows.
String createFact(Ws ws) {
  final recipes = ws.inside!.recipes.length;
  return '${_asked.length + _laid.length + recipes} ready'; // (the shelf shows only these)
}

/// Joined: the inside's recipes, pictures in one call (as Cavalry's tutorials build them), each a tile.
(String, List<_Make>) _recipesOf(Ws ws) => ('Recipes', [for (final (name, what) in ws.inside!.recipes) _Make(name, DockG.copies, Pop.tonePlace, '', what)]);

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

final _refOpen = <String>{};   // (palettes whose photo is opened large in the card)

class _Pill extends StatelessWidget {
  const _Pill(this.label, this.onTap, {this.on = false, this.enabled = true});
  final String label;
  final VoidCallback onTap;
  final bool on, enabled;
  @override
  Widget build(BuildContext context) => DockHover(
    onTap: enabled ? onTap : null,
    builder: (context, h) => Container(
      height: Shape.z(18),
      padding: EdgeInsets.symmetric(horizontal: Shape.s(7)),
      alignment: Alignment.center,
      decoration: BoxDecoration(color: on ? WsT.accent : (h && enabled ? WsT.raised : WsT.well), borderRadius: BorderRadius.all(Shape.r(WsT.chipRadius))),
      child: Text(label, maxLines: 1, style: T.micro(on ? WsT.onAccent : (enabled ? Grey.g91 : Grey.g38)).copyWith(fontWeight: FontWeight.w600)),
    ),
  );
}

class ColorsShelf extends StatelessWidget {
  const ColorsShelf(this.c, {super.key});
  final DockCtx c;

  @override
  Widget build(BuildContext context) {
    _readMine();
    final used = usedColours(c.ws);
    final chip = c.chip, shelf = 'Colors';
    // (a palette's tags: the ones most of its colours have; a swatch's own)
    Set<String> palTags(_Pal p) {
      final n = <String, int>{};
      for (final col in p.colors) {
        for (final t in DockTags.ofColour(col)) {
          n[t] = (n[t] ?? 0) + 1;
        }
      }
      return DockTags.of(shelf, p.id, {for (final e in n.entries) if (e.value * 2 >= p.colors.length) e.key});
    }

    final pals = [
      for (final p in _pals)
        if (DockTags.hit(c.query, p.name, palTags(p), code: p.colors.map(_hex).join(' ')) &&
            (chip == 'All' || (chip == 'Saved' ? p.saved : chip == 'Starter' ? !p.saved : palTags(p).contains(chip)))) p,
    ];
    final grads = [
      for (final g in _grads)
        if (DockTags.hit(c.query, g.name, DockTags.of(shelf, 'g_${g.name}', {for (final col in g.colors) ...DockTags.ofColour(col)}), code: g.colors.map(_hex).join(' ')) &&
            (chip == 'All' || (chip == 'Saved' ? g.saved : chip == 'Starter' ? !g.saved : true)))
          g,
    ];
    // (the swatches that answer a search or a tag, laid in one strip, since the field's grid cannot lose cells)
    final tagged = c.query.isNotEmpty || !const ['All', 'Used', 'Saved', 'Starter'].contains(chip);
    _found = tagged
        ? [
            for (final row in _field)
              for (final col in row)
                if (DockTags.hit(c.query, '', DockTags.of(shelf, _hex(col), DockTags.ofColour(col)), code: _hex(col)) && (const ['All', 'Used', 'Saved', 'Starter'].contains(chip) || DockTags.ofColour(col).contains(chip))) col,
          ]
        : const [];
    final vocab = <String, int>{};
    for (final p in _pals) {
      for (final t in palTags(p)) {
        vocab[t] = (vocab[t] ?? 0) + 1;
      }
    }
    final top = vocab.entries.toList()..sort((a, b) => b.value.compareTo(a.value));
    Widget block(List<Widget> kids) => Padding(
      padding: EdgeInsets.only(bottom: WsT.gutter),
      child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: kids),
    );
    Widget grid(double cell, List<Widget> items) => LayoutBuilder(
      builder: (context, box) {
        final cols = wsCols(box.maxWidth, cell), w = wsCell(box.maxWidth, cols);
        return Wrap(
          spacing: WsT.gutter,
          children: [for (final i in items) SizedBox(width: w, child: i)],
        );
      },
    );
    final all = chip == 'All' && c.query.isEmpty;
    final photo = _pickedPhoto(c);
    final blocks = <(Widget, double)>[
      if (all || (chip == 'All' && c.hit('glass material')))
        (
          block([
            DockSection('Materials', count: 1, hue: _glass.hue),
            const SizedBox(height: WsT.gutter),
            Align(
              alignment: Alignment.centerLeft,
              child: SizedBox(width: Shape.z(74), height: Shape.z(70), child: _MakeTile(_glass, c)),
            ),
          ]),
          1.6,
        ),
      if (all)
        (
          block([
            DockSection('Wheel', hue: Pop.tonePlace),
            _WheelBlock(c),
          ]),
          2.2,
        ),
      if ((chip == 'All' || chip == 'Used') && c.query.isEmpty)
        (
          block([
            DockSection('Used here', count: used.length, hue: WsT.accent, trailing: _Pill('Save as palette', () => c.set(() {
              if (used.isEmpty) return;
              _minePals.insert(0, _Pal('mine${_minePals.length}', 'Used ${_minePals.length + 1}', used.values.toList(), saved: true));
              _keepMine();
            }), enabled: used.isNotEmpty)),
            _Strip('used', used.values.toList(), c, names: used.keys.toList()),
          ]),
          2.0,
        ),
      if (chip != 'Used' && (pals.isNotEmpty || (all && true)))
        (
          block([
            DockSection('Palettes', count: pals.length, hue: Fam.scatter.c, trailing: _Pill(photo == null ? 'From photo: pick one in Media' : 'From photo', () async {
              final p = await _paletteFrom(photo!);
              if (p != null) c.set(() {
                _minePals.insert(0, p);
                _keepMine();
              });
            }, enabled: photo != null)),
            grid(220, [for (final p in pals) _PalCard(p, c)]),
          ]),
          1 + pals.length * 2.6,
        ),
      if ((chip == 'All' || chip == 'Saved' || chip == 'Starter' || !const ['Used'].contains(chip)) && chip != 'Used' && (grads.isNotEmpty || all))
        (
          block([
            DockSection('Gradients', count: grads.length, hue: Fam.stagger.c),
            if (all) _GradMaker(c),
            grid(200, [for (final (i, g) in _grads.indexed) if (grads.contains(g)) _Grad('grad-$i', g, c)]),
          ]),
          3.0 + grads.length,
        ),
      if (_found.isNotEmpty)
        (
          block([
            DockSection('Swatches found', count: _found.length, hue: Fam.face.c),
            const SizedBox(height: WsT.gutter),
            _Strip('found', _found.take(24).toList(), c, height: Shape.z(22), gap: 1),
          ]),
          1.6,
        ),
      if (all || (chip == 'All' && c.hit('swatches')))
        (
          block([
            DockSection('Swatches', count: 60, hue: Fam.face.c),
            const SizedBox(height: WsT.gutter),
            for (final (j, row) in _field.indexed) _Strip('field-$j', row, c, height: Shape.z(18), gap: 1),
          ]),
          4.0,
        ),
    ];
    final sel = _resolve(c.sel, used);
    final selTags = sel == null ? null : DockTags.ofColour(sel.$2).take(4).join(' · ');
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        DockChips(
          [
            ('All', null),
            ('Used', used.length),
            ('Saved', _pals.where((p) => p.saved).length + _grads.where((g) => g.saved).length),
            ('Starter', _starterPals.length + _starterGrads.length),
            for (final t in top.take(8)) (t.key, t.value),
          ],
          active: chip,
          onPick: c.pickChip,
        ),
        Expanded(
          child: blocks.isEmpty
              ? DockEmpty(c.query)
              : DockMasonry(minColumn: 300, weights: [for (final b in blocks) b.$2], children: [for (final b in blocks) b.$1]),
        ),
        if (c.sel == _glass.name)
          DockFooter(
            preview: DockBadge(_glass.g, _glass.hue, size: Shape.z(34)),
            title: _glass.name,
            meta: c.mem.trouble['fill'] ?? _glass.what,
            action: 'Fill',
            onAction: () => _fill(c, 'glass'),
          )
        else if (sel != null)
          DockFooter(
            preview: ColoredBox(color: sel.$2),
            title: _hex(sel.$2),
            meta: c.mem.trouble['fill'] ?? '${c.ws.aim == null ? sel.$1 : 'to ${c.ws.aim}'} · $selTags',
            action: c.ws.aim == null ? 'Fill' : 'Set',
            onAction: () => _fill(c, _wordOf(c.sel) ?? _hex(sel.$2).substring(1).toLowerCase()),
          ),
      ],
    );
  }

  /// Joined: a gradient is all its colours (smooth: with the stops between them), a ramp down the thing (the document's fill of two colours or more).
  String? _wordOf(String? id) {
    final parts = id?.split('-') ?? const [];
    final i = parts.length > 1 ? int.tryParse(parts.last) : null;
    if (i == null) return null;
    final key = parts.sublist(0, parts.length - 1).join('-');
    if (key == 'grad' && i < _grads.length) {
      final g = _grads[i];
      return [for (final k in _stopsOf(g.colors, g.oklab)) _hex(k).substring(1).toLowerCase()].join(' ');
    }
    if (key == 'maker' && _tray.length > 1) return [for (final k in _stopsOf(_tray, _trayOklab)) _hex(k).substring(1).toLowerCase()].join(' ');
    return null;
  }

  (String, Color)? _resolve(String? id, Map<String, Color> used) {
    if (id == null) return null;
    final parts = id.split('-');
    final i = int.tryParse(parts.last);
    if (i == null) return null;
    final key = parts.sublist(0, parts.length - 1).join('-');
    if (key == 'wheel') return ('Wheel · H ${_wheel.hue.round()}° S ${(_wheel.saturation * 100).round()}% V ${(_wheel.value * 100).round()}%', _wheel.toColor());
    if (key == 'found' && i < _found.length) return ('Swatch found', _found[i]);
    if (key == 'tray' && i < _tray.length) return ('Stop ${i + 1} of the gradient being made', _tray[i]);
    if (key == 'maker' && _tray.length > 1) return ('Gradient being made · ${_tray.length} stops', _tray.first);
    if (key == 'used' && i < used.length) return ('Used by ${used.keys.elementAt(i)}', used.values.elementAt(i));
    if (key.startsWith('field') && i < 12) {
      final hsl = HSLColor.fromColor(_field[int.parse(key.split('-').last)][i]);
      return ('Swatch · H ${hsl.hue.round()}° L ${(hsl.lightness * 100).round()}%', _field[int.parse(key.split('-').last)][i]);
    }
    if (key == 'grad' && i < _grads.length) return ('Gradient · ${_grads[i].name} · ${_grads[i].colors.length} stops · ${_grads[i].oklab ? 'CAM16' : 'sRGB'}', _grads[i].colors.first);
    final p = _pals.where((p) => p.id == key).firstOrNull;
    if (p != null && i < p.colors.length) return ('${p.name} · ${i + 1} of ${p.colors.length}', p.colors[i]);
    return null;
  }
}

/// The colour wheel and its value strip (flutter_colorpicker's own): hue round the ring, saturation from the middle
/// out, light to dark on the strip. A hand on it picks the colour at once.
class _WheelBlock extends StatelessWidget {
  const _WheelBlock(this.c);
  final DockCtx c;
  void _to(HSVColor v) => c.set(() {
    _wheel = v;
    c.mem.sel[c.shelf] = 'wheel-0';
  });

  @override
  Widget build(BuildContext context) {
    final size = Shape.z(128), v = _wheel;
    return Padding(
      padding: EdgeInsets.all(WsT.inset),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(width: size, height: size, child: ColorPickerArea(v, _to, PaletteType.hueWheel)),
          SizedBox(width: Shape.s(8)),
          SizedBox(width: Shape.z(28), height: size, child: RotatedBox(quarterTurns: 3, child: ColorPickerSlider(TrackType.value, v, _to, displayThumbColor: true))),
          SizedBox(width: Shape.s(10)),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Container(height: Shape.z(28), decoration: BoxDecoration(color: v.toColor(), borderRadius: BorderRadius.all(Shape.r(3)))),
                SizedBox(height: Shape.s(6)),
                Text(_hex(v.toColor()), style: T.name(Grey.g95).copyWith(fontWeight: FontWeight.w700)),
                Text('H ${v.hue.round()}°  S ${(v.saturation * 100).round()}%  V ${(v.value * 100).round()}%', style: T.label(Grey.g56)),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

/// The gradient being made: its stops (a colour put in with Add stop; a stop pressed is taken out), the ramp they
/// make in the space chosen, and Save (kept with the maker's own).
class _GradMaker extends StatelessWidget {
  const _GradMaker(this.c);
  final DockCtx c;
  @override
  Widget build(BuildContext context) {
    final picked = _pickedColour(c), ramp = _stopsOf(_tray, _trayOklab);
    return Container(
      margin: EdgeInsets.only(top: WsT.gutter),
      color: WsT.card,
      padding: EdgeInsets.all(WsT.inset),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Row(
            children: [
              Expanded(child: Text('Make a gradient', style: T.name(Grey.g95).copyWith(fontWeight: FontWeight.w700))),
              _Pill(_trayOklab ? 'Mix: CAM16' : 'Mix: sRGB', () => c.set(() => _trayOklab = !_trayOklab), on: _trayOklab),
            ],
          ),
          SizedBox(height: Shape.s(6)),
          DockHover(
            onTap: _tray.length > 1 ? () => c.pick('maker-0') : null,
            builder: (context, h) => Container(
              height: Shape.z(24),
              decoration: BoxDecoration(
                color: _tray.length > 1 ? null : WsT.well,
                gradient: _tray.length > 1 ? LinearGradient(colors: ramp) : null,
                borderRadius: BorderRadius.all(Shape.r(WsT.chipRadius)),
                border: c.sel == 'maker-0' ? Border.all(color: Grey.g95, width: 2) : null,
              ),
              alignment: Alignment.center,
              child: _tray.length > 1 ? null : Text('Add two colours or more', style: T.label(Grey.g56)),
            ),
          ),
          SizedBox(height: Shape.s(6)),
          Row(
            children: [
              for (final (i, col) in _tray.indexed) ...[
                DockHover(
                  onTap: () => c.set(() => _tray.removeAt(i)),   // (pressed: taken out)
                  builder: (context, h) => Container(width: Shape.z(18), height: Shape.z(18), decoration: BoxDecoration(color: col, border: h ? Border.all(color: Grey.g95, width: 2) : null, borderRadius: BorderRadius.all(Shape.r(3)))),
                ),
                SizedBox(width: Shape.s(4)),
              ],
              _Pill('Add stop', () => c.set(() => _tray.add(picked!)), enabled: picked != null && _tray.length < 8),
              SizedBox(width: Shape.s(4)),
              _Pill('Save', () => c.set(() {
                _mineGrads.insert(0, _Gradient('Gradient ${_mineGrads.length + 1}', List.of(_tray), oklab: _trayOklab, saved: true));
                _tray.clear();
                _keepMine();
              }), enabled: _tray.length > 1),
              SizedBox(width: Shape.s(4)),
              _Pill('Clear', () => c.set(_tray.clear), enabled: _tray.isNotEmpty),
            ],
          ),
        ],
      ),
    );
  }
}

/// The colour picked on this shelf now, if one is picked.
Color? _pickedColour(DockCtx c) {
  final id = c.sel;
  if (id == null) return null;
  final parts = id.split('-'), i = int.tryParse(parts.last);
  if (i == null) return null;
  final key = parts.sublist(0, parts.length - 1).join('-');
  if (key == 'wheel') return _wheel.toColor();
  if (key == 'found' && i < _found.length) return _found[i];
  if (key == 'used') return usedColours(c.ws).values.elementAtOrNull(i);
  if (key.startsWith('field') && i < 12) return _field[int.parse(key.split('-').last)][i];
  if (key == 'tray') return i < _tray.length ? _tray[i] : null;
  final p = _pals.where((p) => p.id == key).firstOrNull;
  return p != null && i < p.colors.length ? p.colors[i] : null;
}

class _PalCard extends StatelessWidget {
  const _PalCard(this.p, this.c);
  final _Pal p;
  final DockCtx c;
  @override
  Widget build(BuildContext context) {
    final ref = p.reference, open = _refOpen.contains(p.id);
    return Container(
      margin: EdgeInsets.only(top: WsT.gutter),
      color: WsT.card,
      padding: EdgeInsets.fromLTRB(WsT.inset, Shape.s(6), WsT.inset, WsT.inset),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Row(
            children: [
              Expanded(
                child: Text(p.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(Grey.g95).copyWith(fontWeight: FontWeight.w700)),
              ),
              if (p.saved) ...[const DockTag('SAVED'), SizedBox(width: Shape.s(6))],
              Text('${p.colors.length}', style: T.value(Grey.g56).copyWith(fontSize: T.px(10))),
            ],
          ),
          SizedBox(height: Shape.s(6)),
          Row(
            children: [
              if (ref != null && File(ref).existsSync()) ...[   // (the photo it was taken from: pressed, it opens large and stays)
                DockHover(
                  onTap: () => c.set(() => open ? _refOpen.remove(p.id) : _refOpen.add(p.id)),
                  builder: (context, h) => ClipRRect(
                    borderRadius: BorderRadius.all(Shape.r(3)),
                    child: SizedBox(width: Shape.z(40), height: Shape.z(26), child: Image.file(File(ref), fit: BoxFit.cover, cacheWidth: 120)),
                  ),
                ),
                SizedBox(width: Shape.s(6)),
              ],
              Expanded(child: _Strip(p.id, p.colors, c, height: Shape.z(26), pad: false, shares: p.shares)),
            ],
          ),
          if (open && ref != null && File(ref).existsSync()) ...[
            SizedBox(height: Shape.s(6)),
            ClipRRect(
              borderRadius: BorderRadius.all(Shape.r(3)),
              child: ConstrainedBox(constraints: BoxConstraints(maxHeight: Shape.z(160)), child: Image.file(File(ref), fit: BoxFit.contain, cacheWidth: 480)),
            ),
          ],
        ],
      ),
    );
  }
}

/// Flat swatches edge to edge (each as wide as its share of a photo, if it has one); the picked one gets a light frame with a dark keyline inside, readable on any hue.
class _Strip extends StatelessWidget {
  const _Strip(this.id, this.colors, this.c, {this.names, this.height = 24, this.gap = WsT.gutter, this.pad = true, this.shares});
  final String id;
  final List<Color> colors;
  final DockCtx c;
  final List<String>? names;
  final double height, gap;
  final bool pad;
  final List<double>? shares;
  @override
  Widget build(BuildContext context) => Padding(
    padding: EdgeInsets.fromLTRB(pad ? WsT.inset : Shape.s(0), pad ? gap : Shape.s(0), pad ? WsT.inset : Shape.s(0), Shape.s(0)),
    child: SizedBox(
      height: height,
      child: Row(
        children: [
          for (final (i, col) in colors.indexed) ...[
            if (i > 0) SizedBox(width: gap),
            Expanded(
              flex: shares == null ? 1000 : math.max(60, (shares![i] * 1000).round()),
              child: DockHover(
                onTap: () => c.pick('$id-$i'),
                onDoubleTap: () => _fill(c, _hex(col).substring(1).toLowerCase()),   // (two presses fill; carried to the stage it does the same)
                builder: (context, h) {
                  final sel = c.sel == '$id-$i';
                  return Container(
                    decoration: BoxDecoration(
                      color: col,
                      borderRadius: BorderRadius.circular(sel || h ? 3 : 0),
                      border: sel ? Border.all(color: Grey.g95, width: 2) : (h ? Border.all(color: Grey.g76) : null),
                    ),
                    foregroundDecoration: sel
                        ? BoxDecoration(
                            border: Border.all(color: Grey.g07),
                            borderRadius: BorderRadius.all(Shape.r(1)),
                          )
                        : null,
                  );
                },
              ),
            ),
          ],
        ],
      ),
    ),
  );
}

class _Grad extends StatelessWidget {
  const _Grad(this.id, this.g, this.c);
  final String id;
  final _Gradient g;
  final DockCtx c;
  @override
  Widget build(BuildContext context) {
    final sel = c.sel == id;
    return DockHover(
      onTap: () => c.pick(id),
      onDoubleTap: () => _fill(c, [for (final k in _stopsOf(g.colors, g.oklab)) _hex(k).substring(1).toLowerCase()].join(' ')),
      builder: (context, h) => AnimatedContainer(
        duration: Mo.dur,
        curve: Mo.ease,
        height: Shape.z(28),
        margin: EdgeInsets.only(top: WsT.gutter),
        padding: EdgeInsets.symmetric(horizontal: WsT.inset),
        color: dockFill(sel, h, WsT.card),
        child: Row(
          children: [
            Container(
              width: Shape.z(64),
              height: Shape.z(16),
              decoration: BoxDecoration(
                gradient: LinearGradient(colors: _stopsOf(g.colors, g.oklab)),
                borderRadius: BorderRadius.all(Shape.r(WsT.chipRadius)),
              ),
            ),
            SizedBox(width: Shape.s(8)),
            Expanded(
              child: Text(g.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(dockInk(sel))),
            ),
            Text('${g.colors.length} stops · ${g.oklab ? 'CAM16' : 'sRGB'}', style: T.label(dockInk(sel, Grey.g56))),
          ],
        ),
      ),
    );
  }
}

// ---------------------------------------------------------------- create

class _Make {
  const _Make(this.name, this.g, this.hue, this.key, this.what);
  final String name, key, what;
  final DockG g;
  final Color hue;
}

final _makes = <(String, List<_Make>)>[
  (
    'Layers',
    [
      _Make('Text', DockG.text, WsT.toneText, 'T', 'Type layer'),
      _Make('Camera', DockG.camera, WsT.toneCamera, '⇧C', '3D camera, 50 mm'),
      _Make('Null', DockG.nullObj, Grey.g76, '⇧N', 'Invisible parent'),
    ],
  ),
  (
    'Shapes',
    [
      _Make('Rectangle', DockG.rect, WsT.toneShape, 'R', 'Rounded corners'),
      _Make('Ellipse', DockG.ellipse, WsT.toneShape, 'E', 'Circle with ⇧'),
      _Make('Polygon', DockG.polygon, WsT.toneShape, '', 'Sides · Roundness'),
      _Make('Star', DockG.star, WsT.toneShape, '', 'Points · Inner radius'),
      _Make('Line', DockG.line, WsT.toneShape, 'L', 'Open path'),
    ],
  ),
  (
    'Copies',
    [
      _Make('Repeater', DockG.copies, Pop.tonePlace, '', 'Copies of the picked thing: how many, in how many rows, how far bent round a centre'),
    ],
  ),
];

/// The inside's own 3D shapes, and how a tile is asked of it (kind, then its file or shape).
/// What a solid is made of (a colour, glass) is not a tile here: it is its fill, given on the Colors shelf.
final _solids = (
  '3D',
  [
    _Make('Ring', DockG.radial, WsT.toneShape, '', 'A solid ring'),
    _Make('Gem', DockG.polygon, WsT.toneShape, '', 'A cut solid'),
    _Make('Cube', DockG.rect, WsT.toneShape, '', 'A round-edged cube'),
    _Make('Ball', DockG.ellipse, WsT.toneShape, '', 'A ball'),
  ],
);

/// Filament's own lights, things of the document: placed and turned as any thing, they light the solid things.
final _lights = (
  'Light',
  [
    _Make('Sun', DockG.light, WsT.toneCamera, '', 'Light from one way, everywhere'),
    _Make('Point', DockG.light, WsT.toneCamera, '', 'Light from a point, all round'),
    _Make('Spot', DockG.light, WsT.toneCamera, '', 'A cone of light'),
  ],
);
final _asked = {
  'Text': ('text', wsTextFont),
  'Solid': ('shape', 'solid'), // (a flat colour as large as the stage: the inside makes it so)
  'Rectangle': ('shape', 'rect'),
  'Ellipse': ('shape', 'circle'),
  'Polygon': ('shape', 'polygon'),
  'Star': ('shape', 'star'),
  'Line': ('shape', 'line'),
  'Null': ('shape', 'null'), // (nothing to see: a thing to go with)
  'Camera': ('camera', 'camera'), // (the nearest one that is on is the stage's eye)
  'Ring': ('solid', 'ring'),
  'Gem': ('solid', 'gem'),
  'Cube': ('solid', 'cube'),
  'Ball': ('solid', 'ball'),
  'Sun': ('light', 'sun'),
  'Point': ('light', 'point'),
  'Spot': ('light', 'spot'),
};

/// The tiles that lay the picked thing out in copies (the document's copies, as After Effects' repeater), and
/// what each tells it: in a row, side by side and a little smaller each time; round a centre below it.
const _laid = <String, Map<String, double>>{
  'Repeater': {'copies': 5, 'copy_x': 1.3, 'copy_size': .92},
};

/// A tile made on the stage as a new thing, picked; a tile of copies lays the picked thing out. Not done: the foot of
/// the shelf says why.
void _make(DockCtx c, String name) {
  final m = c.ws.inside!;
  if (m.recipes.any((r) => r.$1 == name)) { // (a picture in one call: one step, and it is picked)
    c.ws.after(m.later('make', word: name), (done, why) => c.set(() => done ? c.mem.trouble.remove(name) : c.mem.trouble[name] = why), (made) => c.ws.select('$made'));   // (without waiting: picked once made)
    return;
  }
  if (_laid[name] case final told?) {
    final t = int.tryParse(c.ws.selected ?? '');
    if (t == null) return c.set(() => c.mem.trouble[name] = 'Pick a thing to lay out');
    m.later('together', number: 1);   // (one step, without waiting)
    for (final e in told.entries) {
      m.later('set', thing: t, name: e.key, number: e.value.toDouble());
    }
    c.ws.after(m.later('together'), (done, why) => c.set(() => done ? c.mem.trouble.remove(name) : c.mem.trouble[name] = why));
    return;
  }
  final asked = _asked[name]!;
  c.ws.after(m.later('add', name: asked.$1, word: asked.$2), (done, why) => c.set(() => done ? c.mem.trouble.remove(name) : c.mem.trouble[name] = why), (made) => c.ws.select('$made'));   // (without waiting: picked once made)
}

class CreateShelf extends StatelessWidget {
  const CreateShelf(this.c, {super.key});
  final DockCtx c;
  @override
  Widget build(BuildContext context) {
    final makes = [
      _recipesOf(c.ws),
      ..._makes,
      _solids,
      _lights,
    ].where((e) => e.$2.isNotEmpty).toList();
    final groups = [
      for (final (g, items) in makes)
        if (c.chip == 'All' || c.chip == g) (g, items.where((m) => c.hit(m.name)).toList()),
    ].where((e) => e.$2.isNotEmpty).toList();
    final sel = makes.expand((e) => e.$2).where((m) => m.name == c.sel).firstOrNull;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        DockChips(
          [('All', makes.fold<int>(0, (s, e) => s + e.$2.length)), for (final (g, items) in makes) (g, items.length)],
          active: c.chip,
          onPick: c.pickChip,
        ),
        Expanded(
          child: groups.isEmpty
              ? DockEmpty(c.query)
              : DockMasonry(
                  minColumn: 300,
                  weights: [for (final (_, items) in groups) 1 + items.length / 3],
                  children: [
                    for (final (g, items) in groups)
                      Padding(
                        padding: EdgeInsets.only(bottom: WsT.gutter),
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.stretch,
                          children: [
                            DockSection(g, count: items.length, hue: items.first.hue, folded: c.query.isEmpty && c.mem.createFolded.contains(g), onTap: () => c.set(() => c.mem.createFolded.contains(g) ? c.mem.createFolded.remove(g) : c.mem.createFolded.add(g))),
                            if (!(c.query.isEmpty && c.mem.createFolded.contains(g))) ...[
                            const SizedBox(height: WsT.gutter),
                            LayoutBuilder(
                              builder: (context, box) {
                                final cols = wsCols(box.maxWidth, 74, min: 2), w = wsCell(box.maxWidth, cols);
                                return Wrap(
                                  spacing: WsT.gutter,
                                  runSpacing: WsT.gutter,
                                  children: [for (final m in items) SizedBox(width: w, height: Shape.z(70), child: _MakeTile(m, c))],
                                );
                              },
                            ),
                            ],
                          ],
                        ),
                      ),
                  ],
                ),
        ),
        if (sel != null && sel.name == 'Repeater') _CopiesPanel(c),
        if (sel != null && sel.name != 'Repeater')
          DockFooter(
            preview: DockBadge(sel.g, sel.hue, size: Shape.z(34)),
            title: '${sel.name} layer',
            meta: c.mem.trouble[sel.name] ?? 'At ${c.ws.timecode()} · ${sel.what}',
            action: _laid.containsKey(sel.name) ? 'Lay out' : 'Create',
            fresh: !_laid.containsKey(sel.name), // (a tile makes a new thing; a tile of copies lays out the picked one)
            onAction: () => _make(c, sel.name),
          ),
      ],
    );
  }
}

class _MakeTile extends StatelessWidget {
  const _MakeTile(this.m, this.c);
  final _Make m;
  final DockCtx c;
  @override
  Widget build(BuildContext context) {
    final sel = c.sel == m.name;
    final inside = c.ws.inside;
    final makes = inside != null && (_asked.containsKey(m.name) || _laid.containsKey(m.name) || inside.recipes.any((r) => r.$1 == m.name));
    return DockHover(
      onTap: () => c.pick(m.name),
      onDoubleTap: makes ? () => _make(c, m.name) : null, // (two presses make it, one step)
      builder: (context, h) => AnimatedContainer(
        duration: Mo.dur,
        curve: Mo.ease,
        padding: EdgeInsets.all(Shape.s(7)),
        color: sel ? WsT.accent : (h ? WsT.raised : WsT.card),
        child: Stack(
          children: [
            Align(
              alignment: Alignment.topLeft,
              child: DockGlyph(m.g, size: Shape.z(26), color: sel ? WsT.onAccent : m.hue),
            ),
            if (m.key.isNotEmpty)
              Align(
                alignment: Alignment.topRight,
                child: Text(m.key, style: T.micro(dockInk(sel, Grey.g56))),
              ),
            Align(
              alignment: Alignment.bottomLeft,
              child: AutoSizeText( // (a long name on two lines of whole words, as Finder's icons have it; smaller rather than a word broken)
                m.name,
                maxLines: 2,
                wrapWords: false,
                minFontSize: 6,
                stepGranularity: .5,
                style: T.name(dockInk(sel, Grey.g95)).copyWith(fontWeight: FontWeight.w700),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

// ---- the copies, as values (not as three kinds): how many, how many rows, how far bent round a centre. Copies are the
// document's own numbers (copies, copy_x, copy_y, copy_columns, copy_rotate, anchor_y), each keyable, so a row
// can be bent into a ring, a grid into a row, by keys. The sliders write those numbers on the picked thing; a hand's
// whole pull is one step.
double _copyCount = 5, _copyColumns = 1, _copyBend = 0;

void _lay(DockCtx c) {
  final t = int.tryParse(c.ws.selected ?? ''), m = c.ws.inside;
  if (t == null || m == null) return;
  final n = _copyCount.round(), cols = _copyColumns.round().clamp(1, n), bend = _copyBend, spacing = 1.3;
  final ring = n / (2 * math.pi) * spacing;   // (the radius at which n copies a step apart lie round a centre)
  final values = {
    'copies': n.toDouble(),
    'copy_columns': cols.toDouble(),
    'copy_x': spacing * (1 - bend),
    'copy_y': cols > 1 ? -spacing * (1 - bend) : 0.0,
    'copy_rotate': bend * 360 / n,
    'anchor_y': -bend * ring,
  };
  for (final e in values.entries) {
    m.later('set', thing: t, name: e.key, number: e.value);
  }
}

class _CopiesPanel extends StatelessWidget {
  const _CopiesPanel(this.c);
  final DockCtx c;
  @override
  Widget build(BuildContext context) {
    final picked = int.tryParse(c.ws.selected ?? '') != null, m = c.ws.inside;
    Widget slide(String label, String shown, double v, ValueChanged<double> to) => Padding(
      padding: EdgeInsets.only(bottom: Shape.s(4)),
      child: Row(
        children: [
          SizedBox(width: Shape.z(54), child: Text(label, style: T.label(Grey.g76))),
          Expanded(
            child: LayoutBuilder(
              builder: (context, box) {
                void at(Offset p) => c.set(() {
                  to((p.dx / box.maxWidth).clamp(0.0, 1.0));
                  if (picked) _lay(c);
                });
                return GestureDetector(
                  behavior: HitTestBehavior.opaque,
                  onPanDown: (d) {
                    if (picked) m?.later('together', number: 1);
                    at(d.localPosition);
                  },
                  onPanUpdate: (d) => at(d.localPosition),
                  onPanEnd: (_) => picked ? c.ws.after(m!.later('together')) : null,
                  onPanCancel: () => picked ? c.ws.after(m!.later('together')) : null,
                  child: SizedBox(
                    height: Shape.z(16),
                    child: Stack(
                      alignment: Alignment.centerLeft,
                      children: [
                        Container(height: 4, decoration: BoxDecoration(color: WsT.well, borderRadius: BorderRadius.all(Shape.r(2)))),
                        FractionallySizedBox(widthFactor: v, child: Container(height: 4, decoration: BoxDecoration(color: WsT.accent, borderRadius: BorderRadius.all(Shape.r(2))))),
                      ],
                    ),
                  ),
                );
              },
            ),
          ),
          SizedBox(width: Shape.z(40), child: Text(shown, textAlign: TextAlign.right, style: T.value(Grey.g91))),
        ],
      ),
    );
    return Container(
      color: WsT.card,
      padding: EdgeInsets.fromLTRB(WsT.inset, Shape.s(8), WsT.inset, Shape.s(8)),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Row(
            children: [
              Expanded(child: Text('Repeater', style: T.name(Grey.g95).copyWith(fontWeight: FontWeight.w700))),
              Text(picked ? 'on the picked thing, live' : 'pick a thing to lay out', style: T.label(Grey.g56)),
            ],
          ),
          SizedBox(height: Shape.s(6)),
          slide('Count', '${_copyCount.round()}', (_copyCount - 1) / 39, (f) => _copyCount = 1 + f * 39),
          slide('Rows', '${(_copyCount / _copyColumns.round().clamp(1, _copyCount.round())).ceil()}', (_copyColumns - 1) / 19, (f) => _copyColumns = 1 + f * 19),
          slide('Bend', '${(_copyBend * 360).round()}°', _copyBend, (f) => _copyBend = f),
        ],
      ),
    );
  }
}
