// Explore: the asset universe as a map. Every asset is joined only to its few nearest by a cheap score made from values the
// Catalog already holds (name, folder distance, time, shape, kind, length / size, use in the work), and the graph is laid out
// once by a force layout. Nearness is distance and islands; Type, Source and Folder are not edges, they only tint and shade the
// faces (the overlay). Nothing is analysed and no meaning is invented: an asset with nothing near it stands alone.
//
// Global keeps one stored layout whatever is chosen, filtered or how big the seat is (the seat is only a lens on the world).
// Local is the explicit look round the chosen asset (1 or 2 hops), rooted at it, so it follows the choice.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../../../theme/metrics.dart';
import '../../../theme/neutral.dart';
import '../../item.dart';
import 'overlay.dart';
import '../fluid.dart';

/// How many nearest each asset is joined to. Measured on the real fixture (50 assets: pictures, clips, sounds, 3D models,
/// textures): k=3 gives 92 lines, 3 islands (pictures with clips and sounds / models / textures), at most 6 lines at any asset;
/// k=4 joins everything into one piece (125 lines, 5 at an asset on average), which hides the islands.
const exploreK = 3;

/// The cheap features that join two assets, each 0..1 and weighted; one an asset lacks adds nothing.
const _w = (name: 3.0, folder: 2.5, time: 1.0, shape: 1.0, kind: .5, size: .5, project: 1.5);
const _wAll = 3.0 + 2.5 + 1.0 + 1.0 + .5 + .5 + 1.5;

/// A neighbour must be at least this near to count: an asset with nothing near it stays alone rather than take a random one.
const _floor = .14;

Set<String> _tokens(String name) {
  final stem = name.contains('.') ? name.substring(0, name.lastIndexOf('.')) : name;
  final spaced = stem.replaceAllMapped(RegExp(r'([a-z])([A-Z])'), (m) => '${m[1]} ${m[2]}');
  return {
    for (final t in spaced.toLowerCase().split(RegExp(r'[^a-z]+')))
      if (t.length >= 2) t,
  };
}

List<String> _dirs(BrowserItem i) {
  final parts = i.rel.split('/').where((p) => p.isNotEmpty).toList();
  if (parts.isNotEmpty) parts.removeLast(); // the file name
  return [i.source, ...parts];
}

double _ratio(double a, double b) => (a <= 0 || b <= 0) ? 0 : 1 - math.min(1.0, (math.log(a / b)).abs());

/// Whether the work holds this asset: by content identity (the catalog's fingerprint is the very text the work stores as an
/// asset's content hash), never by where the file stands.
bool inWork(BrowserItem i, Set<String> hashes) => i.used || (i.fingerprint != null && hashes.contains(i.fingerprint));

/// 0..1: how near two assets are by what is already known about them (folder distance is a tree distance, so siblings are near
/// and cousins a little near; a shared kind is only a nudge and never joins two on its own).
double similarity(BrowserItem a, BrowserItem b, {required Set<String> hashes, Map<String, Set<String>>? tokens, Map<String, List<String>>? dirs}) {
  final ta = tokens?[a.id] ?? _tokens(a.name), tb = tokens?[b.id] ?? _tokens(b.name);
  final union = ta.union(tb).length;
  final name = union == 0 ? 0.0 : ta.intersection(tb).length / union;
  final da = dirs?[a.id] ?? _dirs(a), db = dirs?[b.id] ?? _dirs(b);
  var common = 0;
  while (common < da.length && common < db.length && da[common] == db[common]) {
    common++;
  }
  final folder = common == 0 ? 0.0 : 1 / (1 + (da.length - common) + (db.length - common));
  var time = 0.0;
  if (a.mtimeNs != null && b.mtimeNs != null) time = math.exp(-((a.mtimeNs! - b.mtimeNs!).abs() / 1e9) / 3600);
  var shape = 0.0;
  if (a.width != null && b.width != null && a.height != null && b.height != null && a.height! > 0 && b.height! > 0) {
    shape = _ratio(a.width! / a.height!, b.width! / b.height!);
  }
  final kind = a.kind == b.kind ? 1.0 : 0.0;
  var size = 0.0;
  if (a.seconds != null && b.seconds != null) {
    size = _ratio(a.seconds!, b.seconds!);
  } else if (a.size != null && b.size != null) {
    size = _ratio(a.size!.toDouble(), b.size!.toDouble());
  }
  final project = inWork(a, hashes) && inWork(b, hashes) ? 1.0 : 0.0;
  return (_w.name * name + _w.folder * folder + _w.time * time + _w.shape * shape + _w.kind * kind + _w.size * size + _w.project * project) / _wAll;
}

