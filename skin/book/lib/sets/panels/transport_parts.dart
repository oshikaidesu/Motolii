// Private helpers of the transport set: one pressable primitive, one glyph painter, and the controls built on them.
import 'package:flutter/widgets.dart';

import '../../parts/controls.dart';
import '../../tokens.dart';

const _ms = Duration(milliseconds: 120);
// [open] control height of the top bar's buttons and chips (4 px grid)
const double kCtl = 28;
// [open] corner radius of every control in this set (DESIGN: 6-8)
const double kRad = 6;

/// A forced visual state, so a matrix can show hover / pressed without a pointer.
enum BtnState { rest, hover, pressed }

/// Holds a value locally, resets when the knob that feeds it changes.
class Live<V> extends StatefulWidget {
  const Live({super.key, required this.value, required this.builder});
  final V value;
  final Widget Function(V v, void Function(V) set) builder;
  @override
  State<Live<V>> createState() => _LiveState<V>();
}

class _LiveState<V> extends State<Live<V>> {
  late V v = widget.value;
  @override
  void didUpdateWidget(Live<V> old) {
    super.didUpdateWidget(old);
    if (old.value != widget.value) v = widget.value;
  }

  @override
  Widget build(BuildContext context) => widget.builder(v, (n) => setState(() => v = n));
}

/// Hover / pressed tracking with the right cursor. Everything clickable in this set sits on it.
class Pressable extends StatefulWidget {
  const Pressable({super.key, required this.builder, this.onTap, this.enabled = true, this.force = BtnState.rest});
  final Widget Function(bool hover, bool down) builder;
  final VoidCallback? onTap;
  final bool enabled;
  final BtnState force;
  @override
  State<Pressable> createState() => _PressableState();
}

class _PressableState extends State<Pressable> {
  bool _h = false, _d = false;
  @override
  Widget build(BuildContext context) {
    final on = widget.enabled;
    final hover = on && (_h || widget.force != BtnState.rest);
    final down = on && (_d || widget.force == BtnState.pressed);
    return MouseRegion(
      cursor: on ? SystemMouseCursors.click : SystemMouseCursors.basic,
      onEnter: (_) => setState(() => _h = true),
      onExit: (_) => setState(() => _h = _d = false),
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTapDown: on ? (_) => setState(() => _d = true) : null,
        onTapUp: on ? (_) => setState(() => _d = false) : null,
        onTapCancel: () => setState(() => _d = false),
        onTap: on ? widget.onTap : null,
        child: widget.builder(hover, down),
      ),
    );
  }
}

enum G { play, pause, stop, record, undo, redo, loop, chevron, minus, plus, fit, speaker, mute, camera }

class Glyph extends StatelessWidget {
  const Glyph(this.g, {super.key, this.color = N.g76, this.size = 16});
  final G g;
  final Color color;
  final double size;
  @override
  Widget build(BuildContext context) => TweenAnimationBuilder<Color?>(
        tween: ColorTween(end: color),
        duration: _ms,
        builder: (_, c, _) => CustomPaint(size: Size.square(size), painter: _GlyphPainter(g, c ?? color)),
      );
}

