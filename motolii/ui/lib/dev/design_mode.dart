// surface-file: Design Mode is debug-only tool chrome, not product UI; its sizes are its own and it is never in a release build
// Design Mode: in the running debug window, click a thing, see which token / literal / layout decides its size, change the number with
// the wheel or the arrow keys, and see it at once. Flutter's own Inspector does the clicking and the widget's creation location
// (WidgetInspectorService, the same service DevTools uses); this adds the HUD, the token behind each value, and writing the number into
// the source so the existing hot reload (`scripts/motolii-ui.sh design` keeps its pid) puts it on screen. See docs/design/ui-debugging.md.
//
//   F2 or Cmd+Option+D  Design Mode on / off       Tab / Shift+Tab  next / previous value     wheel, Up / Down  +-1 (Shift 0.1, Alt 5)
//   A  BEFORE <-> CURRENT                     R  this value back                          Esc  cancel what you just did to this value
//   Z or Cmd+Z / Shift+Z  undo / redo          C  list of changes                          [ / ]  UI Scale -1 / +1 (Shift 10)
import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:math' as math;

import 'package:flutter/gestures.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../app/ui_scale.dart';
import '../theme/metrics.dart' show UiScale;
import 'design_core.dart';

const _startOn = bool.fromEnvironment('MOTOLII_DESIGN');

/// Wraps the app in debug builds: the HUD above it, the keys and the wheel while Design Mode is on.
class DesignMode extends StatefulWidget {
  const DesignMode({super.key, required this.child});
  final Widget child;

  @override
  State<DesignMode> createState() => _DesignModeState();
}

class _DesignModeState extends State<DesignMode> {
  final c = DesignController();

  @override
  void initState() {
    super.initState();
    c.addListener(() => setState(() {}));
    if (_startOn) WidgetsBinding.instance.addPostFrameCallback((_) => c.toggle());
  }

  /// Runs on every hot reload: the reload the tool was waiting for has landed.
  @override
  void reassemble() {
    super.reassemble();
    c.reloaded();
  }

  @override
  void dispose() {
    c.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Stack(
    textDirection: TextDirection.ltr,
    fit: StackFit.expand,
    children: [
      widget.child,
      // above the whole app, outside its Directionality: the HUD brings its own
      if (c.on)
        Directionality(
          textDirection: TextDirection.ltr,
          child: Stack(
            fit: StackFit.expand,
            children: [
              IgnorePointer(child: CustomPaint(painter: _BoundsPainter(c))),
              _Hud(c),
            ],
          ),
        ),
    ],
  );
}

enum _Badge { token, constToken, exception, raw, derived, named }

class _Row {
  _Row({
    required this.node,
    required this.arg,
    required this.text,
    required this.badge,
    this.value,
    this.ref,
    this.token,
    this.usage,
    this.element,
    this.file,
    this.line = 0,
  });
  final String node, arg, text;
  final _Badge badge;
  double? value;
  final Ref? ref;
  final Token? token;
  final String? usage;
  final Element? element;
  final String? file;
  final int line;
  bool edited = false;
  bool get editable => badge == _Badge.token || badge == _Badge.exception || badge == _Badge.raw;
}

class DesignController extends ChangeNotifier {
  DesignController() {
    session = DesignSession(_read, _write);
    FocusManager.instance.addEarlyKeyEventHandler(_early);
    GestureBinding.instance.pointerRouter.addGlobalRoute(_pointer);
    WidgetInspectorService.instance.selection.addListener(_selected);
  }

  late final DesignSession session;
  bool on = false, exiting = false, showChanges = false;
  List<_Row> rows = [];
  int active = 0;
  String? note;
  String title = '';
  String _root = '';
  Map<String, Token> _tokens = {};
  int _opsSinceAck = 0, _historyAtActivation = 0;
  Timer? _reloadTimer, _ackTimer;
  double _wheel = 0;

  @override
  void dispose() {
    FocusManager.instance.removeEarlyKeyEventHandler(_early);
    GestureBinding.instance.pointerRouter.removeGlobalRoute(_pointer);
    WidgetInspectorService.instance.selection.removeListener(_selected);
    _reloadTimer?.cancel();
    _ackTimer?.cancel();
    super.dispose();
  }

  // ------------------------------------------------------------------------------------------------------------ files

  String _read(String rel) => File('$_root/$rel').readAsStringSync();
  void _write(String rel, String text) => File('$_root/$rel').writeAsStringSync(text);

