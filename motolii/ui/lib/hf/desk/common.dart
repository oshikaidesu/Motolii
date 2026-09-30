// Desk panels: contextual instruments. A phenomenon first, precision second. Prototype only.
import 'dart:math' as math;
import 'package:flutter/widgets.dart';
import '../metrics.dart';
import '../bp/common.dart';
import '../neutral.dart';

// Housing is quiet and dark; the instruments inside are flat colour. Colour separates roles, it names nothing.
const kYellow = Color(0xFFF5C94A);
const kBlue = Color(0xFF4C7DF0);
const kPink = Color(0xFFF0699A);
const kMint = Color(0xFF4FD1A5);
const kViolet = Color(0xFF9A7BEA);
const kAccent = kYellow; // the current thing: selected, being touched
const kAccentDim = Color(0xFF4A3E1C);
const kPalette = [kYellow, kBlue, kPink, kMint, kViolet];

enum DeskKind { ease, depth, blend, history, notes }

class DeskIcon extends StatelessWidget {
  const DeskIcon(this.kind, {super.key, this.size = 26});
  final DeskKind kind;
  final double size;
  @override
  Widget build(BuildContext context) => SizedBox(width: size, height: size, child: CustomPaint(painter: _IconP(kind)));
}

class _IconP extends CustomPainter {
  _IconP(this.k);
  final DeskKind k;
  @override
  void paint(Canvas c, Size s) {
    c.scale(s.width / 26);
    final st = Paint()..color = Surface.ink..style = PaintingStyle.stroke..strokeWidth = 1.7..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round;
    final f = Paint()..color = Surface.ink;
    switch (k) {
      case DeskKind.ease:
        c.drawPath(Path()..moveTo(2, 18)..cubicTo(6, 18, 6, 6, 10, 6)..cubicTo(14, 6, 14, 18, 18, 18)..cubicTo(21, 18, 21, 11, 24, 11), st);
      case DeskKind.depth:
        c.drawCircle(const Offset(13, 13), 10.5, st);
        c.drawCircle(const Offset(13, 13), 6, st);
        c.drawCircle(const Offset(13, 13), 2, f);
      case DeskKind.blend:
        c.drawCircle(const Offset(9.5, 13), 7, st);
        c.drawCircle(const Offset(16.5, 13), 7, Paint()..color = Surface.ink.withValues(alpha: .35));
        c.drawCircle(const Offset(16.5, 13), 7, st);
      case DeskKind.history:
        c.drawCircle(const Offset(13, 13), 10.5, st);
        c.drawLine(const Offset(13, 7), const Offset(13, 13), st);
        c.drawLine(const Offset(13, 13), const Offset(17.5, 15.5), st);
      case DeskKind.notes:
        c.drawPath(Path()..moveTo(6, 3)..lineTo(16, 3)..lineTo(21, 8)..lineTo(21, 23)..lineTo(6, 23)..close(), st);
        c.drawPath(Path()..moveTo(16, 3)..lineTo(16, 8)..lineTo(21, 8), st);
        c.drawLine(const Offset(10, 14), const Offset(17, 14), st..strokeWidth = 1.4);
        c.drawLine(const Offset(10, 18), const Offset(17, 18), st);
    }
  }
  @override
  bool shouldRepaint(_IconP o) => o.k != k;
}

class Dots3 extends StatelessWidget {
  const Dots3({super.key});
  @override
  Widget build(BuildContext context) => SizedBox(width: 22.5, height: 22.5, child: CustomPaint(painter: _D3()));
}

class _D3 extends CustomPainter {
  @override
  void paint(Canvas c, Size s) {
    final p = Paint()..color = N.g69;
    for (final x in [-6.0, 0.0, 6.0]) { c.drawCircle(Offset(s.width / 2 + x, s.height / 2), 1.6, p); }
  }
  @override
  bool shouldRepaint(_D3 o) => false;
}

