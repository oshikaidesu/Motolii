import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter/widgets.dart';

import '../../hf/bp/browser_face.dart';
import '../../hf/bp/colors.dart' show Sw;
import '../../hf/bp/fonts.dart' show FontItem;
import '../../hf/bp/catalog_io.dart';
import '../../hf/bp/effects.dart';
import '../../hf/shell/menu.dart' show showHfMenu;
import '../../hf/bp/seat.dart';
import '../../hf/bp/things.dart';
import '../../session/editor_session.dart';
import 'browser_shelf.dart';
import 'browser_user.dart';

/// The reference's own catalogue: every tile, its family, face and order. The Browser draws this; Live only says which
/// tile does what.
const _thingsDir = String.fromEnvironment(
  'MOTOLII_THINGS',
  defaultValue: '/Users/member_ottoto/rust_ae/Motolii/motolii/ui/lib/proto_hf/data/things',
);

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

/// What one tile does on the host: an op and its arguments.
typedef LiveBinding = (String op, Map<String, dynamic> args);

/// The Browser face over the session. Create: the reference tiles, each bound to a `create` kind when the host has
/// it. Effects: the reference tiles bound by name, and every other effect the host has (placement copies included),
/// drawn by the host's own snapshot. A tile with no binding keeps its face and does nothing.
class LiveBrowser extends StatefulWidget {
  const LiveBrowser({super.key, required this.c, required this.scene});
  final EditorSession c;
  final EffectScene scene;
  @override
  State<LiveBrowser> createState() => _LiveBrowserState();
}

class _LiveBrowserState extends State<LiveBrowser> {
  static const _catalogWatched = ['createKinds', 'catalog'];
  static const _surfaceWatched = [
    'palette',
    'fontFamilies',
    'assets',
    'backgrounds',
    'layers',
    'selectedId',
    'selectedIds',
    'documentRevision',
    'colorTarget',
  ];
  static final Catalog _reference = loadCatalog(_thingsDir);
  EditorSession get c => widget.c;
  int tab = 0;
  late final seat = _LiveSeat(c, widget.scene);
  final _users = <String, LiveBrowserUser>{};
  late Catalog catalog;
  Map<String, Map<String, dynamic>> fontFacts = const {};
  Map<String, dynamic>? get _selectedText =>
      c.activeLayer != null && c.activeLayer!['kind'] == 'Text'
      ? c.activeLayer
      : null;
  String? get _selectedFont =>
      EditorSession.map(_selectedText?['text'])['fontFamily'] as String?;

  @override
  void initState() {
    super.initState();
    _bind();
    c.slice('liveBrowserCatalog', _catalogWatched).addListener(_catalogChanged);
    c.slice('liveBrowserSurface', _surfaceWatched).addListener(_absorb);
    _loadFontFacts();
  }

  Future<void> _loadFontFacts() async {
    try {
      final reply = await c.native('request', {
        'command': jsonEncode({'op': 'fontFacts'}),
      });
      fontFacts = {
        for (final fact in EditorSession.maps(
          EditorSession.map(reply)['facts'],
        ))
          '${fact['family']}': fact,
      };
    } catch (_) {
      fontFacts = const {};
    }
    if (mounted) setState(() {});
  }

  void _catalogChanged() => setState(_bind);
  void _absorb() => setState(() {});
  void _userChanged() {
    if (mounted) setState(() {});
  }

