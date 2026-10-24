// Part of inputs_parts.dart (cut there by size, as it was).
part of 'inputs_parts.dart';

class RadioDot extends StatelessWidget {
  const RadioDot({super.key, required this.label, required this.on, this.onTap, this.look = Look.concept, this.enabled = true});
  final String label;
  final bool on, enabled;
  final Look look;
  final VoidCallback? onTap;
  @override
  Widget build(BuildContext context) => Dim(
        !enabled,
        Hit(
          onTap: onTap,
          onKey: (e) => _activate(e, () => onTap?.call()),
          builder: (c, h) => Row(mainAxisSize: MainAxisSize.min, children: [
            AnimatedContainer(
              duration: kDur,
              curve: kEase,
              width: 14,
              height: 14,
              alignment: Alignment.center,
              decoration: _markDeco(look, on: on, h: h, round: true),
              child: AnimatedContainer(
                duration: kDur,
                curve: kEase,
                width: on ? 6 : 0,
                height: on ? 6 : 0,
                decoration: BoxDecoration(shape: BoxShape.circle, color: look == Look.glow ? C.mode : N.g100),
              ),
            ),
            const SizedBox(width: 8),
            Text(label, style: T.name(h.hover || on ? N.g95 : N.g76)),
          ]),
        ),
      );
}

// ---- 8/9. slider and range -------------------------------------------------------------------------------------------

class TickSlider extends StatefulWidget {
  const TickSlider({super.key, required this.lo, required this.hi, this.range = false, this.min = 0, this.max = 100, this.ticks = 10, this.snap = false, this.onChanged, this.look = Look.concept, this.enabled = true});
  final double lo, hi, min, max;
  final bool range, snap, enabled;
  final int ticks;
  final Look look;
  final void Function(double lo, double hi)? onChanged;
  @override
  State<TickSlider> createState() => _TickSliderState();
}

class _TickSliderState extends State<TickSlider> {
  int _active = 1;
  bool _drag = false;
  static const _thumb = 12.0;

  double _val(double x, double w) {
    final t = ((x - _thumb / 2) / (w - _thumb)).clamp(0.0, 1.0);
    var v = widget.min + t * (widget.max - widget.min);
    if (widget.snap && widget.ticks > 0) {
      final st = (widget.max - widget.min) / widget.ticks;
      v = widget.min + ((v - widget.min) / st).round() * st;
    }
    return v;
  }

  void _apply(double v) {
    if (widget.range) {
      _active == 0 ? widget.onChanged?.call(math.min(v, widget.hi), widget.hi) : widget.onChanged?.call(widget.lo, math.max(v, widget.lo));
    } else {
      widget.onChanged?.call(widget.min, v);
    }
  }

