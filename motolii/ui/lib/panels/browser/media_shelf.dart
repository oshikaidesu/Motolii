import 'dart:io';

import 'package:flutter/widgets.dart';
import 'package:flutter/services.dart';

import '../../foundation/metrics.dart';
import '../../foundation/theme.dart';
import '../../session/editor_session.dart';
import '../../session/media_actions.dart';

export '../../session/media_actions.dart'
    show filePath, revealLabel, mediaFamily, mediaFormat, fileFact, homely;
import 'create_shelf.dart';
import 'parts.dart';
import 'shelf.dart';
import '../../foundation/glyphs.dart';

/// Media: what the document has taken in, plus the bundled HDRIs. A
/// double-click places one; a card drags onto the Timeline.
class MediaShelf extends BrowserShelf {
  @override
  String get name => 'Media';
  @override
  bool get multiSelect => true;

  /// Media opens on pictures alone: its items are told apart by their
  /// picture, not their name.
  @override
  int get defaultView => 2;

  @override
  List<String> rails(BrowserHost host) => const [
    'All',
    'Video',
    'Images',
    'HDR',
    'Audio',
    '3D',
  ];

  @override
  List<Map<String, dynamic>> items(BrowserHost host) => [
    ...EditorSession.maps(host.controller.state['assets']),
    // 同梱の HDRI は素材の棚に、取り込んだ物と同じ札で並ぶ(Finder・置換・削除は無い)。
    for (final b in EditorSession.maps(host.controller.state['backgrounds']))
      {
        ...b,
        'id': 'background:${b['id']}',
        'mime': 'image/hdr',
        'builtin': true,
        'detail': 'HDR · bundled (Poly Haven, CC0)',
      },
  ];

  @override
  String classification(BrowserHost host, Map<String, dynamic> item) {
    final kind = mediaFamily(item);
    return kind == '2D' ? 'Images' : kind;
  }

  /// What the file itself is: its kind and where it came from are declared;
  /// resolution and frame rate are the values the files actually have (a
  /// handful); duration is the one continuous fact, cut into ranges by the
  /// user (the seeds are only a start).
  @override
  List<FilterGroup> groups(BrowserHost host) => const [
    FilterGroup('Kind', ['Video', 'Image', 'HDR', 'Audio', '3D']),
    FilterGroup('Resolution', [], kind: FilterKind.actual),
    FilterGroup('Frame rate', [], kind: FilterKind.actual, unit: 'fps'),
    FilterGroup(
      'Duration',
      ['-5', '5-30', '30-'],
      kind: FilterKind.range,
      unit: 's',
    ),
    FilterGroup('Source', ['Imported', 'Bundled', 'Reference', 'Material']),
  ];

  @override
  Set<String> tagsOf(BrowserHost host, Map<String, dynamic> item) {
    final kind = mediaFamily(item);
    return {
      kind == '2D' ? 'Image' : kind,
      item['builtin'] == true ? 'Bundled' : 'Imported',
      item['role'] == 'reference' ? 'Reference' : 'Material',
    };
  }

  @override
  String? valueOf(BrowserHost host, Map<String, dynamic> item, String group) {
    final facts = EditorSession.map(item['facts']);
    switch (group) {
      case 'Resolution':
        final w = facts['width'], h = facts['height'];
        return w is num && h is num ? '${w.toInt()}×${h.toInt()}' : null;
      case 'Frame rate':
        final fps = facts['fps'];
        if (fps is! num) return null;
        final r = (fps * 1000).round() / 1000;
        return r == r.roundToDouble() ? '${r.toInt()}' : '$r';
    }
    return null;
  }

  @override
  double? numberOf(BrowserHost host, Map<String, dynamic> item, String group) {
    if (group != 'Duration') return null;
    final s = EditorSession.map(item['facts'])['seconds'] ?? item['seconds'];
    return s is num ? s.toDouble() : null;
  }

  @override
  bool supported(BrowserHost host, Map<String, dynamic> item) =>
      item['builtin'] == true
      ? host.has('create')
      : host.has('placeAsset') && item['missing'] != true;

  @override
  String identity(BrowserHost host, Map<String, dynamic> item) =>
      mediaFamily(item);
  @override
  String format(BrowserHost host, Map<String, dynamic> item) =>
      mediaFormat(item);

  @override
  Widget preview(BrowserHost host, Map<String, dynamic> item, Color identity) =>
      mediaThumbnail(item, host.tileScale);

  @override
  Future<void> apply(BrowserHost host, Map<String, dynamic> item) async {
    final c = host.controller;
    if (item['builtin'] == true) {
      if (host.has('create'))
        await c.command('create', {'kind': host.id(item)});
    } else if (host.has('placeAsset') && item['missing'] != true) {
      await c.command('placeAsset', {'id': item['id']});
    }
  }

  @override
  String applyLabel(BrowserHost host, Map<String, dynamic> item) => 'Place';

  @override
  List<Widget> tools(BrowserHost host) => [
    shelfAction(
      'Import',
      host.has('import') ? host.controller.importFiles : null,
    ),
  ];

