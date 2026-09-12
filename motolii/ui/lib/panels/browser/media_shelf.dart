import 'dart:io';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../../foundation/metrics.dart';
import '../../foundation/theme.dart';
import '../../session/editor_session.dart';
import 'colors_shelf.dart';
import 'create_shelf.dart';
import 'parts.dart';
import 'shelf.dart';

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
  List<String> facts(BrowserHost host, Map<String, dynamic> item) {
    final path = filePath(item);
    final missing = item['missing'] == true;
    return [
      [
        if (mediaFormat(item).isNotEmpty) mediaFormat(item),
        if (item['mime'] != null) '${item['mime']}',
      ].join(' · '),
      _mediaFact(item),
      if (path != null && !missing) fileFact(path),
      if (path != null) homely(File(path).parent.path),
      if (missing) 'Missing file',
      if (item['used'] == true) 'In use by a layer',
    ];
  }

  @override
  List<EditorMenuItem<String>> menu(
    BrowserHost host,
    Map<String, dynamic> item,
  ) {
    if (item['builtin'] == true) return const [];
    final missing = item['missing'] == true;
    final path = filePath(item);
    return [
      EditorMenuItem<String>(
        value: 'replace',
        enabled:
            host.has('replaceAsset') &&
            !missing &&
            host.controller.selectedIds.isNotEmpty,
        child: const Text('Replace selected layer'),
      ),
      if (path != null && !missing) ...[
        EditorMenuItem<String>(value: 'reveal', child: Text(revealLabel)),
        const EditorMenuItem<String>(
          value: 'open',
          child: Text('Open with default app'),
        ),
        if ('${item['mime']}'.startsWith('image/'))
          const EditorMenuItem<String>(
            value: 'palette',
            child: Text('Extract palette'),
          ),
      ],
      if (path != null)
        const EditorMenuItem<String>(
          value: 'copyPath',
          child: Text('Copy path'),
        ),
      if (host.has('relinkAsset'))
        EditorMenuItem<String>(
          value: 'relink',
          child: Text(missing ? 'Locate file…' : 'Relink to another file…'),
        ),
      EditorMenuItem<String>(
        value: 'remove',
        enabled: host.has('removeAsset') && item['used'] != true,
        child: const Text('Remove from library'),
      ),
    ];
  }

  @override
  Future<void> act(
    BrowserHost host,
    String action,
    Map<String, dynamic> item,
  ) async {
    final c = host.controller;
    final path = filePath(item);
    switch (action) {
      case 'replace':
        await c.command('replaceAsset', {'id': item['id']});
      case 'reveal':
        await c.native('reveal', {'path': path});
      case 'open':
        await c.native('openFile', {'path': path});
      case 'copyPath':
        await Clipboard.setData(ClipboardData(text: path ?? ''));
      case 'relink':
        await _relink(host, item);
      case 'palette':
        await ColorsShelf.savePalette(host, File('$path').readAsBytesSync());
      case 'remove':
        await c.command('removeAsset', {'id': item['id']});
    }
  }

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
          feedback: Material(
            color: Colors.transparent,
            child: Container(
              height: EditorMetrics.row,
              padding: const EdgeInsets.symmetric(horizontal: EditorMetrics.s6),
              decoration: BoxDecoration(
                color: EditorTheme.panel,
                border: Border.all(color: EditorTheme.accent),
              ),
              child: Text(
                '${item['name']}',
                style: const TextStyle(
                  fontSize: EditorMetrics.dense,
                  color: EditorTheme.ink,
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
            color: EditorTheme.panel.withValues(alpha: .85),
            border: Border.all(
              color: EditorTheme.accent,
              width: EditorMetrics.s2,
            ),
          ),
          alignment: Alignment.center,
          child: const Text(
            'Drop to import',
            style: TextStyle(
              fontSize: EditorMetrics.title,
              color: EditorTheme.ink,
            ),
          ),
        ),
      ),
    ),
  );

  /// Pick a file of the same family and point the asset at it. The layers
  /// keep the asset; only the file behind it changes.
  Future<void> _relink(BrowserHost host, Map<String, dynamic> item) async {
    final family = '${item['mime']}'.split('/').first;
    final extensions = switch (family) {
      'image' => const [
        'png',
        'jpg',
        'jpeg',
        'webp',
        'bmp',
        'gif',
        'tif',
        'tiff',
      ],
      'video' => const ['mp4', 'mov', 'mkv', 'webm'],
      'audio' => const ['wav', 'mp3', 'flac', 'aac'],
      _ => const <String>[],
    };
    final picked = await host.controller.native('pickImport', {
      if (extensions.isNotEmpty) 'extensions': extensions,
    });
    if (picked is! List || picked.isEmpty || !host.mounted) return;
    await host.controller.command('relinkAsset', {
      'id': item['id'],
      'path': '${picked.first}',
    });
  }

  /// Dimensions, rate and length, from what the file itself says.
  static String _mediaFact(Map<String, dynamic> item) {
    final facts = item['facts'];
    final seconds =
        (facts is Map ? facts['seconds'] : null) as num? ??
        item['seconds'] as num?;
    final parts = <String>[
      if (facts is Map && facts['width'] != null)
        '${facts['width']}×${facts['height']}',
      if (facts is Map && facts['fps'] != null)
        '${_trim((facts['fps'] as num).toDouble())} fps',
      if (facts is Map && facts['sampleRate'] != null)
        '${_trim((facts['sampleRate'] as num) / 1000)} kHz',
      if (facts is Map && facts['channels'] != null) '${facts['channels']} ch',
      if (seconds != null) _clock(seconds.toDouble()),
    ];
    return parts.join(' · ');
  }

  /// 29.97 stays 29.97; 30 stays 30.
  static String _trim(double v) =>
      v == v.roundToDouble() ? '${v.round()}' : v.toStringAsFixed(2);

  /// Seconds as m:ss.s, or h:mm:ss past an hour.
  static String _clock(double seconds) {
    final whole = seconds.floor();
    final h = whole ~/ 3600, m = (whole % 3600) ~/ 60;
    final s = seconds - h * 3600 - m * 60;
    if (h > 0)
      return '$h:${m.toString().padLeft(2, '0')}:${s.floor().toString().padLeft(2, '0')}';
    return '$m:${s.toStringAsFixed(1).padLeft(4, '0')}';
  }
}

