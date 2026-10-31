// The desk's Ease tool: the curve between the picked key and the next one, its two handles, presets as thumbnails, and the numbers.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:lab_book/motolii.dart';
import 'package:lab_book/workspace/desk_ease_state.dart';

import 'ws.dart';

enum EaseKind { bezier, hold }

/// One ease: a kind, and the two bezier handles (kept even while another kind shows, so grabbing a handle returns to bezier).
class Ease {
  const Ease(this.kind, this.p1, this.p2, [this.name, this.drawn]);
  final EaseKind kind;
  final Offset p1, p2;
  final String? name;

  /// A way with no seat on this desk: how it goes, as the document itself works it out (how far, at even steps of the stretch).
  final List<double>? drawn;

  double at(double x) => drawn != null ? _along(drawn!, x) : switch (kind) {
    EaseKind.bezier => bezierY(p1, p2, x),
    EaseKind.hold => x < 1 ? 0 : 1,
  };

  Ease withHandles(Offset a, Offset b) => Ease(EaseKind.bezier, a, b);
  Ease reversed() => Ease(kind, Offset(1 - p2.dx, 1 - p2.dy), Offset(1 - p1.dx, 1 - p1.dy), name);

  @override
  bool operator ==(Object other) => other is Ease && other.kind == kind && other.p1 == p1 && other.p2 == p2 && identical(other.drawn, drawn);
  @override
  int get hashCode => Object.hash(kind, p1, p2);
}

double _along(List<double> ys, double x) {
  final at = x.clamp(0.0, 1.0) * (ys.length - 1), i = at.floor().clamp(0, ys.length - 2);
  return ys[i] + (ys[i + 1] - ys[i]) * (at - i);
}

const easePresets = [
  Ease(EaseKind.bezier, Offset(0, 0), Offset(1, 1), 'Linear'),
  Ease(EaseKind.bezier, Offset(.25, .1), Offset(.25, 1), 'Ease'),
  Ease(EaseKind.bezier, Offset(.42, 0), Offset(1, 1), 'In'),
  Ease(EaseKind.bezier, Offset(0, 0), Offset(.58, 1), 'Out'),
  Ease(EaseKind.bezier, Offset(.42, 0), Offset(.58, 1), 'In-out'),
  Ease(EaseKind.bezier, Offset(.34, 1.56), Offset(.64, 1), 'Back'),
  Ease(EaseKind.hold, Offset(.25, .1), Offset(.25, 1), 'Hold'),
];

/// A way of the document that has no seat on this desk yet (Elastic, Random, Steps...): it is carried by its own words, as its name.
/// A preset by its name (never by its place in the row: a tile added anywhere leaves the others as they are).
Ease easePreset(String name) => easePresets.firstWhere((p) => p.name == name);

bool easeOther(Ease e) => e.name != null && easePresets.every((p) => p.name != e.name);
String? easeName(Ease e) => easeOther(e) ? e.name : easePresets.where((p) => p == e).firstOrNull?.name;

double bezierY(Offset p1, Offset p2, double x) {
  double bx(double u) => 3 * (1 - u) * (1 - u) * u * p1.dx + 3 * (1 - u) * u * u * p2.dx + u * u * u;
  var lo = 0.0, hi = 1.0;
  for (var i = 0; i < 24; i++) {
    final m = (lo + hi) / 2;
    if (bx(m) < x) {
      lo = m;
    } else {
      hi = m;
    }
  }
  final u = (lo + hi) / 2;
  return 3 * (1 - u) * (1 - u) * u * p1.dy + 3 * (1 - u) * u * u * p2.dy + u * u * u;
}

/// The stretch of time the ease shapes: the first picked key and the layer's next key.
typedef EaseSpan = ({WsLayer layer, int from, int to, int picked});

