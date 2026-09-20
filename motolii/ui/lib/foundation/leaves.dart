import 'dart:math' as math;

import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import 'glyphs.dart';
import 'metrics.dart';
import 'theme.dart';

/// The leaves the window presses, reads and scrolls with, on
/// `flutter/widgets` alone. Each reproduces what the Material leaf it
/// replaced drew under the editor's theme — the same lifts, sizes and inks —
/// so a panel swapping one for the other looks the same. What Material also
/// did (ripples, tap-target padding, its own metrics) never showed here.

/// A hover-and-press surface: the lift under a hovered or pressed child, a
/// click cursor, keyboard activation (Enter/Space through [ActivateIntent])
/// and the tap semantics. The wash paints beneath [child], so an opaque
/// child covers it the way it covered Material's ink.
class EditorPress extends StatefulWidget {
  const EditorPress({
    super.key,
    this.onTap,
    this.onTapDown,
    this.onDoubleTap,
    this.onLongPress,
    this.onSecondaryTap,
    this.onSecondaryTapDown,
    this.onHover,
    this.onFocusChange,
    this.borderRadius,
    this.canRequestFocus = true,
    this.focusNode,
    this.autofocus = false,
    this.hoverColor,
    this.pressColor,
    this.focusColor,
    this.mouseCursor,
    required this.child,
  });
  final VoidCallback? onTap, onDoubleTap, onLongPress, onSecondaryTap;
  final GestureTapDownCallback? onTapDown, onSecondaryTapDown;
  final ValueChanged<bool>? onHover, onFocusChange;
  final BorderRadius? borderRadius;
  final bool canRequestFocus, autofocus;
  final FocusNode? focusNode;
  final Color? hoverColor, pressColor, focusColor;
  final MouseCursor? mouseCursor;
  final Widget child;

  bool get enabled =>
      onTap != null ||
      onTapDown != null ||
      onDoubleTap != null ||
      onLongPress != null ||
      onSecondaryTap != null ||
      onSecondaryTapDown != null;

  @override
  State<EditorPress> createState() => _EditorPressState();
}

class _EditorPressState extends State<EditorPress> {
  bool _hover = false, _down = false, _focus = false;

  void _set(void Function() change) {
    if (mounted) setState(change);
  }

  void _activate(Intent _) => widget.onTap?.call();

  @override
  Widget build(BuildContext context) {
    final enabled = widget.enabled;
    final showFocus =
        _focus &&
        FocusManager.instance.highlightMode == FocusHighlightMode.traditional;
    final wash = _down
        ? (widget.pressColor ?? EditorTheme.of(context).hover)
        : _hover && enabled
        ? (widget.hoverColor ?? EditorTheme.of(context).hoverWash)
        : showFocus
        ? (widget.focusColor ?? EditorTheme.of(context).focusWash)
        : null;
    return Semantics(
      onTap: widget.onTap,
      onLongPress: widget.onLongPress,
      child: Focus(
        focusNode: widget.focusNode,
        autofocus: widget.autofocus,
        canRequestFocus: enabled && widget.canRequestFocus,
        onFocusChange: (f) {
          _set(() => _focus = f);
          widget.onFocusChange?.call(f);
        },
        child: Actions(
          actions: {
            ActivateIntent: CallbackAction<ActivateIntent>(onInvoke: _activate),
            ButtonActivateIntent: CallbackAction<ButtonActivateIntent>(
              onInvoke: _activate,
            ),
          },
          child: MouseRegion(
            cursor:
                widget.mouseCursor ??
                (enabled ? SystemMouseCursors.click : SystemMouseCursors.basic),
            onEnter: (_) {
              if (enabled) _set(() => _hover = true);
              widget.onHover?.call(true);
            },
            onExit: (_) {
              _set(() => _hover = false);
              widget.onHover?.call(false);
            },
            child: GestureDetector(
              behavior: HitTestBehavior.opaque,
              excludeFromSemantics: true,
              onTapDown: enabled
                  ? (d) {
                      _set(() => _down = true);
                      widget.onTapDown?.call(d);
                    }
                  : null,
              onTapUp: enabled ? (_) => _set(() => _down = false) : null,
              onTapCancel: enabled ? () => _set(() => _down = false) : null,
              onTap: widget.onTap,
              onDoubleTap: widget.onDoubleTap,
              onLongPress: widget.onLongPress,
              onSecondaryTap: widget.onSecondaryTap,
              onSecondaryTapDown: widget.onSecondaryTapDown,
              child: CustomPaint(
                painter: wash == null
                    ? null
                    : _WashPainter(wash, widget.borderRadius),
                child: widget.child,
              ),
            ),
          ),
        ),
      ),
    );
  }
}

