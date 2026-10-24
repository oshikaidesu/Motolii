// Part of session.dart: Session's calls as they were, moved out of its body by size into an extension on it in
// the same library (its private names and its callers as before).
part of 'session.dart';

extension SessionCalls on Session {
  /// Delete: every picked key, or else the key under the playhead. One undo step.
  void deleteKey() {
    final doomed = <LayerM, List<int>>{};
    for (final id in keySel) {
      final i = id.indexOf(':');
      doomed.putIfAbsent(byId(id.substring(0, i)), () => []).add(int.parse(id.substring(i + 1)));
    }
    final p = primary;
    if (doomed.isEmpty && p != null && keyAtHead) doomed[p] = [frame];
    if (doomed.isEmpty) return;
    _world(() {
      for (final e in doomed.entries) {
        e.value.forEach(e.key.pk.remove);
      }
      keySel.clear();
    });
  }

  /// A press on a key: it becomes the picked key (with [add]: toggled in the picked set); its layer is selected.
  void pickKey(String id, int f, {bool add = false}) {
    final l = byId(id);
    if (add) {
      if (!isSelected(id)) select(id, add: true);
      final k = Session.kid(l, f);
      keySel.contains(k) ? keySel.remove(k) : keySel.add(k);
    } else {
      select(id);
      keySel.add(Session.kid(l, f));
    }
    _changed();
  }

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

