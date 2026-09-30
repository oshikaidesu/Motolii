// The generic Toys. A handful of primitives; meaning composes them, it does not add widgets.
// Shared grammar: drag = manipulate, Shift = fine, click = exact input, double click = default,
// arrow keys nudge, Delete resets. Hard min/max clamp silently and never draw a finite bar.
import 'dart:async';
import 'dart:convert';
import 'dart:math' as math;
import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import '../browser/parts.dart';
import '../desks/parts.dart' show kYellow, kBlue, kViolet, kPink;
import 'rows.dart';
import 'slot.dart';
export 'slot.dart';
import '../theme/surface.dart';
import 'tones.dart';
import '../theme/neutral.dart';

const _readout = N.g82; // the number is a readout: present, not loud
const _dark = N.g10;

/// How a number answers a drag: value per pixel and the arrow-key step. The skin's business, not the number's.
extension SlotScrub on Slot {
  double get magnitude => math.max(math.max((def ?? 0).abs(), value.abs()), 1);

  /// Value per pixel of drag. A real reach maps to the tile; an open value scales with its own size; an angle is a degree a pixel.
  double get rate {
    final r = row;
    if (r['speed'] is num) return (r['speed'] as num).toDouble(); // a row can say how fast one pixel of drag moves it
    if (isAngle) return 1;
    final base = tight(r) ? span(r) / 240 : magnitude * .01;
    return whole ? math.max(base, .16) : base;
  }

  double get step => isAngle ? 1 : (whole ? math.max(1, (tight(row) ? span(row) / 100 : magnitude * .05).roundToDouble()) : (tight(row) ? span(row) / 100 : magnitude * .05));
}

bool get _shift => HardwareKeyboard.instance.isShiftPressed;

/// The scrubbable number. Endless by default; a tight declared range earns a soft fill behind the number.
class ValueToy extends StatefulWidget {
  const ValueToy(this.slot, {super.key, this.tag, this.hero = false, this.keyName, this.tone = kBlue, this.focusNode, this.showUnit = true, this.decimals});
  final int? decimals;
  final Color tone;
  final FocusNode? focusNode;
  final bool showUnit; // three numbers across have no room for a unit; the row's glyph and the gizmo say what they are
  final Slot slot;
  final String? tag;
  final bool hero;
  final String? keyName;
  @override
  State<ValueToy> createState() => _ValueToyState();
}

class _ValueToyState extends State<ValueToy> {
  bool dragging = false, editing = false, _cancel = false, _moved = false, hover = false, _scrubbed = false, _aborted = false;
  double _v = 0;
  Offset _down = Offset.zero;
  DateTime? _last;
  late final FocusNode focus = widget.focusNode ?? FocusNode(debugLabel: 'toy');
  final editFocus = FocusNode(debugLabel: 'toy-edit');
  final ctl = TextEditingController();

  @override
  void initState() {
    super.initState();
    focus.addListener(() => setState(() {}));
    editFocus.addListener(() {
      if (!editFocus.hasFocus && editing) _submit(ctl.text, blur: true);
    });
  }

  @override
  void dispose() {
    if (widget.focusNode == null) focus.dispose();
    editFocus.dispose();
    ctl.dispose();
    _settle?.cancel();
    super.dispose();
  }

  bool get frozen => widget.slot.store.frozen;

  void _startEdit() {
    final v = widget.slot.value;
    final shown = v * widget.slot.displayScale;
    ctl.text = widget.slot.whole ? '${shown.round()}' : (shown == shown.roundToDouble() ? shown.toStringAsFixed(1) : '$shown');
    ctl.selection = TextSelection(baseOffset: 0, extentOffset: ctl.text.length);
    _cancel = false;
    _bad = false;
    _subject = widget.slot.store.subject;
    setState(() => editing = true);
  }

  Object? _subject;
  bool _bad = false; // the typed text is not a number

