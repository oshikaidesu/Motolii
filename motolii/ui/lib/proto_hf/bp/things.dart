// Thing metadata: a self-describing descriptor, a closed registry, static validation, a small query
// language, and views derived from metadata. Pure Dart (no Flutter) so a CI script can use it.
// Precedents: freedesktop Categories (closed list, X- extensions), AppStream and UXP manifests (stable id,
// validated by a tool), VS Code (closed categories plus free keywords), Blender catalogs (views independent
// of storage), Ableton and AE (views are saved queries), Premiere (attribute filters).
import 'dart:convert';

class Thing {
  Thing.fromJson(Map<String, dynamic> j)
      : id = j['id'] as String,
        name = j['name'] as String,
        kind = j['kind'] as String,
        family = j['family'] as String,
        tags = [for (final t in (j['tags'] as List? ?? const [])) '$t'],
        capabilities = [for (final t in (j['capabilities'] as List? ?? const [])) '$t'],
        source = j['source'] as String,
        searchTerms = [for (final t in (j['searchTerms'] as List? ?? const [])) '$t'],
        face = Map<String, dynamic>.from(j['face'] as Map);
  final String id, name, kind, family, source;
  final List<String> tags, capabilities, searchTerms;
  final Map<String, dynamic> face;
  String get topFamily => family.split('/').first;
}

class FamilyDef {
  FamilyDef(this.id, this.label, {this.section, this.kinds = const {}, this.order = 0});
  final String id, label;
  final String? section; // display grouping only, e.g. text, shapes and paths read as "Primitives"
  final Set<String> kinds; // empty means any kind
  final int order;
  String? get parent => id.contains('/') ? id.substring(0, id.lastIndexOf('/')) : null;
}

class PanelDef {
  PanelDef(this.id, this.title, this.kinds);
  final String id, title;
  final List<String> kinds;
}

class Registry {
  Registry.fromJson(Map<String, dynamic> j)
      : kinds = {for (final k in (j['kinds'] as Map).entries) k.key: (k.value as Map)['label'] as String},
        tags = {for (final t in (j['tags'] as List)) '$t'},
        capabilities = {for (final t in (j['capabilities'] as Map).entries) t.key: '${t.value}'},
        faces = {for (final f in (j['faces'] as Map).entries) '${f.key}': Map<String, dynamic>.from(f.value as Map)},
        panels = {for (final p in (j['panels'] as Map).entries) p.key: PanelDef(p.key, (p.value as Map)['title'] as String, [for (final k in (p.value as Map)['kinds'] as List) '$k'])} {
    var i = 0;
    for (final f in (j['families'] as Map).entries) {
      final m = f.value as Map;
      families[f.key] = FamilyDef(f.key, m['label'] as String, section: m['section'] as String?, kinds: {for (final k in (m['kinds'] as List? ?? const [])) '$k'}, order: i++);
    }
  }
  final Map<String, String> kinds, capabilities;
  final Set<String> tags;
  final Map<String, FamilyDef> families = {};
  final Map<String, Map<String, dynamic>> faces;
  final Map<String, PanelDef> panels;
  String familyLabel(String id) => families[id]?.label ?? id;
}

// ---------------------------------------------------------------------------------- validation

class Issue {
  Issue(this.file, this.thing, this.rule, this.message);
  final String file, thing, rule, message;
  @override
  String toString() => '$file: ${thing.isEmpty ? '-' : thing}: $rule: $message';
}

int _edit(String a, String b) {
  final d = List.generate(a.length + 1, (i) => List<int>.filled(b.length + 1, 0));
  for (var i = 0; i <= a.length; i++) { d[i][0] = i; }
  for (var j = 0; j <= b.length; j++) { d[0][j] = j; }
  for (var i = 1; i <= a.length; i++) {
    for (var j = 1; j <= b.length; j++) {
      d[i][j] = [d[i - 1][j] + 1, d[i][j - 1] + 1, d[i - 1][j - 1] + (a[i - 1] == b[j - 1] ? 0 : 1)].reduce((x, y) => x < y ? x : y);
    }
  }
  return d[a.length][b.length];
}

String _nearest(String s, Iterable<String> options) {
  String? best;
  var bd = 3;
  for (final o in options) {
    final d = _edit(s, o);
    if (d < bd) { bd = d; best = o; }
  }
  return best == null ? '' : ' (did you mean "$best"?)';
}

final _idRe = RegExp(r'^[a-z][a-z0-9]*(\.[a-z][a-z0-9_]*){1,3}$');
final _sourceRe = RegExp(r'^(builtin|plugin:[a-z][a-z0-9]*(\.[a-z][a-z0-9_]*){1,2})$');
final _xTagRe = RegExp(r'^x-[a-z0-9-]{2,24}$');
final _hexRe = RegExp(r'^#[0-9a-fA-F]{6}$');

