import 'package:flutter/widgets.dart';

import '../../browser/parts.dart';

import '../../theme/glyphs.dart';
import '../../session/editor_session.dart';
import '../../theme/neutral.dart';
import '../../theme/metrics.dart' show Dn, Surface;

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
        color: Surface.base,
        padding: EdgeInsets.all(Surface.panelInset),
        child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          SizedBox(width: Surface.px(22), height: Surface.px(22), child: CustomPaint(painter: HgPainter(HG.search, Surface.muted, Surface.base))),
          SizedBox(height: Surface.px(9)),
          Padding(padding: EdgeInsets.only(bottom: Surface.inlineGap), child: Text('WEBSITE', style: sans(Dn.microSize, c: N.g51, w: FontWeight.w600, ls: 1.2))),
          Container(
            decoration: BoxDecoration(color: Surface.raised, borderRadius: BorderRadius.circular(Surface.px(4))),
            padding: EdgeInsets.symmetric(horizontal: Surface.sectionGap),
            child: EditableText(
              key: const ValueKey('web-url'),
              controller: _field,
              focusNode: FocusNode(),
              style: sans(Dn.nameSize, c: Surface.ink),
              cursorColor: Surface.ink,
              backgroundCursorColor: Surface.muted,
              onSubmitted: (v) => c.storeDesk('webUrl', v),
              onEditingComplete: () => c.storeDesk('webUrl', _field.text),
            ),
          ),
          SizedBox(height: Surface.sectionGap),
          GestureDetector(
            key: const ValueKey('web-open'),
            behavior: HitTestBehavior.opaque,
            onTap: () async {
              await c.flushEditors();
              await c.native('openWeb', {'url': _url});
            },
            child: Container(
              height: Surface.control,
              alignment: Alignment.center,
              decoration: BoxDecoration(color: Surface.raised, borderRadius: BorderRadius.circular(Surface.px(4))),
              child: Text('Open in browser', style: sans(Dn.nameSize, c: Surface.ink, w: FontWeight.w500)),
            ),
          ),
        ]),
      );
}
