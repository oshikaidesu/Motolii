import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter/widgets.dart';

import '../../../hf/bp/browser_face.dart';
import '../../../hf/bp/catalog_io.dart';
import '../../../hf/bp/effects.dart';
import '../../../hf/bp/seat.dart';
import '../../../hf/bp/things.dart';
import '../../../session/editor_session.dart';

/// The reference's own catalogue: every tile, its family, face and order. The Browser draws this; Live only says which
/// tile does what.
const _thingsDir = String.fromEnvironment('MOTOLII_THINGS', defaultValue: '/Users/member_ottoto/rust_ae/Motolii/motolii/ui/lib/proto_hf/data/things');

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
  static const _watched = ['createKinds', 'catalog'];
  static final Catalog _reference = loadCatalog(_thingsDir);
  EditorSession get c => widget.c;
  int tab = 0;
  late final seat = _LiveSeat(c);
  late Catalog catalog;

  @override
  void initState() {
    super.initState();
    _bind();
    c.slice('liveBrowser', _watched).addListener(_absorb);
  }

  void _absorb() => setState(_bind);

  void _bind() {
    final bindings = <String, LiveBinding>{};
    final kinds = {for (final k in EditorSession.maps(c.state['createKinds'])) '${k['id']}'};
    _createKinds.forEach((tile, kind) {
      if (kinds.contains(kind)) bindings[tile] = ('create', {'kind': kind});
    });
    final byName = {for (final (_, t) in _reference.files) if (t['kind'] == 'effect') '${t['name']}'.toLowerCase(): '${t['id']}'};
    final extra = <(String, Map<String, dynamic>)>[];
    for (final e in EditorSession.maps(c.state['catalog'])) {
      final id = '${e['id']}', name = '${e['name'] ?? id}';
      final tile = byName[name.toLowerCase()];
      if (tile != null) {
        bindings[tile] = ('applyEffect', {'pluginId': id});
        continue;
      }
      bindings[id] = ('applyEffect', {'pluginId': id});
      seat.snapshots.add(id);
      extra.add(('host', {
        'id': id,
        'name': name,
        'kind': 'effect',
        'family': '${e['stage'] ?? ''}',
        'source': 'host',
        'face': const {'type': 'fx', 'base': 'Blur', 'hue': 0},
      }));
    }
    seat.bindings = bindings;
    catalog = extra.isEmpty ? _reference : Catalog(_reference.registry, [..._reference.files, ...extra]);
  }

  @override
  void dispose() {
    c.slice('liveBrowser', _watched).removeListener(_absorb);
    seat.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => BrowserSeatScope(
        seat: seat,
        child: browserFace(BrowserModel(catalog: catalog, tab: tab, scene: widget.scene, onTab: (i) => setState(() => tab = i))),
      );
}

class _LiveSeat extends ChangeNotifier implements BrowserSeat {
  _LiveSeat(this.c);
  final EditorSession c;
  Map<String, LiveBinding> bindings = const {};

  /// Effects the reference has no tile for: drawn by the host's own snapshot.
  final Set<String> snapshots = {};
  final Map<String, Future<Uint8List?>> _pictures = {};

  Future<Uint8List?> _picture(String id) => _pictures[id] ??= () async {
        try {
          final reply = await c.native('request', {
            'command': jsonEncode({'op': 'visualSample', 'kind': 'effect', 'id': id}),
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
    if (b == null) return tile;
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      child: GestureDetector(behavior: HitTestBehavior.opaque, onTap: () => c.command(b.$1, b.$2), child: tile),
    );
  }

  @override
  Widget? face(BuildContext context, Thing thing) {
    if (!snapshots.contains(thing.id)) return null;
    return FutureBuilder<Uint8List?>(
      future: _picture(thing.id),
      builder: (_, s) => s.data == null ? const SizedBox.expand() : Image.memory(s.data!, fit: BoxFit.cover, gaplessPlayback: true),
    );
  }

  @override
  ({double column, double extent, double gap, double padding})? tiling(BuildContext context, double width) => null;
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
  void shows(List<Thing> things, int columns) {}
  @override
  void more(BuildContext context, Offset at) {}
  @override
  UserViews? get user => null;
  @override
  Listenable get changes => this;
}