/// Registry rules: every panel binds known kinds, every family is well formed.
List<Issue> validateRegistry(Registry r) {
  final out = <Issue>[];
  for (final p in r.panels.values) {
    for (final k in p.kinds) {
      if (!r.kinds.containsKey(k)) out.add(Issue('registry', p.id, 'REG-PANEL-KIND', 'panel binds unknown kind "$k"${_nearest(k, r.kinds.keys)}'));
    }
  }
  for (final k in r.kinds.keys) {
    if (!r.panels.values.any((p) => p.kinds.contains(k))) out.add(Issue('registry', k, 'REG-KIND-UNBOUND', 'no panel lists kind "$k", its things would never appear'));
  }
  for (final f in r.families.values) {
    final parent = f.parent;
    if (parent != null && !r.families.containsKey(parent)) out.add(Issue('registry', f.id, 'REG-FAMILY-PARENT', 'parent family "$parent" is not registered'));
    for (final k in f.kinds) {
      if (!r.kinds.containsKey(k)) out.add(Issue('registry', f.id, 'REG-FAMILY-KIND', 'family allows unknown kind "$k"'));
    }
  }
  return out;
}

/// Static checks on raw descriptor maps, so a malformed file is reported rather than thrown.
List<Issue> validateThings(Registry r, Iterable<(String, Map<String, dynamic>)> files, {Set<String> implemented = const {}}) {
  final out = <Issue>[];
  final seen = <String, String>{};
  for (final (file, j) in files) {
    final id = '${j['id'] ?? ''}';
    void bad(String rule, String msg) => out.add(Issue(file, id, rule, msg));
    if (!_idRe.hasMatch(id)) bad('THING-ID-FORMAT', 'id must look like "namespace.name" in lower case; got "$id"');
    if (seen.containsKey(id) && id.isNotEmpty) bad('THING-ID-DUPLICATE', 'id is already used in ${seen[id]}');
    seen.putIfAbsent(id, () => file);
    final name = j['name'];
    if (name is! String || name.trim().isEmpty || name.length > 40) bad('THING-NAME', 'name must be 1 to 40 characters');
    final kind = j['kind'];
    if (kind is! String || !r.kinds.containsKey(kind)) {
      bad('THING-KIND', 'unknown kind "$kind"${kind is String ? _nearest(kind, r.kinds.keys) : ''}');
    }
    final family = j['family'];
    if (family is! String || !r.families.containsKey(family)) {
      bad('THING-FAMILY', 'unknown family "$family"; a new family is registered, never invented in a descriptor${family is String ? _nearest(family, r.families.keys) : ''}');
    } else if (kind is String && r.families[family]!.kinds.isNotEmpty && !r.families[family]!.kinds.contains(kind)) {
      bad('THING-FAMILY-KIND', 'family "$family" does not accept kind "$kind"');
    }
    final tags = j['tags'];
    if (tags is! List) {
      bad('THING-TAGS', 'tags must be a list');
    } else {
      for (final t in tags) {
        if (t is! String || !(r.tags.contains(t) || _xTagRe.hasMatch(t))) {
          bad('THING-TAG-UNKNOWN', 'tag "$t" is not registered and is not an x- extension${t is String ? _nearest(t, r.tags) : ''}');
        }
      }
    }
    final caps = j['capabilities'];
    if (caps is! List) {
      bad('THING-CAPABILITIES', 'capabilities must be a list');
    } else {
      for (final c in caps) {
        if (!r.capabilities.containsKey(c)) {
          bad('THING-CAPABILITY-UNKNOWN', 'capability "$c" is not registered${c is String ? _nearest(c, r.capabilities.keys) : ''}');
        } else if (implemented.isNotEmpty && !implemented.contains('$kind:$c')) {
          bad('THING-CAPABILITY-UNIMPLEMENTED', 'capability "$c" is declared for kind "$kind" but nothing implements it');
        }
      }
    }
    final source = j['source'];
    if (source is! String || !_sourceRe.hasMatch(source)) bad('THING-SOURCE', 'source must be "builtin" or "plugin:vendor.pack"; got "$source"');
    final terms = j['searchTerms'];
    final tagsOk = tags is List && tags.isNotEmpty;
    if (terms is! List || (terms.isEmpty && !tagsOk)) bad('THING-NOT-SEARCHABLE', 'declare searchTerms or at least one tag so it can be found beyond its name');
    if (terms is List && terms.length > 12) bad('THING-SEARCHTERMS-LONG', 'at most 12 search terms');
    final face = j['face'];
    if (face is! Map) {
      bad('THING-FACE-MISSING', 'every thing declares a face');
    } else {
      final type = face['type'];
      final def = r.faces['$type'];
      if (def == null) {
        bad('THING-FACE-TYPE', 'unknown face type "$type"${type is String ? _nearest(type, r.faces.keys) : ''}');
      } else {
        for (final p in def.entries) {
          final v = face[p.key];
          final spec = p.value;
          if (v == null) {
            bad('THING-FACE-PARAM', 'face "$type" needs "${p.key}"');
          } else if (spec is List && !spec.contains(v)) {
            bad('THING-FACE-PARAM', 'face "$type" ${p.key} "$v" is not one of the allowed values${_nearest('$v', spec.map((e) => '$e'))}');
          } else if (spec == 'hex' && !(v is String && _hexRe.hasMatch(v))) {
            bad('THING-FACE-PARAM', 'face "$type" ${p.key} must be #rrggbb');
          } else if (spec == 'number' && v is! num) {
            bad('THING-FACE-PARAM', 'face "$type" ${p.key} must be a number');
          }
        }
      }
    }
  }
  return out;
}

