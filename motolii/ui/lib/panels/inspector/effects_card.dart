part of '../inspector.dart';

/// One card per effect, built from the declaration: heroes in front, the
/// rest under them, the advanced rows behind a fold, and the head's menu.
mixin _InspectorEffectsCard on _InspectorControls, _InspectorFolds {
  // ---- Effects: one sheet per effect, controls from the declaration -------

  /// Everything an effect can be told to do, in one list: the head keeps a
  /// single mark instead of a row of equal glyphs.
  Future<void> _effectMenu(
    BuildContext context,
    Map<String, dynamic> layer,
    Map<String, dynamic> effect,
    int index,
    int count,
  ) async {
    final box = context.findRenderObject() as RenderBox?;
    final at = box == null
        ? Offset.zero
        : box.localToGlobal(box.size.bottomLeft(Offset.zero));
    final chosen = await showEditorMenu<String>(context, at, [
      EditorMenuItem<String>(
        value: 'earlier',
        enabled: panelCan(c, 'moveEffect') && index > 0,
        child: const Text('Apply earlier'),
      ),
      EditorMenuItem<String>(
        value: 'later',
        enabled: panelCan(c, 'moveEffect') && index < count - 1,
        child: const Text('Apply later'),
      ),
      EditorMenuItem<String>(
        value: 'roll',
        enabled: _canEdit(layer),
        child: const Text('Throw every number within its reach'),
      ),
      EditorMenuItem<String>(
        value: 'rest',
        enabled: _canEdit(layer),
        child: const Text('Back to where the numbers rest'),
      ),
      if (effect['placement'] == true)
        EditorMenuItem<String>(
          value: 'expand',
          enabled: panelCan(c, 'expandEffect'),
          child: const Text('Expand copies into layers'),
        ),
      EditorMenuItem<String>(
        value: 'remove',
        enabled: panelCan(c, 'removeEffect'),
        child: const Text('Remove effect'),
      ),
    ]);
    switch (chosen) {
      case 'earlier' || 'later':
        await c.command('moveEffect', {
          'layer': layer['id'],
          'id': effect['id'],
          'to': chosen == 'earlier' ? index - 1 : index + 1,
        });
      case 'roll':
        await _roll(layer, effect);
      case 'rest':
        await _rest(layer, effect);
      case 'expand':
        await c.command('expandEffect', {
          'layer': layer['id'],
          'id': effect['id'],
        });
      case 'remove':
        await c.command('removeEffect', {
          'layer': layer['id'],
          'id': effect['id'],
        });
    }
  }

  Widget _effect(
    Map<String, dynamic> layer,
    Map<String, dynamic> effect, {
    int index = 0,
    int count = 1,
  }) {
    final params = panelRows(effect['params']);
    // Advanced: declared on the row (ADVANCED in the manifest) or, for a
    // placement, on the grid rows the layout marks.
    final advancedIds = <String>{
      for (final r in params)
        if (r['advanced'] == true) '${r['id']}',
      for (final g in panelRows(panelMap(effect['layout'])['rows']))
        if (g['advanced'] == true) ...[
          if (g['each'] is String) '${g['each']}',
          if (g['random'] is String) '${g['random']}',
        ],
    };
    // Heroes: declared, else the first four that are not advanced (the OP-1
    // rule: four knobs in front, the rest behind shift).
    final plain = params
        .where((r) => !advancedIds.contains('${r['id']}'))
        .toList();
    final declaredHeroes = plain.where((r) => r['hero'] == true).toList();
    final heroIds = <String>{
      for (final r
          in declaredHeroes.isNotEmpty
              ? declaredHeroes
              : plain.length > 4
              ? plain.take(4)
              : const <Map<String, dynamic>>[])
        '${r['id']}',
    };
    final heroes = <_Cell>[];
    final controls = <_Cell>[];
    final advanced = <_Cell>[];
    String? section;
    final byId = {for (final r in params) '${r['id']}': r};
    final folded = <String>{};
    for (final row in params) {
      final id = '${row['id']}';
      if (folded.contains(id)) continue;
      final hero = heroIds.contains(id);
      final into = advancedIds.contains(id)
          ? advanced
          : hero
          ? heroes
          : controls;
      final here = row['section'] as String?;
      if (here != null && here != section && !advancedIds.contains(id)) {
        controls.add(_Cell(_SectionLabel(here), wide: true));
      }
      section = here;
      // A pair the shader adds to a coordinate together (group), or declared
      // as `name_x` + `name_y`, is one point: a pad and two wells.
      final group = row['group'];
      final partner = group is String && group == id
          ? params
                .where((r) => r['group'] == group && r['id'] != id)
                .firstOrNull
          : null;
      final yId = partner != null
          ? '${partner['id']}'
          : id.endsWith('_x')
          ? '${id.substring(0, id.length - 2)}_y'
          : null;
      final y = yId == null ? null : byId[yId];
      if (y != null && row['value'] is num && y['value'] is num) {
        folded.add(yId!);
        into.add(_Cell(_pointControl(layer, id, yId, hero: hero), tall: true));
        continue;
      }
      into.add(
        _Cell(
          _control(layer, id, hero: hero),
          wide: _kindOf(row) == _Kind.color,
        ),
      );
    }
    final key = '${effect['id']}';
    return _card(
      key: ValueKey('effect:$key'),
      section: _effectSection(layer, effect['id']),
      title: '${effect['name']}',
      dim: effect['enabled'] == false,
      // The order is the pipeline: grab the head to move the effect up or
      // down it; the menu keeps the same move for one step at a time.
      leading: panelCan(c, 'moveEffect') ? _EffectGrip(index: index) : null,
      trailing: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          // Only the state a glance needs stays on the head; the seven
          // same-sized glyphs that used to sit here now live behind one.
          _headGlyph(
            effect['enabled'] == false
                ? Glyph.visibility_off_outlined
                : Glyph.visibility_outlined,
            effect['enabled'] == false
                ? 'Off — press to apply'
                : 'Applied — press to bypass',
            panelCan(c, 'enableEffect')
                ? () => c.command('enableEffect', {
                    'layer': layer['id'],
                    'id': effect['id'],
                    'enabled': effect['enabled'] == false,
                  })
                : null,
          ),
          Builder(
            builder: (context) => _headGlyph(
              Glyph.more_horiz,
              'Effect actions',
              () => _effectMenu(context, layer, effect, index, count),
            ),
          ),
        ],
      ),
      children: [
        if (heroes.isNotEmpty) ...[
          _cells(heroes),
          if (controls.isNotEmpty) ...[
            const SizedBox(height: EditorMetrics.s6),
            const EditorRule(height: 1),
            const SizedBox(height: EditorMetrics.s6),
          ],
        ],
        _cells(controls),
        if (advanced.isNotEmpty)
          _AdvancedFold(
            opened: _advancedOpen,
            id: key,
            builder: () => _cells(advanced),
          ),
      ],
    );
  }
}
