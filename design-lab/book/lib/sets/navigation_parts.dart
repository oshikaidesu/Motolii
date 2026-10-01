// Private helpers for the navigation set: hover surface, line glyphs, row chrome, buttons, a floating shadow.
import 'package:flutter/widgets.dart';

import '../parts/controls.dart';
import '../tokens.dart';

const kFast = Duration(milliseconds: 120);
const kEase = Curves.easeOut;

/// The one shadow, for floating things only (menu, popover, palette, dialog).
List<BoxShadow> floatShadow() => [BoxShadow(color: N.g00.withValues(alpha: .5), blurRadius: 24, offset: const Offset(0, 8))];

/// The surface's single accent for a finish: grey-white when quiet, the mode colour otherwise.
Color accOf(Look l) => l == Look.quiet ? N.g91 : C.mode;

Color clear() => N.g00.withValues(alpha: 0);

/// Hover and pressed state around anything.
class Hot extends StatefulWidget {
  const Hot({super.key, required this.builder, this.onTap, this.enabled = true, this.onHover});
  final Widget Function(BuildContext context, bool hover, bool down) builder;
  final VoidCallback? onTap;
  final bool enabled;
  final ValueChanged<bool>? onHover;
  @override
  State<Hot> createState() => _HotState();
}

class _HotState extends State<Hot> {
  bool _h = false, _d = false;
  @override
  Widget build(BuildContext context) => MouseRegion(
        cursor: widget.enabled && widget.onTap != null ? SystemMouseCursors.click : MouseCursor.defer,
        onEnter: (_) {
          setState(() => _h = true);
          widget.onHover?.call(true);
        },
        onExit: (_) {
          setState(() {
            _h = false;
            _d = false;
          });
          widget.onHover?.call(false);
        },
        child: GestureDetector(
          behavior: HitTestBehavior.opaque,
          onTapDown: widget.enabled ? (_) => setState(() => _d = true) : null,
          onTapUp: widget.enabled ? (_) => setState(() => _d = false) : null,
          onTapCancel: widget.enabled ? () => setState(() => _d = false) : null,
          onTap: widget.enabled ? widget.onTap : null,
          child: widget.builder(context, widget.enabled && _h, widget.enabled && _d),
        ),
      );
}

enum G { caretR, caretD, close, grip, search, plus, dots, check, chevR, folder, layer, text, eye, sub }

/// A 12 px line glyph, drawn here so no icon font is needed.
class Glyph extends StatelessWidget {
  const Glyph(this.g, {super.key, this.color = N.g63, this.size = 12});
  final G g;
  final Color color;
  final double size;
  @override
  Widget build(BuildContext context) => SizedBox(width: size, height: size, child: CustomPaint(painter: _GlyphPainter(g, color)));
}

class _GlyphPainter extends CustomPainter {
  _GlyphPainter(this.g, this.color);
  final G g;
  final Color color;
  @override
  void paint(Canvas canvas, Size size) {
    final s = size.width / 12;
    canvas.scale(s);
    final line = Paint()
      ..color = color
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.3
      ..strokeCap = StrokeCap.round
      ..strokeJoin = StrokeJoin.round;
    final fill = Paint()..color = color;
    Path poly(List<Offset> p) => Path()..addPolygon(p, false);
    switch (g) {
      case G.caretR:
        canvas.drawPath(poly(const [Offset(4, 2.5), Offset(8.5, 6), Offset(4, 9.5)])..close(), fill);
      case G.caretD:
      case G.sub:
        canvas.drawPath(poly(const [Offset(2.5, 4), Offset(9.5, 4), Offset(6, 8.5)])..close(), fill);
      case G.close:
        canvas.drawLine(const Offset(3, 3), const Offset(9, 9), line);
        canvas.drawLine(const Offset(9, 3), const Offset(3, 9), line);
      case G.grip:
        for (final x in [4.5, 7.5]) {
          for (final y in [3.0, 6.0, 9.0]) {
            canvas.drawCircle(Offset(x, y), .85, fill);
          }
        }
      case G.search:
        canvas.drawCircle(const Offset(5.2, 5.2), 3.2, line);
        canvas.drawLine(const Offset(7.6, 7.6), const Offset(10, 10), line);
      case G.plus:
        canvas.drawLine(const Offset(6, 2.5), const Offset(6, 9.5), line);
        canvas.drawLine(const Offset(2.5, 6), const Offset(9.5, 6), line);
      case G.dots:
        for (final x in [2.5, 6.0, 9.5]) {
          canvas.drawCircle(Offset(x, 6), .95, fill);
        }
      case G.check:
        canvas.drawPath(poly(const [Offset(2.5, 6.4), Offset(5, 8.8), Offset(9.5, 3.4)]), line);
      case G.chevR:
        canvas.drawPath(poly(const [Offset(4.5, 2.5), Offset(8, 6), Offset(4.5, 9.5)]), line);
      case G.folder:
        canvas.drawRRect(RRect.fromRectAndRadius(const Rect.fromLTWH(1.5, 3, 9, 7), const Radius.circular(1.5)), line);
        canvas.drawLine(const Offset(1.5, 5.2), const Offset(10.5, 5.2), line);
      case G.layer:
        canvas.drawRRect(RRect.fromRectAndRadius(const Rect.fromLTWH(2, 2.5, 8, 7), const Radius.circular(1.5)), line);
      case G.text:
        canvas.drawLine(const Offset(3, 3), const Offset(9, 3), line);
        canvas.drawLine(const Offset(6, 3), const Offset(6, 9.5), line);
      case G.eye:
        canvas.drawOval(const Rect.fromLTWH(1.2, 3.4, 9.6, 5.2), line);
        canvas.drawCircle(const Offset(6, 6), 1.1, fill);
    }
  }

