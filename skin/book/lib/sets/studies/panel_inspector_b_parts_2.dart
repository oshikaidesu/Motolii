// Part of panel_inspector_b_parts.dart (cut there by size, as it was).
part of 'panel_inspector_b.dart';

class _FieldState extends State<_Field> {
  final focus = FocusNode();
  bool hover = false;
  @override
  void initState() {
    super.initState();
    focus.addListener(() => setState(() {}));
    widget.ctrl.addListener(() => setState(() {}));
  }

  @override
  void dispose() {
    focus.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final has = widget.ctrl.text.isNotEmpty;
    return MouseRegion(
      onEnter: (_) => setState(() => hover = true),
      onExit: (_) => setState(() => hover = false),
      child: AnimatedContainer(
        duration: Mo.dur,
        curve: Mo.ease,
        height: 28,
        padding: const EdgeInsets.only(left: 8, right: 4),
        decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(4), border: Border.all(color: focus.hasFocus || widget.pose == _Pose.focused ? N.g63 : (hover || widget.pose == _Pose.hover ? N.g38 : N.g20))),
        child: Row(children: [
          Expanded(
            child: Stack(alignment: Alignment.centerLeft, children: [
              if (!has) Text(widget.hint, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g63).copyWith(fontSize: 11)),
              Focus(
                onKeyEvent: (n, e) {
                  if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape && has) {
                    widget.ctrl.clear();
                    return KeyEventResult.handled;
                  }
                  return KeyEventResult.ignored;
                },
                child: EditableText(
                  controller: widget.ctrl,
                  focusNode: focus,
                  style: T.name(N.g95).copyWith(fontWeight: FontWeight.w400),
                  cursorColor: N.g95,
                  backgroundCursorColor: N.g26,
                  selectionColor: Role.selected.withValues(alpha: .35),
                  cursorWidth: 1,
                  maxLines: 1,
                ),
              ),
            ]),
          ),
          if (widget.count != null && has) Padding(padding: const EdgeInsets.only(left: 8, right: 4), child: Text(widget.count!, style: _cnt(N.g76))),
          if (has) _Word('Clear', onTap: () {
            widget.ctrl.clear();
          }),
        ]),
      ),
    );
  }
}

// ---- controls: the number cell and its neighbours -----------------------------------------------------------------------------

/// A unit as it is written: the degree ring is too small at 10 px, so it is the word deg.
String _unit(String u) => u == '°' ? 'deg' : u;

/// The one number rule of the book (D-I3): drag right = up, 1 step per px x Shift 10 / Alt 0.1, arrows and (optionally) wheel the same, click types, Backspace resets.
class _Num extends StatefulWidget {
  const _Num({required this.value, required this.onDelta, this.onType, this.onReset, this.dec = 0, this.unit = '', this.per = 1, this.step = 1, this.mixed = false, this.enabled = true, this.wheel = false, this.word, this.pose, this.typing = false, this.changed = false});
  final _Pose? pose;
  final bool changed; // a changed value gets a 1 px g63 edge under the cell (shape, not only tone)
  final double value, per, step;
  final int dec;
  final String unit;
  final bool mixed, enabled, wheel;
  final bool typing; // holds the typing state (Parts sheets): the caret cell, nothing focused
  final String? word; // replaces the number with a word (Mixed, Locked)
  final ValueChanged<double> onDelta;
  final ValueChanged<double>? onType;
  final VoidCallback? onReset;
  @override
  State<_Num> createState() => _NumState();
}

class _NumState extends State<_Num> {
  final focus = FocusNode(), editFocus = FocusNode();
  final ctrl = TextEditingController();
  bool hover = false, dragging = false, editing = false;

  @override
  void initState() {
    super.initState();
    if (widget.typing) {
      ctrl.text = _f(widget.value, widget.dec);
    }
    focus.addListener(() => setState(() {}));
    editFocus.addListener(() {
      if (!editFocus.hasFocus && editing) {
        _commit();
      }
    });
  }

