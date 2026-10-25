// session_timeline.dart: an edit on the timeline as one step: begun, bars and keys set, snapped, kept or put back; the timeline's view.
part of 'session.dart';

extension SessionTimeline on Session {
  // ---- W4: one timeline gesture = begin, any number of sets, then commit (one undo step) or cancel (everything as it was) --------

  bool get tlActive => _tlBase != null;

  Map<LayerM, TlBase> get tlBase => _tlBase!;

  Set<String> get tlKeySel => _tlKeySel;

  void tlBegin() {
    _tlBase = {for (final l in layers) l: TlBase(l.start, l.end, Map.of(l.pk))};
    _tlKeySel = Set.of(keySel);
  }

  void tlSet(LayerM l, {double? start, double? end, Map<int, Offset>? pk, Set<String>? picked}) {
    if (_tlBase == null) return;
    if (start != null) l.start = start;
    if (end != null) l.end = end;
    if (pk != null) {
      l.pk
        ..clear()
        ..addAll(pk);
    }
    if (picked != null) {
      keySel
        ..clear()
        ..addAll(picked);
    }
    _applyAll();
    _changed();
  }

  void snap(int? f) {
    if (snapFrame == f) return;
    snapFrame = f;
    _changed();
  }

  void tlCommit() {
    final b = _tlBase;
    _tlBase = null;
    snapFrame = null;
    note = null;
    if (b != null && _timeChanged(b)) {
      _undo.add(_Step.world(b));
      for (final l in layers) {
        l.committedKeys = Map.of(l.pk);
      }
    }
    _changed();
  }

  bool tlCancel() {
    final b = _tlBase;
    if (b == null) return false;
    _restore(b);
    keySel
      ..clear()
      ..addAll(_tlKeySel);
    _tlBase = null;
    snapFrame = null;
    note = null;
    _changed();
    return true;
  }

  void view({double? ppf, double? scroll}) {
    if (ppf != null) this.ppf = ppf;
    if (scroll != null) this.scroll = scroll;
    _changed();
  }

  bool _timeChanged(Map<LayerM, TlBase> b) => b.entries.any((e) => e.key.start != e.value.start || e.key.end != e.value.end || !Session._sameKeys(e.key.pk, e.value.pk));

  void _restore(Map<LayerM, TlBase> b) {
    for (final e in b.entries) {
      e.key.start = e.value.start;
      e.key.end = e.value.end;
      e.key.pk
        ..clear()
        ..addAll(e.value.pk);
      e.key.committedKeys = Map.of(e.value.pk);
      final v = e.value.values;
      if (v != null) {
        e.key.doc.v
          ..clear()
          ..addAll(v);
        e.key.committed = Map.of(v);
      }
    }
    _applyAll();
    _syncGroup();
  }
}
