import 'dart:async';
import 'dart:math' as math;

import 'package:flutter/foundation.dart' show ValueListenable;
import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:lab_book/forms/number_paint.dart';
import 'package:lab_book/parts/controls.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart';
import 'package:lab_book/tokens.dart';
import '../foundations/text_hand.dart';

class NumFieldState extends State<NumField> {
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

  // (its own number at a new moment, told alone: the inspector is not built again for it)
  ValueListenable<int>? _watch;
  void _retold() {
    if (mounted) setState(() {});
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    final w = _x.doc.watch(widget.id), k = _x.doc.watch('key:${widget.id}');   // (and its key mark's news: showAt tells it, so the mark is drawn as the document says it, not a step behind)
    if (w != _watch) {
      _watch?.removeListener(_retold);
      _watch = w..addListener(_retold);
    }
    if (k != _keyWatch) {
      _keyWatch?.removeListener(_retold);
      _keyWatch = k..addListener(_retold);
    }
  }

  ValueListenable<int>? _keyWatch;

  @override
  void dispose() {
    _watch?.removeListener(_retold);
    _keyWatch?.removeListener(_retold);
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
    if (_moved) _x.doc.endGesture();
    setState(() => _drag = false);
  }

  double _w = 0;
  bool _inGlyph(Offset p) => p.dx > _w - Pop.glyph - Pop.tilePad && (widget.glyphTall || p.dy < Pop.glyph + Pop.tilePad);

  double _ladder(double dy) => dy < -24 ? 10 : (dy > 64 ? .01 : (dy > 24 ? .1 : 1));

  // (where the key mark is drawn, and what it says: a press near it keys the value there, or takes the key here away)
  Offset? _keyAt;
  KeyS? _keys;

  void _onDown(PointerDownEvent e) {
    if (!_on || _editing) return;
    if (_keyAt case final k? when e.buttons == kPrimaryButton && (e.localPosition - k).distance <= 9) {
      final now = _x.doc.keys[widget.id] ?? _keys;   // (as the document says it now: the field may not have been built since an undo)
      return _x.doc.setKey(widget.id, now == KeyS.at ? KeyS.anim : KeyS.at);   // (the row's KeyMark contract: inspector_rows_prop_row.dart)
    }
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
    if (!_moved) {
      _x.doc.beginGesture();   // (a scrub is one gesture: what it shows is settled when the hand lets go)
      setState(() => _moved = true);
    }
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
    if (mounted) setState(() {}); // (while a hand is on it the inspector is not built again: this field shows its own number)
  }

