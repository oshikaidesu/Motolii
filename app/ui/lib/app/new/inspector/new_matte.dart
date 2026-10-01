import 'package:flutter/widgets.dart';

import '../../../foundation/theme.dart' show showEditorMenu, EditorMenuItem;
import '../../../hf/bp/common.dart';
import '../../../hf/bp/shell.dart' show kTile;
import '../../../hf/desk/common.dart' show kInk;
import '../../../session/editor_session.dart';

const _modes = ['Alpha', 'AlphaInverted', 'Luma', 'LumaInverted'];
const _modeLabels = {'Alpha': 'Alpha', 'AlphaInverted': 'Alpha inverted', 'Luma': 'Luma', 'LumaInverted': 'Luma inverted'};

/// The old Matte card's New face: source layer and mode, the same `setMatte` operation Classic sends.
class NewMatte extends StatelessWidget {
  const NewMatte({super.key, required this.controller, required this.layer, required this.matte});
  final EditorSession controller;
  final Map<String, dynamic> layer;
  final Map<String, dynamic> matte;
  EditorSession get c => controller;

  Widget _row(BuildContext context, String label, String shown, Future<void> Function(BuildContext) onTap) => Padding(
        padding: const EdgeInsets.only(bottom: 4),
        child: Builder(builder: (context) => GestureDetector(
              key: ValueKey('matte-$label'),
              behavior: HitTestBehavior.opaque,
              onTap: c.supports('setMatte') ? () => onTap(context) : null,
              child: Container(
                height: 26,
                padding: const EdgeInsets.symmetric(horizontal: 8),
                decoration: BoxDecoration(color: kTile, borderRadius: BorderRadius.circular(5)),
                child: Row(children: [
                  Text(label, style: sans(11, c: kMuted)),
                  const Spacer(),
                  Flexible(child: Text(shown, maxLines: 1, overflow: TextOverflow.ellipsis, style: sans(11, c: kInk))),
                ]),
              ),
            )),
      );

  @override
  Widget build(BuildContext context) {
    final others = c.layers.where((v) => v['id'] != layer['id']).toList();
    final sourceName = others.where((v) => v['id'] == matte['source']).map((v) => '${v['name']}').firstOrNull ?? '—';
    return Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      Padding(padding: const EdgeInsets.only(bottom: 4), child: Text('MATTE', style: sans(9, c: const Color(0xFF7E7F86), w: FontWeight.w600, ls: 1.2))),
      _row(context, 'Source', sourceName, (context) async {
        final box = context.findRenderObject() as RenderBox?;
        final at = box == null ? Offset.zero : box.localToGlobal(box.size.bottomLeft(Offset.zero));
        final chosen = await showEditorMenu<int>(context, at, [for (final v in others) EditorMenuItem<int>(value: v['id'] as int, child: Text('${v['name']}'))]);
        if (chosen != null) await c.command('setMatte', {'layer': layer['id'], 'source': chosen, 'mode': matte['mode']});
      }),
      _row(context, 'Mode', _modeLabels[matte['mode']] ?? '—', (context) async {
        final box = context.findRenderObject() as RenderBox?;
        final at = box == null ? Offset.zero : box.localToGlobal(box.size.bottomLeft(Offset.zero));
        final chosen = await showEditorMenu<String>(context, at, [for (final m in _modes) EditorMenuItem<String>(value: m, child: Text(_modeLabels[m]!))]);
        if (chosen != null) await c.command('setMatte', {'layer': layer['id'], 'source': matte['source'], 'mode': chosen});
      }),
    ]);
  }
}