/// The sparse graph: each asset's [k] nearest, joined (an asset may be the nearest of many, but no more than `2k` lines end at
/// one asset, so nothing becomes a star), and every pair of the same bytes (a real, strong relation).
List<GraphEdge> knnEdges(List<BrowserItem> items, int k, Set<String> hashes) {
  final tokens = {for (final i in items) i.id: _tokens(i.name)};
  final dirs = {for (final i in items) i.id: _dirs(i)};
  final candidates = <(String, String, double)>[];
  for (final a in items) {
    final scored = <(String, double)>[];
    for (final b in items) {
      if (identical(a, b) || a.id == b.id) continue;
      final s = similarity(a, b, hashes: hashes, tokens: tokens, dirs: dirs);
      if (s >= _floor) scored.add((b.id, s));
    }
    scored.sort((x, y) => y.$2.compareTo(x.$2));
    for (final (b, s) in scored.take(k)) {
      candidates.add((a.id, b, s));
    }
  }
  // the strongest first, so a cap keeps the lines that mean most
  candidates.sort((x, y) => y.$3.compareTo(x.$3));
  final degree = <String, int>{};
  final seen = <String>{};
  final edges = <GraphEdge>[];
  String key(String a, String b) => a.compareTo(b) < 0 ? '$a|$b' : '$b|$a';
  for (final (a, b, _) in candidates) {
    if ((degree[a] ?? 0) >= 2 * k || (degree[b] ?? 0) >= 2 * k) continue;
    if (!seen.add(key(a, b))) continue;
    edges.add(GraphEdge(a, b, 'similar'));
    degree[a] = (degree[a] ?? 0) + 1;
    degree[b] = (degree[b] ?? 0) + 1;
  }
  final byPrint = <String, List<BrowserItem>>{};
  for (final i in items) {
    if (i.fingerprint != null && i.fingerprint!.isNotEmpty) byPrint.putIfAbsent(i.fingerprint!, () => []).add(i);
  }
  for (final g in byPrint.values) {
    for (var i = 1; i < g.length; i++) {
      if (seen.add(key(g[0].id, g[i].id))) edges.add(GraphEdge(g[0].id, g[i].id, 'duplicate'));
    }
  }
  return edges;
}

const kindTints = {
  'image': Color(0xFF7FB2E5),
  'video': Color(0xFFE5B27F),
  'audio': Color(0xFF8FD6A0),
  'model': Color(0xFFA48FE0),
  'environment': Color(0xFFE58FB8),
};

/// The regions of shared place, as soft blobs behind the faces (a folder with at least three of them in view).
List<({Offset at, double radius, Color color})> folderBlobs(List<BrowserItem> items, Map<String, Rect> faces) {
  final groups = <String, List<BrowserItem>>{};
  for (final i in items) {
    if (!faces.containsKey(i.id)) continue;
    final d = _dirs(i);
    if (d.length < 2) continue; // files at a source's root share only the source
    groups.putIfAbsent(d.join('/'), () => []).add(i);
  }
  final out = <({Offset at, double radius, Color color})>[];
  for (final e in groups.entries) {
    if (e.value.length < 3) continue;
    final hue = (e.key.hashCode & 0xffff) / 65535 * 360;
    final color = HSLColor.fromAHSL(.10, hue, .55, .6).toColor();
    for (final i in e.value) {
      out.add((at: faces[i.id]!.center, radius: 34.0, color: color));
    }
  }
  return out;
}

/// What the person chose for Explore: global or local (1 or 2 hops round the chosen asset), and whether the Type / Folder
/// overlay is on.
class ExploreChoice extends ChangeNotifier {
  ExploreChoice({this.hops = 0, this.overlay = false, this.scale});

  /// 0 = global (everything), 1 or 2 = local.
  int hops;
  bool overlay;

  /// A fixed first zoom for the camera (contact sheets compare at one magnification).
  final double? scale;

  /// The map: where each node of the Global graph settled, in the layout's own coordinates. It outlives selections, and the
  /// Explore view being closed and opened, so an asset stays where the person last saw it (a new node starts near what it is
  /// joined to and the rest hardly move).
  final Map<String, Offset> memory = {};

  void setHops(int n) {
    hops = n;
    notifyListeners();
  }

  void toggleOverlay() {
    overlay = !overlay;
    notifyListeners();
  }
}

