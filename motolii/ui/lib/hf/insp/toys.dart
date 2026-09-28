// The generic Toys. A handful of primitives; meaning composes them, it does not add widgets.
// Shared grammar: drag = manipulate, Shift = fine, click = exact input, double click = default,
// arrow keys nudge, Delete resets. Hard min/max clamp silently and never draw a finite bar.
import 'dart:convert';
import 'dart:math' as math;
import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import '../bp/common.dart';
import '../desk/common.dart' show kYellow, kBlue, kViolet, kPink, kInk;
import 'rows.dart';
import 'tones.dart';

const kTile = Color(0xFF212123);
const kTileHi = Color(0xFF2B2B2E);
const _readout = Color(0xFFD0D1D5); // the number is a readout: present, not loud
const _dark = Color(0xFF1B1B1D);

/// One number inside a row: the row's own value, or one axis of a vector. It also knows the little that the
/// row's character adds to a Value (units, whole steps, a finite percent), and which numbers move with it when linked.
class Slot {
  Slot(this.store, this.id, [this.axis, this.linkId]);
  final ParamStore store;
  final String id;
  final int? axis;
  final String? linkId; // set when this number belongs to a linkable group
  List<Slot> peers = const [];
  Map<String, dynamic> get row => store.row(id);
  Character get character => axis == null ? characterOf(row) : Character.none;
  bool get isAngle => character == Character.angle;
  bool get whole => const ['i32', 'u32', 'int'].contains('${row['kind']}') || ((character == Character.count || character == Character.seed) && value == value.roundToDouble());
  double get displayScale => character == Character.opacity || row['percent'] == true ? 100 : 1;
  String? get unit => (row['unit'] as String?) ?? (isAngle ? '°' : (displayScale == 100 ? '%' : null));
  bool get mixed => store.mixed(id, axis);

  /// A typed number is absolute for every target, unlike a scrub.
  void typed(double v) {
    store.typing = true;
    try {
      preview(v);
      commit();
    } finally {
      store.typing = false;
    }
  }

  double get value {
    final v = row['value'];
    return ((axis == null ? v : (v as List)[axis!]) as num).toDouble();
  }

  double? get def {
    final d = row['default'];
    if (d == null) return null;
    return ((axis == null ? d : (d as List)[axis!]) as num).toDouble();
  }

  void _raw(double v) {
    if (axis == null) {
      store.preview(id, v);
    } else {
      final l = List<Object?>.of(row['value'] as List);
      l[axis!] = v;
      store.preview(id, l);
    }
  }

  void preview(double v) {
    final old = value;
    _raw(v);
    // linked axes keep their ratio: one drag scales the whole group
    if (linkId != null && store.linked.contains(linkId) && old != 0) {
      final r = value / old;
      for (final p in peers) { p._raw(p.value * r); }
    }
  }

  void commit() => store.commit(id);
  void cancel() => store.cancel(id);
  void reset() {
    final d = def;
    if (d == null) return;
    preview(d);
    commit();
  }

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

  /// What a scrub lands on: whole degrees (tenths with Shift), whole counts and seeds.
  double quantize(double v, {bool fine = false}) => isAngle ? (fine ? (v * 10).round() / 10 : v.roundToDouble()) : (whole ? v.roundToDouble() : v);

  String format(double v, {int? decimals}) {
    if (row['zeroWord'] is String && v == 0) return row['zeroWord'] as String; // 0 rows means auto
    final shown = v * displayScale;
    if (decimals == 0) return '${shown.round()}'; // a narrow tile keeps its digits whole, as Classic does
    if (isAngle || displayScale != 1) return shown == shown.roundToDouble() ? '${shown.round()}' : shown.toStringAsFixed(1);
    return formatValue(v, whole: whole);
  }
}