  @override
  List<String> facts(BrowserHost host, Map<String, dynamic> item) =>
      mediaFacts(item);

  @override
  List<EditorMenuItem<String>> menu(
    BrowserHost host,
    Map<String, dynamic> item,
  ) => [
    for (final action in mediaActions(host.controller, item))
      EditorMenuItem<String>(
        value: action.value,
        enabled: action.enabled,
        child: Text(action.label),
      ),
  ];

  @override
  Future<void> act(
    BrowserHost host,
    String action,
    Map<String, dynamic> item,
  ) => mediaAct(
    host.controller,
    action,
    item,
    paletteSaved: () {
      if (host.mounted) host.showCategory('Colors', 'Saved');
    },
  );

  @override
  void delete(BrowserHost host, Map<String, dynamic> item) {
    if (host.has('removeAsset') && item['used'] != true)
      host.controller.command('removeAsset', {'id': item['id']});
  }

  /// Timeline の行へ落とすと、その位置に置く。掴んだ札は名前だけ持ち出す。
  @override
  Widget draggable(BrowserHost host, Map<String, dynamic> item, Widget tile) =>
      !supported(host, item)
      ? tile
      : Draggable<Map<String, dynamic>>(
          data: {'asset': item['id'], 'name': item['name']},
          dragAnchorStrategy: pointerDragAnchorStrategy,
          feedback: DefaultTextStyle(
            style: EditorTheme.of(host.context).text,
            child: Container(
              height: EditorMetrics.row,
              padding: const EdgeInsets.symmetric(horizontal: EditorMetrics.s6),
              decoration: BoxDecoration(
                color: EditorTheme.of(host.context).panel,
                border: Border.all(color: EditorTheme.of(host.context).accent),
              ),
              child: Text(
                '${item['name']}',
                style: TextStyle(
                  fontSize: EditorMetrics.dense,
                  color: EditorTheme.of(host.context).ink,
                ),
              ),
            ),
          ),
          child: tile,
        );

  /// While files are carried over the window, the shelf says where they go.
  /// It fades in and out, and never takes the pointer.
  @override
  Widget overlay(BrowserHost host) => ValueListenableBuilder<bool>(
    valueListenable: host.controller.dragging,
    builder: (context, dragging, _) => IgnorePointer(
      child: AnimatedOpacity(
        opacity: dragging ? 1 : 0,
        duration: const Duration(milliseconds: 120),
        child: Container(
          key: const ValueKey('browser:drop-hint'),
          margin: const EdgeInsets.all(EditorMetrics.s4),
          decoration: BoxDecoration(
            color: EditorTheme.of(host.context).panel.withValues(alpha: .85),
            border: Border.all(
              color: EditorTheme.of(host.context).accent,
              width: EditorMetrics.s2,
            ),
          ),
          alignment: Alignment.center,
          child: Text(
            'Drop to import',
            style: TextStyle(
              fontSize: EditorMetrics.title,
              color: EditorTheme.of(host.context).ink,
            ),
          ),
        ),
      ),
    ),
  );

  /// Pick a file of the same family and point the asset at it. The layers
  /// keep the asset; only the file behind it changes.
}

final _dataUriCache = <String, Uint8List>{};

/// The item's picture, else an icon of its family; a mesh draws the body its
/// name says.
Widget mediaThumbnail(Map<String, dynamic> item, double tileScale) => Builder(
  builder: (context) {
    final path = item['thumbnail'] as String?;
    final shape = item['missing'] == true || mediaFamily(item) != '3D'
        ? null
        : shapeOf('${item['name'] ?? item['id']}');
    final fallback = Center(
      child: shape != null
          ? SizedBox(
              width: EditorMetrics.s32 * tileScale,
              height: EditorMetrics.s32 * tileScale,
              child: CustomPaint(
                key: ValueKey('browser:shape:${shape.name}'),
                painter: ShapeMark(
                  shape,
                  EditorTheme.of(context).kindColor('3d'),
                ),
              ),
            )
          : Icon(
              item['missing'] == true
                  ? Glyph.broken_image_outlined
                  : mediaFamily(item) == 'Audio'
                  ? Glyph.audiotrack
                  : mediaFamily(item) == '3D'
                  ? Glyph.view_in_ar
                  : Glyph.image_outlined,
              size: EditorMetrics.s19 * tileScale,
              color: EditorTheme.of(context).muted,
            ),
    );
    if (path == null || path.isEmpty) return fallback;
    try {
      if (path.startsWith('data:'))
        return Image.memory(
          // The same bytes each draw, so the image cache recognises them.
          _dataUriCache.putIfAbsent(
            path,
            () => Uri.parse(path).data!.contentAsBytes(),
          ),
          fit: BoxFit.cover,
          gaplessPlayback: true,
          errorBuilder: (_, _, _) => fallback,
        );
      return Image.file(
        File(path.startsWith('file:') ? Uri.parse(path).toFilePath() : path),
        fit: BoxFit.cover,
        gaplessPlayback: true,
        errorBuilder: (_, _, _) => fallback,
      );
    } catch (_) {
      return fallback;
    }
  },
);