/// Header and body placement for a Desk. Each Desk supplies its own body for each morphology.
class DeskShell extends StatelessWidget {
  const DeskShell({super.key, required this.kind, required this.title, required this.subtitle, required this.full, required this.strip, required this.tall, this.trailing});
  final Widget? trailing;
  final DeskKind kind;
  final String title, subtitle;
  final Widget Function(BuildContext, Size) full, strip, tall;
  @override
  Widget build(BuildContext context) => LayoutBuilder(builder: (context, box) {
        final w = box.maxWidth, h = box.maxHeight;
        final isStrip = h < 200;
        final isTall = !isStrip && w < 230;
        final showSub = !isStrip && !isTall;
        final docked = DockedPanel.of(context);
        final hh = docked ? Surface.chromeRow : (showSub ? Surface.namedHeader + 8 : Surface.namedHeader);
        final body = Size(w, h - hh);
        return Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          Container(
            height: hh,
            padding: EdgeInsets.only(left: showSub ? 16 : 12, right: 4.5),
            decoration: const BoxDecoration(border: Border(bottom: BorderSide(color: Surface.dividerFine))),
            child: Row(children: [
              if (!docked) ...[DeskIcon(kind, size: showSub ? 20 : 18), SizedBox(width: showSub ? 10 : 8)],
              // In a dock tab the tab names the desk; what stays is the line that says what it edits.
              if (docked) Expanded(child: Text(subtitle, softWrap: false, overflow: TextOverflow.clip, style: sans(Dn.microSize, c: Surface.muted, w: FontWeight.w500, ls: 1.1))) else Expanded(
                child: Column(mainAxisAlignment: MainAxisAlignment.center, crossAxisAlignment: CrossAxisAlignment.start, children: [
                  Text(title, softWrap: false, overflow: TextOverflow.clip, style: sans(showSub ? 14 : 13, c: Surface.ink, w: FontWeight.w600, ls: -0.2)),
                  if (showSub) ...[const SizedBox(height: 1.5), Text(subtitle, softWrap: false, overflow: TextOverflow.clip, style: sans(Dn.microSize, c: Surface.muted, w: FontWeight.w500, ls: 1.1))],
                ]),
              ),
              if (trailing != null && showSub) trailing!,
            ]),
          ),
          Expanded(child: ClipRect(child: isStrip ? strip(context, body) : (isTall ? tall(context, body) : full(context, body)))),
        ]);
      });
}

/// A precision box: small label, large value.
class NumBox extends StatelessWidget {
  const NumBox(this.label, this.value, {super.key, this.width, this.compact = false});
  final String label, value;
  final double? width;
  final bool compact;
  @override
  Widget build(BuildContext context) => Container(
        width: width,
        height: compact ? 38 : 52,
        padding: EdgeInsets.fromLTRB(7, compact ? 5 : 7, 6, 4.5),
        decoration: BoxDecoration(color: Surface.well, border: Border.all(color: Surface.dividerFine), borderRadius: BorderRadius.circular(2)),
        child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
          Text(label, style: sans(Dn.microSize, c: Surface.muted)),
          const Spacer(),
          Text(value, softWrap: false, style: sans(compact ? 13 : 17, c: Surface.ink, w: FontWeight.w400)),
        ]),
      );
}

/// A labelled slider: label left, track, value right. The track is a thin line with a round thumb.
class SliderRow extends StatelessWidget {
  const SliderRow(this.label, this.t, this.value, {super.key, this.labelWidth = 104, this.onChanged});
  final String label, value;
  final double t, labelWidth;
  final ValueChanged<double>? onChanged;
  @override
  Widget build(BuildContext context) => SizedBox(
        height: 21,
        child: Row(children: [
          SizedBox(width: labelWidth, child: Text(label, softWrap: false, overflow: TextOverflow.clip, style: sans(Dn.nameSize, c: N.g82))),
          Expanded(
            child: LayoutBuilder(builder: (context, b) {
              void at(double x) => onChanged?.call(clampD((x - 6) / (b.maxWidth - 12), 0, 1));
              return GestureDetector(
                behavior: HitTestBehavior.opaque,
                onTapDown: (d) => at(d.localPosition.dx),
                onPanUpdate: (d) => at(d.localPosition.dx),
                child: SizedBox(height: 21, child: CustomPaint(painter: _Track(t))),
              );
            }),
          ),
          const SizedBox(width: 7.5),
          SizedBox(width: 28.5, child: Text(value, textAlign: TextAlign.right, softWrap: false, style: sans(Dn.nameSize, c: N.g82))),
        ]),
      );
}

