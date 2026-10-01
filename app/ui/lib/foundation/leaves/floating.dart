import 'dart:math' as math;

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../metrics.dart';
import '../theme.dart';

/// The leaves that float over the panels in the overlay: the tooltip's sheet,
/// the menu's sheet, where it hangs and the rows inside it.

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