class _GlyphPainter extends CustomPainter {
  _GlyphPainter(this.g, this.c);
  final G g;
  final Color c;
  @override
  void paint(Canvas canvas, Size s) {
    canvas.scale(s.width / 16);
    final line = Paint()..color = c..style = PaintingStyle.stroke..strokeWidth = 1.5..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round;
    final fill = Paint()..color = c;
    switch (g) {
      case G.play:
        canvas.drawPath(Path()..moveTo(5, 3)..lineTo(13, 8)..lineTo(5, 13)..close(), fill..strokeJoin = StrokeJoin.round);
      case G.pause:
        canvas.drawRRect(RRect.fromLTRBR(4, 3, 7, 13, const Radius.circular(1)), fill);
        canvas.drawRRect(RRect.fromLTRBR(9, 3, 12, 13, const Radius.circular(1)), fill);
      case G.stop:
        canvas.drawRRect(RRect.fromLTRBR(4, 4, 12, 12, const Radius.circular(1.5)), fill);
      case G.record:
        canvas.drawCircle(const Offset(8, 8), 4.5, fill);
      case G.undo || G.redo:
        if (g == G.redo) { canvas.translate(16, 0); canvas.scale(-1, 1); }
        canvas.drawPath(Path()..moveTo(6, 3.5)..lineTo(3, 6.5)..lineTo(6, 9.5), line);
        canvas.drawPath(Path()..moveTo(3.5, 6.5)..lineTo(9, 6.5)..quadraticBezierTo(13, 6.5, 13, 10.5)..quadraticBezierTo(13, 13, 10, 13), line);
      case G.loop:
        canvas.drawPath(Path()..moveTo(5, 4.5)..lineTo(11, 4.5)..quadraticBezierTo(13.5, 4.5, 13.5, 7)..moveTo(11, 11.5)..lineTo(5, 11.5)..quadraticBezierTo(2.5, 11.5, 2.5, 9), line);
        canvas.drawPath(Path()..moveTo(9.5, 2.5)..lineTo(11.5, 4.5)..lineTo(9.5, 6.5)..moveTo(6.5, 9.5)..lineTo(4.5, 11.5)..lineTo(6.5, 13.5), line);
      case G.chevron:
        canvas.drawPath(Path()..moveTo(4.5, 6.5)..lineTo(8, 10)..lineTo(11.5, 6.5), line);
      case G.minus:
        canvas.drawLine(const Offset(3.5, 8), const Offset(12.5, 8), line);
      case G.plus:
        canvas.drawLine(const Offset(3.5, 8), const Offset(12.5, 8), line);
        canvas.drawLine(const Offset(8, 3.5), const Offset(8, 12.5), line);
      case G.fit:
        for (final sx in [1.0, -1.0]) {
          for (final sy in [1.0, -1.0]) {
            final o = Offset(sx > 0 ? 3 : 13, sy > 0 ? 3 : 13);
            canvas.drawPath(Path()..moveTo(o.dx, o.dy + sy * 3.5)..lineTo(o.dx, o.dy)..lineTo(o.dx + sx * 3.5, o.dy), line);
          }
        }
      case G.speaker || G.mute:
        canvas.drawPath(Path()..moveTo(2.5, 6)..lineTo(5.5, 6)..lineTo(9, 3)..lineTo(9, 13)..lineTo(5.5, 10)..lineTo(2.5, 10)..close(), fill);
        if (g == G.speaker) {
          canvas.drawArc(const Rect.fromLTRB(7, 5, 13, 11), -.9, 1.8, false, line);
        } else {
          canvas.drawLine(const Offset(11, 6), const Offset(14, 10), line);
          canvas.drawLine(const Offset(14, 6), const Offset(11, 10), line);
        }
      case G.camera:
        canvas.drawRRect(RRect.fromLTRBR(2, 4.5, 10.5, 11.5, const Radius.circular(1.5)), line);
        canvas.drawPath(Path()..moveTo(10.5, 7)..lineTo(14, 5)..lineTo(14, 11)..lineTo(10.5, 9), line);
    }
  }

  @override
  bool shouldRepaint(_GlyphPainter o) => o.g != g || o.c != c;
}

Color _tint(Color a, double alpha) => a.withValues(alpha: alpha);

/// A square-or-round icon button of the set. `on` is the toggled state; `accent` is the one colour it may carry.
class TpButton extends StatelessWidget {
  const TpButton({
    super.key,
    required this.g,
    this.look = Look.concept,
    this.accent = N.g95,
    this.on = false,
    this.enabled = true,
    this.round = false,
    this.size = kCtl,
    this.force = BtnState.rest,
    this.onTap,
    this.tip,
    this.idle = N.g76,
    this.glyph,
  });
  final G g;
  final Look look;
  final Color accent;
  final bool on, enabled, round;
  final double size;
  final BtnState force;
  final VoidCallback? onTap;
  final String? tip;
  final Color idle;
  final double? glyph;