  // ------------------------------------------------------------------------------------------------------------ mode

  void toggle() {
    if (on && !exiting && session.changed) {
      exiting = true; // keep or discard first
      notifyListeners();
      return;
    }
    exiting = false;
    on = !on;
    WidgetsBinding.instance.debugShowWidgetInspectorOverride = on;
    if (on) {
      _selected();
    } else {
      showChanges = false;
    }
    notifyListeners();
  }

  void toggleChanges() {
    showChanges = !showChanges;
    notifyListeners();
  }

  /// Back from the keep / discard question to working.
  void toggleOnly() {
    exiting = false;
    notifyListeners();
  }

  void keep() {
    if (session.showingBefore) _after(session.toggleBefore());
    session.keep();
    exiting = false;
    toggle();
  }

  void discard() {
    _after(session.discard());
    exiting = false;
    note = null;
    toggle();
  }

  // ------------------------------------------------------------------------------------------------------------ selection

  void _selected() {
    if (!on) return;
    scheduleMicrotask(refresh);
  }

  static final _measure = RegExp(
    r'^(padding|margin|width|height|size|spacing|runSpacing|radius|borderRadius|top|left|right|bottom|dimension|strokeWidth|thickness|fontSize|letterSpacing|flex|minWidth|maxWidth|minHeight|maxHeight|gap|inset|offset|blurRadius|spreadRadius|constraints|decoration|border|alignment)$',
  );

  static const _geometry = {
    'Padding',
    'Container',
    'SizedBox',
    'ConstrainedBox',
    'Align',
    'Center',
    'Positioned',
    'Expanded',
    'Flexible',
    'Row',
    'Column',
    'DecoratedBox',
    'ClipRRect',
    'Text',
    'DefaultTextStyle',
    'EditableText',
    'GestureDetector',
    'LayoutBuilder',
    'Stack',
  };