  void _bind() {
    final bindings = <String, LiveBinding>{};
    final kinds = {
      for (final k in EditorSession.maps(c.state['createKinds'])) '${k['id']}',
    };
    _createKinds.forEach((tile, kind) {
      if (kinds.contains(kind)) bindings[tile] = ('create', {'kind': kind});
    });
    seat.snapshots.clear();
    seat.fixtureFallback.clear();
    final byId = {
      for (final (_, t) in _reference.files)
        if (t['kind'] == 'effect') '${t['id']}': '${t['id']}',
    };
    final byName = {
      for (final (_, t) in _reference.files)
        if (t['kind'] == 'effect') '${t['name']}'.toLowerCase(): '${t['id']}',
    };
    final extra = <(String, Map<String, dynamic>)>[];
    for (final e in EditorSession.maps(c.state['catalog'])) {
      final id = '${e['id']}', name = '${e['name'] ?? id}';
      final tile = byId[id] ?? byName[name.toLowerCase()];
      if (tile != null) {
        bindings[tile] = ('applyEffect', {'pluginId': id});
        seat.snapshots.add(id);
        seat.fixtureFallback.add(tile);
        continue;
      }
      bindings[id] = ('applyEffect', {'pluginId': id});
      seat.snapshots.add(id);
      extra.add((
        'host',
        {
          'id': id,
          'name': name,
          'kind': 'effect',
          'family': 'host',
          'source': 'host',
          'face': {'type': 'fx', 'base': name, 'hue': 0},
        },
      ));
    }
    seat.bindings = bindings;
    seat._pictures.clear();
    catalog = extra.isEmpty
        ? _reference
        : Catalog(_registryWithHostFamily(), [..._reference.files, ...extra]);
    seat.refresh();
  }

  Registry _registryWithHostFamily() => Registry.fromJson({
    'kinds': {
      for (final e in _reference.registry.kinds.entries)
        e.key: {'label': e.value},
    },
    'tags': _reference.registry.tags.toList(),
    'capabilities': _reference.registry.capabilities,
    'faces': _reference.registry.faces,
    'panels': {
      for (final e in _reference.registry.panels.entries)
        e.key: {'title': e.value.title, 'kinds': e.value.kinds},
    },
    'families': {
      for (final e in _reference.registry.families.entries)
        e.key: {
          'label': e.value.label,
          if (e.value.section != null) 'section': e.value.section,
          'kinds': e.value.kinds.toList(),
        },
      'host': {
        'label': 'Host',
        'kinds': ['effect'],
      },
    },
  });

  LiveBrowserUser _user(String name) => _users.putIfAbsent(name, () {
    final user = LiveBrowserUser(c, name)..addListener(_userChanged);
    return user;
  });

  @override
  void dispose() {
    c
        .slice('liveBrowserCatalog', _catalogWatched)
        .removeListener(_catalogChanged);
    c.slice('liveBrowserSurface', _surfaceWatched).removeListener(_absorb);
    seat.dispose();
    for (final user in _users.values) {
      user.removeListener(_userChanged);
      user.dispose();
    }
    super.dispose();
  }

  List<Sw> _colors() => [
    for (final swatch in EditorSession.maps(c.state['palette']))
      (
        '${swatch['hex'] ?? swatch['id'] ?? 'Color'}',
        _argb(swatch['rgba']),
        swatch['used'] == true ? 'Used Here' : 'Saved',
      ),
    for (final (index, saved) in _savedSwatches().indexed)
      if ((saved['stops'] as List? ?? const []).length == 1)
        ('saved:$index', _argb((saved['stops'] as List).first), 'Saved'),
  ];

  List<Map<String, dynamic>> _savedSwatches() =>
      EditorSession.maps(c.deskWork.value['swatches']);

  int _argb(dynamic rgba) {
    final values = rgba is List ? rgba : const [0.0, 0.0, 0.0, 1.0];
    double at(int i, double fallback) => i < values.length && values[i] is num
        ? (values[i] as num).toDouble()
        : fallback;
    return Color.fromARGB(
      (at(3, 1) * 255).round().clamp(0, 255),
      (at(0, 0) * 255).round().clamp(0, 255),
      (at(1, 0) * 255).round().clamp(0, 255),
      (at(2, 0) * 255).round().clamp(0, 255),
    ).toARGB32();
  }