  @override
  bool shouldRepaint(_GlyphPainter o) => o.g != g || o.color != color;
}

/// A 20 px square icon button, hover and pressed.
class IconBtn extends StatelessWidget {
  const IconBtn(this.g, {super.key, this.onTap, this.on = false, this.enabled = true});
  final G g;
  final VoidCallback? onTap;
  final bool on, enabled;
  @override
  Widget build(BuildContext context) => Hot(
        onTap: onTap,
        enabled: enabled,
        builder: (c, h, d) => AnimatedContainer(
          duration: kFast,
          curve: kEase,
          width: 20,
          height: 20,
          alignment: Alignment.center,
          decoration: BoxDecoration(color: d ? N.g26 : (h || on) ? N.g20 : clear(), borderRadius: BorderRadius.circular(4)),
          child: Glyph(g, color: !enabled ? N.g38 : (h || on) ? N.g95 : N.g56),
        ),
      );
}

/// The background of a row in every list-like thing: rest, hover, selected, by finish.
class RowSurface extends StatelessWidget {
  const RowSurface({super.key, required this.sel, required this.hover, required this.look, required this.child, this.height, this.radius = 4, this.padding = EdgeInsets.zero});
  final bool sel, hover;
  final Look look;
  final Widget child;
  final double? height, radius;
  final EdgeInsets padding;
  @override
  Widget build(BuildContext context) {
    final bg = sel
        ? (look == Look.glow ? C.mode.withValues(alpha: .16) : N.g20)
        : hover
            ? (look == Look.quiet ? N.g13 : N.g15)
            : clear();
    return Stack(children: [
      AnimatedContainer(duration: kFast, curve: kEase, height: height, padding: padding, decoration: BoxDecoration(color: bg, borderRadius: BorderRadius.circular(radius ?? 4)), child: child),
      Positioned(left: 0, top: 5, bottom: 5, width: 2, child: AnimatedOpacity(duration: kFast, curve: kEase, opacity: sel ? 1 : 0, child: DecoratedBox(decoration: BoxDecoration(color: look == Look.glow ? C.mode : N.g95, borderRadius: BorderRadius.circular(1))))),
    ]);
  }
}

/// A button: primary is one accent fill, secondary is a grey well.
class PBtn extends StatelessWidget {
  const PBtn(this.label, {super.key, this.primary = false, this.color, this.enabled = true, this.onTap});
  final String label;
  final bool primary, enabled;
  final Color? color;
  final VoidCallback? onTap;
  @override
  Widget build(BuildContext context) => Hot(
        onTap: onTap,
        enabled: enabled,
        builder: (c, h, d) {
          final base = color ?? C.mode;
          final bg = primary ? (d ? Color.lerp(base, N.g00, .2)! : h ? Color.lerp(base, N.g100, .12)! : base) : (d ? N.g26 : h ? N.g20 : N.g15);
          return AnimatedContainer(
            duration: kFast,
            curve: kEase,
            height: 28,
            constraints: const BoxConstraints(minWidth: 72),
            padding: const EdgeInsets.symmetric(horizontal: 14),
            alignment: Alignment.center,
            decoration: BoxDecoration(
              color: enabled ? bg : N.g13,
              borderRadius: BorderRadius.circular(6),
              border: Border.all(color: primary && enabled ? clear() : N.g20),
            ),
            child: Text(label, style: T.name(!enabled ? N.g38 : primary ? N.g100 : N.g91)),
          );
        },
      );
}

/// A keyboard hint chip.
class Kbd extends StatelessWidget {
  const Kbd(this.text, {super.key});
  final String text;
  @override
  Widget build(BuildContext context) => Container(
        height: 16,
        constraints: const BoxConstraints(minWidth: 16),
        padding: const EdgeInsets.symmetric(horizontal: 4),
        alignment: Alignment.center,
        decoration: BoxDecoration(color: N.g10, borderRadius: BorderRadius.circular(4), border: Border.all(color: N.g20)),
        child: Text(text, style: T.label(N.g63).copyWith(fontFamily: T.mono, fontSize: 10)),
      );
}

/// A count chip, grey.
class CountChip extends StatelessWidget {
  const CountChip(this.n, {super.key, this.color = N.g56});
  final int n;
  final Color color;
  @override
  Widget build(BuildContext context) => Container(
        height: 16,
        constraints: const BoxConstraints(minWidth: 16),
        padding: const EdgeInsets.symmetric(horizontal: 5),
        alignment: Alignment.center,
        decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(8)),
        child: Text('$n', style: T.value(color).copyWith(fontSize: 10)),
      );
}

/// A line of placeholder content for panel bodies.
class FauxLine extends StatelessWidget {
  const FauxLine(this.w, {super.key});
  final double w;
  @override
  Widget build(BuildContext context) => Container(width: w, height: 6, decoration: BoxDecoration(color: N.g20, borderRadius: BorderRadius.circular(3)));
}

/// A line of text that never wraps.
class One extends StatelessWidget {
  const One(this.text, this.style, {super.key});
  final String text;
  final TextStyle style;
  @override
  Widget build(BuildContext context) => Text(text, style: style, maxLines: 1, overflow: TextOverflow.ellipsis, softWrap: false);
}
