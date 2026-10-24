// Part of inputs_parts.dart (cut there by size, as it was).
part of 'inputs_parts.dart';

class _RepeatBtn extends StatefulWidget {
  const _RepeatBtn({required this.glyph, required this.step, required this.enabled, required this.look});
  final Glyph glyph;
  final bool Function() step;
  final bool enabled;
  final Look look;
  @override
  State<_RepeatBtn> createState() => _RepeatBtnState();
}

class _RepeatBtnState extends State<_RepeatBtn> {
  Timer? _t;
  void _stop() {
    _t?.cancel();
    _t = null;
  }

  void _start() {
    if (!widget.step()) return;
    _t = Timer(const Duration(milliseconds: 380), () {
      _t = Timer.periodic(const Duration(milliseconds: 70), (t) {
        if (!widget.step()) _stop();
      });
    });
  }

  @override
  void dispose() {
    _stop();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Hit(
        enabled: widget.enabled,
        builder: (c, h) => Listener(
          onPointerDown: (_) => _start(),
          onPointerUp: (_) => _stop(),
          onPointerCancel: (_) => _stop(),
          child: AnimatedContainer(
            duration: kDur,
            curve: kEase,
            width: kH - 2,
            alignment: Alignment.center,
            color: h.down ? N.g26 : (h.hover ? N.g20 : const Color(0x00000000)),
            child: GlyphIcon(widget.glyph, size: 12, color: !widget.enabled ? N.g38 : (h.hover ? N.g100 : N.g76), weight: 1.4),
          ),
        ),
      );
}

class StepperField extends StatelessWidget {
  const StepperField({super.key, required this.value, required this.min, required this.max, this.step = 1, this.unit = '', this.onChanged, this.look = Look.concept, this.enabled = true});
  final double value, min, max, step;
  final String unit;
  final ValueChanged<double>? onChanged;
  final Look look;
  final bool enabled;
  @override
  Widget build(BuildContext context) {
    bool move(int dir) {
      final n = (value + dir * step * (HardwareKeyboard.instance.isShiftPressed ? 10 : 1)).clamp(min, max).toDouble();
      if (n == value) return false;
      onChanged?.call(n);
      return true;
    }

    return Dim(
      !enabled,
      Container(
        height: kH,
        decoration: wellDeco(look),
        child: ClipRRect(
          borderRadius: BorderRadius.circular(3),
          child: Row(children: [
            _RepeatBtn(glyph: Glyph.minus, step: () => move(-1), enabled: value > min, look: look),
            const SizedBox(width: 1, child: ColoredBox(color: N.g20)),
            Expanded(child: Center(child: Text.rich(TextSpan(children: [TextSpan(text: value.toStringAsFixed(step < 1 ? 1 : 0), style: T.value(N.g95)), if (unit.isNotEmpty) TextSpan(text: ' $unit', style: T.value(N.g56))])))),
            const SizedBox(width: 1, child: ColoredBox(color: N.g20)),
            _RepeatBtn(glyph: Glyph.plus, step: () => move(1), enabled: value < max, look: look),
          ]),
        ),
      ),
    );
  }
}

// ---- 15. toggle button group -----------------------------------------------------------------------------------------

class ToggleGroup extends StatelessWidget {
  const ToggleGroup({super.key, required this.icons, required this.selected, this.multi = false, this.onChanged, this.look = Look.concept, this.enabled = true});
  final List<Widget Function(Color)> icons;
  final Set<int> selected;
  final bool multi, enabled;
  final Look look;
  final ValueChanged<Set<int>>? onChanged;
  @override
  Widget build(BuildContext context) => Dim(
        !enabled,
        Container(
          padding: const EdgeInsets.all(2),
          decoration: BoxDecoration(color: look == Look.quiet ? N.g13 : N.g07, borderRadius: BorderRadius.circular(6), border: look == Look.concept ? Border.all(color: N.g20) : null),
          child: Row(mainAxisSize: MainAxisSize.min, children: [
            for (var i = 0; i < icons.length; i++) ...[
              if (i > 0) const SizedBox(width: 2),
              Hit(
                onTap: () {
                  final n = {...selected};
                  if (multi) {
                    n.contains(i) ? n.remove(i) : n.add(i);
                  } else {
                    n..clear()..add(i);
                  }
                  onChanged?.call(n);
                },
                onKey: (e) => _activate(e, () {
                  final n = {...selected};
                  if (multi) {
                    n.contains(i) ? n.remove(i) : n.add(i);
                  } else {
                    n..clear()..add(i);
                  }
                  onChanged?.call(n);
                }),
                builder: (c, h) {
                  final on = selected.contains(i);
                  final Color fill = on ? N.g20 : (h.hover ? N.g15 : const Color(0x00000000));
                  final Color ink = on || h.hover ? N.g95 : N.g56;
                  return AnimatedContainer(
                    duration: kDur,
                    curve: kEase,
                    width: 28,
                    height: kH - 4,
                    alignment: Alignment.center,
                    decoration: BoxDecoration(
                      color: h.down ? N.g26 : fill,
                      borderRadius: BorderRadius.circular(4),
                      border: Border.all(color: h.focus ? Role.selected : const Color(0x00000000)),
                    ),
                    child: Stack(alignment: Alignment.center, children: [
                      icons[i](ink),
                      Positioned(left: 6, right: 6, bottom: 1, height: 1, child: AnimatedOpacity(opacity: on ? 1 : 0, duration: kDur, curve: kEase, child: ColoredBox(color: Role.selected))),
                    ]),
                  );
                },
              ),
            ],
          ]),
        ),
      );
}

// ---- 16. curve picker ------------------------------------------------------------------------------------------------

const curvePresets = <(String, double, double, double, double)>[
  ('Linear', 0, 0, 1, 1),
  ('In', .42, 0, 1, 1),
  ('Out', 0, 0, .58, 1),
  ('In-Out', .42, 0, .58, 1),
  ('Back Out', .34, 1.56, .64, 1),
  ('Back In', .36, 0, .66, -.56),
];

class _CurvePainter extends CustomPainter {
  const _CurvePainter(this.c, this.ink);
  final (String, double, double, double, double) c;
  final Color ink;
  @override
  void paint(Canvas cv, Size s) {
    const padX = 10.0, padY = 13.0;
    final inner = s.height - padY * 2;
    double px(double x) => padX + x * (s.width - padX * 2);
    double py(double y) => padY + (1 - y) * inner;
    final base = Paint()
      ..color = N.g20
      ..strokeWidth = 1;
    cv.drawLine(Offset(padX, py(0)), Offset(s.width - padX, py(0)), base);
    cv.drawLine(Offset(padX, py(1)), Offset(s.width - padX, py(1)), base);
    final st = Paint()
      ..color = ink
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.5
      ..strokeCap = StrokeCap.round;
    cv.drawPath(Path()..moveTo(px(0), py(0))..cubicTo(px(c.$2), py(c.$3), px(c.$4), py(c.$5), px(1), py(1)), st);
    final dot = Paint()..color = ink;
    cv.drawCircle(Offset(px(0), py(0)), 2, dot);
    cv.drawCircle(Offset(px(1), py(1)), 2, dot);
  }

