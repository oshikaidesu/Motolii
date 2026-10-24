// Part of inputs_parts.dart (cut there by size, as it was).
part of 'inputs_parts.dart';

class _Vec3RowState extends State<Vec3Row> {
  late List<double> v = [widget.x, widget.y, widget.z];
  @override
  void didUpdateWidget(Vec3Row o) {
    super.didUpdateWidget(o);
    if (o.x != widget.x || o.y != widget.y || o.z != widget.z) v = [widget.x, widget.y, widget.z];
  }

  @override
  Widget build(BuildContext context) {
    // [open] axis colours: muted tints, shown only as a 2 px tick
    final axes = [('X', Fam.scatter.c.withValues(alpha: .6)), ('Y', Fam.along.c.withValues(alpha: .6)), ('Z', Fam.stagger.c.withValues(alpha: .6))];
    return Row(children: [
      for (var i = 0; i < 3; i++) ...[
        if (i > 0) const SizedBox(width: 4),
        Expanded(
          child: ScrubField(
            label: axes[i].$1,
            accent: axes[i].$2,
            axis: true,
            fill: false,
            value: v[i],
            decimals: widget.decimals,
            unit: widget.unit,
            look: widget.look,
            enabled: widget.enabled,
            onChanged: (n) => setState(() => v[i] = n),
          ),
        ),
      ],
    ]);
  }
}

// ---- 3. link / ratio -------------------------------------------------------------------------------------------------

class LinkToggle extends StatelessWidget {
  const LinkToggle({super.key, required this.on, this.onChanged, this.look = Look.concept});
  final bool on;
  final Look look;
  final ValueChanged<bool>? onChanged;
  @override
  Widget build(BuildContext context) => Hit(
        onTap: () => onChanged?.call(!on),
        onKey: (e) {
          if (e.logicalKey == LogicalKeyboardKey.space || e.logicalKey == LogicalKeyboardKey.enter) {
            onChanged?.call(!on);
            return KeyEventResult.handled;
          }
          return KeyEventResult.ignored;
        },
        builder: (ctx, h) {
          final Color fill = on ? (look == Look.glow ? C.mode.withValues(alpha: .18) : N.g20) : (h.hover ? N.g15 : (look == Look.quiet ? N.g13 : N.g10));
          final Color ink = on ? (look == Look.quiet ? N.g95 : C.mode) : (h.hover ? N.g95 : N.g56);
          return AnimatedContainer(
            duration: kDur,
            curve: kEase,
            width: kH,
            height: kH,
            alignment: Alignment.center,
            decoration: BoxDecoration(
              color: h.down ? N.g26 : fill,
              borderRadius: BorderRadius.circular(4),
              border: Border.all(color: h.focus ? Role.selected : (on && look == Look.glow ? Role.selected.withValues(alpha: .6) : (look == Look.quiet ? fill : N.g20))),
              boxShadow: on && look == Look.glow ? [BoxShadow(color: C.mode.withValues(alpha: .3), blurRadius: 8)] : const [],
            ),
            child: GlyphIcon(on ? Glyph.linked : Glyph.unlinked, size: 14, color: ink),
          );
        },
      );
}

class RatioPair extends StatefulWidget {
  const RatioPair({super.key, required this.w, required this.h, required this.linked, this.look = Look.concept, this.enabled = true});
  final double w, h;
  final bool linked, enabled;
  final Look look;
  @override
  State<RatioPair> createState() => _RatioPairState();
}

class _RatioPairState extends State<RatioPair> {
  late double w = widget.w, h = widget.h;
  late bool linked = widget.linked;
  late double ratio = widget.w / widget.h;
  @override
  void didUpdateWidget(RatioPair o) {
    super.didUpdateWidget(o);
    if (o.w != widget.w || o.h != widget.h || o.linked != widget.linked) {
      w = widget.w;
      h = widget.h;
      linked = widget.linked;
      ratio = w / h;
    }
  }

