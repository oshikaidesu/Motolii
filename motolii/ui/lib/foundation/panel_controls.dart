import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'theme.dart';
import 'metrics.dart';

Widget panelButton(
  String label,
  VoidCallback? action, {
  String? tooltip,
  bool selected = false,
}) =>
    EditorButton(label, action, tooltip: tooltip ?? label, selected: selected);
Widget panelTitle(String title) => Container(
  height: EditorMetrics.s22,
  alignment: Alignment.centerLeft,
  padding: const EdgeInsets.symmetric(horizontal: EditorMetrics.s8),
  decoration: const BoxDecoration(
    color: EditorTheme.raised,
    border: Border(bottom: BorderSide(color: EditorTheme.line)),
  ),
  child: Text(
    title,
    style: const TextStyle(
      fontSize: EditorMetrics.font,
      color: EditorTheme.ink,
    ),
  ),
);

class EditorDraftField extends StatefulWidget {
  const EditorDraftField({
    super.key,
    required this.value,
    required this.label,
    required this.onCommit,
    this.multiline = false,
    this.enabled = true,
    this.validator,
  });
  final String value, label;
  final bool multiline, enabled;
  final String? Function(String)? validator;
  final Future<void> Function(String) onCommit;
  @override
  State<EditorDraftField> createState() => _EditorDraftFieldState();
}

class _EditorDraftFieldState extends State<EditorDraftField> {
  late final TextEditingController _text = TextEditingController(
    text: widget.value,
  );
  late final FocusNode _focus = FocusNode(
    onKeyEvent: (_, event) {
      if (event is KeyDownEvent &&
          event.logicalKey == LogicalKeyboardKey.escape &&
          !_text.value.composing.isValid) {
        _cancelled = true;
        _text.text = widget.value;
        setState(() => _error = null);
        _focus.unfocus();
        return KeyEventResult.handled;
      }
      return KeyEventResult.ignored;
    },
  )..addListener(_focusChanged);
  String? _error;
  bool _cancelled = false, _committing = false;
  void _focusChanged() {
    if (!_focus.hasFocus && !_cancelled) _commit();
    _cancelled = false;
  }

  void _commit() {
    final error = widget.validator?.call(_text.text);
    setState(() => _error = error);
    if (error != null) {
      _focus.requestFocus();
      return;
    }
    if (_text.text != widget.value && !_committing) {
      _committing = true;
      widget.onCommit(_text.text).whenComplete(() => _committing = false);
    }
  }

  @override
  void didUpdateWidget(covariant EditorDraftField old) {
    super.didUpdateWidget(old);
    if (!_focus.hasFocus && old.value != widget.value)
      _text.text = widget.value;
  }

  @override
  void dispose() {
    _focus.removeListener(_focusChanged);
    _focus.dispose();
    _text.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Focus(
    onKeyEvent: (_, event) {
      if (event is KeyDownEvent &&
          event.logicalKey == LogicalKeyboardKey.escape) {
        _cancelled = true;
        _text.text = widget.value;
        setState(() => _error = null);
        _focus.unfocus();
        return KeyEventResult.handled;
      }
      return KeyEventResult.ignored;
    },
    child: TextField(
      controller: _text,
      focusNode: _focus,
      enabled: widget.enabled,
      minLines: 1,
      maxLines: widget.multiline ? 4 : 1,
      style: const TextStyle(
        fontSize: EditorMetrics.font,
        color: EditorTheme.ink,
      ),
      decoration: InputDecoration(
        isDense: true,
        contentPadding: const EdgeInsets.symmetric(
          horizontal: EditorMetrics.s5,
          vertical: EditorMetrics.s3,
        ),
        border: InputBorder.none,
        labelText: null,
        hintText: widget.label,
        errorText: _error,
      ),
      onSubmitted: (_) => _commit(),
      onChanged: (_) {
        if (_error != null) setState(() => _error = null);
      },
    ),
  );
}

class EditorNumericField extends StatefulWidget {
  const EditorNumericField({
    super.key,
    required this.value,
    required this.label,
    required this.onPreview,
    required this.onCommit,
    required this.onFinish,
    required this.onCancel,
    this.speed = 1,
    this.min,
    this.max,
    this.enabled = true,
    this.mixed = false,
    this.idleFocus,
  });
  final double value, speed;
  final double? min, max;
  final String label;
  final bool enabled, mixed;
  final FocusNode? idleFocus;
  final Future<void> Function(double) onPreview, onCommit;
  final Future<void> Function() onFinish, onCancel;
  @override
  State<EditorNumericField> createState() => _EditorNumericFieldState();
}

class _EditorNumericFieldState extends State<EditorNumericField> {
  final _text = TextEditingController();
  final _focus = FocusNode();
  final _ownIdle = FocusNode();
  FocusNode get _idle => widget.idleFocus ?? _ownIdle;
  bool _editing = false, _dragging = false, _sending = false;
  double _start = 0, _delta = 0;
  double? _pending, _shown;
  Future<void> _drained = Future<void>.value();
  String? _error;
  @override
  void initState() {
    super.initState();
    _focus.addListener(_lost);
    _focus.onKeyEvent = (_, event) {
      if (event is KeyDownEvent &&
          event.logicalKey == LogicalKeyboardKey.escape) {
        setState(() {
          _editing = false;
          _error = null;
        });
        _idle.requestFocus();
        return KeyEventResult.handled;
      }
      return KeyEventResult.ignored;
    };
  }

