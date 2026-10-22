// The Trim Paths card's drawing, in the Transform card's manner (a control is a drawing of its own result, pulled by
// hand): the path as one line, the part that is left drawn bright between its start and its end, slid along by the
// offset (round the end and back in at the beginning, as After Effects' trim does); take an end to cut there, take the
// bright part to slide it. Its numbers stay below as the card's tiles.
import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_direct_transform.dart' show TransformZone;
import 'package:lab_book/sets/inspector/inspector_parts.dart';
import 'package:lab_book/sets/inspector/inspector_pulled.dart';
import 'package:lab_book/tokens.dart';

class TrimDrawing extends StatefulWidget {
  const TrimDrawing({super.key});
  static double get height => Shape.z(44.0);
  @override
  State<TrimDrawing> createState() => _TrimDrawingState();
}

enum _Knob { begin, end, slide }

class _TrimDrawingState extends State<TrimDrawing> {
  _Knob? _held;
  double _from = 0; // (where along the line the hand took the bright part, 0 to 1)

  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    return ListenableBuilder( // (drawn again when one of its numbers is told anew, by its tiles or the inside)
      listenable: Listenable.merge([for (final id in const ['v_trim', 'v_trim_begin', 'v_trim_offset']) doc.watch(id)]),
      builder: (context, _) => _drawing(doc),
    );
  }

  Widget _drawing(Doc doc) {
    return Padding(
      padding: EdgeInsets.only(bottom: Shape.s(6)),
      child: SizedBox(
        height: TrimDrawing.height,
        child: LayoutBuilder(
          builder: (context, box) {
            // (the card counts them in hundredths: trim is what is cut off the end, the offset is in turns)
            final g = _Trim(box.biggest, begin: (doc.get('v_trim_begin') ?? 0) / 100, end: 1 - (doc.get('v_trim') ?? 0) / 100, slide: (doc.get('v_trim_offset') ?? 0) / 100);
            _Knob? pick(Offset p) {
              if ((p - g.at(g.begin + g.slide)).distance <= 9) return _Knob.begin;
              if ((p - g.at(g.end + g.slide)).distance <= 9) return _Knob.end;
              return (p.dy - g.y).abs() <= 10 && g.kept(g.along(p.dx)) ? _Knob.slide : null;
            }

            return Pulled(
              takes: (at) => pick(at) != null,
              onDown: (d) {
                _held = pick(d);
                _from = g.along(d.dx);
                if (_held != null) doc.beginGesture();
              },
              onMove: (d) {
                final u = g.along(d.dx);
                switch (_held) {
                  case _Knob.begin:
                    doc.set('v_trim_begin', (_Trim.wrap(u - g.slide).clamp(0.0, g.end) * 100));
                  case _Knob.end:
                    doc.set('v_trim', ((1 - _Trim.wrap(u - g.slide)).clamp(0.0, 1 - g.begin) * 100));
                  case _Knob.slide:
                    doc.set('v_trim_offset', (doc.get('v_trim_offset') ?? 0) + (u - _from) * 100); // (from what it is now: the drawing may not be drawn again between two moves)
                    _from = u;
                  case null:
                }
              },
              onUp: () {
                if (_held != null) doc.endGesture();
                setState(() => _held = null);
              },
              child: CustomPaint(size: box.biggest, painter: _TrimPaint(g, _held)),
            );
          },
        ),
      ),
    );
  }
}

/// Where the line lies and where a share of it is.
class _Trim {
  _Trim(this.size, {required this.begin, required this.end, required this.slide});
  final Size size;
  final double begin, end, slide;
  static const pad = 16.0;
  double get y => size.height / 2;
  static double wrap(double u) => u - u.floorToDouble();
  Offset at(double u) => Offset(pad + wrap(u == 1 ? 1 - 1e-9 : u) * (size.width - 2 * pad), y);
  double along(double x) => ((x - pad) / (size.width - 2 * pad)).clamp(0.0, 1.0);
  bool kept(double u) {
    final s = wrap(u - slide);
    return s >= begin && s <= end;
  }

  /// The bright part, as one or two stretches of the line (it goes round the end and comes back at the beginning).
  List<(double, double)> get stretches {
    if (end - begin >= 1) return [(0, 1)];
    final a = wrap(begin + slide), b = a + (end - begin);
    return b <= 1 ? [(a, b)] : [(a, 1), (0, b - 1)];
  }
}

class _TrimPaint extends CustomPainter {
  const _TrimPaint(this.g, this.held);
  final _Trim g;
  final _Knob? held;

  @override
  void paint(Canvas c, Size s) {
    c.drawRRect(RRect.fromRectAndRadius(Offset.zero & s, Shape.r(8)), Paint()..color = Grey.g07);
    Paint line(Color colour, double width) => Paint()
      ..color = colour
      ..strokeWidth = width
      ..strokeCap = StrokeCap.round;
    c.drawLine(Offset(_Trim.pad, g.y), Offset(s.width - _Trim.pad, g.y), line(Grey.g20, 3)); // (the whole path, faint)
    for (final (a, b) in g.stretches) {
      c.drawLine(Offset(_Trim.pad + a * (s.width - 2 * _Trim.pad), g.y), Offset(_Trim.pad + b * (s.width - 2 * _Trim.pad), g.y), line(held == _Knob.slide ? Pop.accent : Pop.tonePlace, 4));
    }
    for (final (k, u) in [(_Knob.begin, g.begin + g.slide), (_Knob.end, g.end + g.slide)]) {
      c.drawCircle(g.at(u), held == k ? 6.5 : 5, Paint()..color = held == k ? Pop.accent : TransformZone.lift.color);
    }
    // (which end is the start: a small bar across it)
    final b = g.at(g.begin + g.slide);
    c.drawLine(b + const Offset(0, -9), b + const Offset(0, -6), line(Grey.g56, 1.5));
  }

  @override
  bool shouldRepaint(_TrimPaint o) => true;
}
