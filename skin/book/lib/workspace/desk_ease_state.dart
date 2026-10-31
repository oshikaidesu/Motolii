// The desk's Ease tool: the curve between the picked key and the next one, its two handles, presets as thumbnails, and the numbers.

import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:lab_book/motolii.dart';
import 'package:lab_book/tokens.dart';
import 'package:lab_book/workspace/desk_ease.dart';
import 'package:lab_book/workspace/desk_ease_cards.dart';
import 'package:lab_book/workspace/desk_ease_graph.dart';
import 'package:lab_book/workspace/desk_ease_side.dart';
import 'package:lab_book/workspace/ws.dart';

/// The curve copied in the Ease desk, kept while the window is open (Alight Motion's Copy of a curve).
Ease? easeCopied;

/// Whether an end of a stretch pulled up or down jumps there: a key a frame outside keeps the value as it was, so the
/// value jumps at the key (three points, as the user drew it, 10/30); off, only the key's value moves and the stretch
/// beside follows. The user's to choose, on the desk (10/30: "飛びたくないなら後でおしりの点を消せばいい、もしくは設定で
/// 決めれると良い、ユーザに託す", "easeにon/offであるといい"). Kept while the window is open.
bool easeJump = true;

/// Whether the graph looks past 0 and 1 (Alight Motion's Enable Overshoot). Kept while the window is open.
bool easeOvershoot = false;

class DeskEaseState extends State<DeskEase> {
  // Joined: while a hand is down the ease is told to the document as it goes (the stage and the bars follow), the newest
  // one at a time (one still on its way: the next waits, those between are skipped); all of it one step of undo.
  bool _down = false, _open = false, _going = false;
  Ease? _pending, _waiting;
  int _card = 0;   // (of a way laid from cards, the one the graph shows; another stretch: its first)
  String _cardOf = '';
  Ease? _held;   // (what a tile held over the graph would make: seen before it is let go, Final Cut Pro's skimming of an effect)
  bool _overLocked = false, _noOver = false;   // (AbsentMotion's lockOvershoot, disableOvershoot: once it looks past 0 and 1 it stays so, until the menu says)
  String? _trouble;

  // (with no key picked the stretch follows the playhead: built again when it comes into another stretch, not every frame)
  Ws? _ws;
  String _spanAt = '';
  void _moved() {
    final id = spanId(easeSpan(_ws!));
    if (id != _spanAt && mounted) setState(() => _spanAt = id);
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    final ws = WsScope.read(context);
    if (ws == _ws) return;
    _ws?.moment.removeListener(_moved);
    _ws = ws..moment.addListener(_moved);
    _spanAt = spanId(easeSpan(ws));
  }

  @override
  void dispose() {
    _ws?.moment.removeListener(_moved);
    super.dispose();
  }

  // Every picked key's stretch, on whatever bar it is: each value keyed there. A last key has no stretch after it: the
  // one that ends at it, as the desk shows.
  List<(int, String, double)> _stretches(Ws ws) => [
      if (ws.keys.isEmpty)   // (none picked: the stretch the playhead is in)
        if (easeSpan(ws) case final s?)
          for (final at in keysAt(ws, s).entries) (int.parse(s.layer.id), at.key, at.value),
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
    _define(m, said(e));
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

  void _define(Motolii m, String word) {
    for (final (name, goes) in easeDefines(word)) {
      m.later('define', name: name, word: goes);
    }
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
      _define(m, to);
      for (final (t, name, at) in told) {
        m.later('way', thing: t, name: name, number: at, word: to);
      }
      _define(m, to);   // (again, the stretch on it: what it went by till now, no longer asked for, goes)
      ws.after(m.later('together'), (done, why) => mounted ? setState(() => _trouble = done ? null : why) : null);
      return;
    }
    if (told.isEmpty) return setState(() => _trouble = all.isEmpty ? 'No key where the stretch begins' : null);
    m.later('together', number: 1);   // (one step, without waiting)
    _define(m, to);
    for (final (t, name, at) in told) {
      m.later('way', thing: t, name: name, number: at, word: to);
    }
    _define(m, to);
    ws.after(m.later('together'), (done, why) => mounted ? setState(() => _trouble = done ? null : why) : null);
  }