  void _submit(String t, {bool blur = false}) {
    if (_cancel) { _cancel = false; return; }
    final v = double.tryParse(t.trim());
    // Enter on something that is not a number keeps the field open and says so (Classic IN-022); leaving drops it
    if (v == null && !blur) {
      if (mounted) setState(() => _bad = true);
      editFocus.requestFocus();
      return;
    }
    _bad = false;
    if (mounted) setState(() => editing = false);
    if (widget.slot.store.subject != _subject) return; // typed for a layer that is no longer the one shown
    if (v != null && !frozen) {
      widget.slot.typed(v / widget.slot.displayScale);
    }
  }

  // A wheel step shows at once and is written when the wheel rests (one undo for a run of steps).
  Timer? _settle;
  bool _stepping = false;
  void _wheelStep(int dir) {
    final s = widget.slot;
    if (!_stepping) { _v = s.value; _stepping = true; }
    final by = s.step * (_shift ? .1 : 1);
    _v += dir * (s.whole ? math.max(1, by.roundToDouble()) : by);
    s.preview(s.quantize(_v, fine: _shift));
    _settle?.cancel();
    _settle = Timer(const Duration(milliseconds: 300), () {
      _stepping = false;
      if (mounted) s.commit();
    });
  }

  // A sideways two-finger pan scrubs like a drag; a mostly vertical one is left to the panel's scroll.
  bool? _panning;

  void _nudge(int dir) {
    final s = widget.slot;
    final by = s.step * (_shift ? .1 : 1);
    s.preview(s.quantize(s.value + dir * (s.whole ? math.max(1, by.roundToDouble()) : by), fine: _shift));
    s.commit();
  }