  /// Builds the rows for what is selected: the clicked widget's own values first, then those of the nearest ancestors that set geometry.
  void refresh() {
    final svc = WidgetInspectorService.instance;
    final el = svc.selection.currentElement;
    if (el == null) {
      rows = [];
      title = '';
      notifyListeners();
      return;
    }
    const g = 'design';
    // The inspector's object-group calls are @protected for tools that subclass the service; this is the tool, it calls the singleton.
    // ignore: invalid_use_of_protected_member
    svc.disposeGroup(g);
    final summary = svc.getSelectedSummaryWidget(null, g);
    final sel = jsonDecode(summary);
    if (sel is! Map) {
      rows = [];
      notifyListeners();
      return;
    }
    // ignore: invalid_use_of_protected_member
    final chainJson = svc.getParentChain(sel['valueId'] as String, g);
    final chain = (jsonDecode(chainJson) as List).map((e) => (e as Map)['node'] as Map).toList().reversed.toList();
    final shown = <Map>[
      sel,
      ...chain
          .skip(1)
          .where((n) => projectPath((n['creationLocation'] as Map?)?['file'] as String?) != null && _geometry.contains(_typeOf(n)))
          .take(3),
    ];
    if (_root.isEmpty) {
      for (final n in shown) {
        final f = (n['creationLocation'] as Map?)?['file'] as String?;
        if (projectPath(f) != null) {
          _root = f!.replaceFirst('file://', '').replaceFirst(RegExp(r'/lib/.*$'), '');
          break;
        }
      }
    }
    if (_root.isEmpty) return;
    _tokens = parseTokens(_read('lib/theme/metrics.dart')); // fresh: an edit changes the value a token has
    final next = <_Row>[];
    for (final n in shown) {
      final loc = n['creationLocation'] as Map?;
      final path = projectPath(loc?['file'] as String?);
      if (loc == null || path == null) continue;
      // ignore: invalid_use_of_protected_member
      final node = svc.toObject(n['valueId'] as String, g);
      final elem = node is Element ? node : null;
      final head = '${_typeOf(n)}  ${path.split('/').last}:${loc['line']}';
      for (final r in refsAt(_read(path), loc['line'] as int, loc['column'] as int)) {
        // a number the widget uses as a measure; not a count, a callback's index or an alpha
        if ((r.kind == RefKind.raw || r.kind == RefKind.exception) && !_measure.hasMatch(r.arg)) continue;
        final tk = _tokens[r.text];
        switch (r.kind) {
          case RefKind.token when tk != null && tk.sizeToken == null:
            next.add(
              _Row(
                node: head,
                arg: r.arg,
                text: r.text,
                badge: tk.isConst ? _Badge.constToken : _Badge.token,
                value: tk.value,
                ref: r,
                token: tk,
                usage: _usage(r.text),
                element: elem,
                file: tk.file,
                line: tk.line,
              ),
            );
          case RefKind.token when tk != null:
            final size = _tokens[tk.sizeToken];
            next.add(
              _Row(
                node: head,
                arg: r.arg,
                text: '${r.text} → ${tk.sizeToken}',
                badge: _Badge.token,
                value: size?.value,
                ref: r,
                token: size,
                usage: size == null ? null : _usage(size.name),
                element: elem,
                file: size?.file,
                line: size?.line ?? 0,
              ),
            );
          case RefKind.token || RefKind.named:
            next.add(_Row(node: head, arg: r.arg, text: r.text, badge: _Badge.named, ref: r, element: elem));
          case RefKind.exception:
            next.add(
              _Row(
                node: head,
                arg: r.arg,
                text: r.text,
                badge: _Badge.exception,
                value: r.value,
                ref: r,
                element: elem,
                file: path,
                line: loc['line'] as int,
              ),
            );
          case RefKind.raw:
            next.add(
              _Row(
                node: head,
                arg: r.arg,
                text: r.text,
                badge: _Badge.raw,
                value: r.value,
                ref: r,
                element: elem,
                file: path,
                line: loc['line'] as int,
              ),
            );
        }
      }
      if (n == sel) next.addAll(_derived(elem, head));
    }
    // colours and styles are not sizes: one line per widget, after its sizes
    final merged = <_Row>[];
    for (final node in next.map((r) => r.node).toSet()) {
      final mine = next.where((r) => r.node == node).toList();
      merged.addAll(mine.where((r) => r.badge != _Badge.named));
      final named = mine.where((r) => r.badge == _Badge.named).map((r) => r.text).toSet();
      if (named.isNotEmpty)
        merged.add(_Row(node: node, arg: 'also', text: named.join(' '), badge: _Badge.named, element: mine.first.element));
    }
    next
      ..clear()
      ..addAll(merged);
    for (final r in next) {
      final f = r.file;
      if (f != null && session.start.containsKey(f) && r.line > 0) {
        final a = session.start[f]!.split('\n'), b = _read(f).split('\n');
        r.edited = r.line <= a.length && r.line <= b.length && a[r.line - 1] != b[r.line - 1];
      }
    }
    // the same row stays active across a rebuild
    final was = active < rows.length ? rows[active] : null;
    rows = next;
    active = was == null
        ? _firstEditable()
        : math.max(0, rows.indexWhere((r) => r.text == was.text && r.arg == was.arg && r.node == was.node && r.line == was.line));
    if (active >= rows.length) active = 0;
    if (was == null || rows.isEmpty || rows[active].text != was.text) active = _firstEditable();
    title = shown.isEmpty
        ? ''
        : '${_typeOf(sel)}  ${projectPath((sel['creationLocation'] as Map?)?['file'] as String?)}:${(sel['creationLocation'] as Map?)?['line']}';
    notifyListeners();
  }

  int _firstEditable() {
    final i = rows.indexWhere((r) => r.editable);
    return i < 0 ? 0 : i;
  }

  static String _typeOf(Map n) => ('${n['description']}').split(RegExp(r'[-<(\s]')).first;

  /// What Flutter decided: the size, the constraints it was given, and the flex that made it (not editable: the cause is).
  List<_Row> _derived(Element? el, String node) {
    final ro = el?.renderObject;
    final out = <_Row>[];
    if (ro is RenderBox && ro.hasSize) {
      out.add(_Row(node: node, arg: 'size', text: '${fmt(ro.size.width)} × ${fmt(ro.size.height)}', badge: _Badge.derived, element: el));
      try {
        out.add(
          _Row(
            node: node,
            arg: 'constraints',
            text: '${ro.constraints}'.replaceFirst('BoxConstraints', ''),
            badge: _Badge.derived,
            element: el,
          ),
        );
      } catch (_) {
        // a render object whose constraints are not box constraints: nothing to show
      }
    }
    RenderObject? at = ro;
    while (at != null) {
      final pd = at.parentData;
      final own = pd is FlexParentData ? pd.flex ?? 0 : 0;
      if (own > 0 && at.parent is RenderFlex) {
        final flex = at.parent! as RenderFlex;
        final others = <int>[];
        flex.visitChildren((c) {
          final f = c.parentData is FlexParentData ? (c.parentData! as FlexParentData).flex ?? 0 : 0;
          others.add(f);
        });
        out.add(
          _Row(
            node: node,
            arg: 'width',
            text: 'flex $own of ${others.join(' : ')} in ${flex.direction == Axis.horizontal ? 'Row' : 'Column'}',
            badge: _Badge.derived,
            element: el,
          ),
        );
        break;
      }
      at = at.parent;
    }
    return out;
  }

