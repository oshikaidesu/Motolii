import 'package:flutter/widgets.dart';
import '../../foundation/notes_view.dart';
import '../../foundation/panel_controls.dart' show EditorDraftField;
import '../bp/common.dart';
import 'common.dart';
import '../neutral.dart';

/// The cards' look in the finished Notes: flat colour, a violet pill for a reference, a dotted ground.
const hfNoteLook = NoteLook(
  ground: kGround,
  dots: N.g15,
  tints: [kYellow, kPink, kBlue, kMint],
  reference: kViolet,
  ink: N.g10,
  accent: kAccent,
  raised: kInk,
);

/// The finished Notes desk placing what the desk gives it (pages, canvas, actions): a row of tools, the pages as
/// chips, the page's title, the canvas, and a bar that says where the view is. Nothing here is a note of its own.
class NotesSkin extends StatelessWidget {
  const NotesSkin(this.view, {super.key});
  final NotesView view;

  @override
  Widget build(BuildContext context) => DeskShell(
        kind: DeskKind.notes,
        title: 'Notes',
        subtitle: 'PAGES · REFERENCES',
        full: (c, s) => Column(children: [_tools(), _title(), Expanded(child: _canvas()), _bar()]),
        strip: (c, s) => Padding(padding: const EdgeInsets.fromLTRB(8, 2, 8, 8), child: _canvas()),
        tall: (c, s) => Padding(padding: const EdgeInsets.fromLTRB(8, 4, 8, 8), child: _canvas()),
      );

  Widget _canvas() => ClipRRect(borderRadius: BorderRadius.circular(4), child: view.canvas);

  Widget _key(String key, int icon, String tip, VoidCallback f) => GestureDetector(
        key: ValueKey(key),
        onTap: f,
        child: Container(
          height: 30,
          width: 30,
          margin: const EdgeInsets.only(right: 4),
          decoration: BoxDecoration(border: Border.all(color: kRule2), borderRadius: BorderRadius.circular(3)),
          child: CustomPaint(painter: _ToolP(icon)),
        ),
      );

  Widget _tools() => Container(
        height: 44,
        padding: const EdgeInsets.symmetric(horizontal: 10),
        child: Row(children: [
          _key('notes-paste', 1, 'Paste', view.paste),
          _key('notes-image', 2, 'Insert image', view.insertImage),
          _key('notes-link', 3, 'Link selection', view.linkSelection),
          if (view.pageTitle != null) _key('notes-delete-page', 4, 'Delete page', view.deletePage),
          const Spacer(),
          Flexible(
            child: SingleChildScrollView(
              scrollDirection: Axis.horizontal,
              reverse: true,
              child: Row(children: [
                for (final (i, p) in view.pages.indexed)
                  GestureDetector(
                    key: ValueKey('notes-page-$i'),
                    onTap: () => view.selectPage(p.id),
                    child: Container(
                      height: 26,
                      constraints: const BoxConstraints(minWidth: 26, maxWidth: 90),
                      margin: const EdgeInsets.only(left: 4),
                      padding: const EdgeInsets.symmetric(horizontal: 7),
                      alignment: Alignment.center,
                      decoration: BoxDecoration(border: Border.all(color: p.active ? kAccent : kRule2), borderRadius: BorderRadius.circular(3), color: p.active ? kAccentDim.withValues(alpha: .4) : null),
                      child: Text(p.title.isEmpty ? '${i + 1}' : p.title, softWrap: false, overflow: TextOverflow.clip, style: sans(11, c: p.active ? kInk : kMuted)),
                    ),
                  ),
                GestureDetector(
                  key: const ValueKey('notes-new-page'),
                  onTap: view.newPage,
                  child: Container(width: 26, height: 26, margin: const EdgeInsets.only(left: 4), alignment: Alignment.center, decoration: BoxDecoration(border: Border.all(color: kRule2), borderRadius: BorderRadius.circular(3)), child: Text('+', style: sans(14, c: kMuted))),
                ),
              ]),
            ),
          ),
        ]),
      );

  Widget _title() => view.pageTitle == null
      ? Padding(
          padding: const EdgeInsets.fromLTRB(12, 4, 12, 8),
          child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
            Text('Click anywhere to write', style: sans(11.5, c: kMuted)),
            if (view.legacy)
              GestureDetector(
                key: const ValueKey('notes-legacy'),
                onTap: view.importLegacy,
                child: Padding(padding: const EdgeInsets.only(top: 6), child: Text('Import previous text / references', style: sans(11, c: kAccent))),
              ),
          ]),
        )
      : Padding(
          padding: const EdgeInsets.fromLTRB(12, 0, 12, 6),
          child: EditorDraftField(key: ValueKey('page-title:${view.pageTitle}'), value: view.pageTitle!, label: 'Page title', onCommit: (t) async => view.renamePage(t)),
        );

  // Where the view is: the page, its cards, the scale, and the ways to change or reset it.
  Widget _bar() => ValueListenableBuilder<Matrix4>(
        valueListenable: view.transform,
        builder: (_, m, __) {
          final z = m.getMaxScaleOnAxis();
          Widget step(String key, String t, double to) => GestureDetector(key: ValueKey(key), onTap: () => view.setZoom(to.clamp(.25, 2.0)), child: Padding(padding: const EdgeInsets.symmetric(horizontal: 6), child: Text(t, style: sans(18, c: N.g82))));
          return Container(
            height: 40,
            padding: const EdgeInsets.symmetric(horizontal: 12),
            child: Row(children: [
              Expanded(child: Text('${view.blocks} blocks', softWrap: false, overflow: TextOverflow.clip, style: sans(11, c: kMuted))),
              step('zoom-out', '−', z - .25),
              SizedBox(width: 48, child: Text('${(z * 100).round()}%', key: const ValueKey('zoom-label'), textAlign: TextAlign.center, style: mono(11, c: N.g82))),
              step('zoom-in', '+', z + .25),
              const SizedBox(width: 10),
              GestureDetector(key: const ValueKey('zoom-fit'), onTap: view.resetView, child: Text('Reset', style: sans(11, c: kAccent))),
            ]),
          );
        },
      );
}

class _ToolP extends CustomPainter {
  _ToolP(this.i);
  final int i;
  @override
  void paint(Canvas c, Size s) {
    final p = Paint()..color = N.g82..style = PaintingStyle.stroke..strokeWidth = 1.4..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round;
    final m = s.center(Offset.zero);
    switch (i) {
      case 1:
        c.drawRect(Rect.fromCenter(center: m, width: 12, height: 12), p);
        c.drawLine(m + const Offset(-3, -1), m + const Offset(3, -1), p);
        c.drawLine(m + const Offset(-3, 2), m + const Offset(1, 2), p);
      case 2:
        c.drawRect(Rect.fromCenter(center: m, width: 14, height: 11), p);
        c.drawPath(Path()..moveTo(m.dx - 6, m.dy + 4)..lineTo(m.dx - 2, m.dy)..lineTo(m.dx + 1, m.dy + 3)..lineTo(m.dx + 3, m.dy + 1)..lineTo(m.dx + 6, m.dy + 4), p);
      case 3:
        c.drawCircle(m + const Offset(-3, 0), 3.5, p);
        c.drawCircle(m + const Offset(3, 0), 3.5, p);
      default:
        c.drawLine(m + const Offset(-5, -5), m + const Offset(5, 5), p);
        c.drawLine(m + const Offset(5, -5), m + const Offset(-5, 5), p);
    }
  }
  @override
  bool shouldRepaint(_ToolP o) => false;
}