  @override
  void dispose() {
    focus.dispose();
    editFocus.dispose();
    ctrl.dispose();
    super.dispose();
  }

  double get mult => (HardwareKeyboard.instance.isShiftPressed ? 10 : 1) * (HardwareKeyboard.instance.isAltPressed ? .1 : 1);

  void _edit() {
    if (!widget.enabled) {
      return;
    }
    ctrl.text = widget.mixed ? '' : _f(widget.value, widget.dec);
    ctrl.selection = TextSelection(baseOffset: 0, extentOffset: ctrl.text.length);
    setState(() => editing = true);
    WidgetsBinding.instance.addPostFrameCallback((_) => editFocus.requestFocus());
  }

  void _commit() {
    final x = double.tryParse(ctrl.text.trim());
    setState(() => editing = false);
    if (x != null) {
      widget.onType?.call(x);
    }
  }

  KeyEventResult _key(FocusNode n, KeyEvent e) {
    if (e is! KeyDownEvent && e is! KeyRepeatEvent || !widget.enabled) {
      return KeyEventResult.ignored;
    }
    final k = e.logicalKey;
    if (k == LogicalKeyboardKey.arrowRight || k == LogicalKeyboardKey.arrowUp) {
      widget.onDelta(widget.step * mult);
    } else if (k == LogicalKeyboardKey.arrowLeft || k == LogicalKeyboardKey.arrowDown) {
      widget.onDelta(-widget.step * mult);
    } else if (k == LogicalKeyboardKey.backspace || k == LogicalKeyboardKey.delete) {
      widget.onReset?.call();
    } else if (k == LogicalKeyboardKey.enter) {
      _edit();
    } else {
      return KeyEventResult.ignored;
    }
    return KeyEventResult.handled;
  }