  @override
  Widget build(BuildContext context) {
    final span = widget.max - widget.min;
    final look = widget.look;
    return Dim(
      !widget.enabled,
      LayoutBuilder(builder: (context, box) {
        final w = box.maxWidth, inner = w - _thumb;
        double tx(double v) => (v - widget.min) / span * inner;
        final lo = widget.range ? widget.lo : widget.min, hi = widget.hi;
        return Hit(
          cursor: SystemMouseCursors.grab,
          onPanDown: (d) {
            if (widget.range) {
              final x = d.localPosition.dx;
              _active = (x - (tx(lo) + _thumb / 2)).abs() <= (x - (tx(hi) + _thumb / 2)).abs() ? 0 : 1;
              if ((tx(lo) - tx(hi)).abs() < 1) _active = x < w / 2 ? 0 : 1;
            }
            setState(() => _drag = true);
            _apply(_val(d.localPosition.dx, w));
          },
          onPanUpdate: (d) => _apply(_val(d.localPosition.dx, w)),
          onPanEnd: (_) => setState(() => _drag = false),
          onTap: () => setState(() => _drag = false),
          onKey: (e) {
            final k = e.logicalKey;
            final dir = k == LogicalKeyboardKey.arrowRight || k == LogicalKeyboardKey.arrowUp ? 1 : (k == LogicalKeyboardKey.arrowLeft || k == LogicalKeyboardKey.arrowDown ? -1 : 0);
            if (dir == 0) return KeyEventResult.ignored;
            final st = widget.snap && widget.ticks > 0 ? span / widget.ticks : span / 100;
            final cur = _active == 0 && widget.range ? lo : hi;
            _apply((cur + dir * st * (HardwareKeyboard.instance.isShiftPressed ? 10 : 1)).clamp(widget.min, widget.max));
            return KeyEventResult.handled;
          },
          builder: (c, h) {
            final lit = h.hover || _drag || h.focus;
            final railFill = look == Look.quiet ? N.g63 : C.mode;
            Widget thumb(double v, bool isActive) {
              final ring = lit && (isActive || !widget.range);
              final base = look == Look.quiet ? N.g76 : N.g95;
              return Positioned(
                left: tx(v),
                top: 4,
                width: _thumb,
                height: _thumb,
                child: AnimatedContainer(
                  duration: kDur,
                  curve: kEase,
                  decoration: BoxDecoration(
                    shape: BoxShape.circle,
                    color: base,
                    border: Border.all(color: ring ? (look == Look.quiet ? N.g100 : Role.selected) : base, width: 2),
                    boxShadow: look == Look.glow && ring ? [BoxShadow(color: C.mode.withValues(alpha: .55), blurRadius: 8)] : const [],
                  ),
                ),
              );
            }

            return SizedBox(
              height: 24,
              child: Stack(clipBehavior: Clip.none, children: [
                Positioned(left: _thumb / 2, right: _thumb / 2, top: 8, height: 4, child: DecoratedBox(decoration: BoxDecoration(color: look == Look.quiet ? N.g20 : N.g15, borderRadius: BorderRadius.circular(2), border: look == Look.concept ? Border.all(color: N.g20) : null))),
                Positioned(left: _thumb / 2 + tx(lo), width: tx(hi) - tx(lo), top: 8, height: 4, child: DecoratedBox(decoration: BoxDecoration(color: railFill, borderRadius: BorderRadius.circular(2)))),
                for (var i = 0; i <= widget.ticks; i++)
                  Positioned(
                    left: _thumb / 2 + inner * i / widget.ticks - .5,
                    top: 18,
                    width: 1,
                    height: 4,
                    child: ColoredBox(color: (i / widget.ticks) * span + widget.min >= lo - 1e-9 && (i / widget.ticks) * span + widget.min <= hi + 1e-9 ? N.g76 : N.g38),
                  ),
                if (widget.range) thumb(lo, _active == 0),
                thumb(hi, _active == 1),
              ]),
            );
          },
        );
      }),
    );
  }
}

// ---- 11/12. colour ---------------------------------------------------------------------------------------------------

Color? parseHex(String s) {
  var t = s.trim().replaceAll('#', '');
  if (t.length == 3) t = t.split('').map((c) => c + c).join();
  if (t.length != 6) return null;
  final v = int.tryParse(t, radix: 16);
  return v == null ? null : Color(0xFF000000 | v);
}

String toHex(Color c) => (c.toARGB32() & 0xFFFFFF).toRadixString(16).padLeft(6, '0').toUpperCase();

class HsvPicker extends StatelessWidget {
  const HsvPicker({super.key, required this.hsv, required this.onChanged});
  final HSVColor hsv;
  final ValueChanged<HSVColor> onChanged;
  @override
  Widget build(BuildContext context) {
    const svH = 112.0, hueH = 12.0;
    return LayoutBuilder(builder: (context, box) {
      final w = box.maxWidth;
      void sv(Offset p) => onChanged(hsv.withSaturation((p.dx / w).clamp(0.0, 1.0)).withValue((1 - p.dy / svH).clamp(0.0, 1.0)));
      void hue(Offset p) => onChanged(hsv.withHue((p.dx / w).clamp(0.0, 1.0) * 359.99));
      final pure = HSVColor.fromAHSV(1, hsv.hue, 1, 1).toColor();
      return Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        MouseRegion(
          cursor: SystemMouseCursors.precise,
          child: GestureDetector(
            behavior: HitTestBehavior.opaque,
            onPanDown: (d) => sv(d.localPosition),
            onPanUpdate: (d) => sv(d.localPosition),
            child: SizedBox(
              height: svH,
              child: Stack(clipBehavior: Clip.none, children: [
                Positioned.fill(
                  child: ClipRRect(
                    borderRadius: BorderRadius.circular(4),
                    child: DecoratedBox(
                      decoration: BoxDecoration(gradient: LinearGradient(colors: [N.g100, pure])),
                      child: const DecoratedBox(decoration: BoxDecoration(gradient: LinearGradient(begin: Alignment.topCenter, end: Alignment.bottomCenter, colors: [Color(0x00000000), N.g00]))),
                    ),
                  ),
                ),
                Positioned(
                  left: hsv.saturation * w - 6,
                  top: (1 - hsv.value) * svH - 6,
                  width: 12,
                  height: 12,
                  child: DecoratedBox(decoration: BoxDecoration(shape: BoxShape.circle, color: hsv.toColor(), border: Border.all(color: N.g100, width: 2))),
                ),
              ]),
            ),
          ),
        ),
        const SizedBox(height: 8),
        MouseRegion(
          cursor: SystemMouseCursors.precise,
          child: GestureDetector(
            behavior: HitTestBehavior.opaque,
            onPanDown: (d) => hue(d.localPosition),
            onPanUpdate: (d) => hue(d.localPosition),
            child: SizedBox(
              height: hueH,
              child: Stack(clipBehavior: Clip.none, children: [
                Positioned.fill(
                  child: DecoratedBox(
                    decoration: BoxDecoration(
                      borderRadius: BorderRadius.circular(4),
                      gradient: LinearGradient(colors: [for (var i = 0; i <= 6; i++) HSVColor.fromAHSV(1, i * 60.0 % 360, 1, 1).toColor()]),
                    ),
                  ),
                ),
                Positioned(
                  left: hsv.hue / 360 * (w - 6),
                  top: -2,
                  width: 6,
                  height: hueH + 4,
                  child: DecoratedBox(decoration: BoxDecoration(color: pure, borderRadius: BorderRadius.circular(3), border: Border.all(color: N.g100, width: 1.5))),
                ),
              ]),
            ),
          ),
        ),
      ]);
    });
  }
}

