import 'dart:convert';
import 'dart:typed_data';
import 'dart:ui' show Color;

import 'package:flutter/foundation.dart';

import '../../hf/bp/catalog_io.dart';
import '../../hf/bp/colors.dart' show Sw;
import '../../hf/bp/fonts.dart' show FontItem;
import '../../hf/bp/things.dart';
import '../../session/color_edit.dart';
import '../../session/editor_session.dart';
import '../../session/swatches.dart';
import 'browser_user.dart';

/// The reference's own catalogue: every tile, its family, face and order.
const _thingsDir = String.fromEnvironment('MOTOLII_THINGS', defaultValue: 'lib/hf/data/things');

/// Reference tile -> the host's create kind.
const _createKinds = {
  'motolii.text': 'text',
  'motolii.rect': 'rectangle',
  'motolii.rounded': 'roundedRectangle',
  'motolii.ellipse': 'ellipse',
  'motolii.star': 'star',
  'motolii.polygon': 'polygon',
  'motolii.line': 'line',
  'motolii.path': 'bezier',
  'motolii.cube': 'cube',
  'motolii.sphere': 'sphere',
  'motolii.torus': 'torus',
  'motolii.cylinder': 'cylinder',
  'motolii.cone': 'cone',
  'motolii.plane': 'plane',
  'motolii.nul': 'null',
  'motolii.camera': 'camera',
  'motolii.particles': 'particles',
  'motolii.stage': 'stage',
};

/// What one thing does on the host: an op and its arguments.
typedef LiveBinding = (String op, Map<String, dynamic> args);

/// The Browser's meaning over a document session, with no screen in it: the catalogue (the reference's things and
/// every effect the host has), what each thing does, the user's own views per shelf (favorites, recent,
/// collections), the swatches, the fonts, and the operations (use a thing, apply a colour, a gradient, a font). One
/// per EditorSession; skins read it and call it.
class BrowserSession extends ChangeNotifier {
  BrowserSession._(this.c) {
    _bind();
    c.slice('browserCatalog', const ['createKinds', 'catalog']).addListener(_rebind);
    c.slice('browserSurface', const ['palette', 'fontFamilies', 'assets', 'backgrounds', 'layers', 'selectedId', 'selectedIds', 'documentRevision', 'colorTarget']).addListener(notifyListeners);
    c.deskWork.addListener(notifyListeners);
    _loadFontFacts();
  }
  static final _all = Expando<BrowserSession>();
  static BrowserSession of(EditorSession c) => _all[c] ??= BrowserSession._(c);
  final EditorSession c;
  static final Catalog _reference = loadCatalog(_thingsDir);

  // ---- the catalogue and what each thing does --------------------------------------------------------------------
  late Catalog catalog;
  Map<String, LiveBinding> bindings = const {};

  /// Things the host draws itself (its effect snapshots), and those among them the reference also has a face for.
  final snapshots = <String>{}, fixtureFallback = <String>{};

  void _rebind() {
    _bind();
    notifyListeners();
  }

  void _bind() {
    final out = <String, LiveBinding>{};
    final kinds = {for (final k in EditorSession.maps(c.state['createKinds'])) '${k['id']}'};
    _createKinds.forEach((tile, kind) {
      if (kinds.contains(kind)) out[tile] = ('create', {'kind': kind});
    });
    snapshots.clear();
    fixtureFallback.clear();
    _pictures.clear();
    final byId = {for (final (_, t) in _reference.files) if (t['kind'] == 'effect') '${t['id']}': '${t['id']}'};
    final byName = {for (final (_, t) in _reference.files) if (t['kind'] == 'effect') '${t['name']}'.toLowerCase(): '${t['id']}'};
    final extra = <(String, Map<String, dynamic>)>[];
    for (final e in EditorSession.maps(c.state['catalog'])) {
      final id = '${e['id']}', name = '${e['name'] ?? id}';
      final tile = byId[id] ?? byName[name.toLowerCase()];
      if (tile != null) {
        out[tile] = ('applyEffect', {'pluginIds': [id]});
        snapshots.add(id);
        fixtureFallback.add(tile);
        continue;
      }
      out[id] = ('applyEffect', {'pluginIds': [id]});
      snapshots.add(id);
      extra.add(('host', {'id': id, 'name': name, 'kind': 'effect', 'family': 'host', 'source': 'host', 'face': {'type': 'fx', 'base': name, 'hue': 0}}));
    }
    bindings = out;
    catalog = extra.isEmpty ? _reference : Catalog(_registryWithHostFamily(), [...extra, ..._reference.files]);
  }