  final Map<String, String> _usageCache = {};

  String _usage(String name) => _usageCache.putIfAbsent(name, () {
    final re = RegExp('\\b${RegExp.escape(name)}\\b');
    var n = 0;
    for (final f in Directory('$_root/lib').listSync(recursive: true).whereType<File>().where((f) => f.path.endsWith('.dart'))) {
      n += re.allMatches(f.readAsStringSync()).length;
    }
    return '$n×';
  });

  // ------------------------------------------------------------------------------------------------------------ editing

  void activate(int i) {
    if (i < 0 || i >= rows.length) return;
    active = i;
    _historyAtActivation = session.history.length;
    notifyListeners();
  }

  int _nextEditable(int dir) {
    for (var k = 1; k <= rows.length; k++) {
      final i = (active + dir * k) % rows.length;
      if (rows[(i + rows.length) % rows.length].editable) return (i + rows.length) % rows.length;
    }
    return active;
  }

  void step(int dir) {
    final shift = HardwareKeyboard.instance.isShiftPressed, alt = HardwareKeyboard.instance.isAltPressed;
    adjust(dir * (shift ? 0.1 : (alt ? 5.0 : 1.0)));
  }

  void adjust(double delta) {
    if (rows.isEmpty || active >= rows.length) return;
    final r = rows[active];
    if (!r.editable || r.value == null || session.showingBefore) {
      note = session.showingBefore
          ? 'BEFORE is shown: press A to edit'
          : (r.badge == _Badge.constToken ? 'const: needs a hot restart, not editable here' : 'derived: change what causes it');
      notifyListeners();
      return;
    }
    final v = math.max(0.0, double.parse((r.value! + delta).toStringAsFixed(3)));
    _apply(r, v);
  }

  void _apply(_Row r, double v) {
    final file = r.file!;
    final text = _read(file);
    String? after;
    if (r.token != null) {
      after = rewriteToken(text, r.token!, v);
    } else if (r.ref != null && r.ref!.start >= 0) {
      final ref = r.ref!;
      if (ref.start > text.length || text.substring(ref.start, ref.end) != fmt(ref.value ?? 0)) {
        note = 'the source moved: click the widget again';
        notifyListeners();
        return;
      }
      after = text.replaceRange(ref.start, ref.end, fmt(v));
      final shift = fmt(v).length - (ref.end - ref.start);
      ref.end += shift;
    }
    if (after == null) {
      note = 'cannot write this value';
      notifyListeners();
      return;
    }
    final changed = session.apply(file, after, '${r.text} ${fmt(r.value!)} → ${fmt(v)}');
    if (changed.isEmpty) return;
    r.value = v;
    r.edited = true;
    _opsSinceAck++;
    _after(changed);
  }

  void resetActive() {
    if (rows.isEmpty) return;
    final r = rows[active];
    if (r.file == null || r.line == 0) return;
    _after(session.resetLine(r.file!, r.line));
    note = 'reset';
    refresh();
  }

  void cancelActive() {
    var n = session.history.length - _historyAtActivation;
    while (n-- > 0) {
      _after(session.undo());
    }
    refresh();
  }

  void undo() {
    _after(session.undo());
    refresh();
  }

  void redo() {
    _after(session.redo());
    refresh();
  }

  void ab() {
    final changed = session.toggleBefore();
    note = session.showingBefore ? 'BEFORE' : null;
    _after(changed);
  }

  // ------------------------------------------------------------------------------------------------------------ hot reload

  void _after(List<String> files) {
    if (files.isEmpty) return;
    _reloadTimer?.cancel();
    _reloadTimer = Timer(const Duration(milliseconds: 140), _sendReload);
    notifyListeners();
  }

