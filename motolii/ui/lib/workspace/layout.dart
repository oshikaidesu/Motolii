import 'package:flutter/widgets.dart';

import 'panel_ids.dart';

class DockNode {
  DockNode.leaf(this.id, this.tabs, {String? active})
    : active = active ?? (tabs.isEmpty ? '' : tabs.first);
  DockNode.split(this.axis, this.ratio, this.first, this.second)
    : id = '',
      tabs = [],
      active = '';
  String id, active;
  List<String> tabs;
  Axis? axis;
  double ratio = .5;
  DockNode? first, second;
  bool get leaf => axis == null;
  Iterable<DockNode> get leaves sync* {
    if (leaf) {
      yield this;
    } else {
      yield* first!.leaves;
      yield* second!.leaves;
    }
  }

  Map<String, dynamic> json() => leaf
      ? {'id': id, 'tabs': tabs, 'active': active}
      : {
          'axis': axis!.name,
          'ratio': ratio,
          'first': first!.json(),
          'second': second!.json(),
        };
  static DockNode read(Map<String, dynamic> m) {
    if (m['axis'] != null)
      return DockNode.split(
        m['axis'] == 'horizontal' ? Axis.horizontal : Axis.vertical,
        (m['ratio'] as num).toDouble().clamp(.1, .9),
        read(Map<String, dynamic>.from(m['first'] as Map)),
        read(Map<String, dynamic>.from(m['second'] as Map)),
      );
    return DockNode.leaf(
      '${m['id']}',
      (m['tabs'] as List)
          .whereType<String>()
          .where(paneNames.contains)
          .toList(),
      active: m['active'] as String?,
    );
  }
}

DockNode initialDock() => DockNode.split(
  Axis.vertical,
  .68,
  DockNode.split(
    Axis.horizontal,
    .205,
    DockNode.leaf('browser', ['Create', 'Media', 'Effects', 'Colors']),
    DockNode.split(
      Axis.horizontal,
      .775,
      DockNode.leaf('stage', ['Stage']),
      DockNode.split(
        Axis.vertical,
        .76,
        DockNode.leaf('inspector', ['Inspector']),
        DockNode.leaf('desk', ['Desk']),
      ),
    ),
  ),
  DockNode.leaf('timeline', ['Timeline']),
);

class WorkspaceLayout {
  DockNode root = initialDock();
  void show(String name) {
    if (!paneNames.contains(name)) return;
    for (final node in root.leaves) {
      if (node.tabs.contains(name)) {
        node.active = name;
        return;
      }
    }
    root.leaves.first
      ..tabs.add(name)
      ..active = name;
  }

  void close(String name) {
    for (final node in root.leaves) {
      node.tabs.remove(name);
      if (node.active == name)
        node.active = node.tabs.isEmpty ? '' : node.tabs.first;
    }
  }

  void move(String name, DockNode target, String edge) {
    if (!paneNames.contains(name)) return;
    close(name);
    if (edge == 'center' || target.tabs.isEmpty) {
      target.tabs.add(name);
      target.active = name;
      return;
    }
    final old = DockNode.leaf(
      target.id,
      List.of(target.tabs),
      active: target.active,
    );
    final next = DockNode.leaf(
      'pane-${DateTime.now().microsecondsSinceEpoch}',
      [name],
    );
    target.axis = (edge == 'left' || edge == 'right')
        ? Axis.horizontal
        : Axis.vertical;
    target.ratio = .5;
    target.first = (edge == 'left' || edge == 'top') ? next : old;
    target.second = (edge == 'left' || edge == 'top') ? old : next;
    target.tabs = [];
    target.active = '';
  }
}