  @override
  Widget build(BuildContext context) {
    final s = widget.slot;
    final row = s.row;
    final track = s.axis == null && tight(row);
    final frac = track ? ((s.value - (row['min'] as num)) / span(row)).clamp(0.0, 1.0) : 0.0;
    final h = widget.hero ? Surface.controlHero : Surface.control;
    final ink = frozen ? N.g33 : _readout;
    final unit = widget.showUnit ? s.unit : null;
    // colour mass follows importance, quietly: a thin edge; a touched value a wider edge and a tint; a hero a faint tint
    final active = dragging || hover || focus.hasFocus || editing;
    final tone = frozen ? dimTone(widget.tone) : widget.tone;
    final edge = active ? 4.0 : 3.0;
    final tint = widget.hero ? .07 : (active ? .09 : 0.0);
    return Focus(
      focusNode: focus,
      onKeyEvent: (_, e) {
        if (e is! KeyDownEvent) return KeyEventResult.ignored;
        final k = e.logicalKey;
        // Esc during a drag puts the value back; the rest of the drag does nothing
        if (dragging && k == LogicalKeyboardKey.escape) { _aborted = true; s.cancel(); return KeyEventResult.handled; }
        // Esc leaves exact input without applying it
        if (editing && k == LogicalKeyboardKey.escape) { _cancel = true; setState(() => editing = false); focus.requestFocus(); return KeyEventResult.handled; }
        if (editing || frozen) return KeyEventResult.ignored;
        if (k == LogicalKeyboardKey.arrowUp || k == LogicalKeyboardKey.arrowRight) { _nudge(1); return KeyEventResult.handled; }
        if (k == LogicalKeyboardKey.arrowDown || k == LogicalKeyboardKey.arrowLeft) { _nudge(-1); return KeyEventResult.handled; }
        if (k == LogicalKeyboardKey.enter) { _startEdit(); return KeyEventResult.handled; }
        if (k == LogicalKeyboardKey.delete || k == LogicalKeyboardKey.backspace) { s.reset(); return KeyEventResult.handled; }
        if (k == LogicalKeyboardKey.escape) { focus.unfocus(); return KeyEventResult.handled; }
        return KeyEventResult.ignored;
      },
      child: MouseRegion(
        onEnter: (_) => setState(() => hover = true),
        onExit: (_) => setState(() => hover = false),
        child: Listener(
        onPointerSignal: (e) {
          if (e is! PointerScrollEvent || frozen || editing) return;
          final dx = e.scrollDelta.dx, dy = e.scrollDelta.dy;
          // a sideways wheel steps the value; a wheel turned with the button held steps it too (Classic IN-019/020)
          final dir = dx.abs() > dy.abs() && dx != 0
              ? (dx > 0 ? 1 : -1)
              : ((e.buttons & kPrimaryButton) != 0 && dy != 0 ? (dy < 0 ? 1 : -1) : 0);
          if (dir == 0) return;
          GestureBinding.instance.pointerSignalResolver.register(e, (_) => _wheelStep(dir));
        },
        onPointerPanZoomStart: frozen ? null : (_) => _panning = null,
        onPointerPanZoomUpdate: frozen ? null : (e) {
          final d = e.panDelta;
          _panning ??= d.distance < 2 ? null : d.dx.abs() > d.dy.abs();
          if (_panning != true || editing) return;
          if (!dragging) { _v = widget.slot.value; setState(() => dragging = true); }
          _v += d.dx * widget.slot.rate * (_shift ? .1 : 1);
          widget.slot.preview(widget.slot.quantize(_v, fine: _shift));
        },
        onPointerPanZoomEnd: frozen ? null : (_) {
          if (_panning == true && dragging) { widget.slot.commit(); setState(() => dragging = false); }
          _panning = null;
        },
        // clicks are told from drags by distance, so a click acts at once and a quick second click is a double
        onPointerDown: (e) { if (e.buttons == kSecondaryButton) { s.store.menu(context, s.id, s.axis, e.position); return; } _down = e.position; _moved = false; if (!frozen && !editing) focus.requestFocus(); },
        onPointerMove: (e) { if ((e.position - _down).distance > 4) _moved = true; },
        onPointerUp: (e) {
          if (_moved || frozen) return;
          final now = DateTime.now();
          if (_last != null && now.difference(_last!) < const Duration(milliseconds: 320)) {
            _last = null;
            setState(() => editing = false);
            s.reset();
          } else {
            _last = now;
            if (!editing) _startEdit();
          }
        },
        child: GestureDetector(
          key: ValueKey(widget.keyName ?? 'toy-${s.id}${s.axis == null ? '' : '-${s.axis}'}'),
          dragStartBehavior: DragStartBehavior.down,
          onHorizontalDragStart: frozen ? null : (d) { _v = s.value; _scrubbed = false; _aborted = false; setState(() { dragging = true; editing = false; }); },
          onHorizontalDragUpdate: frozen ? null : (d) {
            if (_aborted) return;
            _scrubbed = true;
            _v += d.delta.dx * s.rate * (_shift ? .1 : 1);
            s.preview(s.quantize(_v, fine: _shift));
          },
          onHorizontalDragEnd: frozen ? null : (_) { if (_scrubbed && !_aborted) s.commit(); _aborted = false; setState(() => dragging = false); },
          child: Container(
            height: h,
            decoration: BoxDecoration(
              color: dragging || (hover && !frozen) ? Surface.hover : Surface.raised,
              borderRadius: BorderRadius.circular(4),
              border: editing && _bad ? Border.all(color: kPink, width: 1.4) : (focus.hasFocus || editing ? Border.all(color: Surface.ink.withValues(alpha: .85), width: 1.4) : null),
            ),
            child: ClipRRect(
              borderRadius: BorderRadius.circular(4),
              child: Stack(children: [
                if (tint > 0) Positioned.fill(child: ColoredBox(color: tone.withValues(alpha: tint))),
                if (track) Positioned.fill(key: ValueKey('track-${s.id}'), child: Align(alignment: Alignment.centerLeft, child: FractionallySizedBox(widthFactor: frac, heightFactor: 1, child: DecoratedBox(decoration: BoxDecoration(color: tone.withValues(alpha: frozen ? .08 : .16), border: Border(bottom: BorderSide(color: tone.withValues(alpha: frozen ? .3 : .75), width: 1.5))))))),
                Positioned(left: 0, top: 0, bottom: 0, width: edge, child: ColoredBox(color: tone)),
                Positioned.fill(child: Padding(
                  padding: EdgeInsets.only(left: edge + 9, right: 7),
                  child: Row(children: [
                    if (widget.tag != null) Padding(padding: const EdgeInsets.only(right: 5), child: Text(widget.tag!, style: sans(Dn.microSize, c: N.g44, w: FontWeight.w600))),
                    Expanded(
                      child: editing
                          ? EditableText(controller: ctl, focusNode: editFocus, autofocus: true, style: sans(widget.hero ? 14 : 13, c: Surface.ink, w: FontWeight.w600), cursorColor: Surface.ink, backgroundCursorColor: Surface.muted, selectionColor: tone.withValues(alpha: .4), keyboardType: const TextInputType.numberWithOptions(decimal: true, signed: true), onSubmitted: _submit)
                          : LayoutBuilder(builder: (context, box) {
                              // a number is never cut mid-digit ("10(" for 100): a narrow well first drops its decimals
                              // (the value is unchanged, only its display), and only a number still too wide is drawn smaller
                              final style = sans(widget.hero ? 14 : 13, c: dragging ? Surface.ink : ink, w: FontWeight.w500);
                              var text = s.mixed ? '—' : s.format(s.value, decimals: widget.decimals);
                              bool fits(String t) {
                                final p = TextPainter(text: TextSpan(text: t, style: style), textDirection: TextDirection.ltr, maxLines: 1)..layout();
                                final w = p.width;
                                p.dispose();
                                return w <= box.maxWidth;
                              }

                              // fewer decimals until it fits, but never so few that a value that is not zero reads as 0
                              for (var d = 2; !s.mixed && !fits(text) && d >= 0; d--) {
                                final shorter = d == 0 ? s.format(s.value, decimals: 0) : (s.value * s.displayScale).toStringAsFixed(d);
                                if (shorter.length >= text.length && d > 0) continue;
                                if (s.value != 0 && double.tryParse(shorter) == 0) break;
                                text = shorter;
                              }
                              return FittedBox(fit: BoxFit.scaleDown, alignment: Alignment.centerLeft, child: Text(text, softWrap: false, style: style));
                            }),
                    ),
                    if (editing && _bad) Text('Number required', key: ValueKey('bad-${s.id}'), style: sans(Dn.labelSize, c: kPink))
                    else if (unit != null) Text(unit, style: sans(Dn.labelSize, c: Surface.muted)),
                    // the way to grab it, shown when the pointer is near
                    if (unit == null && hover && !editing && !frozen && !dragging) Text('‹ ›', style: sans(Dn.nameSize, c: N.g44, w: FontWeight.w600)),
                  ]),
                )),
              ]),
            ),
          ),
        ),
      ),
      ),
    );
  }
}