class _Node {
  _Node(this.id, this.w, this.h);
  final String id;
  final double w, h;
  Offset p = Offset.zero;
  bool pinned = false;
  bool old = false; // was already on the map
  double get r => math.max(w, h) / 2;
}

Frame _empty() => (faces: const <String, Rect>{}, labels: const <String, Rect>{}, links: const <String>[], graph: null, content: const Size(1, 1));

/// The empty space kept round the map, so no island is pressed against the edge of the world.
const _margin = 64.0;

/// Explore's places, as the Browser's Explore hook takes them. [workHashes] is the content hashes of what the work holds.
ExploreLayout exploreLayout(ExploreChoice choice, Set<String> Function() workHashes) {
  String? lastKey;
  Frame? last;
  return (items, selected, viewport) {
    final hashes = workHashes();
    final topology = '${items.map((i) => '${i.id}${i.mtimeNs}${i.fingerprint}${i.used}').join(',').hashCode}|${hashes.length}';
    // Global: the layout is a function of the graph alone, so choosing an asset (or resizing the seat) never asks for one.
    final key = choice.hops == 0 ? 'g|$topology|${choice.overlay}' : 'l${choice.hops}|$selected|$topology|${choice.overlay}';
    if (key != lastKey || last == null) {
      last = _build(choice, items, selected, hashes);
      lastKey = key;
    }
    final frame = last!;
    final chosen = frame.faces[selected];
    if (choice.hops != 0 || chosen == null) return frame;
    // only the chosen one's name follows the choice
    return (faces: frame.faces, labels: {selected!: Rect.fromLTWH(chosen.left - 20, chosen.bottom + 1, chosen.width + 40, 14)}, links: frame.links, graph: frame.graph, content: frame.content);
  };
}

