// The desk's Markers tool: the document's marks (moments with a name) along its time, the playhead among them; a tap on
// the strip or a row goes there, + marks where the playhead is, a name is written over in place, x takes one away.
// The marks are the inside's (motolii_mark, motolii_marks): a mark said with no name is taken away.
import 'package:flutter/widgets.dart';

import '../sets/inspector/inspector_parts.dart' show Pop;
import '../tokens.dart';
import 'desk_kit.dart';
import 'ws.dart';

class DeskMarkers extends StatelessWidget {
  const DeskMarkers({super.key});

  static void _say(Ws ws, double at, String name) {
    if (ws.inside?.mark(at, name) ?? false) ws.reread();
  }

  /// A new mark where the playhead is, named by its order (M1, M2...), unless one is there already.
  static void _add(Ws ws) {
    final at = ws.frame / Ws.fps, marks = ws.marks;
    if (marks.any((m) => (m.$1 - at).abs() < 0.5 / Ws.fps)) return;
    var n = marks.length + 1;
    while (marks.any((m) => m.$2 == 'M$n')) {
      n++;
    }
    _say(ws, at, 'M$n');
  }

  /// The mark the playhead last went past (or is on): where it is in the document's sections.
  static double? _passed(Ws ws, List<(double, String)> marks) {
    double? last;
    for (final (at, _) in marks) {
      if ((at * Ws.fps).round() <= ws.frame) last = at;
    }
    return last;
  }

  /// A mark moved to another moment, its name kept: one step.
  static void _move(Ws ws, double from, double to, String name) {
    final m = ws.inside;
    if (m == null || (from - to).abs() < 0.5 / Ws.fps) return;
    if (m.together(() => m.mark(from, '') && m.mark(to, name))) ws.reread();
  }

  @override
  Widget build(BuildContext context) {
    final ws = WsScope.of(context), marks = ws.marks;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        DeskStrip(left: [DeskReadout(value: '${marks.length}', unit: marks.length == 1 ? 'mark' : 'marks')], right: [
          DeskIconButton(key: const ValueKey('marks.add'), glyph: (ink) => _Plus(ink), onTap: () => _add(ws)),
        ]),
        SizedBox(height: 34, child: _Strip(marks: marks)),
        Expanded(
          child: ListView(
            padding: const EdgeInsets.symmetric(horizontal: WsT.gap, vertical: 4),
            children: [
              for (final (at, name) in marks)
                _Row(key: ValueKey(at), at: at, name: name, here: at == _passed(ws, marks), past: (at * Ws.fps).round() > Ws.duration),
            ],
          ),
        ),
      ],
    );
  }
}

/// The document's time end to end: each mark a tick, the playhead a line; a tap goes there.
class _Strip extends StatefulWidget {
  const _Strip({required this.marks});
  final List<(double, String)> marks;
  @override
  State<_Strip> createState() => _StripState();
}

/// A tap goes there; a drag that begins on a tick carries that mark (let go: moved, its name kept), else the playhead.
class _StripState extends State<_Strip> {
  (double, String)? _carried;
  double? _to;
  @override
  Widget build(BuildContext context) {
    final ws = WsScope.of(context), seconds = Ws.duration / Ws.fps;
    return LayoutBuilder(
      builder: (context, box) {
        final wide = box.maxWidth - 2 * WsT.gap;
        double at(double x) => (((x - WsT.gap) / wide).clamp(0.0, 1.0) * Ws.duration).round() / Ws.fps;
        void go(double x) => ws.frame = (at(x) * Ws.fps).round();
        return GestureDetector(
          behavior: HitTestBehavior.opaque,
          onTapDown: (d) => go(d.localPosition.dx),
          onHorizontalDragDown: (d) {   // (where the hand came down: a tick there is taken, before the drag is known as one)
            final x = d.localPosition.dx;
            final near = widget.marks.where((m) => ((WsT.gap + wide * (m.$1 / seconds).clamp(0.0, 1.0)) - x).abs() < 6).firstOrNull;
            setState(() {
              _carried = near;
              _to = near?.$1;
            });
          },
          onHorizontalDragStart: (d) => _carried == null ? go(d.localPosition.dx) : null,
          onHorizontalDragUpdate: (d) => _carried == null ? go(d.localPosition.dx) : setState(() => _to = at(d.localPosition.dx)),
          onHorizontalDragEnd: (_) {
            final (c, to) = (_carried, _to);
            setState(() {
              _carried = null;
              _to = null;
            });
            if (c != null && to != null) DeskMarkers._move(ws, c.$1, to, c.$2);
          },
          child: CustomPaint(painter: _StripPaint([for (final m in widget.marks) m == _carried ? (_to!, m.$2) : m], ws.frame / Ws.duration, seconds)),
        );
      },
    );
  }
}

