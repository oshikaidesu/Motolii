// Declaration rows and the kind decision, kept from the Classic Inspector:
// what a control is comes from the declaration, never from the label; unknown means generic.
import 'dart:convert';
import 'dart:math' as math;
import 'package:flutter/foundation.dart';

enum PKind { scalar, bounded, integer, toggle, choice, vec2, vec3, pair, text, reference, route, raw }

bool _isNumList(Object? v, int n) => v is List && v.length == n && v.every((e) => e is num);

/// Kind guarantees access: every row lands on one of these, an unrecognised one on [PKind.raw].
PKind kindOf(Map<String, dynamic> r) {
  final kind = '${r['kind']}';
  final v = r['value'];
  if (kind == 'pair') return PKind.pair; // two declared rows read as one point
  final numeric = v is num || _isNumList(v, 2) || _isNumList(v, 3);
  // The host owns a specialist for it (Colors, Fonts, Blend, Ease): show the value and hand over.
  // A number that merely has a specialist view (Position -> Depth) stays an editable Value with a route beside it.
  if (r['route'] is String && !numeric) return PKind.route;
  if (const {'color', 'font', 'blend', 'ease'}.contains(kind)) return PKind.route;
  if (r['layer'] == true || kind == 'layer' || kind == 'resource') return PKind.reference;
  if (r['choices'] is List || kind == 'enum') return PKind.choice;
  if (kind == 'bool' || v is bool) return PKind.toggle;
  if (kind == 'text' || kind == 'string') return PKind.text;
  if (kind == 'vec2' || _isNumList(v, 2)) return PKind.vec2;
  if (kind == 'vec3' || _isNumList(v, 3)) return PKind.vec3;
  if (kind == 'i32' || kind == 'u32' || kind == 'int') return PKind.integer;
  if (v is num) return r['min'] != null && r['max'] != null ? PKind.bounded : PKind.scalar;
  if (v is String) return PKind.text;
  return PKind.raw;
}

double span(Map<String, dynamic> r) => ((r['max'] as num?)?.toDouble() ?? 0) - ((r['min'] as num?)?.toDouble() ?? 0);

/// A range that is a real reach (0..1, octaves 1..8), not a guard (0..100000): only a real reach earns a track.
/// Geometry reflects meaning, not storage constraints.
bool tight(Map<String, dynamic> r) {
  if (r['min'] == null || r['max'] == null) return false;
  final v = (r['value'] as num?)?.toDouble() ?? 0;
  final d = (r['default'] as num?)?.toDouble() ?? v;
  return span(r) <= 20 * math.max(d.abs(), 1);
}

bool sameValue(Object? a, Object? b) => a is List && b is List ? listEquals(a, b) : a == b;
bool modified(Map<String, dynamic> r) => r.containsKey('default') && !sameValue(r['value'], r['default']);

/// The rows and every edit made to them. A drag is many previews and one commit; the counters let tests say so.
class ParamStore extends ChangeNotifier {
  ParamStore(List<Map<String, dynamic>> rows, {this.frozen = false}) : rows = [for (final r in rows) Map.of(r)];
  final List<Map<String, dynamic>> rows;
  bool frozen;
  bool typing = false; // a typed number is absolute for every target; a scrub is relative
  int commits = 0, previews = 0;
  bool mixed(String id, int? axis) => false; // does another selected target disagree on this number?
  /// A store over a document can key a value at the current frame; the in-memory store cannot.
  bool get keyable => false;
  void toggleKey(String id) {}

  final routes = <String>[];
  final routeFrom = <String>[]; // which property each hand-over came from, so the specialist can return to it
  final actions = <String>[];
  final linked = <String>{}; // vector or pair ids whose axes move together
  void toggleLink(String id) {
    linked.contains(id) ? linked.remove(id) : linked.add(id);
    notifyListeners();
  }


  Map<String, dynamic> row(String id) => rows.firstWhere((r) => r['id'] == id);

