import 'package:flutter/foundation.dart';

import '../../session/read_model.dart' show panelRows;
import '../../session/editor_session.dart';
import 'model.dart';

/// The Relations desk's work, with no screen in it: the member set being chosen, a relation being drafted (its source,
/// destination and ranges), a range being scrubbed, and the operations that write relations. One per EditorSession.
class RelationsSession extends ChangeNotifier {
  RelationsSession._(this.c) {
    c.relationDraft.addListener(_draftArrived);
    _draftArrived();
  }
  static final _all = Expando<RelationsSession>();
  static RelationsSession of(EditorSession c) => _all[c] ??= RelationsSession._(c);
  final EditorSession c;

  /// The member set being chosen (for a draft, or when a relation's members are being edited).
  Set<int>? picking;

  /// A relation being made: {source, destination, inMin, inMax, outMin, outMax}.
  Map<String, dynamic>? draft;

  /// Tells a skin a new draft arrived (it takes the keys, forgets a lasso half drawn).
  final arrived = ChangeNotifier();

  void _draftArrived() {
    final d = c.relationDraft.value;
    if (d == null) return;
    final value = sourceValue(d);
    draft = {
      'source': Map<String, dynamic>.from(d),
      'destination': null,
      // a range around where the source is now, in its own units
      'inMin': value - 300,
      'inMax': value + 300,
      'outMin': null,
      'outMax': null,
    };
    picking = {};
    notifyListeners();
    // ignore: invalid_use_of_protected_member, invalid_use_of_visible_for_testing_member
    arrived.notifyListeners();
  }

  double sourceValue(Map<String, dynamic> src) {
    final l = c.layers.where((l) => l['id'] == src['layer']).firstOrNull;
    final row = l == null ? null : panelRows(l['properties']).where((r) => r['id'] == src['property']).firstOrNull;
    final v = row?['value'];
    if (v is List) return ((v[src['component'] as int? ?? 0]) as num).toDouble();
    return (v as num?)?.toDouble() ?? 0;
  }

  List<Relation> get relations => relationsOf(c);
  Relation? get focused {
    final f = c.relationFocus.value;
    if (f == null) return null;
    return relations.where((r) => r.source.layer == f['layer'] && r.source.property == f['property'] && r.source.component == (f['component'] ?? 0)).firstOrNull;
  }

  Future<void> relate(RelationSource src, double inMin, double inMax, Set<int> members, String property, double outMin, double outMax) => c.command('relate', {
        'source': {'layer': src.layer, 'property': src.property, 'component': src.component},
        'inMin': inMin,
        'inMax': inMax,
        'members': members.toList()..sort(),
        'property': property,
        'outMin': outMin,
        'outMax': outMax,
      });

  Future<void> create() async {
    final d = draft!;
    final src = RelationSource.fromDraft(d['source'] as Map<String, dynamic>);
    final prop = d['destination'] as String;
    final unit = unitOf(prop);
    await relate(src, d['inMin'], d['inMax'], picking!, prop, unit.toDoc(d['outMin']), unit.toDoc(d['outMax']));
    c.relationDraft.value = null;
    c.relationFocus.value = {'layer': src.layer, 'property': src.property, 'component': src.component};
    draft = null;
    picking = null;
    notifyListeners();
  }

  void cancel() {
    c.relationDraft.value = null;
    draft = null;
    picking = null;
    notifyListeners();
  }

  // ---- a relation's ranges: scrubbed as a draft, written once on release ----------------------------------------
  /// A range being scrubbed (by its key: 'in', or 'out-<property>'), shown until it is let go. A link has no preview,
  /// so the relation is written once, on release, with every destination in one edit (one undo step).
  final ranges = <String, (double, double)>{};
  (double, double) shown(String key, double lo, double hi) => ranges[key] ?? (lo, hi);
  void scrubRange(String key, double lo, double hi) {
    ranges[key] = (lo, hi);
    notifyListeners();
  }

  /// The relation comes off every destination at once (one undo); each thing keeps what it shows.
  Future<void> remove(Relation r) async {
    await c.command('unrelate', {'layers': r.members, 'properties': [for (final m in r.mappings) m.property]});
    c.relationFocus.value = null;
  }

  /// Written on release, every destination at once (a link has no preview). [members]: the new member set; anyone who
  /// left it is let go in the same step.
  Future<void> write(Relation r, {Set<int>? members}) async {
    final (inMin, inMax) = shown('in', r.inMin, r.inMax);
    final gone = members == null ? const <int>[] : [for (final m in r.members) if (!members.contains(m)) m];
    await c.command('relate', {
      'source': {'layer': r.source.layer, 'property': r.source.property, 'component': r.source.component},
      'inMin': inMin,
      'inMax': inMax,
      'members': (members ?? r.members.toSet()).toList()..sort(),
      if (gone.isNotEmpty) 'release': gone,
      'mappings': [
        for (final m in r.mappings)
          () {
            final u = unitOf(m.property);
            final range = ranges['out-${m.property}'];
            final (lo, hi) = range != null ? (u.toDoc(range.$1), u.toDoc(range.$2)) : (m.outMin, m.outMax);
            return {'property': m.property, 'outMin': lo, 'outMax': hi};
          }(),
      ],
    });
    ranges.clear();
    notifyListeners();
  }
}
