// Explore B: the asset universe as a graph of relations that can be proven (this folder, this Source, used by this project,
// the same bytes, the same type), not a similarity ranking. Position comes from the graph's own topology (a force layout),
// never from colour, so a sound or a model is as much at home as a picture. Local shows the chosen asset and what is within
// a hop or two of it; Global shows everything. Prototype: Explorer only, nothing here is production.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:motolii_stage5/hf/metrics.dart';
import 'package:motolii_stage5/hf/neutral.dart';
import 'package:motolii_stage5/live_hf/adapters/browser_item.dart';
import 'package:motolii_stage5/live_hf/adapters/graph_overlay.dart';
import 'package:motolii_stage5/live_hf/adapters/media_fluid.dart';

/// What the person chose for the graph: which relations to draw, and global or local (1 or 2 hops round the chosen one).
class GraphChoice extends ChangeNotifier {
  GraphChoice({this.hops = 0, Set<String>? relations}) : relations = relations ?? {'folder', 'source', 'project', 'duplicate'};

  /// 0 = global (everything), 1 or 2 = local.
  int hops;
  final Set<String> relations;

  /// The map: where each node of the Global graph settled. It outlives selections, filters that come and go, and the
  /// Explore view being closed and opened, so an asset stays where the person last saw it (a new node starts near what it
  /// is joined to and the rest hardly move).
  final Map<String, Offset> memory = {};

  void setHops(int n) {
    hops = n;
    notifyListeners();
  }

  void toggle(String relation) {
    relations.contains(relation) ? relations.remove(relation) : relations.add(relation);
    notifyListeners();
  }
}

class _Node {
  _Node(this.id, this.w, this.h, {this.hub});
  final String id;
  final double w, h;
  final GraphHub? hub;
  Offset p = Offset.zero;
  bool pinned = false;
  bool old = false; // was already on the map
  double get r => math.max(w, h) / 2;
}

String _dir(String rel) => rel.contains('/') ? rel.substring(0, rel.lastIndexOf('/')) : '';

/// The relations that can be proven from what the catalog and the work hold (edge weights say how tightly they pull).
({List<GraphEdge> edges, Map<String, String> hubLabel, Map<String, String> hubRelation}) relationsOf(List<BrowserItem> items, Set<String> on, Set<String> usedPaths) {
  final edges = <GraphEdge>[];
  final label = <String, String>{}, relation = <String, String>{};
  void hub(String id, String text, String rel) {
    label[id] = text;
    relation[id] = rel;
  }

  final folders = on.contains('folder'), sources = on.contains('source');
  for (final it in items) {
    final dir = _dir(it.rel);
    if (folders && (dir.isNotEmpty || it.rel.isNotEmpty)) {
      final id = 'folder:${it.source}:$dir';
      hub(id, dir.isEmpty ? '/ ${it.source}' : dir.split('/').last, 'folder');
      edges.add(GraphEdge(it.id, id, 'folder'));
      // the chain of parents up to the source
      var d = dir;
      var child = id;
      while (d.isNotEmpty) {
        final parent = _dir(d);
        final pid = 'folder:${it.source}:$parent';
        hub(pid, parent.isEmpty ? '/ ${it.source}' : parent.split('/').last, 'folder');
        if (!edges.any((e) => e.a == child && e.b == pid)) edges.add(GraphEdge(child, pid, 'folder'));
        child = pid;
        d = parent;
      }
    }
    if (sources && it.source.isNotEmpty && !(folders && it.rel.isNotEmpty)) {
      hub('source:${it.source}', it.source, 'source');
      edges.add(GraphEdge(it.id, 'source:${it.source}', 'source'));
    }
    if (on.contains('type')) {
      hub('type:${it.kind}', it.typeWord, 'type');
      edges.add(GraphEdge(it.id, 'type:${it.kind}', 'type'));
    }
    if (on.contains('project') && usedPaths.contains(it.path)) {
      hub('project:this', 'This project', 'project');
      edges.add(GraphEdge(it.id, 'project:this', 'project'));
    }
  }
  if (sources) {
    // the folder roots hang from their Source
    for (final id in [...label.keys]) {
      if (!id.startsWith('folder:')) continue;
      final rest = id.substring('folder:'.length);
      final cut = rest.indexOf(':');
      final src = rest.substring(0, cut), dir = rest.substring(cut + 1);
      if (dir.isEmpty && folders) {
        hub('source:$src', src, 'source');
        edges.add(GraphEdge(id, 'source:$src', 'source'));
      }
    }
  }
  if (on.contains('duplicate')) {
    final byPrint = <String, List<BrowserItem>>{};
    for (final it in items) {
      if (it.fingerprint != null && it.fingerprint!.isNotEmpty) byPrint.putIfAbsent(it.fingerprint!, () => []).add(it);
    }
    for (final group in byPrint.values) {
      for (var i = 1; i < group.length; i++) {
        edges.add(GraphEdge(group[0].id, group[i].id, 'duplicate'));
      }
    }
  }
  return (edges: edges, hubLabel: label, hubRelation: relation);
}