String formatValue(double v, {bool whole = false}) {
  if (whole) return '${v.round()}';
  final a = v.abs();
  if (a != 0 && a < .001) return v.toStringAsExponential(2); // a small value must not read as zero
  return v.toStringAsFixed(a < 10 ? 3 : (a < 1000 ? 2 : 1));
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
      if (!editFocus.hasFocus && editing) _submit(ctl.text);
    });
  }

  @override
  void dispose() {
    if (widget.focusNode == null) focus.dispose();
    editFocus.dispose();
    ctl.dispose();
    super.dispose();
  }

  bool get frozen => widget.slot.store.frozen;

  void _startEdit() {
    final v = widget.slot.value;
    final shown = v * widget.slot.displayScale;
    ctl.text = widget.slot.whole ? '${shown.round()}' : (shown == shown.roundToDouble() ? shown.toStringAsFixed(1) : '$shown');
    ctl.selection = TextSelection(baseOffset: 0, extentOffset: ctl.text.length);
    _cancel = false;
    _subject = widget.slot.store.subject;
    setState(() => editing = true);
  }

  Object? _subject;

  void _submit(String t) {
    if (_cancel) { _cancel = false; return; }
    final v = double.tryParse(t.trim());
    if (mounted) setState(() => editing = false);
    if (widget.slot.store.subject != _subject) return; // typed for a layer that is no longer the one shown
    if (v != null && !frozen) {
      widget.slot.typed(v / widget.slot.displayScale);
    }
  }

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
    final h = widget.hero ? 34.0 : 30.0;
    final ink = frozen ? const Color(0xFF55565C) : _readout;
    final unit = widget.showUnit ? s.unit : null;
    // colour mass follows importance: an ordinary Value has a thin edge, a touched one a wider edge and a tint, a hero more
    final active = dragging || hover || focus.hasFocus || editing;
    final tone = frozen ? dimTone(widget.tone) : widget.tone;
    final edge = widget.hero || active ? 6.0 : 3.0;
    final tint = widget.hero ? .16 : (active ? .11 : 0.0);
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
              color: dragging || (hover && !frozen) ? kTileHi : kTile,
              borderRadius: BorderRadius.circular(5),
              border: focus.hasFocus || editing ? Border.all(color: kInk.withValues(alpha: .85), width: 1.4) : null,
            ),
            child: ClipRRect(
              borderRadius: BorderRadius.circular(5),
              child: Stack(children: [
                if (tint > 0) Positioned.fill(child: ColoredBox(color: tone.withValues(alpha: tint))),
                if (track) Positioned.fill(key: ValueKey('track-${s.id}'), child: Align(alignment: Alignment.centerLeft, child: FractionallySizedBox(widthFactor: frac, heightFactor: 1, child: ColoredBox(color: tone.withValues(alpha: frozen ? .12 : .32))))),
                Positioned(left: 0, top: 0, bottom: 0, width: edge, child: ColoredBox(color: tone)),
                Positioned.fill(child: Padding(
                  padding: EdgeInsets.only(left: edge + 9, right: 9),
                  child: Row(children: [
                    if (widget.tag != null) Padding(padding: const EdgeInsets.only(right: 7), child: Text(widget.tag!, style: sans(9.5, c: const Color(0xFF6E6F76), w: FontWeight.w600))),
                    Expanded(
                      child: editing
                          ? EditableText(controller: ctl, focusNode: editFocus, autofocus: true, style: sans(widget.hero ? 17 : 15, c: kInk, w: FontWeight.w600), cursorColor: kInk, backgroundCursorColor: kMuted, selectionColor: tone.withValues(alpha: .4), keyboardType: const TextInputType.numberWithOptions(decimal: true, signed: true), onSubmitted: _submit)
                          : Text(s.mixed ? '—' : s.format(s.value, decimals: widget.decimals), softWrap: false, overflow: TextOverflow.clip, style: sans(widget.hero ? 14 : 13, c: dragging ? kInk : ink, w: FontWeight.w500)),
                    ),
                    if (unit != null) Text(unit, style: sans(10.5, c: kMuted)),
                    // the way to grab it, shown when the pointer is near
                    if (unit == null && hover && !editing && !frozen && !dragging) Text('‹ ›', style: sans(12, c: const Color(0xFF6A6B72), w: FontWeight.w600)),
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
        if (i < axes - 1) const SizedBox(width: 3),
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
        if (i < slots.length - 1) const SizedBox(width: 3),
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
        height: 30,
        padding: const EdgeInsets.symmetric(horizontal: 10),
        decoration: BoxDecoration(color: on && hero ? tone.withValues(alpha: .26) : kTile, borderRadius: BorderRadius.circular(5)),
        child: Row(children: [
          Container(width: 30, height: 18, padding: const EdgeInsets.all(2), alignment: on ? Alignment.centerRight : Alignment.centerLeft, decoration: BoxDecoration(color: on ? tone : const Color(0xFF3A3B40), borderRadius: BorderRadius.circular(9)), child: Container(width: 14, height: 14, decoration: BoxDecoration(color: frozen ? const Color(0xFF8A8B90) : kInk, shape: BoxShape.circle))),
          const SizedBox(width: 10),
          Text(on ? 'On' : 'Off', style: sans(11.5, c: frozen ? const Color(0xFF55565C) : (on ? _readout : kMuted), w: FontWeight.w500)),
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
      return Row(children: [
        for (final (i, c) in choices.indexed) ...[
          Expanded(
            child: GestureDetector(
              key: ValueKey('choice-$id-$i'),
              behavior: HitTestBehavior.opaque,
              onTap: () => pick(i),
              child: Container(height: 30, alignment: Alignment.center, decoration: BoxDecoration(color: i == cur ? (frozen ? dimTone(tone) : tone) : kTile, borderRadius: BorderRadius.circular(5)), child: Text(c, softWrap: false, overflow: TextOverflow.clip, style: sans(11.5, c: i == cur ? (frozen ? const Color(0xFF8A8B90) : _dark) : (frozen ? const Color(0xFF55565C) : const Color(0xFFB4B6BB)), w: i == cur ? FontWeight.w700 : FontWeight.w500))),
            ),
          ),
          if (i < choices.length - 1) const SizedBox(width: 3),
        ],
      ]);
    }
    // the stepper always names one choice: an out-of-range value reads as the nearest, as before
    final shown = raw.clamp(0, choices.length - 1);
    Widget arrow(String k, String t, int d) => GestureDetector(key: ValueKey('choice-$id-$k'), behavior: HitTestBehavior.opaque, onTap: () => pick(shown + d), child: SizedBox(width: 28, height: 30, child: Center(child: Text(t, style: sans(16, c: kMuted)))));
    return Container(
      height: 30,
      decoration: BoxDecoration(color: kTile, borderRadius: BorderRadius.circular(5)),
      child: Row(children: [arrow('prev', '‹', -1), Expanded(child: Center(child: Text(choices[shown], key: ValueKey('choice-$id-name'), softWrap: false, overflow: TextOverflow.clip, style: sans(12, c: frozen ? const Color(0xFF55565C) : _readout, w: FontWeight.w500)))), arrow('next', '›', 1)]),
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
      height: 30,
      padding: const EdgeInsets.symmetric(horizontal: 9),
      alignment: Alignment.centerLeft,
      decoration: BoxDecoration(color: kTile, borderRadius: BorderRadius.circular(5), border: focus.hasFocus ? Border.all(color: kYellow, width: 1.6) : null),
      child: EditableText(key: ValueKey('text-${widget.id}'), controller: ctl, focusNode: focus, readOnly: frozen, maxLines: 1, style: widget.rawJson ? mono(11, c: frozen ? const Color(0xFF55565C) : _readout) : sans(12, c: frozen ? const Color(0xFF55565C) : _readout), cursorColor: kYellow, backgroundCursorColor: kMuted, onSubmitted: (_) => _apply()),
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
        height: 30,
        padding: const EdgeInsets.symmetric(horizontal: 10),
        decoration: BoxDecoration(color: kTile, borderRadius: BorderRadius.circular(5)),
        child: Row(children: [
          Container(width: 13, height: 13, decoration: BoxDecoration(color: cur == null ? null : (store.frozen ? dimTone(tone) : tone), border: cur == null ? Border.all(color: kMuted, width: 1.4) : null, borderRadius: BorderRadius.circular(3))),
          const SizedBox(width: 8),
          Expanded(child: Text(cur ?? 'None', softWrap: false, overflow: TextOverflow.clip, style: sans(12, c: cur == null ? kMuted : _readout, w: FontWeight.w500))),
          Text('⌄', style: sans(13, c: kMuted)),
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
        height: 30,
        padding: const EdgeInsets.only(left: 9, right: 10),
        decoration: BoxDecoration(color: kTile, borderRadius: BorderRadius.circular(5)),
        child: Row(children: [
          if (row['kind'] == 'color' && v is String) Container(width: 14, height: 14, decoration: BoxDecoration(color: hexColor(v), shape: BoxShape.circle)),
          if (row['kind'] == 'color') const SizedBox(width: 9),
          Expanded(child: Text('$v', softWrap: false, overflow: TextOverflow.clip, style: mono(11, c: _readout))),
          Container(padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3), decoration: BoxDecoration(color: store.frozen ? dimTone(tone) : tone, borderRadius: BorderRadius.circular(9)), child: Text('$to  →', style: sans(10, c: _dark, w: FontWeight.w700))),
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
        child: Container(height: 24, padding: const EdgeInsets.symmetric(horizontal: 12), alignment: Alignment.center, decoration: BoxDecoration(color: store.frozen ? dimTone(tone) : tone, borderRadius: BorderRadius.circular(12)), child: Text(name, style: sans(10.5, c: _dark, w: FontWeight.w700))),
      );
}
