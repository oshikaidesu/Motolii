import 'package:flutter/widgets.dart';

import '../../../hf/bp/common.dart';
import '../../../hf/bp/shell.dart' show kTile;
import '../../../hf/desk/common.dart' show kInk;
import '../../../hf/insp/panel.dart';
import '../../../panels/rich_text_editor.dart' show RichTextEditor;
import '../../../session/editor_session.dart';
import '../../../session/read_model.dart';
import 'session_layer_rows.dart';

bool _isTextRow(Map<String, dynamic> r) {
  final id = '${r['id']}';
  return id.startsWith('text') && id != 'text_justify' && !id.endsWith('.size') && r['kind'] != 'color';
}

/// The Text card's New face: the same [RichTextEditor] Classic uses (what it says), a Font row, and the rest of the
/// text rows (line height, tracking, ...) as generic Toys over [SessionLayerRowsStore] — the same shape
/// [NewEffectParams] gives an effect's params.
class NewText extends StatefulWidget {
  const NewText({super.key, required this.controller, required this.layer});
  final EditorSession controller;
  final Map<String, dynamic> layer;
  @override
  State<NewText> createState() => _NewTextState();
}

class _NewTextState extends State<NewText> {
  static const _watched = ['layers', 'selectedId', 'selectedIds', 'capabilities', 'documentRevision'];
  EditorSession get c => widget.controller;
  late SessionLayerRowsStore store = SessionLayerRowsStore(c, widget.layer['id'] as int, _isTextRow);

  @override
  void initState() {
    super.initState();
    c.slice('newText', _watched).addListener(store.absorb);
    c.rendered.addListener(store.absorb);
  }

  @override
  void didUpdateWidget(NewText old) {
    super.didUpdateWidget(old);
    if (old.layer['id'] != widget.layer['id']) {
      store.dispose();
      store = SessionLayerRowsStore(c, widget.layer['id'] as int, _isTextRow);
    } else {
      store.absorb();
    }
  }

  @override
  void dispose() {
    c.slice('newText', _watched).removeListener(store.absorb);
    c.rendered.removeListener(store.absorb);
    store.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final text = panelMap(widget.layer['text']);
    final family = '${text['fontFamily'] ?? ''}';
    final layerId = widget.layer['id'] as int;
    final canEdit = widget.layer['locked'] != true && c.supports('setFont');
    return Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      Padding(padding: const EdgeInsets.only(bottom: 4), child: Text('TEXT', style: sans(9, c: const Color(0xFF7E7F86), w: FontWeight.w600, ls: 1.2))),
      RichTextEditor(key: ValueKey('rich:$layerId'), controller: c, layer: widget.layer, text: text),
      const SizedBox(height: 6),
      GestureDetector(
        key: const ValueKey('inspector:font'),
        behavior: HitTestBehavior.opaque,
        onTap: canEdit ? () => c.focusFont(widget.layer) : null,
        child: Container(
          height: 26,
          padding: const EdgeInsets.symmetric(horizontal: 8),
          decoration: BoxDecoration(color: kTile, borderRadius: BorderRadius.circular(5)),
          child: Row(children: [
            Text('Font', style: sans(11, c: kMuted)),
            const Spacer(),
            Flexible(child: Text(family, maxLines: 1, overflow: TextOverflow.ellipsis, style: sans(11, c: kInk))),
          ]),
        ),
      ),
      const SizedBox(height: 6),
      ListenableBuilder(listenable: store, builder: (context, _) => store.rows.isEmpty ? const SizedBox.shrink() : ParamSheet(store, thingId: 'text:$layerId')),
    ]);
  }
}