  List<FontItem> _fonts() {
    final user = _user('Fonts').views;
    return [
      for (final family
          in (c.state['fontFamilies'] as List? ?? const []).whereType<String>())
        if (fontFacts[family] case final facts?)
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
            favorite: user.favorites.contains('font:$family'),
          )
        else
          FontItem(
            family,
            'All',
            1,
            favorite: user.favorites.contains('font:$family'),
          ),
    ];
  }

  List<List<String>> _fontGroups(List<FontItem> fonts) {
    final available = {for (final font in fonts) font.cls};
    const order = ['Sans', 'Serif', 'Display', 'Mono', 'Hand', 'JP', 'KR'];
    return [
      [
        'All',
        for (final group in order)
          if (available.contains(group)) group,
      ],
      ['Favorites', 'Installed'],
    ];
  }

  String _sampleGlyph(Map<String, dynamic> facts) {
    final scripts = (facts['scripts'] as List? ?? const []).cast<String>();
    if (scripts.contains('Kanji')) return '漢';
    if (scripts.contains('Kana')) return 'あ';
    if (scripts.contains('Hangul')) return '가';
    if (scripts.contains('Arabic')) return 'ع';
    if (scripts.contains('Cyrillic')) return 'Я';
    return 'Aa';
  }

  @override
  Widget build(BuildContext context) {
    final name = const ['Create', 'Effects', 'Colors', 'Fonts', 'Media'][tab];
    final user = _user(name);
    seat.userSource = user;
    seat.recentlyUsed = user.used;
    seat.collect = user.collect;
    final fonts = _fonts();
    final fontGroups = _fontGroups(fonts);
    return BrowserSeatScope(
      seat: seat,
      child: browserFace(
        BrowserModel(
          catalog: catalog,
          tab: tab,
          user: user.views,
          scene: widget.scene,
          colors: _colors(),
          gradients: [
            for (final item in _savedSwatches())
              if ((item['stops'] as List? ?? const []).length > 1) item,
          ],
          onColor: (swatch) => _applyColor(swatch),
          onGradient: (gradient) => _applyGradient(gradient),
          colorEditor: null,
          fonts: fonts,
          fontGroups: fontGroups,
          selectedFont: _selectedFont,
          onFont: (font) => _applyFont(font, create: false),
          onFontCreate: (font) =>
              _applyFont(font, create: _selectedText == null),
          onFontFavorite: (font) => _user('Fonts').collect([
            'font:${font.family}',
          ], _user('Fonts').collectionOf('font:${font.family}') == 1 ? 0 : 1),
          fontController: c,
          media: tab == 4
              ? LiveBrowserShelf(
                  controller: c,
                  name: 'Media',
                  user: _user('Media'),
                )
              : null,
          onTab: (i) => setState(() => tab = i),
        ),
      ),
    );
  }

  Future<void> _applyColor(Sw swatch) async {
    if (!c.supports('applyPalette')) return;
    if (swatch.$1.startsWith('saved:')) {
      final index = int.tryParse(swatch.$1.substring(6));
      final saved = _savedSwatches();
      if (index != null && index >= 0 && index < saved.length) {
        final stops = saved[index]['stops'] as List? ?? const [];
        if (stops.length == 1)
          await c.command('applyPalette', {'rgba': stops.single});
        return;
      }
    }
    final color = Color(swatch.$2);
    await c.command('applyPalette', {
      'rgba': [color.r, color.g, color.b, color.a],
    });
  }

  Future<void> _applyGradient(Map<String, dynamic> item) async {
    final fill = EditorSession.map(c.activeLayer?['fill']);
    final target = EditorSession.map(c.state['colorTarget']);
    final slot = target['slot'] ?? fill['slot'];
    if (slot == null || !c.supports('setGradient')) return;
    await c.command('setGradient', {
      'slot': slot,
      'stops': item['stops'],
      if (item['blend'] != null) 'blend': item['blend'],
    });
  }

  Future<void> _applyFont(FontItem font, {required bool create}) async {
    if (create) {
      if (_selectedText == null && c.supports('create'))
        await c.command('create', {'kind': 'text', 'family': font.family});
      await _user('Fonts').used('font:${font.family}');
      return;
    }
    if (_selectedText == null) return;
    if (!c.supports('setFont') || _selectedText?['locked'] == true) return;
    final target = c.textStyleTarget.value;
    await c.command('setFont', {
      ...(target != null && target['layer'] == _selectedText?['id']
          ? target
          : {'scope': 'all'}),
      'layer': _selectedText?['id'],
      'family': font.family,
    });
    await _user('Fonts').used('font:${font.family}');
  }
}

