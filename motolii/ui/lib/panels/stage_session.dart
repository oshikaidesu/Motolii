import 'dart:math' as math;
import 'dart:ui' show Offset, Size;

import 'package:flutter/foundation.dart';

import '../session/editor_session.dart';
import 'stage/geometry.dart' show numOf;

/// The Stage as a person works it, with no screen in it: where each view (the User stage, the Camera) is looking —
/// its zoom and the composition point at the middle of its tab — in composition coordinates. A skin brings its own
/// tab size and turns it into pixels; a skin can be thrown away or swapped and the view stays where it was.
class StageSession extends ChangeNotifier {
  StageSession._(this.c, this.view);
  static final _all = Expando<Map<String, StageSession>>();
  static StageSession of(EditorSession c, String view) => (_all[c] ??= {})[view] ??= StageSession._(c, view);

  final EditorSession c;

  /// 'User' (the Stage tab) or 'Camera'.
  final String view;

  double get width => (numOf(c.state['width'], 1920)).clamp(1, double.infinity).toDouble();
  double get height => (numOf(c.state['height'], 1080)).clamp(1, double.infinity).toDouble();

  // ---- where it looks ------------------------------------------------------------------------------------------------
  /// Pixels a composition pixel takes (null: fit the tab), and the composition point at the tab's middle (null: the
  /// composition's middle).
  double? zoom;
  Offset? centre;

  double scaleIn(Size tab) => zoom ?? math.min(math.max(1, tab.width - 32) / width, math.max(1, tab.height - 32) / height);
  Offset originIn(Size tab) => tab.center(Offset.zero) - (centre ?? Offset(width / 2, height / 2)) * scaleIn(tab);
  Offset toComp(Offset screen, Size tab) => (screen - originIn(tab)) / scaleIn(tab);
  Offset toScreen(Offset comp, Size tab) => originIn(tab) + comp * scaleIn(tab);

  void fit() {
    zoom = null;
    centre = null;
    notifyListeners();
    if (view == 'User' && c.supports('stageView')) c.command('stageView', {'fit': true});
  }

  void actual() {
    zoom = 1;
    centre = null;
    notifyListeners();
  }

  /// Zoom to [next] keeping the composition point under [anchor] (a point on the tab) where it is.
  void zoomAt(double next, Offset anchor, Size tab) {
    final point = toComp(anchor, tab);
    final scale = next.clamp(.02, 16.0).toDouble();
    zoom = scale;
    centre = point - (anchor - tab.center(Offset.zero)) / scale;
    notifyListeners();
  }

  /// The picture carried by [delta] on the tab (a hand-pan).
  void panBy(Offset delta, Size tab) {
    centre = (centre ?? Offset(width / 2, height / 2)) - delta / scaleIn(tab);
    notifyListeners();
  }
}
