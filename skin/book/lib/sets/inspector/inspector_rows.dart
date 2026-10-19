import 'package:flutter/foundation.dart' show mapEquals;
import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_rows_sect.dart';

import '../../tokens.dart';
import 'inspector_parts.dart';

// ---- 5. sections and rows -------------------------------------------------------------------------------------------
class Rule extends StatelessWidget {
  const Rule({super.key});
  @override
  Widget build(BuildContext context) => Container(height: 1, color: Ed(Ctx.of(context).cfg.ed).rule);
}


/// A card's fold mark: points right when folded, down when open, so a card that can fold says so before it is touched.
class FoldMark extends StatelessWidget {
  const FoldMark({super.key, required this.open, this.hot = false});
  final bool open, hot;
  @override
  Widget build(BuildContext context) => AnimatedRotation(
    turns: open ? .25 : 0,
    duration: const Duration(milliseconds: 120),
    child: CustomPaint(size: const Size(8, 8), painter: _FoldPaint(hot ? Grey.g95 : Grey.g56)),
  );
}

class _FoldPaint extends CustomPainter {
  const _FoldPaint(this.ink);
  final Color ink;
  @override
  void paint(Canvas c, Size s) => c.drawPath(
    Path()
      ..moveTo(s.width * .3, s.height * .12)
      ..lineTo(s.width * .7, s.height * .5)
      ..lineTo(s.width * .3, s.height * .88),
    Paint()
      ..color = ink
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.3
      ..strokeCap = StrokeCap.round
      ..strokeJoin = StrokeJoin.round,
  );
  @override
  bool shouldRepaint(_FoldPaint o) => o.ink != ink;
}

/// The Pop card behind [Sect]: every titled card folds (its own toggle, or a per-title flag), untitled ones never do.
/// Folded, a card is one line that still says what it holds: its colour, its name, a key dot if anything in it is animated, and its values in brief.
/// Folded summaries all start here, so a closed stack reads as a two-column list: what, then how much.
const briefX = 136.0;

class RowInfo extends InheritedWidget {
  const RowInfo({super.key, required this.unit, required this.keys, required super.child});
  final String? unit;
  final KeyS keys;
  static RowInfo? of(BuildContext c) => c.dependOnInheritedWidgetOfExactType<RowInfo>();
  @override
  bool updateShouldNotify(RowInfo old) => old.unit != unit || old.keys != keys;
}

/// Under it, coordinate fields drop their axis letters: one header names the columns.
class AxisColumns extends InheritedWidget {
  const AxisColumns({super.key, required super.child});
  static bool on(BuildContext c) => c.getInheritedWidgetOfExactType<AxisColumns>() != null;
  @override
  bool updateShouldNotify(AxisColumns old) => false;
}

/// The X / Y / Z column header, laid on the same grid as PropRow's cells.
class AxisHeader extends StatelessWidget {
  const AxisHeader({super.key});
  @override
  Widget build(BuildContext context) {
    final cfg = Ctx.of(context).cfg;
    return LayoutBuilder(
      builder: (context, box) => SizedBox(
        height: Shape.z(16),
        child: Row(
          children: [
            SizedBox(width: labelWPop),
            Expanded(
              child: Row(
                children: [
                  for (final (i, a) in const [(0, 'X'), (1, 'Y'), (2, 'Z')]) ...[
                    if (i > 0) SizedBox(width: Shape.s(4)),
                    Expanded(
                      child: Text(
                        a,
                        textAlign: TextAlign.center,
                        style: T.micro(Color.lerp(axesParts[i], Grey.g100, .3)!).copyWith(fontWeight: FontWeight.w700),
                      ),
                    ),
                  ],
                ],
              ),
            ),
            if (cfg.resetArrow) SizedBox(width: resetW),
          ],
        ),
      ),
    );
  }
}

/// A small picture of a row's value, drawn before its label, so the value reads as a shape before it reads as a number.
class RowMark extends InheritedWidget {
  const RowMark({super.key, required this.marks, required super.child});
  final Map<String, Widget> marks;
  static Widget? of(BuildContext context, String id) => context.dependOnInheritedWidgetOfExactType<RowMark>()?.marks[id];
  static bool has(BuildContext context, String id) => context.getInheritedWidgetOfExactType<RowMark>()?.marks.containsKey(id) ?? false;
  @override
  bool updateShouldNotify(RowMark old) => !mapEquals(old.marks, marks);
}
