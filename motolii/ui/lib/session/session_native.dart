part of 'editor_session.dart';

/// native との往復: the channel and the same-frame runtime, the queue that
/// keeps calls in order, and taking a reply in.
mixin SessionNative on SessionCore {
  Future<dynamic> native(
    String method, [
    Map<String, dynamic> args = const {},
  ]) async {
    final connects = method == 'attach' || method == 'open';
    final reply = await _bridge.invoke(method, {
      ...args,
      if (connects && _attachmentId != null) 'attachmentId': _attachmentId,
    });
    if (connects) {
      final attachment = EditorSession.map(reply)['attachmentId'];
      if (attachment is num) {
        if (_disposed) {
          try {
            await _bridge.invoke('detach', {
              'attachmentId': attachment.toInt(),
            });
          } catch (_) {}
        } else {
          _attachmentId = attachment.toInt();
        }
      }
    }
    if (method == 'openPanelWindow') _panelWindows++;
    return reply;
  }

  void _bindFrames(Map<String, dynamic> envelope) {
    final library = envelope['library'], context = envelope['context'];
    if (library is! String || context is! int || windowInfo['main'] == false)
      return;
    try {
      final held = _frames;
      if (held == null) {
        _frames = FfiFrames(library, context);
      } else if (held.context != context) {
        held.context = context;
        _clearSurfaces();
      }
    } catch (e) {
      debugPrint('PROBE room=bridge verdict=channel-fallback reason=$e');
      _frames = null;
    }
  }

  void _flushDeferred() {
    final held = _deferred;
    _deferred = null;
    if (held == null) return;
    for (final apply in held) {
      try {
        apply();
      } catch (e) {
        if (!_disposed) error.value = '$e';
      }
    }
  }

  void _accept(dynamic reply, {bool notify = true}) {
    if (_disposed) return;
    if (_deferred case final held?) {
      held.add(() => _accept(reply, notify: notify));
      return;
    }
    final envelope = EditorSession.map(EditorSession.typed(reply));
    final epoch = envelope['runtimeEpoch'];
    if (epoch is num && epoch.toInt() != _runtimeEpoch.value) {
      _clearSurfaces();
      rendered.value = {};
      _runtimeEpoch.value = epoch.toInt();
      if (windowInfo['main'] != false) {
        _sentFlatProjection = null;
        _syncPreferences();
      }
    }
    _bindFrames(envelope);
    final inner = EditorSession.map(envelope['status']);
    final next = inner.isEmpty ? envelope : inner;
    if (next['error'] != null && '${next['error']}'.isNotEmpty) {
      throw StateError('${next['error']}');
    }
    if (envelope['textureId'] is num)
      textureId.value = (envelope['textureId'] as num).toInt();
    if (envelope['textureIds'] is Map) {
      final ids = {
        for (final e in (envelope['textureIds'] as Map).entries)
          if (e.value is num) '${e.key}': (e.value as num).toInt(),
      };
      if (!sameValue(ids, textureIds.value)) textureIds.value = ids;
    }
    if (envelope['frameReady'] == true) {
      if (next['frame'] is num) frame.value = (next['frame'] as num).toInt();
      rendered.value = next;
    } else if (next['needsRender'] == false &&
        next['contentRevision'] != null &&
        next['contentRevision'] == rendered.value['contentRevision'] &&
        next['frame'] == rendered.value['frame']) {
      EditorSession.take(rendered, next);
    }
    if (next['playing'] is bool) {
      playing.value = next['playing'] as bool;
      if (!playing.value && _cadenceRunning) _cancelCadence();
      if (playing.value &&
          !_cadenceRunning &&
          _frames == null &&
          textureId.value != null &&
          windowInfo['main'] != false) {
        _playRequested = true;
        _beginCadence(++_generation);
      }
    }
    if (notify || next['layers'] is List) absorb(next);
  }

  Future<void> _serial(
    Future<void> Function() action, {
    bool displayBusy = true,
  }) {
    if (_disposed) return Future<void>.value();
    _pendingWork++;
    final work = _tail.then((_) async {
      try {
        if (_disposed) return;
        if (displayBusy) {
          busy.value = true;
          error.value = null;
        }
        await action();
      } catch (e) {
        if (!_disposed) error.value = '$e';
      } finally {
        _pendingWork--;
        if (!_disposed && displayBusy) busy.value = false;
      }
    });
    _tail = work;
    return work;
  }

  Map<String, dynamic> _snapshotContext(DocumentOperation operation) => {
    'knownSnapshotId': state['snapshotId'],
    'knownReferenceId': state['referenceId'],
    'deferSnapshot':
        operation != DocumentOperation.play &&
        operation != DocumentOperation.pause,
  };

  Future<dynamic> _request(
    DocumentOperation operation, [
    Map<String, dynamic> args = const {},
  ]) {
    if (_frames case final frames?
        when !playing.value &&
            operation != DocumentOperation.play &&
            operation != DocumentOperation.pause) {
      return Future.value(_requestNow(frames, operation, args));
    }
    return _bridge.request(operation, args, _snapshotContext(operation));
  }

  /// A specimen or a measurement answers this window alone; anything else
  /// may have moved the document and is worth telling the other windows.
  static const _askedAlone = {'visualSample', 'renderInfo', 'easeModel'};

  Map<String, dynamic> _requestNow(
    FfiFrames frames,
    DocumentOperation operation,
    Map<String, dynamic> args,
  ) {
    final raw = frames.request(
      operation.encode({...args, ..._snapshotContext(operation)}),
    );
    final reply = EditorSession.map(EditorSession.typed(jsonDecode(raw)));
    if (!_askedAlone.contains(operation.wireName) && reply['error'] == null) {
      _broadcast(raw);
    }
    return reply;
  }

  void _broadcast(
    String status, {
    bool frameReady = false,
    bool frameOnly = false,
  }) {
    if (_panelWindows <= 0 || _disposed) return;
    _bridge
        .invoke('broadcast', {
          'status': status,
          'frameReady': frameReady,
          'frameOnly': frameOnly,
        })
        .catchError((_) => null);
  }

  bool supports(String op) =>
      (state['capabilities'] as List? ?? const []).contains(op);

  /// What the host says, unprompted: a document that moved elsewhere, a panel
  /// window that closed, files carried over this one.
  void _listenToHost() {
    _bridge.listen((call) async {
      if (_disposed) return call.method == 'confirmClose' ? true : null;
      if (call.method == 'confirmClose') {
        await flushEditors();
        return await confirmClose?.call() ?? true;
      }
      if (call.method == 'flushEditors') {
        await flushEditors();
        return true;
      }
      if (call.method == 'placePanel') {
        final args = EditorSession.map(call.arguments);
        await panelPlacementRequested?.call(
          '${args['name']}',
          '${args['placement']}',
        );
        return true;
      }
      if (call.method == 'paneState') {
        final state = EditorSession.map(call.arguments);
        panePlaces.value = EditorSession.map(state['places']);
        deskDrawer.value = state['drawer'] as String?;
        if (state.containsKey('theme') &&
            !sameValue(deskWork.value['theme'], state['theme'])) {
          deskWork.value = {
            ...deskWork.value,
            'theme': EditorSession.typed(state['theme']),
          };
        }
      }
      if (call.method == 'documentChanged') {
        final envelope = EditorSession.map(call.arguments);
        try {
          _accept(envelope, notify: envelope['frameOnly'] != true);
        } catch (e) {
          if (!_disposed) error.value = '$e';
        }
        // vism/ の file が変わった。絵は main の窓が 1 回だけ描き直す(他の窓は絵を受け取る側)。
        if (envelope['effectsReloaded'] == true &&
            EditorSession.map(envelope['status'])['needsRender'] == true &&
            windowInfo['main'] != false) {
          refreshPreview();
        }
      }
      if (call.method == 'playbackFrame') {
        final current = EditorSession.map(call.arguments)['frame'];
        if (current is num) frame.value = current.toInt();
      }
      // vism/ の file が変わった(host の見張り)。棚を読み直すのは main の窓の仕事。
      if (call.method == 'effectsChanged' && windowInfo['main'] != false) {
        command('reloadEffects');
      }
      if (call.method == 'documentClosed') {
        _cancelCadence();
        playing.value = false;
        textureId.value = null;
        rendered.value = {};
        document.value = {};
        _clearSurfaces();
        _frames?.dispose();
        _frames = null;
      }
      if (call.method == 'windowClosed') {
        if (_panelWindows > 0) _panelWindows--;
        windowClosed?.call(EditorSession.map(call.arguments));
      }
      if (call.method == 'dragHover')
        dragging.value = EditorSession.map(call.arguments)['active'] == true;
      if (call.method == 'filesDropped' &&
          fileDropTarget?.call(EditorSession.map(call.arguments)) != true)
        filesDropped?.call(
          (EditorSession.map(call.arguments)['paths'] as List? ?? [])
              .whereType<String>()
              .toList(),
        );
      return null;
    });
  }
}