// -------------------------------------------------------------------------------------- query

/// `kind:effect family:physics tag:spring sphere`. Unknown keys are ordinary words.
class ThingQuery {
  ThingQuery(this.words, this.filters);
  final List<String> words;
  final Map<String, Set<String>> filters;
  static const keys = {'kind', 'family', 'tag', 'cap', 'source'};

  factory ThingQuery.parse(String text) {
    final words = <String>[];
    final filters = <String, Set<String>>{};
    for (final t in text.toLowerCase().split(RegExp(r'\s+'))) {
      if (t.isEmpty) continue;
      final i = t.indexOf(':');
      if (i > 0 && i < t.length - 1 && keys.contains(t.substring(0, i))) {
        (filters[t.substring(0, i)] ??= {}).add(t.substring(i + 1));
      } else {
        words.add(t);
      }
    }
    return ThingQuery(words, filters);
  }

  bool get isEmpty => words.isEmpty && filters.isEmpty;

  bool matches(Thing t, Registry r) {
    for (final e in filters.entries) {
      final any = e.value.any((v) => switch (e.key) {
            'kind' => t.kind == v,
            'family' => t.family == v || t.family.startsWith('$v/'),
            'tag' => t.tags.contains(v),
            'cap' => t.capabilities.contains(v),
            'source' => t.source == v || t.source.startsWith('$v:'),
            _ => false,
          });
      if (!any) return false;
    }
    final hay = [t.name, ...t.searchTerms, ...t.tags, r.familyLabel(t.topFamily), t.kind].map((e) => e.toLowerCase()).toList();
    return words.every((w) => hay.any((h) => h.contains(w)));
  }
}

// ---------------------------------------------------------------------------------------- views

/// What a person keeps: not metadata of the thing, so it lives beside it (Blender, Ableton, Premiere).
class UserViews {
  UserViews({Set<String>? favorites, List<String>? recent, Map<String, String>? saved, Map<String, Set<String>>? collections})
      : favorites = favorites ?? {},
        recent = recent ?? [],
        saved = saved ?? {},
        collections = collections ?? {};
  final Set<String> favorites;
  final List<String> recent;
  final Map<String, String> saved; // name -> query text
  final Map<String, Set<String>> collections; // name -> ids
}

/// Views derived from metadata: system views from families present in scope, user views from the store.
class ThingViews {
  ThingViews(this.registry, this.things, this.panel, this.user) : scope = [for (final t in things) if (panel.kinds.contains(t.kind)) t];
  final Registry registry;
  final List<Thing> things, scope;
  final PanelDef panel;
  final UserViews user;
  static const all = 'All';

  /// Families that have something in [within], in registry order, then the user's own views.
  List<List<String>> groups(Iterable<Thing> within) {
    final present = {for (final t in within) t.topFamily};
    final fams = registry.families.values.where((f) => f.parent == null && present.contains(f.id)).toList()..sort((a, b) => a.order.compareTo(b.order));
    return [
      [all, for (final f in fams) f.label],
      ['Favorites', 'Recent', ...user.collections.keys, ...user.saved.keys],
    ];
  }

  bool contains(String view, Thing t) {
    if (view == all) return true;
    if (view == 'Favorites') return user.favorites.contains(t.id);
    if (view == 'Recent') return user.recent.contains(t.id);
    if (user.collections.containsKey(view)) return user.collections[view]!.contains(t.id);
    if (user.saved.containsKey(view)) return ThingQuery.parse(user.saved[view]!).matches(t, registry);
    final fam = registry.families.values.where((f) => f.parent == null && f.label == view);
    return fam.isNotEmpty && t.topFamily == fam.first.id;
  }

  String sectionOf(Thing t) {
    final f = registry.families[t.topFamily];
    return f?.section ?? f?.label ?? t.family;
  }
}

// ---------------------------------------------------------------------------------------- catalog

class Catalog {
  Catalog(this.registry, this.files);
  final Registry registry;
  final List<(String, Map<String, dynamic>)> files;
  List<Thing> get things => [for (final (_, j) in files) Thing.fromJson(j)];
  List<Issue> validate({Set<String> implemented = const {}}) => [...validateRegistry(registry), ...validateThings(registry, files, implemented: implemented)];

  static Catalog parse(String registryJson, Map<String, String> thingFiles) {
    final reg = Registry.fromJson(jsonDecode(registryJson) as Map<String, dynamic>);
    final all = <(String, Map<String, dynamic>)>[];
    final names = thingFiles.keys.toList()..sort();
    for (final n in names) {
      final list = (jsonDecode(thingFiles[n]!) as Map<String, dynamic>)['things'] as List;
      for (final t in list) {
        all.add((n, Map<String, dynamic>.from(t as Map)));
      }
    }
    return Catalog(reg, all);
  }
}