  /// The marquee: the layers and keys inside it become the selection (with [add], they join it).
  void marquee(Set<String> layerIds, Set<String> keys, {required bool add}) {
    if (!add) {
      _sel.clear();
      keySel.clear();
    }
    for (final id in layerIds) {
      if (!_sel.contains(id)) _sel.add(id);
    }
    keySel.addAll(keys);
    keyRing = false;
    _syncGroup();
    _changed();
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

  /// A change to bars or keys made outside a drag (Delete): snapshot, change, one step.
  void _world(VoidCallback change) {
    final b = {for (final l in layers) l: TlBase(l.start, l.end, Map.of(l.pk))};
    change();
    if (_timeChanged(b)) {
      _undo.add(_Step.world(b));
      for (final l in layers) {
        l.committedKeys = Map.of(l.pk);
      }
      _applyAll();
    }
    _changed();
  }

  void _edit(LayerM l, VoidCallback change) {
    final before = Map.of(l.committed), beforeKeys = Map.of(l.committedKeys);
    change();
    _undo.add(_Step(l, before, beforeKeys));
    l.committedKeys = Map.of(l.pk);
    _applyAll();
    _changed();
  }

  /// Every keyed layer takes its value at the playhead (not a user edit: no undo step, no key written).
  void _applyAll() {
    for (final l in layers) {
      if (l.pk.isEmpty || l.doc.live) continue;
      final p = l.posAt(frame);
      l.doc.v
        ..['pos.x'] = p.dx
        ..['pos.y'] = p.dy;
      l.committed
        ..['pos.x'] = p.dx
        ..['pos.y'] = p.dy;
    }
    if (_sel.length > 1 && !group.live) _syncGroup();
  }

  /// One step back: the last finished gesture, whichever panel it came from.
  bool undo() {
    if (_undo.isEmpty) return false;
    final s = _undo.removeLast();
    keySel.clear();
    if (s.fn != null) {
      s.fn!();
      _syncGroup();
      _changed();
      return true;
    }
    if (s.world != null) {
      _restore(s.world!);
      _changed();
      return true;
    }
    final layer = s.layer!;
    layer.doc.v
      ..clear()
      ..addAll(s.before);
    layer.pk
      ..clear()
      ..addAll(s.beforeKeys);
    layer.committed = Map.of(s.before);
    layer.committedKeys = Map.of(s.beforeKeys);
    if (layer == primary) _pPrev = Map.of(s.before);
    layer.doc.poke();
    _syncGroup();
    return true;
  }

  // ---- the one undo list: any finished change of a layer's document is a step, however it was made ----------------------------

  // ---- W5: several layers selected. A scrub adds the same delta to each, a typed value sets the absolute value on all, one gesture is one step ----

  List<LayerM> get _selLayers => [for (final id in _sel) byId(id)];

  void _syncGroup() {
    final p = primary;
    if (_sel.length < 2 || p == null || group.live) return;
    final ts = _selLayers;
    group.v
      ..clear()
      ..addAll(p.doc.v);
    group.mixed
      ..clear()
      ..addAll([for (final id in p.doc.v.keys) if (ts.any((l) => !Session._near(l.doc.v[id] ?? 0, p.doc.v[id]!))) id]);
    _gPrev = Map.of(group.v);
    _gMixedPrev = Set.of(group.mixed);
    _pPrev = Map.of(p.doc.v);
  }

  /// An edit made in the Inspector's fields (or its pad, dial, handles) while several layers are selected.
  void _onGroup() {
    final ts = _selLayers;
    if (ts.length < 2) return;
    for (final e in group.v.entries) {
      final old = _gPrev[e.key] ?? e.value, was = _gMixedPrev.contains(e.key), now = group.mixed.contains(e.key);
      if (Session._near(old, e.value) && was == now) continue;
      for (final l in ts) {
        l.doc.v[e.key] = (was && !now) ? e.value : (l.doc.v[e.key] ?? 0) + (e.value - old);
      }
    }
    _gPrev = Map.of(group.v);
    _gMixedPrev = Set.of(group.mixed);
    if (!group.live) {
      _commitGroup(ts);
      _syncGroup();
    }
    _changed();
  }

  /// An edit made on the Stage gizmo, which edits the primary's own document: every other selected layer receives the same delta.
  void _fromPrimary(LayerM p) {
    final ts = _selLayers;
    for (final e in p.doc.v.entries) {
      final old = _pPrev[e.key] ?? e.value;
      if (Session._near(old, e.value)) continue;
      for (final l in ts) {
        if (l != p) l.doc.v[e.key] = (l.doc.v[e.key] ?? 0) + (e.value - old);
      }
    }
    _pPrev = Map.of(p.doc.v);
    if (!p.doc.live) {
      _commitGroup(ts);
      _syncGroup();
    }
    _changed();
  }

  void _commitGroup(List<LayerM> ts) {
    final changed = ts.any((l) => l.doc.v.entries.any((e) => !Session._near(e.value, l.committed[e.key] ?? e.value)));
    if (!changed) {
      for (final l in ts) {
        l.doc.v
          ..clear()
          ..addAll(l.committed); // an Esc leaves exactly what was there, not what arithmetic gives back
      }
      return;
    }
    final before = {for (final l in ts) l: TlBase(l.start, l.end, Map.of(l.committedKeys), Map.of(l.committed))};
    for (final l in ts) {
      if (l.pk.isNotEmpty && (!Session._near(l.doc['pos.x'], l.committed['pos.x']!) || !Session._near(l.doc['pos.y'], l.committed['pos.y']!))) l.pk[frame] = Offset(l.doc['pos.x'], l.doc['pos.y']);
    }
    _undo.add(_Step.world(before));
    for (final l in ts) {
      l.committed = Map.of(l.doc.v);
      l.committedKeys = Map.of(l.pk);
    }
  }

  void _onDoc(LayerM l) {
    if (_sel.length > 1 && l == primary) {
      _fromPrimary(l);
      return;
    }
    if (!l.doc.live && Session._differs(l.doc.v, l.committed)) {
      // a keyed layer edited at the playhead writes its key there (the first key is the key mark's job): the gesture and its key are one step
      if (l.pk.isNotEmpty && (l.doc['pos.x'] != l.committed['pos.x'] || l.doc['pos.y'] != l.committed['pos.y'])) l.pk[frame] = Offset(l.doc['pos.x'], l.doc['pos.y']);
      _undo.add(_Step(l, l.committed, l.committedKeys));
      l.committed = Map.of(l.doc.v);
      l.committedKeys = Map.of(l.pk);
    }
    _changed();
  }
}
