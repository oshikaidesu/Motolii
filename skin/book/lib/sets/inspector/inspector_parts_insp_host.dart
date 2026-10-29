// Owns the mock document and re-provides it whenever it changes. A different kind or count makes a new document (give it a key).
// How it LOOKS is the lab's (DESIGN.md): hairlines, type carries hierarchy, grey levels, accent only C.mode. The old Motolii is not a source.
// Open questions are Cfg fields (knobs in inspector_set.dart); every behaviour below follows them.

import 'package:flutter/foundation.dart' show ValueListenable, listEquals, mapEquals, setEquals;
import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart';

/// Owns the mock document and re-provides it whenever it changes. A different kind or count makes a new document (give it a key).
class InspHost extends StatefulWidget {
  const InspHost({
    super.key,
    required this.cfg,
    required this.kind,
    this.count = 1,
    this.effects = 2,
    this.fx,
    this.kindWord,
    this.seats,
    this.onFxRemove,
    this.onMidi,
    this.midiBpm,
    this.onFxMove,
    this.movable = const [],
    this.onCardMove,
    this.onCardReset,
    this.onCardRemove,
    this.onCardEye,
    this.onCardSave,
    this.onCardInfo,
    this.onCardDuplicate,
    this.cardsOff = const [],
    this.onCardCopy,
    this.onCardPaste,
    this.onPath,
    this.cards,
    this.title,
    this.onRoute,
    this.onAim,
    this.values = const {},
    this.strs = const {},
    this.keyMarks = const {},
    this.onValues,
    this.onDrop,
    this.onKey,
    this.onWords,
    this.onAnimateAll,
    this.animating,
    this.flags = const {},
    this.onFlags,
    this.choices = const {},
    this.together,
    this.live,
    required this.child,
  });
  final Cfg cfg;
  final Kind kind;
  final int count, effects;
  final Widget child;

  /// The effects on the layer, as the host has them (each one's card from its Params); null: the sample's first [effects].
  final List<Fx>? fx;

  /// What the host calls the kind (see [Doc.kindWord]), and the values it has (see [Doc.seats]).
  final String? kindWord;
  final Set<String>? seats;

  /// The values and key marks at a new moment, told without the host being built again (played, only the moment moves:
  /// the fields that show them follow, the rest of the inspector stays as it was built).
  final ValueListenable<(Map<String, double>, Map<String, KeyS>)>? live;

  /// The host's say on an effect taken off, and on their order (see [Doc.onFxRemove]).
  final bool Function(String id)? onFxRemove;

  /// The host's MIDI keys and a file's BPM (see [Doc.onMidi]).
  final bool Function(String prop, String file, String kind, int channel, double bpm)? onMidi;
  final double Function(String file)? midiBpm;
  final bool Function()? onFxMove;

  /// The host's effects as cards that move up or down its stack (see [Doc.movable]).
  final List<String> movable;
  final void Function(String title, int by)? onCardMove;
  final void Function(List<String> names)? onCardReset;   // (a card's values as they come: Reset on its head)
  final void Function(String title)? onCardRemove;   // (one of its effects taken off: Remove on its card's head, Alight Motion's Delete)
  final void Function(String title)? onCardEye;   // (one of its effects turned off or on again, its values kept: Alight Motion's eye on an effect)
  final void Function(String title, List<String> names)? onCardSave;   // (an effect's values, keys and all, kept on the shelf: Alight Motion's preset of one effect)
  final void Function(String title)? onCardInfo;   // (what an effect does, in its maker's words: AbsentMotion's Effect Info)
  final void Function(String title)? onCardDuplicate;   // (the effect put on again after it, its values with it: Alight Motion's Duplicate)
  final List<String> cardsOff;   // (the titles of those turned off)
  final void Function(List<String> names)? onCardCopy;   // (a card's values copied, to be given to another thing: Copy on its head)
  final VoidCallback? onCardPaste;   // (and given: Paste)

  /// How the thing moves over a while, as the host places it (see [Doc.onPath]).
  final Future<List<double>?> Function(double to, int n, int copy)? onPath;   // (a question of the host's: answered later)


  /// The host's own cards of values (see [Doc.cards]).
  final List<Fx>? cards;

  /// The selected layer's name, when the host knows it.
  final String? title;

  /// A panel link asked for a panel (`Browser › Fonts`, `Ease`); the host opens it.
  final ValueChanged<String>? onRoute;

  /// The slot a panel link was aimed at when it asked for its panel ('' : none), told just before [onRoute].
  final ValueChanged<String>? onAim;

  /// Values the host owns (the layer's transform at this frame): written in silently when they change, so edits made here last until then.
  final Map<String, double> values;
  final Map<String, String> strs;

  /// The key mark of each property (pos, scale, rot ...) as the host's document has it.
  final Map<String, KeyS> keyMarks;

  /// Values changed in the inspector: [live] while a hand is still on them (not settled), then once more when it lets go.
  final void Function(Map<String, double> changed, {required bool live})? onValues;

