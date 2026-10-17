// The desk's Markers tool: the document's marks (moments with a name) along its time, the playhead among them; a tap on
// the strip or a row goes there, + marks where the playhead is, a name is written over in place, x takes one away.
// The marks are the inside's (motolii_mark, motolii_marks): a mark said with no name is taken away.
import 'package:flutter/services.dart';
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
            children: [for (final (at, name) in marks) _Row(at: at, name: name, here: (at * Ws.fps).round() == ws.frame)],
          ),
        ),
      ],
    );
  }
}

/// The document's time end to end: each mark a tick, the playhead a line; a tap goes there.
class _Strip extends StatelessWidget {
  const _Strip({required this.marks});
  final List<(double, String)> marks;
  @override
  Widget build(BuildContext context) {
    final ws = WsScope.of(context);
    return LayoutBuilder(
      builder: (context, box) {
        void go(double x) => ws.frame = ((x - WsT.gap) / (box.maxWidth - 2 * WsT.gap) * Ws.duration).round().clamp(0, Ws.duration);
        return GestureDetector(
          behavior: HitTestBehavior.opaque,
          onTapDown: (d) => go(d.localPosition.dx),
          onHorizontalDragUpdate: (d) => go(d.localPosition.dx),
          child: CustomPaint(painter: _StripPaint(marks, ws.frame / Ws.duration, Ws.duration / Ws.fps)),
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
    for (final (at, _) in marks) {
      final x = r.left + r.width * (seconds > 0 ? (at / seconds).clamp(0.0, 1.0) : 0);
      c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: Offset(x, r.center.dy), width: 3, height: r.height - 4), const Radius.circular(1.5)), Paint()..color = Pop.toneTextAnim);
    }
    final x = r.left + r.width * head.clamp(0.0, 1.0);
    c.drawLine(Offset(x, r.top - 4), Offset(x, r.bottom + 4), Paint()
      ..color = WsT.accent
      ..strokeWidth = 1.5);
  }

  @override
  bool shouldRepaint(_StripPaint o) => o.head != head || o.marks != marks || o.seconds != seconds;
}

/// One mark: its moment (a tap goes there), its name (written over in place: said when the field is left or Return
/// is pressed; emptied, the mark is taken away), x to take it away.
class _Row extends StatefulWidget {
  const _Row({required this.at, required this.name, required this.here});
  final double at;
  final String name;
  final bool here;
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

  String get _time {
    final f = (widget.at * Ws.fps).round(), s = f ~/ Ws.fps, fr = f % Ws.fps;
    return '${(s ~/ 60).toString().padLeft(2, '0')}:${(s % 60).toString().padLeft(2, '0')}:${fr.toString().padLeft(2, '0')}';
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
            onTap: () => ws.frame = (widget.at * Ws.fps).round(),
            child: Row(
              children: [
                Container(width: 4, height: 14, decoration: BoxDecoration(color: Pop.toneTextAnim, borderRadius: BorderRadius.circular(2))),
                const SizedBox(width: 8),
                Text(_time, style: T.value(widget.here ? WsT.accent : Grey.g76)),
              ],
            ),
          ),
          const SizedBox(width: 10),
          Expanded(
            child: EditableText(
              controller: _words,
              focusNode: _focus,
              style: T.name(Grey.g91),
              cursorColor: WsT.accent,
              backgroundCursorColor: Grey.g38,
              maxLines: 1,
              inputFormatters: [LengthLimitingTextInputFormatter(40)],
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
