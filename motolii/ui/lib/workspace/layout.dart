import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../foundation/panel_catalog.dart';
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

  /// Share of the axis given to [first] when neither side has a fixed extent.
  double ratio = .5;

  /// Pixels the user dragged a fixed side away from its catalog extent.
  double offset = 0;
  DockNode? first, second;
  bool get leaf => axis == null;

  /// Divider thickness between two sides of a split.
  static const double gap = 4;
  bool get isEmpty => leaves.every((n) => n.tabs.isEmpty);

  /// What this subtree asks for along [along]. A leaf asks for the widest of
  /// its tabs; fill beats fixed beats any. Sides laid out along [along] add
  /// up only when all are fixed; sides stacked across it share the widest.
  /// Empty sides are skipped because [dockView] collapses them.
  Extent extent(Axis along) {
    if (leaf) {
      var fixed = 0.0;
      for (final t in tabs) {
        final e =
            (t == 'Desk' ? deskHostSpec : panelSpec(t))?.extent(along) ??
            Extent.any;
        if (e.isFill) return Extent.fill;
        if (e.isFixed && e.px > fixed) fixed = e.px;
      }
      return fixed > 0 ? Extent.fixed(fixed) : Extent.any;
    }
    final sides = [first!, second!].where((n) => !n.isEmpty).toList();
    final asks = sides.map((n) => n.extent(along)).toList();
    if (asks.any((e) => e.isFill)) return Extent.fill;
    final fixed = asks.where((e) => e.isFixed).map((e) => e.px).toList();
    if (fixed.isEmpty) return Extent.any;
    if (axis != along) return Extent.fixed(fixed.reduce(math.max));
    if (fixed.length < sides.length) return Extent.any;
    return Extent.fixed(fixed.reduce((a, b) => a + b) + gap);
  }

  /// Pixels for [first] when this split is laid out in [size] pixels. A side
  /// with a fixed catalog extent gets exactly that plus the user's [offset];
  /// when both are fixed, [first] keeps its extent and [second] takes the
  /// slack. Only two elastic sides share by [ratio].
  double firstExtent(double size, {double least = 40}) {
    final room = size - gap;
    final a = first!.extent(axis!), b = second!.extent(axis!);
    double keep(double px) => px.clamp(least, math.max(least, room - least));
    if (a.isFixed) return keep(a.px + offset);
    if (b.isFixed) return room - keep(b.px - offset);
    return math.max(least, room * ratio);
  }

  /// Applies a divider drag of [delta] pixels along the axis.
  void drag(double delta, double size) {
    final a = first!.extent(axis!), b = second!.extent(axis!);
    if (a.isFixed || b.isFixed) {
      offset += delta;
    } else {
      ratio = (ratio + delta / size).clamp(.1, .9);
    }
  }

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
          'offset': offset,
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
      )..offset = (m['offset'] as num? ?? 0).toDouble();
    final tabs = (m['tabs'] as List)
        .whereType<String>()
        .map((name) => name == 'Test' ? 'Inspector' : name)
        .toSet()
        .where(paneNames.contains)
        .toList();
    // A layout saved before the Files shelf existed gains it beside the
    // other shelves, where the default dock keeps it.
    if ('${m['id']}' == 'browser' &&
        !tabs.contains('Files') &&
        paneNames.contains('Files'))
      tabs.add('Files');
    return DockNode.leaf(
      '${m['id']}',
      tabs,
      active: m['active'] == 'Test' ? 'Inspector' : m['active'] as String?,
    );
  }
}

DockNode initialDock() => DockNode.split(
  Axis.vertical,
  .68,
  DockNode.split(
    Axis.horizontal,
    .205,
    DockNode.leaf('browser', ['Create', 'Media', 'Effects', 'Colors', 'Files']),
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
    target.offset = 0;
    target.first = (edge == 'left' || edge == 'top') ? next : old;
    target.second = (edge == 'left' || edge == 'top') ? old : next;
    target.tabs = [];
    target.active = '';
  }
}
