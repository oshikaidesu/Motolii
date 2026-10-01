import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/foundation.dart'
    show ValueListenable, kDebugMode, listEquals, mapEquals;
import 'package:flutter/gestures.dart';
import 'package:flutter/scheduler.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter/services.dart';

import '../session/editor_session.dart';
import '../foundation/panel_controls.dart';
import '../foundation/theme.dart';
import '../foundation/metrics.dart';
import '../foundation/glyphs.dart';
import 'stage/geometry.dart';

part 'stage/overlay.dart';
part 'stage/view.dart';
part 'stage/window.dart';
part 'stage/spatial.dart';
part 'stage/camera.dart';
part 'stage/touch.dart';
part 'stage/chrome.dart';

/// One tab per view: `Stage` looks through the observer, `Camera` through the
/// document camera. The tab that is showing tells native which one to draw.
class StagePanel extends StatefulWidget {
  const StagePanel({super.key, required this.controller, this.view = 'User'});
  final EditorSession controller;
  final String view;
  @override
  State<StagePanel> createState() => _StagePanelState();
}

/// What one finger is holding: which pointer it is, where it went down, and
/// whether the gesture it started is already being put away. Both the layer
/// gestures and the camera grabs read it, so it is the seam between them.
mixin _StageGrip on State<StagePanel> {
  int? _pointer;
  Offset? _startScreen, _lastScreen, _startComp;
  bool _finishing = false;
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
  @override
  void initState() {
    super.initState();
    c.viewCommand.addListener(_viewCommand);
    c.runtimeEpoch.addListener(_runtimeChanged);
    HardwareKeyboard.instance.addHandler(_heldKey);
  }

  @override
  void dispose() {
    HardwareKeyboard.instance.removeHandler(_heldKey);
    if (_dragging) {
      _pending = null;
      final args = _gesture('cancel', _startComp!);
      _drained.whenComplete(() => c.command('stageGesture', args));
    }
    c.viewCommand.removeListener(_viewCommand);
    c.runtimeEpoch.removeListener(_runtimeChanged);
    c.detachView(widget.view);
    if (_userStage && _sentWindow != null && c.supports('stageWindow'))
      c.command('stageWindow', {'width': 0, 'height': 0});
    _focus.dispose();
    super.dispose();
  }
}