  @override
  Widget build(BuildContext context) => Pressable(
        enabled: enabled,
        force: force,
        onTap: onTap,
        builder: (hover, down) {
          final quiet = look == Look.quiet;
          Color fill = quiet ? N.g13.withValues(alpha: 0) : N.g13;
          Color edge = quiet ? N.g20.withValues(alpha: 0) : N.g20;
          Color ink = enabled ? (hover ? N.g100 : idle) : N.g38;
          if (hover) { fill = N.g15; edge = quiet ? edge : N.g26; }
          if (down) fill = quiet ? N.g20 : N.g10;
          List<BoxShadow>? glow;
          if (on && enabled) {
            fill = _tint(accent, down ? .26 : (hover ? .22 : .16));
            edge = _tint(accent, .6);
            ink = accent;
            if (look == Look.glow) glow = [BoxShadow(color: _tint(accent, .38), blurRadius: 12)];
          }
          if (!enabled) edge = quiet ? edge : N.g15;
          return AnimatedContainer(
            duration: _ms,
            curve: Curves.easeOut,
            width: size,
            height: size,
            alignment: Alignment.center,
            decoration: BoxDecoration(
              color: fill,
              shape: round ? BoxShape.circle : BoxShape.rectangle,
              borderRadius: round ? null : BorderRadius.circular(kRad),
              border: Border.all(color: edge),
              boxShadow: glow,
            ),
            child: Glyph(g, color: ink, size: glyph ?? (size >= 32 ? 18 : 16)),
          );
        },
      );
}

/// A chip: a label (and optionally a chevron) that is a button.
class TpChip extends StatelessWidget {
  const TpChip({super.key, required this.label, this.mono = false, this.chevron = false, this.open = false, this.accent, this.icon, this.force = BtnState.rest, this.enabled = true, this.onTap, this.minWidth = 0});
  final String label;
  final bool mono, chevron, open, enabled;
  final Color? accent;
  final G? icon;
  final BtnState force;
  final VoidCallback? onTap;
  final double minWidth;
  @override
  Widget build(BuildContext context) => Pressable(
        force: force,
        enabled: enabled,
        onTap: onTap,
        builder: (hover, down) {
          final a = accent;
          final lit = open || (a != null && hover);
          final fill = down ? N.g10 : (open ? N.g20 : (hover ? N.g15 : N.g13));
          final ink = !enabled ? N.g38 : (a != null && (open || hover) ? a : (hover || open ? N.g95 : N.g91));
          return AnimatedContainer(
            duration: _ms,
            curve: Curves.easeOut,
            height: kCtl,
            constraints: BoxConstraints(minWidth: minWidth),
            padding: EdgeInsets.only(left: icon == null ? 10 : 8, right: chevron ? 6 : 10),
            decoration: BoxDecoration(color: fill, borderRadius: BorderRadius.circular(kRad), border: Border.all(color: lit && a != null ? _tint(a, .55) : (hover ? N.g26 : N.g20))),
            child: Row(mainAxisSize: MainAxisSize.min, mainAxisAlignment: MainAxisAlignment.center, children: [
              if (icon != null) ...[Glyph(icon!, color: ink, size: 14), const SizedBox(width: 6)],
              Text(label, style: mono ? T.value(ink) : T.name(ink)),
              if (chevron) ...[const SizedBox(width: 4), Glyph(G.chevron, color: enabled ? N.g63 : N.g38, size: 14)],
            ]),
          );
        },
      );
}

/// A floating list (menu / dropdown): the only thing in the set with a shadow.
class TpMenu extends StatelessWidget {
  const TpMenu({super.key, required this.items, this.selected, this.width = 168, this.keys = const [], this.onPick});
  final List<String> items;
  final int? selected;
  final double width;
  final List<String> keys;
  final ValueChanged<int>? onPick;
  @override
  Widget build(BuildContext context) => Container(
        width: width,
        padding: const EdgeInsets.all(4),
        decoration: BoxDecoration(
          color: N.g13,
          borderRadius: BorderRadius.circular(8),
          border: Border.all(color: N.g26),
          boxShadow: const [BoxShadow(color: Color(0x66000000), blurRadius: 16, offset: Offset(0, 6))],
        ),
        child: Column(mainAxisSize: MainAxisSize.min, children: [
          for (var i = 0; i < items.length; i++)
            Pressable(
              onTap: () => onPick?.call(i),
              builder: (hover, down) => AnimatedContainer(
                duration: _ms,
                curve: Curves.easeOut,
                height: 24,
                padding: const EdgeInsets.symmetric(horizontal: 8),
                decoration: BoxDecoration(color: down ? N.g26 : (hover ? N.g20 : const Color(0x00000000)), borderRadius: BorderRadius.circular(4)),
                child: Row(children: [
                  SizedBox(width: 14, child: selected == i ? Container(width: 4, height: 4, decoration: const BoxDecoration(color: N.g95, shape: BoxShape.circle)) : null),
                  Expanded(child: Text(items[i], style: T.name(hover || selected == i ? N.g95 : N.g76))),
                  if (i < keys.length) Text(keys[i], style: T.label(N.g56)),
                ]),
              ),
            ),
        ]),
      );
}