  /// A hand let go without keeping what it did (Esc): what it showed is to be dropped.
  final VoidCallback? onDrop;

  /// A property's key mark was changed in the inspector.
  final void Function(String prop, KeyS mark)? onKey;

  /// Words changed in the inspector (a text's content, a chooser's choice). false: the host did not take them, and they go back to what they were.
  final bool Function(Map<String, String> changed)? onWords;

  /// Animate all was turned on or off (the host's document may key whatever a hand changes, wherever the hand is).
  final ValueChanged<bool>? onAnimateAll;

  /// The host's own Animate all (null: the inspector keeps its own): shown as it is, never reset by a new kind.
  final bool? animating;

  /// The on/off seats the host owns (Clip to below ...), and a change of them told to it. false: the host did not
  /// take it (nothing is behind that seat), and the seat goes back.
  final Map<String, bool> flags;
  final bool Function(Map<String, bool> changed)? onFlags;

  /// What a chooser may offer, by its id, where the host's document has its own words for it (a blend's).
  final Map<String, List<String>> choices;

  /// What one change of the inspector tells the host (values, then key marks) is told inside this, so a host can keep it as one act.
  final void Function(VoidCallback tells)? together;
  @override
  State<InspHost> createState() => _InspHostState();
}

class _InspHostState extends State<InspHost> {
  late Doc doc = _make();
  Doc _make() {
    final d = Doc(widget.kind, count: widget.count, effects: widget.effects, fx: widget.fx, cards: widget.cards ?? const [])
      ..title = widget.title
      ..marksEdits = widget.onKey == null   // (a host told of keys keys what a hand changes itself)
      ..kindWord = widget.kindWord
      ..seats = widget.seats
      ..onFxRemove = widget.onFxRemove
      ..onMidi = widget.onMidi
      ..midiBpm = widget.midiBpm
      ..onFxMove = widget.onFxMove
      ..movable = widget.movable
      ..onCardMove = widget.onCardMove
      ..onCardReset = widget.onCardReset
      ..onCardRemove = widget.onCardRemove
      ..onCardEye = widget.onCardEye
      ..onCardSave = widget.onCardSave
      ..onCardInfo = widget.onCardInfo
      ..onCardDuplicate = widget.onCardDuplicate
      ..cardsOff = widget.cardsOff
      ..onCardCopy = widget.onCardCopy
      ..onCardPaste = widget.onCardPaste
      ..onPath = widget.onPath
      ..v.addAll(widget.values)
      ..s2.addAll(widget.strs)
      ..choices.addAll(widget.choices)
      ..b.addAll(widget.flags)
      ..keys.addAll(widget.keyMarks);
    _seen
      ..clear()
      ..addAll(d.v);
    _seenKeys
      ..clear()
      ..addAll(d.keys);
    _seenWords
      ..clear()
      ..addAll(d.s2);
    _seenFlags
      ..clear()
      ..addAll(d.b);
    if (widget.animating case final on?) {
      d.animateAll = _animating = on;   // (the host's: the inspector shows it)
    } else {
      widget.onAnimateAll?.call(_animating = d.animateAll); // (a new document of the inspector begins with it off: the host is told)
    }
    return d..addListener(_r);
  }

  // what the host has been told of (or gave): only what differs from it is told
  final _seen = <String, double>{};
  final _seenKeys = <String, KeyS>{};
  final _seenWords = <String, String>{};
  final _seenFlags = <String, bool>{};
  bool _live = false;
  void _tell() => widget.together == null || doc.inGesture ? _tells() : widget.together!(_tells); // (under a hand only where it is is told, nothing settled: no step to make one of)
  bool _animating = false;
  void _tells() {
    if (doc.animateAll != _animating) widget.onAnimateAll?.call(_animating = doc.animateAll);
    final live = doc.inGesture, changed = {
      for (final e in doc.v.entries)
        if (_seen[e.key] != e.value) e.key: e.value,
    };
    if (changed.isNotEmpty) {
      if (!live) _seen.addAll(changed);
      widget.onValues?.call(changed, live: live);
    } else if (_live && !live) {
      widget.onDrop?.call();
    }
    _live = live;
    for (final e in doc.keys.entries.toList()) {
      if (live || _seenKeys[e.key] == e.value) continue; // (while a hand is on it nothing is settled: told when it lets go)
      _seenKeys[e.key] = e.value;
      widget.onKey?.call(e.key, e.value);
    }
    final words = {
      for (final e in doc.s2.entries)
        if (e.key != 'route' && e.key != 'route.at' && _seenWords[e.key] != e.value) e.key: e.value,
    };
    if (widget.onFlags case final tell?) {
      final flags = {
        for (final e in doc.b.entries)
          if ((_seenFlags[e.key] ?? false) != e.value) e.key: e.value,
      };
      if (flags.isNotEmpty) {
        if (tell(flags)) {
          _seenFlags.addAll(flags);
        } else {
          for (final id in flags.keys) {
            doc.b[id] = _seenFlags[id] ?? false;
          }
        }
      }
    }
    if (words.isEmpty) return;
    if (widget.onWords?.call(words) ?? true) {
      _seenWords.addAll(words);
    } else {
      for (final id in words.keys) {
        _seenWords.containsKey(id) ? doc.s2[id] = _seenWords[id]! : doc.s2.remove(id);
      }
    }
  }

