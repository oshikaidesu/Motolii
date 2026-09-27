import 'dart:math' as math;

import 'editor_session.dart';
import 'property_character.dart';
import 'read_model.dart';

/// What "roll" and "rest" mean for an effect's parameters, shared by every face that shows an effect: Classic's
/// Inspector card and a New one alike write through the same `previewProperties` / `commitPreview` operations.

/// Every bounded number of an effect nudged by chance — within a fifth of its reach around where it is, so a throw
/// stays playable — seeds fully, counts whole, as one edit (the Ableton dice).
Future<void> rollEffect(EditorSession c, int layerId, Map<String, dynamic> effect) async {
  final rnd = math.Random();
  final values = <String, double>{};
  for (final row in panelRows(effect['params'])) {
    final id = '${row['id']}';
    if (row['value'] is! num || row['choices'] is List) continue;
    final min = (row['min'] as num?)?.toDouble(), max = (row['max'] as num?)?.toDouble();
    if (id.endsWith('.seed')) {
      values[id] = rnd.nextInt(10000).toDouble();
    } else if (min != null && max != null) {
      final reach = (max - min) * .2;
      var next = ((row['value'] as num).toDouble() + (rnd.nextDouble() * 2 - 1) * reach).clamp(min, max).toDouble();
      if (characterOf(row) == PropertyCharacter.count) next = next.roundToDouble();
      values[id] = next;
    }
  }
  if (values.isNotEmpty) await _writeMany(c, layerId, values);
}

/// Every number of an effect back to where it rests, as one edit.
Future<void> restEffect(EditorSession c, int layerId, Map<String, dynamic> effect) async {
  final values = <String, double>{};
  for (final row in panelRows(effect['params'])) {
    final d = row['default'];
    if (row['value'] is num && d is num && row['choices'] is! List) {
      values['${row['id']}'] = d.toDouble();
    }
  }
  if (values.isNotEmpty) await _writeMany(c, layerId, values);
}

Future<void> _writeMany(EditorSession c, int layerId, Map<String, double> values) async {
  await c.command('previewProperties', {
    'edits': [
      for (final e in values.entries) {'layer': layerId, 'property': e.key, 'value': e.value, 'spread': 'absolute'},
    ],
  });
  await c.command('commitPreview');
}
