import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../metrics.dart';
import '../theme.dart';
import 'floating.dart';

/// The leaf that takes typing: text, a caret, a selection and the right
/// button's menu.

/// Text and a caret, nothing more — the box is [EditorFieldFrame]'s. The
/// pointer places the caret and drags a selection, the right button opens
/// Cut / Copy / Paste / Select all, [hint] shows in [EditorTheme.of(context).muted]
/// while empty, Enter submits a single line.
class EditorTextField extends StatefulWidget {
  const EditorTextField({
    super.key,
    this.controller,
    this.focusNode,
    this.style,
    this.hint,
    this.enabled = true,
    this.autofocus = false,
    this.readOnly = false,
    this.minLines,
    this.maxLines = 1,
    this.expands = false,
    this.padding = EdgeInsets.zero,
    this.prefix,
    this.textAlign = TextAlign.start,
    this.cursorWidth = 2,
    this.cursorColor,
    this.keyboardType,
    this.textInputAction,
    this.inputFormatters,
    this.onChanged,
    this.onSubmitted,
    this.onEditingComplete,
    this.onTap,
  });
  final TextEditingController? controller;
  final FocusNode? focusNode;
  final TextStyle? style;
  final String? hint;
  final bool enabled, autofocus, readOnly, expands;
  final int? minLines, maxLines;

  /// Room around the text inside the frame, and what stands before it
  /// (a glyph, say) — Material's contentPadding and prefixIcon.
  final EdgeInsets padding;
  final Widget? prefix;
  final TextAlign textAlign;
  final double cursorWidth;
  final Color? cursorColor;
  final TextInputType? keyboardType;
  final TextInputAction? textInputAction;
  final List<TextInputFormatter>? inputFormatters;
  final ValueChanged<String>? onChanged, onSubmitted;
  final VoidCallback? onEditingComplete, onTap;

  /// What Material's field wrote with under the editor's theme: body size,
  /// ink, medium weight.
  static TextStyle defaultStyle(BuildContext context) =>
      EditorTheme.of(context).text;
  @override
  State<EditorTextField> createState() => _EditorTextFieldState();
}

class _EditorTextFieldState extends State<EditorTextField>
    implements TextSelectionGestureDetectorBuilderDelegate {
  TextEditingController? _ownController;
  FocusNode? _ownFocus;
  TextEditingController get _controller =>
      widget.controller ?? (_ownController ??= TextEditingController());
  FocusNode get _focus => widget.focusNode ?? (_ownFocus ??= FocusNode());
  late final _gestures = _FieldGestures(this);

  @override
  final editableTextKey = GlobalKey<EditableTextState>();
  @override
  bool get forcePressEnabled => false;
  @override
  bool get selectionEnabled => true;

  @override
  void dispose() {
    _ownController?.dispose();
    _ownFocus?.dispose();
    super.dispose();
  }

  Widget _menu(BuildContext context, EditableTextState state) {
    final anchor = state.contextMenuAnchors.primaryAnchor;
    return EditorMenuSheet(
      anchor: Rect.fromLTWH(anchor.dx, anchor.dy, 0, 0),
      minWidth: 0,
      consumeOutsideTap: true,
      onClose: state.hideToolbar,
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          for (final item in state.contextMenuButtonItems)
            EditorMenuRow(
              onPressed: item.onPressed,
              child: Text(
                item.label ??
                    switch (item.type) {
                      ContextMenuButtonType.cut => 'Cut',
                      ContextMenuButtonType.copy => 'Copy',
                      ContextMenuButtonType.paste => 'Paste',
                      ContextMenuButtonType.selectAll => 'Select all',
                      _ => item.type.name,
                    },
              ),
            ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final style = EditorTextField.defaultStyle(context).merge(widget.style);
    final enabled = widget.enabled;
    final dpr = MediaQuery.devicePixelRatioOf(context);
    final field = EditableText(
      key: editableTextKey,
      controller: _controller,
      focusNode: _focus,
      style: style,
      cursorColor: widget.cursorColor ?? EditorTheme.of(context).caret,
      backgroundCursorColor: EditorTheme.of(context).muted,
      selectionColor: EditorTheme.of(context).selection,
      cursorWidth: widget.cursorWidth,
      cursorRadius: const Radius.circular(EditorMetrics.s2),
      cursorOffset: Offset(-2 / dpr, 0),
      paintCursorAboveText: true,
      cursorOpacityAnimates: false,
      textAlign: widget.textAlign,
      readOnly: widget.readOnly || !enabled,
      autofocus: widget.autofocus,
      minLines: widget.minLines,
      maxLines: widget.maxLines,
      expands: widget.expands,
      keyboardType:
          widget.keyboardType ??
          (widget.maxLines == 1 ? TextInputType.text : TextInputType.multiline),
      textInputAction: widget.textInputAction,
      inputFormatters: widget.inputFormatters,
      onChanged: widget.onChanged,
      onSubmitted: widget.onSubmitted,
      onEditingComplete: widget.onEditingComplete,
      onSelectionHandleTapped: () {},
      rendererIgnoresPointer: true,
      selectionControls: null,
      contextMenuBuilder: _menu,
      // Material's field on the desktop: the toolbar lives on the right
      // button, and a drag selects without showing handles.
      showSelectionHandles: false,
      mouseCursor: SystemMouseCursors.text,
    );
    Widget body = _gestures.buildGestureDetector(
      behavior: HitTestBehavior.translucent,
      child: field,
    );
    if (widget.hint != null) {
      body = Stack(
        fit: widget.expands ? StackFit.expand : StackFit.loose,
        children: [
          ValueListenableBuilder<TextEditingValue>(
            valueListenable: _controller,
            builder: (context, value, child) =>
                value.text.isEmpty ? child! : const SizedBox.shrink(),
            child: IgnorePointer(
              child: Text(
                widget.hint!,
                maxLines: widget.maxLines,
                textAlign: widget.textAlign,
                overflow: TextOverflow.ellipsis,
                style: style.copyWith(color: EditorTheme.of(context).muted),
              ),
            ),
          ),
          body,
        ],
      );
    }
    if (widget.padding != EdgeInsets.zero) {
      body = Padding(padding: widget.padding, child: body);
    }
    if (widget.prefix != null) {
      body = Row(
        children: [
          widget.prefix!,
          Expanded(child: body),
        ],
      );
    }
    return MouseRegion(
      cursor: enabled ? SystemMouseCursors.text : SystemMouseCursors.basic,
      child: IgnorePointer(
        ignoring: !enabled,
        child: Semantics(textField: true, enabled: enabled, child: body),
      ),
    );
  }
}

/// The field's pointer, as Material wired it: a tap places the caret (and
/// tells [EditorTextField.onTap]), a drag selects, a double tap takes the
/// word.
class _FieldGestures extends TextSelectionGestureDetectorBuilder {
  _FieldGestures(this.field) : super(delegate: field);
  final _EditorTextFieldState field;
  @override
  void onSingleTapUp(TapDragUpDetails details) {
    super.onSingleTapUp(details);
    field.widget.onTap?.call();
  }
}