  @override
  Widget build(BuildContext context) {
    final w = widget;
    final live = dragging || editing || w.typing || w.pose == _Pose.pressed;
    final border = !w.enabled ? N.g15 : (live ? Role.selected : (focus.hasFocus || w.pose == _Pose.focused ? N.g63 : (hover || w.pose == _Pose.hover ? N.g38 : N.g20)));
    final textStyle = T.value(w.enabled ? N.g95 : N.g63);
    Widget inner;
    if (editing || w.typing) {
      inner = EditableText(
        controller: ctrl,
        focusNode: editFocus,
        style: textStyle,
        cursorColor: N.g95,
        backgroundCursorColor: N.g26,
        selectionColor: Role.selected.withValues(alpha: .35),
        cursorWidth: 1,
        maxLines: 1,
        textAlign: TextAlign.right,
        onSubmitted: (_) => _commit(),
      );
    } else if (w.word != null || w.mixed) {
      inner = Row(mainAxisAlignment: MainAxisAlignment.end, children: [
        if (w.word == null) Padding(padding: const EdgeInsets.only(right: 4), child: Container(width: 5, height: 1, color: w.enabled ? N.g76 : N.g44)), // the mixed dash: two values differ
        Flexible(child: Text(w.word ?? 'Mixed', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(w.enabled ? N.g76 : N.g63).copyWith(fontSize: 11, fontStyle: FontStyle.italic))),
      ]);
    } else {
      // One unit rule: the unit is written when the cell is 96 px or wider, never in a narrow cell (the label says what it is). deg is a word at 10 px, like % and px.
      inner = LayoutBuilder(builder: (context, box) {
        final showUnit = w.unit.isNotEmpty && box.maxWidth + 16 >= 96;
        return Align(
          alignment: Alignment.centerRight,
          child: Text.rich(TextSpan(children: [
            TextSpan(text: _f(w.value, w.dec), style: textStyle),
            if (showUnit) TextSpan(text: ' ${_unit(w.unit)}', style: T.label(N.g76).copyWith(fontWeight: FontWeight.w500)),
          ]), maxLines: 1, overflow: TextOverflow.ellipsis),
        );
      });
    }
    return Focus(
      focusNode: focus,
      onKeyEvent: _key,
      child: Listener(
        onPointerSignal: (e) {
          if (w.wheel && w.enabled && e is PointerScrollEvent && (hover || focus.hasFocus)) {
            w.onDelta((e.scrollDelta.dy < 0 ? 1 : -1) * w.step * mult);
          }
        },
        child: MouseRegion(
          cursor: w.enabled ? SystemMouseCursors.resizeLeftRight : SystemMouseCursors.forbidden,
          onEnter: (_) => setState(() => hover = true),
          onExit: (_) => setState(() => hover = false),
          child: GestureDetector(
            behavior: HitTestBehavior.opaque,
            onTap: () {
              focus.requestFocus();
              _edit();
            },
            onDoubleTap: w.onReset,
            onHorizontalDragStart: w.enabled ? (_) => setState(() {
                  dragging = true;
                  focus.requestFocus();
                }) : null,
            onHorizontalDragUpdate: w.enabled ? (d) => w.onDelta(d.delta.dx * w.per * mult) : null,
            onHorizontalDragEnd: (_) => setState(() => dragging = false),
            onHorizontalDragCancel: () => setState(() => dragging = false),
            child: AnimatedContainer(
              duration: Mo.dur,
              curve: Mo.ease,
              height: 20,
              padding: const EdgeInsets.symmetric(horizontal: 8),
              decoration: BoxDecoration(color: w.enabled ? N.g07 : null, borderRadius: BorderRadius.circular(4), border: Border.all(color: border)),
              child: Stack(clipBehavior: Clip.none, children: [
                Positioned.fill(child: inner),
                // the grip: three 1 px ticks on the left say the cell can be dragged (hover brightens them)
                if (w.enabled && !editing && !w.typing && w.word == null && !w.mixed) Positioned(left: -5, top: 0, bottom: 0, child: Center(child: CustomPaint(size: const Size(5, 8), painter: _GripTicks(hover || dragging || w.pose == _Pose.hover ? N.g76 : N.g38)))),
                if (w.changed && w.enabled) Positioned(left: -4, right: -4, bottom: -1, height: 1, child: ColoredBox(color: Role.of(N.g63, Role.changed))),
              ]),
            ),
          ),
        ),
      ),
    );
  }
}

class _GripTicks extends CustomPainter {
  const _GripTicks(this.c);
  final Color c;
  @override
  void paint(Canvas cv, Size s) {
    final p = Paint()..color = c..strokeWidth = 1;
    for (var i = 0; i < 3; i++) {
      cv.drawLine(Offset(.5 + i * 2, 0), Offset(.5 + i * 2, s.height), p);
    }
  }

  @override
  bool shouldRepaint(_GripTicks o) => o.c != c;
}

/// A choice cell: the current word and a small chevron; a click steps to the next option (the real product opens a menu).
class _Pick extends StatelessWidget {
  const _Pick({required this.text, required this.onTap, this.pose, this.changed = false});
  final _Pose? pose;
  final bool changed;
  final String text;
  final VoidCallback onTap;
  @override
  Widget build(BuildContext context) => _Hv(
        pose: pose,
        onTap: onTap,
        builder: (_, h, d) => _Fcs(
          onTap: onTap,
          pose: pose,
          radius: 4,
          child: AnimatedContainer(
            duration: Mo.dur,
            curve: Mo.ease,
            height: 20,
            padding: const EdgeInsets.only(left: 8, right: 4),
            decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(4), border: Border.all(color: d ? Role.selected : (h ? N.g38 : N.g20))),
            child: Stack(clipBehavior: Clip.none, children: [
              Positioned.fill(
                child: Row(children: [
                  Expanded(child: Text(text, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.value(N.g95).copyWith(fontFamily: T.sans, fontSize: 11))),
                  const SizedBox(width: 4),
                  Text('›', style: T.name(h || d || pose == _Pose.focused ? N.g95 : N.g63)), // the one end mark of a choice cell, as in Inspector A
                ]),
              ),
              if (changed) Positioned(left: -4, right: -4, bottom: -1, height: 1, child: ColoredBox(color: Role.of(N.g63, Role.changed))),
            ]),
          ),
        ),
      );
}

