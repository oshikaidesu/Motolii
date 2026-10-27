// The desk's Ease tool: the curve between the picked key and the next one, its two handles, presets as thumbnails, and the numbers.

import 'package:flutter/widgets.dart';
import 'package:lab_book/tokens.dart';
import 'package:lab_book/workspace/desk_ease.dart';
import 'package:lab_book/workspace/desk_ease_graph.dart';
import 'package:lab_book/workspace/desk_ease_numbers.dart';
import 'package:lab_book/workspace/desk_ease_presets.dart';
import 'package:lab_book/workspace/desk_ease_span_strip.dart';
import 'package:lab_book/workspace/ws.dart';

/// The curve copied in the Ease desk, kept while the window is open (Alight Motion's Copy of a curve).
Ease? easeCopied;

class DeskEaseState extends State<DeskEase> {
  // Joined: while a hand is down the ease is told to the document as it goes (the stage and the bars follow), the newest
  // one at a time (one still on its way: the next waits, those between are skipped); all of it one step of undo.
  bool _down = false, _open = false, _going = false;
  Ease? _pending, _waiting;
  String? _trouble;

  // Every picked key's stretch, on whatever bar it is: each value keyed there. A last key has no stretch after it: the
  // one that ends at it, as the desk shows.
  List<(int, String, double)> _stretches(Ws ws) => [
      for (final k in ws.keys)
        if (ws.layers.where((l) => l.id == k.layer).firstOrNull case final l?)
          for (final at in keysAt(ws, (layer: l, from: l.keys.any((f) => f > k.frame) ? k.frame : (l.keys.where((f) => f < k.frame).fold<int?>(null, (a, f) => a == null || f > a ? f : a) ?? k.frame), to: k.frame, picked: k.frame)).entries)
            (int.parse(l.id), at.key, at.value),
      ];

  // (while the hand is down: the newest ease told, one at a time; the step opened at the first)
  void _live(Ws ws, Ease e) {
    final m = ws.inside!;
    if (_going) {
      _waiting = e;
      return;
    }
    final all = _stretches(ws);
    if (all.isEmpty) return;
    if (!_open) {
      m.later('together', number: 1);
      _open = true;
    }
    _going = true;
    var last = 0;
    for (final (t, name, at) in all) {
      last = m.later('way', thing: t, name: name, number: at, word: said(e));
    }
    ws.after(last, (_, _) {
      _going = false;
      final next = _waiting;
      _waiting = null;
      if (next != null) _live(ws, next);
    });
  }

  void _settle(Ws ws, EaseSpan? span) {
    final e = _pending;
    _pending = null;
    _waiting = null;
    if (e == null || span == null) return;
    final m = ws.inside!, to = said(e);
    final all = _stretches(ws);
    // (as it already goes, each: nothing is told, so a way's own numbers that have no seat here are not written over;
    //  told as the hand went: told once more as it let go, and the step it opened closed)
    final told = [for (final (t, name, at) in all) if (_open || said(heard(m.wayOf(t, name, at))) != to) (t, name, at)];
    if (_open) {
      _open = false;
      for (final (t, name, at) in told) {
        m.later('way', thing: t, name: name, number: at, word: to);
      }
      ws.after(m.later('together'), (done, why) => mounted ? setState(() => _trouble = done ? null : why) : null);
      return;
    }
    if (told.isEmpty) return setState(() => _trouble = all.isEmpty ? 'No key where the stretch begins' : null);
    m.later('together', number: 1);   // (one step, without waiting)
    for (final (t, name, at) in told) {
      m.later('way', thing: t, name: name, number: at, word: to);
    }
    ws.after(m.later('together'), (done, why) => mounted ? setState(() => _trouble = done ? null : why) : null);
  }

  // (the curve after every key of each picked key's layer, every value keyed there: one step)
  void _onEveryKey(Ws ws, Ease e) {
    final m = ws.inside!, layers = {for (final k in ws.keys) k.layer};
    m.later('together', number: 1);
    for (final l in ws.layers.where((l) => layers.contains(l.id))) {
      for (final f in l.keys) {
        for (final at in keysAt(ws, (layer: l, from: f, to: f, picked: f)).entries) {
          m.later('way', thing: int.parse(l.id), name: at.key, number: at.value, word: said(e));
        }
      }
    }
    ws.after(m.later('together'), (done, why) => mounted ? setState(() => _trouble = done ? null : why) : null);
  }

  @override
  Widget build(BuildContext context) {
    final ws = WsScope.of(context, {WsAspect.document, WsAspect.picked}), span = easeSpan(ws), store = eases[ws] ??= {}, id = spanId(span);
    if (ws.inside case final m?) {
      final at = span == null ? const <String, double>{} : keysAt(ws, span);
      var ease = _pending ?? (at.isEmpty ? easePreset('Linear') : heard(m.wayOf(int.parse(span!.layer.id), at.keys.first, at.values.first)));
      if (easeOther(ease) && ease.drawn == null) {
        // (no seat for this kind: its curve is the inside's own working of it, as said now)
        ease = Ease(ease.kind, ease.p1, ease.p2, ease.name, [for (var i = 0; i <= 48; i++) m.wayAt(ease.name!, i / 48)]);
      }
      void set(Ease e) {
        setState(() => _pending = e);
        _down ? _live(ws, e) : _settle(ws, span);
      }

      return Listener(
        onPointerDown: (_) => _down = true,
        onPointerUp: (_) {
          _down = false;
          _settle(ws, span);
        },
        onPointerCancel: (_) {
          _down = false;
          setState(() => _pending = null);
        },
        child: _body(ws, span, ease, set),
      );
    }
    final ease = store[id] ?? easePreset('Ease');
    return _body(ws, span, ease, (e) => setState(() => store[id] = e));
  }

  Widget _body(Ws ws, EaseSpan? span, Ease ease, ValueChanged<Ease> set) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        SpanStrip(
          span: span,
          onReverse: () => set(ease.reversed()),
          onCopy: () => setState(() => easeCopied = ease),
          onPaste: easeCopied == null ? null : () => set(easeCopied!),
          onPasteAll: easeCopied == null || ws.inside == null ? null : () => _onEveryKey(ws, easeCopied!),
        ),
        if (_trouble case final why?) Text(why, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(Grey.g63)),
        const SizedBox(height: WsT.gutter),
        Expanded(
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Expanded(
                child: ColoredBox(
                  color: WsT.card,
                  child: ValueListenableBuilder<int>(valueListenable: ws.moment, builder: (_, frame, _) => EaseGraph(ease: ease, span: span, frame: frame, onChanged: set))   // (only the graph follows the playhead),
                ),
              ),
              const SizedBox(width: WsT.gutter),
              SizedBox(
                width: Shape.z(128),
                child: EaseNumbers(ease: ease, span: span, onChanged: set),
              ),
            ],
          ),
        ),
        const SizedBox(height: WsT.gutter),
        Presets(ease: ease, onPick: set, more: unseated(ws)),
      ],
    );
  }
}