/// Related numbers side by side: a vector is its axes, each one a Value.
class ValuesToy extends StatelessWidget {
  const ValuesToy(this.store, this.id, this.axes, {super.key, this.hero = false, this.tone = kBlue});
  final Color tone;
  final ParamStore store;
  final String id;
  final int axes;
  final bool hero;
  @override
  Widget build(BuildContext context) {
    final slots = [for (var i = 0; i < axes; i++) Slot(store, id, i, id)];
    for (final s in slots) { s.peers = [for (final o in slots) if (o != s) o]; }
    return Row(children: [
      for (var i = 0; i < axes; i++) ...[
        Expanded(child: ValueToy(slots[i], tag: const ['X', 'Y', 'Z'][i], hero: hero, tone: tone)),
        if (i < axes - 1) const SizedBox(width: 2),
      ],
    ]);
  }
}

/// Two declared rows that are one point: their Values side by side, linkable like a vector.
class PairToy extends StatelessWidget {
  const PairToy(this.store, this.ids, this.linkId, {super.key, this.hero = false, this.tone = kBlue});
  final ParamStore store;
  final List<String> ids;
  final String linkId;
  final bool hero;
  final Color tone;
  @override
  Widget build(BuildContext context) {
    final slots = [for (final id in ids) Slot(store, id, null, linkId)];
    for (final s in slots) { s.peers = [for (final o in slots) if (o != s) o]; }
    return Row(children: [
      for (var i = 0; i < slots.length; i++) ...[
        Expanded(child: ValueToy(slots[i], tag: const ['X', 'Y'][i], hero: hero, tone: tone)),
        if (i < slots.length - 1) const SizedBox(width: 2),
      ],
    ]);
  }
}