class HexColorField extends StatefulWidget {
  const HexColorField({super.key, required this.hex, this.look = Look.concept, this.enabled = true, this.initialOpen = false, this.ring = false});
  final String hex;
  final Look look;
  final bool enabled, initialOpen, ring;
  @override
  State<HexColorField> createState() => _HexColorFieldState();
}

class _HexColorFieldState extends State<HexColorField> {
  late HSVColor hsv = HSVColor.fromColor(parseHex(widget.hex) ?? N.g95);
  late String typed = widget.hex.replaceAll('#', '').toUpperCase();
  @override
  void didUpdateWidget(HexColorField o) {
    super.didUpdateWidget(o);
    if (o.hex != widget.hex) {
      hsv = HSVColor.fromColor(parseHex(widget.hex) ?? N.g95);
      typed = widget.hex.replaceAll('#', '').toUpperCase();
    }
  }

  void _pick(HSVColor n) => setState(() {
        hsv = n;
        typed = toHex(n.toColor());
      });

  @override
  Widget build(BuildContext context) {
    final col = hsv.toColor();
    final look = widget.look;
    return Dim(
      !widget.enabled,
      Row(children: [
        PopAnchor(
          width: 184,
          initialOpen: widget.initialOpen,
          anchor: (ctx, open, toggle) => Hit(
            onTap: toggle,
            onKey: (e) => _activate(e, toggle),
            builder: (c, h) => FocusRing(
              on: widget.ring || h.focus,
              child: AnimatedContainer(
              duration: kDur,
              curve: kEase,
              width: kH,
              height: kH,
              padding: const EdgeInsets.all(2),
              decoration: BoxDecoration(
                color: look == Look.quiet ? N.g13 : N.g07,
                borderRadius: BorderRadius.circular(4),
                border: Border.all(color: h.focus || open || widget.ring ? Role.selected : (h.hover ? N.g44 : (look == Look.quiet ? N.g13 : N.g20))),
                boxShadow: look == Look.glow && open ? [BoxShadow(color: C.mode.withValues(alpha: .35), blurRadius: 8)] : const [],
              ),
              child: DecoratedBox(decoration: BoxDecoration(color: col, borderRadius: BorderRadius.circular(2))),
            ),
            ),
          ),
          menu: (ctx, close) => FloatCard(
            padding: const EdgeInsets.all(8),
            child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
              HsvPicker(hsv: hsv, onChanged: _pick),
              const SizedBox(height: 8),
              Row(children: [
                for (final f in Fam.all) ...[
                  Hit(
                    focusable: false,
                    onTap: () => _pick(HSVColor.fromColor(f.c)),
                    builder: (c, h) => AnimatedContainer(
                      duration: kDur,
                      curve: kEase,
                      width: 20,
                      height: 20,
                      decoration: BoxDecoration(color: f.c, borderRadius: BorderRadius.circular(4), border: Border.all(color: h.hover ? N.g100 : N.g13, width: 1.5)),
                    ),
                  ),
                  if (f != Fam.all.last) const Spacer(),
                ],
              ]),
            ]),
          ),
        ),
        const SizedBox(width: 4),
        Expanded(
          child: TextBox(
            text: typed,
            mono: true,
            look: look,
            ring: widget.ring,
            maxLength: 7,
            leading: Text('#', style: T.value(N.g56)),
            invalid: parseHex(typed) == null,
            onChanged: (s) => setState(() {
              typed = s.replaceAll('#', '').toUpperCase();
              final c = parseHex(typed);
              if (c != null && typed.length == 6) hsv = HSVColor.fromColor(c);
            }),
            onSubmit: (s) => setState(() => typed = toHex(hsv.toColor())),
          ),
        ),
      ]),
    );
  }
}

