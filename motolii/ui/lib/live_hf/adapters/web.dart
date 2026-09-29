import 'package:flutter/widgets.dart';

import '../../hf/bp/common.dart';
import '../../hf/bp/shell.dart' show kTile;
import '../../hf/desk/common.dart' show kInk;
import '../../hf/glyphs.dart';
import '../../session/editor_session.dart';
import '../../hf/neutral.dart';

/// The Web desk's New face: one URL, kept as the document's own desk setting (`storeDesk('webUrl', ...)`, the same
/// operation Classic's WebPanel uses), and a button that asks the host to open it (`native('openWeb', ...)`).
class NewWeb extends StatefulWidget {
  const NewWeb({super.key, required this.controller});
  final EditorSession controller;
  @override
  State<NewWeb> createState() => _NewWebState();
}

class _NewWebState extends State<NewWeb> {
  EditorSession get c => widget.controller;
  late final _field = TextEditingController(text: _url);
  String get _url => '${c.deskWork.value['webUrl'] ?? 'https://www.pinterest.com/'}';

  @override
  void dispose() {
    _field.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Container(
        color: kGround,
        padding: const EdgeInsets.all(9),
        child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          SizedBox(width: 30, height: 30, child: CustomPaint(painter: HgPainter(HG.search, kMuted, kGround))),
          const SizedBox(height: 9),
          Padding(padding: const EdgeInsets.only(bottom: 3), child: Text('WEBSITE', style: sans(9.5, c: N.g51, w: FontWeight.w600, ls: 1.2))),
          Container(
            decoration: BoxDecoration(color: kTile, borderRadius: BorderRadius.circular(4)),
            padding: const EdgeInsets.symmetric(horizontal: 6),
            child: EditableText(
              key: const ValueKey('web-url'),
              controller: _field,
              focusNode: FocusNode(),
              style: sans(11, c: kInk),
              cursorColor: kInk,
              backgroundCursorColor: kMuted,
              onSubmitted: (v) => c.storeDesk('webUrl', v),
              onEditingComplete: () => c.storeDesk('webUrl', _field.text),
            ),
          ),
          const SizedBox(height: 6),
          GestureDetector(
            key: const ValueKey('web-open'),
            behavior: HitTestBehavior.opaque,
            onTap: () async {
              await c.flushEditors();
              await c.native('openWeb', {'url': _url});
            },
            child: Container(
              height: 21,
              alignment: Alignment.center,
              decoration: BoxDecoration(color: kTile, borderRadius: BorderRadius.circular(4)),
              child: Text('Open in browser', style: sans(11, c: kInk, w: FontWeight.w500)),
            ),
          ),
        ]),
      );
}
