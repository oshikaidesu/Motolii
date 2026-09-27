import 'package:flutter/widgets.dart';

import '../../../foundation/theme.dart' show showEditorMenu, EditorMenuItem;
import '../../../hf/bp/common.dart';
import '../../../hf/desk/common.dart' show kAccent;
import '../../../hf/glyphs.dart';
import '../../../session/editor_session.dart';
import '../../../session/effect_actions.dart';
import '../../../session/read_model.dart';
import 'new_effect.dart';

/// One effect's whole card — head (grip, applied/bypassed, actions) and body — the New face for
/// `InspectorInstruments.effectCard`. The body is the same generic params sheet [NewEffectParams] already draws;
/// only the head (Classic's `_effectMenu`/`_headGlyph`) is redrawn here, over the same operations.
class NewEffectCard extends StatelessWidget {
  const NewEffectCard({super.key, required this.controller, required this.layer, required this.effect, required this.index, required this.count});
  final EditorSession controller;
  final Map<String, dynamic> layer;
  final Map<String, dynamic> effect;
  final int index;
  final int count;
  EditorSession get c => controller;

  Future<void> _menu(BuildContext context, Offset at) async {
    final chosen = await showEditorMenu<String>(context, at, [
      EditorMenuItem<String>(value: 'earlier', enabled: c.supports('moveEffect') && index > 0, child: const Text('Apply earlier')),
      EditorMenuItem<String>(value: 'later', enabled: c.supports('moveEffect') && index < count - 1, child: const Text('Apply later')),
      const EditorMenuItem<String>(value: 'roll', child: Text('Throw every number within its reach')),
      const EditorMenuItem<String>(value: 'rest', child: Text('Back to where the numbers rest')),
      if (effect['placement'] == true) EditorMenuItem<String>(value: 'expand', enabled: c.supports('expandEffect'), child: const Text('Expand copies into layers')),
      EditorMenuItem<String>(value: 'remove', enabled: c.supports('removeEffect'), child: const Text('Remove effect')),
    ]);
    switch (chosen) {
      case 'earlier' || 'later':
        await c.command('moveEffect', {'layer': layer['id'], 'id': effect['id'], 'to': chosen == 'earlier' ? index - 1 : index + 1});
      case 'roll':
        await rollEffect(c, layer['id'] as int, effect);
      case 'rest':
        await restEffect(c, layer['id'] as int, effect);
      case 'expand':
        await c.command('expandEffect', {'layer': layer['id'], 'id': effect['id']});
      case 'remove':
        await c.command('removeEffect', {'layer': layer['id'], 'id': effect['id']});
    }
  }

  @override
  Widget build(BuildContext context) {
    final on = effect['enabled'] != false;
    return Container(
      key: ValueKey('effect-card:${effect['id']}'),
      margin: const EdgeInsets.only(bottom: 8),
      decoration: BoxDecoration(color: kRaised, borderRadius: BorderRadius.circular(6), border: Border.all(color: kRule)),
      child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        Container(
          height: 30,
          padding: const EdgeInsets.symmetric(horizontal: 8),
          decoration: BoxDecoration(border: Border(bottom: BorderSide(color: kRule2)), borderRadius: const BorderRadius.vertical(top: Radius.circular(6))),
          child: Row(children: [
            SizedBox(width: 12, height: 12, child: CustomPaint(painter: HgPainter(HG.list, kMuted, kRaised))),
            const SizedBox(width: 6),
            Expanded(child: Text('${effect['name']}', maxLines: 1, overflow: TextOverflow.ellipsis, style: sans(11.5, c: on ? const Color(0xFFDADBDC) : kMuted, w: FontWeight.w600))),
            GestureDetector(
              key: ValueKey('effect-toggle:${effect['id']}'),
              behavior: HitTestBehavior.opaque,
              onTap: c.supports('enableEffect') ? () => c.command('enableEffect', {'layer': layer['id'], 'id': effect['id'], 'enabled': !on}) : null,
              child: Padding(padding: const EdgeInsets.all(4), child: SizedBox(width: 13, height: 13, child: CustomPaint(painter: HgPainter(HG.power, on ? kAccent : kMuted, kRaised)))),
            ),
            Builder(builder: (context) => GestureDetector(
              key: ValueKey('effect-menu:${effect['id']}'),
              behavior: HitTestBehavior.opaque,
              onTap: () {
                final box = context.findRenderObject() as RenderBox?;
                _menu(context, box == null ? Offset.zero : box.localToGlobal(box.size.bottomLeft(Offset.zero)));
              },
              child: Padding(padding: const EdgeInsets.all(4), child: SizedBox(width: 13, height: 13, child: CustomPaint(painter: HgPainter(HG.kebab, kMuted, kRaised)))),
            )),
          ]),
        ),
        Padding(
          padding: const EdgeInsets.all(8),
          child: panelMap(effect['layout']).isEmpty
              ? NewEffectParams(key: ValueKey('new-effect-params:${layer['id']}:${effect['id']}'), controller: c, layerId: layer['id'] as int, effectId: effect['id'] as Object)
              : Text('This effect lays out its copies on a grid; that view is still the Inspector\'s own.', style: sans(10, c: kMuted)),
        ),
      ]),
    );
  }
}
