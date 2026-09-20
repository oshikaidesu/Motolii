import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../leaves.dart';
import '../metrics.dart';
import '../theme.dart';

/// The box typing happens in, and the field that holds a draft until it is
/// committed or refused.

/// The one box a field sits in, at rest and while typing. The theme draws no
/// frame around a TextField, so a field has this border and no other: line at
/// rest, accent while it owns the keys, error while it refuses a draft. A tap
/// anywhere in the box hands the keys to [focus].
class EditorFieldFrame extends StatelessWidget {
  const EditorFieldFrame({
    super.key,
    required this.child,
    this.focus,
    this.error = false,
    this.height = EditorMetrics.row,
    this.minHeight,
    this.maxHeight,
    this.padding = const EdgeInsets.symmetric(horizontal: EditorMetrics.s5),
    this.color,
  });
  final Widget child;
  final FocusNode? focus;
  final bool error;
  final double? height, minHeight, maxHeight;
  final EdgeInsets padding;
  final Color? color;

  Widget _box(BuildContext context, bool focused) => GestureDetector(
    behavior: HitTestBehavior.translucent,
    onTap: focus?.requestFocus,
    child: Container(
      height: height,
      constraints: minHeight == null && maxHeight == null
          ? null
          : BoxConstraints(
              minHeight: minHeight ?? 0,
              maxHeight: maxHeight ?? double.infinity,
            ),
      padding: padding,
      decoration: BoxDecoration(
        color: color ?? EditorTheme.of(context).app,
        border: Border.all(
          color: error
              ? EditorTheme.of(context).error
              : focused
              ? EditorTheme.of(context).accent
              : EditorTheme.of(context).line,
        ),
      ),
      child: child,
    ),
  );

  @override
  Widget build(BuildContext context) => focus == null
      ? _box(context, false)
      : ListenableBuilder(
          listenable: focus!,
          builder: (_, __) => _box(context, focus!.hasFocus),
        );
}

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
  Widget build(BuildContext context) => EditorTooltip(
    message: _error ?? widget.label,
    child: EditorFieldFrame(
      focus: _focus,
      error: _error != null,
      height: widget.multiline ? null : EditorMetrics.row,
      minHeight: widget.multiline ? EditorMetrics.row : null,
      child: EditorTextField(
        controller: _text,
        focusNode: _focus,
        enabled: widget.enabled,
        minLines: 1,
        maxLines: widget.multiline ? 4 : 1,
        style: TextStyle(
          fontSize: EditorMetrics.font,
          color: EditorTheme.of(context).ink,
        ),
        hint: widget.label,
        onSubmitted: (_) => _commit(),
        onChanged: (_) {
          if (_error != null) setState(() => _error = null);
        },
      ),
    ),
  );
}