class _Track extends CustomPainter {
  _Track(this.t);
  final double t;
  @override
  void paint(Canvas c, Size s) {
    final y = s.height / 2;
    c.drawLine(Offset(0, y), Offset(s.width, y), Paint()..color = N.g26..strokeWidth = 2..strokeCap = StrokeCap.round);
    final x = 6 + (s.width - 12) * t;
    c.drawLine(Offset(0, y), Offset(x, y), Paint()..color = kAccent..strokeWidth = 2..strokeCap = StrokeCap.round);
    c.drawCircle(Offset(x, y), 5.5, Paint()..color = Surface.ink);
    c.drawCircle(Offset(x, y), 5.5, Paint()..color = N.g07..style = PaintingStyle.stroke..strokeWidth = 1.2);
  }
  @override
  bool shouldRepaint(_Track o) => o.t != t;
}

class Segmented extends StatelessWidget {
  const Segmented(this.items, this.active, {super.key, this.height = 34, this.onChanged});
  final List<String> items;
  final int active;
  final double height;
  final ValueChanged<int>? onChanged;
  @override
  Widget build(BuildContext context) => Row(children: [
        for (var i = 0; i < items.length; i++)
          Expanded(
            child: GestureDetector(
              behavior: HitTestBehavior.opaque,
              onTap: () => onChanged?.call(i),
              child: Container(
              height: height,
              margin: EdgeInsets.only(right: i == items.length - 1 ? 0 : 6),
              decoration: BoxDecoration(color: i == active ? kAccentDim.withValues(alpha: .45) : Surface.well, border: Border.all(color: i == active ? kAccent : Surface.dividerFine), borderRadius: BorderRadius.circular(2)),
              child: Center(child: Text(items[i], softWrap: false, style: sans(Dn.nameSize, c: i == active ? Surface.ink : N.g69))),
            ),
            ),
          ),
      ]);
}

class DropRow extends StatelessWidget {
  const DropRow(this.label, this.value, {super.key, this.labelWidth = 104, this.onTap});
  final String label, value;
  final double labelWidth;
  final VoidCallback? onTap;
  @override
  Widget build(BuildContext context) => SizedBox(
        height: 25.5,
        child: Row(children: [
          SizedBox(width: labelWidth, child: Text(label, softWrap: false, style: sans(Dn.nameSize, c: N.g82))),
          Expanded(
            child: GestureDetector(
              behavior: HitTestBehavior.opaque,
              onTap: onTap,
              child: Container(
                height: 22.5,
                padding: const EdgeInsets.symmetric(horizontal: 7.5),
                decoration: BoxDecoration(color: Surface.well, border: Border.all(color: Surface.dividerFine), borderRadius: BorderRadius.circular(2)),
                child: Row(children: [Text(value, style: sans(Dn.nameSize, c: Surface.ink)), const Spacer(), SizedBox(width: 9, height: 6, child: CustomPaint(painter: _Chev()))]),
              ),
            ),
          ),
        ]),
      );
}

class _Chev extends CustomPainter {
  @override
  void paint(Canvas c, Size s) => c.drawPath(Path()..moveTo(1, 1.5)..lineTo(s.width / 2, s.height - 1.5)..lineTo(s.width - 1, 1.5), Paint()..color = N.g69..style = PaintingStyle.stroke..strokeWidth = 1.5..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round);
  @override
  bool shouldRepaint(_Chev o) => false;
}

double clampD(double v, double a, double b) => math.max(a, math.min(b, v));
