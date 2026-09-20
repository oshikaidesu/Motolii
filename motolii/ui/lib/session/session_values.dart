part of 'editor_session.dart';

String effectsNotice(Map<String, dynamic> status) {
  final errors = (status['catalogErrors'] as List? ?? const [])
      .map((e) => '$e')
      .where((e) => e.isNotEmpty)
      .toList();
  return errors.isEmpty ? '' : 'Effects: ${errors.join('; ')}';
}

/// A slice of the status. Panels listen to the keys they read, so a drag that
/// moves `layers` leaves Fonts, History and the Browser alone. `derived` names
/// what a panel reads through a getter (the active layer, say) rather than a
/// key; it is compared by value.
class DocumentSlice extends ChangeNotifier
    implements ValueListenable<Map<String, dynamic>> {
  DocumentSlice._(this._session, this._keys, this._derived) {
    _last = _derived?.call();
  }
  final SessionCore _session;
  final Set<String> _keys;
  final Object? Function()? _derived;
  Object? _last;
  @override
  Map<String, dynamic> get value => _session.state;
  void _consider(Set<String> changed) {
    final now = _derived?.call();
    final moved = !sameValue(_last, now);
    _last = now;
    if (moved || _keys.any(changed.contains)) notifyListeners();
  }
}

/// Value equality over decoded JSON.
bool sameValue(Object? a, Object? b) {
  if (identical(a, b)) return true;
  if (a is List) {
    if (b is! List || a.length != b.length) return false;
    for (var i = 0; i < a.length; i++) {
      if (!sameValue(a[i], b[i])) return false;
    }
    return true;
  }
  if (a is Map) {
    if (b is! Map || a.length != b.length) return false;
    for (final entry in a.entries) {
      if (!b.containsKey(entry.key) || !sameValue(entry.value, b[entry.key]))
        return false;
    }
    return true;
  }
  return a == b;
}
