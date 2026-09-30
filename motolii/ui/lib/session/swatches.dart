import 'dart:io';

import 'color_edit.dart';
import 'editor_session.dart';
import 'media_actions.dart' show savePalette;

/// The saved swatches (the desk's `swatches`): each is its stops (one for a solid) and, for a gradient, its blend.
List<Map<String, dynamic>> savedSwatches(EditorSession c) =>
    EditorSession.maps(c.deskWork.value['swatches']);

/// Keep what the wheel edits: the active layer's fill (every stop, with its blend), else the colour target. False
/// when there is nothing to keep.
Future<bool> saveCurrentSwatch(EditorSession c) async {
  final fill = EditorSession.map(c.activeLayer?['fill']);
  final target = colorTarget(c);
  final colors = fill.isNotEmpty
      ? [for (final stop in EditorSession.maps(fill['stops'])) rgbaOf(stop['rgba'])]
      : target == null
      ? <List<double>>[]
      : [rgbaOf(target['rgba'])];
  if (colors.isEmpty) return false;
  await c.storeDesk('swatches', [
    ...savedSwatches(c),
    {
      'stops': colors,
      if (colors.length > 1) 'blend': fill['blend'] ?? 'oklab',
    },
  ]);
  return true;
}

Future<void> forgetSwatch(EditorSession c, int index) async {
  final kept = savedSwatches(c);
  if (index < 0 || index >= kept.length) return;
  kept.removeAt(index);
  await c.storeDesk('swatches', kept);
}

const paletteImageExtensions = ['png', 'jpg', 'jpeg', 'webp', 'bmp', 'gif', 'tif', 'tiff'];

/// Pictures picked in the system dialog give their main colours as saved solids. True when any colours were kept.
Future<bool> paletteFromImages(EditorSession c) async {
  final picked = await c.native('pickImport', {'extensions': paletteImageExtensions});
  if (picked is! List) return false;
  var kept = false;
  for (final path in picked.whereType<String>()) {
    kept = await savePalette(c, await File(path).readAsBytes()) || kept;
  }
  return kept;
}