class _LiveSeat extends ChangeNotifier implements BrowserSeat {
  _LiveSeat(this.c, this.scene);
  final EditorSession c;
  final EffectScene scene;
  Map<String, LiveBinding> bindings = const {};
  LiveBrowserUser? _userSource;
  set userSource(LiveBrowserUser? value) {
    if (identical(value, _userSource)) return;
    _userSource?.removeListener(_userChanged);
    _userSource = value;
    _userSource?.addListener(_userChanged);
    notifyListeners();
  }

  void _userChanged() => notifyListeners();
  @override
  UserViews? get user => _userSource?.views;
  Future<void> Function(String id)? recentlyUsed;
  Future<void> Function(Iterable<String> ids, int collection)? collect;
  final _things = <Thing>[];
  String? _selectedId;
  void refresh() => notifyListeners();

  /// Effects the reference has no tile for: drawn by the host's own snapshot.
  final Set<String> snapshots = {};
  final Set<String> fixtureFallback = {};
  final Map<String, Future<Uint8List?>> _pictures = {};

  Future<Uint8List?> _picture(String id) => _pictures[id] ??= () async {
    try {
      final reply = await c.native('request', {
        'command': jsonEncode({
          'op': 'visualSample',
          'kind': 'effect',
          'id': id,
        }),
      });
      final data = EditorSession.map(reply)['image'];
      return data is String ? base64Decode(data) : null;
    } catch (_) {
      return null;
    }
  }();

  @override
  Widget tile(BuildContext context, Thing thing, Widget tile) {
    final b = bindings[thing.id];
    if (b == null) return Opacity(opacity: .48, child: tile);
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: () async {
          _selectedId = thing.id;
          await recentlyUsed?.call(thing.id);
          await c.command(b.$1, b.$2);
          notifyListeners();
        },
        onSecondaryTapDown: (event) {
          _selectedId = thing.id;
          more(context, event.globalPosition);
        },
        child: tile,
      ),
    );
  }

  @override
  Widget? face(BuildContext context, Thing thing) {
    if (!snapshots.contains(thing.id)) return null;
    return FutureBuilder<Uint8List?>(
      future: _picture(thing.id),
      builder: (_, s) => s.data == null
          ? fixtureFallback.contains(thing.id)
                ? CustomPaint(
                    painter: FxPainter('${thing.face['base']}', scene),
                  )
                : const Center(
                    child: Text(
                      'Preview unavailable',
                      style: TextStyle(color: Color(0xFFB0B0B2), fontSize: 10),
                    ),
                  )
          : Image.memory(s.data!, fit: BoxFit.cover, gaplessPlayback: true),
    );
  }

  @override
  ({double column, double extent, double gap, double padding})? tiling(
    BuildContext context,
    double width,
  ) => null;
  @override
  double get tileScale => 1;
  @override
  Widget? tools(BuildContext context) => null;
  @override
  Widget? header(BuildContext context) => null;
  @override
  Widget? editor(BuildContext context) => null;
  @override
  KeyEventResult key(FocusNode node, KeyEvent event) => KeyEventResult.ignored;
  @override
  void shows(List<Thing> things, int columns) {
    _things
      ..clear()
      ..addAll(things);
  }

  @override
  void more(BuildContext context, Offset at) {
    final id = _selectedId;
    if (id == null || _userSource == null) return;
    final selected = _userSource!.collectionOf(id) == 1;
    showHfMenu<String>(context, Rect.fromLTWH(at.dx, at.dy, 200, 0), [
      (
        selected ? 'remove' : 'favorite',
        selected ? 'Remove from Favorites' : 'Add to Favorites',
      ),
    ]).then((action) {
      if (action == null) return;
      collect?.call([id], action == 'remove' ? 0 : 1);
    });
  }

  @override
  Listenable get changes => this;

  @override
  void dispose() {
    _userSource?.removeListener(_userChanged);
    super.dispose();
  }
}
