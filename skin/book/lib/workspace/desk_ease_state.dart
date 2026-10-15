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
    final m = ws.inside!, t = int.parse(span.layer.id), at = keysAt(ws, span);
    // (as it already goes: nothing is told, so a way's own numbers that have no seat here are not written over)
    if (at.isNotEmpty && e == heard(m.wayOf(t, at.keys.first, at.values.first))) return;
    final done = m.together(() => [for (final k in at.entries) m.way(t, k.key, k.value, said(e))].every((ok) => ok));
    setState(() => _trouble = at.isEmpty ? 'No key where the stretch begins' : (done ? null : m.trouble));
  }

  @override
  Widget build(BuildContext context) {
    final ws = WsScope.of(context), span = easeSpan(ws), store = eases[ws] ??= {}, id = spanId(span);
    if (ws.inside case final m?) {
      final at = span == null ? const <String, double>{} : keysAt(ws, span);
      var ease = _pending ?? (at.isEmpty ? easePresets.first : heard(m.wayOf(int.parse(span!.layer.id), at.keys.first, at.values.first)));
      if (easeOther(ease) && ease.drawn == null) {
        // (no seat for this kind: its curve is read off the document, the value itself between the two keys)
        final t = int.parse(span!.layer.id), name = at.keys.first, from = at.values.first;
        final to = m.keyMoments(t, name).where((s) => s > from).firstOrNull;
        final a = m.value(t, name, from), b = to == null ? null : m.value(t, name, to);
        if (to != null && a != null && b != null && (b - a).abs() > 1e-9) {
          ease = Ease(ease.kind, ease.p1, ease.p2, ease.name, [for (var i = 0; i <= 48; i++) ((m.value(t, name, from + (to - from) * i / 48) ?? a) - a) / (b - a)]);
        }
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
    final ease = store[id] ?? easePresets[1];
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
                width: 128,
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