class ToggleToy extends StatelessWidget {
  const ToggleToy(this.store, this.id, {super.key, this.tone = kBlue, this.hero = false});
  final ParamStore store;
  final String id;
  final Color tone;
  final bool hero;
  @override
  Widget build(BuildContext context) {
    final on = store.row(id)['value'] == true;
    final frozen = store.frozen;
    final tone = frozen ? dimTone(this.tone) : this.tone;
    return GestureDetector(
      key: ValueKey('toy-$id'),
      behavior: HitTestBehavior.opaque,
      onTap: frozen ? null : () => store.set(id, !on),
      child: Container(
        height: Surface.control,
        padding: const EdgeInsets.symmetric(horizontal: 7.5),
        decoration: BoxDecoration(color: on && hero ? tone.withValues(alpha: .26) : Surface.raised, borderRadius: BorderRadius.circular(4)),
        child: Row(children: [
          Container(width: 22.5, height: 13.5, padding: const EdgeInsets.all(1.5), alignment: on ? Alignment.centerRight : Alignment.centerLeft, decoration: BoxDecoration(color: on ? tone : N.g26, borderRadius: BorderRadius.circular(7)), child: Container(width: 10.5, height: 10.5, decoration: BoxDecoration(color: frozen ? N.g56 : Surface.ink, shape: BoxShape.circle))),
          const SizedBox(width: 7.5),
          Text(on ? 'On' : 'Off', style: sans(Dn.nameSize, c: frozen ? N.g33 : (on ? _readout : Surface.muted), w: FontWeight.w500)),
        ]),
      ),
    );
  }
}

/// A short list is chips; a long one is a stepper. Either way it is a choice from the declared list.
class ChoiceToy extends StatelessWidget {
  const ChoiceToy(this.store, this.id, {super.key, this.tone = kYellow});
  final ParamStore store;
  final String id;
  final Color tone;
  @override
  Widget build(BuildContext context) {
    final row = store.row(id);
    final choices = [for (final c in (row['choices'] as List)) '$c'];
    // a value that is none of the choices (a colour no preset names) highlights none
    final raw = ((row['value'] as num?) ?? 0).round();
    final cur = raw < 0 || raw >= choices.length ? -1 : raw;
    final frozen = store.frozen;
    void pick(int i) { if (!frozen) store.set(id, (i + choices.length) % choices.length); }
    if (choices.length <= 4) {
      // one control: the options in a single track, the chosen one in the property's tone (a selection, not a chip)
      return Container(
        height: Surface.control,
        padding: const EdgeInsets.all(1.5),
        decoration: BoxDecoration(color: N.g07, border: Border.all(color: N.g20), borderRadius: BorderRadius.circular(4)),
        child: Row(children: [
        for (final (i, c) in choices.indexed) ...[
          Expanded(
            child: GestureDetector(
              key: ValueKey('choice-$id-$i'),
              behavior: HitTestBehavior.opaque,
              onTap: () => pick(i),
              child: Container(alignment: Alignment.center, decoration: BoxDecoration(color: i == cur ? (frozen ? dimTone(tone) : tone) : null, borderRadius: BorderRadius.circular(2)), child: Text(c, softWrap: false, overflow: TextOverflow.clip, style: sans(Dn.nameSize, c: i == cur ? (frozen ? N.g56 : _dark) : (frozen ? N.g33 : N.g69), w: i == cur ? FontWeight.w700 : FontWeight.w500))),
            ),
          ),
          if (i < choices.length - 1) const SizedBox(width: 1.5),
        ],
      ]),
      );
    }
    // the stepper always names one choice: an out-of-range value reads as the nearest, as before
    final shown = raw.clamp(0, choices.length - 1);
    Widget arrow(String k, String t, int d) => GestureDetector(key: ValueKey('choice-$id-$k'), behavior: HitTestBehavior.opaque, onTap: () => pick(shown + d), child: SizedBox(width: 18, height: Surface.control, child: Center(child: Text(t, style: sans(Dn.nameSize, c: Surface.muted)))));
    return Container(
      height: 22.5,
      decoration: BoxDecoration(color: Surface.raised, borderRadius: BorderRadius.circular(4)),
      child: Row(children: [arrow('prev', '‹', -1), Expanded(child: Center(child: Text(choices[shown], key: ValueKey('choice-$id-name'), softWrap: false, overflow: TextOverflow.clip, style: sans(Dn.nameSize, c: frozen ? N.g33 : _readout, w: FontWeight.w500)))), arrow('next', '›', 1)]),
    );
  }
}