class _Tog extends StatelessWidget {
  const _Tog(this.on, this.onChanged);
  final bool on;
  final ValueChanged<bool> onChanged;
  @override
  Widget build(BuildContext context) => Align(
        alignment: Alignment.centerRight,
        child: Row(mainAxisSize: MainAxisSize.min, children: [
          PillSwitch(on: on, onChanged: onChanged),
          const SizedBox(width: 8),
          SizedBox(width: 20, child: Text(on ? 'On' : 'Off', maxLines: 1, style: T.label(on ? N.g100 : N.g76))),
        ]),
      );
}

class _ColorCell extends StatelessWidget {
  const _ColorCell({required this.index, required this.onTap, this.pose, this.changed = false});
  final _Pose? pose;
  final bool changed;
  final int index;
  final VoidCallback onTap;
  @override
  Widget build(BuildContext context) => _Hv(
        pose: pose,
        onTap: onTap,
        builder: (_, h, d) => _Fcs(
          onTap: onTap,
          pose: pose,
          radius: 4,
          child: AnimatedContainer(
            duration: Mo.dur,
            curve: Mo.ease,
            height: 20,
            padding: const EdgeInsets.symmetric(horizontal: 4),
            decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(4), border: Border.all(color: d ? Role.selected : (h ? N.g38 : N.g20))),
            child: Stack(clipBehavior: Clip.none, children: [
              Positioned.fill(
                child: Row(children: [
                  Container(width: 24, height: 12, decoration: BoxDecoration(color: _swatch[index], borderRadius: BorderRadius.circular(2), border: Border.all(color: N.g26))),
                  const SizedBox(width: 8),
                  Expanded(child: Text(hexOf(_swatch[index]), maxLines: 1, overflow: TextOverflow.ellipsis, style: T.value(N.g91))),
                  const SizedBox(width: 4),
                  Text('›', style: T.name(h || d || pose == _Pose.focused ? N.g95 : N.g63)), // the one end mark of a choice cell, as in the choice cell
                ]),
              ),
              if (changed) Positioned(left: -4, right: -4, bottom: -1, height: 1, child: ColoredBox(color: Role.of(N.g63, Role.changed))),
            ]),
          ),
        ),
      );
}

InlineSpan _hl(String t, String q, TextStyle base) {
  final i = q.isEmpty ? -1 : t.toLowerCase().indexOf(q.toLowerCase());
  if (i < 0) {
    return TextSpan(text: t, style: base);
  }
  return TextSpan(style: base, children: [
    TextSpan(text: t.substring(0, i)),
    TextSpan(text: t.substring(i, i + q.length), style: TextStyle(color: T.hidden.value ? const Color(0x00000000) : N.g100, backgroundColor: N.g26, fontWeight: FontWeight.w700)),
    TextSpan(text: t.substring(i + q.length)),
  ]);
}

/// One property row. The label is brighter when the value was changed (D-I6); a number is scrubbed in place; below 220 px the label stacks over the value (one narrowing rule, X7).
class _PRow extends StatelessWidget {
  const _PRow(this.p, this.vals, {super.key, this.lang = 0, this.jp = false, this.trailH, this.query = '', this.tick, this.wheel = false, this.pose});
  final _Pose? pose;
  final _P p;
  final _Vals vals;
  final int lang;
  final bool jp, wheel;
  final Widget Function(bool hover)? trailH; // a slot at the row end (the star): the label x is the same in every row
  final String query;
  final Color? tick;

