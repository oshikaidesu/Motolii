// The desk's Notes tool: what a maker writes of a thing for whoever opens the document next (its note, the inside's
// value: After Effects' layer comment). The picked thing's note written in place, told to the document when the hand
// leaves it (one step); under it every thing that has one, a tap picks it.
import 'package:flutter/widgets.dart';

import '../tokens.dart';
import 'desk_kit.dart';
import 'ws.dart';

class DeskNotes extends StatefulWidget {
  const DeskNotes({super.key});
  @override
  State<DeskNotes> createState() => _DeskNotesState();
}

class _DeskNotesState extends State<DeskNotes> {
  final _words = TextEditingController();
  final _focus = FocusNode();
  int? _of;   // (whose note the field holds)
  (int, List<(WsLayer, String)>)? _noted;

  @override
  void initState() {
    super.initState();
    _focus.addListener(() {
      if (!_focus.hasFocus) _say();
    });
  }

  void _say() {
    final ws = WsScope.read(context), m = ws.inside, t = _of;
    if (m == null || t == null || _words.text == m.word(t, 'note')) return;
    if (m.say(t, 'note', _words.text)) ws.reread();
  }

  @override
  void dispose() {
    _focus.dispose();
    _words.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final ws = WsScope.of(context), m = ws.inside, t = int.tryParse(ws.selected ?? '');
    if (m == null) return const SizedBox.shrink();
    if (t != _of) {   // (another thing picked: the one before told what was written, after this picture, the field holds this one's)
      if (_focus.hasFocus && _of != null) {
        final (was, words) = (_of!, _words.text);
        WidgetsBinding.instance.addPostFrameCallback((_) {
          if (words != m.word(was, 'note') && m.say(was, 'note', words)) ws.reread();
        });
      }
      _of = t;
      _words.text = t == null ? '' : m.word(t, 'note');
    } else if (!_focus.hasFocus && t != null && _words.text != m.word(t, 'note')) {
      _words.text = m.word(t, 'note');   // (undone, or said elsewhere)
    }
    if (_noted?.$1 != ws.reads) _noted = (ws.reads, [for (final l in ws.layers) if (m.word(int.parse(l.id), 'note') case final n when n.isNotEmpty) (l, n)]);   // (asked again when the document is read again, not each picture)
    final noted = _noted!.$2;
    return ListView(
      padding: const EdgeInsets.all(WsT.gap),
      children: [
        if (ws.layer case final l?) DeskStrip(left: [DeskLayerChip(l.name, wsTone(l.kind))]),
        const SizedBox(height: 8),
        Opacity(
          opacity: t == null ? .35 : 1,
          child: Container(
            height: 84,
            padding: const EdgeInsets.all(8),
            decoration: BoxDecoration(color: WsT.well, borderRadius: BorderRadius.circular(5)),
            child: EditableText(
              key: const ValueKey('notes.field'),
              controller: _words,
              focusNode: _focus,
              readOnly: t == null,
              style: T.name(Grey.g91),
              cursorColor: WsT.accent,
              backgroundCursorColor: Grey.g38,
              maxLines: null,
              keyboardType: TextInputType.multiline,
            ),
          ),
        ),
        const SizedBox(height: 10),
        for (final (l, n) in noted)
          GestureDetector(
            key: ValueKey('notes.${l.id}'),
            behavior: HitTestBehavior.opaque,
            onTap: () => ws.select(l.id),
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 5),
              margin: const EdgeInsets.only(bottom: 2),
              decoration: BoxDecoration(color: l.id == ws.selected ? WsT.well : null, borderRadius: BorderRadius.circular(5)),
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Container(width: 4, height: 14, decoration: BoxDecoration(color: wsTone(l.kind), borderRadius: BorderRadius.circular(2))),
                  const SizedBox(width: 8),
                  SizedBox(width: 72, child: Text(l.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(Grey.g76))),
                  Expanded(child: Text(n.split('\n').first, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(Grey.g91))),
                ],
              ),
            ),
          ),
      ],
    );
  }
}