// ---- 13. tags --------------------------------------------------------------------------------------------------------

class TagInput extends StatefulWidget {
  const TagInput({super.key, required this.tags, this.look = Look.concept, this.enabled = true});
  final List<String> tags;
  final Look look;
  final bool enabled;
  @override
  State<TagInput> createState() => _TagInputState();
}

class _TagInputState extends State<TagInput> {
  late List<String> tags = [...widget.tags];
  final _c = TextEditingController();
  final _n = FocusNode();
  bool _hover = false, _focus = false;
  @override
  void initState() {
    super.initState();
    _n.addListener(() => setState(() => _focus = _n.hasFocus));
  }

  @override
  void didUpdateWidget(TagInput o) {
    super.didUpdateWidget(o);
    if (o.tags.join('|') != widget.tags.join('|')) tags = [...widget.tags];
  }

  @override
  void dispose() {
    _c.dispose();
    _n.dispose();
    super.dispose();
  }

  void _add() {
    final t = _c.text.replaceAll(',', '').trim();
    _c.clear();
    if (t.isNotEmpty && !tags.contains(t)) setState(() => tags.add(t));
  }

  @override
  Widget build(BuildContext context) {
    final quiet = widget.look == Look.quiet;
    return Dim(
      !widget.enabled,
      MouseRegion(
        cursor: SystemMouseCursors.text,
        onEnter: (_) => setState(() => _hover = true),
        onExit: (_) => setState(() => _hover = false),
        child: GestureDetector(
          behavior: HitTestBehavior.opaque,
          onTap: _n.requestFocus,
          child: AnimatedContainer(
            duration: kDur,
            curve: kEase,
            constraints: const BoxConstraints(minHeight: kH),
            padding: const EdgeInsets.all(3),
            decoration: wellDeco(widget.look, hover: _hover, focus: _focus),
            child: Wrap(spacing: 4, runSpacing: 4, crossAxisAlignment: WrapCrossAlignment.center, children: [
              for (final t in tags)
                Container(
                  height: 16,
                  padding: const EdgeInsets.only(left: 6, right: 2),
                  decoration: BoxDecoration(color: quiet ? N.g20 : N.g15, borderRadius: BorderRadius.circular(3), border: Border.all(color: quiet ? N.g20 : N.g20)),
                  child: Row(mainAxisSize: MainAxisSize.min, children: [
                    Text(t, style: T.label(N.g91)),
                    const SizedBox(width: 2),
                    Hit(
                      focusable: false,
                      onTap: () => setState(() => tags.remove(t)),
                      builder: (c, h) => AnimatedContainer(
                        duration: kDur,
                        curve: kEase,
                        width: 12,
                        height: 12,
                        alignment: Alignment.center,
                        decoration: BoxDecoration(color: h.hover ? N.g38 : const Color(0x00000000), borderRadius: BorderRadius.circular(2)),
                        child: GlyphIcon(Glyph.close, size: 8, color: h.hover ? N.g100 : N.g56, weight: 1.2),
                      ),
                    ),
                  ]),
                ),
              SizedBox(
                width: 72,
                height: 16,
                child: Focus(
                  canRequestFocus: false,
                  skipTraversal: true,
                  onKeyEvent: (n, e) {
                    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.backspace && _c.text.isEmpty && tags.isNotEmpty) {
                      setState(() => tags.removeLast());
                      return KeyEventResult.handled;
                    }
                    return KeyEventResult.ignored;
                  },
                  child: Stack(alignment: Alignment.centerLeft, children: [
                    if (!_focus && tags.isEmpty) IgnorePointer(child: Text('Add tag', style: T.label(N.g44))),
                    Padding(
                      padding: const EdgeInsets.only(left: 2),
                      child: EditableText(
                        controller: _c,
                        focusNode: _n,
                        style: T.label(N.g95),
                        cursorColor: Role.selected,
                        backgroundCursorColor: N.g26,
                        selectionColor: Role.selected.withValues(alpha: .35),
                        maxLines: 1,
                        cursorWidth: 1,
                        onChanged: (s) {
                          if (s.endsWith(',')) _add();
                        },
                        onEditingComplete: _add,
                      ),
                    ),
                  ]),
                ),
              ),
            ]),
          ),
        ),
      ),
    );
  }
}

// ---- 14. stepper -----------------------------------------------------------------------------------------------------
