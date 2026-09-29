import 'dart:math' as math;
import 'package:flutter/widgets.dart';
import '../bp/common.dart';
import '../bp/shell.dart' show emptyBody;
import 'common.dart';
import '../neutral.dart';

enum Mark { none, save, open, warn, error, end }

class Entry {
  const Entry(this.label, this.time, [this.mark = Mark.none]);
  final String label, time;
  final Mark mark;
}

const historyEntries = <Entry>[
  Entry('Open Project', '10:02', Mark.open),
  Entry('Add Layer', '10:04'),
  Entry('Move Layer 2', '10:06'),
  Entry('Scale Layer 2', '10:07'),
  Entry('Set Blend: Multiply', '10:11'),
  Entry('Save', '10:14', Mark.save),
  Entry('Keyframe Position', '10:18'),
  Entry('Ease Bezier', '10:19'),
  Entry('Missing Font', '10:21', Mark.warn),
  Entry('Set Camera FOV', '10:24'),
  Entry('Render Failed', '10:26', Mark.error),
  Entry('Delete Layer 4', '10:28'),
];

const _bright = N.g86;
const _redo = N.g26;
const _amber = Color(0xFFF08A3C);
const _red = Color(0xFFE2554F);

Color _markColor(Mark m) => switch (m) { Mark.warn => _amber, Mark.error => _red, Mark.save => kMint, Mark.open => kViolet, Mark.end => kBlue, _ => _bright };

/// One rail is the structure. The current step is a large ring, reached steps are a solid line,
/// the redo tail is dashed and dim. Marks are silhouettes on the rail; text is annotation.
///
/// With no [entries] it is the fixture (its own list, its own position). With a host's entries it draws them and asks the
/// host to go to one: [at] is the current index and [onGo] is how a click, Undo or Redo moves it.
class HistoryDesk extends StatefulWidget {
  const HistoryDesk({super.key, this.entries, this.at = 0, this.onGo, this.canGo = true, this.onUndo, this.onRedo});
  final List<Entry>? entries;
  final int at;
  final ValueChanged<int>? onGo;
  final bool canGo;

  /// Undo / Redo: one edit back or forward, what ⌘Z and ⇧⌘Z do (null: the buttons step along the list). A null
  /// callback with a host means there is nothing to undo or redo.
  final VoidCallback? onUndo, onRedo;
  @override
  State<HistoryDesk> createState() => _HistoryDeskState();
}

