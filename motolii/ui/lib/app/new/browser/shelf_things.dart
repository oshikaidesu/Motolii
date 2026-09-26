import '../../../hf/bp/things.dart';
import '../../../panels/browser/files_shelf.dart' show FilesShelf;
import '../../../panels/browser/shelf.dart' show FilterKind;
import 'shelf_host.dart';
import 'shelf_user.dart';

/// The shelf's items as the finished Browser's Things. The shelf stays the owner: an item is a row it lists; a Thing is
/// how that row is described to search, classes and views (name, family, tags), plus which face draws it.
class ShelfCatalog {
  ShelfCatalog(this.host, this.panelId, {this.faceOf, this.own}) {
    final shelf = host.shelf;
    // Files' rail is places to walk to, not classes of what is listed.
    final rails = shelf is FilesShelf ? const <String>[] : shelf.rails(host);
    final families = <String, Map<String, dynamic>>{};
    String family(String label) {
      final slug = label.toLowerCase().replaceAll(RegExp(r'[^a-z0-9]+'), '_');
      families.putIfAbsent(slug, () => {'label': label, 'kinds': [_kind]});
      return slug;
    }

    // rail order first, so the class column lists them as the shelf does
    for (final rail in rails.skip(1)) {
      family(rail);
    }
    final tags = <String>{};
    final groups = shelf.groups(host);
    final files = <(String, Map<String, dynamic>)>[];
    for (final item in host.items()) {
      final id = host.id(item);
      byId[id] = item;
      // What the item is (the shelf's declared tags and the values its groups read off it) and what the user called it.
      final itemTags = [
        for (final t in shelf.tagsOf(host, item)) _slug(t),
        for (final g in groups)
          if (g.kind == FilterKind.actual)
            if (shelf.valueOf(host, item, g.name) case final v?) _slug(v),
        if (own != null) for (final t in own!.library.tagsOf(shelf.name, id)) _slug(t),
      ];
      tags.addAll(itemTags);
      final name = '${item['name'] ?? item['hex'] ?? item['id']}';
      files.add((
        shelf.name,
        {
          'id': id,
          'name': name,
          'kind': _kind,
          'family': family(shelf.classification(host, item)),
          'tags': itemTags,
          'capabilities': <String>[],
          'source': 'builtin',
          'searchTerms': [
            if (item['detail'] != null) ...'${item['detail']}'.toLowerCase().split(RegExp(r'\s+')).where((w) => w.length > 1),
          ],
          'face': faceOf?.call(item, name) ?? {'type': 'shelf'},
        },
      ));
    }
    registry = Registry.fromJson({
      'kinds': {_kind: {'label': shelf.name}},
      'tags': tags.toList(),
      'capabilities': <String, String>{},
      'faces': {'shelf': <String, dynamic>{}, 'mark': <String, dynamic>{}},
      'panels': {panelId: {'title': shelf.name, 'kinds': [_kind]}},
      'families': families,
    });
    catalog = Catalog(registry, files);
  }

  final ShelfHost host;
  final String panelId;
  final ShelfUser? own;

  /// A declared face for an item (a mark the design already draws), or null to have the host draw it.
  final Map<String, dynamic>? Function(Map<String, dynamic> item, String name)? faceOf;
  final byId = <String, Map<String, dynamic>>{};
  late final Registry registry;
  late final Catalog catalog;

  String get _kind => host.shelf.name.toLowerCase();
  static String _slug(String t) => t.toLowerCase().replaceAll(RegExp(r'\s+'), '-');

  /// Changes when what the shelf lists changes: the panel rebuilds its views then.
  String get signature => '${catalog.files.length}:${catalog.files.map((f) => '${f.$2['id']}${f.$2['family']}${f.$2['tags']}').join(',').hashCode}';
}

/// The marks the Create body draws, by the name the shelf gives the thing. A name not here is drawn by the shelf.
const createMarks = <String, (String, String)>{
  'Text': ('text', '#FFCB3D'),
  'Rectangle': ('rect', '#6C7CFF'),
  'Rounded Rectangle': ('rounded', '#6C7CFF'),
  'Ellipse': ('ellipse', '#6C7CFF'),
  'Star': ('star', '#6C7CFF'),
  'Polygon': ('polygon', '#6C7CFF'),
  'Line': ('line', '#5E7BFF'),
  'Bezier': ('path', '#5E7BFF'),
  'Null': ('nul', '#B7B9BD'),
  'Camera': ('camera', '#B7B9BD'),
  'Light': ('light', '#FFCB3D'),
  'Particles': ('particles', '#B7B9BD'),
  'Stage': ('stage', '#B7B9BD'),
  'Cube': ('cube', '#7A66F0'),
  'Sphere': ('sphere', '#6C7CFF'),
  'Torus': ('torus', '#7A66F0'),
  'Cylinder': ('cylinder', '#6C7CFF'),
  'Cone': ('cone', '#6C7CFF'),
  'Pyramid': ('pyramid', '#7A66F0'),
  'Plane': ('plane', '#7A66F0'),
};

Map<String, dynamic>? createFace(Map<String, dynamic> item, String name) {
  final m = createMarks[name];
  return m == null ? null : {'type': 'mark', 'mark': m.$1, 'color': m.$2};
}