/// Mode switch: the track is exactly as wide as its labels; the selected segment is a g20 pill with a 1 px accent underline.
class TpMode extends StatelessWidget {
  const TpMode({super.key, required this.items, required this.index, this.look = Look.concept, this.accent, this.onChanged});
  final List<String> items;
  final int index;
  final Look look;
  final Color? accent;
  final ValueChanged<int>? onChanged;
  @override
  Widget build(BuildContext context) {
    final accent = this.accent ?? Role.selected;
    final quiet = look == Look.quiet;
    final style = T.micro(N.g95).copyWith(letterSpacing: .6);
    final ws = [
      for (final t in items) (TextPainter(text: TextSpan(text: t.toUpperCase(), style: style), textDirection: TextDirection.ltr)..layout()).width + 28,
    ];
    final left = ws.take(index).fold(0.0, (a, b) => a + b);
    // Align loosens a tight parent so the track keeps its own width.
    return Align(
      widthFactor: 1,
      heightFactor: 1,
      child: Container(
        height: kCtl,
        width: ws.fold(0.0, (a, b) => a + b) + 6, // 2 x (1 border + 2 padding)
        padding: const EdgeInsets.all(2),
        decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(kRad + 1), border: Border.all(color: N.g15)),
        child: Stack(children: [
          AnimatedPositioned(
            duration: _ms,
            curve: Curves.easeOut,
            left: left,
            top: 0,
            bottom: 0,
            width: ws[index],
            child: DecoratedBox(
              decoration: BoxDecoration(
                color: N.g20,
                borderRadius: BorderRadius.circular(kRad - 1),
                boxShadow: look == Look.glow ? [BoxShadow(color: _tint(accent, .3), blurRadius: 10)] : null,
              ),
              child: quiet && Role.grey ? null : Align(alignment: Alignment.bottomCenter, child: Container(height: 1, margin: const EdgeInsets.symmetric(horizontal: 8), color: accent)),
            ),
          ),
          Row(children: [
            for (var i = 0; i < items.length; i++)
              Pressable(
                onTap: () => onChanged?.call(i),
                builder: (hover, down) => SizedBox(
                  width: ws[i],
                  child: Center(child: AnimatedDefaultTextStyle(
                    duration: _ms,
                    style: T.micro(i == index ? N.g95 : (hover ? N.g95 : N.g63)).copyWith(letterSpacing: .6),
                    child: Text(items[i].toUpperCase()),
                  )),
                ),
              ),
          ]),
        ]),
      ),
    );
  }
}

/// A horizontal slider with a thumb; the value is the loudest thing.
class TpSlider extends StatelessWidget {
  const TpSlider({super.key, required this.value, this.onChanged, this.width = 96, this.enabled = true, this.look = Look.concept});
  final double value, width;
  final bool enabled;
  final Look look;
  final ValueChanged<double>? onChanged;
  @override
  Widget build(BuildContext context) {
    final t = value.clamp(0.0, 1.0);
    return Pressable(
      enabled: enabled,
      builder: (hover, down) => GestureDetector(
        behavior: HitTestBehavior.opaque,
        onHorizontalDragDown: enabled ? (d) => onChanged?.call((d.localPosition.dx / width).clamp(0.0, 1.0)) : null,
        onHorizontalDragUpdate: enabled ? (d) => onChanged?.call((d.localPosition.dx / width).clamp(0.0, 1.0)) : null,
        child: SizedBox(
          width: width,
          height: kCtl,
          child: Stack(alignment: Alignment.centerLeft, clipBehavior: Clip.none, children: [
            Container(height: 4, decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(2), border: Border.all(color: N.g15))),
            AnimatedContainer(
              duration: const Duration(milliseconds: 40),
              width: width * t,
              height: 4,
              decoration: BoxDecoration(color: !enabled ? N.g38 : (hover || down ? N.g91 : N.g76), borderRadius: BorderRadius.circular(2)),
            ),
            Positioned(
              left: (width - 10) * t,
              child: AnimatedContainer(
                duration: _ms,
                curve: Curves.easeOut,
                width: 10,
                height: 10,
                decoration: BoxDecoration(
                  color: !enabled ? Role.disabled : (down ? N.g100 : N.g95),
                  shape: BoxShape.circle,
                  boxShadow: look == Look.glow && enabled ? const [BoxShadow(color: Color(0x55FFFFFF), blurRadius: 8)] : null,
                  border: Border.all(color: N.g07, width: hover ? 2 : 1),
                ),
              ),
            ),
          ]),
        ),
      ),
    );
  }
}