/// Video / Audio / 3D / HDR / 2D / Folder, from the MIME or the extension.
String mediaFamily(Map<String, dynamic> item) {
  if (item['folder'] == true) return 'Folder';
  final mime = '${item['mime'] ?? ''}'.toLowerCase();
  final path = '${item['path'] ?? ''}'.toLowerCase();
  if (mime.contains('video') || RegExp(r'\.(mp4|mov|mkv|webm)$').hasMatch(path))
    return 'Video';
  if (mime.contains('audio') || RegExp(r'\.(wav|mp3|flac|aac)$').hasMatch(path))
    return 'Audio';
  if (RegExp(r'\.(obj|glb|gltf|ply)$').hasMatch(path)) return '3D';
  // 空として置く画(1.0 超を持つ形式)。native の ENVIRONMENT_EXTENSIONS と同じ 2 つ。
  if (mime.contains('/hdr') ||
      mime.contains('/exr') ||
      RegExp(r'\.(hdr|exr)$').hasMatch(path))
    return 'HDR';
  return '2D';
}

/// The badge: the file's extension, else the MIME's tail, else the family.
String mediaFormat(Map<String, dynamic> item) {
  if (item['folder'] == true) return '';
  if (item['builtin'] == true) return 'HDR';
  final filename = '${item['path'] ?? item['name'] ?? ''}'.split('/').last;
  if (filename.contains('.')) return filename.split('.').last.toUpperCase();
  final mime = '${item['mime'] ?? ''}';
  return mime.contains('/')
      ? mime.split('/').last.toUpperCase()
      : mediaFamily(item);
}

final _dataUriCache = <String, Uint8List>{};

/// The item's picture, else an icon of its family; a mesh draws the body its
/// name says.
Widget mediaThumbnail(Map<String, dynamic> item, double tileScale) {
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
              painter: ShapeMark(shape, EditorTheme.kindColor('3d')),
            ),
          )
        : Icon(
            item['missing'] == true
                ? Icons.broken_image_outlined
                : mediaFamily(item) == 'Audio'
                ? Icons.audiotrack
                : mediaFamily(item) == '3D'
                ? Icons.view_in_ar
                : Icons.image_outlined,
            size: EditorMetrics.s19 * tileScale,
            color: EditorTheme.muted,
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
}

/// The plain path behind a card; the status may carry it as a file URI.
String? filePath(Map<String, dynamic> item) {
  final raw = item['path'] as String?;
  if (raw == null || raw.isEmpty) return null;
  return raw.startsWith('file:') ? Uri.parse(raw).toFilePath() : raw;
}

/// The file's size, for the menu's facts.
String fileFact(String path) {
  final file = File(path);
  final bytes = file.existsSync() ? file.lengthSync() : 0;
  return bytes >= 1 << 20
      ? '${(bytes / (1 << 20)).toStringAsFixed(1)} MB'
      : '${(bytes / (1 << 10)).round()} KB';
}

/// A path the way a person reads it: the home folder as ~.
String homely(String path) {
  final home = Platform.environment['HOME'];
  return home != null && path.startsWith(home)
      ? '~${path.substring(home.length)}'
      : path;
}

/// The system's own name for showing a file where it lives.
String get revealLabel => Platform.isMacOS
    ? 'Reveal in Finder'
    : Platform.isWindows
    ? 'Show in Explorer'
    : 'Show in file manager';
