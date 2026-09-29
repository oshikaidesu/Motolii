// Explore B is a map: choosing an asset moves nothing, a changed graph settles only what it must, and what was placed stays.
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:motolii_explorer/paper/graph_explore.dart';
import 'package:motolii_stage5/live_hf/adapters/browser_item.dart';

BrowserItem item(String id, String rel, {String source = 'S', String kind = 'image', String? fp}) => BrowserItem(id: id, name: id, path: '/r/$rel', kind: kind, mime: 'x/y', source: source, rel: rel, fingerprint: fp);

void main() {
  final items = [
    item('a', 'x/a.png'), item('b', 'x/b.png'), item('c', 'y/c.png'), item('d', 'y/d.png'),
    item('e', 'e.wav', kind: 'audio'), item('f', 'x/f.glb', kind: 'model', fp: 'same'), item('g', 'y/g.glb', kind: 'model', fp: 'same'),
  ];
  const seat = Size(420, 600);

  test('choosing another asset does not move any node in the Global map', () {
    final choice = GraphChoice();
    final layout = graphLayout(choice, () => {});
    final first = layout(items, 'a', seat);
    final other = layout(items, 'g', seat);
    expect(other.faces, first.faces, reason: 'selection is a highlight, not a layout');
    expect(other.content, first.content);
    expect(layout(items, 'g', const Size(300, 900)).faces, first.faces, reason: 'nor does the seat changing size');
  });

  test('a graph asked again from a fresh layout (Explore reopened) lands where it did', () {
    final choice = GraphChoice();
    final a = graphLayout(choice, () => {})(items, 'a', seat);
    final b = graphLayout(choice, () => {})(items, 'b', seat); // a new layout function, the same remembered map
    expect(b.faces, a.faces);
  });

  test('a new asset settles beside what it is joined to and the others hardly move', () {
    final choice = GraphChoice();
    final layout = graphLayout(choice, () => {});
    final before = layout(items, 'a', seat);
    final more = [...items, item('h', 'x/h.png')];
    final after = layout(more, 'a', seat);
    var moved = 0.0;
    for (final e in before.faces.entries) {
      moved = moved > (after.faces[e.key]!.center - e.value.center).distance ? moved : (after.faces[e.key]!.center - e.value.center).distance;
    }
    expect(moved, lessThan(60), reason: 'the world is not shuffled for one new node (largest move $moved)');
    final folderX = after.graph!.hubs.firstWhere((h) => h.label == 'x').rect.center;
    expect((after.faces['h']!.center - folderX).distance, lessThan(220), reason: 'it stands near its folder');
  });

  test('Local is the explicit look round one asset: it is rooted at the chosen asset and follows the choice', () {
    final choice = GraphChoice(hops: 1);
    final layout = graphLayout(choice, () => {});
    final a = layout(items, 'a', seat);
    expect(a.faces.containsKey('e'), isFalse, reason: 'the audio in the root folder is not one hop from a');
    expect(a.faces.containsKey('b'), isTrue, reason: 'a folder mate is');
    final e = layout(items, 'e', seat);
    expect(e.faces.containsKey('a'), isFalse);
  });

  test('sound and models are as placed as pictures: no colour decides anything', () {
    final frame = graphLayout(GraphChoice(), () => {})(items, 'a', seat);
    expect(frame.faces.keys.toSet(), {for (final i in items) i.id});
  });
}
