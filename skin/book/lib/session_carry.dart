// session_carry.dart: a layer carried on the stage: taken, moved, let go, put back.
part of 'session.dart';

extension SessionCarry on Session {
  /// A press on a box that is not selected yet: select it and carry on as a drag, one gesture (the doc keeps the snapshot).
  void beginMove(LayerM l, Offset at) {
    if (isSelected(l.id) && _sel.length > 1) {
      // a press on a box that is already in the selection drags the whole selection (it becomes the primary, the others follow)
      _sel
        ..remove(l.id)
        ..add(l.id);
      keyRing = false;
      _syncGroup();
      _changed();
    } else {
      select(l.id);
    }
    _moving = l;
    _moveLast = at;
    l.doc.begin();
  }

  /// [k] = composition px per canvas px; Shift is fine (x0.1), the same rule as every other hand.
  void moveTo(Offset at, double k, bool fine) {
    final l = _moving;
    if (l == null) return;
    final dd = (at - _moveLast) * k * (fine ? Gui.fine : 1);
    _moveLast = at;
    l.doc
      ..add('pos.x', dd.dx)
      ..add('pos.y', dd.dy);
  }

  void endMove() {
    _moving?.doc.end();
    _moving = null;
  }

  /// Esc during a drag: everything is where the gesture found it.
  bool cancelMove() {
    if (tlCancel()) return true;
    final l = _moving;
    if (l == null) return false;
    l.doc.cancel();
    _moving = null;
    return true;
  }

  bool get moving => _moving != null;
}