  @override
  Widget build(BuildContext context) => Row(children: [
        Expanded(
          child: ScrubField(
            label: 'W', unit: 'px', decimals: 0, min: 1, perPx: 1, fill: false, value: w, look: widget.look, enabled: widget.enabled,
            onChanged: (n) => setState(() {
              w = n;
              if (linked) h = math.max(1, (n / ratio).roundToDouble());
            }),
          ),
        ),
        const SizedBox(width: 4),
        Dim(!widget.enabled, LinkToggle(on: linked, look: widget.look, onChanged: (v) => setState(() {
              linked = v;
              if (v) ratio = w / h;
            }))),
        const SizedBox(width: 4),
        Expanded(
          child: ScrubField(
            label: 'H', unit: 'px', decimals: 0, min: 1, perPx: 1, fill: false, value: h, look: widget.look, enabled: widget.enabled,
            onChanged: (n) => setState(() {
              h = n;
              if (linked) w = math.max(1, (n * ratio).roundToDouble());
            }),
          ),
        ),
      ]);
}

// ---- 4/5. text and search --------------------------------------------------------------------------------------------

class TextBox extends StatefulWidget {
  const TextBox({super.key, required this.text, this.hint = '', this.onChanged, this.onSubmit, this.look = Look.concept, this.enabled = true, this.leading, this.clearable = false, this.mono = false, this.invalid = false, this.maxLength, this.ring = false});
  final String text, hint;
  final ValueChanged<String>? onChanged, onSubmit;
  final Look look;
  final bool enabled, clearable, mono, invalid, ring;
  final Widget? leading;
  final int? maxLength;
  @override
  State<TextBox> createState() => _TextBoxState();
}

class _TextBoxState extends State<TextBox> {
  late final _c = TextEditingController(text: widget.text);
  final _n = FocusNode();
  bool _hover = false, _focus = false;
  @override
  void initState() {
    super.initState();
    _c.addListener(() => setState(() {}));
    _n.addListener(() {
      setState(() => _focus = _n.hasFocus);
      if (!_n.hasFocus) widget.onSubmit?.call(_c.text);
    });
  }

  @override
  void didUpdateWidget(TextBox o) {
    super.didUpdateWidget(o);
    if (widget.text != _c.text && (!_n.hasFocus || widget.text != o.text)) _c.text = widget.text;
  }

  @override
  void dispose() {
    _c.dispose();
    _n.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final style = widget.mono ? T.value(N.g95) : T.name(N.g95);
    return Dim(
      !widget.enabled,
      FocusRing(
        on: widget.ring,
        child: MouseRegion(
        cursor: SystemMouseCursors.text,
        onEnter: (_) => setState(() => _hover = true),
        onExit: (_) => setState(() => _hover = false),
        child: GestureDetector(
          behavior: HitTestBehavior.opaque,
          onTap: _n.requestFocus,
          child: AnimatedContainer(
            duration: kDur,
            curve: kEase,
            height: kH,
            padding: const EdgeInsets.symmetric(horizontal: 8),
            decoration: wellDeco(widget.look, hover: _hover, focus: _focus || widget.ring, invalid: widget.invalid),
            child: Row(crossAxisAlignment: CrossAxisAlignment.center, children: [
              if (widget.leading != null) ...[widget.leading!, const SizedBox(width: 6)],
              Expanded(
                child: Stack(alignment: Alignment.centerLeft, children: [
                  if (_c.text.isEmpty) IgnorePointer(child: Text(widget.hint, style: T.name(N.g44), maxLines: 1)),
                  EditableText(
                    controller: _c,
                    focusNode: _n,
                    style: style,
                    cursorColor: Role.selected,
                    backgroundCursorColor: N.g26,
                    selectionColor: Role.selected.withValues(alpha: .35),
                    maxLines: 1,
                    cursorWidth: 1,
                    readOnly: !widget.enabled,
                    inputFormatters: widget.maxLength == null ? null : [LengthLimitingTextInputFormatter(widget.maxLength)],
                    onChanged: widget.onChanged,
                    onSubmitted: (s) {
                      widget.onSubmit?.call(s);
                      _n.requestFocus();
                    },
                  ),
                ]),
              ),
              if (widget.invalid) const ErrMark(lead: 6),
              if (widget.clearable && _c.text.isNotEmpty)
                Hit(
                  focusable: false,
                  onTap: () {
                    _c.clear();
                    widget.onChanged?.call('');
                    _n.requestFocus();
                  },
                  builder: (ctx, h) => AnimatedContainer(
                    duration: kDur,
                    curve: kEase,
                    width: 16,
                    height: 16,
                    alignment: Alignment.center,
                    decoration: BoxDecoration(color: h.down ? N.g38 : (h.hover ? N.g26 : N.g20), borderRadius: BorderRadius.circular(8)),
                    child: GlyphIcon(Glyph.close, size: 10, color: h.hover ? N.g95 : N.g63, weight: 1.2),
                  ),
                ),
            ]),
          ),
        ),
      ),
      ),
    );
  }
}