  void _r() {
    _tell();
    final r = doc.s2['route'], go = widget.onRoute;
    if (r != null && go != null) {
      doc.s2.remove('route');
      widget.onAim?.call(doc.s2.remove('route.at') ?? '');
      go(r);
    }
    // (while a hand is on a value the whole inspector is not built again each step: the field under the hand shows its
    //  own number and the stage follows what the host is told; everything is built once when the hand lets go)
    if (mounted && !doc.inGesture && !doc.ending) setState(() => _tick++);
  }

  @override
  void didUpdateWidget(InspHost o) {
    super.didUpdateWidget(o);
    if (o.live != widget.live) {
      o.live?.removeListener(_atMoment);
      widget.live?.addListener(_atMoment);
    }
    if (o.kind != widget.kind || o.count != widget.count || o.effects != widget.effects || !listEquals(o.fx?.map((f) => f.wgsl).toList(), widget.fx?.map((f) => f.wgsl).toList()) || !listEquals(o.cards?.map((f) => f.name + f.wgsl.replaceAll(RegExp(r'@default\([^)]*\)'), '')).toList(), widget.cards?.map((f) => f.name + f.wgsl.replaceAll(RegExp(r'@default\([^)]*\)'), '')).toList())) {
      doc.dispose();
      doc = _make();
      _tick++;
    } else {
      // A host that is told of changes owns the values: whatever it did not take goes back to what it says, as soon
      // as no hand is on them (never under a hand: a value being dragged is not written over, even while it plays).
      // (numbers and key marks: told to the fields that show them; words, flags, choices and the rest: everything again)
      final owned = widget.onValues != null;
      if (owned ? !doc.inGesture : !mapEquals(o.values, widget.values)) {
        if (doc.showAt(widget.values, const {})) _tick++;   // (a value no field watches changed: shown by building again)
        _seen.addAll(widget.values);
      }
      if (widget.onFlags != null) {
        if (widget.flags.entries.any((e) => doc.b[e.key] != e.value)) _tick++;
        doc.b.addAll(widget.flags);
        _seenFlags.addAll(widget.flags);
      }
      if (widget.onWords != null || !mapEquals(o.strs, widget.strs)) {
        if (widget.strs.entries.any((e) => doc.s2[e.key] != e.value)) _tick++;
        doc.s2.addAll(widget.strs);
        _seenWords.addAll(widget.strs);
      }
      if (!mapEquals(o.keyMarks, widget.keyMarks)) {
        doc.showAt(const {}, widget.keyMarks);
        _seenKeys.addAll(widget.keyMarks);
      }
    }
    if (widget.animating case final on? when on != _animating) {
      doc.animateAll = _animating = on;
      _tick++;
    }
    if (o.title != widget.title || o.kindWord != widget.kindWord || !setEquals(o.seats, widget.seats) || o.choices.toString() != widget.choices.toString()) _tick++;
    doc
      ..title = widget.title
      ..kindWord = widget.kindWord
      ..seats = widget.seats
      ..onFxRemove = widget.onFxRemove
      ..onMidi = widget.onMidi
      ..midiBpm = widget.midiBpm
      ..onFxMove = widget.onFxMove
      ..movable = widget.movable
      ..onCardMove = widget.onCardMove
      ..onCardReset = widget.onCardReset
      ..onCardRemove = widget.onCardRemove
      ..onCardEye = widget.onCardEye
      ..onCardSave = widget.onCardSave
      ..onCardInfo = widget.onCardInfo
      ..onCardDuplicate = widget.onCardDuplicate
      ..cardsOff = widget.cardsOff
      ..onCardCopy = widget.onCardCopy
      ..onCardPaste = widget.onCardPaste
      ..onPath = widget.onPath;
    doc.choices
      ..clear()
      ..addAll(widget.choices);
  }

  void _atMoment() {
    final (values, marks) = widget.live!.value;
    if (doc.inGesture) return; // (never under a hand)
    _seen.addAll(values);
    _seenKeys.addAll(marks);
    if (doc.showAt(values, marks) && mounted) setState(() => _tick++); // (a value no field watches, a choice, changed: shown by building again)
  }

  @override
  void initState() {
    super.initState();
    widget.live?.addListener(_atMoment);
  }

  @override
  void dispose() {
    widget.live?.removeListener(_atMoment);
    doc.dispose();
    super.dispose();
  }

  int _tick = 0;
  @override
  Widget build(BuildContext context) => Ctx(cfg: widget.cfg, doc: doc, tick: _tick, child: widget.child);
}