  Registry _registryWithHostFamily() => Registry.fromJson({
        'kinds': {for (final e in _reference.registry.kinds.entries) e.key: {'label': e.value}},
        'tags': _reference.registry.tags.toList(),
        'capabilities': _reference.registry.capabilities,
        'faces': _reference.registry.faces,
        'panels': {for (final e in _reference.registry.panels.entries) e.key: {'title': e.value.title, 'kinds': e.value.kinds}},
        'families': {
          for (final e in _reference.registry.families.entries)
            e.key: {'label': e.value.label, if (e.value.section != null) 'section': e.value.section, 'kinds': e.value.kinds.toList()},
          'host': {'label': 'Host', 'kinds': ['effect']},
        },
      });

  final Map<String, Future<Uint8List?>> _pictures = {};

  /// The host's own picture of an effect (null: it ships none).
  Future<Uint8List?> picture(String id) => _pictures[id] ??= () async {
        try {
          final reply = await c.native('request', {'command': jsonEncode({'op': 'visualSample', 'kind': 'effect', 'id': id})});
          final data = EditorSession.map(reply)['image'];
          return data is String ? base64Decode(data) : null;
        } catch (_) {
          return null;
        }
      }();

  // ---- the user's views, per shelf --------------------------------------------------------------------------------
  final _users = <String, LiveBrowserUser>{};
  LiveBrowserUser user(String shelf) => _users.putIfAbsent(shelf, () => LiveBrowserUser(c, shelf)..addListener(notifyListeners));

  /// The thing last picked, per shelf.
  final picked = <String, String>{};

  /// Use a thing: it becomes recent on its shelf and does what it does on the host.
  Future<void> use(String shelf, String id) async {
    final b = bindings[id];
    picked[shelf] = id;
    notifyListeners();
    if (b == null) return;
    await user(shelf).used(id);
    await c.command(b.$1, b.$2);
  }

  bool get canReloadEffects => c.supports('reloadEffects');
  void reloadEffects() => c.command('reloadEffects');

  // ---- colours -----------------------------------------------------------------------------------------------------
  List<Map<String, dynamic>> get savedSwatches => EditorSession.maps(c.deskWork.value['swatches']);
  List<Map<String, dynamic>> get gradients => [for (final item in savedSwatches) if ((item['stops'] as List? ?? const []).length > 1) item];

  List<Sw> get colors => [
        for (final swatch in EditorSession.maps(c.state['palette']))
          ('${swatch['hex'] ?? swatch['id'] ?? 'Color'}', _argb(swatch['rgba']), swatch['used'] == true ? 'Used Here' : 'Starter'),
        for (final (index, saved) in savedSwatches.indexed)
          if ((saved['stops'] as List? ?? const []).length == 1) ('saved:$index', _argb((saved['stops'] as List).first), 'Saved'),
      ];

  static int _argb(dynamic rgba) {
    final values = rgba is List ? rgba : const [0.0, 0.0, 0.0, 1.0];
    double at(int i, double fallback) => i < values.length && values[i] is num ? (values[i] as num).toDouble() : fallback;
    return Color.fromARGB((at(3, 1) * 255).round().clamp(0, 255), (at(0, 0) * 255).round().clamp(0, 255), (at(1, 0) * 255).round().clamp(0, 255), (at(2, 0) * 255).round().clamp(0, 255)).toARGB32();
  }

  /// The colour being edited (the colour target), if any.
  Color? get currentColor {
    final target = colorTarget(c);
    return target == null ? null : colorOf(rgbaOf(target['rgba']));
  }

  bool get canSaveColor => colorTarget(c) != null || EditorSession.map(c.activeLayer?['fill']).isNotEmpty;
  void saveCurrentColor() => saveCurrentSwatch(c);
  void paletteFromImage() => paletteFromImages(c);
  String get wheelShape => c.deskWork.value['colorShape'] == 'triangle' ? 'triangle' : 'square';
  void setWheelShape(String shape) => c.storeDesk('colorShape', shape);

  Future<void> applyColor(Sw swatch) async {
    if (!c.supports('applyPalette')) return;
    if (swatch.$1.startsWith('saved:')) {
      final index = int.tryParse(swatch.$1.substring(6));
      final saved = savedSwatches;
      if (index != null && index >= 0 && index < saved.length) {
        final stops = saved[index]['stops'] as List? ?? const [];
        if (stops.length == 1) await c.command('applyPalette', {'rgba': stops.single});
        return;
      }
    }
    final color = Color(swatch.$2);
    await c.command('applyPalette', {'rgba': [color.r, color.g, color.b, color.a]});
  }

