// The making shelves: Colors (pop palettes, a swatch field, saved gradients) and Create (tiles for every new layer, shape and copier).
import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:auto_size_text/auto_size_text.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_colorpicker/flutter_colorpicker.dart';
import 'package:material_color_utilities/material_color_utilities.dart';

import '../sets/inspector/inspector_parts.dart' show Pop;
import '../tokens.dart';
import 'desk_kit.dart' show DeskChip;
import 'dock_glyphs.dart';
import 'dock_parts.dart';
import 'dock_tags.dart';
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
  // (a colour given to a thing is also what the next thing drawn comes in: its fill the ink, its outline the edge, as
  //  CLIP STUDIO PAINT has one drawing colour that every tool draws in, its text too, the last one chosen; glass fills
  //  a solid alone, so it is not kept)
  void keep(String name) {
    if (word == 'glass') return;
    if (name == 'fill') c.ws.ink = word;
    if (name == 'stroke') c.ws.edge = word;
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

  /// The Mac's eyedropper (NSColorSampler, MainFlutterWindow.swift): the colour taken is the wheel's, and given to the
  /// picked thing at once (or the ink, none picked), as Keynote's eyedropper gives it.
  Future<void> _sample() async {
    final hex = await const MethodChannel('motolii/sampler').invokeMethod<String>('sample');
    if (hex == null || int.tryParse(hex, radix: 16) == null) return;
    _to(HSVColor.fromColor(Color(0xFF000000 | int.parse(hex, radix: 16))));
    _fill(c, hex);
  }

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
                SizedBox(height: Shape.s(6)),
                DeskChip(key: const ValueKey('colour-sample'), label: 'Sample', onTap: _sample),   // (a colour from anywhere on the screen: the Mac's own eyedropper)
              ],
            ),
          ),
        ],
      ),
    );
  }
}