class TextToy extends StatefulWidget {
  const TextToy(this.store, this.id, {super.key, this.rawJson = false});
  final ParamStore store;
  final String id;
  final bool rawJson; // an unrecognised row: its value as text, applied when it parses
  @override
  State<TextToy> createState() => _TextToyState();
}

class _TextToyState extends State<TextToy> {
  final focus = FocusNode();
  late final ctl = TextEditingController(text: _text);
  String get _text {
    final v = widget.store.row(widget.id)['value'];
    return widget.rawJson ? jsonEncode(v) : '${v ?? ''}';
  }

  @override
  void initState() {
    super.initState();
    focus.addListener(() { if (!focus.hasFocus) _apply(); setState(() {}); });
  }

  void _apply() {
    if (widget.store.frozen) return;
    Object? v = ctl.text;
    if (widget.rawJson) {
      try { v = jsonDecode(ctl.text); } catch (_) { ctl.text = _text; return; }
    }
    if (v == widget.store.row(widget.id)['value']) return;
    widget.store.set(widget.id, v);
  }

  @override
  void dispose() {
    focus.dispose();
    ctl.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final frozen = widget.store.frozen;
    return Container(
      height: 22.5,
      padding: const EdgeInsets.symmetric(horizontal: 7),
      alignment: Alignment.centerLeft,
      decoration: BoxDecoration(color: Surface.raised, borderRadius: BorderRadius.circular(4), border: focus.hasFocus ? Border.all(color: kYellow, width: 1) : null),
      // Esc puts the value back and leaves (Classic IN-098)
      child: Focus(
        onKeyEvent: (_, e) {
          if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape && focus.hasFocus) {
            ctl.text = _text;
            focus.unfocus();
            return KeyEventResult.handled;
          }
          return KeyEventResult.ignored;
        },
        child: EditableText(key: ValueKey('text-${widget.id}'), controller: ctl, focusNode: focus, readOnly: frozen, maxLines: 1, style: widget.rawJson ? mono(Dn.nameSize, c: frozen ? N.g33 : _readout) : sans(Dn.nameSize, c: frozen ? N.g33 : _readout), cursorColor: kYellow, backgroundCursorColor: Surface.muted, onSubmitted: (_) => _apply()),
      ),
    );
  }
}

