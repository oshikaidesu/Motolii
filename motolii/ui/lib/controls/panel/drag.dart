import 'dart:ui' show ViewFocusEvent, ViewFocusState;

import 'package:flutter/widgets.dart';

/// The one route every continuous control takes: drag, preview, commit, with
/// the previews queued so only the latest is in flight.

// Uses the numeric field's latest-pending-value rule for continuous controls.
/// drag → preview → commit, once, for every continuous control. The field,
/// the dial, the pad and the gradient stops mix this in and keep only their
/// own value shape: what a preview sends, what a commit and a cancel run, and
/// what to clear once the drag has settled. The window losing focus, or the
/// app leaving the foreground, cancels the drag in flight.
mixin EditorDragSession<T, W extends StatefulWidget>
    on State<W>, WidgetsBindingObserver {
  late final queue = EditorPreviewQueue<T>(sendPreview);

  /// True from [beginDrag] until [endDrag] takes the drag over.
  bool dragging = false;

  /// True while the commit or cancel of the last drag is in flight; a new
  /// drag waits for it.
  bool ending = false;

  Future<void> sendPreview(T value);
  Future<void> commitDrag();
  Future<void> cancelDrag();

  /// Whether the window's focus and the app's lifecycle end the drag.
  bool get watchesWindow => true;

  /// A commit that throws away what is still queued and runs [cancelDrag]
  /// instead (a stop pulled off its bar).
  bool get discardsPreview => false;

  /// Right after the drag stops, before the commit or cancel goes out.
  void dragStopped(bool cancel) {}

  /// Once the commit or cancel has settled and the control is still mounted.
  void dragSettled() {}

  void beginDrag() => dragging = true;

  Future<void> endDrag(bool cancel) async {
    if (!dragging) return;
    dragging = false;
    dragStopped(cancel);
    ending = true;
    final discard = cancel || discardsPreview;
    try {
      await queue.finish(discard, discard ? cancelDrag : commitDrag);
    } finally {
      ending = false;
      if (mounted) dragSettled();
    }
  }

  @override
  void initState() {
    super.initState();
    if (watchesWindow) WidgetsBinding.instance.addObserver(this);
  }

  @override
  void dispose() {
    if (dragging) queue.finish(true, cancelDrag);
    if (watchesWindow) WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state != AppLifecycleState.resumed) endDrag(true);
  }

  @override
  void didChangeViewFocus(ViewFocusEvent event) {
    if (event.state == ViewFocusState.unfocused) endDrag(true);
  }
}

class EditorPreviewQueue<T> {
  EditorPreviewQueue(this.send);
  final Future<void> Function(T) send;
  T? _pending;
  bool _sending = false;
  Future<void> _drained = Future<void>.value();

  /// Settles when nothing is left to send.
  Future<void> get drained => _drained;
  void add(T value) {
    _pending = value;
    if (_sending) return;
    _sending = true;
    _drained = () async {
      try {
        while (_pending != null) {
          final next = _pending as T;
          _pending = null;
          await send(next);
        }
      } finally {
        _sending = false;
      }
    }();
  }

  Future<void> finish(bool cancel, Future<void> Function() action) async {
    if (cancel) _pending = null;
    await _drained;
    await action();
  }
}