double _weight(String relation) => switch (relation) { 'duplicate' => 1.4, 'folder' => 1.0, 'project' => .8, 'source' => .6, _ => .35 };

/// The Explore layout for the graph, as the Browser's Explore hook takes it.
ExploreLayout graphLayout(GraphChoice choice, Set<String> Function() usedPaths) {
  String? lastKey;
  Frame? last;
  return (items, selected, viewport) {
    final on = {...choice.relations};
    final used = usedPaths();
    final topology = '${items.map((i) => i.id).join(',')}|${on.toList()..sort()}|${used.length}|${items.map((i) => i.fingerprint).join(',').hashCode}';
    // Global: the layout is a function of the graph alone, so choosing an asset (or resizing the seat) never asks for one.
    // Local is the explicit "look round this asset": it is rooted at the chosen one, so it follows the choice.
    final key = choice.hops == 0 ? 'g|$topology' : 'l${choice.hops}|$selected|$topology';
    if (key != lastKey || last == null) {
      last = _build(choice, items, selected, choice.hops, on, used);
      lastKey = key;
    }
    final frame = last!;
    final chosen = frame.faces[selected];
    if (choice.hops != 0 || chosen == null) return frame;
    // only the chosen one's name follows the choice
    return (faces: frame.faces, labels: {selected!: Rect.fromLTWH(chosen.left - 20, chosen.bottom + 1, chosen.width + 40, 14)}, links: frame.links, graph: frame.graph, content: frame.content);
  };
}

