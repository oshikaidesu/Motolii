import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../metrics.dart';
import '../theme.dart';

/// The leaves a pointer presses: the hover-and-press surface itself, and the
/// two buttons built on it — the icon and the label.

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
