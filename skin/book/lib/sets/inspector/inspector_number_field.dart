part of 'inspector_parts.dart';

TextStyle _blockValue(Color c) => T.value(c).copyWith(fontSize: 14, fontWeight: FontWeight.w600, height: 1.1);

class NumField extends StatefulWidget {
  const NumField({
    super.key,
    required this.id,
    this.label,
    this.unit = '',
    this.decimals = 0,
    this.min,
    this.max,
    this.step = 1,
    this.perPx = 1,
    this.zeroWord,
    this.enabled = true,
    this.axis,
    this.turns = false,
    this.onSet,
    this.onReset,
    this.bipolar = false,
    this.caption,
    this.glyph,
    this.glyphOwnsPointer = false,
    this.glyphTall = false,
  });
  final String id;
  final String? label, unit, zeroWord;
  final int decimals;
  final double? min, max;
  final double step, perPx;
  final bool enabled, turns;
  final Color? axis;
  final ValueChanged<double>? onSet;
  final VoidCallback? onReset;

  /// A range around zero fills out from the middle.
  final bool bipolar;

  /// Block form: [caption] on top, the value large under it, [glyph] (a picture of the value) in the top-right corner.
  /// With [glyphOwnsPointer] a press on the glyph is the glyph's (a dial, a die), not a scrub.
  /// contract: every number in the Inspector scrubs sideways, whatever it means (a Y, a height); only a glyph may move otherwise.
  final String? caption;
  final Widget? glyph;
  final bool glyphOwnsPointer;

  /// The glyph runs the block's full height on the right (stacked buttons) instead of sitting in the corner.
  final bool glyphTall;
  @override
  State<NumField> createState() => _NumFieldState();
}

class _NumFieldState extends State<NumField> {
  final _node = FocusNode(debugLabel: 'num');
  late final FocusNode _ef = FocusNode(debugLabel: 'num-edit', onKeyEvent: _editKey);
  final _ctl = TextEditingController();
  bool _editing = false, _err = false, _hover = false, _focus = false, _drag = false, _moved = false, _cancelled = false;
  Offset _down = Offset.zero;
  double _start = 0, _acc = 0, _sum = 0, _rung = 1;
  DateTime? _lastUp;
  int _gen = 0;
  Timer? _pending;

  Ctx get _x => Ctx.read(context);
  bool get _on => widget.enabled && !_x.cfg.locked && !(context.getInheritedWidgetOfExactType<RowOff>()?.off ?? false);
  bool get _bounded => widget.min != null && widget.max != null;
  double get _zeroAt => _bounded ? ((0 - widget.min!) / (widget.max! - widget.min!)).clamp(0.0, 1.0) : 0;

  @override
  void initState() {
    super.initState();
    _ef.addListener(_editFocus);
  }

  @override
  void dispose() {
    _pending?.cancel();
    _ef.dispose();
    _node.dispose();
    _ctl.dispose();
    super.dispose();
  }

  double _clamp(double v) => math.min(widget.max ?? double.infinity, math.max(widget.min ?? double.negativeInfinity, v));
  double _tidy(double v) => double.parse(v.toStringAsFixed(6));

  // contract: with several fields picked, typing sets them all; a scrub or nudge moves them all by the same amount.
  void _apply(double v, {bool same = false}) {
    final c = _clamp(v);
    final doc = _x.doc;
    if (doc.isPicked(widget.id)) {
      final d = c - (doc.get(widget.id) ?? c);
      doc.setMany({for (final id in doc.picked) id: id == widget.id || same ? c : (doc.get(id) ?? 0) + d});
      return;
    }
    final f = widget.onSet;
    if (f != null) {
      f(c);
    } else {
      _x.doc.set(widget.id, c);
    }
  }

  void _reset() {
    final f = widget.onReset;
    if (f != null) {
      f();
    } else {
      _x.doc.resetIds([widget.id]);
    }
  }

  // contract: Esc during a drag puts the value back and ignores the rest of the drag; pointer cancel does the same (C3).
  void _abort() {
    final cur = _x.doc.get(widget.id);
    if (cur == null) {
      _x.doc.delta(widget.id, -_sum);
    } else {
      _apply(_start);
    }
    _cancelled = true;
    _sum = 0;
    setState(() => _drag = false);
  }

