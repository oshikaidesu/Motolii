part of '../browser.dart';

/// The panel's keyboard: find, clear, collect, select all, then the grid's
/// arrows, Enter to apply and Delete on the shelf.
extension BrowserFrameKeys on _BrowserPanelState {
  KeyEventResult key(FocusNode node, KeyEvent event) {
    if (event is! KeyDownEvent ||
        searchFocus.hasFocus ||
        FocusManager.instance.primaryFocus?.context
                ?.findAncestorWidgetOfExactType<EditableText>() !=
            null)
      return KeyEventResult.ignored;
    final k = event.logicalKey;
    final primary =
        HardwareKeyboard.instance.isMetaPressed ||
        HardwareKeyboard.instance.isControlPressed;
    if (primary && k == LogicalKeyboardKey.keyF) {
      searchFocus.requestFocus();
      search.selection = TextSelection(
        baseOffset: 0,
        extentOffset: search.text.length,
      );
      return KeyEventResult.handled;
    }
    if (k == LogicalKeyboardKey.escape) {
      if (search.text.isNotEmpty) {
        search.clear();
        relist();
      } else {
        selected[tab]?.clear();
        active.remove(tab);
        _publish();
      }
      return KeyEventResult.handled;
    }
    if (primary && k == LogicalKeyboardKey.keyE) {
      if (selectedIds.isNotEmpty) quickAddFocus.requestFocus();
      return KeyEventResult.handled;
    }
    final digit = k.keyLabel.length == 1 ? int.tryParse(k.keyLabel) : null;
    if (!primary &&
        digit != null &&
        digit <= BrowserLibrary.collectionCount &&
        selectedIds.isNotEmpty) {
      library.collect(tab, selectedIds.toList(), digit);
      return KeyEventResult.handled;
    }
    if (primary && k == LogicalKeyboardKey.keyA && shelf.multiSelect) {
      selected[tab] = visible.map(id).toSet();
      _publish();
      return KeyEventResult.handled;
    }
    if (visible.isEmpty) return KeyEventResult.ignored;
    final current = visible.indexWhere((e) => id(e) == active[tab]);
    if (k == LogicalKeyboardKey.enter || k == LogicalKeyboardKey.numpadEnter) {
      apply(visible[current < 0 ? 0 : current]);
      return KeyEventResult.handled;
    }
    int? next;
    if (k == LogicalKeyboardKey.arrowLeft) next = current - 1;
    if (k == LogicalKeyboardKey.arrowRight) next = current + 1;
    if (k == LogicalKeyboardKey.arrowUp) next = current - columns;
    if (k == LogicalKeyboardKey.arrowDown) next = current + columns;
    if (k == LogicalKeyboardKey.home) next = 0;
    if (k == LogicalKeyboardKey.end) next = visible.length - 1;
    if (next != null) {
      select(visible[next.clamp(0, visible.length - 1)]);
      return KeyEventResult.handled;
    }
    if (k == LogicalKeyboardKey.delete || k == LogicalKeyboardKey.backspace) {
      shelf.delete(this, visible[current < 0 ? 0 : current]);
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }
}