/// A pointer to another thing: it says what it points at and picks another from the declared candidates.
class ReferenceToy extends StatelessWidget {
  const ReferenceToy(this.store, this.id, {super.key, this.tone = kViolet});
  final ParamStore store;
  final String id;
  final Color tone;
  @override
  Widget build(BuildContext context) {
    final row = store.row(id);
    final refs = [for (final r in (row['refs'] as List? ?? const [])) '$r'];
    final cur = row['value'] as String?;
    return GestureDetector(
      key: ValueKey('ref-$id'),
      behavior: HitTestBehavior.opaque,
      onTap: store.frozen ? null : () {
        final i = cur == null ? -1 : refs.indexOf(cur);
        store.set(id, i + 1 >= refs.length ? null : refs[i + 1]);
      },
      child: Container(
        height: Surface.control,
        padding: const EdgeInsets.symmetric(horizontal: 7.5),
        decoration: BoxDecoration(color: Surface.raised, borderRadius: BorderRadius.circular(4)),
        child: Row(children: [
          Container(width: 10, height: 10, decoration: BoxDecoration(color: cur == null ? null : (store.frozen ? dimTone(tone) : tone), border: cur == null ? Border.all(color: Surface.muted, width: 1.4) : null, borderRadius: BorderRadius.circular(2))),
          const SizedBox(width: 6),
          Expanded(child: Text(cur ?? 'None', softWrap: false, overflow: TextOverflow.clip, style: sans(Dn.nameSize, c: cur == null ? Surface.muted : _readout, w: FontWeight.w500))),
          Text('⌄', style: sans(Dn.nameSize, c: Surface.muted)),
        ]),
      ),
    );
  }
}

Color hexColor(String h) {
  final s = h.replaceAll('#', '');
  return Color(int.tryParse('FF$s', radix: 16) ?? 0xFF808080);
}

/// Where the host owns a specialist: the Inspector shows the current value and hands over to it.
class RouteToy extends StatelessWidget {
  const RouteToy(this.store, this.id, {super.key, this.tone = kBlue});
  final ParamStore store;
  final String id;
  final Color tone;
  @override
  Widget build(BuildContext context) {
    final row = store.row(id);
    final to = (row['route'] as String?) ?? const {'color': 'Colors', 'font': 'Fonts', 'blend': 'Blend', 'ease': 'Ease'}['${row['kind']}'] ?? 'Browser';
    final v = row['value'];
    return GestureDetector(
      key: ValueKey('route-$id'),
      behavior: HitTestBehavior.opaque,
      onTap: () => store.route(to, id),
      child: Container(
        height: Surface.control,
        padding: const EdgeInsets.only(left: 7, right: 7.5),
        decoration: BoxDecoration(color: Surface.raised, borderRadius: BorderRadius.circular(4)),
        child: Row(children: [
          if (row['kind'] == 'color' && v is String) Container(width: 10.5, height: 10.5, decoration: BoxDecoration(color: hexColor(v), shape: BoxShape.circle)),
          if (row['kind'] == 'color') const SizedBox(width: 7),
          Expanded(child: Text('$v', softWrap: false, overflow: TextOverflow.clip, style: mono(Dn.nameSize, c: _readout))),
          Container(padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2), decoration: BoxDecoration(color: store.frozen ? dimTone(tone) : tone, borderRadius: BorderRadius.circular(7)), child: Text('$to  →', style: sans(Dn.labelSize, c: _dark, w: FontWeight.w700))),
        ]),
      ),
    );
  }
}

class ActionChip extends StatelessWidget {
  const ActionChip(this.store, this.id, this.name, {super.key, this.tone = kPink});
  final ParamStore store;
  final String id, name;
  final Color tone;
  @override
  Widget build(BuildContext context) => GestureDetector(
        key: ValueKey('action-$id-$name'),
        behavior: HitTestBehavior.opaque,
        onTap: () => store.act(id, name),
        child: Container(height: 18, padding: const EdgeInsets.symmetric(horizontal: 9), alignment: Alignment.center, decoration: BoxDecoration(color: store.frozen ? dimTone(tone) : tone, borderRadius: BorderRadius.circular(9)), child: Text(name, style: sans(Dn.labelSize, c: _dark, w: FontWeight.w700))),
      );
}