class _HistoryDeskState extends State<HistoryDesk> {
  int _at = 8;
  List<Entry> get entries => widget.entries ?? historyEntries;
  int get at => widget.entries == null ? _at : widget.at;
  final _scroll = ScrollController();
  static const _rowH = 38.0;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) => _follow(animate: false));
  }

  @override
  void dispose() {
    _scroll.dispose();
    super.dispose();
  }

  // Whatever moves the current step (click, Undo, Redo) brings it to the middle of the rail.
  void _follow({bool animate = true}) {
    if (!_scroll.hasClients) return;
    final pos = _scroll.position;
    final target = (at * _rowH + _rowH / 2 - pos.viewportDimension / 2 + 8).clamp(0.0, pos.maxScrollExtent);
    if (animate) { _scroll.animateTo(target, duration: const Duration(milliseconds: 160), curve: Curves.easeOut); } else { _scroll.jumpTo(target); }
  }

  @override
  void didUpdateWidget(HistoryDesk old) {
    super.didUpdateWidget(old);
    if (old.at != widget.at) WidgetsBinding.instance.addPostFrameCallback((_) => _follow());
  }

  void go(int i) {
    final to = i.clamp(0, entries.length - 1);
    if (widget.entries != null) {
      if (widget.canGo && to != at) widget.onGo?.call(to);
      return;
    }
    setState(() => _at = to);
    WidgetsBinding.instance.addPostFrameCallback((_) => _follow());
  }

  @override
  Widget build(BuildContext context) => DeskShell(
        kind: DeskKind.history,
        title: 'History',
        subtitle: 'UNDO · REDO',
        full: (c, s) => entries.isEmpty ? emptyBody('No history') : Column(children: [
          Expanded(
            child: ListView.builder(
              controller: _scroll,
              itemExtent: _rowH,
              padding: const EdgeInsets.fromLTRB(0, 8, 0, 8),
              itemCount: entries.length,
              itemBuilder: (_, i) => GestureDetector(key: ValueKey(i == at ? 'row-current' : 'row-$i'), behavior: HitTestBehavior.opaque, onTap: () => go(i), child: _row(i, _rowH)),
            ),
          ),
          Container(
            padding: const EdgeInsets.fromLTRB(14, 10, 14, 10),
            decoration: const BoxDecoration(border: Border(top: BorderSide(color: kRule2))),
            child: Row(children: [
              Expanded(child: widget.entries != null ? _btn('undo', 'Undo', '⌘Z', widget.onUndo != null, () => widget.onUndo?.call()) : _btn('undo', 'Undo', '⌘Z', at > 0, () => go(at - 1))),
              const SizedBox(width: 8),
              Expanded(child: widget.entries != null ? _btn('redo', 'Redo', '⇧⌘Z', widget.onRedo != null, () => widget.onRedo?.call()) : _btn('redo', 'Redo', '⇧⌘Z', at < entries.length - 1, () => go(at + 1))),
            ]),
          ),
        ]),
        strip: (c, s) => LayoutBuilder(builder: (context, b) => GestureDetector(
              key: const ValueKey('history-strip'),
              onTapDown: (d) => go(((d.localPosition.dx - 16) / (b.maxWidth - 32) * (entries.length - 1).clamp(1, 1 << 30)).round()),
              child: CustomPaint(size: Size(b.maxWidth, b.maxHeight), painter: _HRail(at, entries)),
            )),
        tall: (c, s) {
          final h = ((s.height - 16) / entries.length).clamp(14.0, 34.0);
          return Padding(
            padding: const EdgeInsets.symmetric(vertical: 8),
            child: Column(children: [for (var i = 0; i < entries.length; i++) GestureDetector(key: ValueKey(i == at ? 'row-current' : 'row-$i'), behavior: HitTestBehavior.opaque, onTap: () => go(i), child: _row(i, h, labels: false, w: s.width))]),
          );
        },
      );

  Widget _btn(String id, String l, String k, bool on, VoidCallback f) => GestureDetector(
        key: ValueKey('history-$id'),
        onTap: on ? f : null,
        child: Container(
          height: 36,
          decoration: BoxDecoration(color: kWell, border: Border.all(color: kRule2), borderRadius: BorderRadius.circular(3)),
          padding: const EdgeInsets.symmetric(horizontal: 12),
          child: Row(children: [Text(l, style: sans(12.5, c: on ? kInk : N.g38)), const Spacer(), Text(k, style: sans(11, c: kMuted))]),
        ),
      );

  Widget _row(int i, double h, {bool labels = true, double w = 310}) {
    final e = entries[i];
    final cur = i == at, reached = i <= at;
    final marked = e.mark != Mark.none;
    // Text is annotation: quiet by default; the current step and marked records speak.
    final ink = cur ? kInk : (reached ? (marked ? N.g76 : N.g56) : N.g33);
    return Container(
      height: h,
      color: cur ? kYellow.withValues(alpha: .10) : null,
      child: Row(children: [
        SizedBox(width: labels ? 58 : w, height: h, child: CustomPaint(painter: _Node(i == 0, i == entries.length - 1, cur, reached, i < at, e.mark))),
        if (labels) ...[
          Expanded(child: Text(e.label, softWrap: false, overflow: TextOverflow.clip, style: sans(cur ? 13.5 : 12, c: ink, w: cur ? FontWeight.w600 : FontWeight.w400))),
          Padding(padding: const EdgeInsets.only(right: 16), child: Text(e.time, style: mono(9.5, c: reached ? N.g44 : N.g26))),
        ],
      ]),
    );
  }
}

void _dashV(Canvas c, double x, double y0, double y1, Paint p) {
  for (var y = y0; y < y1; y += 6) { c.drawLine(Offset(x, y), Offset(x, math.min(y + 3, y1)), p); }
}

