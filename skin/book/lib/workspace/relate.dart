// Joined: a relation made of the things picked together (Ws.many): a behaviour of that kind (the inside's around, row,
// follow or push) and the things put in it as they show now (the inside's relate: their order, its middle and size,
// a leader, what each was in before), one step, picked once made.
import 'dart:math' as math;

import 'ws.dart';

const wsRelations = ['around', 'row', 'follow', 'push'];

void wsRelate(Ws ws, String kind) {
  final m = ws.inside;
  if (m == null || ws.many.length < 2) return;
  m.later('together', number: 1);   // (one step, asked without waiting: the behaviour is the thing the add makes)
  var made = 0;
  ws.after(m.later('add', name: 'behaviour', word: kind), null, (b) => made = b);
  m
    ..later('set', thing: -1, name: 'strength', number: 1)   // (said, so its card shows it as it works)
    ..later('relate', thing: -1, word: ws.many.join(' '));
  ws.after(m.later('together'), (done, _) {
    ws.reread();   // (made or not, the window shows what the document is)
    if (!done) return;
    final newest = made > 0 ? made : m.things.where((t) => m.word(t, 'kind') == 'behaviour').fold(0, math.max);
    if (newest > 0) ws.select('$newest');   // (the relation picked: its card, its row, its shape)
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
  if (ws.many.contains(id)) return ws.pickMany({...ws.many}..remove(id));   // (picked already: one less)
  ws.pickMany({...ws.many, if (ws.selected case final s? when !wsMembersOf(ws).contains(s) && ws.many.isEmpty && int.tryParse(s) != null) s, id});
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

/// The relations the picked thing is in, the one on top first (what its place is connected to, then what each starts
/// from for it): shown beside it, a touch picks one.
List<String> wsRelationsOf(Ws ws) {
  final m = ws.inside, t = int.tryParse(ws.selected ?? '');
  if (m == null || t == null || m.word(t, 'kind') == 'behaviour') return const [];
  final all = <String>[];
  for (var r = (m.value(t, 'x_by', 0) ?? 0).round(); r > 0 && all.length < 8 && !all.contains('$r') && wsRelations.contains(m.word(r, 'file')); ) {
    all.add('$r');
    final members = m.word(r, 'members').split(' '), under = m.word(r, 'under').split(' '), at = members.indexOf('$t');
    r = at >= 0 && at < under.length ? int.tryParse(under[at]) ?? 0 : 0;
  }
  return all;
}
