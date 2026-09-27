import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../session/editor_session.dart';
import 'adapters/document.dart';

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
        op('select', {'ids': <int>[], 'keys': []});
      case LogicalKeyboardKey.space:
        c.togglePlayback();
      case LogicalKeyboardKey.f9:
        op('ease', {
          'kind': cmd && shift
              ? 'EasyEaseOut'
              : (shift ? 'EasyEaseIn' : 'EasyEase'),
        });
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
      case LogicalKeyboardKey.keyN when cmd:
        _replace(() => c.command('new'));
      case LogicalKeyboardKey.keyO when cmd:
        _replace(c.chooseOpen);
      case LogicalKeyboardKey.keyI when cmd:
        c.importFiles();
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
        } else {
          _seek(c.frame.value + d);
        }
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
        c.focusProperty.value = null;
        c.focusProperty.value = {
          LogicalKeyboardKey.keyP: 'position',
          LogicalKeyboardKey.keyS: 'scale',
          LogicalKeyboardKey.keyR: 'rotation',
          LogicalKeyboardKey.keyT: 'opacity',
        }[k];
      default:
        return KeyEventResult.ignored;
    }
    return KeyEventResult.handled;
  }
}