  @override
  bool shouldRepaint(_CurvePainter o) => o.c != c || o.ink != ink;
}

class CurvePicker extends StatelessWidget {
  const CurvePicker({super.key, required this.index, this.onChanged, this.look = Look.concept, this.enabled = true});
  final int index;
  final ValueChanged<int>? onChanged;
  final Look look;
  final bool enabled;
  @override
  Widget build(BuildContext context) {
    String f(double v) => v.toStringAsFixed(2);
    final s = curvePresets[index];
    return Dim(
      !enabled,
      Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
        Wrap(spacing: 6, runSpacing: 8, children: [
          for (var i = 0; i < curvePresets.length; i++)
            Hit(
              onTap: () => onChanged?.call(i),
              onKey: (e) => _activate(e, () => onChanged?.call(i)),
              builder: (c, h) {
                final on = i == index;
                final Color fill = on ? N.g20 : (look == Look.quiet ? (h.hover ? N.g15 : N.g13) : (h.hover ? N.g10 : N.g07));
                final Color border = h.focus ? N.g100 : (on ? Role.selected : (h.hover ? N.g38 : (look == Look.quiet ? fill : N.g20)));
                return Column(mainAxisSize: MainAxisSize.min, children: [
                  AnimatedContainer(
                    duration: kDur,
                    curve: kEase,
                    width: 64,
                    height: 48,
                    decoration: BoxDecoration(
                      color: h.down ? N.g15 : fill,
                      borderRadius: BorderRadius.circular(6),
                      border: Border.all(color: border),
                      boxShadow: look == Look.glow && on ? [BoxShadow(color: C.mode.withValues(alpha: .35), blurRadius: 8)] : const [],
                    ),
                    child: CustomPaint(painter: _CurvePainter(curvePresets[i], on ? N.g100 : (h.hover ? N.g91 : N.g63))),
                  ),
                  const SizedBox(height: 4),
                  Text(curvePresets[i].$1, style: T.micro(on ? N.g95 : N.g56)),
                ]);
              },
            ),
        ]),
        const SizedBox(height: 12),
        Text('cubic-bezier(${f(s.$2)}, ${f(s.$3)}, ${f(s.$4)}, ${f(s.$5)})', style: T.value(N.g63), maxLines: 1),
      ]),
    );
  }
}