  /// Hard min / max are a storage guard: clamp silently, and round whole numbers.
  Object? _fit(Map<String, dynamic> r, Object? v) {
    if (v is! num) return v;
    var d = v.toDouble();
    if (r['min'] is num) d = math.max(d, (r['min'] as num).toDouble());
    if (r['max'] is num) d = math.min(d, (r['max'] as num).toDouble());
    final ch = characterOf(r);
    if (ch == Character.count && r['min'] == null) d = math.max(d, 0); // a count is never negative
    final whole = const ['i32', 'u32', 'int'].contains('${r['kind']}') || ch == Character.count || ch == Character.seed;
    return whole ? d.round() : d;
  }

  void preview(String id, Object? v) {
    if (frozen) return;
    final r = row(id);
    r['value'] = v is List ? [for (final e in v) _fit(r, e)] : _fit(r, v);
    previews++;
    notifyListeners();
  }

  void commit(String id) {
    if (frozen) return;
    commits++;
    notifyListeners();
  }

  void set(String id, Object? v) {
    preview(id, v);
    commit(id);
  }

  void reset(String id) {
    final r = row(id);
    if (!r.containsKey('default')) return;
    final d = r['default'];
    set(id, d is List ? List.of(d) : d);
  }

  void route(String to, [String? from]) {
    routes.add(to);
    if (from != null) routeFrom.add(from);
    notifyListeners();
  }

  void act(String id, String name) {
    if (frozen) return;
    actions.add('$id:$name');
    // Reroll is the one action whose meaning is known: the next value of a simple sequence, deterministic
    if (name == 'Reroll') {
      final v = (row(id)['value'] as num).toInt();
      set(id, (v * 1664525 + 1013904223) & 0x7FFFFFFF);
      return;
    }
    notifyListeners();
  }

  static String raw(Object? v) => jsonEncode(v);
}

/// What a scalar is *for*, only when the declaration says so (`subtype` or `character`). No guessing from names:
/// a row called "angle" that declares nothing is a plain Value. A small vocabulary: only what changes how the Value behaves.
enum Character { angle, seed, count, opacity, position, scale, none }

Character characterOf(Map<String, dynamic> r) {
  final said = '${r['character'] ?? r['subtype'] ?? ''}'.toLowerCase();
  final c = switch (said) {
    'angle' => Character.angle,
    'seed' => Character.seed,
    'count' => Character.count,
    'opacity' => Character.opacity,
    'position' || 'translation' => Character.position,
    'scale' => Character.scale,
    _ => Character.none,
  };
  if (c == Character.none) return c;
  if (c == Character.position || c == Character.scale) return c; // groups; their numbers stay plain Values
  if (r['value'] is! num) return Character.none;
  // a finite bar needs the declared range to agree that 0..1 is the whole domain
  if (c == Character.opacity && !_finite01(r)) return Character.none;
  return c;
}

bool _finite01(Map<String, dynamic> r) => (r['min'] as num?)?.toDouble() == 0 && (r['max'] as num?)?.toDouble() == 1;

/// Two declared rows that share a `group` are one point (Position, Scale): one row of kind `pair`.
List<Map<String, dynamic>> foldPairs(List<Map<String, dynamic>> rows) {
  final out = <Map<String, dynamic>>[];
  for (var i = 0; i < rows.length; i++) {
    final a = rows[i];
    final b = i + 1 < rows.length ? rows[i + 1] : null;
    final same = b != null && a['group'] is String && a['group'] == b['group'] && a['value'] is num && b['value'] is num;
    if (same) {
      final ch = characterOf({...a, 'value': 0}) != Character.none ? characterOf({...a, 'value': 0}) : characterOf({...b, 'value': 0});
      out.add({
        'id': '${a['group']}',
        'label': a['groupLabel'] ?? '${a['group']}'.replaceFirstMapped(RegExp('^.'), (m) => m[0]!.toUpperCase()),
        'kind': 'pair',
        'ids': ['${a['id']}', '${b['id']}'],
        'section': a['section'] ?? b['section'],
        if (a['advanced'] == true || b['advanced'] == true) 'advanced': true,
        if (a['hero'] == true || b['hero'] == true) 'hero': true,
        if ((a['route'] ?? b['route']) != null) 'route': a['route'] ?? b['route'],
        if (a['linkable'] == true || b['linkable'] == true || ch == Character.scale) 'linkable': true,
        if ((a['unit'] ?? b['unit']) != null) 'unit': a['unit'] ?? b['unit'],
      });
      i++;
    } else {
      out.add(a);
    }
  }
  return out;
}
