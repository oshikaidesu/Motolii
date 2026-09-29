// Explore B, second topology: a sparse similarity graph. Every asset is joined only to its few nearest by a cheap score made
// from values the Catalog already holds (name, folder distance, time, shape, kind, length / size, use in the work), and the
// graph is laid out once. Type, Source, Folder and Project are not edges: they colour and shade the faces. Nothing is analysed
// and no meaning is invented; an asset with nothing near it stands alone.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:motolii_stage5/live_hf/adapters/browser_item.dart';
import 'package:motolii_stage5/live_hf/adapters/graph_overlay.dart';
import 'package:motolii_stage5/live_hf/adapters/media_fluid.dart';

/// Which cheap features join two assets, each 0..1 and weighted; a feature one of them lacks adds nothing.
const _w = (name: 3.0, folder: 2.5, time: 1.0, shape: 1.0, kind: .5, size: .5, project: 1.5);
const _wAll = 3.0 + 2.5 + 1.0 + 1.0 + .5 + .5 + 1.5;

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

/// 0..1: how near two assets are by what is already known about them (folder distance is a tree distance, so siblings are
/// near and cousins a little near; a shared kind is only a nudge and never joins two on its own).
double similarity(BrowserItem a, BrowserItem b, {required Set<String> used, Map<String, Set<String>>? tokens, Map<String, List<String>>? dirs}) {
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
  final project = used.contains(a.path) && used.contains(b.path) ? 1.0 : 0.0;
  return (_w.name * name + _w.folder * folder + _w.time * time + _w.shape * shape + _w.kind * kind + _w.size * size + _w.project * project) / _wAll;
}

/// A neighbour must be at least this near to count: an asset with nothing near it stays alone rather than take a random one.
const _floor = .14;

/// The sparse graph: each asset's [k] nearest, joined, and every pair of the same bytes (a real, strong relation).
List<GraphEdge> knnEdges(List<BrowserItem> items, int k, Set<String> used) {
  final tokens = {for (final i in items) i.id: _tokens(i.name)};
  final dirs = {for (final i in items) i.id: _dirs(i)};
  final seen = <String>{};
  final edges = <GraphEdge>[];
  for (final a in items) {
    final scored = <(BrowserItem, double)>[];
    for (final b in items) {
      if (identical(a, b) || a.id == b.id) continue;
      final s = similarity(a, b, used: used, tokens: tokens, dirs: dirs);
      if (s >= _floor) scored.add((b, s));
    }
    scored.sort((x, y) => y.$2.compareTo(x.$2));
    for (final (b, _) in scored.take(k)) {
      final key = a.id.compareTo(b.id) < 0 ? '${a.id}|${b.id}' : '${b.id}|${a.id}';
      if (seen.add(key)) edges.add(GraphEdge(a.id, b.id, 'similar'));
    }
  }
  final byPrint = <String, List<BrowserItem>>{};
  for (final i in items) {
    if (i.fingerprint != null && i.fingerprint!.isNotEmpty) byPrint.putIfAbsent(i.fingerprint!, () => []).add(i);
  }
  for (final g in byPrint.values) {
    for (var i = 1; i < g.length; i++) {
      final key = g[0].id.compareTo(g[i].id) < 0 ? '${g[0].id}|${g[i].id}' : '${g[i].id}|${g[0].id}';
      if (seen.add(key)) edges.add(GraphEdge(g[0].id, g[i].id, 'duplicate'));
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