  Widget _control(bool changed) {
    switch (p.k) {
      case _K.num:
        return _Num(
          value: vals.of(p),
          dec: p.dec,
          unit: p.unit,
          per: p.per,
          step: p.step,
          wheel: wheel,
          changed: changed,
          onDelta: (d) => vals.set(p, vals.of(p) + d),
          onType: (x) => vals.set(p, x),
          onReset: () => vals.reset(p),
        );
      case _K.toggle:
        return _Tog(vals.of(p) > .5, (b) => vals.set(p, b ? 1 : 0));
      case _K.choice:
        final i = vals.of(p).round();
        return _Pick(text: p.opts[i], changed: changed, onTap: () => vals.set(p, ((i + 1) % p.opts.length).toDouble()));
      case _K.color:
        return _ColorCell(index: vals.of(p).round(), changed: changed, onTap: () => vals.set(p, ((vals.of(p).round() + 1) % _swatch.length).toDouble()));
    }
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: vals,
        builder: (context, _) {
          final changed = vals.changed(p);
          final nm = _nm(p, lang, jp: jp);
          // one line, ellipsis at the end; a changed label is brighter AND underlined (a toggle has no cell to underline)
          final label = _Mid(nm, T.label(changed ? Role.of(N.g95, Role.changed) : N.g63).copyWith(fontSize: 11, height: 1.2, decoration: changed && p.k == _K.toggle ? TextDecoration.underline : null, decorationColor: Role.of(N.g63, Role.changed)), query: query);
          return LayoutBuilder(builder: (context, box) {
              final narrow = box.maxWidth < 220;
              Widget body(bool h) => narrow
                  ? Column(crossAxisAlignment: CrossAxisAlignment.stretch, mainAxisSize: MainAxisSize.min, children: [
                      Row(children: [Expanded(child: label), _Reset(show: changed, onTap: () => vals.reset(p)), ?trailH?.call(h)]),
                      const SizedBox(height: 4),
                      _control(changed),
                    ])
                  : Row(crossAxisAlignment: CrossAxisAlignment.center, children: [
                      Expanded(flex: 6, child: label),
                      const SizedBox(width: 8),
                      Expanded(flex: 5, child: _control(changed)),
                      _Reset(show: changed, onTap: () => vals.reset(p)),
                      ?trailH?.call(h),
                    ]);
              return _Hv(
                pose: pose,
                cursor: SystemMouseCursors.basic,
                builder: (_, h, _) => _Gut(
                  tick: tick,
                  child: Container(
                    constraints: const BoxConstraints(minHeight: 28),
                    padding: const EdgeInsets.only(left: 12, right: 12, top: 4, bottom: 4),
                    color: h ? N.g15 : null,
                    child: body(h),
                  ),
                ),
              );
            });
        },
      );
}

/// The head of an effect inside the Inspector: name, how many properties (the one place the total is stated), a master On/Off with its word.
class _FxHead extends StatelessWidget {
  const _FxHead(this.name, this.sub, this.on, this.onChanged);
  final String name, sub;
  final bool on;
  final ValueChanged<bool> onChanged;
  @override
  Widget build(BuildContext context) => Container(
        height: 40,
        padding: const EdgeInsets.symmetric(horizontal: 12),
        decoration: const BoxDecoration(color: N.g13, border: Border(bottom: BorderSide(color: N.g07))),
        child: Row(children: [
          Expanded(
            child: Column(mainAxisAlignment: MainAxisAlignment.center, crossAxisAlignment: CrossAxisAlignment.start, children: [
              Text(name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(on ? N.g95 : N.g63)),
              if (sub.isNotEmpty) ...[
                const SizedBox(height: 4),
                Text(sub, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g76)),
              ],
            ]),
          ),
          PillSwitch(on: on, onChanged: onChanged),
          const SizedBox(width: 8),
          SizedBox(width: 20, child: Text(on ? 'On' : 'Off', maxLines: 1, style: T.label(on ? N.g100 : N.g76))),
        ]),
      );
}