  File get _pidFile =>
      File('${Platform.environment['XDG_STATE_HOME'] ?? '${Platform.environment['HOME']}/.local/state'}/motolii-stage5/flutter.pid');

  void _sendReload() {
    if (!_pidFile.existsSync()) {
      note = 'no dev session pid: written, press r in the flutter terminal';
      notifyListeners();
      return;
    }
    Process.runSync('kill', ['-USR1', _pidFile.readAsStringSync().trim()]);
    note = 'reloading…';
    _ackTimer?.cancel();
    _ackTimer = Timer(const Duration(seconds: 6), _noAck);
    notifyListeners();
  }

  /// A hot reload landed (the framework reassembled): what is on disk is valid.
  void reloaded() {
    _ackTimer?.cancel();
    _opsSinceAck = 0;
    note = session.showingBefore ? 'BEFORE' : null;
    if (on) Timer(const Duration(milliseconds: 60), refresh);
  }

  /// The reload did not land (a compile error): put the last edits back and reload again, so one wrong input cannot end the session.
  void _noAck() {
    var n = _opsSinceAck;
    _opsSinceAck = 0;
    if (n > 0 && !session.showingBefore) {
      while (n-- > 0 && session.canUndo) {
        session.undo();
      }
      note = 'reload failed (see the terminal): last change put back';
      _sendReloadQuiet();
      refresh();
    } else {
      note = 'reload did not answer: see the terminal';
    }
    notifyListeners();
  }

  void _sendReloadQuiet() {
    if (_pidFile.existsSync()) Process.runSync('kill', ['-USR1', _pidFile.readAsStringSync().trim()]);
  }

  // ------------------------------------------------------------------------------------------------------------ input

  /// Before the app's own focus tree sees a key: while Design Mode is on, the keys it uses are its own and go no further.
  KeyEventResult _early(KeyEvent e) => _key(e) ? KeyEventResult.handled : KeyEventResult.ignored;

  bool _key(KeyEvent e) {
    if (e is KeyUpEvent) return false;
    final k = e.logicalKey;
    final kb = HardwareKeyboard.instance;
    if (k == LogicalKeyboardKey.f2 || (k == LogicalKeyboardKey.keyD && kb.isMetaPressed && kb.isAltPressed)) {
      toggle();
      return true;
    }
    if (!on) return false;
    if (exiting) {
      if (k == LogicalKeyboardKey.enter) keep();
      if (k == LogicalKeyboardKey.escape) discard();
      return true;
    }
    // Cmd/Ctrl+Z as everywhere; plain Z too, one hand on the mouse and one on Z
    if (k == LogicalKeyboardKey.keyZ) {
      kb.isShiftPressed ? redo() : undo();
      return true;
    }
    if (kb.isMetaPressed || kb.isControlPressed) return false;
    if (k == LogicalKeyboardKey.arrowUp) return _then(() => step(1));
    if (k == LogicalKeyboardKey.arrowDown) return _then(() => step(-1));
    if (k == LogicalKeyboardKey.tab) {
      return _then(() => activate(_nextEditable(kb.isShiftPressed ? -1 : 1)));
    }
    if (k == LogicalKeyboardKey.keyR && e is KeyDownEvent) return _then(resetActive);
    if (k == LogicalKeyboardKey.keyA && e is KeyDownEvent) return _then(ab);
    if (k == LogicalKeyboardKey.keyC && e is KeyDownEvent) {
      return _then(toggleChanges);
    }
    if (k == LogicalKeyboardKey.escape && e is KeyDownEvent) return _then(cancelActive);
    if ((k == LogicalKeyboardKey.bracketLeft || k == LogicalKeyboardKey.bracketRight) && e is KeyDownEvent) {
      return _then(() => scale(k == LogicalKeyboardKey.bracketRight ? 1 : -1, kb.isShiftPressed ? 10 : 1));
    }
    return false;
  }

  bool _then(VoidCallback f) {
    f();
    return true;
  }

  void scale(int dir, [int by = 1]) {
    LiveUiScale.instance.set(UiScale.percent + dir * by, keep: false);
    notifyListeners();
  }

  void _pointer(PointerEvent e) {
    if (!on || exiting || e is! PointerScrollEvent) return;
    _wheel += e.scrollDelta.dy;
    final steps = (_wheel / 20).truncate();
    if (steps != 0) {
      _wheel -= steps * 20;
      step(-steps.sign * math.min(steps.abs(), 3));
    }
  }
}

// ---------------------------------------------------------------------------------------------------------------- what is drawn

class _BoundsPainter extends CustomPainter {
  _BoundsPainter(this.c) : super(repaint: c);
  final DesignController c;

