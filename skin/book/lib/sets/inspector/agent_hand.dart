// A hand for a tester who only sees pictures: lines appended to <dir>/inbox.txt are done to the table as a mouse would do them,
// and after each the table is shot to <dir>/NNN.png and "NNN <line>" is added to <dir>/done.txt. Coordinates are the shot's pixels.
//   tap X Y                 press and let go
//   hold X Y                press, wait 0.7 s, let go
//   drag X1 Y1 X2 Y2        press, move, let go
//   holddrag X1 Y1 X2 Y2    press, wait 0.5 s (a hold), move, let go
//   dragrest X1 Y1 X2 Y2    press, move, rest there 0.6 s, let go
//   drag3 X1 Y1 X2 Y2 X3 Y3 press, move to the second point, rest 0.6 s, move on to the third, let go
//   type TEXT               types into the number being typed and presses Return
//   key cmd+z | cmd+shift+z | escape
//   look                    only a shot
// <dir> is the AGENT_DIR environment variable (a test window per tester), else no hand.
import 'dart:async';
import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/gestures.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart' show TextEditingValue, TextInputAction, TextSelection;
import 'package:flutter/widgets.dart';

import 'cards_table.dart';

class AgentHand extends StatefulWidget {
  const AgentHand(this.t, {super.key, required this.child});
  final CardTable t;
  final Widget child;
  @override
  State<AgentHand> createState() => _AgentHandState();
}

class _AgentHandState extends State<AgentHand> {
  final _shot = GlobalKey();
  final _dir = Platform.environment['AGENT_DIR'];
  Timer? _poll;
  int _read = 0, _n = 0, _pointer = 1000;
  bool _busy = false;

  @override
  void initState() {
    super.initState();
    if (_dir == null) return;
    Directory(_dir).createSync(recursive: true);
    File('$_dir/inbox.txt').writeAsStringSync('');
    File('$_dir/done.txt').writeAsStringSync('');
    WidgetsBinding.instance.addPostFrameCallback((_) => _do('look'));
    _poll = Timer.periodic(const Duration(milliseconds: 150), (_) => _check());
  }

  @override
  void dispose() {
    _poll?.cancel();
    super.dispose();
  }

  Future<void> _check() async {
    if (_busy) return;
    final lines = File('$_dir/inbox.txt').readAsLinesSync().where((l) => l.trim().isNotEmpty).toList();
    if (lines.length <= _read) return;
    _busy = true;
    while (_read < lines.length) {
      await _do(lines[_read++].trim());
    }
    _busy = false;
  }

  Offset _g(double x, double y) => (_shot.currentContext!.findRenderObject()! as RenderBox).localToGlobal(Offset(x, y));
  Future<void> _wait(int ms) => Future.delayed(Duration(milliseconds: ms));
  void _send(PointerEvent e) => GestureBinding.instance.handlePointerEvent(e);

  Future<void> _press(Offset at) async {
    _pointer++;
    _send(PointerAddedEvent(kind: PointerDeviceKind.mouse, device: 77, position: at));
    _send(PointerDownEvent(pointer: _pointer, kind: PointerDeviceKind.mouse, device: 77, position: at, buttons: kPrimaryButton));
    await _wait(16);
  }

  Offset _at = Offset.zero;
  Future<void> _move(Offset from, Offset to) async {
    for (var k = 1; k <= 16; k++) {
      final p = Offset.lerp(from, to, k / 16)!;
      _send(PointerMoveEvent(pointer: _pointer, kind: PointerDeviceKind.mouse, device: 77, position: p, delta: p - _at, buttons: kPrimaryButton));
      _at = p;
      await _wait(16);
    }
  }

  Future<void> _let(Offset at) async {
    _send(PointerUpEvent(pointer: _pointer, kind: PointerDeviceKind.mouse, device: 77, position: at));
    await _wait(16);
  }

  /// One line done; anything not understood is written back as such.
  Future<void> _do(String line) async {
    final w = line.split(RegExp(r'\s+')), n = w.skip(1).map(double.tryParse).toList();
    Offset p(int i) => _g(n[i * 2] ?? 0, n[i * 2 + 1] ?? 0);
    var said = line;
    try {
      switch (w.first) {
        case 'tap':
          await _press(_at = p(0));
          await _let(_at);
        case 'hold':
          await _press(_at = p(0));
          await _wait(700);
          await _let(_at);
        case 'drag':
          await _press(_at = p(0));
          await _move(_at, p(1));
          await _let(_at);
        case 'holddrag':
          await _press(_at = p(0));
          await _wait(500);
          await _move(_at, p(1));
          await _let(_at);
        case 'dragrest':
          await _press(_at = p(0));
          await _move(_at, p(1));
          await _wait(600);
          await _let(_at);
        case 'drag3':
          await _press(_at = p(0));
          await _move(_at, p(1));
          await _wait(600);
          await _move(_at, p(2));
          await _let(_at);
        case 'type':
          _typeInto(context, line.substring(5));
        case 'key':
          switch (w.length > 1 ? w[1] : '') {
            case 'cmd+z':
              widget.t.undo();
            case 'cmd+shift+z':
              widget.t.redo();
            case 'escape':
              widget.t.type(null);
            default:
              said = '$line   (not a key this hand knows)';
          }
        case 'look':
          break;
        default:
          said = '$line   (not understood: tap hold drag holddrag dragrest drag3 type key look)';
      }
    } catch (e) {
      said = '$line   (failed: $e)';
    }
    await _wait(450);
    await _snap(said);
  }

  void _typeInto(BuildContext root, String text) {
    EditableTextState? found;
    void walk(Element e) {
      if (e is StatefulElement && e.state is EditableTextState) found = e.state as EditableTextState;
      e.visitChildElements(walk);
    }
    (root as Element).visitChildElements(walk);
    final s = found;
    if (s == null) throw 'no number is being typed (tap a number first)';
    s.updateEditingValue(TextEditingValue(text: text, selection: TextSelection.collapsed(offset: text.length)));
    s.performAction(TextInputAction.done);
  }

  Future<void> _snap(String said) async {
    final b = _shot.currentContext?.findRenderObject() as RenderRepaintBoundary?;
    if (b == null) return;
    final img = await b.toImage(pixelRatio: 1);
    final png = await img.toByteData(format: ui.ImageByteFormat.png);
    final name = (++_n).toString().padLeft(3, '0');
    File('$_dir/$name.png').writeAsBytesSync(png!.buffer.asUint8List());
    File('$_dir/done.txt').writeAsStringSync('$name $said\n', mode: FileMode.append);
  }

  @override
  Widget build(BuildContext context) => RepaintBoundary(key: _shot, child: widget.child);
}