  void _lost() {
    if (!_focus.hasFocus && _editing) _commitText();
  }

  double _bounded(double n) => n
      .clamp(
        widget.min ?? double.negativeInfinity,
        widget.max ?? double.infinity,
      )
      .toDouble();
  void _open() {
    if (!widget.enabled) return;
    setState(() {
      _editing = true;
      _text.text = widget.mixed ? '' : widget.value.toString();
      _text.selection = TextSelection(
        baseOffset: 0,
        extentOffset: _text.text.length,
      );
    });
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) _focus.requestFocus();
    });
  }

  void _commitText() {
    final n = double.tryParse(_text.text.trim());
    if (n == null || !n.isFinite) {
      setState(() => _error = 'Number required');
      _focus.requestFocus();
      return;
    }
    setState(() {
      _editing = false;
      _error = null;
    });
    widget.onCommit(_bounded(n));
  }

  void _tick(double n) {
    _pending = n;
    if (_sending) return;
    _sending = true;
    _drained = () async {
      try {
        while (_pending != null) {
          final value = _pending!;
          _pending = null;
          await widget.onPreview(value);
        }
      } finally {
        _sending = false;
      }
    }();
  }

  Future<void> _end(bool cancel) async {
    if (!_dragging) return;
    setState(() => _dragging = false);
    if (cancel) _pending = null;
    await _drained;
    if (cancel)
      await widget.onCancel();
    else
      await widget.onFinish();
    if (mounted) setState(() => _shown = null);
  }

  @override
  void dispose() {
    if (_dragging) {
      _pending = null;
      final cancel = widget.onCancel;
      _drained.whenComplete(cancel);
    }
    _focus.removeListener(_lost);
    _focus.dispose();
    _ownIdle.dispose();
    _text.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Focus(
    focusNode: _idle,
    onKeyEvent: (_, event) {
      if (event is! KeyDownEvent) return KeyEventResult.ignored;
      if (event.logicalKey == LogicalKeyboardKey.escape) {
        if (_dragging) {
          _end(true);
          return KeyEventResult.handled;
        }
        if (_editing) {
          setState(() {
            _editing = false;
            _error = null;
          });
          return KeyEventResult.handled;
        }
      }
      if (!_editing && !_dragging && widget.enabled) {
        if (event.logicalKey == LogicalKeyboardKey.enter) {
          _open();
          return KeyEventResult.handled;
        }
        if (event.logicalKey == LogicalKeyboardKey.arrowUp ||
            event.logicalKey == LogicalKeyboardKey.arrowDown) {
          widget.onCommit(
            _bounded(
              widget.value +
                  (event.logicalKey == LogicalKeyboardKey.arrowUp ? 1 : -1) *
                      widget.speed *
                      (HardwareKeyboard.instance.isShiftPressed ? 10 : 1),
            ),
          );
          return KeyEventResult.handled;
        }
      }
      return KeyEventResult.ignored;
    },
    child: _editing
        ? TextField(
            controller: _text,
            focusNode: _focus,
            autofocus: true,
            style: const TextStyle(
              fontSize: EditorMetrics.font,
              color: EditorTheme.ink,
            ),
            decoration: InputDecoration(
              isDense: true,
              contentPadding: const EdgeInsets.symmetric(
                horizontal: EditorMetrics.s2,
                vertical: EditorMetrics.s2,
              ),
              border: InputBorder.none,
              errorText: _error,
            ),
            onSubmitted: (_) => _commitText(),
          )
        : MouseRegion(
            cursor: widget.enabled
                ? SystemMouseCursors.resizeLeftRight
                : SystemMouseCursors.basic,
            child: GestureDetector(
              onDoubleTap: _open,
              onHorizontalDragStart: widget.enabled
                  ? (_) {
                      _idle.requestFocus();
                      _start = widget.value;
                      _delta = 0;
                      setState(() => _dragging = true);
                    }
                  : null,
              onHorizontalDragUpdate: widget.enabled
                  ? (d) {
                      _delta += d.delta.dx;
                      final n = _bounded(_start + _delta * widget.speed);
                      setState(() => _shown = n);
                      _tick(n);
                    }
                  : null,
              onHorizontalDragEnd: (_) => _end(false),
              onHorizontalDragCancel: () => _end(true),
              child: Tooltip(
                message: widget.label,
                child: Container(
                  height: EditorMetrics.s18,
                  alignment: Alignment.centerRight,
                  padding: const EdgeInsets.symmetric(
                    horizontal: EditorMetrics.s2,
                  ),
                  color: _dragging ? EditorTheme.hover : EditorTheme.app,
                  child: Text(
                    widget.mixed && _shown == null
                        ? '—'
                        : (_shown ?? widget.value).toStringAsFixed(2),
                    maxLines: 1,
                    overflow: TextOverflow.clip,
                    style: TextStyle(
                      fontSize: EditorMetrics.font,
                      color: widget.enabled
                          ? EditorTheme.ink
                          : EditorTheme.muted,
                    ),
                  ),
                ),
              ),
            ),
          ),
  );
}