  @override
  void paint(Canvas canvas, Size size) {
    if (c.rows.isEmpty || c.active >= c.rows.length) return;
    final r = c.rows[c.active];
    final ro = r.element?.renderObject;
    if (r.element == null || r.element!.debugIsDefunct || ro is! RenderBox || !ro.attached || !ro.hasSize) return;
    final box = ro.localToGlobal(Offset.zero) & ro.size;
    if (ro is RenderPadding && r.arg == 'padding') {
      final p = ro.padding.resolve(TextDirection.ltr);
      final inner = Rect.fromLTRB(box.left + p.left, box.top + p.top, box.right - p.right, box.bottom - p.bottom);
      canvas.drawPath(
        Path.combine(PathOperation.difference, Path()..addRect(box), Path()..addRect(inner)),
        Paint()..color = const Color(0x66FFB347),
      );
      canvas.drawRect(
        inner,
        Paint()
          ..style = PaintingStyle.stroke
          ..color = const Color(0xFF6FC3FF),
      );
    }
    canvas.drawRect(
      box,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.5
        ..color = const Color(0xFFFFB347),
    );
  }

  @override
  bool shouldRepaint(_BoundsPainter old) => true;
}

class _Hud extends StatelessWidget {
  const _Hud(this.c);
  final DesignController c;

  static const _bg = Color(0xF2101114),
      _ink = Color(0xFFE6E6E6),
      _dim = Color(0xFF8A8D93),
      _hot = Color(0xFFFFB347),
      _row = Color(0xFF23252B);

  TextStyle _t([Color col = _ink, double size = 11, FontWeight w = FontWeight.w400]) =>
      TextStyle(fontFamily: 'Menlo', fontSize: size, color: col, fontWeight: w, height: 1.25, decoration: TextDecoration.none);

  String _badge(_Row r) => switch (r.badge) {
    _Badge.token => 'TOKEN ${r.usage ?? ''}',
    _Badge.constToken => 'CONST',
    _Badge.exception => 'EXCEPTION',
    _Badge.raw => 'RAW',
    _Badge.derived => 'DERIVED',
    _Badge.named => 'NAMED',
  };

  @override
  Widget build(BuildContext context) {
    final size = MediaQuery.sizeOf(context);
    final ro = c.rows.isEmpty ? null : c.rows[c.active.clamp(0, c.rows.length - 1)].element?.renderObject;
    final at = ro is RenderBox && ro.attached && ro.hasSize
        ? ro.localToGlobal(ro.size.center(Offset.zero))
        : Offset(size.width, size.height);
    final hudLeft = at.dx > size.width / 2, hudTop = at.dy > size.height / 2; // the HUD stays out of the way of what is selected
    final children = <Widget>[];
    if (c.exiting) {
      children.addAll([
        Text('${c.session.changes().length} changed lines', style: _t(_hot, 12, FontWeight.w600)),
        Text('Enter KEEP: the changes stay in the source', style: _t()),
        Text('Esc   DISCARD: back to the session start', style: _t()),
        Row(children: [_Chip('KEEP', c.keep, _t), _Chip('DISCARD', c.discard, _t), _Chip('CONTINUE', c.toggleOnly, _t)]),
      ]);
    } else if (c.showChanges) {
      children.add(Text('SESSION CHANGES', style: _t(_hot, 11, FontWeight.w600)));
      final ch = c.session.changes();
      if (ch.isEmpty) children.add(Text('nothing changed', style: _t(_dim)));
      for (final (f, l, a, b) in ch) {
        children.add(Text('${f.split('/').last}:$l', style: _t(_dim)));
        children.add(Text('  $a', style: _t(const Color(0xFFD97B7B))));
        children.add(Text('  $b', style: _t(const Color(0xFF7BD98F))));
      }
    } else {
      children.add(
        Text(
          c.title.isEmpty ? 'Design Mode: click a thing' : c.title,
          style: _t(_hot, 11, FontWeight.w600),
          maxLines: 1,
          overflow: TextOverflow.ellipsis,
        ),
      );
      String? node;
      for (var i = 0; i < c.rows.length; i++) {
        final r = c.rows[i];
        if (r.node != node) {
          node = r.node;
          if (i > 0) children.add(Text(r.node, style: _t(_dim, 10), maxLines: 1, overflow: TextOverflow.ellipsis));
        }
        children.add(_RowView(c, i, r, _t, _badge(r)));
      }
    }
    return Positioned(
      left: hudLeft ? 8 : null,
      right: hudLeft ? null : 8,
      top: hudTop ? 8 : null,
      bottom: hudTop ? null : 8,
      width: 380,
      child: Container(
        padding: const EdgeInsets.all(8),
        decoration: BoxDecoration(
          color: _bg,
          borderRadius: BorderRadius.circular(6),
          border: Border.all(color: const Color(0xFF3A3D45)),
        ),
        child: DefaultTextStyle(
          style: _t(),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  Text(
                    c.session.showingBefore ? 'BEFORE' : 'CURRENT',
                    style: _t(c.session.showingBefore ? const Color(0xFFD97B7B) : const Color(0xFF7BD98F), 11, FontWeight.w600),
                  ),
                  const Spacer(),
                  Text('UI ${UiScale.percent}%', style: _t(_dim)),
                ],
              ),
              const SizedBox(height: 4),
              ...children,
              if (c.note != null)
                Padding(
                  padding: const EdgeInsets.only(top: 4),
                  child: Text(c.note!, style: _t(_hot)),
                ),
              const SizedBox(height: 4),
              Row(
                children: [
                  _Chip('A/B', c.ab, _t),
                  _Chip('UNDO', c.undo, _t),
                  _Chip('REDO', c.redo, _t),
                  _Chip('RESET', c.resetActive, _t),
                  _Chip('CHANGES', c.toggleChanges, _t),
                  _Chip('DONE', c.toggle, _t),
                ],
              ),
              Text('Tab row · wheel/↑↓ ±1 (⇧.1 ⌥5) · A before · R reset · Esc cancel · ⌘Z · C changes · [ ] scale', style: _t(_dim, 9)),
            ],
          ),
        ),
      ),
    );
  }
}