  double _w = 0;
  bool _inGlyph(Offset p) => p.dx > _w - Pop.glyph - Pop.tilePad && (widget.glyphTall || p.dy < Pop.glyph + Pop.tilePad);

  double _ladder(double dy) => dy < -24 ? 10 : (dy > 64 ? .01 : (dy > 24 ? .1 : 1));

  void _onDown(PointerDownEvent e) {
    if (!_on || _editing) return;
    if (widget.glyph != null && widget.glyphOwnsPointer && _inGlyph(e.localPosition)) return;
    if (e.buttons != kPrimaryButton) return;
    final doc = _x.doc;
    if (HardwareKeyboard.instance.isShiftPressed) {
      doc.pick(widget.id, span: true);
      _node.requestFocus();
      return;
    }
    if (!doc.isPicked(widget.id)) doc.pick(widget.id);
    _gen = doc.pickGen;
    _node.requestFocus();
    _pending?.cancel();
    _down = e.position;
    _moved = false;
    _cancelled = false;
    _sum = 0;
    _rung = 1;
    _start = _x.doc.get(widget.id) ?? 0;
    _acc = _start;
    _drag = true;
  }

  void _onMove(PointerMoveEvent e) {
    if (!_drag || _cancelled) return;
    if (!_moved && (e.position - _down).distance < 4) return;
    final cfg = _x.cfg;
    if (!_moved) setState(() => _moved = true);
    var m = cfg.mult();
    if (cfg.maybe) {
      final r = _ladder(e.position.dy - _down.dy);
      if (r != _rung) setState(() => _rung = r);
      m *= _rung;
    }
    final d = e.delta.dx * (cfg.rightUp ? 1 : -1) * widget.perPx * m;
    if (_x.doc.get(widget.id) == null) {
      _sum += d;
      _x.doc.delta(widget.id, d);
    } else {
      _acc = _clamp(_acc + d);
      _apply(_acc);
    }
  }

  void _onUp(PointerUpEvent e) {
    if (!_drag) return;
    _drag = false;
    if (_moved) {
      setState(() {
        _moved = false;
        _rung = 1;
      });
      return;
    }
    final now = DateTime.now(), dbl = _lastUp != null && now.difference(_lastUp!) < _dbl;
    _lastUp = dbl ? null : now;
    final cfg = _x.cfg;
    if (dbl) {
      _pending?.cancel();
      if (cfg.resetDbl) {
        _reset();
      } else {
        _startEdit();
      }
    } else if (cfg.resetDbl) {
      // contract: with double-click = reset, a single click types only after the double-click window, so no editor flashes (do-not-repeat 5).
      _pending = Timer(_dbl, () {
        if (_x.doc.pickGen == _gen) _startEdit();
      });
    }
  }

  void _onCancel(PointerCancelEvent e) {
    if (_drag && _moved) _abort();
    _drag = false;
    _moved = false;
  }

  // contract: hover and slide sideways, every number the same way: slide right = more, left = less, at the drag's rate (perPx, Shift/Alt per the
  // panel's modifier rule), so a slide and a scrub of the same length move a value the same amount. An up/down slide is not claimed, so it scrolls
  // the panel. No button needs holding (do-not-repeat 4).
  void _onSignal(PointerSignalEvent e) {
    if (e is! PointerScrollEvent || !_on || _editing) return;
    final d = e.scrollDelta;
    if (d.dx.abs() <= d.dy.abs()) return;
    GestureBinding.instance.pointerSignalResolver.register(e, (ev) => _slide((ev as PointerScrollEvent).scrollDelta.dx));
  }

  void _onPan(PointerPanZoomUpdateEvent e) {
    if (!_on || _editing) return;
    final d = e.localPanDelta;
    if (d.dx.abs() > d.dy.abs()) _slide(d.dx);
  }

  void _slide(double right) {
    final d = right * widget.perPx * _x.cfg.mult();
    final cur = _x.doc.get(widget.id);
    if (cur == null) {
      _x.doc.delta(widget.id, d);
    } else {
      _apply(_tidy(cur + d));
    }
  }

  void _nudge(int dir) {
    final d = dir * widget.step * _x.cfg.mult();
    final cur = _x.doc.get(widget.id);
    if (cur == null) {
      _x.doc.delta(widget.id, d);
    } else {
      _apply(_tidy(cur + d));
    }
  }

