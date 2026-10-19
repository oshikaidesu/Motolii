// The desk's Ease tool: the curve between the picked key and the next one, its two handles, presets as thumbnails, and the numbers.

import 'package:flutter/widgets.dart';
import 'package:lab_book/tokens.dart';
import 'package:lab_book/workspace/desk_ease.dart';
import 'package:lab_book/workspace/desk_ease_graph.dart';
import 'package:lab_book/workspace/desk_ease_numbers.dart';
import 'package:lab_book/workspace/desk_ease_presets.dart';
import 'package:lab_book/workspace/desk_ease_span_strip.dart';
import 'package:lab_book/workspace/ws.dart';

class DeskEaseState extends State<DeskEase> {
  // Joined: while a hand is down the ease is only shown here; when it lets go it is told to the document once.
  bool _down = false;
  Ease? _pending;
  String? _trouble;

  void _settle(Ws ws, EaseSpan? span) {
    final e = _pending;
    _pending = null;
    if (e == null || span == null) return;
    final m = ws.inside!, to = said(e);
    // (every picked key's stretch, on whatever bar it is: each value keyed there; one step)
    final all = [
      for (final k in ws.keys)
        if (ws.layers.where((l) => l.id == k.layer).firstOrNull case final l?)
          for (final at in keysAt(ws, (layer: l, from: k.frame, to: k.frame, picked: k.frame)).entries) (int.parse(l.id), at.key, at.value),
    ];
    // (as it already goes, each: nothing is told, so a way's own numbers that have no seat here are not written over)
    final told = [for (final (t, name, at) in all) if (said(heard(m.wayOf(t, name, at))) != to) (t, name, at)];
    if (told.isEmpty) return setState(() => _trouble = all.isEmpty ? 'No key where the stretch begins' : null);
    m.later('together', number: 1);   // (one step, without waiting)
    for (final (t, name, at) in told) {
      m.later('way', thing: t, name: name, number: at, word: to);
    }
    ws.after(m.later('together'), (done, why) => mounted ? setState(() => _trouble = done ? null : why) : null);
  }

  @override
  Widget build(BuildContext context) {
    final ws = WsScope.of(context), span = easeSpan(ws), store = eases[ws] ??= {}, id = spanId(span);
    if (ws.inside case final m?) {
      final at = span == null ? const <String, double>{} : keysAt(ws, span);
      var ease = _pending ?? (at.isEmpty ? easePreset('Linear') : heard(m.wayOf(int.parse(span!.layer.id), at.keys.first, at.values.first)));
      if (easeOther(ease) && ease.drawn == null) {
        // (no seat for this kind: its curve is the inside's own working of it, as said now)
        ease = Ease(ease.kind, ease.p1, ease.p2, ease.name, [for (var i = 0; i <= 48; i++) m.wayAt(ease.name!, i / 48)]);
      }
      void set(Ease e) {
        setState(() => _pending = e);
        if (!_down) _settle(ws, span);
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
        SpanStrip(span: span, onReverse: () => set(ease.reversed())),
        if (_trouble case final why?) Text(why, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(Grey.g63)),
        const SizedBox(height: WsT.gutter),
        Expanded(
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Expanded(
                child: ColoredBox(
                  color: WsT.card,
                  child: EaseGraph(ease: ease, span: span, frame: ws.frame, onChanged: set),
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