// ---- 6. popover anchor and dropdown ----------------------------------------------------------------------------------

class PopAnchor extends StatefulWidget {
  const PopAnchor({super.key, required this.anchor, required this.menu, this.width, this.initialOpen = false});
  final Widget Function(BuildContext, bool open, VoidCallback toggle) anchor;
  final Widget Function(BuildContext, VoidCallback close) menu;
  final double? width;
  final bool initialOpen;
  @override
  State<PopAnchor> createState() => _PopAnchorState();
}

class _PopAnchorState extends State<PopAnchor> {
  final _link = LayerLink();
  final _ctl = OverlayPortalController();
  bool _open = false;
  void _set(bool o) {
    if (o == _open) return;
    setState(() => _open = o);
    o ? _ctl.show() : _ctl.hide();
  }

  @override
  void initState() {
    super.initState();
    if (widget.initialOpen) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted) _set(true);
      });
    }
  }

  @override
  void didUpdateWidget(PopAnchor o) {
    super.didUpdateWidget(o);
    if (o.initialOpen != widget.initialOpen) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted) _set(widget.initialOpen);
      });
    }
  }

  @override
  Widget build(BuildContext context) => CompositedTransformTarget(
        link: _link,
        child: OverlayPortal(
          controller: _ctl,
          overlayChildBuilder: (_) {
            final w = widget.width ?? (this.context.findRenderObject() as RenderBox?)?.size.width ?? 160;
            return Stack(children: [
              Positioned.fill(child: GestureDetector(behavior: HitTestBehavior.translucent, onTap: () => _set(false), child: const SizedBox.expand())),
              CompositedTransformFollower(
                link: _link,
                showWhenUnlinked: false,
                targetAnchor: Alignment.bottomLeft,
                followerAnchor: Alignment.topLeft,
                offset: const Offset(0, 4),
                child: Align(
                  alignment: Alignment.topLeft,
                  child: SizedBox(
                    width: w,
                    child: TweenAnimationBuilder<double>(
                      tween: Tween(begin: 0, end: 1),
                      duration: kDur,
                      curve: kEase,
                      builder: (c, t, child) => Opacity(opacity: t, child: Transform.translate(offset: Offset(0, (1 - t) * -4), child: child)),
                      child: widget.menu(context, () => _set(false)),
                    ),
                  ),
                ),
              ),
            ]);
          },
          child: widget.anchor(context, _open, () => _set(!_open)),
        ),
      );
}

/// A floating surface: the only kind of thing in the inputs that carries a shadow.
class FloatCard extends StatelessWidget {
  const FloatCard({super.key, required this.child, this.padding = const EdgeInsets.all(4)});
  final Widget child;
  final EdgeInsets padding;
  @override
  Widget build(BuildContext context) => Container(
        padding: padding,
        decoration: BoxDecoration(
          color: N.g13,
          borderRadius: BorderRadius.circular(6),
          border: Border.all(color: N.g20),
          boxShadow: [BoxShadow(color: N.g00.withValues(alpha: .5), blurRadius: 16, offset: const Offset(0, 6))],
        ),
        child: child,
      );
}

