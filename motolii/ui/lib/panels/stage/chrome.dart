part of '../stage.dart';

/// Putting the tab together: the bar above, the picture with everything
/// drawn over it, and the bar below. This is the only place that builds
/// widgets; every value it shows is read through the other mixins.
mixin _StageChrome
    on
        State<StagePanel>,
        _StageView,
        _StageWindow,
        _StageSpatial,
        _StageCamera,
        _StageTouch {
  bool _initialFrameRequested = false;

  /// 地が透明か。alpha が 1 未満なら書き出しは alpha を持ち、Stage は市松で見せる。
  bool get _transparentGround {
    final bg = _state['background'];
    return bg is List && bg.length == 4 && numOf(bg[3], 1) < 1;
  }

  /// 透明と solid を往復する。solid へ戻す時は、透明にする前の色ではなく黒
  /// (色は Composition の欄が持つ)。1 ジェスチャ = 1 undo。
  void _toggleGround() {
    final bg = _state['background'];
    final grey = bg is List && bg.length == 4
        ? [numOf(bg[0]), numOf(bg[1]), numOf(bg[2])]
        : [0.0, 0.0, 0.0];
    c.command('composition', {
      'background': [...grey, _transparentGround ? 1.0 : 0.0],
    });
  }

  /// Snap guides arrive as composition x / y; the frame's corners carry
  /// them to the screen (top-left, top-right, bottom-right, bottom-left).
  List<(Offset, Offset)> _snapGuides() {
    final guides = _observer['snapGuides'];
    final frame = [
      for (final p in (_observer['frame'] as List?) ?? []) ?_point(p),
    ];
    if (guides is! List || guides.length < 2 || frame.length != 4) {
      return const [];
    }
    Offset along(Offset a, Offset b, double u) => a + (b - a) * u;
    return [
      if (guides[0] case final num x)
        (
          along(frame[0], frame[1], x / _width),
          along(frame[3], frame[2], x / _width),
        ),
      if (guides[1] case final num y)
        (
          along(frame[0], frame[3], y / _height),
          along(frame[1], frame[2], y / _height),
        ),
    ];
  }

  Widget _button(String title, VoidCallback? onPressed) => EditorButton(
    title,
    onPressed,
    tooltip: switch (title) {
      'Fit' => 'Fit the working area',
      'Extend' => 'Allow the working area edges to be dragged',
      '● Extend' => 'Stop dragging the working area edges',
      '100%' => 'Actual size',
      '−' => 'Zoom out',
      '+' => 'Zoom in',
      'Front' => 'Look straight at the composition (double-click background)',
      _ => title,
    },
  );
  DocumentSlice get _slice => c.slice('stage', const [
    'layers',
    'selectedId',
    'selectedIds',
    'width',
    'height',
    'observer',
    'spatialGizmo',
    'stageSpatialGizmo',
    'cameraGizmos',
    'pickedColor',
    'capabilities',
    'contentRevision',
    'documentRevision',
  ]);

  /// What the bars above and below the picture read. Kept apart from [_slice]
  /// so a moved layer, and every frame drawn while it moves, leaves the bars
  /// standing instead of re-measuring their intrinsic widths.
  DocumentSlice get _chrome => c.slice('stage:chrome', const [
    'observer',
    'width',
    'height',
    'capabilities',
  ]);

  @override
  Widget build(BuildContext context) => !Visibility.of(context)
      ? const SizedBox.shrink()
      : Column(
          children: [
            AnimatedBuilder(
              animation: _chrome,
              builder: (context, _) => EditorBar(
                decoration: BoxDecoration(
                  color: EditorTheme.of(context).panel,
                  border: Border(
                    bottom: BorderSide(color: EditorTheme.of(context).line),
                  ),
                ),
                children: [
                  const SizedBox(width: EditorMetrics.s8),
                  if (_userStage)
                    _button(
                      'Front',
                      _home
                          ? null
                          : () => c.command('stageView', {'reset': true}),
                    ),
                  const Spacer(),
                  _button('Fit', _fit),
                  _button(
                    '100%',
                    () => setState(() {
                      _zoom = 1;
                      _pan = Offset.zero;
                    }),
                  ),
                  _button(
                    '−',
                    () => _zoomAt(
                      ((_scale * 100).round() - 1) / 100,
                      _viewport.center(Offset.zero),
                    ),
                  ),
                  EditorPercentField(
                    value: _scale * 100,
                    min: 2,
                    max: 1600,
                    label: 'Stage zoom',
                    onChanged: (v) =>
                        _zoomAt(v / 100, _viewport.center(Offset.zero)),
                  ),
                  _button(
                    '+',
                    () => _zoomAt(
                      ((_scale * 100).round() + 1) / 100,
                      _viewport.center(Offset.zero),
                    ),
                  ),
                ],
              ),
            ),
            Expanded(
              child: LayoutBuilder(
                builder: (context, box) {
                  final resized = _viewport != box.biggest;
                  _viewport = box.biggest;
                  // The new size goes to native now, in the same frame the
                  // tab was measured; the post-frame path below is the fallback.
                  if (resized) _syncWindow();
                  if (!_initialFrameRequested || resized) {
                    final needsFrame = !_initialFrameRequested;
                    _initialFrameRequested = true;
                    WidgetsBinding.instance.addPostFrameCallback((_) {
                      if (!mounted) return;
                      setState(() {});
                      if (needsFrame) c.refreshPreview();
                    });
                  }
                  return Focus(
                    focusNode: _focus,
                    onKeyEvent: (_, event) {
                      if (event is KeyDownEvent &&
                          event.logicalKey == LogicalKeyboardKey.escape &&
                          (_pointer != null || _dragging)) {
                        _finish(true);
                        return KeyEventResult.handled;
                      }
                      return KeyEventResult.ignored;
                    },
                    child: MouseRegion(
                      // The Camera tab is the picture: cages and handles show
                      // only while the pointer is in it (2026-09-19, user).
                      // The Stage tab is the workbench and always shows them.
                      onEnter: (_) => _setInside(true),
                      onExit: (_) => _setInside(false),
                      child: Listener(
                        behavior: HitTestBehavior.opaque,
                        onPointerDown: _down,
                        onPointerMove: _move,
                        onPointerUp: _up,
                        onPointerHover: _hover,
                        onPointerCancel: (_) => _finish(true),
                        onPointerSignal: (event) {
                          if (event is PointerScrollEvent) {
                            GestureBinding.instance.pointerSignalResolver
                                .register(event, (signal) {
                                  final scroll = signal as PointerScrollEvent;
                                  _zoomAt(
                                    _scale *
                                        math.exp(
                                          -scroll.scrollDelta.dy * .0015,
                                        ),
                                    scroll.localPosition,
                                  );
                                });
                          }
                        },
                        child: ClipRect(
                          child: ColoredBox(
                            color: EditorTheme.of(context).app,
                            // What moves with the frame, the selection and the
                            // zoom is built here; the chrome above is built once
                            // per layout.
                            child: AnimatedBuilder(
                              animation: Listenable.merge([
                                _slice,
                                c.runtimeEpoch,
                                c.rendered,
                                c.textureIds,
                                c.playing,
                                c.anchorPreview,
                              ]),
                              builder: (context, _) => _picture(),
                            ),
                          ),
                        ),
                      ),
                    ),
                  );
                },
              ),
            ),
            AnimatedBuilder(
              animation: _chrome,
              builder: (context, _) => EditorBar(
                padding: const EdgeInsets.symmetric(
                  horizontal: EditorMetrics.s8,
                ),
                children: [
                  Text(
                    '${_width.toInt()} × ${_height.toInt()}',
                    style: TextStyle(
                      fontSize: EditorMetrics.dense,
                      color: EditorTheme.of(context).muted,
                    ),
                  ),
                  Padding(
                    padding: const EdgeInsets.only(left: EditorMetrics.s8),
                    child: EditorSwitch(
                      key: const ValueKey('stage:transparentGround'),
                      on: _transparentGround,
                      compact: true,
                      glyph: Glyph.grid_on,
                      label: _transparentGround
                          ? 'The frame has no ground; the export carries alpha'
                          : 'Drop the ground so the export carries alpha',
                      onChanged: c.supports('composition')
                          ? (_) => _toggleGround()
                          : null,
                    ),
                  ),
                  if (_userStage)
                    Padding(
                      padding: const EdgeInsets.only(left: EditorMetrics.s8),
                      child: ValueListenableBuilder<Map<String, dynamic>>(
                        valueListenable: c.deskWork,
                        builder: (context, _, _) => _button(
                          _extend ? '● Extend' : 'Extend',
                          _toggleExtend,
                        ),
                      ),
                    ),
                  const Spacer(),
                  ValueListenableBuilder<int>(
                    valueListenable: c.frame,
                    builder: (context, frame, _) => Text(
                      'Frame $frame',
                      style: TextStyle(
                        fontSize: EditorMetrics.dense,
                        color: EditorTheme.of(context).muted,
                      ),
                    ),
                  ),
                  if (!c.supports('stageGesture'))
                    Padding(
                      padding: EdgeInsets.only(left: EditorMetrics.s8),
                      child: Text(
                        'Transform gestures unavailable',
                        style: TextStyle(
                          fontSize: EditorMetrics.dense,
                          color: EditorTheme.of(context).muted,
                        ),
                      ),
                    ),
                ],
              ),
            ),
          ],
        );

  /// The picture and what is drawn over it: the part of the Stage that moves
  /// with the frame, the selection and the zoom. Built inside the frame's
  /// listenable; the chrome around it is built once per layout.
  Widget _picture() {
    _queueWindowSync();
    final origin = _origin;
    final scale = _scale;
    // The Stage is drawn at the tab's own size, the frame
    // inside it; the Camera is the output picture itself.
    // The Stage shows the latest frame where *that frame's* window sits under
    // the current zoom and pan: a Fit or a resize moves the old picture at
    // once, and the re-rendered one lands on the same spot (= the cover box).
    final renderedWindow = c.rendered.value['stageWindow'];
    if (_drawnWindow != null && _sameWindow(_drawnWindow, renderedWindow))
      _drawnWindow = null;
    _noteWindowLag(renderedWindow);
    final picture = _userStage
        ? _frameRect(_drawnWindow ?? renderedWindow) ?? _coverBox()
        : Rect.fromLTWH(origin.dx, origin.dy, _width * scale, _height * scale);
    final gizmos = !c.playing.value;
    // While dragging, the selection is what is being touched; otherwise only
    // the layer under the pointer gets a cage.
    final touching = _pointer != null
        ? c.selectedIds
        : {if (_touched != null) _touched!};
    final selectedTouched =
        gizmos && c.selectedIds.any((id) => touching.contains(id));
    final outlines = <List<Offset>>[];
    Offset? anchorPreview;
    for (final layer in _visible.where(
      (l) => gizmos && _grabbable(l) && c.selectedIds.contains(l['id']),
    )) {
      final points = _corners(layer);
      if (points.isNotEmpty && touching.contains(layer['id']))
        outlines.add(points.map(_toScreen).toList());
      // Where a hovered anchor would sit: bilinear in the corners. The
      // Inspector's anchor pad is itself the intent, so the mark shows on
      // the selection whether or not the pointer is on it.
      final f = c.anchorPreview.value;
      if (f != null && points.length >= 4 && anchorPreview == null) {
        final u = f[0], v = f[1];
        final top = points[0] + (points[1] - points[0]) * u;
        final bottom = points[3] + (points[2] - points[3]) * u;
        anchorPreview = _toScreen(top + (bottom - top) * v);
      }
    }
    return Stack(
      children: [
        // 地が無い枠は市松で見せる。枠そのものの見え方なので、
        // 描いた絵の有無に関わらず枠いっぱいに敷く(書き出しには乗らない)。
        if (_transparentGround)
          Positioned.fromRect(
            rect: Rect.fromLTWH(
              origin.dx,
              origin.dy,
              _width * scale,
              _height * scale,
            ),
            child: const RepaintBoundary(
              child: CustomPaint(painter: CheckerPainter()),
            ),
          ),
        Positioned.fromRect(
          rect: picture,
          child: Stack(
            fit: StackFit.expand,
            children: [
              _StageTexture(textureIds: c.textureIds, view: widget.view),
            ],
          ),
        ),
        Positioned.fill(
          child: IgnorePointer(
            child: RepaintBoundary(
              child: CustomPaint(
                painter: _StageOverlay(
                  colors: EditorTheme.of(context),
                  ink: EditorInk.of(context),
                  dimOutside: _userStage,
                  anchorPreview: anchorPreview,
                  cameras: gizmos
                      ? _cameras.map(_cameraCorners).toList()
                      : const [],
                  cameraEyes: gizmos
                      ? _cameras.map(_cameraEye).toList()
                      : const [],
                  cameraFrustums: gizmos
                      ? _cameras.map(_cameraFrustum).toList()
                      : const [],
                  cameraUps: gizmos
                      ? _cameras.map(_cameraUp).toList()
                      : const [],
                  cameraTargets: [
                    if (gizmos)
                      for (final camera in _cameras) ?_point(camera['target']),
                  ],
                  cameraHandles: gizmos && _front && _selectedCamera != null
                      ? _cameraHandles(_selectedCamera!)
                      : const {},
                  front: _front,
                  extent: _extentPoints(),
                  extendable: _front && _extend,
                  observerTarget: _front ? null : _point(_observer['target']),
                  outlines: _outlinesCopy(outlines),
                  handles: selectedTouched ? _handles() : const {},
                  spatialMesh: gizmos && _spatialActive ? _screenMesh() : null,
                  snapGuides: _snapGuides(),
                  marquee: _marquee == null
                      ? null
                      : Rect.fromPoints(
                          _toScreen(_marquee!.topLeft),
                          _toScreen(_marquee!.bottomRight),
                        ),
                  // Front-on, the frame is the comp rectangle under the
                  // current zoom and pan (the painter's `viewport`), known
                  // here and now. The observer's projected frame is only
                  // needed once the Stage is orbited, and it arrives with
                  // the next frame from native.
                  frame: _front
                      ? const []
                      : [
                          for (final p in (_observer['frame'] as List?) ?? [])
                            ?_point(p),
                        ],
                  viewport: Rect.fromLTWH(
                    origin.dx,
                    origin.dy,
                    _width * scale,
                    _height * scale,
                  ),
                ),
              ),
            ),
          ),
        ),
      ],
    );
  }

  List<List<Offset>> _outlinesCopy(List<List<Offset>> p) =>
      p.map((v) => List<Offset>.of(v)).toList();
}
