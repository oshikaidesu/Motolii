import 'package:flutter/widgets.dart';

import '../hf/shell/menu.dart';
import '../browser/parts.dart';
import '../desks/parts.dart' show kAccent;
import '../hf/glyphs.dart';
import '../session/editor_session.dart';
import '../session/effect_actions.dart';
import 'params.dart';
import '../hf/metrics.dart';
import '../hf/neutral.dart';
import '../inspector/session.dart';
import '../hf/metrics.dart' show Dn, Surface;

/// One effect's whole card — head (grip, applied/bypassed, actions) and body — the New face for
/// `InspectorInstruments.effectCard`. The body is the same generic params sheet [NewEffectParams] already draws;
/// only the head (Classic's `_effectMenu`/`_headGlyph`) is redrawn here, over the same operations.
class NewEffectCard extends StatelessWidget {
  const NewEffectCard({super.key, required this.controller, required this.layer, required this.effect, required this.index, required this.count, this.reorder = false, this.folded = false, this.onFold});
  final EditorSession controller;
  final Map<String, dynamic> layer;
  final Map<String, dynamic> effect;
  final int index;
  final int count;

  /// The card sits in a reorderable list: its header is the grip (drag it to apply the effect earlier or later).
  final bool reorder;

  /// A tap on the header folds the card to its header (Classic IN-014); the host keeps which are folded.
  final bool folded;
  final VoidCallback? onFold;
  EditorSession get c => controller;

  Future<void> _menu(BuildContext context, Offset at) async {
    final moves = c.supports('moveEffect');
    final chosen = await showHfMenu<String>(context, Rect.fromLTWH(at.dx, at.dy, 240, 0), [
      ('earlier', 'Apply earlier'),
      ('later', 'Apply later'),
      ('roll', 'Throw every number within its reach'),
      ('rest', 'Back to where the numbers rest'),
      if (effect['placement'] == true) ('expand', 'Expand copies into layers'),
      ('remove', 'Remove effect'),
    ], disabled: {
      if (!moves || index == 0) 'earlier',
      if (!moves || index >= count - 1) 'later',
      if (!c.supports('expandEffect')) 'expand',
      if (!c.supports('removeEffect')) 'remove',
    });
    switch (chosen) {
      case 'earlier' || 'later':
        InspectorSession.of(c).stepEffect(layer['id'] as int, effect['id'], chosen == 'earlier' ? -1 : 1);
      case 'roll':
        await rollEffect(c, layer['id'] as int, effect);
      case 'rest':
        await restEffect(c, layer['id'] as int, effect);
      case 'expand':
        await InspectorSession.of(c).expandEffect(layer['id'] as int, effect['id']);
      case 'remove':
        await InspectorSession.of(c).removeEffect(layer['id'] as int, effect['id']);
    }
  }

  Widget _grip(Widget header) => !reorder
      ? header
      : MouseRegion(
          cursor: SystemMouseCursors.grab,
          child: ReorderableDragStartListener(key: ValueKey('effect-grip:${effect['id']}'), index: index, child: header),
        );

  /// A frozen layer's effects are baked, a locked layer's untouchable: shown, not changed (Classic IN-141).
  bool get _held => layer['frozen'] == true || layer['locked'] == true;

  @override
  Widget build(BuildContext context) {
    final on = effect['enabled'] != false;
    // Work density: an effect is a band on the raised level over a body on the base, separated by a hairline; not a card.
    return DecoratedBox(
      key: ValueKey('effect-card:${effect['id']}'),
      decoration: const BoxDecoration(border: Border(top: BorderSide(color: Surface.divider, width: Surface.hair))),
      child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        _grip(GestureDetector(key: ValueKey('effect-head:${effect['id']}'), behavior: HitTestBehavior.opaque, onTap: onFold, child: Container(
          height: Surface.workRow,
          padding: const EdgeInsets.symmetric(horizontal: Surface.sectionGap),
          color: Surface.raised,
          child: Row(children: [
            SizedBox(width: Surface.glyph, height: Surface.glyph, child: CustomPaint(painter: HgPainter(HG.list, Surface.muted, Surface.raised))),
            const SizedBox(width: Surface.inlineGap),
            Expanded(child: Text('${effect['name']}', maxLines: 1, overflow: TextOverflow.ellipsis, style: sans(Dn.nameSize, c: on ? N.g86 : Surface.muted, w: FontWeight.w600))),
            GestureDetector(
              key: ValueKey('effect-toggle:${effect['id']}'),
              behavior: HitTestBehavior.opaque,
              onTap: InspectorSession.of(c).canEnableEffects && !_held ? () => InspectorSession.of(c).flipEffect(layer['id'] as int, effect['id']) : null,
              child: Padding(padding: const EdgeInsets.all(Surface.inlineGap), child: SizedBox(width: Surface.glyph, height: Surface.glyph, child: CustomPaint(painter: HgPainter(HG.power, on ? kAccent : Surface.muted, Surface.raised)))),
            ),
            Builder(builder: (context) => GestureDetector(
              key: ValueKey('effect-menu:${effect['id']}'),
              behavior: HitTestBehavior.opaque,
              onTap: _held
                  ? null
                  : () {
                      final box = context.findRenderObject() as RenderBox?;
                      _menu(context, box == null ? Offset.zero : box.localToGlobal(box.size.bottomLeft(Offset.zero)));
                    },
              child: Padding(padding: const EdgeInsets.all(Surface.inlineGap), child: SizedBox(width: Surface.glyph, height: Surface.glyph, child: CustomPaint(painter: HgPainter(HG.kebab, Surface.muted, Surface.raised)))),
            )),
          ]),
        ))),
        if (!folded && layer['frozen'] == true)
          Padding(
            padding: const EdgeInsets.fromLTRB(Surface.sectionGap, Surface.sectionGap, Surface.sectionGap, 0),
            child: Text('Frozen — effects are baked. Unfreeze to edit.', key: ValueKey('effect-frozen:${effect['id']}'), style: sans(Dn.labelSize, c: Surface.muted)),
          ),
        // a placement effect's parameters are its params like any other (its grid view is not drawn here yet)
        if (!folded)
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: Surface.sectionGap, vertical: Surface.inlineGap),
          child: NewEffectParams(key: ValueKey('new-effect-params:${layer['id']}:${effect['id']}'), controller: c, layerId: layer['id'] as int, effectId: effect['id'] as Object),
        ),
      ]),
    );
  }
}
