import 'package:flutter_test/flutter_test.dart';

import '../lib/workspace/layout.dart';

DockNode _find(DockNode root, String tab) =>
    root.leaves.firstWhere((n) => n.tabs.contains(tab));

/// The split whose [second] holds [tab]'s leaf, searching from [root].
DockNode? _splitAbove(DockNode root, String tab) {
  if (root.leaf) return null;
  if (root.second!.leaves.any((n) => n.tabs.contains(tab)) &&
      root.second!.leaves.length == 1)
    return root;
  return _splitAbove(root.first!, tab) ?? _splitAbove(root.second!, tab);
}

void main() {
  test('catalog widths size the side columns, not the window', () {
    final dock = initialDock();
    final top = dock.first!; // browser | (stage | inspector column)
    expect(top.firstExtent(1280), 260);
    expect(top.firstExtent(2560), 260);
    final inner = top.second!; // stage | (inspector / desk)
    final room = 1280 - 260 - 4;
    expect(room - inner.firstExtent(room.toDouble()) - 4, 300);
    final wide = 2560 - 260 - 4;
    expect(wide - inner.firstExtent(wide.toDouble()) - 4, 300);
  });

  test('two elastic sides still share by ratio', () {
    final dock = initialDock(); // work area / timeline
    expect(dock.firstExtent(900), closeTo((900 - 4) * .68, .001));
  });

  test('desk keeps its catalog height under the inspector', () {
    final column = _splitAbove(initialDock(), 'Desk')!;
    expect(column.firstExtent(600), 600 - 4 - 200);
  });

  test('dropping a fixed panel beside stage does not take half', () {
    final layout = WorkspaceLayout();
    layout.close('Inspector');
    layout.move('Inspector', _find(layout.root, 'Stage'), 'right');
    final split = _splitAbove(layout.root, 'Inspector')!;
    expect(split.first!.tabs, ['Stage', 'Camera']);
    expect(1000 - split.firstExtent(1000) - 4, 300);
  });

  test('dragging a fixed divider moves by pixels and survives json', () {
    final layout = WorkspaceLayout();
    final split = layout.root.first!.second!; // stage | (inspector / desk)
    split.drag(-40, 1000);
    expect(1000 - split.firstExtent(1000) - 4, 340);
    final again = DockNode.read(layout.root.json()).first!.second!;
    expect(1000 - again.firstExtent(1000) - 4, 340);
  });

  test('a fixed desk inside the work area does not pin the timeline split', () {
    final layout = WorkspaceLayout();
    layout.close('Stage');
    // work area (browser | inspector / desk) over timeline: still by ratio
    expect(layout.root.firstExtent(900), closeTo((900 - 4) * .68, .001));
    // without stage both sides are fixed: browser keeps 260, inspector absorbs
    expect(layout.root.first!.firstExtent(1000), 260);
  });

  test('legacy layouts without offset still read', () {
    final json = initialDock().json()..remove('offset');
    expect(DockNode.read(json).offset, 0);
  });
}
