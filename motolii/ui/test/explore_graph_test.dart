// Explore is a map of nearness: sparse, stable under selection, with room round it, and what the work holds is told by content.
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:motolii_ui/browser/item.dart';
import 'package:motolii_ui/browser/media/explore/graph.dart';

BrowserItem item(String id, String rel, {String source = 'S', String kind = 'image', String? fp, String path = '', int? mtime, bool used = false}) =>
    BrowserItem(id: id, name: rel.split('/').last, path: path.isEmpty ? '/r/$rel' : path, kind: kind, mime: 'x/y', source: source, rel: rel, fingerprint: fp, mtimeNs: mtime, used: used);

void main() {
  const seat = Size(420, 600);
  final items = [
    for (var i = 0; i < 6; i++) item('a$i', 'shots/sky_$i.png', mtime: 1000000000 * i),
    for (var i = 0; i < 5; i++) item('m$i', 'models/chair_$i.glb', kind: 'model', mtime: 9000000000000 + i),
    for (var i = 0; i < 4; i++) item('s$i', 'sound/loop_$i.wav', kind: 'audio'),
    item('dup1', 'x/same.png', fp: 'f1'), item('dup2', 'y/copy.png', fp: 'f1'),
  ];

  test('every asset is joined to a few nearest only: no star, no clique, and no metadata as a node', () {
    final many = [for (var i = 0; i < 60; i++) item('n$i', 'z/n$i.png', mtime: 1000000000 * i)];
    final edges = knnEdges(many, exploreK, {});
    expect(edges.length, lessThan(many.length * exploreK), reason: '60 assets: a few lines each, never 60*59/2');
    final degree = <String, int>{};
    for (final e in edges) {
      degree[e.a] = (degree[e.a] ?? 0) + 1;
      degree[e.b] = (degree[e.b] ?? 0) + 1;
    }
    expect(degree.values.fold(0, (a, b) => a > b ? a : b), lessThanOrEqualTo(2 * exploreK), reason: 'no asset collects a star of lines');
    final ids = {for (final i in many) i.id};
    expect(edges.every((e) => ids.contains(e.a) && ids.contains(e.b)), isTrue, reason: 'every end is an asset: no folder, type or source hub');
    expect(edges.map((e) => e.relation).toSet(), {'similar'});
  });

  test('islands stay islands: pictures, models and sounds do not fuse into one piece', () {
    final edges = knnEdges(items, exploreK, {});
    final adj = <String, Set<String>>{};
    for (final e in edges) {
      (adj[e.a] ??= {}).add(e.b);
      (adj[e.b] ??= {}).add(e.a);
    }
    final seen = <String>{};
    var pieces = 0;
    for (final i in items) {
      if (!seen.add(i.id)) continue;
      pieces++;
      final q = [i.id];
      while (q.isNotEmpty) {
        for (final y in adj[q.removeLast()] ?? const <String>{}) {
          if (seen.add(y)) q.add(y);
        }
      }
    }
    expect(pieces, greaterThan(1), reason: 'an asset with nothing near it does not take a random neighbour');
  });

  test('the same bytes are joined by a duplicate line whatever their names', () {
    final edges = knnEdges(items, exploreK, {});
    expect(edges.any((e) => e.relation == 'duplicate' && {e.a, e.b}.containsAll(['dup1', 'dup2'])), isTrue);
  });

  test('choosing another asset, resizing the seat or toggling the overlay does not move any face in the Global map', () {
    final choice = ExploreChoice();
    final layout = exploreLayout(choice, () => {});
    final first = layout(items, 'a0', seat);
    expect(layout(items, 'm3', seat).faces, first.faces, reason: 'selection is a highlight, not a layout');
    expect(layout(items, 'm3', const Size(300, 900)).faces, first.faces, reason: 'nor is the seat');
    choice.toggleOverlay();
    expect(layout(items, 'm3', seat).faces, first.faces, reason: 'the overlay only tints');
  });

  test('the map has room round it: no face is pressed against the edge of the world', () {
    final frame = exploreLayout(ExploreChoice(), () => {})(items, 'a0', seat);
    for (final r in frame.faces.values) {
      expect(r.left, greaterThanOrEqualTo(40));
      expect(r.top, greaterThanOrEqualTo(40));
      expect(frame.content.width - r.right, greaterThanOrEqualTo(40));
      expect(frame.content.height - r.bottom, greaterThanOrEqualTo(40));
    }
  });

  test('Local is the explicit look round the chosen asset: rooted at it, big, and following the choice', () {
    final choice = ExploreChoice(hops: 1);
    final layout = exploreLayout(choice, () => {});
    final a = layout(items, 'a0', seat);
    expect(a.faces.containsKey('a0'), isTrue);
    expect(a.faces.length, lessThan(items.length), reason: 'only the near ones');
    expect(a.faces['a0']!.width, greaterThan(a.faces.entries.where((e) => e.key != 'a0').first.value.width));
    final m = layout(items, 'm0', seat);
    expect(m.faces.containsKey('m0'), isTrue);
    expect(m.faces.keys.toSet(), isNot(a.faces.keys.toSet()));
    choice.setHops(2);
    expect(layout(items, 'a0', seat).faces.length, greaterThanOrEqualTo(a.faces.length));
  });

  test('a new asset settles beside what it is near and the others hardly move (a folder watcher added a file)', () {
    final choice = ExploreChoice();
    final layout = exploreLayout(choice, () => {});
    final before = layout(items, 'a0', seat);
    final more = [...items, item('a6', 'shots/sky_6.png', mtime: 1000000000 * 6)];
    final after = layout(more, 'a0', seat);
    // the map may re-centre as a whole: measure against the common drift (the median displacement), not against the page
    final drifts = [for (final e in before.faces.entries) after.faces[e.key]!.center - e.value.center];
    double median(List<double> v) => (v..sort())[v.length ~/ 2];
    final shift = Offset(median([for (final d in drifts) d.dx]), median([for (final d in drifts) d.dy]));
    final moves = [for (final d in drifts) (d - shift).distance]..sort();
    // the newcomer's neighbours give way by about a face to make room; the rest of the world stays where it was
    expect(moves[moves.length ~/ 2], lessThan(20), reason: 'most of the world moved less than half a face (median ${moves[moves.length ~/ 2]})');
    expect(moves.last, lessThan(100), reason: 'and nobody was flung (largest move ${moves.last})');
    final near = (after.faces['a6']!.center - after.faces['a5']!.center).distance;
    expect(near, lessThan(260), reason: 'it stands near its neighbour in the pictures island');
  });

  test('what the work holds is told by content hash, never by where the file stands', () {
    final inProject = item('p', 'x/p.png', fp: 'motolii-source-v1:sha256-edges:abc', path: '/some/other/place/p.png');
    final elsewhere = item('q', 'y/q.png', fp: 'motolii-source-v1:sha256-edges:def');
    expect(inWork(inProject, {'motolii-source-v1:sha256-edges:abc'}), isTrue, reason: 'the path differs from the work\'s, the content does not');
    expect(inWork(elsewhere, {'motolii-source-v1:sha256-edges:abc'}), isFalse);
    expect(inWork(item('own', 'a.png', used: true), {}), isTrue, reason: 'the work\'s own asset is its own');
    final held = {'motolii-source-v1:sha256-edges:abc'};
    final near = similarity(inProject, item('r', 'x/r.png', fp: 'motolii-source-v1:sha256-edges:abc'), hashes: held);
    final far = similarity(inProject, elsewhere, hashes: held);
    expect(near, greaterThan(far), reason: 'two assets the work holds are nearer for it');
  });
}