Frame _build(GraphChoice choice, List<BrowserItem> items, String? selected, int hops, Set<String> on, Set<String> used) {
  final focus = items.any((i) => i.id == selected) ? selected : (items.isEmpty ? null : items.first.id);
  final rel = relationsOf(items, on, used);
  // who is joined to whom, with how far each step is (an asset and its hub are half a hop: two assets in one folder are one)
  final adj = <String, List<(String, double)>>{};
  for (final e in rel.edges) {
    final hubEnds = (rel.hubLabel.containsKey(e.a) ? 1 : 0) + (rel.hubLabel.containsKey(e.b) ? 1 : 0);
    final cost = e.relation == 'duplicate' ? 1.0 : (hubEnds == 0 ? 1.0 : .5);
    adj.putIfAbsent(e.a, () => []).add((e.b, cost));
    adj.putIfAbsent(e.b, () => []).add((e.a, cost));
  }
  final include = <String>{};
  if (hops == 0 || focus == null) {
    include.addAll(items.map((i) => i.id));
    include.addAll(rel.hubLabel.keys);
  } else {
    final dist = <String, double>{focus: 0};
    final queue = [focus];
    while (queue.isNotEmpty) {
      queue.sort((a, b) => dist[a]!.compareTo(dist[b]!));
      final at = queue.removeAt(0);
      for (final (to, cost) in adj[at] ?? const <(String, double)>[]) {
        final d = dist[at]! + cost;
        if (d <= hops && d < (dist[to] ?? 1e9)) {
          dist[to] = d;
          queue.add(to);
        }
      }
    }
    include.addAll(dist.keys);
  }
  final byId = {for (final i in items) i.id: i};
  final nodes = <_Node>[];
  for (final id in include) {
    final it = byId[id];
    if (it != null) {
      // only the Local graph makes its root large; in the Global map every asset keeps its size whoever is chosen
      final big = id == focus && hops > 0;
      final base = big ? 76.0 : 42.0;
      final a = it.aspect;
      nodes.add(_Node(id, a >= 1 ? base : base * a, a >= 1 ? base / a : base));
    } else if (rel.hubLabel.containsKey(id)) {
      final text = rel.hubLabel[id]!;
      nodes.add(_Node(id, math.min(120.0, 14 + text.length * 5.6), 16, hub: null));
    }
  }
  final n = nodes.length;
  // the world's size follows how many nodes there are, never the seat (the seat only decides how much of it is in view)
  final area = math.max(560.0 * 400.0, n * 110.0 * 110.0);
  final w = math.sqrt(area * 1.4), h = math.sqrt(area / 1.4);
  final k = math.sqrt(w * h / math.max(1, n)) * .8;
  final idx = {for (final (i, node) in nodes.indexed) node.id: i};
  final links = [for (final e in rel.edges) if (idx.containsKey(e.a) && idx.containsKey(e.b)) (idx[e.a]!, idx[e.b]!, _weight(e.relation))];
  final global = hops == 0;
  final remembered = global ? [for (final node in nodes) if (choice.memory.containsKey(node.id)) node] : <_Node>[];
  var iters = n <= 120 ? 260 : 120;
  var t = math.min(w, h) / 6;
  var simulate = true;
  for (final (i, node) in nodes.indexed) {
    final mem = global ? choice.memory[node.id] : null;
    if (mem != null) {
      node.p = mem;
      node.old = true;
      continue;
    }
    // a new node starts beside what it is joined to (the middle of those already placed), else on a nudged circle
    final placed = [for (final (to, _) in adj[node.id] ?? const <(String, double)>[]) if (global && choice.memory[to] != null) choice.memory[to]!];
    final hash = node.id.hashCode & 0xffff;
    if (placed.isNotEmpty) {
      final mean = placed.fold(Offset.zero, (a, b) => a + b) / placed.length.toDouble();
      node.p = mean + Offset(math.cos(hash / 65535 * 6.28) * 36, math.sin(hash / 65535 * 6.28) * 36);
    } else {
      final ang = 2 * math.pi * i / math.max(1, n) + hash / 65535 * .3;
      final rad = math.min(w, h) * (.25 + (hash % 97) / 97 * .2);
      node.p = Offset(w / 2 + math.cos(ang) * rad, h / 2 + math.sin(ang) * rad);
    }
    if (node.id == focus && hops > 0) {
      node.p = Offset(w / 2, h / 2);
      node.pinned = true;
    }
  }
  if (global && remembered.length == nodes.length) {
    simulate = false; // nothing about the graph changed: the map is as it was
  } else if (global && remembered.isNotEmpty) {
    // something came or went: the ones already on the map only settle a little
    iters = 90;
    t = math.min(w, h) / 40;
  }
  for (var it = 0; simulate && it < iters; it++) {
    final disp = List<Offset>.filled(n, Offset.zero);
    for (var i = 0; i < n; i++) {
      for (var j = i + 1; j < n; j++) {
        var d = nodes[i].p - nodes[j].p;
        var dist = d.distance;
        if (dist < .01) {
          d = Offset((i - j) * .01, .01);
          dist = d.distance;
        }
        // repel, counting the faces' own size so they do not sit on one another
        final eff = math.max(dist - (nodes[i].r + nodes[j].r) * .8, 2.0);
        final f = k * k / eff;
        final v = d / dist * f;
        disp[i] += v;
        disp[j] -= v;
      }
    }
    for (final (a, b, wt) in links) {
      final d = nodes[a].p - nodes[b].p;
      final dist = math.max(d.distance, .01);
      final f = dist * dist / k * wt;
      final v = d / dist * f;
      disp[a] -= v;
      disp[b] += v;
    }
    for (var i = 0; i < n; i++) {
      // a light pull to the middle keeps separate islands in view
      disp[i] += (Offset(w / 2, h / 2) - nodes[i].p) * .05;
      if (nodes[i].pinned) continue;
      // ones that were already placed on the map keep to it; only a settle nudges them
      final len = math.max(disp[i].distance, .01);
      // what is already on the map gives way only a little; the newcomer does the moving
      nodes[i].p += disp[i] / len * math.min(len, nodes[i].old ? t * .12 : t);
      nodes[i].p = Offset(nodes[i].p.dx.clamp(nodes[i].w / 2 + 30, w - nodes[i].w / 2 - 30), nodes[i].p.dy.clamp(nodes[i].h / 2 + 30, h - nodes[i].h / 2 - 30));
    }
    t *= .985;
  }
  if (global) {
    for (final node in nodes) {
      choice.memory[node.id] = node.p;
    }
  }
  final faces = <String, Rect>{}, labels = <String, Rect>{};
  final hubs = <GraphHub>[];
  for (final node in nodes) {
    final rect = Rect.fromCenter(center: node.p, width: node.w, height: node.h);
    if (byId.containsKey(node.id)) {
      faces[node.id] = rect;
      if (node.id == focus) labels[node.id] = Rect.fromLTWH(rect.left - 20, rect.bottom + 1, rect.width + 40, 14);
    } else {
      hubs.add(GraphHub(id: node.id, label: rel.hubLabel[node.id]!, relation: rel.hubRelation[node.id]!, rect: rect));
    }
  }
  return (faces: faces, labels: labels, links: const <String>[], graph: GraphOverlay(hubs: hubs, edges: [for (final e in rel.edges) if (idx.containsKey(e.a) && idx.containsKey(e.b)) e]), content: Size(w, h));
}