  // (an end of the stretch pulled [by], in the graph's measure: each value keyed at both its ends moves its key there by
  //  so much of the way from the one key's value to the other's, or of its knob's span where the two are the same, as
  //  far as a hand takes it; with Jump on, a key a frame outside first keeps the key's value as it was. One step.)
  // (each value keyed at both ends of the stretch: its keys' frames and values, the one at the end pulled, and where
  //  [by] takes it: so much of the way from the one key's value to the other's, or of its knob's span where the two are
  //  the same, as far as a hand takes it)
  Iterable<(String, List<int>, List<double>, int, double)> _ends(Motolii m, int t, EaseSpan span, bool start, double by) sync* {
    for (final name in m.moving(t)) {
      final at = [for (final s in m.keyMoments(t, name)) (s * Ws.fps).round()], vs = m.keyValues(t, name);
      final i = at.indexOf(span.from), j = at.indexOf(span.to), k = start ? i : j;
      if (i < 0 || j < 0 || k >= vs.length || j >= vs.length) continue;
      final shown = m.shown(name), lo = shown?.low ?? -double.infinity, hi = shown?.high ?? double.infinity;
      final scale = (vs[j] - vs[i]).abs() > 1e-9 ? vs[j] - vs[i] : (shown == null ? 1.0 : hi - lo);
      yield (name, at, vs, k, (vs[k] + by * scale).clamp(math.min(lo, vs[k]), math.max(hi, vs[k])));
    }
  }

  // (an end of the stretch pulled [by]: with Jump on, a key a frame outside first keeps the key's value as it was. One step.)
  void _moveEnd(Ws ws, EaseSpan? span, bool start, double by) {
    final m = ws.inside;
    if (m == null || span == null || by.abs() < 1e-4) return;
    final t = int.parse(span.layer.id), key = start ? span.from : span.to, outside = start ? span.from - 1 : span.to + 1;
    m.later('together', number: 1);
    var told = 0;
    for (final (name, at, vs, k, now) in _ends(m, t, span, start, by)) {
      if (easeJump && outside >= 0 && !at.contains(outside)) {
        m.later('key', thing: t, name: name, word: '${outside / Ws.fps}', number: vs[k]);   // (the key's value before it moved: a frame off, what the value was there)
      }
      m.later('key', thing: t, name: name, word: '${key / Ws.fps}', number: now);
      told++;
    }
    ws.after(m.later('together'), (done, why) => mounted ? setState(() => _trouble = done ? (told == 0 ? 'No value keyed at both ends' : null) : why) : null);
  }

  // (what the end pulled [by] makes the first value keyed at both ends, as the inspector shows it: its name, its number
  //  in its units; After Effects' value graph shows the value of the point a hand pulls: golf_noise.md (70))
  String? _endSays(Ws ws, EaseSpan? span, bool start, double by) {
    final m = ws.inside;
    if (m == null || span == null) return null;
    final t = int.parse(span.layer.id);
    for (final (name, _, _, _, now) in _ends(m, t, span, start, by)) {
      final (k, f) = m.reading(t, name);
      final v = now * k + f, shown = m.shown(name);
      return '${shown?.label ?? name} ${v.toStringAsFixed(v.abs() >= 100 ? 0 : 2)}${shown?.unit ?? ''}';
    }
    return null;
  }