class _WashPainter extends CustomPainter {
  const _WashPainter(this.color, this.radius);
  final Color color;
  final BorderRadius? radius;
  @override
  void paint(Canvas canvas, Size size) {
    final rect = Offset.zero & size;
    final paint = Paint()..color = color;
    if (radius == null) {
      canvas.drawRect(rect, paint);
    } else {
      canvas.drawRRect(radius!.toRRect(rect), paint);
    }
  }

  @override
  bool shouldRepaint(_WashPainter old) =>
      old.color != color || old.radius != radius;
}

/// The icon is the button: exactly [iconSize] square (the icon theme's size
/// when unset), the panel's [EditorTheme.of(context).hover] under it while hovered or
/// pressed, ink or [EditorTheme.of(context).disabledInk] on the glyph.
class EditorIconButton extends StatelessWidget {
  const EditorIconButton({
    super.key,
    required this.icon,
    required this.onPressed,
    this.iconSize,
    this.color,
    this.tooltip,
    this.isSelected,
    this.focusNode,
  });
  final Widget icon;
  final VoidCallback? onPressed;
  final double? iconSize;
  final Color? color;
  final String? tooltip;
  final bool? isSelected;
  final FocusNode? focusNode;
  @override
  Widget build(BuildContext context) {
    final enabled = onPressed != null;
    final size =
        iconSize ?? IconTheme.of(context).size ?? EditorMetrics.control;
    final button = Semantics(
      button: true,
      enabled: enabled,
      selected: isSelected,
      child: EditorPress(
        onTap: onPressed,
        focusNode: focusNode,
        hoverColor: EditorTheme.of(context).hover,
        pressColor: EditorTheme.of(context).hover,
        focusColor: EditorTheme.clear,
        child: SizedBox.square(
          dimension: math.max(size, EditorMetrics.row),
          child: IconTheme.merge(
            data: IconThemeData(
              size: size,
              color: enabled
                  ? color ?? EditorTheme.of(context).ink
                  : EditorTheme.of(context).disabledInk,
            ),
            child: Center(child: icon),
          ),
        ),
      ),
    );
    return tooltip == null
        ? button
        : EditorTooltip(message: tooltip!, child: button);
  }
}

/// A text button: [minimumSize] at least, [padding] around a centred child in
/// [foreground] (the accent unless told otherwise), [background] behind and a
/// [border] around it, and the foreground washed over it at the M3 state
/// opacities while hovered (8%) or pressed / keyboard-focused (10%).
class EditorTextButton extends StatefulWidget {
  const EditorTextButton({
    super.key,
    required this.onPressed,
    required this.child,
    this.foreground,
    this.background = EditorTheme.clear,
    this.disabledForeground,
    this.disabledBackground = EditorTheme.clear,
    this.border,
    this.radius = BorderRadius.zero,
    this.padding = const EdgeInsets.symmetric(horizontal: EditorMetrics.s4),
    this.minimumSize = const Size(EditorMetrics.s11, EditorMetrics.s12),
    this.textStyle,
  });
  final VoidCallback? onPressed;
  final Widget child;
  final Color? foreground, disabledForeground;
  final Color background, disabledBackground;
  final BorderSide? border;
  final BorderRadius radius;
  final EdgeInsets padding;
  final Size minimumSize;
  final TextStyle? textStyle;
  @override
  State<EditorTextButton> createState() => _EditorTextButtonState();
}

class _EditorTextButtonState extends State<EditorTextButton> {
  bool _hover = false, _down = false, _focus = false;
  void _set(void Function() change) {
    if (mounted) setState(change);
  }

  void _activate(Intent _) => widget.onPressed?.call();