class _Chip extends StatelessWidget {
  const _Chip(this.label, this.onTap, this.t);
  final String label;
  final VoidCallback onTap;
  final TextStyle Function([Color, double, FontWeight]) t;

  @override
  Widget build(BuildContext context) => GestureDetector(
    behavior: HitTestBehavior.opaque,
    onTap: onTap,
    child: Container(
      margin: const EdgeInsets.only(right: 4, top: 4),
      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
      decoration: BoxDecoration(color: _Hud._row, borderRadius: BorderRadius.circular(3)),
      child: Text(label, style: t(_Hud._ink, 9, FontWeight.w600)),
    ),
  );
}

class _RowView extends StatelessWidget {
  const _RowView(this.c, this.i, this.r, this.t, this.badge);
  final DesignController c;
  final int i;
  final _Row r;
  final TextStyle Function([Color, double, FontWeight]) t;
  final String badge;

  @override
  Widget build(BuildContext context) {
    final on = i == c.active;
    final value = r.value;
    return GestureDetector(
      behavior: HitTestBehavior.opaque,
      onTap: () => c.activate(i),
      onHorizontalDragStart: (_) => c.activate(i),
      onHorizontalDragUpdate: (d) {
        if (r.editable) c.adjust(d.delta.dx.sign * (HardwareKeyboard.instance.isShiftPressed ? 0.1 : 1.0));
      },
      child: Container(
        color: on ? _Hud._row : null,
        padding: const EdgeInsets.symmetric(vertical: 1, horizontal: 2),
        child: Row(
          children: [
            SizedBox(width: 12, child: Text(r.edited ? '●' : (on ? '▸' : ''), style: t(_Hud._hot))),
            SizedBox(
              width: 74,
              child: Text(r.arg, style: t(_Hud._dim), maxLines: 1, overflow: TextOverflow.clip),
            ),
            Expanded(
              child: Text(r.text, style: t(r.editable ? _Hud._ink : _Hud._dim), maxLines: 1, overflow: TextOverflow.ellipsis),
            ),
            if (value != null)
              SizedBox(
                width: 46,
                child: Text(fmt(value), textAlign: TextAlign.right, style: t(on ? _Hud._hot : _Hud._ink, 11, FontWeight.w600)),
              ),
            SizedBox(
              width: 80,
              child: Text(badge, textAlign: TextAlign.right, style: t(_Hud._dim, 9), maxLines: 1),
            ),
          ],
        ),
      ),
    );
  }
}
