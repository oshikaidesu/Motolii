// The desk's Ease tool: the curve between the picked key and the next one, its two handles, presets as thumbnails, and the numbers.

import 'dart:async' show scheduleMicrotask;
import 'dart:math' as math;

import 'package:flutter/services.dart' show Clipboard, ClipboardData;
import 'package:flutter/widgets.dart';
import 'package:lab_book/motolii.dart';
import 'package:lab_book/tokens.dart';
import 'package:lab_book/workspace/desk_ease.dart';
import 'package:lab_book/workspace/desk_ease_cards.dart';
import 'package:lab_book/workspace/desk_ease_graph.dart';
import 'package:lab_book/workspace/desk_ease_numbers.dart';
import 'package:lab_book/workspace/desk_ease_shelf.dart';
import 'package:lab_book/workspace/desk_ease_side.dart';
import 'package:lab_book/workspace/ws.dart';

part 'desk_ease_ends.dart';

/// The curve copied in the Ease desk, kept while the window is open (Alight Motion's Copy of a curve).
Ease? easeCopied;

/// Whether an end of a stretch pulled up or down jumps there: a key a frame outside keeps the value as it was, so the
/// value jumps at the key (three points, as the user drew it, 10/30); off, only the key's value moves and the stretch
/// beside follows. The user's to choose, on the desk (10/30: "飛びたくないなら後でおしりの点を消せばいい、もしくは設定で
/// 決めれると良い、ユーザに託す", "easeにon/offであるといい"). Kept while the window is open.
bool easeJump = true;

/// Whether the graph looks past 0 and 1 (Alight Motion's Enable Overshoot). Kept while the window is open.
bool easeOvershoot = false;