class Dropdown extends StatelessWidget {
  const Dropdown({super.key, required this.items, required this.index, this.onChanged, this.look = Look.concept, this.enabled = true, this.initialOpen = false, this.ring = false});
  final List<String> items;
  final int index;
  final ValueChanged<int>? onChanged;
  final Look look;
  final bool enabled, initialOpen, ring;
  @override
  Widget build(BuildContext context) => Dim(
        !enabled,
        PopAnchor(
          initialOpen: initialOpen,
          anchor: (ctx, open, toggle) => Hit(
            onTap: toggle,
            ringFromKeyboardOnly: true,
            onKey: (e) {
              final k = e.logicalKey;
              if (k == LogicalKeyboardKey.enter || k == LogicalKeyboardKey.space || k == LogicalKeyboardKey.arrowDown) {
                if (!open) toggle();
                return KeyEventResult.handled;
              }
              if (k == LogicalKeyboardKey.escape && open) {
                toggle();
                return KeyEventResult.handled;
              }
              return KeyEventResult.ignored;
            },
            builder: (c, h) => FocusRing(
              on: ring || h.focus,
              child: AnimatedContainer(
                duration: kDur,
                curve: kEase,
                height: kH,
                padding: const EdgeInsets.only(left: 8, right: 6),
                decoration: wellDeco(look, hover: h.hover || h.down, focus: h.focus || open || ring),
                child: Row(children: [
                  Expanded(child: Text(items[index], style: T.name(N.g95), maxLines: 1, overflow: TextOverflow.ellipsis)),
                  AnimatedRotation(turns: open ? .5 : 0, duration: kDur, curve: kEase, child: GlyphIcon(Glyph.chevron, size: 14, color: h.hover || open ? N.g95 : N.g56)),
                ]),
              ),
            ),
          ),
          menu: (ctx, close) => FloatCard(
            child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
              for (var i = 0; i < items.length; i++)
                Hit(
                  focusable: false,
                  onTap: () {
                    onChanged?.call(i);
                    close();
                  },
                  builder: (c, h) => AnimatedContainer(
                    duration: kDur,
                    curve: kEase,
                    height: kH,
                    padding: const EdgeInsets.symmetric(horizontal: 4),
                    decoration: BoxDecoration(color: h.hover ? N.g20 : null, borderRadius: BorderRadius.circular(4)),
                    child: Row(children: [
                      SizedBox(width: 16, child: i == index ? GlyphIcon(Glyph.check, size: 12, color: C.mode) : null),
                      const SizedBox(width: 4),
                      Expanded(child: Text(items[i], style: T.name(i == index || h.hover ? N.g95 : N.g76))),
                    ]),
                  ),
                ),
            ]),
          ),
        ),
      );
}

// ---- 7. checkbox and radio -------------------------------------------------------------------------------------------

KeyEventResult _activate(KeyEvent e, VoidCallback f) {
  if (e.logicalKey == LogicalKeyboardKey.space || e.logicalKey == LogicalKeyboardKey.enter) {
    f();
    return KeyEventResult.handled;
  }
  return KeyEventResult.ignored;
}

/// Shared look of the 14 px mark of a checkbox or radio.
BoxDecoration _markDeco(Look look, {required bool on, required HitState h, required bool round}) {
  Color fill, border;
  var glow = const <BoxShadow>[];
  switch (look) {
    case Look.concept:
      fill = on ? C.mode : (h.hover ? N.g10 : N.g07);
      border = h.focus ? N.g100 : (on ? Role.selected : (h.hover ? N.g44 : N.g26));
    case Look.quiet:
      fill = on ? N.g20 : (h.hover ? N.g15 : N.g13);
      border = h.focus ? N.g100 : (on ? N.g63 : N.g20);
    case Look.glow:
      fill = on ? C.mode.withValues(alpha: .22) : N.g07;
      border = h.focus ? N.g100 : (on ? Role.selected : (h.hover ? N.g44 : N.g26));
      if (on) glow = [BoxShadow(color: C.mode.withValues(alpha: .35), blurRadius: 6)];
  }
  if (h.down) fill = N.g26;
  return BoxDecoration(color: fill, shape: round ? BoxShape.circle : BoxShape.rectangle, borderRadius: round ? null : BorderRadius.circular(3), border: Border.all(color: border), boxShadow: glow);
}

class CheckBox extends StatelessWidget {
  const CheckBox({super.key, required this.label, required this.value, this.onChanged, this.look = Look.concept, this.enabled = true});
  final String label;
  final bool? value; // null = mixed
  final ValueChanged<bool>? onChanged;
  final Look look;
  final bool enabled;
  @override
  Widget build(BuildContext context) {
    void flip() => onChanged?.call(value != true);
    return Dim(
      !enabled,
      Hit(
        onTap: flip,
        onKey: (e) => _activate(e, flip),
        builder: (c, h) {
          final on = value != false;
          final ink = look == Look.glow ? C.mode : (look == Look.quiet ? N.g95 : N.g100);
          return Row(mainAxisSize: MainAxisSize.min, children: [
            AnimatedContainer(
              duration: kDur,
              curve: kEase,
              width: 14,
              height: 14,
              alignment: Alignment.center,
              decoration: _markDeco(look, on: on, h: h, round: false),
              child: AnimatedOpacity(opacity: on ? 1 : 0, duration: kDur, curve: kEase, child: GlyphIcon(value == null ? Glyph.minus : Glyph.check, size: 12, color: ink, weight: 1.6)),
            ),
            const SizedBox(width: 8),
            Text(label, style: T.name(h.hover || on ? N.g95 : N.g76)),
          ]);
        },
      ),
    );
  }
}
