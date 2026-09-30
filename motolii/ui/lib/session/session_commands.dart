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

  /// The Animate switch pressed: the host flips what it holds (a second press before the reply still flips back).
  Future<void> toggleAnimate() => command('animate', {
    'toggle': true,
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

  Future<void> command(String op, [Map<String, dynamic> args = const {}]) {
    if (_disposed) return Future<void>.value();
    LatencyProbe.mark('cmd:$op');
    // An edit that comes while continuous previews are queued takes its place in that queue: after the previews before
    // it, before the ones that come later (a gesture's begin is never overtaken by its own updates).
    if (_directFlying || _direct.isNotEmpty) return _enqueueDirect(_DirectItem(op, args, null));
    return _run(op, args);
  }

  Future<void> _run(String op, Map<String, dynamic> args) => op == 'save'
      ? native('flushEditors').then((_) => _command(op, args))
      : _command(op, args);

  // ---- direct manipulation: the newest preview wins, order is kept --------------------------------------------------
  final _direct = <_DirectItem>[];
  bool _directFlying = false;

  /// A command sent while a hand is holding something (a preview, and the commit or cancel that ends it). Nothing waits for
  /// the previous preview's render: while one is in flight, a newer preview *replaces* the one waiting behind it (only the
  /// newest position of a gesture is worth drawing), and a commit, a cancel or any other command keeps its place after the
  /// previews before it. Same-[key] previews are the same gesture; without a key an item is never replaced.
  Future<void> commandDirect(String op, [Map<String, dynamic> args = const {}, String? key]) {
    if (_disposed) return Future<void>.value();
    LatencyProbe.mark('cmd:$op');
    if (key != null && _direct.isNotEmpty && _direct.last.key == key && _direct.last.op == op) {
      _direct.last.args = args;
      return _direct.last.done.future;
    }
    // a preview nobody awaits must not report its failure as unhandled; a caller that awaits still gets it
    return _enqueueDirect(_DirectItem(op, args, key))..ignore();
  }

  Future<void> _enqueueDirect(_DirectItem item) {
    _direct.add(item);
    if (!_directFlying) {
      _directFlying = true;
      _flyDirect();
    }
    return item.done.future;
  }

  Future<void> _flyDirect() async {
    try {
      while (_direct.isNotEmpty) {
        final item = _direct.removeAt(0);
        try {
          await _run(item.op, item.args);
          item.done.complete();
        } catch (e, st) {
          item.done.completeError(e, st);
        }
      }
    } finally {
      _directFlying = false;
    }
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
      LatencyProbe.mark('req:$op');
      final response = EditorSession.map(await _request(operation, args));
      LatencyProbe.mark('reply:$op');
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
      LatencyProbe.mark('accepted:$op');
      if (needsRender) await _render();
      LatencyProbe.mark('done:$op');
    });
  }

  /// Where the playhead was last asked to go, until the host has answered: repeated steps (a held arrow) add to this,
  /// not to the frame the last reply reported, so none is lost; the newest wish is the only seek that waits.
  int? _seekTarget;
  int get frameTarget => _seekTarget ?? frame.value;
  Future<void> seek(int value) {
    _seekTarget = value;
    final flight = commandDirect('seek', {'frame': value}, 'seek');
    flight.whenComplete(() {
      if (_seekTarget == value) _seekTarget = null;
    });
    return flight;
  }

  void cancelPreview() {
    command('cancelPreview');
  }
}

class _DirectItem {
  _DirectItem(this.op, this.args, this.key);
  final String op;
  Map<String, dynamic> args;
  final String? key;
  final done = Completer<void>();
}
