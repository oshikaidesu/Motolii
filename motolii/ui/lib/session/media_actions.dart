import 'dart:io';

import 'package:flutter/services.dart';

import 'color_palette.dart';
import 'editor_session.dart';

/// The plain path behind a media item; the status may carry it as a file URI.
String? filePath(Map<String, dynamic> item) {
  final raw = item['path'] as String?;
  if (raw == null || raw.isEmpty) return null;
  return raw.startsWith('file:') ? Uri.parse(raw).toFilePath() : raw;
}

/// The system's own name for showing a file where it lives.
String get revealLabel => Platform.isMacOS
    ? 'Reveal in Finder'
    : Platform.isWindows
    ? 'Show in Explorer'
    : 'Show in file manager';

/// One line of an imported media item's menu.
typedef MediaAction = ({String value, String label, bool enabled});

/// What can be done to an imported media item (a bundled one has nothing): use it on the selected layer, find its
/// file, take its colours, point it at another file, or drop it from the library when nothing uses it. Every Media
/// presentation offers these and hands the choice to [mediaAct].
List<MediaAction> mediaActions(EditorSession c, Map<String, dynamic> item) {
  if (item['builtin'] == true) return const [];
  final missing = item['missing'] == true;
  final path = filePath(item);
  return [
    (
      value: 'replace',
      label: 'Replace selected layer',
      enabled: c.supports('replaceAsset') && !missing && c.selectedIds.isNotEmpty,
    ),
    if (path != null && !missing) ...[
      (value: 'reveal', label: revealLabel, enabled: true),
      (value: 'open', label: 'Open with default app', enabled: true),
      if ('${item['mime']}'.startsWith('image/'))
        (value: 'palette', label: 'Extract palette', enabled: true),
    ],
    if (path != null) (value: 'copyPath', label: 'Copy path', enabled: true),
    if (c.supports('relinkAsset'))
      (
        value: 'relink',
        label: missing ? 'Locate file…' : 'Relink to another file…',
        enabled: true,
      ),
    (
      value: 'remove',
      label: 'Remove from library',
      enabled: c.supports('removeAsset') && item['used'] != true,
    ),
  ];
}

/// Does [action] to [item]. The item's own id is what the host knows it by ([id], when the presentation keys it
/// differently). A palette is kept with the saved swatches, then [paletteSaved] runs.
Future<void> mediaAct(
  EditorSession c,
  String action,
  Map<String, dynamic> item, {
  Object? id,
  void Function()? paletteSaved,
}) async {
  final asset = id ?? item['id'];
  final path = filePath(item);
  switch (action) {
    case 'replace':
      await c.command('replaceAsset', {'id': asset});
    case 'reveal':
      await c.native('reveal', {'path': path});
    case 'open':
      await c.native('openFile', {'path': path});
    case 'copyPath':
      await Clipboard.setData(ClipboardData(text: path ?? ''));
    case 'relink':
      await _relink(c, item, asset);
    case 'palette':
      if (await savePalette(c, File('$path').readAsBytesSync()))
        paletteSaved?.call();
    case 'remove':
      await c.command('removeAsset', {'id': asset});
  }
}

/// The colours of a picture, kept with the saved swatches (`storeDesk('swatches')`). False, with the reason on the
/// session's error line, when there were none.
Future<bool> savePalette(EditorSession c, Uint8List bytes) async {
  final colors = await paletteOf(bytes);
  if (colors.isEmpty) {
    c.error.value = 'No colours found in that image';
    return false;
  }
  await c.storeDesk('swatches', [
    ...EditorSession.maps(c.deskWork.value['swatches']),
    for (final rgba in colors)
      {
        'stops': [rgba],
      },
  ]);
  return true;
}

Future<void> _relink(
  EditorSession c,
  Map<String, dynamic> item,
  Object? asset,
) async {
  final family = '${item['mime']}'.split('/').first;
  final extensions = switch (family) {
    'image' => const ['png', 'jpg', 'jpeg', 'webp', 'bmp', 'gif', 'tif', 'tiff'],
    'video' => const ['mp4', 'mov', 'mkv', 'webm'],
    'audio' => const ['wav', 'mp3', 'flac', 'aac'],
    _ => const <String>[],
  };
  final picked = await c.native('pickImport', {
    if (extensions.isNotEmpty) 'extensions': extensions,
  });
  if (picked is! List || picked.isEmpty) return;
  await c.command('relinkAsset', {'id': asset, 'path': '${picked.first}'});
}

/// What the file is and where it lives, one line each (format, dimensions and length, size, folder, state): the
/// facts a Media menu shows, the only place they are seen.
List<String> mediaFacts(Map<String, dynamic> item) {
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
  ].where((line) => line.isNotEmpty).toList();
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
  if (RegExp(r'\.(obj|glb|gltf|ply)$').hasMatch(path) || mime.contains('mesh') || mime.contains('model')) return '3D';
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

  /// Dimensions, rate and length, from what the file itself says.
String _mediaFact(Map<String, dynamic> item) {
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
String _trim(double v) =>
    v == v.roundToDouble() ? '${v.round()}' : v.toStringAsFixed(2);

/// Seconds as m:ss.s, or h:mm:ss past an hour.
String _clock(double seconds) {
  final whole = seconds.floor();
  final h = whole ~/ 3600, m = (whole % 3600) ~/ 60;
  final s = seconds - h * 3600 - m * 60;
  if (h > 0)
    return '$h:${m.toString().padLeft(2, '0')}:${s.floor().toString().padLeft(2, '0')}';
  return '$m:${s.toStringAsFixed(1).padLeft(4, '0')}';
}
