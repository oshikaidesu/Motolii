// Joined: a relation made of the things picked together (Ws.many): a behaviour of that kind (the inside's around, row,
// follow or push) with each of them connected by its place (x_by, y_by), one step. Its middle, size and gap are where
// they stand now, so nothing moves but what the relation itself does: round a ring, into a row; a follower keeps its
// distance from its leader (the first from the left), a pushed thing moves only near the pusher (the largest).
import 'dart:math' as math;

import 'ws.dart';

const wsRelations = ['around', 'row', 'follow', 'push'];

void wsRelate(Ws ws, String kind) {
  final m = ws.inside;
  if (m == null) return;
  final t = ws.frame / Ws.fps;
  final things = [for (final s in ws.many) int.tryParse(s)].whereType<int>().toList()..sort();
  if (things.length < 2) return;
  final x = {for (final i in things) i: m.value(i, 'x', t) ?? 0.0}, y = {for (final i in things) i: m.value(i, 'y', t) ?? 0.0};
  final mx = things.map((i) => x[i]!).reduce((a, b) => a + b) / things.length, my = things.map((i) => y[i]!).reduce((a, b) => a + b) / things.length;
  final left = things.reduce((a, b) => x[a]! <= x[b]! ? a : b);
  final big = things.reduce((a, b) => (m.value(a, 'size', t) ?? 1) >= (m.value(b, 'size', t) ?? 1) ? a : b);
  final leader = switch (kind) { 'follow' => left, 'push' => big, _ => 0 };
  m.later('together', number: 1);   // (one step, asked without waiting: the behaviour is the thing the add makes)
  var made = 0;
  ws.after(m.later('add', name: 'behaviour', word: kind), null, (b) => made = b);
  void knob(String name, double v) => m.later('set', thing: -1, name: name, number: v);
  switch (kind) {
    case 'around':
      final r = things.map((i) => math.sqrt(math.pow(x[i]! - mx, 2) + math.pow(y[i]! - my, 2))).reduce((a, b) => a + b) / things.length;
      knob('centre_x', mx);
      knob('centre_y', my);
      knob('radius', math.max(r, .2));
    case 'row':
      final xs = things.map((i) => x[i]!).toList()..sort();
      knob('centre_x', xs.first);
      knob('centre_y', my);
      knob('gap', (xs.last - xs.first) / (things.length - 1));
    case 'follow':
      knob('lag', .1);
    case 'push':
      knob('radius', .5);
  }
  if (leader != 0) m.later('set', thing: -1, name: 'leader', number: leader.toDouble());
  // (their order in it: round the middle as they lie (no two cross), or from the left; a place kept when one is taken out)
  final order = [...things]..sort((a, b) => kind == 'around' ? math.atan2(x[a]! - mx, y[a]! - my).compareTo(math.atan2(x[b]! - mx, y[b]! - my)) : x[a]!.compareTo(x[b]!));
  m.later('say', thing: -1, name: 'members', word: order.where((i) => i != leader).join(' '));
  for (final i in things) {
    if (i == leader) continue;
    if (kind == 'follow') {   // (its place a step from its leader's: where it stands)
      m
        ..later('set', thing: i, name: 'x', number: x[i]! - x[leader]!)
        ..later('set', thing: i, name: 'y', number: y[i]! - y[leader]!);
    }
    m
      ..later('set', thing: i, name: 'x_by', word: 'made')
      ..later('set', thing: i, name: 'y_by', word: 'made');
  }
  ws.after(m.later('together'), (done, _) {
    if (!done || made <= 0) return;
    ws.reread();
    ws.select('$made');   // (the relation picked: its card, its row)
  });
}

/// The things a relation moves (any of their values connected to it), when the picked thing is a relation; else none.
/// (Read through the window's copy of the document: asked of the inside again only once it has changed.)
Set<String> wsMembersOf(Ws ws) {
  final m = ws.inside, sel = int.tryParse(ws.selected ?? '');
  if (m == null || sel == null || m.word(sel, 'kind') != 'behaviour' || !wsRelations.contains(m.word(sel, 'file'))) return const {};
  return {for (final t in m.things) if (t != sel && ((m.value(t, 'x_by', 0) ?? 0) == sel || (m.value(t, 'y_by', 0) ?? 0) == sel)) '$t'};
}

/// A thing added to those picked together (Shift or Cmd with a click): the one picked alone goes in with it.
void wsPickAlso(Ws ws, String id) {
  ws.pickMany({...ws.many, if (ws.selected case final s? when !wsMembersOf(ws).contains(s) && ws.many.isEmpty) s, id});
}

/// One thing out of the picked relation, where it shows now (the others stay where they are).
void wsFree(Ws ws, String id) {
  final m = ws.inside, by = int.tryParse(ws.selected ?? ''), t = int.tryParse(id);
  if (m == null || by == null || t == null) return;
  ws.after(m.later('free', thing: t, number: by.toDouble()), (done, _) => done ? ws.reread() : null);
}

/// The picked relation let go: each of its things where it shows now, the relation gone.
void wsRelease(Ws ws) {
  final m = ws.inside, by = int.tryParse(ws.selected ?? '');
  if (m == null || by == null) return;
  ws.after(m.later('release', thing: by), (done, _) {
    if (!done) return;
    ws.reread();
    ws.select(null);
  });
}