  // (the curve after every key of each picked key's layer, every value keyed there: one step)
  void _onEveryKey(Ws ws, Ease e) {
    final m = ws.inside!, layers = {for (final k in ws.keys) k.layer};
    m.later('together', number: 1);
    _define(m, said(e));
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
      if (easeOther(ease) && ease.drawn == null) ease = easeCard(said(ease), m);   // (no seat for this kind: its curve is the inside's own working of it, as said now)
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

  // The panel as AbsentMotion's EasingPanel.kt lays it, its widths its weights: the left column, the graph, the shapes.
  Widget _body(Ws ws, EaseSpan? span, Ease ease, ValueChanged<Ease> set) {
    final cards = easeIsLaid(said(ease)) ? [for (final c in easeCards(said(ease))) easeCard(c, ws.inside)] : [ease];
    Ease laidOn(Ease e) => easeLaid([...cards, for (final c in easeCards(said(e))) easeCard(c, ws.inside)]);
    final pile = _held == null ? cards : [for (final c in easeCards(said(_held!))) easeCard(c, ws.inside)];
    if (spanId(span) != _cardOf) (_cardOf = spanId(span), _card = 0);
    final at = _card.clamp(0, cards.length - 1), card = cards[at], laid = cards.length > 1;
    void put(Ease e) => set(laid ? easeLaid([...cards]..[at] = easeCard(said(e), ws.inside)) : e);   // (a press, a handle, a turn: on the card pressed)
    final bezier = easeIsBezier(card);
    final overshoot = bezier && !_noOver && (easeOvershoot || _overLocked || [card.p1.dy, card.p2.dy].any((y) => y < 0 || y > 1));
    _overLocked = overshoot;
    final turned = bezier ? card.reversed() : Ease(card.kind, card.p1, card.p2, easeTurn(said(card)));   // (AbsentMotion's reversed(): a bezier's handles turned, any other marked so)
    return Row(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Expanded(
          flex: 110,
          child: EaseLeft(
            onReverse: card != easePreset('Linear') && turned != card ? () => put(turned) : null,
            menu: [
              ('copy', 'Copy Curve', () => setState(() => easeCopied = ease)),
              if (easeCopied case final copied?) ...[
                ('paste', 'Paste Curve', () => set(copied)),
                if (ws.inside != null) ('all', 'Paste Curve to All Keyframes', () => _onEveryKey(ws, copied)),
              ],
              if (bezier) ('overshoot', overshoot ? 'Disable Overshoot' : 'Enable Overshoot', () => setState(() {
                easeOvershoot = !overshoot;
                _noOver = overshoot;
                _overLocked = !overshoot;
              })),
              ('jump', easeJump ? 'Jump On' : 'Jump Off', () => setState(() => easeJump = !easeJump)),
            ],
          ),
        ),
        Expanded(
          flex: 575 + 150,
          child: DragTarget<Object>(
            onWillAcceptWithDetails: (d) {
              if (d.data case final Ease e) setState(() => _held = laidOn(e));
              return true;   // (a card of the flow too: let go on it, it stays)
            },
            onLeave: (_) => setState(() => _held = null),
            onAcceptWithDetails: (d) {
              _held = null;
              if (d.data case final Ease e) {
                _card = cards.length;   // (the one just laid is the one in hand)
                set(laidOn(e));
              }
            },
            builder: (_, hot, _) => DecoratedBox(
              position: DecorationPosition.foreground,
              decoration: BoxDecoration(border: hot.isEmpty ? null : Border.all(color: WsT.accent, width: 2)),   // (as the stage shows a tile held over it)
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  Expanded(
                    flex: 575,
                    child: ColoredBox(
                      color: WsT.well,
                      child: Stack(
                        children: [
                          Positioned.fill(
                            child: ValueListenableBuilder<int>(valueListenable: ws.moment, builder: (_, frame, _) => EaseGraph(ease: card, onion: _held ?? (laid ? ease : null), span: span, frame: frame, onChanged: put, jump: easeJump, overshoot: overshoot, onEnd: ws.inside == null ? null : (start, by) => _moveEnd(ws, span, start, by), endSays: (start, by) => _endSays(ws, span, start, by))),   // (only the graph follows the playhead)
                          ),
                          if (span == null) Positioned.fill(child: Center(child: Padding(padding: EdgeInsets.all(Shape.s(8)), child: Text('Put the playhead between two keys, or pick a key', textAlign: TextAlign.center, style: T.label(Grey.g63))))),
                          if (_trouble case final why?) Positioned(left: Shape.s(4), right: Shape.s(4), top: Shape.s(2), child: Text(why, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(Grey.g63))),
                          if (span case final s?) Positioned(left: Shape.s(6), top: Shape.s(4), right: Shape.s(6), child: IgnorePointer(child: Text('${s.layer.name}  ${s.from} → ${s.to}', key: const ValueKey('ease-span'), maxLines: 1, overflow: TextOverflow.ellipsis, style: T.micro(Grey.g63)))),   // (which thing, which stretch)
                          Positioned(left: Shape.s(2), bottom: Shape.s(2), child: EaseStep(span: span, back: true)),
                          Positioned(right: Shape.s(2), bottom: Shape.s(2), child: EaseStep(span: span, back: false)),
                        ],
                      ),
                    ),
                  ),
                  Expanded(flex: 150, child: EaseCards(cards: pile, now: at, whole: pile.length > 1 ? _held ?? ease : null, onPress: (i) => setState(() => _card = i), onOff: (i) => set(easeLaid([...cards]..removeAt(i))), onMove: (from, to) {
                    final next = [...cards];
                    next.insert(to, next.removeAt(from));
                    _card = to;
                    set(easeLaid(next));
                  }, onLay: (e, i) {
                    _held = null;
                    _card = i;
                    set(easeLaid([...cards]..insertAll(i, [for (final c in easeCards(said(e))) easeCard(c, ws.inside)])));
                  }, onHold: (e, i) => setState(() => _held = e == null ? null : easeLaid([...cards]..insertAll(i, [for (final c in easeCards(said(e))) easeCard(c, ws.inside)]))))),
                ],
              ),
            ),
          ),
        ),
        Expanded(flex: 150, child: EaseShapes(ease: card, shapes: easeRow(unseated(ws)), onPick: put)),
      ],
    );
  }
}
