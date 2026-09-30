import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/foundation.dart'
    show ValueListenable, kDebugMode, listEquals, mapEquals;
import 'package:flutter/gestures.dart';
import 'package:flutter/scheduler.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter/services.dart';

import '../session/editor_session.dart';
import '../controls/panel.dart';
import '../theme/editor_theme.dart';
import '../theme/material_icons.dart';
import '../theme/metrics.dart';
import 'geometry.dart';
import 'session.dart';

part 'overlay.dart';
part 'view.dart';
part 'window.dart';
part 'spatial.dart';
part 'camera.dart';
part 'touch.dart';
part 'chrome.dart';

/// What a host's own toolbar needs from the Stage: the view/zoom operations the bars above and below the picture
/// call (Classic's own `_button` row), and the values they show. Nothing about the picture itself, the gesture
/// surface or the gizmos — those stay exactly as they are, whichever toolbar is shown over them.
abstract class StageToolbarApi implements Listenable {
  bool get userStage;
  bool get canGoHome; // false: "Front" is already where it is, greyed
  void goHome();
  void fit();
  void resetZoom();
  void zoomOut();
  void zoomIn();
  double get zoomPercent;
  void setZoomPercent(double percent);
  double get width;
  double get height;
  bool get transparentGround;
  bool get canToggleGround;
  void toggleGround();
  bool get extending;
  void toggleExtend();
  int get frame;
  bool get gesturesAvailable;
}

/// One tab per view: `Stage` looks through the observer, `Camera` through the
/// document camera. The tab that is showing tells native which one to draw.
class StagePanel extends StatefulWidget {
  const StagePanel({super.key, required this.controller, this.view = 'User', this.topBar, this.bottomBar});
  final EditorSession controller;
  final String view;

  /// A host's own bars over the picture (New's, in hf presentation): given the same [StageToolbarApi] Classic's
  /// bars use. Null keeps Classic's own bars.
  final Widget Function(BuildContext context, StageToolbarApi api)? topBar;
  final Widget Function(BuildContext context, StageToolbarApi api)? bottomBar;
  @override
  State<StagePanel> createState() => _StagePanelState();
}

/// What one finger is holding: which pointer it is, where it went down, and
/// whether the gesture it started is already being put away. Both the layer
/// gestures and the camera grabs read it, so it is the seam between them.
mixin _StageGrip on State<StagePanel> {
  int? _pointer;
  Offset? _startScreen, _lastScreen, _startComp;
}

/// The Stage is one responsibility per mixin: what it looks at, the window it
/// asks native to draw, the 3D gizmo it hit-tests, the camera and working area
/// it grabs, the hand on the picture, and the widgets that put it together.
class _StagePanelState extends State<StagePanel>
    with
        _StageView,
        _StageGrip,
        _StageWindow,
        _StageSpatial,
        _StageCamera,
        _StageTouch,
        _StageChrome {
  void _viewChanged() {
    if (!mounted) return;
    setState(() {});
    // a pan or zoom is the person's hand: the window native draws is asked for in this same event, not after the next
    // build (which would put a frame between the hand and the picture)
    _syncWindowFromEvent();
  }

  @override
  void initState() {
    super.initState();
    c.viewCommand.addListener(_viewCommand);
    c.runtimeEpoch.addListener(_runtimeChanged);
    _session.addListener(_viewChanged);
    HardwareKeyboard.instance.addHandler(_heldKey);
  }

  @override
  void dispose() {
    HardwareKeyboard.instance.removeHandler(_heldKey);
    _session.removeListener(_viewChanged);
    // a pointer gone with its tab lets its gesture go
    if (_pointer != null) _finish(true);
    c.viewCommand.removeListener(_viewCommand);
    c.runtimeEpoch.removeListener(_runtimeChanged);
    c.detachView(widget.view);
    if (_userStage && _sentWindow != null && c.supports('stageWindow'))
      c.command('stageWindow', {'width': 0, 'height': 0});
    _focus.dispose();
    super.dispose();
  }
}