/// Global / Local, and which relations are drawn (their colours are the lines').
class GraphBar extends StatelessWidget {
  const GraphBar({super.key, required this.choice});
  final GraphChoice choice;

  static const _kinds = [('folder', 'Folder'), ('source', 'Source'), ('project', 'Project'), ('duplicate', 'Same bytes'), ('type', 'Type')];

  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: choice,
        builder: (context, _) => Padding(
          padding: const EdgeInsets.fromLTRB(9, 3, 9, 0),
          child: Wrap(spacing: 4, runSpacing: 3, crossAxisAlignment: WrapCrossAlignment.center, children: [
            for (final (n, label) in const [(0, 'Global'), (1, 'Local 1'), (2, 'Local 2')]) _Chip(label, choice.hops == n, () => choice.setHops(n)),
            const SizedBox(width: 6),
            for (final (kind, label) in _kinds) _Chip(label, choice.relations.contains(kind), () => choice.toggle(kind), dot: relationColors[kind]),
          ]),
        ),
      );
}

class _Chip extends StatelessWidget {
  const _Chip(this.label, this.on, this.tap, {this.dot});
  final String label;
  final bool on;
  final VoidCallback tap;
  final Color? dot;
  @override
  Widget build(BuildContext context) => GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: tap,
        child: Container(
          padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
          decoration: BoxDecoration(color: on ? N.g20 : null, borderRadius: BorderRadius.circular(3)),
          child: Row(mainAxisSize: MainAxisSize.min, children: [
            if (dot != null) Container(width: 6, height: 6, margin: const EdgeInsets.only(right: 4), decoration: BoxDecoration(color: on ? dot : N.g44, shape: BoxShape.circle)),
            Text(label, softWrap: false, style: Dn.label(on ? N.g95 : N.g63, on ? FontWeight.w600 : FontWeight.w500)),
          ]),
        ),
      );
}
