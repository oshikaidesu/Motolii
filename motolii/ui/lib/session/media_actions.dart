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
