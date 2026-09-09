import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../session/editor_session.dart';

class EditorShortcuts {
  EditorShortcuts(
    this.c, {
    required this.onMenu,
    required this.hasSheet,
    required this.closeSheet,
    required this.showComposition,
    required this.showInspector,
  });
  final EditorSession c;
  final void Function(String) onMenu;
  final bool Function() hasSheet;
  final VoidCallback closeSheet, showComposition, showInspector;
  bool get typing {
    final x = FocusManager.instance.primaryFocus?.context;
    return x != null &&
        (x.widget is EditableText ||
            x.findAncestorWidgetOfExactType<EditableText>() != null);
  }

  KeyEventResult handle(FocusNode node, KeyEvent event) {
    if (event is! KeyDownEvent || typing) return KeyEventResult.ignored;
    final h = HardwareKeyboard.instance;
    final cmd = h.isMetaPressed || h.isControlPressed;
    final shift = h.isShiftPressed;
    final alt = h.isAltPressed;
    final k = event.logicalKey;
    String? op;
    if (k == LogicalKeyboardKey.escape) {
      c.cancelPreview();
      if (hasSheet())
        closeSheet();
      else if (c.deskDrawer.value != null)
        c.deskDrawer.value = null;
      else
        c.command('select', {'ids': <int>[], 'keys': []});
      return KeyEventResult.handled;
    }
    if (k == LogicalKeyboardKey.space) {
      c.togglePlayback();
      return KeyEventResult.handled;
    }
    if (k == LogicalKeyboardKey.f9) {
      c.command('ease', {
        'kind': cmd && shift
            ? 'EasyEaseOut'
            : shift
            ? 'EasyEaseIn'
            : 'EasyEase',
      });
      if (c.activeLayer != null) {
        c.focusEditing(c.activeLayer!['id'] as int, 'keyframes');
      }
      return KeyEventResult.handled;
    }
    if (cmd) {
      if (k == LogicalKeyboardKey.keyZ) op = shift ? 'redo' : 'undo';
      if (k == LogicalKeyboardKey.keyC) op = 'copy';
      if (k == LogicalKeyboardKey.keyX) op = 'cut';
      if (k == LogicalKeyboardKey.keyV) op = 'paste';
      if (k == LogicalKeyboardKey.keyD) {
        if (!shift) {
          op = 'duplicate';
        } else {
          c.reselectKeys();
          return KeyEventResult.handled;
        }
      }
      if (k == LogicalKeyboardKey.keyG) op = shift ? 'ungroup' : 'group';
      if (k == LogicalKeyboardKey.keyA) {
        c.command('select', {'ids': c.layers.map((l) => l['id']).toList()});
        return KeyEventResult.handled;
      }
      if (k == LogicalKeyboardKey.keyS) {
        onMenu(shift ? 'Save as' : 'Save');
        return KeyEventResult.handled;
      }
      if (k == LogicalKeyboardKey.keyO) {
        onMenu('Open');
        return KeyEventResult.handled;
      }
      if (k == LogicalKeyboardKey.keyN) {
        onMenu('New');
        return KeyEventResult.handled;
      }
      if (k == LogicalKeyboardKey.keyK) {
        if (alt)
          showComposition();
        else
          op = 'split';
      }
      if (k == LogicalKeyboardKey.digit0) {
        c.viewCommand.value = null;
        c.viewCommand.value = 'Fit';
        return KeyEventResult.handled;
      }
      if (k == LogicalKeyboardKey.digit1) {
        c.viewCommand.value = null;
        c.viewCommand.value = 'Actual';
        return KeyEventResult.handled;
      }
      if (k == LogicalKeyboardKey.equal) {
        c.viewCommand.value = null;
        c.viewCommand.value = 'In';
        return KeyEventResult.handled;
      }
      if (k == LogicalKeyboardKey.minus) {
        c.viewCommand.value = null;
        c.viewCommand.value = 'Out';
        return KeyEventResult.handled;
      }
    } else {
      if (k == LogicalKeyboardKey.delete || k == LogicalKeyboardKey.backspace)
        op = 'delete';
      if (k == LogicalKeyboardKey.home) {
        c.seek(0);
        return KeyEventResult.handled;
      }
      if (k == LogicalKeyboardKey.end) {
        c.seek((c.state['durationFrames'] as num? ?? 1).toInt() - 1);
        return KeyEventResult.handled;
      }
      if (k == LogicalKeyboardKey.arrowLeft ||
          k == LogicalKeyboardKey.arrowRight) {
        final d =
            (k == LogicalKeyboardKey.arrowLeft ? -1 : 1) * (shift ? 10 : 1);
        if (alt) {
          if ((c.state['selectedKeys'] as List? ?? []).isNotEmpty)
            c.command('moveKeys', {'deltaFrames': d});
          else
            _nudge(d.toDouble(), 0);
        } else
          c.seek(c.frame.value + d);
        return KeyEventResult.handled;
      }
      if (k == LogicalKeyboardKey.arrowUp ||
          k == LogicalKeyboardKey.arrowDown) {
        final d = (k == LogicalKeyboardKey.arrowUp ? -1 : 1) * (shift ? 10 : 1);
        if (alt)
          _nudge(0, d.toDouble());
        else {
          final ids = c.layers.map((l) => (l['id'] as num).toInt()).toList();
          final ix = ids.indexOf(
            c.selectedIds.isEmpty ? -1 : c.selectedIds.last,
          );
          if (ids.isNotEmpty)
            c.command('select', {
              'ids': [ids[(ix + d.sign).clamp(0, ids.length - 1)]],
            });
        }
        return KeyEventResult.handled;
      }
      if (k == LogicalKeyboardKey.keyM) op = 'addMarker';
      if (k == LogicalKeyboardKey.keyU) {
        c.keyedOnly.value = !c.keyedOnly.value;
        return KeyEventResult.handled;
      }
      if (k == LogicalKeyboardKey.keyA && !shift) {
        c.setAnimate(!c.animating);
        return KeyEventResult.handled;
      }
      final props = {
        LogicalKeyboardKey.keyP: 'position',
        LogicalKeyboardKey.keyS: 'scale',
        LogicalKeyboardKey.keyR: 'rotation',
        LogicalKeyboardKey.keyT: 'opacity',
        LogicalKeyboardKey.keyA: 'anchor',
      };
      if (props.containsKey(k)) {
        showInspector();
        c.focusProperty.value = null;
        c.focusProperty.value = props[k];
        return KeyEventResult.handled;
      }
    }
    if (op != null) {
      c.command(op);
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  Future<void> _nudge(double x, double y) async {
    final corners = c.activeLayer?['corners'] as List?;
    if (corners == null || corners.isEmpty) return;
    final p = (corners.first as List)
        .map((v) => (v as num).toDouble())
        .toList();
    await c.command('stageGesture', {
      'phase': 'begin',
      'mode': 'move',
      'ids': c.selectedIds,
      'start': p,
      'point': p,
      'handle': 'body',
    });
    await c.command('stageGesture', {
      'phase': 'update',
      'point': [p[0] + x, p[1] + y],
    });
    await c.command('stageGesture', {'phase': 'commit'});
  }
}