  @override
  Widget build(BuildContext context) {
    final enabled = widget.onPressed != null;
    final fg = enabled
        ? (widget.foreground ?? EditorTheme.of(context).accent)
        : (widget.disabledForeground ?? EditorTheme.of(context).inkDisabled);
    final border =
        widget.border ?? BorderSide(color: EditorTheme.of(context).line);
    final bg = enabled ? widget.background : widget.disabledBackground;
    final showFocus =
        _focus &&
        FocusManager.instance.highlightMode == FocusHighlightMode.traditional;
    final wash = !enabled
        ? null
        : _down || showFocus
        ? fg.withValues(alpha: EditorTheme.pressedLift)
        : _hover
        ? fg.withValues(alpha: EditorTheme.hoverLift)
        : null;
    return Semantics(
      button: true,
      enabled: enabled,
      child: Focus(
        canRequestFocus: enabled,
        onFocusChange: (f) => _set(() => _focus = f),
        child: Actions(
          actions: {
            ActivateIntent: CallbackAction<ActivateIntent>(onInvoke: _activate),
            ButtonActivateIntent: CallbackAction<ButtonActivateIntent>(
              onInvoke: _activate,
            ),
          },
          child: MouseRegion(
            cursor: enabled
                ? SystemMouseCursors.click
                : SystemMouseCursors.basic,
            onEnter: (_) => _set(() => _hover = true),
            onExit: (_) => _set(() => _hover = false),
            child: GestureDetector(
              behavior: HitTestBehavior.opaque,
              excludeFromSemantics: true,
              onTapDown: enabled ? (_) => _set(() => _down = true) : null,
              onTapUp: enabled ? (_) => _set(() => _down = false) : null,
              onTapCancel: enabled ? () => _set(() => _down = false) : null,
              onTap: widget.onPressed,
              child: DecoratedBox(
                decoration: BoxDecoration(
                  color: bg,
                  border: border.style == BorderStyle.none
                      ? null
                      : Border.fromBorderSide(border),
                  borderRadius: widget.radius,
                ),
                child: CustomPaint(
                  painter: wash == null
                      ? null
                      : _WashPainter(wash, widget.radius),
                  child: ConstrainedBox(
                    constraints: BoxConstraints(
                      minWidth: widget.minimumSize.width,
                      minHeight: widget.minimumSize.height,
                    ),
                    child: Align(
                      widthFactor: 1,
                      heightFactor: 1,
                      child: Padding(
                        padding: widget.padding,
                        child: DefaultTextStyle.merge(
                          style: TextStyle(
                            fontSize: EditorMetrics.font,
                            color: fg,
                          ).merge(widget.textStyle),
                          child: IconTheme.merge(
                            data: IconThemeData(color: fg),
                            child: widget.child,
                          ),
                        ),
                      ),
                    ),
                  ),
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}

/// A horizontal rule in [EditorTheme.of(context).line], [thickness] thick (0 = a
/// hairline) centred in [height] — what Material's Divider drew here.
class EditorRule extends StatelessWidget {
  const EditorRule({
    super.key,
    this.height = EditorMetrics.s16,
    this.thickness = 0,
    this.color,
  });
  final double height, thickness;
  final Color? color;
  @override
  Widget build(BuildContext context) => SizedBox(
    height: height,
    child: Center(
      child: Container(
        height: thickness,
        decoration: BoxDecoration(
          border: Border(
            bottom: BorderSide(
              color: color ?? EditorTheme.of(context).line,
              width: thickness,
            ),
          ),
        ),
      ),
    ),
  );
}

/// The desktop scrollbar: a flat [EditorMetrics.s6] bar, no radius, white at
/// 30% (65% under the pointer, 75% while dragged), shown while scrolling,
/// hovered or dragged and fading 600 ms after; a thumb no shorter than 48.
class EditorScrollbar extends StatefulWidget {
  const EditorScrollbar({
    super.key,
    required this.child,
    this.controller,
    this.thumbVisibility,
  });
  final Widget child;
  final ScrollController? controller;
  final bool? thumbVisibility;
  @override
  State<EditorScrollbar> createState() => _EditorScrollbarState();
}

class _EditorScrollbarState extends State<EditorScrollbar> {
  bool _hover = false, _drag = false;
  @override
  Widget build(BuildContext context) => Listener(
    onPointerDown: (_) => _drag = true,
    onPointerUp: (_) => _drag = false,
    onPointerCancel: (_) => _drag = false,
    child: RawScrollbar(
      controller: widget.controller,
      thumbVisibility: widget.thumbVisibility,
      thickness: EditorMetrics.s6,
      radius: Radius.zero,
      crossAxisMargin: EditorMetrics.s2,
      minThumbLength: EditorMetrics.s48,
      interactive: true,
      thumbColor: _drag && _hover
          ? EditorTheme.of(context).scrollThumbDragged
          : _hover
          ? EditorTheme.of(context).scrollThumbHovered
          : EditorTheme.of(context).scrollThumb,
      child: MouseRegion(
        opaque: false,
        onEnter: (_) => setState(() => _hover = true),
        onExit: (_) => setState(() => _hover = false),
        child: widget.child,
      ),
    ),
  );
}

/// A value on a flat 2 px track between [min] and [max]: [EditorTheme.of(context).muted]
/// up to the thumb, [EditorTheme.of(context).line] after, a 5 px ink thumb with a 1 dp
/// shadow (6 while pressed); with [divisions] the value snaps and a label of
/// it stands above the thumb while it is dragged.
class EditorSlider extends StatefulWidget {
  const EditorSlider({
    super.key,
    required this.value,
    required this.onChanged,
    this.onChangeEnd,
    this.min = 0,
    this.max = 1,
    this.divisions,
  });
  final double value, min, max;
  final int? divisions;
  final ValueChanged<double>? onChanged, onChangeEnd;
  @override
  State<EditorSlider> createState() => _EditorSliderState();
}

class _EditorSliderState extends State<EditorSlider> {
  bool _down = false;
  static const _thumb = EditorMetrics.s5;

  double _fraction(double dx, double width) {
    final span = width - 2 * _thumb;
    return span <= 0 ? 0 : ((dx - _thumb) / span).clamp(0.0, 1.0);
  }

  void _at(double dx, double width) {
    var t = _fraction(dx, width);
    if (widget.divisions != null && widget.divisions! > 0) {
      t = (t * widget.divisions!).round() / widget.divisions!;
    }
    widget.onChanged?.call(widget.min + t * (widget.max - widget.min));
  }

  @override
  Widget build(BuildContext context) {
    final enabled = widget.onChanged != null;
    final span = widget.max - widget.min;
    final t = span <= 0
        ? 0.0
        : ((widget.value - widget.min) / span).clamp(0.0, 1.0);
    return Semantics(
      slider: true,
      enabled: enabled,
      value: widget.value.toStringAsFixed(0),
      child: LayoutBuilder(
        builder: (context, box) => GestureDetector(
          behavior: HitTestBehavior.opaque,
          onTapDown: enabled ? (d) => setState(() => _down = true) : null,
          onTapUp: enabled
              ? (d) {
                  _at(d.localPosition.dx, box.maxWidth);
                  setState(() => _down = false);
                  widget.onChangeEnd?.call(widget.value);
                }
              : null,
          onTapCancel: enabled ? () => setState(() => _down = false) : null,
          onHorizontalDragStart: enabled
              ? (d) {
                  setState(() => _down = true);
                  _at(d.localPosition.dx, box.maxWidth);
                }
              : null,
          onHorizontalDragUpdate: enabled
              ? (d) => _at(d.localPosition.dx, box.maxWidth)
              : null,
          onHorizontalDragEnd: enabled
              ? (_) {
                  setState(() => _down = false);
                  widget.onChangeEnd?.call(widget.value);
                }
              : null,
          onHorizontalDragCancel: enabled
              ? () => setState(() => _down = false)
              : null,
          child: CustomPaint(
            size: Size(
              box.maxWidth,
              box.hasBoundedHeight ? box.maxHeight : EditorMetrics.row,
            ),
            painter: _SliderPainter(
              colors: EditorTheme.of(context),
              t: t,
              enabled: enabled,
              pressed: _down,
              divisions: widget.divisions,
              label: _down && widget.divisions != null
                  ? widget.value.round().toString()
                  : null,
            ),
          ),
        ),
      ),
    );
  }
}

class _SliderPainter extends CustomPainter {
  final EditorTheme colors;

  const _SliderPainter({
    this.colors = EditorTheme.chromatic,
    required this.t,
    required this.enabled,
    required this.pressed,
    required this.divisions,
    required this.label,
  });
  final double t;
  final bool enabled, pressed;
  final int? divisions;
  final String? label;
  static const _thumb = EditorMetrics.s5, _track = EditorMetrics.s2;
  @override
  void paint(Canvas canvas, Size size) {
    final left = _thumb, right = size.width - _thumb;
    final cy = size.height / 2;
    final x = left + (right - left) * t;
    final track = Rect.fromLTRB(left, cy - _track / 2, right, cy + _track / 2);
    const r = Radius.circular(_track / 2);
    final active = Paint()..color = enabled ? colors.muted : colors.inkDisabled;
    final inactive = Paint()..color = colors.line;
    canvas.drawRRect(
      RRect.fromRectAndCorners(
        Rect.fromLTRB(track.left, track.top, x, track.bottom),
        topLeft: r,
        bottomLeft: r,
      ),
      active,
    );
    canvas.drawRRect(
      RRect.fromRectAndCorners(
        Rect.fromLTRB(x, track.top, track.right, track.bottom),
        topRight: r,
        bottomRight: r,
      ),
      inactive,
    );
    if (divisions != null && divisions! > 0) {
      final adjusted = track.width - track.height;
      const tick = _track / 2;
      if (adjusted / divisions! >= 3 * tick) {
        for (var i = 0; i <= divisions!; i++) {
          final tx = left + (right - left) * i / divisions!;
          canvas.drawCircle(
            Offset(tx, cy),
            tick / 2,
            Paint()..color = tx <= x ? colors.tickActive : colors.tickInactive,
          );
        }
      }
    }
    final thumb = Path()
      ..addOval(Rect.fromCircle(center: Offset(x, cy), radius: _thumb));
    canvas.drawShadow(thumb, EditorTheme.black, pressed ? 6 : 1, true);
    canvas.drawCircle(
      Offset(x, cy),
      _thumb,
      Paint()..color = enabled ? colors.ink : colors.inkDisabled,
    );
    if (label != null) {
      final text = TextPainter(
        text: TextSpan(
          text: label,
          style: TextStyle(
            fontFamily: EditorTheme.fontFamily,
            fontSize: EditorMetrics.font,
            color: colors.tabInk,
          ),
        ),
        textDirection: TextDirection.ltr,
      )..layout();
      final w = text.width + EditorMetrics.s16,
          h = text.height + EditorMetrics.s8;
      final box = Rect.fromCenter(
        center: Offset(x, cy - _thumb - EditorMetrics.s8 - h / 2),
        width: w,
        height: h,
      );
      canvas.drawRRect(
        RRect.fromRectAndRadius(box, const Radius.circular(EditorMetrics.s4)),
        Paint()..color = colors.accent,
      );
      text.paint(
        canvas,
        box.topLeft + const Offset(EditorMetrics.s8, EditorMetrics.s4),
      );
    }
  }

  @override
  bool shouldRepaint(_SliderPainter old) =>
      colors != old.colors ||
      old.t != t ||
      old.enabled != enabled ||
      old.pressed != pressed ||
      old.divisions != divisions ||
      old.label != label;
}

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

/// The tooltip's sheet: white at 90%, black 12 px text, a 4 px corner, at
/// least 24 tall — the desktop tooltip of the dark theme.
class EditorTooltipSheet extends StatelessWidget {
  const EditorTooltipSheet({super.key, required this.message});
  final String message;
  @override
  Widget build(BuildContext context) => Container(
    constraints: const BoxConstraints(minHeight: EditorMetrics.control),
    padding: const EdgeInsets.symmetric(
      horizontal: EditorMetrics.s8,
      vertical: EditorMetrics.s4,
    ),
    decoration: BoxDecoration(
      color: EditorTheme.of(context).tooltip,
      borderRadius: BorderRadius.all(Radius.circular(EditorMetrics.s4)),
    ),
    child: Center(
      widthFactor: 1,
      heightFactor: 1,
      child: Text(
        message,
        style: const TextStyle(
          fontSize: EditorMetrics.s12,
          color: EditorTheme.black,
        ),
      ),
    ),
  );
}

/// The menu's sheet, placed in the overlay at [anchor]: below it, flipped
/// above when the window's bottom is nearer, kept inside the window. Esc and
/// a tap outside close it; ↑ ↓ walk the rows and Enter presses one. The
/// sheet is [EditorTheme.of(context).menu] with a [EditorTheme.of(context).menuEdge] edge and
/// [EditorTheme.menuPadding], as wide as its widest row and no narrower
/// than [minWidth].
class EditorMenuSheet extends StatefulWidget {
  const EditorMenuSheet({
    super.key,
    required this.anchor,
    required this.onClose,
    required this.child,
    this.minWidth = EditorTheme.menuMinWidth,
    this.consumeOutsideTap = false,
  });

  /// Where the menu hangs from, in the overlay's coordinates.
  final Rect anchor;
  final VoidCallback onClose;
  final Widget child;
  final double minWidth;

  /// Whether the tap that closes the menu is swallowed or reaches what
  /// lies under it.
  final bool consumeOutsideTap;
  @override
  State<EditorMenuSheet> createState() => _EditorMenuSheetState();
}

class _EditorMenuSheetState extends State<EditorMenuSheet> {
  final _scope = FocusScopeNode(debugLabel: 'EditorMenu');
  FocusNode? _before;

  @override
  void initState() {
    super.initState();
    _before = FocusManager.instance.primaryFocus;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) _scope.requestFocus();
    });
  }

  @override
  void dispose() {
    final back = _before;
    _scope.dispose();
    if (back != null && back.context != null && back.canRequestFocus) {
      back.requestFocus();
    }
    super.dispose();
  }

  KeyEventResult _key(FocusNode node, KeyEvent event) {
    if (event is! KeyDownEvent && event is! KeyRepeatEvent) {
      return KeyEventResult.ignored;
    }
    final key = event.logicalKey;
    if (key == LogicalKeyboardKey.escape) {
      widget.onClose();
      return KeyEventResult.handled;
    }
    if (key == LogicalKeyboardKey.arrowDown) {
      final row = _scope.focusedChild;
      if (row == null || !row.nextFocus()) {
        _scope.traversalDescendants.firstOrNull?.requestFocus();
      }
      return KeyEventResult.handled;
    }
    if (key == LogicalKeyboardKey.arrowUp) {
      final row = _scope.focusedChild;
      if (row == null || !row.previousFocus()) {
        _scope.traversalDescendants.lastOrNull?.requestFocus();
      }
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  @override
  Widget build(BuildContext context) => Stack(
    children: [
      Positioned.fill(
        child: Listener(
          behavior: widget.consumeOutsideTap
              ? HitTestBehavior.opaque
              : HitTestBehavior.translucent,
          // A press on the anchor itself is the anchor's to toggle.
          onPointerDown: (e) {
            if (!widget.anchor.contains(e.localPosition)) widget.onClose();
          },
        ),
      ),
      CustomSingleChildLayout(
        delegate: _MenuLayout(widget.anchor),
        child: FocusScope(
          node: _scope,
          onKeyEvent: _key,
          child: DefaultTextStyle(
            style: EditorTheme.of(context).text,
            child: ConstrainedBox(
              constraints: BoxConstraints(minWidth: widget.minWidth),
              child: IntrinsicWidth(
                child: Container(
                  padding: EditorTheme.menuPadding,
                  decoration: BoxDecoration(
                    color: EditorTheme.of(context).menu,
                    border: Border.fromBorderSide(
                      BorderSide(color: EditorTheme.of(context).menuEdge),
                    ),
                  ),
                  child: widget.child,
                ),
              ),
            ),
          ),
        ),
      ),
    ],
  );
}

/// Below the anchor's start corner, above it if that fits better, and
/// never past the overlay's edges.
class _MenuLayout extends SingleChildLayoutDelegate {
  const _MenuLayout(this.anchor);
  final Rect anchor;
  @override
  BoxConstraints getConstraintsForChild(BoxConstraints constraints) =>
      constraints.loosen();
  @override
  Offset getPositionForChild(Size size, Size child) {
    var x = anchor.left;
    var y = anchor.bottom;
    if (y + child.height > size.height) {
      final above = anchor.top - child.height;
      y = above >= 0 ? above : math.max(0, size.height - child.height);
    }
    if (x + child.width > size.width) x = math.max(0, size.width - child.width);
    return Offset(x, y);
  }

  @override
  bool shouldRelayout(_MenuLayout old) => old.anchor != anchor;
}

/// One row of a menu: [EditorMetrics.row] high, [EditorTheme.menuRowPadding]
/// across, ink on the sheet, [EditorTheme.of(context).selectInk] on [EditorTheme.of(context).select]
/// while hovered or focused, [EditorTheme.of(context).disabledInk] when it cannot be
/// pressed.
class EditorMenuRow extends StatefulWidget {
  const EditorMenuRow({
    super.key,
    required this.onPressed,
    required this.child,
  });
  final VoidCallback? onPressed;
  final Widget child;
  @override
  State<EditorMenuRow> createState() => _EditorMenuRowState();
}

class _EditorMenuRowState extends State<EditorMenuRow> {
  bool _hover = false, _focus = false;
  @override
  Widget build(BuildContext context) {
    final enabled = widget.onPressed != null;
    final lit = enabled && (_hover || _focus);
    return Semantics(
      button: true,
      enabled: enabled,
      child: Focus(
        canRequestFocus: enabled,
        onFocusChange: (f) => setState(() => _focus = f),
        child: Actions(
          actions: {
            ActivateIntent: CallbackAction<ActivateIntent>(
              onInvoke: (_) => widget.onPressed?.call(),
            ),
            ButtonActivateIntent: CallbackAction<ButtonActivateIntent>(
              onInvoke: (_) => widget.onPressed?.call(),
            ),
          },
          child: MouseRegion(
            cursor: enabled
                ? SystemMouseCursors.click
                : SystemMouseCursors.basic,
            onEnter: (_) => setState(() => _hover = true),
            onExit: (_) => setState(() => _hover = false),
            child: GestureDetector(
              behavior: HitTestBehavior.opaque,
              excludeFromSemantics: true,
              onTap: widget.onPressed,
              child: Container(
                height: EditorMetrics.row,
                padding: EditorTheme.menuRowPadding,
                color: lit ? EditorTheme.of(context).select : EditorTheme.clear,
                alignment: Alignment.centerLeft,
                child: DefaultTextStyle.merge(
                  style: TextStyle(
                    fontSize: EditorMetrics.font,
                    color: !enabled
                        ? EditorTheme.of(context).disabledInk
                        : lit
                        ? EditorTheme.of(context).selectInk
                        : EditorTheme.of(context).ink,
                  ),
                  child: widget.child,
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}

/// A menu hung from a widget: press [builder]'s widget to open the sheet of
/// [items] below it (the choice's rows are [EditorMenuRow]s).
class EditorMenuAnchor extends StatefulWidget {
  const EditorMenuAnchor({
    super.key,
    required this.items,
    required this.builder,
  });
  final List<Widget> items;
  final Widget Function(BuildContext context, EditorMenuHandle menu) builder;
  @override
  State<EditorMenuAnchor> createState() => _EditorMenuAnchorState();
}

/// What the anchor's child can do with its menu.
abstract class EditorMenuHandle {
  bool get isOpen;
  void open();
  void close();
}

class _EditorMenuAnchorState extends State<EditorMenuAnchor>
    implements EditorMenuHandle {
  final _portal = OverlayPortalController();
  @override
  bool get isOpen => _portal.isShowing;
  @override
  void open() {
    if (!_portal.isShowing) setState(_portal.show);
  }

  @override
  void close() {
    if (_portal.isShowing) setState(_portal.hide);
  }

  Rect _anchorRect(BuildContext overlayContext) {
    final box = context.findRenderObject() as RenderBox;
    final overlay =
        Overlay.of(overlayContext).context.findRenderObject() as RenderBox;
    final origin = overlay.globalToLocal(box.localToGlobal(Offset.zero));
    return origin & box.size;
  }

  @override
  Widget build(BuildContext context) => OverlayPortal(
    controller: _portal,
    overlayChildBuilder: (overlayContext) => EditorMenuSheet(
      anchor: _anchorRect(overlayContext),
      onClose: close,
      child: _EditorMenuClose(
        close: close,
        child: Column(mainAxisSize: MainAxisSize.min, children: widget.items),
      ),
    ),
    child: widget.builder(context, this),
  );
}

/// Lets a row inside an [EditorMenuAnchor] close the menu it sits in.
class _EditorMenuClose extends InheritedWidget {
  const _EditorMenuClose({required this.close, required super.child});
  final VoidCallback close;
  @override
  bool updateShouldNotify(_EditorMenuClose old) => false;
  static VoidCallback? of(BuildContext context) =>
      context.dependOnInheritedWidgetOfExactType<_EditorMenuClose>()?.close;
}

/// A row that closes the menu around it once pressed.
class EditorMenuChoiceRow extends StatelessWidget {
  const EditorMenuChoiceRow({
    super.key,
    required this.onPressed,
    required this.child,
  });
  final VoidCallback? onPressed;
  final Widget child;
  @override
  Widget build(BuildContext context) => EditorMenuRow(
    onPressed: onPressed == null
        ? null
        : () {
            _EditorMenuClose.of(context)?.call();
            onPressed!();
          },
    child: child,
  );
}

/// A modal sheet over a black scrim, faded in over 150 ms: [EditorTheme.of(context).panel]
/// with a [EditorTheme.of(context).border] edge, a title, a body and a row of actions at
/// the end — the dialog Material drew under the editor's theme.
Future<T?> showEditorDialog<T>({
  required BuildContext context,
  required WidgetBuilder builder,
}) => showGeneralDialog<T>(
  context: context,
  barrierDismissible: true,
  barrierLabel: 'Dismiss',
  barrierColor: EditorTheme.of(context).scrim,
  transitionDuration: const Duration(milliseconds: 150),
  transitionBuilder: (context, animation, secondary, child) => FadeTransition(
    opacity: CurvedAnimation(parent: animation, curve: Curves.easeOut),
    child: child,
  ),
  pageBuilder: (context, animation, secondary) => Builder(builder: builder),
);

class EditorDialog extends StatelessWidget {
  const EditorDialog({
    super.key,
    this.title,
    this.content,
    this.actions = const [],
  });
  final Widget? title, content;
  final List<Widget> actions;
  @override
  Widget build(BuildContext context) => SafeArea(
    child: Center(
      child: Padding(
        padding: const EdgeInsets.symmetric(
          horizontal: EditorMetrics.s44,
          vertical: EditorMetrics.control,
        ),
        child: ConstrainedBox(
          constraints: const BoxConstraints(minWidth: EditorMetrics.s280),
          child: Container(
            decoration: BoxDecoration(
              color: EditorTheme.of(context).panel,
              border: Border.fromBorderSide(
                BorderSide(color: EditorTheme.of(context).border),
              ),
            ),
            child: IntrinsicWidth(
              child: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  if (title != null)
                    Padding(
                      padding: const EdgeInsets.fromLTRB(
                        EditorMetrics.control,
                        EditorMetrics.control,
                        EditorMetrics.control,
                        0,
                      ),
                      child: DefaultTextStyle(
                        style: TextStyle(
                          fontSize: EditorMetrics.title,
                          color: EditorTheme.of(context).ink,
                        ),
                        child: Semantics(namesRoute: true, child: title),
                      ),
                    ),
                  if (content != null)
                    Padding(
                      padding: const EdgeInsets.fromLTRB(
                        EditorMetrics.control,
                        EditorMetrics.s16,
                        EditorMetrics.control,
                        EditorMetrics.control,
                      ),
                      child: DefaultTextStyle(
                        style: TextStyle(
                          fontSize: EditorMetrics.font,
                          color: EditorTheme.of(context).ink,
                        ),
                        child: content!,
                      ),
                    ),
                  if (actions.isNotEmpty)
                    Padding(
                      padding: const EdgeInsets.fromLTRB(
                        EditorMetrics.control,
                        0,
                        EditorMetrics.control,
                        EditorMetrics.control,
                      ),
                      child: Row(
                        mainAxisAlignment: MainAxisAlignment.end,
                        children: [
                          for (var i = 0; i < actions.length; i++) ...[
                            if (i > 0) const SizedBox(width: EditorMetrics.s8),
                            actions[i],
                          ],
                        ],
                      ),
                    ),
                ],
              ),
            ),
          ),
        ),
      ),
    ),
  );
}

/// An indeterminate wait: a 36 px ring, a 4 px accent arc sweeping round it.
class EditorSpinner extends StatefulWidget {
  const EditorSpinner({super.key});
  @override
  State<EditorSpinner> createState() => _EditorSpinnerState();
}

class _EditorSpinnerState extends State<EditorSpinner>
    with SingleTickerProviderStateMixin {
  late final _turn = AnimationController(
    vsync: this,
    duration: const Duration(milliseconds: 1333),
  )..repeat();
  @override
  void dispose() {
    _turn.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => SizedBox.square(
    dimension: EditorMetrics.s36,
    child: CustomPaint(
      painter: _SpinnerPainter(colors: EditorTheme.of(context), _turn),
    ),
  );
}

class _SpinnerPainter extends CustomPainter {
  final EditorTheme colors;

  _SpinnerPainter(this.turn, {this.colors = EditorTheme.chromatic})
    : super(repaint: turn);
  final Animation<double> turn;
  @override
  void paint(Canvas canvas, Size size) {
    final t = turn.value;
    final head = Curves.easeInOut.transform(((t * 2) % 1));
    final start = t * 2 * math.pi * 2 + head * math.pi;
    final sweep = math.pi / 2 + head * math.pi;
    canvas.drawArc(
      (Offset.zero & size).deflate(EditorMetrics.s2),
      start,
      sweep,
      false,
      Paint()
        ..color = colors.accent
        ..style = PaintingStyle.stroke
        ..strokeWidth = EditorMetrics.s4
        ..strokeCap = StrokeCap.butt,
    );
  }

  @override
  bool shouldRepaint(_SpinnerPainter old) =>
      colors != old.colors || old.turn != turn;
}

/// The one value out of a short list, drawn as before: the current label in
/// a bordered box with a drop glyph, the menu below it.
class EditorChoice<T> extends StatelessWidget {
  const EditorChoice({
    super.key,
    required this.value,
    required this.choices,
    required this.onChanged,
  });
  final T? value;
  final List<MapEntry<T, String>> choices;
  final ValueChanged<T>? onChanged;
  @override
  Widget build(BuildContext context) {
    final enabled = onChanged != null;
    final label = choices
        .where((e) => e.key == value)
        .map((e) => e.value)
        .firstOrNull;
    return EditorMenuAnchor(
      items: [
        for (final e in choices)
          EditorMenuChoiceRow(
            onPressed: enabled ? () => onChanged!(e.key) : null,
            child: Text(e.value, maxLines: 1, overflow: TextOverflow.ellipsis),
          ),
      ],
      builder: (context, menu) => GestureDetector(
        onTap: enabled ? (menu.isOpen ? menu.close : menu.open) : null,
        child: Container(
          height: EditorMetrics.row,
          padding: const EdgeInsets.only(left: EditorMetrics.s4),
          decoration: BoxDecoration(
            color: EditorTheme.of(context).app,
            border: Border.all(
              color: enabled
                  ? EditorTheme.of(context).border
                  : EditorTheme.of(context).line,
            ),
          ),
          child: Row(
            children: [
              Expanded(
                child: Text(
                  label ?? '',
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: TextStyle(
                    fontSize: EditorMetrics.font,
                    color: enabled
                        ? EditorTheme.of(context).ink
                        : EditorTheme.of(context).muted,
                  ),
                ),
              ),
              Icon(
                Glyph.arrow_drop_down,
                size: EditorMetrics.s16,
                color: EditorTheme.of(context).muted,
              ),
            ],
          ),
        ),
      ),
    );
  }
}
