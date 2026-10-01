import 'package:flutter/widgets.dart';

import '../../../session/editor_session.dart';
import '../../../session/read_model.dart';

const kRed = Color(0xFFFF4D3D);
const kInk2 = Color(0xFFC4C6CB);

/// The source of a relation: one component of one property of one thing.
class RelationSource {
  const RelationSource(this.layer, this.property, this.component);
  final int layer;
  final String property;
  final int component;

  factory RelationSource.fromDraft(Map<String, dynamic> d) => RelationSource(d['layer'] as int, '${d['property']}', (d['component'] as int?) ?? 0);

  String name(EditorSession c) {
    final l = c.layers.where((l) => l['id'] == layer).firstOrNull;
    final row = l == null ? null : panelRows(l['properties']).where((r) => r['id'] == property).firstOrNull;
    final axis = row?['value'] is List ? ' ${const ['X', 'Y', 'Z'][component.clamp(0, 2)]}' : '';
    return '${l?['name'] ?? 'Thing $layer'} · ${row?['label'] ?? labelOf(property)}$axis';
  }

  @override
  bool operator ==(Object o) => o is RelationSource && o.layer == layer && o.property == property && o.component == component;
  @override
  int get hashCode => Object.hash(layer, property, component);
}

class Mapping {
  const Mapping(this.property, this.outMin, this.outMax);
  final String property;
  final double outMin, outMax; // in the document's units
}

/// A relation as the links show it: one source and input range, the things it drives, and what it drives on them.
class Relation {
  Relation(this.source, this.inMin, this.inMax, this.members, this.mappings);
  final RelationSource source;
  final double inMin, inMax;
  final List<int> members;
  final List<Mapping> mappings;
}

/// The relations the document holds, read off the status: every link with the remap kind, grouped by its source and
/// input range; the members are the layers that carry such a link, the mappings the properties they carry it on.
List<Relation> relationsOf(EditorSession c) {
  final groups = <String, Relation>{};
  for (final l in c.layers) {
    for (final r in panelRows(l['properties'])) {
      final link = r['link'];
      if (link is! Map || link['kind'] != 'motolii.link.remap') continue;
      final src = RelationSource(link['layer'] as int, '${link['property']}', (link['component'] as num?)?.toInt() ?? 0);
      final inMin = (link['inMin'] as num?)?.toDouble() ?? 0, inMax = (link['inMax'] as num?)?.toDouble() ?? 1;
      final key = '${src.layer}|${src.property}|${src.component}|$inMin|$inMax';
      final rel = groups.putIfAbsent(key, () => Relation(src, inMin, inMax, [], []));
      final id = l['id'] as int;
      if (!rel.members.contains(id)) rel.members.add(id);
      final prop = '${r['id']}';
      if (!rel.mappings.any((m) => m.property == prop)) rel.mappings.add(Mapping(prop, (link['outMin'] as num?)?.toDouble() ?? 0, (link['outMax'] as num?)?.toDouble() ?? 1));
    }
  }
  return groups.values.toList();
}

/// What v0 can drive: the four transform rows every thing has.
const destinations = ['scale', 'rotation', 'opacity', 'position'];

String labelOf(String p) => const {'scale': 'Scale', 'rotation': 'Rotation', 'opacity': 'Opacity', 'position': 'Position'}[p] ?? p;

/// How a destination is shown against how it is stored: Scale and Opacity are percent of a ratio, Rotation degrees, Position px.
class Unit {
  const Unit(this.suffix, this.factor, this.defaultRange);
  final String suffix;
  final double factor;
  final (double, double) defaultRange; // shown units
  double toDoc(double shown) => shown / factor;
  double fromDoc(double doc) => doc * factor;
}

Unit unitOf(String p) => switch (p) {
      'scale' => const Unit('%', 100, (50, 150)),
      'opacity' => const Unit('%', 100, (0, 100)),
      'rotation' => const Unit('°', 1, (-30, 30)),
      _ => const Unit('px', 1, (0, 400)),
    };
