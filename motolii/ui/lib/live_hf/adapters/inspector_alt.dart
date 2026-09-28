// TEMPORARY — the Skin Swap Proof for the Inspector. A deliberately different skin over the same InspectorSession: a
// plain table (name · value typed in · key), every number an EditableText committed on Enter, effects as rows with
// on/off and remove. If this edits what the instrument edits with no change outside it, the Inspector's meaning lives
// in the session and its stores. Delete after the proof.
import 'package:flutter/widgets.dart';

import '../../hf/bp/common.dart' show mono;
import '../../hf/insp/rows.dart';
import '../../hf/insp/slot.dart';
import '../../hf/neutral.dart';
import '../../session/editor_session.dart';
import 'inspector_session.dart';

final altInspectorSkin = ValueNotifier(false);

class AltInspector extends StatelessWidget {
  const AltInspector({super.key, required this.c});
  final EditorSession c;
  @override
  Widget build(BuildContext context) {
    final s = InspectorSession.of(c);
    return ListenableBuilder(
      listenable: Listenable.merge([s, c.slice('altInspector', const ['layers', 'selectedIds', 'documentRevision'])]),
      builder: (context, _) {
        final subject = s.subject;
        final lines = <Widget>[Text('SUBJECT  ${subject.runtimeType}', style: mono(10, c: N.g63))];
        final t = s.transform;
        if (t != null) lines.addAll(_store(t, 'TRANSFORM'));
        if (subject case InspectorLayer(layer: final l?, effects: final effects)) {
          for (final e in effects) {
            lines.add(Row(children: [
              Expanded(child: Text('EFFECT  ${e['name']}', style: mono(10, c: N.g86))),
              GestureDetector(onTap: () => s.enableEffect(l['id'] as int, e['id'], e['enabled'] == false), child: Text(e['enabled'] == false ? '[off]' : '[on]', style: mono(10, c: N.g95))),
              const SizedBox(width: 8),
              GestureDetector(onTap: () => s.removeEffect(l['id'] as int, e['id']), child: Text('[x]', style: mono(10, c: N.g95))),
            ]));
            lines.addAll(_store(s.effect(l['id'] as int, e['id']), null));
          }
        }
        return ColoredBox(color: N.g00, child: ListView(padding: const EdgeInsets.all(8), children: lines));
      },
    );
  }

  List<Widget> _store(ParamStore st, String? title) => [
        if (title != null) Padding(padding: const EdgeInsets.only(top: 8), child: Text(title, style: mono(10, c: N.g86))),
        for (final r in st.rows)
          if (r['value'] is num || (r['value'] is List && (r['value'] as List).every((v) => v is num)))
            Row(children: [
              SizedBox(width: 120, child: Text('${r['id']}', style: mono(10, c: N.g69), overflow: TextOverflow.ellipsis)),
              for (final axis in r['value'] is List ? [for (var a = 0; a < (r['value'] as List).length; a++) a] : [null])
                Expanded(child: Row(children: [
                  Expanded(child: _Number(Slot(st, '${r['id']}', axis))),
                  GestureDetector(key: ValueKey('alt-plus-${r['id']}-$axis'), onTap: () { final sl = Slot(st, '${r['id']}', axis); sl.typed(sl.value + 10); }, child: Text('+10 ', style: mono(10, c: N.g95))),
                ])),
              if (st.keyable) GestureDetector(key: ValueKey('alt-key-${r['id']}'), onTap: () => st.toggleKey('${r['id']}'), child: Text(' ◇', style: mono(10, c: N.g95))),
            ]),
      ];
}

class _Number extends StatefulWidget {
  const _Number(this.slot);
  final Slot slot;
  @override
  State<_Number> createState() => _NumberState();
}

class _NumberState extends State<_Number> {
  final ctl = TextEditingController(), focus = FocusNode();
  @override
  void dispose() {
    ctl.dispose();
    focus.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    if (!focus.hasFocus) ctl.text = widget.slot.format(widget.slot.value);
    return GestureDetector(
      onTap: focus.requestFocus,
      child: Container(
      margin: const EdgeInsets.all(1),
      color: N.g13,
      child: EditableText(
        controller: ctl,
        focusNode: focus,
        style: mono(11, c: N.g100),
        cursorColor: N.g100,
        backgroundCursorColor: N.g38,
        onSubmitted: (v) {
          final d = double.tryParse(v);
          if (d != null) widget.slot.typed(d / widget.slot.displayScale);
        },
      ),
      ),
    );
  }
}
