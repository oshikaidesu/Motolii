import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../session/editor_session.dart';
import '../session/stage_actions.dart';
import 'adapters/document.dart';
import 'adapters/sheets.dart';

/// The window's keys, each one an existing host operation. A key typed into a text field is the field's.
class LiveKeys {
  LiveKeys(this.c, this.context);
  final EditorSession c;
  final BuildContext Function() context;

  bool get _typing {
    final x = FocusManager.instance.primaryFocus?.context;
    return x != null &&
        (x.widget is EditableText ||
            x.findAncestorWidgetOfExactType<EditableText>() != null);
  }

  void _seek(int frame) => c.seek(
    frame.clamp(
      0,
      ((c.state['durationFrames'] as num? ?? 1).toInt() - 1).clamp(0, 1 << 30),
    ),
  );

  Future<void> _replace(Future<void> Function() then) async {
    final ctx = context();
    if (ctx.mounted && await mayReplace(ctx, c)) await then();
  }

  KeyEventResult handle(FocusNode node, KeyEvent event) {
    if (event is! KeyDownEvent || _typing) return KeyEventResult.ignored;
    final h = HardwareKeyboard.instance;
    final cmd = h.isMetaPressed || h.isControlPressed,
        shift = h.isShiftPressed,
        alt = h.isAltPressed;
    final k = event.logicalKey;
    void op(String name, [Map<String, dynamic> args = const {}]) => c.command(name, args);
    switch (k) {
      case LogicalKeyboardKey.escape:
        c.cancelPreview();
        if (c.deskDrawer.value != null) {
          c.deskDrawer.value = null;
        } else {
          op('select', {'ids': <int>[], 'keys': []});
        }
      case LogicalKeyboardKey.space:
        c.togglePlayback();
      case LogicalKeyboardKey.f9:
        op('ease', {
          'kind': cmd && shift
              ? 'EasyEaseOut'
              : (shift ? 'EasyEaseIn' : 'EasyEase'),
        });
        final active = c.activeLayer;
        if (active != null) c.focusEditing(active['id'] as int, 'keyframes');
      case LogicalKeyboardKey.keyZ when cmd:
        op(shift ? 'redo' : 'undo');
      case LogicalKeyboardKey.keyC when cmd:
        op('copy');
      case LogicalKeyboardKey.keyX when cmd:
        op('cut');
      case LogicalKeyboardKey.keyV when cmd:
        op('paste');
      case LogicalKeyboardKey.keyD when cmd:
        shift ? c.reselectKeys() : op('duplicate');
      case LogicalKeyboardKey.keyG when cmd:
        op(shift ? 'ungroup' : 'group');
      case LogicalKeyboardKey.keyA when cmd:
        op('select', {
          'ids': [for (final l in c.layers) l['id']],
        });
      case LogicalKeyboardKey.keyS when cmd:
        c.save(as: shift);
      case LogicalKeyboardKey.keyK when cmd && !alt:
        op('split');
      case LogicalKeyboardKey.keyK when cmd && alt:
        final ctx = context();
        if (ctx.mounted) showCompositionSheet(ctx, c);
      case LogicalKeyboardKey.keyN when cmd:
        _replace(() => c.command('new'));
      case LogicalKeyboardKey.keyO when cmd:
        _replace(c.chooseOpen);
      case LogicalKeyboardKey.keyI when cmd:
        c.importFiles();
      case LogicalKeyboardKey.digit0 when cmd:
        stageView(c, 'Fit');
      case LogicalKeyboardKey.digit1 when cmd:
        stageView(c, 'Actual');
      case LogicalKeyboardKey.equal when cmd:
        stageView(c, 'In');
      case LogicalKeyboardKey.minus when cmd:
        stageView(c, 'Out');
      case LogicalKeyboardKey.delete || LogicalKeyboardKey.backspace when !cmd:
        op('delete');
      case LogicalKeyboardKey.home:
        _seek(0);
      case LogicalKeyboardKey.end:
        _seek(1 << 30);
      case LogicalKeyboardKey.arrowLeft || LogicalKeyboardKey.arrowRight
          when !cmd:
        final d =
            (k == LogicalKeyboardKey.arrowLeft ? -1 : 1) * (shift ? 10 : 1);
        if (alt && (c.state['selectedKeys'] as List? ?? const []).isNotEmpty) {
          op('moveKeys', {'deltaFrames': d});
        } else if (alt) {
          nudgeSelection(c, d.toDouble(), 0);
        } else {
          _seek(c.frame.value + d);
        }
      case LogicalKeyboardKey.arrowUp || LogicalKeyboardKey.arrowDown
          when !cmd && alt:
        nudgeSelection(c, 0, (k == LogicalKeyboardKey.arrowUp ? -1.0 : 1.0) * (shift ? 10 : 1));
      case LogicalKeyboardKey.arrowUp || LogicalKeyboardKey.arrowDown
          when !cmd && !alt:
        final ids = [for (final l in c.layers) (l['id'] as num).toInt()];
        if (ids.isNotEmpty) {
          final ix = ids.indexOf(
            c.selectedIds.isEmpty ? -1 : c.selectedIds.last,
          );
          op('select', {
            'ids': [
              ids[(ix + (k == LogicalKeyboardKey.arrowUp ? -1 : 1)).clamp(
                0,
                ids.length - 1,
              )],
            ],
          });
        }
      case LogicalKeyboardKey.keyM when !cmd:
        op('addMarker');
      case LogicalKeyboardKey.keyA when !cmd && !shift:
        if (c.supports('animate')) c.setAnimate(!c.animating);
      case LogicalKeyboardKey.keyP ||
              LogicalKeyboardKey.keyS ||
              LogicalKeyboardKey.keyR ||
              LogicalKeyboardKey.keyT
          when !cmd:
        final property = {
          LogicalKeyboardKey.keyP: 'position',
          LogicalKeyboardKey.keyS: 'scale',
          LogicalKeyboardKey.keyR: 'rotation',
          LogicalKeyboardKey.keyT: 'opacity',
        }[k];
        // A hidden or closed Inspector is built by the frame after it is shown: ask for the field once it exists.
        () async {
          await c.placePanel('Inspector', 'show');
          await WidgetsBinding.instance.endOfFrame;
          c.focusProperty.value = null;
          c.focusProperty.value = property;
        }();
      default:
        return KeyEventResult.ignored;
    }
    return KeyEventResult.handled;
  }
}