Frame _build(ExploreChoice choice, List<BrowserItem> items, String? selected, Set<String> hashes) {
  if (items.isEmpty) return _empty();
  final hops = choice.hops;
  final String focus = items.any((i) => i.id == selected) ? selected! : items.first.id;
  final all = knnEdges(items, exploreK, hashes);
  final adj = <String, List<String>>{};
  for (final e in all) {
    adj.putIfAbsent(e.a, () => []).add(e.b);
    adj.putIfAbsent(e.b, () => []).add(e.a);
  }
  final include = <String>{};
  if (hops == 0) {
    include.addAll(items.map((i) => i.id));
  } else {
    Set<String> frontier = {focus};
    include.add(focus);
    for (var d = 0; d < hops; d++) {
      frontier = {for (final id in frontier) for (final to in adj[id] ?? const <String>[]) if (include.add(to)) to};
    }
  }
  final byId = {for (final i in items) i.id: i};
  final nodes = <_Node>[];
  for (final i in items) {
    if (!include.contains(i.id)) continue;
    final base = (i.id == focus && hops > 0) ? 84.0 : 50.0;
    final a = i.aspect;
    nodes.add(_Node(i.id, a >= 1 ? base : base * a, a >= 1 ? base / a : base));
  }
  final n = nodes.length;
  // the world's size follows how many nodes there are, never the seat (the seat only decides how much of it is in view)
  // (counted in steps of 16 so that one asset more or less does not rescale the whole world)
  final steps = ((n + 15) ~/ 16) * 16;
  final area = math.max(560.0 * 400.0, steps * 120.0 * 120.0);
  final w = math.sqrt(area * 1.4), h = math.sqrt(area / 1.4);
  final k = math.sqrt(w * h / math.max(1, steps)) * .8;
  final idx = {for (final (i, node) in nodes.indexed) node.id: i};
  final links = [for (final e in all) if (idx.containsKey(e.a) && idx.containsKey(e.b)) (idx[e.a]!, idx[e.b]!, e.relation == 'duplicate' ? 1.4 : .9)];
  final global = hops == 0;
  var iters = n <= 120 ? 260 : 120;
  var t = math.min(w, h) / 6;
  var remembered = 0;
  for (final (i, node) in nodes.indexed) {
    final mem = global ? choice.memory[node.id] : null;
    if (mem != null) {
      node.p = mem;
      node.old = true;
      remembered++;
      continue;
    }
    final placed = [for (final to in adj[node.id] ?? const <String>[]) if (global && choice.memory[to] != null) choice.memory[to]!];
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
  var simulate = true;
  if (global && remembered == nodes.length) {
    simulate = false; // nothing about the graph changed: the map is as it was
  } else if (global && remembered > 0) {
    iters = 60; // something came or went: the ones already on the map only settle a little
    t = math.min(w, h) / 60;
  }
  if (simulate) _settle(nodes, links, w, h, k, iters, t);
  if (global) {
    for (final node in nodes) {
      choice.memory[node.id] = node.p;
    }
  }
  // the map sits inside its own margin: what the layout made, moved so that the empty space round it is the same on every side
  var box = Rect.fromCenter(center: nodes.first.p, width: nodes.first.w, height: nodes.first.h);
  for (final node in nodes) {
    box = box.expandToInclude(Rect.fromCenter(center: node.p, width: node.w, height: node.h));
  }
  final shift = const Offset(_margin, _margin) - box.topLeft;
  final faces = <String, Rect>{};
  for (final node in nodes) {
    faces[node.id] = Rect.fromCenter(center: node.p + shift, width: node.w, height: node.h);
  }
  final overlay = choice.overlay;
  return (
    faces: faces,
    labels: const <String, Rect>{},
    links: const <String>[],
    graph: GraphOverlay(
      initialScale: choice.scale,
      hubs: const [],
      edges: [for (final e in all) if (idx.containsKey(e.a) && idx.containsKey(e.b)) e],
      tints: overlay ? {for (final id in faces.keys) id: kindTints[byId[id]!.kind] ?? const Color(0xFF888888)} : const {},
      blobs: overlay ? folderBlobs(items, faces) : const [],
    ),
    content: Size(box.width + 2 * _margin, box.height + 2 * _margin),
  );
}

/// The force layout: repel by size (only within reach, so a stray asset is not pushed to the end of the world), attract along
/// links, a light pull to the middle, cool down; what is `old` (already on the map) only gives way a little. There is no wall:
/// the map is as wide as it turns out.
void _settle(List<_Node> nodes, List<(int, int, double)> links, double w, double h, double k, int iters, double t0) {
  final n = nodes.length;
  final reach = 3.2 * k;
  var t = t0;
  for (var it = 0; it < iters; it++) {
    final disp = List<Offset>.filled(n, Offset.zero);
    for (var i = 0; i < n; i++) {
      for (var j = i + 1; j < n; j++) {
        var d = nodes[i].p - nodes[j].p;
        var dist = d.distance;
        if (dist < .01) {
          d = Offset((i - j) * .01, .01);
          dist = d.distance;
        }
        final eff = math.max(dist - (nodes[i].r + nodes[j].r) * .8, 2.0);
        if (eff > reach) continue;
        final v = d / dist * (k * k / eff);
        disp[i] += v;
        disp[j] -= v;
      }
    }
    for (final (a, b, wt) in links) {
      final d = nodes[a].p - nodes[b].p;
      final dist = math.max(d.distance, .01);
      final v = d / dist * (dist * dist / k * wt);
      disp[a] -= v;
      disp[b] += v;
    }
    for (var i = 0; i < n; i++) {
      disp[i] += (Offset(w / 2, h / 2) - nodes[i].p) * .05;
      if (nodes[i].pinned) continue;
      final len = math.max(disp[i].distance, .01);
      nodes[i].p += disp[i] / len * math.min(len, nodes[i].old ? t * .06 : t);
    }
    t *= .985;
  }
}

/// Global / Local 1 / Local 2, and the Type · Folder overlay.
class ExploreBar extends StatelessWidget {
  const ExploreBar({super.key, required this.choice});
  final ExploreChoice choice;

  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: choice,
        builder: (context, _) => Padding(
          padding: const EdgeInsets.fromLTRB(9, 3, 9, 0),
          child: Wrap(spacing: 4, runSpacing: 3, crossAxisAlignment: WrapCrossAlignment.center, children: [
            for (final (n, label) in const [(0, 'Global'), (1, 'Local 1'), (2, 'Local 2')]) _Chip(label, choice.hops == n, () => choice.setHops(n)),
            const SizedBox(width: Surface.sectionGap),
            _Chip('Type · Folder', choice.overlay, choice.toggleOverlay),
          ]),
        ),
      );
}

class _Chip extends StatelessWidget {
  const _Chip(this.label, this.on, this.tap);
  final String label;
  final bool on;
  final VoidCallback tap;
  @override
  Widget build(BuildContext context) => GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: tap,
        child: Container(
          padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
          decoration: BoxDecoration(color: on ? N.g20 : null, borderRadius: BorderRadius.circular(Surface.controlRadius)),
          child: Text(label, softWrap: false, style: Dn.label(on ? N.g95 : N.g63, on ? FontWeight.w600 : FontWeight.w500)),
        ),
      );
}