  void _onUp(PointerUpEvent e) {
    if (!_drag) return;
    _drag = false;
    if (_moved) {
      _x.doc.endGesture();
      setState(() {
        _moved = false;
        _rung = 1;
      });
      return;
    }
    final now = DateTime.now(), dbl = _lastUp != null && now.difference(_lastUp!) < dblParts;
    _lastUp = dbl ? null : now;
    final cfg = _x.cfg;
    if (dbl) {
      _pending?.cancel();
      if (cfg.resetDbl) {
        _reset();
      } else {
        _startEdit();
      }
    } else if (cfg.typesOn) {
      // (Ableton Live's: a click picks the field and no more; a digit or Enter starts the number: typesOn)
    } else if (cfg.resetDbl) {
      // contract: with double-click = reset, a single click types only after the double-click window, so no editor flashes (do-not-repeat 5).
      _pending = Timer(dblParts, () {
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
    if (_x.cfg.typesOn && RegExp(r'^[0-9.\-]$').hasMatch(e.character ?? '')) {
      _startEdit(e.character);
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  void _startEdit([String? typed]) {   // (typed: the number begun with that character)
    if (!mounted || _editing || !_on) return;
    final v = _x.doc.get(widget.id);
    _ctl.text = typed ?? (v == null ? '' : fmt(v, widget.decimals));
    _ctl.selection = typed != null ? TextSelection.collapsed(offset: _ctl.text.length) : TextSelection(baseOffset: 0, extentOffset: _ctl.text.length);
    setState(() {
      _editing = true;
      _err = false;
    });
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted || !_editing) return;
      _ef.requestFocus();
      _ctl.selection = typed != null ? TextSelection.collapsed(offset: _ctl.text.length) : TextSelection(baseOffset: 0, extentOffset: _ctl.text.length);
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
        final row = pop ? RowInfo.of(context) : null;
        final showUnit = w >= wUnit && !narrow && row?.unit == null, tight = w < wTight;
        final t = (_bounded && v != null) ? ((v - widget.min!) / (widget.max! - widget.min!)).clamp(0.0, 1.0) : 0.0;
        String unit = widget.unit ?? '';
        if (widget.turns && v != null && v.abs() >= 360 && w >= 120) unit = '° ${(v / 360).truncate()}x ${fmt(v.remainder(360), 0)}';
        final word = v == 0 && widget.zeroWord != null ? widget.zeroWord! : null;
        Widget valueText() {
          if (v == null) return Text(Role.grey ? 'Mixed' : '— Mixed', style: T.label(Role.of(Grey.g76, Role.info)));
          if (word != null) return Text(word, style: T.label(on ? Grey.g76 : Grey.g56));
          return FittedBox( // (a number wider than its field is drawn smaller, never cut)
            fit: BoxFit.scaleDown,
            alignment: Alignment.centerLeft,
            child: Text.rich(
            TextSpan(
              children: [
                TextSpan(
                  text: fmt(v, widget.decimals, tight: tight), // (its places fixed, as Push shows them: a number does not jump in width as it moves)
                  style: widget.caption != null
                      ? blockValue(on ? Grey.g95 : Grey.g56)
                      : pop
                      ? T.value(on ? Grey.g95 : Grey.g56).copyWith(fontWeight: FontWeight.w600)
                      : T.value(on ? Grey.g95 : Grey.g56),
                ),
                if (showUnit && unit.isNotEmpty) TextSpan(text: pop ? '\u2009$unit' : ' $unit', style: T.label(pop ? Grey.g44 : Grey.g56)),
              ],
            ),
            maxLines: 1,
            softWrap: false,
          ),
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
        // contract: animated = a dim dot, keyed at the playhead = a lit dot (the automation blip, not a column). A value
        // that takes keys shows its mark always, hollow when it has none, and a press on it keys it there (Alight Motion's
        // diamond beside each value, After Effects' and Premiere Pro's stopwatch: golf_noise.md (59)).
        final ks = pop ? (x.doc.keys[widget.id] ?? row?.keys) : null;
        _keyAt = ks != null && (ks != KeyS.off || x.doc.keys.containsKey(widget.id)) ? (block ? Offset(w - Pop.tilePad - 3, Pop.tile - 10) : Offset(w - 6, 6)) : null;
        _keys = ks;
        final editor = _editing
            ? TextHand(field: (k) => EditableText(key: k, rendererIgnoresPointer: true,
                controller: _ctl,
                focusNode: _ef,
                style: block ? blockValue(Grey.g95) : T.value(Grey.g95),
                cursorColor: Grey.g95,
                backgroundCursorColor: Grey.g20,
                selectionColor: Role.selected.withValues(alpha: .4),
                textAlign: block ? TextAlign.left : (pop ? TextAlign.center : TextAlign.right),
                maxLines: 1,
                selectAllOnFocus: false,   // (_startEdit says the selection: all of the number when begun by a click or Return; the caret after a number begun by typing it, which Flutter's own select-all on a desktop's focus took, so the next key replaced the first: golf P05, "230" came out 0)
                keyboardType: const TextInputType.numberWithOptions(decimal: true, signed: true),
                inputFormatters: [FilteringTextInputFormatter.allow(RegExp(r'[0-9.\-+eE]'))],
                onChanged: (_) {
                  if (_err) setState(() => _err = false);
                },
              ))
            : null;
        final body = paintNumber(
          context,
          NumberSpec(
            value: valueText(),
            editor: editor,
            lead: lead ?? (_err ? const ErrMark() : null),
            glyph: widget.glyph,
            caption: widget.caption,
            width: w,
            height: block ? Pop.tile : (pop ? Pop.cell : cellH),
            fillColor: fillC,
            borderColor: picked ? Pop.accentInk.withValues(alpha: .7) : (pop && !(_focus || _editing || live) ? const Color(0x00000000) : edge),
            rangeColor: _bounded && widget.bipolar ? (on ? Grey.g56 : Grey.g44) : (look == Look.glow ? Role.selected : (on ? Grey.g56 : Grey.g44)),
            fill: _bounded && v != null ? t : null,
            zeroAt: _zeroAt,
            bipolar: _bounded && widget.bipolar,
            block: block,
            pop: pop,
            editing: _editing,
            glyphTall: widget.glyphTall,
            on: on,
            hot: _hover,
            live: live,
            axis: widget.axis,
            keyDot: _keyAt == null ? null : (ks == KeyS.at ? Pop.keyDot : (ks == KeyS.anim ? Pop.keyDot.withValues(alpha: .5) : Grey.g56)),
            keyEmpty: ks == KeyS.off,
            pad: Pop.tilePad,
            glyphBox: Pop.glyph,
          ),
        );
        return Focus(
          focusNode: _node,
          onKeyEvent: _key,
          onFocusChange: (f) => setState(() => _focus = f),
          child: RawGestureDetector(
            // (a press that scrubs is the field's at once: a scroll round the inspector would take a drag that goes up or
            //  down too and carry the field away under the hand; a press on a glyph that owns it, or while typing, is not)
            gestures: {ScrubClaim: GestureRecognizerFactoryWithHandlers<ScrubClaim>(() => ScrubClaim(), (r) => r.takes = (e) => _on && !_editing && !(widget.glyph != null && widget.glyphOwnsPointer && _inGlyph(e.localPosition)))},
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
          ),
        );
      },
    );
  }
}