  /// The saved swatch [item] (as it was when chosen: the list may have changed since), forgotten.
  Future<void> forget(Map<String, dynamic> item) async {
    final key = jsonEncode(item);
    final at = savedSwatches.indexWhere((s) => jsonEncode(s) == key);
    if (at >= 0) await forgetSwatch(c, at);
  }

  Future<void> applyGradient(Map<String, dynamic> item) async {
    final fill = EditorSession.map(c.activeLayer?['fill']);
    final target = EditorSession.map(c.state['colorTarget']);
    final slot = target['slot'] ?? fill['slot'];
    if (slot == null || !c.supports('setGradient')) return;
    await c.command('setGradient', {'slot': slot, 'stops': item['stops'], if (item['blend'] != null) 'blend': item['blend']});
  }

  // ---- fonts -------------------------------------------------------------------------------------------------------
  Map<String, Map<String, dynamic>> _fontFacts = const {};
  Future<void> _loadFontFacts() async {
    try {
      final reply = await c.native('request', {'command': jsonEncode({'op': 'fontFacts'})});
      _fontFacts = {for (final fact in EditorSession.maps(EditorSession.map(reply)['facts'])) '${fact['family']}': fact};
    } catch (_) {
      _fontFacts = const {};
    }
    notifyListeners();
  }

  /// The text layer a font dresses (null: a font makes a new text layer).
  Map<String, dynamic>? get dressing => c.activeLayer != null && c.activeLayer!['kind'] == 'Text' ? c.activeLayer : null;
  String? get dressingFont => EditorSession.map(dressing?['text'])['fontFamily'] as String?;

  List<FontItem> get fonts {
    final views = user('Fonts').views;
    return [
      for (final family in (c.state['fontFamilies'] as List? ?? const []).whereType<String>())
        if (_fontFacts[family] case final facts?)
          FontItem(
            family,
            facts['monospaced'] == true
                ? 'Mono'
                : (facts['scripts'] as List? ?? const []).contains('Kana')
                    ? 'JP'
                    : (facts['scripts'] as List? ?? const []).contains('Hangul')
                        ? 'KR'
                        : 'All',
            (facts['styles'] as num? ?? 1).toInt(),
            sample: _sampleGlyph(facts),
            facts: [
              ...(facts['scripts'] as List? ?? const []).cast<String>(),
              ...(facts['axes'] as List? ?? const []).cast<String>(),
              if (facts['monospaced'] == true) 'Mono',
              if (facts['color'] == true) 'Color',
            ],
            favorite: views.favorites.contains('font:$family'),
          )
        else
          FontItem(family, 'All', 1, favorite: views.favorites.contains('font:$family')),
    ];
  }

  List<List<String>> fontGroups(List<FontItem> fonts) {
    final available = {for (final font in fonts) font.cls};
    const order = ['Sans', 'Serif', 'Display', 'Mono', 'Hand', 'JP', 'KR'];
    return [
      ['All', for (final group in order) if (available.contains(group)) group],
      ['Used Here', 'Favorites', 'Installed'],
    ];
  }

  /// The families the document's text layers use.
  Set<String> get usedFonts => {for (final l in c.layers) if (EditorSession.map(l['text'])['fontFamily'] case final String f) f};

  static String _sampleGlyph(Map<String, dynamic> facts) {
    final scripts = (facts['scripts'] as List? ?? const []).cast<String>();
    if (scripts.contains('Kanji')) return '漢';
    if (scripts.contains('Kana')) return 'あ';
    if (scripts.contains('Hangul')) return '가';
    if (scripts.contains('Arabic')) return 'ع';
    if (scripts.contains('Cyrillic')) return 'Я';
    return 'Aa';
  }

  void toggleFavoriteFont(FontItem font) {
    final u = user('Fonts');
    u.collect(['font:${font.family}'], u.collectionOf('font:${font.family}') == 1 ? 0 : 1);
  }

  /// A font: dress the text layer picked (the text style's own scope when one is set), or make a new text layer with
  /// it when [create] and none is picked.
  Future<void> applyFont(FontItem font, {required bool create}) async {
    final text = dressing;
    if (create) {
      if (text == null && c.supports('create')) await c.command('create', {'kind': 'text', 'family': font.family});
      await user('Fonts').used('font:${font.family}');
      return;
    }
    if (text == null || !c.supports('setFont') || text['locked'] == true) return;
    final target = c.textStyleTarget.value;
    await c.command('setFont', {
      ...(target != null && target['layer'] == text['id'] ? target : {'scope': 'all'}),
      'layer': text['id'],
      'family': font.family,
    });
    await user('Fonts').used('font:${font.family}');
  }
}