class _StripPaint extends CustomPainter {
  _StripPaint(this.marks, this.head, this.seconds);
  final List<(double, String)> marks;
  final double head, seconds;
  @override
  void paint(Canvas c, Size s) {
    final r = Rect.fromLTRB(WsT.gap, 8, s.width - WsT.gap, s.height - 8);
    c.drawRRect(RRect.fromRectAndRadius(r, const Radius.circular(4)), Paint()..color = WsT.well);
    final x = r.left + r.width * head.clamp(0.0, 1.0);   // (the playhead under the marks: a mark it is on stays seen)
    c.drawLine(Offset(x, r.top - 4), Offset(x, r.bottom + 4), Paint()
      ..color = WsT.accent
      ..strokeWidth = 1.5);
    var beyond = false;
    for (final (at, _) in marks) {
      if (seconds > 0 && at > seconds + 1e-9) { beyond = true; continue; }
      final x = r.left + r.width * (seconds > 0 ? (at / seconds).clamp(0.0, 1.0) : 0);
      c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: Offset(x, r.center.dy), width: 3, height: r.height + 2), const Radius.circular(1.5)), Paint()..color = Pop.toneTextAnim);
    }
    if (beyond) {   // (marks past the document's end: an arrow off its right edge)
      c.drawPath(Path()
        ..moveTo(r.right + 2, r.center.dy - 5)
        ..lineTo(r.right + 7, r.center.dy)
        ..lineTo(r.right + 2, r.center.dy + 5)
        ..close(), Paint()..color = Pop.toneTextAnim.withValues(alpha: .6));
    }
  }

  @override
  bool shouldRepaint(_StripPaint o) => o.head != head || o.marks != marks || o.seconds != seconds;
}

/// One mark: its moment (a tap goes there), its name (written over in place: said when the field is left or Return
/// is pressed; emptied, the mark is taken away), x to take it away.
class _Row extends StatefulWidget {
  const _Row({super.key, required this.at, required this.name, required this.here, required this.past});
  final double at;
  final String name;
  final bool here, past;   // (past: beyond the document's end, not to be gone to)
  @override
  State<_Row> createState() => _RowState();
}

class _RowState extends State<_Row> {
  late final _words = TextEditingController(text: widget.name);
  final _focus = FocusNode();

  @override
  void initState() {
    super.initState();
    _focus.addListener(() {
      if (!_focus.hasFocus) _say();
    });
  }

  @override
  void didUpdateWidget(_Row o) {
    super.didUpdateWidget(o);
    if (!_focus.hasFocus && _words.text != widget.name) _words.text = widget.name;
  }

  void _say() {
    final now = _words.text.trim();
    if (now == widget.name || !mounted) return;
    final ws = WsScope.read(context);
    if (now.isEmpty) {
      _words.text = widget.name; // (an empty name is not a way to take it away by accident: x does that)
      return;
    }
    DeskMarkers._say(ws, widget.at, now);
  }

  @override
  void dispose() {
    _focus.dispose();
    _words.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final ws = WsScope.read(context);
    return Container(
      height: 28,
      margin: const EdgeInsets.only(bottom: 2),
      padding: const EdgeInsets.symmetric(horizontal: 6),
      decoration: BoxDecoration(color: widget.here ? WsT.well : null, borderRadius: BorderRadius.circular(5)),
      child: Row(
        children: [
          GestureDetector(
            onTap: widget.past ? null : () => ws.frame = (widget.at * Ws.fps).round(),
            child: Row(
              children: [
                Container(width: 4, height: 14, decoration: BoxDecoration(color: Pop.toneTextAnim, borderRadius: BorderRadius.circular(2))),
                const SizedBox(width: 8),
                Text(ws.timecode((widget.at * Ws.fps).round()), style: T.value(widget.past ? Grey.g38 : widget.here ? WsT.accent : Grey.g76)),   // (as the timeline counts it)
              ],
            ),
          ),
          const SizedBox(width: 10),
          Expanded(
            child: EditableText(
              controller: _words,
              focusNode: _focus,
              style: T.name(widget.past ? Grey.g56 : Grey.g91),
              cursorColor: WsT.accent,
              backgroundCursorColor: Grey.g38,
              maxLines: 1,
              onSubmitted: (_) => _focus.unfocus(),
            ),
          ),
          DeskIconButton(key: const ValueKey('mark.remove'), glyph: (ink) => _Cross(ink), onTap: () => DeskMarkers._say(ws, widget.at, '')),
        ],
      ),
    );
  }
}

class _Plus extends CustomPainter {
  _Plus(this.ink);
  final Color ink;
  @override
  void paint(Canvas c, Size s) {
    final p = Paint()
      ..color = ink
      ..strokeWidth = 1.5
      ..strokeCap = StrokeCap.round;
    c.drawLine(Offset(s.width / 2, s.height * .25), Offset(s.width / 2, s.height * .75), p);
    c.drawLine(Offset(s.width * .25, s.height / 2), Offset(s.width * .75, s.height / 2), p);
  }

  @override
  bool shouldRepaint(_Plus o) => o.ink != ink;
}

class _Cross extends CustomPainter {
  _Cross(this.ink);
  final Color ink;
  @override
  void paint(Canvas c, Size s) {
    final p = Paint()
      ..color = ink
      ..strokeWidth = 1.3
      ..strokeCap = StrokeCap.round;
    c.drawLine(Offset(s.width * .32, s.height * .32), Offset(s.width * .68, s.height * .68), p);
    c.drawLine(Offset(s.width * .68, s.height * .32), Offset(s.width * .32, s.height * .68), p);
  }

  @override
  bool shouldRepaint(_Cross o) => o.ink != ink;
}