void _drawMark(Canvas c, Offset o, Mark m, Color col) {
  final p = Paint()..color = col..style = PaintingStyle.stroke..strokeWidth = 1.7..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round;
  switch (m) {
    case Mark.save:
      c.drawRect(Rect.fromCenter(center: o, width: 9, height: 9), Paint()..color = col);
    case Mark.open:
      c.drawPath(Path()..moveTo(o.dx - 5, o.dy + 4)..lineTo(o.dx - 5, o.dy - 4)..lineTo(o.dx - 1.5, o.dy - 4)..lineTo(o.dx, o.dy - 2)..lineTo(o.dx + 5, o.dy - 2)..lineTo(o.dx + 5, o.dy + 4)..close(), p);
    case Mark.warn:
      c.drawPath(Path()..moveTo(o.dx, o.dy - 6)..lineTo(o.dx + 6, o.dy + 5)..lineTo(o.dx - 6, o.dy + 5)..close(), Paint()..color = col);
    case Mark.error:
      c.drawCircle(o, 6.5, Paint()..color = col);
      c.drawLine(o + const Offset(-2.6, -2.6), o + const Offset(2.6, 2.6), Paint()..color = kWell..style = PaintingStyle.stroke..strokeWidth = 1.7..strokeCap = StrokeCap.round);
      c.drawLine(o + const Offset(2.6, -2.6), o + const Offset(-2.6, 2.6), Paint()..color = kWell..style = PaintingStyle.stroke..strokeWidth = 1.7..strokeCap = StrokeCap.round);
    case Mark.end:
      c.drawRect(Rect.fromCenter(center: o, width: 10, height: 10), p);
    case Mark.none:
      break;
  }
}

void _dot(Canvas c, Offset o, Mark mark, bool cur, bool reached, double r) {
  if (cur) {
    c.drawCircle(o, r + 7, Paint()..color = kYellow.withValues(alpha: .25));
    c.drawCircle(o, r + 2, Paint()..color = kWell);
    c.drawCircle(o, r + 2, Paint()..color = kYellow..style = PaintingStyle.stroke..strokeWidth = 3.4);
    if (mark == Mark.none) { c.drawCircle(o, r - 2.5, Paint()..color = kInk); } else { _drawMark(c, o, mark, _markColor(mark)); }
  } else if (mark != Mark.none) {
    c.drawCircle(o, 12, Paint()..color = kWell);
    _drawMark(c, o, mark, reached ? _markColor(mark) : _markColor(mark).withValues(alpha: .32));
  } else if (reached) {
    c.drawCircle(o, 5.4, Paint()..color = kBlue);
  } else {
    c.drawCircle(o, 3.4, Paint()..color = kWell);
    c.drawCircle(o, 3.4, Paint()..color = N.g33..style = PaintingStyle.stroke..strokeWidth = 1.3);
  }
}

class _Node extends CustomPainter {
  _Node(this.first, this.last, this.cur, this.reached, this.lineReached, this.mark);
  final bool first, last, cur, reached, lineReached;
  final Mark mark;
  @override
  void paint(Canvas c, Size s) {
    final x = s.width > 100 ? 28.0 : s.width / 2, y = s.height / 2;
    final solid = Paint()..color = kBlue..strokeWidth = 3.4;
    final dash = Paint()..color = _redo..strokeWidth = 1.6;
    if (!first) { if (reached) { c.drawLine(Offset(x, 0), Offset(x, y), solid); } else { _dashV(c, x, 0, y, dash); } }
    if (!last) { if (lineReached) { c.drawLine(Offset(x, y), Offset(x, s.height), solid); } else { _dashV(c, x, y, s.height, dash); } }
    _dot(c, Offset(x, y), mark, cur, reached, 8);
  }
  @override
  bool shouldRepaint(_Node o) => true;
}

class _HRail extends CustomPainter {
  _HRail(this.at, this.entries);
  final int at;
  final List<Entry> entries;
  @override
  void paint(Canvas c, Size s) {
    final n = entries.length;
    final y = s.height / 2;
    Offset p(int i) => Offset(16 + (s.width - 32) * i / (n < 2 ? 1 : n - 1), y);
    c.drawLine(p(0), p(at), Paint()..color = kBlue..strokeWidth = 3.4);
    for (var x = p(at).dx; x < p(n - 1).dx; x += 6) { c.drawLine(Offset(x, y), Offset(math.min(x + 3, p(n - 1).dx), y), Paint()..color = _redo..strokeWidth = 1.6); }
    for (var i = 0; i < n; i++) { _dot(c, p(i), entries[i].mark, i == at, i <= at, 8); }
  }
  @override
  bool shouldRepaint(_HRail o) => o.at != at || o.entries != entries;
}
