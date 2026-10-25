// session_picking.dart: what is picked and hovered: layers, keys, a marquee's catch.
part of 'session.dart';

extension SessionPicking on Session {
  bool keyOn(LayerM l, int f) => keySel.contains(Session.kid(l, f));

  List<String> get selected => List.unmodifiable(_sel);

  bool isSelected(String id) => _sel.contains(id);

  LayerM? get primary => _sel.isEmpty ? null : byId(_sel.last);

  // ---- verbs ---------------------------------------------------------------------------------------------------------------

  /// A plain click picks one; with [add] (Shift / Cmd / Ctrl) it adds or removes one.
  void select(String id, {bool add = false, bool key = false}) {
    if (!add) keySel.clear();
    if (add) {
      _sel.contains(id) ? _sel.remove(id) : _sel.add(id);
    } else {
      _sel
        ..clear()
        ..add(id);
    }
    keyRing = key;
    _syncGroup();
    _changed();
  }

  /// Esc or a click on empty ground.
  void clear() {
    keySel.clear();
    if (_sel.isEmpty) {
      _changed();
      return;
    }
    _sel.clear();
    keyRing = false;
    _syncGroup();
    _changed();
  }

  void hover(String? id) {
    if (id == hovered) return;
    hovered = id;
    _changed();
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
}