class DeskEaseState extends State<DeskEase> with _EaseEnds {
  // Joined: while a hand is down the ease is told to the document as it goes (the stage and the bars follow), the newest
  // one at a time (one still on its way: the next waits, those between are skipped); all of it one step of undo.
  bool _down = false, _open = false, _going = false;
  Ease? _pending, _waiting;
  Ease? _told;   // (told the document and not yet heard back: shown meanwhile, not what it was)
  int _card = 0;   // (of a way laid from cards, the one the graph shows, below 0 the whole; another stretch: its first)
  String _cardOf = '';
  Ease? _held;   // (what a tile held over the graph would make: seen before it is let go, Final Cut Pro's skimming of an effect)
  bool _adding = false;   // (the flow's empty frame pressed: what is picked next is laid there, as what is picked from Bitwig's + goes in at it)
  Ease? _base;   // (the way as it was: while one is held the stage shows it in an open step, and nothing is kept unless it is let go there)
  bool _shelf = false, _shelfShown = false;   // (drawn out; and built, until it has slid away)
  bool _overLocked = false, _noOver = false;   // (AbsentMotion's lockOvershoot, disableOvershoot: once it looks past 0 and 1 it stays so, until the menu says)

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
    for (final t in {for (final (t, _, _) in all) t}) {
      last = m.later('show', thing: t);   // (the stage, the bars and the numbers go with the hand, the step still open)
    }
    ws.after(last, (_, _) {
      _going = false;
      final next = _waiting;
      _waiting = null;
      if (next != null) _live(ws, next);
    });
  }

  // (a try on the stage given up: the step it opened goes whole, so nothing is kept, not even a step)
  void _giveUp(Ws ws) {
    _waiting = null;
    if (!_open) {
      _base = null;
      return;
    }
    _open = false;
    ws.after(ws.inside!.later('together', number: -1), (_, _) => mounted ? setState(() => _base = null) : null);
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
    if (e == null && _base != null) {   // (a tile tried and let go: on the graph or the flow it is told in this same event)
      scheduleMicrotask(() => _pending == null && !_down ? _giveUp(ws) : null);
      return;
    }
    _base = null;
    if (e == null || span == null) return;
    _told = e;
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

  // (the curve after every key of each picked key's layer, every value keyed there: one step)
  void _onEveryKey(Ws ws, Ease e) {
    final m = ws.inside!, layers = ws.keys.isNotEmpty ? {for (final k in ws.keys) k.layer} : {if (easeSpan(ws) case final s?) s.layer.id};   // (none picked: the layer of the stretch the playhead is in)
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
      final inDocument = at.isEmpty ? easePreset('Linear') : heard(m.wayOf(int.parse(span!.layer.id), at.keys.first, at.values.first));
      if (_told case final t? when !ws.waiting || said(t) == said(inDocument)) _told = null;
      var ease = _pending ?? _told ?? _base ?? inDocument;
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
    // (on a straight line alone there is nothing to lay it on: it goes there as a press would)
    Ease laidOn(Ease e) => said(ease) == 'Linear' ? e : easeLaid([...cards, for (final c in easeCards(said(e))) easeCard(c, ws.inside)]);
    final pile = _held == null ? cards : [for (final c in easeCards(said(_held!))) easeCard(c, ws.inside)];
    if (spanId(span) != _cardOf) (_cardOf = spanId(span), _card = 0, _adding = false);
    final at = _card.clamp(0, cards.length - 1), card = cards[at], laid = cards.length > 1, whole = laid && _card < 0;
    void put(Ease e) => set(laid ? easeLaid([...cards]..[at] = easeCard(said(e), ws.inside)) : e);   // (a press, a handle, a turn: on the card pressed)
    final bezier = easeIsBezier(card);
    final overshoot = bezier && !_noOver && (easeOvershoot || _overLocked || [card.p1.dy, card.p2.dy].any((y) => y < 0 || y > 1));
    _overLocked = overshoot;
    // (where the beats fall on the card in hand: one that takes what is above it through it is walked by that, not by the
    //  clock, so a beat is where that has come to then, or a handle put on a line would miss the beat in the film)
    final grid = ws.beats, clock = !whole && at > 0 && easeThrough(card) ? easeLaid(cards.sublist(0, at)) : null;
    final beats = grid == null || span == null ? const <double>[] : [
      for (final (f, _) in grid.between(span.from + 1, span.to - 1))
        if ((f - span.from) / (span.to - span.from) case final u) clock?.at(u).clamp(0.0, 1.0) ?? u,
    ];
    void add(Ease e) {
      _adding = false;
      _card = cards.length;
      set(laidOn(e));
    }
    void hold(Ease? e) {   // (on the stage too, as the graph shows it)
      setState(() => _held = e);
      if (ws.inside == null) return;
      if (e == null) return _giveUp(ws);
      _base ??= ease;
      _live(ws, e);
    }
    Future<void> paste(void Function(Ease) put) async {
      ClipboardData? words;
      try {
        words = await Clipboard.getData(Clipboard.kTextPlain);
      } catch (_) {}   // (no clipboard to ask: what was copied here)
      final e = easeFromText(words?.text, ws.inside) ?? easeCopied;
      if (e != null) {
        put(e);
      } else {
        ws.notice = 'No curve to paste';
      }
    }
    final turned = bezier ? card.reversed() : Ease(card.kind, card.p1, card.p2, easeTurn(said(card)));   // (AbsentMotion's reversed(): a bezier's handles turned, any other marked so)
    return LayoutBuilder(builder: (_, box) {
      // (the left column as wide as its two marks; the shapes as large as the height lets seven and a bit be, or the width
      //  leaves, the two columns round them; all the rest the graph: a pane of any size has no empty part)
      //  In a tall pane the two go under the graph, or it would be a thin post: what a card does in its container, as CSS's
      //  container queries have it.
      final left = Shape.z(34), tall = box.maxHeight > box.maxWidth * 1.1;
      final side = (tall
              ? math.min(box.maxWidth / easeFits(box.maxWidth) - Shape.s(4), (box.maxHeight * .4) / 2 - Shape.s(12))
              : math.min(box.maxHeight / easeFits(box.maxHeight) - Shape.s(6), (box.maxWidth * .55 - left) / 2 - Shape.s(12)))
          .clamp(Shape.z(16), Shape.z(64));
      final column = side + Shape.s(12);
      final leftColumn = SizedBox(
          width: left,
          child: EaseLeft(
            onReverse: !whole && card != easePreset('Linear') && turned != card ? () => put(turned) : null,
            menu: [
              ('copy', 'Copy Curve', () {
                setState(() => easeCopied = ease);
                Clipboard.setData(ClipboardData(text: easeText(ease))).catchError((_) {});   // (as words: for a page, or another window)
              }),
              ('paste', 'Paste Curve', () => paste(set)),   // (what was copied here, or a cubic-bezier() or linear() from a page: easeFromText)
              if (easeCopied != null && ws.inside != null) ('all', 'Paste Curve to All Keyframes', () => paste((e) => _onEveryKey(ws, e))),
              if (bezier) ('overshoot', overshoot ? 'Disable Overshoot' : 'Enable Overshoot', () => setState(() {
                easeOvershoot = !overshoot;
                _noOver = overshoot;
                _overLocked = !overshoot;
              })),
              ('jump', easeJump ? 'Jump On' : 'Jump Off', () => setState(() => easeJump = !easeJump)),
            ],
          ),
        );
      void shelf(bool open) => setState(() {
        _shelf = open;
        _shelfShown = _shelfShown || open;
      });
      void keep(int i) {
        final word = said(i < 0 ? easeLaid(cards) : cards[i]);
        if (word != 'Linear') setState(() => easeKeep(word));
      }
      final shapes = DragTarget<int>(
        onWillAcceptWithDetails: (_) {
          shelf(true);
          return true;
        },
        onAcceptWithDetails: (d) => keep(d.data),
        builder: (_, _, _) => EaseShapes(ease: whole ? ease : card, side: side, across: tall, shapes: easeRow(unseated(ws)), onPick: (e) {
          if (_adding) return add(e);
          if (whole) {   // (the whole in hand: it takes the place of the whole, as a press takes that of a card)
            _card = 0;
            return set(e);
          }
          put(e);
        }),
      );
      final target = DragTarget<Object>(
            onWillAcceptWithDetails: (d) {
              if (d.data case final Ease e) hold(laidOn(e));
              return true;   // (a card of the flow too: let go on it, it stays)
            },
            onLeave: (d) => d is Ease ? hold(null) : null,
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
              child: Flex(
                direction: tall ? Axis.vertical : Axis.horizontal,
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  Expanded(
                    child: ColoredBox(
                      color: WsT.well,
                      child: Stack(
                        children: [
                          Positioned.fill(
                            child: ValueListenableBuilder<int>(valueListenable: ws.moment, builder: (_, frame, _) => EaseGraph(beats: beats, ease: whole ? easeLaid(cards) : card, onion: _held ?? (laid && !whole ? easeLaid(cards) : null), ends: _endsOf(ws, span), span: span, frame: frame, onChanged: put, jump: easeJump, overshoot: overshoot && !whole, onEnd: ws.inside == null ? null : (start, by) => _toEnd(ws, span, start, by, done: true), onEndMove: ws.inside == null ? null : (start, by) => _toEnd(ws, span, start, by), endSays: (start, by) => _endSays(ws, span, start, by))),   // (only the graph follows the playhead)
                          ),
                          if (span == null) Positioned.fill(child: Center(child: Padding(padding: EdgeInsets.all(Shape.s(8)), child: Text('Put the playhead between two keys, or pick a key', textAlign: TextAlign.center, style: T.label(Grey.g63))))),
                          if (_trouble case final why?) Positioned(left: Shape.s(4), right: Shape.s(4), top: Shape.s(2), child: Text(why, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(Grey.g63))),
                          if (span case final s?) Positioned(left: Shape.s(6), top: Shape.s(4), right: Shape.s(6), child: IgnorePointer(child: Text('${s.layer.name}  ${s.from} → ${s.to}', key: const ValueKey('ease-span'), maxLines: 1, overflow: TextOverflow.ellipsis, style: T.micro(Grey.g63)))),   // (which thing, which stretch)
                          Positioned.fill(child: EaseNumbers(card: whole ? easeLaid(cards) : card, onChanged: put)),
                          Positioned(left: Shape.s(2), bottom: Shape.s(2), child: EaseStep(span: span, back: true)),
                          Positioned(right: Shape.s(2), bottom: Shape.s(2), child: EaseStep(span: span, back: false)),
                        ],
                      ),
                    ),
                  ),
                  SizedBox(width: tall ? null : column, height: tall ? column : null, child: EaseCards(side: side, across: tall, cards: pile, adding: _adding, onSlot: () => setState(() => _adding = !_adding), now: whole ? -1 : at, whole: pile.length > 1 ? _held ?? easeLaid(cards) : null, onWhole: () => setState(() => _card = -1), onPress: (i) => setState(() => (_card = i, _adding = false)), onOff: (i) => set(cards.length > 1 ? easeLaid([...cards]..removeAt(i)) : easePreset('Linear')), onMove: (from, to) {
                    final next = [...cards];
                    next.insert(to, next.removeAt(from));
                    _card = to;
                    set(easeLaid(next));
                  }, onLay: (e, i) {
                    _held = null;
                    _card = i;
                    set(easeLaid([...cards]..insertAll(i, [for (final c in easeCards(said(e))) easeCard(c, ws.inside)])));
                  }, onHold: (e, i) => hold(e == null ? null : easeLaid([...cards]..insertAll(i, [for (final c in easeCards(said(e))) easeCard(c, ws.inside)]))))),
                ],
              ),
            ),
          );
      final across = (box.maxWidth * .3 / column).floor().clamp(1, 6), wide = across * column;
      const slide = Duration(milliseconds: 200);   // (as the colours' boxes move, dock_make_2.dart)
      return Stack(
        clipBehavior: Clip.hardEdge,
        children: [
          Positioned.fill(
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                Expanded(
                  child: tall
                      ? Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
                          Expanded(child: Row(crossAxisAlignment: CrossAxisAlignment.stretch, children: [leftColumn, Expanded(child: target)])),
                          SizedBox(height: column, child: shapes),
                        ])
                      : Row(crossAxisAlignment: CrossAxisAlignment.stretch, children: [leftColumn, Expanded(child: target), SizedBox(width: column, child: shapes)]),
                ),
                ClipRect(
                  child: AnimatedContainer(
                    duration: slide,
                    curve: Curves.easeOutCubic,
                    width: _shelf ? wide : 0,
                    onEnd: () => _shelf ? null : setState(() => _shelfShown = false),   // (put away: not built as a hand moves a handle)
                    child: !_shelfShown
                        ? null
                        : OverflowBox(
                            alignment: Alignment.centerLeft,
                            minWidth: wide,
                            maxWidth: wide,
                            child: EaseShelf(now: said(ease), ways: easeOthers(unseated(ws)), columns: across, pitch: side + Shape.s(6), side: side, inside: ws.inside, onKeep: keep, onForget: (e) => setState(() => easeKeep(said(e), off: true)), onPick: (e) {
                              if (_adding) return add(e);
                              _card = 0;   // (a preset is the whole way)
                              set(e);
                            }),
                          ),
                  ),
                ),
              ],
            ),
          ),
          AnimatedPositioned(
            duration: slide,
            curve: Curves.easeOutCubic,
            top: Shape.s(8),
            right: _shelf ? wide : 0,
            child: EaseGrip(key: const ValueKey('ease.shelf'), open: _shelf, onTap: () => shelf(!_shelf)),
          ),
        ],
      );
    });
  }
}