  KeyEventResult _key(FocusNode n, KeyEvent e) {
    if (_editing || !_on) return KeyEventResult.ignored;
    if (e is! KeyDownEvent && e is! KeyRepeatEvent) return KeyEventResult.ignored;
    final k = e.logicalKey;
    if (k == LogicalKeyboardKey.escape && _drag && _moved) {
      _abort();
      return KeyEventResult.handled;
    }
    if (k == LogicalKeyboardKey.escape && _x.doc.picked.length > 1) {
      _x.doc.unpick();
      return KeyEventResult.handled;
    }
    if (k == LogicalKeyboardKey.arrowUp || k == LogicalKeyboardKey.arrowRight) {
      _nudge(1);
      return KeyEventResult.handled;
    }
    if (k == LogicalKeyboardKey.arrowDown || k == LogicalKeyboardKey.arrowLeft) {
      _nudge(-1);
      return KeyEventResult.handled;
    }
    if (k == LogicalKeyboardKey.enter || k == LogicalKeyboardKey.numpadEnter) {
      _startEdit();
      return KeyEventResult.handled;
    }
    if (_x.cfg.resetDel && (k == LogicalKeyboardKey.delete || k == LogicalKeyboardKey.backspace)) {
      _reset();
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  void _startEdit() {
    if (!mounted || _editing || !_on) return;
    final v = _x.doc.get(widget.id);
    _ctl.text = v == null ? '' : fmt(v, widget.decimals);
    _ctl.selection = TextSelection(baseOffset: 0, extentOffset: _ctl.text.length);
    setState(() {
      _editing = true;
      _err = false;
    });
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted || !_editing) return;
      _ef.requestFocus();
      _ctl.selection = TextSelection(baseOffset: 0, extentOffset: _ctl.text.length);
    });
  }

  KeyEventResult _editKey(FocusNode n, KeyEvent e) {
    if (e is! KeyDownEvent) return KeyEventResult.ignored;
    if (e.logicalKey == LogicalKeyboardKey.escape) {
      // contract: Esc while typing discards and restores the old value (C3).
      _close();
      return KeyEventResult.handled;
    }
    if (e.logicalKey == LogicalKeyboardKey.enter || e.logicalKey == LogicalKeyboardKey.numpadEnter) {
      _commit();
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  void _commit() {
    final t = double.tryParse(_ctl.text.trim());
    if (t == null) {
      // contract: invalid text keeps the field open and says "Number required" (C2).
      setState(() => _err = true);
      return;
    }
    _apply(t, same: true);
    _close();
  }

  void _close({bool refocus = true}) {
    if (!_editing) return;
    setState(() => _editing = false);
    if (refocus) _node.requestFocus();
  }

  void _editFocus() {
    if (_editing && !_ef.hasFocus) {
      // contract: leaving the field applies a valid number and drops an invalid one.
      final t = double.tryParse(_ctl.text.trim());
      if (t != null) _apply(t, same: true);
      _close(refocus: false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), cfg = x.cfg, ed = Ed(cfg.ed), look = cfg.look;
    final v = x.doc.get(widget.id), on = _on, narrow = RowOff.maybe(context)?.narrow ?? false;
    final live = _drag && _moved;
    final group = widget.id.split('.').first, linked = on && RowMark.has(context, group);
    final Color idle = switch (look) {
      Look.concept || Look.glow => ed.rule,
      Look.quiet => cfg.ed == Editorial.a ? const Color(0x00000000) : ed.rule,
    };
    final Color edge = live
        ? C.mode.withValues(alpha: .8)
        : (_focus || _editing ? (look == Look.glow ? C.mode.withValues(alpha: .6) : Grey.g63) : (_hover && on ? Grey.g38 : idle));
    final pop = Pop.on(context);
    final picked = pop && x.doc.isPicked(widget.id);
    final fillC = pop
        ? (!on ? Grey.g10 : (picked ? Color.alphaBlend(Pop.accent.withValues(alpha: .14), Pop.well) : (_hover || live ? Grey.g20 : Pop.well)))
        : (!on ? Grey.g10 : (look == Look.quiet ? Grey.g13 : Grey.g07));
    return LayoutBuilder(
      builder: (context, box) {
        final w = box.maxWidth;
        _w = w;
        final row = pop ? _RowInfo.of(context) : null;
        final showUnit = w >= _wUnit && !narrow && row?.unit == null, tight = w < _wTight;
        final t = (_bounded && v != null) ? ((v - widget.min!) / (widget.max! - widget.min!)).clamp(0.0, 1.0) : 0.0;
        String unit = widget.unit ?? '';
        if (widget.turns && v != null && v.abs() >= 360 && w >= 120) unit = '° ${(v / 360).truncate()}x ${fmt(v.remainder(360), 0)}';
        final word = v == 0 && widget.zeroWord != null ? widget.zeroWord! : null;
        Widget valueText() {
          if (v == null) return Text(Role.grey ? 'Mixed' : '— Mixed', style: T.label(Role.of(Grey.g76, Role.info)));
          if (word != null) return Text(word, style: T.label(on ? Grey.g76 : Grey.g56));
          return Text.rich(
            TextSpan(
              children: [
                TextSpan(
                  text: pop ? trimZeros(fmt(v, widget.decimals, tight: tight)) : fmt(v, widget.decimals, tight: tight),
                  style: widget.caption != null
                      ? _blockValue(on ? Grey.g95 : Grey.g56)
                      : pop
                      ? T.value(on ? Grey.g95 : Grey.g56).copyWith(fontWeight: FontWeight.w600)
                      : T.value(on ? Grey.g95 : Grey.g56),
                ),
                if (showUnit && unit.isNotEmpty) TextSpan(text: pop ? '\u2009$unit' : ' $unit', style: T.label(pop ? Grey.g44 : Grey.g56)),
              ],
            ),
            maxLines: 1,
            softWrap: false,
            overflow: TextOverflow.clip,
          );
        }

        final lead = _err && w >= 150
            ? Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  const ErrMark(gap: 4),
                  Text('Number required', style: T.label(Role.error)),
                ],
              )
            : (live && cfg.maybe && _rung != 1
                  ? Text('x${_rung == .01 ? '.01' : (_rung == .1 ? '.1' : '10')}', style: T.micro(Grey.g95))
                  : (widget.label != null && !(pop && widget.axis != null && AxisColumns.on(context))
                        ? Text(
                            ed.lab(widget.label!),
                            maxLines: 1,
                            softWrap: false,
                            style: pop && widget.axis != null
                                ? T.label(on ? Color.lerp(widget.axis!, Grey.g100, .3)! : Grey.g56).copyWith(fontWeight: FontWeight.w700)
                                : ed.labStyle(on ? Grey.g63 : Grey.g56),
                          )
                        : null));
        final block = widget.caption != null;
        final body = Container(
          height: block ? Pop.tile : (pop ? Pop.cell : _cellH),
          decoration: BoxDecoration(
            color: fillC,
            borderRadius: BorderRadius.circular(pop ? 6 : 4),
            border: Border.all(color: picked ? Pop.accentInk.withValues(alpha: .7) : (pop && !(_focus || _editing || live) ? const Color(0x00000000) : edge)),
          ),
          child: ClipRRect(
            borderRadius: BorderRadius.circular(pop ? 5 : 3),
            child: Stack(
              children: [
                // the range tick sits on the field's bottom edge, never behind the text
                if (_bounded && widget.bipolar)
                  Positioned(
                    left: (w - 2) * math.min(t, _zeroAt),
                    bottom: 0,
                    height: 2,
                    width: math.max(1, (w - 2) * (t - _zeroAt).abs()),
                    child: ColoredBox(color: on ? Grey.g56 : Grey.g44),
                  )
                else if (_bounded)
                  Positioned(
                    left: 0,
                    bottom: 0,
                    height: 2,
                    width: math.max(0, (w - 2) * t),
                    child: ColoredBox(color: look == Look.glow ? Role.selected : (on ? Grey.g56 : Grey.g44)),
                  ),
                // contract: animated = a dim dot, keyed at the playhead = a lit dot (the automation blip, not a column).
                if (row != null && row.keys != KeyS.off)
                  Positioned(
                    top: block ? null : 3,
                    bottom: block ? 7 : null,
                    right: block ? Pop.tilePad : 3,
                    child: Container(
                      width: 4,
                      height: 4,
                      decoration: BoxDecoration(color: row.keys == KeyS.at ? Pop.keyDot : Pop.keyDot.withValues(alpha: .4), shape: BoxShape.circle),
                    ),
                  ),
                if (widget.axis != null && !pop) Positioned(left: 0, top: 5, bottom: 5, width: 2, child: ColoredBox(color: widget.axis!)),
                if (widget.glyph != null)
                  Positioned(
                    right: widget.glyphTall ? 4 : Pop.tilePad - 3,
                    top: 4,
                    width: Pop.glyph,
                    height: widget.glyphTall ? null : Pop.glyph,
                    bottom: widget.glyphTall ? 4 : null,
                    child: Center(child: widget.glyph),
                  ),
                if (block)
                  Positioned.fill(
                    child: Padding(
                      padding: EdgeInsets.fromLTRB(Pop.tilePad, 6, widget.glyph != null ? Pop.glyph + Pop.tilePad : Pop.tilePad, 7),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        mainAxisAlignment: MainAxisAlignment.spaceBetween,
                        children: [
                          Text(widget.caption!, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.micro(on ? Grey.g63 : Grey.g44).copyWith(height: 1.1)),
                          if (_editing)
                            SizedBox(
                              height: 18,
                              child: EditableText(
                                controller: _ctl,
                                focusNode: _ef,
                                style: _blockValue(Grey.g95),
                                cursorColor: Grey.g95,
                                backgroundCursorColor: Grey.g20,
                                selectionColor: Role.selected.withValues(alpha: .4),
                                maxLines: 1,
                                keyboardType: const TextInputType.numberWithOptions(decimal: true, signed: true),
                                inputFormatters: [FilteringTextInputFormatter.allow(RegExp(r'[0-9.\-+eE]'))],
                                onChanged: (_) {
                                  if (_err) setState(() => _err = false);
                                },
                              ),
                            )
                          else
                            SizedBox(
                              height: 18,
                              child: Align(alignment: Alignment.centerLeft, child: valueText()),
                            ),
                        ],
                      ),
                    ),
                  )
                else
                  Positioned.fill(
                    child: Padding(
                      padding: EdgeInsets.only(left: widget.axis != null && !pop ? 8 : 7, right: 6),
                      child: Row(
                        crossAxisAlignment: CrossAxisAlignment.center,
                        children: [
                          if (lead != null) Flexible(child: lead) else if (_err) const ErrMark(),
                          if (!_editing)
                            Expanded(
                              flex: 4,
                              child: Align(alignment: pop ? Alignment.center : Alignment.centerRight, child: valueText()),
                            )
                          else
                            const Spacer(),
                          if (_editing)
                            SizedBox(
                              width: math.min(72, math.max(28, w - 36)),
                              child: EditableText(
                                controller: _ctl,
                                focusNode: _ef,
                                style: T.value(Grey.g95),
                                cursorColor: Grey.g95,
                                backgroundCursorColor: Grey.g20,
                                selectionColor: Role.selected.withValues(alpha: .4),
                                textAlign: pop ? TextAlign.center : TextAlign.right,
                                maxLines: 1,
                                keyboardType: const TextInputType.numberWithOptions(decimal: true, signed: true),
                                inputFormatters: [FilteringTextInputFormatter.allow(RegExp(r'[0-9.\-+eE]'))],
                                onChanged: (_) {
                                  if (_err) setState(() => _err = false);
                                },
                              ),
                            ),
                        ],
                      ),
                    ),
                  ),
              ],
            ),
          ),
        );
        return Focus(
          focusNode: _node,
          onKeyEvent: _key,
          onFocusChange: (f) => setState(() => _focus = f),
          child: Listener(
            onPointerDown: _onDown,
            onPointerMove: _onMove,
            onPointerUp: _onUp,
            onPointerCancel: _onCancel,
            onPointerSignal: _onSignal,
            onPointerPanZoomUpdate: _onPan,
            child: MouseRegion(
              cursor: !on ? SystemMouseCursors.basic : (_editing ? SystemMouseCursors.text : SystemMouseCursors.resizeLeftRight),
              onEnter: (_) {
                setState(() => _hover = true);
                if (linked) x.doc.setDirectHover(group);
              },
              onExit: (_) {
                setState(() => _hover = false);
                if (linked && x.doc.directHover == group) x.doc.setDirectHover(null);
              },
              child: body,
            ),
          ),
        );
      },
    );
  }
}