EaseSpan? easeSpan(Ws ws) {
  if (ws.keys.isEmpty) {   // (no key picked: the stretch the playhead is in, of the picked thing, as Alight Motion opens its ease when the playhead is between two keys: reports/alight_ui.md, イーズ)
    final layer = ws.layers.where((l) => l.id == ws.selected).firstOrNull, f = ws.frame;
    final keys = layer == null ? const <int>[] : ([...layer.keys]..sort());
    final from = keys.where((k) => k <= f).lastOrNull, to = keys.where((k) => k > f).firstOrNull;
    return from == null || to == null ? null : (layer: layer!, from: from, to: to, picked: 0);
  }
  final picked = ws.keys.toList()..sort((a, b) => a.frame.compareTo(b.frame));
  final k = picked.first, layer = ws.layers.where((l) => l.id == k.layer).firstOrNull;
  if (layer == null) return null;
  final keys = [...layer.keys]..sort();
  final next = keys.where((f) => f > k.frame).firstOrNull, prev = keys.where((f) => f < k.frame).lastOrNull;
  return next != null
      ? (layer: layer, from: k.frame, to: next, picked: picked.length)
      : (layer: layer, from: prev ?? math.max(0, k.frame - 30), to: k.frame, picked: picked.length);
}

/// The eases the user shaped, per document and per span, so leaving the desk and coming back keeps them.
final eases = Expando<Map<String, Ease>>();
String spanId(EaseSpan? s) => s == null ? 'new' : '${s.layer.id}:${s.from}';

class DeskEase extends StatefulWidget {
  const DeskEase({super.key});
  @override
  State<DeskEase> createState() => DeskEaseState();
}

/// Joined: an ease as the document says a stretch's way (three of its nine kinds have a seat on this desk), and back.
String said(Ease e) => easeOther(e) ? e.name! : switch (e.kind) {
  EaseKind.hold => 'Hold',
  EaseKind.bezier => e == easePreset('Linear') ? 'Linear' : 'Bezier ${e.p1.dx} ${e.p1.dy} ${e.p2.dx} ${e.p2.dy}',
};
Ease heard(String said) {
  // (the inside says a way back in one form: the kind's name, then all its numbers)
  final w = said.split(' '), n = [for (final x in w.skip(1)) double.tryParse(x) ?? 0];
  return switch (w.first) {
    'Hold' => easePreset('Hold'),
    'Linear' => easePreset('Linear'),
    'Bezier' when n.length == 4 => Ease(EaseKind.bezier, Offset(n[0], n[1]), Offset(n[2], n[3])),
    _ => Ease(EaseKind.bezier, Offset.zero, const Offset(1, 1), said), // (a kind with no seat here yet: its own words, over a straight line)
  };
}

/// Joined: the values of the span's thing that have a key where the span begins, each with that key's own moment.
Map<String, double> keysAt(Ws ws, EaseSpan span) {
  final m = ws.inside!, t = int.parse(span.layer.id);
  // (asked again only when the document is read again: not each picture a playing stage shows)
  if (_keysAtOf[ws] case (final r, final id, final f, final at) when r == ws.reads && id == span.layer.id && f == span.from) return at;
  final at = _keysAt(m, t, span);
  _keysAtOf[ws] = (ws.reads, span.layer.id, span.from, at);
  return at;
}
final _keysAtOf = Expando<(int, String, int, Map<String, double>)>();
Map<String, double> _keysAt(Motolii m, int t, EaseSpan span) {
  return {
    for (final name in m.moving(t))
      for (final s in m.keyMoments(t, name))
        if ((s * Ws.fps).round() == span.from) name: s,
  };
}

/// Joined: the ways of the document that the sample's eight tiles have no seat for (Elastic, Cyclic, Random, Steps,
/// ElasticSteps, and any defined in the document), each a tile of its own: its curve is the inside's own working of it.
/// (worked out again only when the document's ways are not the ones they were: a way defined since shows at once)
final _unseatedOf = Expando<(String, List<Ease>)>();
List<Ease> unseated(Ws ws) {
  final m = ws.inside;
  if (m == null) return const [];
  if (_unseatedOf[m] case (final was, final tiles) when was == '${ws.reads}') return tiles;   // (the ways asked again only when the document is read again)
  final names = m.wayNames, said = '${ws.reads}';
  final tiles = [
    for (final name in names)
      if (name != 'Bezier' && easePresets.every((p) => p.name != name)) Ease(EaseKind.bezier, Offset.zero, const Offset(1, 1), name, [for (var i = 0; i <= 32; i++) m.wayAt(name, i / 32)]),
  ];
  _unseatedOf[m] = (said, tiles);
  return tiles;
}
