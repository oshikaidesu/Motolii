part of 'editor_session.dart';

/// 命令: what the window asks the document to do, and the focus a request
/// hands to a shelf.
mixin SessionCommands on SessionCore {
  Future<void> flushEditors() async {
    for (final flush in pendingEditors.toList()) {
      await flush();
    }
  }

  void focusEditing(int layer, String property) {
    editingFocus.value = {'layer': layer, 'property': property};
  }

  Future<void> setAnimate(bool on) => command('animate', {
    'enabled': on,
    'from': animateFrom,
    'shape': newKeyShape,
  });

  /// Point the Fonts shelf at a text layer: the Inspector's font value is the
  /// name, choosing is the shelf's job (the colour swatch and wheel, likewise).
  Future<void> focusFont(Map<String, dynamic> layer) async {
    final held = textStyleTarget.value;
    textStyleTarget.value = {
      if (held != null && held['layer'] == layer['id']) ...held,
      'layer': layer['id'],
      'scope': held != null && held['layer'] == layer['id']
          ? held['scope'] ?? 'all'
          : 'all',
    };
    browserTab.value = 'Fonts';
    await placePanel('Fonts', 'show');
  }

  Future<void> focusColor(Map<String, dynamic> args) async {
    await command('focusColor', args);
    browserTab.value = 'Colors';
    await placePanel('Colors', 'show');
  }

  Future<void> reselectKeys() async {
    final back = previousKeys;
    if (back == null) return;
    await command('select', back);
  }

  /// The same-frame path for a view change: the request, its render and the
  /// status all complete before this frame is built, so the picture the
  /// raster shows is the one asked for. The status reaches the notifiers
  /// after the frame (a build may not mark widgets outside its own subtree).
  /// Returns true when every view was drawn with the request applied; false
  /// means the request went the ordinary way and the picture follows later.
  bool commandNow(String op, [Map<String, dynamic> args = const {}]) {
    final frames = _frames;
    if (frames == null || _disposed || _pendingWork > 0 || _deferred != null) {
      command(op, args);
      return false;
    }
    final DocumentOperation operation;
    try {
      operation = DocumentOperation.parse(op);
      DocumentOperation.validateArguments(args);
    } catch (e) {
      error.value = '$e';
      return false;
    }
    _deferred = [];
    scheduleMicrotask(_flushDeferred);
    try {
      final response = _requestNow(frames, operation, args);
      final needsRender =
          response['needsRender'] as bool? ?? operation.requiresRender;
      final stateless =
          response['ok'] == true ||
          (response.length == 1 && response['needsRender'] == true);
      if (!stateless) _accept(response);
      if (!needsRender) return false;
      return _renderNow(frames);
    } catch (e) {
      _deferred?.add(() => throw e);
      return false;
    }
  }

  Future<void> command(String op, [Map<String, dynamic> args = const {}]) {
    if (_disposed) return Future<void>.value();
    if (op == 'save') {
      return native('flushEditors').then((_) => _command(op, args));
    }
    return _command(op, args);
  }

  Future<void> _command(String op, Map<String, dynamic> args) {
    if ((op == 'create' || op == 'placeAsset') &&
        !args.containsKey('visibleFrames') &&
        visibleFrames.value != null)
      args = {...args, 'visibleFrames': visibleFrames.value};
    final DocumentOperation operation;
    try {
      operation = DocumentOperation.parse(op);
      DocumentOperation.validateArguments(args);
    } catch (e) {
      error.value = '$e';
      return Future<void>.value();
    }
    if (operation == DocumentOperation.play) {
      _startPlayback();
      return _tail;
    }
    if (operation == DocumentOperation.pause) {
      stopPlayback();
      return _tail;
    }
    final requiresPause = operation.requiresPause;
    if (requiresPause) _schedulePause(renderFinal: false);
    return _serial(() async {
      if (requiresPause && playing.value)
        throw StateError('Playback did not stop before $op');
      final response = EditorSession.map(await _request(operation, args));
      final needsRender =
          response['needsRender'] as bool? ?? operation.requiresRender;
      // 状態を持たない返信は 2 つだけ — 繰り延べた {"needsRender":true} と
      // quiet な seek/tick の {"ok":true}。それ以外は必ず取り込む。
      final stateless =
          response['ok'] == true ||
          (response.length == 1 && response['needsRender'] == true);
      if (!stateless) _accept(response);
      if (operation == DocumentOperation.select) {
        editingFocus.value = {'selection': true};
      }
      if (needsRender) await _render();
    });
  }

  void seek(int value) {
    command('seek', {'frame': value});
  }

  void cancelPreview() {
    command('cancelPreview');
  }
}