/// Two-row segmented level meter with a peak-hold tick. Colour is level, not decoration: ok / hot / clip.
class TpMeter extends StatelessWidget {
  const TpMeter({super.key, required this.l, required this.r, this.peakL, this.peakR, this.width = 96});
  final double l, r, width;
  final double? peakL, peakR;
  @override
  Widget build(BuildContext context) => SizedBox(
        width: width,
        height: 22,
        child: Row(children: [
          Column(children: [
            for (final t in ['L', 'R']) SizedBox(width: 10, height: 11, child: Text(t, style: T.label(N.g56))),
          ]),
          const SizedBox(width: 4),
          Expanded(child: CustomPaint(size: const Size.fromHeight(22), painter: _MeterPainter(l, r, peakL, peakR))),
        ]),
      );
}

class _MeterPainter extends CustomPainter {
  _MeterPainter(this.l, this.r, this.pl, this.pr);
  final double l, r;
  final double? pl, pr;
  @override
  void paint(Canvas canvas, Size s) {
    const n = 24;
    final w = (s.width - (n - 1)) / n;
    void row(double y, double v, double? peak) {
      final lit = (v.clamp(0.0, 1.0) * n).round();
      final pk = peak == null ? -1 : (peak.clamp(0.0, 1.0) * n).round() - 1;
      for (var i = 0; i < n; i++) {
        final x = i * (w + 1);
        final color = i == pk && i >= lit ? C.playhead : (i < lit ? N.g76 : N.g44);
        canvas.drawRRect(RRect.fromLTRBR(x, y, x + w, y + 4, const Radius.circular(1)), Paint()..color = color);
      }
    }
    row(3, l, pl);
    row(15, r, pr);
  }

  @override
  bool shouldRepaint(_MeterPainter o) => o.l != l || o.r != r || o.pl != pl || o.pr != pr;
}

/// A vertical hairline between groups of a bar.
class TpDivider extends StatelessWidget {
  const TpDivider({super.key, this.height = 16});
  final double height;
  @override
  Widget build(BuildContext context) => Container(width: 1, height: height, margin: const EdgeInsets.symmetric(horizontal: 8), color: N.g20);
}

/// The wordmark: name, and the tagline in the small-caps register.
class TpWordmark extends StatelessWidget {
  const TpWordmark({super.key, this.tagline = true, this.version = ''});
  final bool tagline;
  final String version;
  @override
  Widget build(BuildContext context) => Row(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.center, children: [
        Container(
          width: 20,
          height: 20,
          alignment: Alignment.center,
          decoration: BoxDecoration(color: N.g20, borderRadius: BorderRadius.circular(kRad), border: Border.all(color: N.g26)),
          child: Text('M', style: T.title(N.g95).copyWith(fontSize: 12, fontWeight: FontWeight.w700)),
        ),
        const SizedBox(width: 8),
        Text('Motolii', style: T.title().copyWith(fontSize: 16, fontWeight: FontWeight.w700, letterSpacing: -.2)),
        if (version.isNotEmpty) ...[const SizedBox(width: 8), Text(version, style: T.value(N.g56).copyWith(fontSize: 10))],
        if (tagline) ...[
          const SizedBox(width: 12),
          Text('Motion\nfor More Relations.', style: T.label(N.g56).copyWith(height: 1.25)),
        ],
      ]);
}
