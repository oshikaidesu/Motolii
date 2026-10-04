// surface-file: Design Mode is debug-only tool chrome, not product UI; its sizes are its own and it is never in a release build
// Design Mode: in the running debug window, click a thing, see what decides how it looks (its numbers, switches, named choices, colours and
// the token each comes from), change them with the wheel, the arrow keys, a click on a choice or typing, and see it at once. Flutter's own
// Inspector does the clicking and the widget's creation location (WidgetInspectorService, the service DevTools uses); design_props.dart
// finds each property in the source; this is the HUD and the keys. A change is written into the source and the existing hot reload
// (`scripts/motolii-ui.sh design` keeps its pid) puts it on screen. See docs/design/ui-debugging.md.
//
//   F2 / Cmd+Option+D  on / off     Tab  next value     wheel, Up / Down  change (numbers ±1, Shift 0.1, Alt 5; choices step)
//   Enter  type a value / open the choices     T  this place only <-> the token     A  BEFORE <-> CURRENT     R  value back
//   Z / Cmd+Z, Shift+Z  undo, redo     Esc  cancel     C  changes     [ ]  UI Scale
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
import 'design_props.dart';

const _startOn = bool.fromEnvironment('MOTOLII_DESIGN');

/// Wraps the app in debug builds: the HUD above it, the keys and the wheel while Design Mode is on.
class DesignMode extends StatefulWidget {
  const DesignMode({super.key, required this.child, this.controller});
  final Widget child;

  /// A session's controller made by a test (its own source root and state directory); the app makes its own.
  final DesignController? controller;

  @override
  State<DesignMode> createState() => _DesignModeState();
}

class _DesignModeState extends State<DesignMode> {
  late final c = widget.controller ?? DesignController();

  @override
  void initState() {
    super.initState();
    c.addListener(() => setState(() {}));
    if (c.restore() || _startOn) WidgetsBinding.instance.addPostFrameCallback((_) => c.toggle());
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
              _Grip(c),
              _Hud(c),
            ],
          ),
        ),
    ],
  );
}

/// A widget the click led to: where its call is written, and its element (for the box drawn in the window).
class _Node {
  _Node(this.widget, this.file, this.line, this.col, this.element);
  final String widget, file;
  int line, col;
  final Element? element;
  String get head => '$widget  ${file.split('/').last}:$line';
}

/// What identifies the selected element across a remount, from what Flutter already knows: its widget type, the project widgets it sits in
/// (root first; the inspector's own notion of "local", not the framework's internals) and which of the elements with that identity, in
/// tree order, it is.
class _Anchor {
  _Anchor(this.type, this.sig, this.ordinal, this.file);
  final String type, file;
  final List<String> sig;
  final int ordinal;

  Map<String, Object?> toJson() => {'type': type, 'sig': sig, 'ordinal': ordinal, 'file': file};
  static _Anchor fromJson(Map<String, Object?> j) =>
      _Anchor(j['type'] as String, (j['sig'] as List).cast<String>(), j['ordinal'] as int, j['file'] as String);
}

/// One line of the HUD: a property of a node, or something Flutter decided (derived).
class _Row {
  _Row(this.node, {this.prop, this.derivedName, this.derivedText, this.scope = Scope.instance, this.usage});
  final _Node node;
  final Prop? prop;
  final String? derivedName, derivedText;
  Scope scope;
  final String? usage;
  bool edited = false;

  /// The line of the property in its file as it was read (an edit does not change a file's line count).
  int line = 0;
  bool get editable => prop != null && prop!.editable;
  String get name => prop?.name ?? derivedName ?? '';
  String get key => '${node.head}|${prop?.name ?? derivedName}|${prop?.condition ?? ''}';
}

/// The element's render box, when it still has one: a hot reload or an edit can unmount what was clicked, and asking an unmounted element
/// for its render object is an error.
RenderBox? boxOf(Element? e) {
  if (e == null || !e.mounted) return null;
  try {
    final ro = e.renderObject;
    return ro is RenderBox && ro.attached && ro.hasSize ? ro : null;
  } catch (_) {
    return null;
  }
}

class DesignController extends ChangeNotifier {
  DesignController({String root = '', Directory? state}) : _root = root, _state = state {
    session = DesignSession(_read, _write);
    FocusManager.instance.addEarlyKeyEventHandler(_early);
    GestureBinding.instance.pointerRouter.addGlobalRoute(_pointer);
    WidgetInspectorService.instance.selection.addListener(_selected);
  }

  late final DesignSession session;
  bool on = false, exiting = false, showChanges = false, typing = false, choosing = false;
  List<_Row> rows = [];
  List<_Node> _nodes = [];
  int active = 0;
  String? note;
  String title = '';
  String _root;
  final Directory? _state;
  Map<String, String> _colors = {};
  List<String> _libFiles = [];
  int _opsSinceAck = 0, _historyAtActivation = 0;
  Timer? _reloadTimer, _ackTimer;
  double _wheel = 0;
  final input = TextEditingController();
  final inputFocus = FocusNode();
  Rect hudRect = Rect.zero;

  @override
  void dispose() {
    FocusManager.instance.removeEarlyKeyEventHandler(_early);
    GestureBinding.instance.pointerRouter.removeGlobalRoute(_pointer);
    WidgetInspectorService.instance.selection.removeListener(_selected);
    _reloadTimer?.cancel();
    _ackTimer?.cancel();
    input.dispose();
    inputFocus.dispose();
    super.dispose();
  }

  // ------------------------------------------------------------------------------------------------------------ files

  String _read(String rel) => File('$_root/$rel').readAsStringSync();
  void _write(String rel, String text) => File('$_root/$rel').writeAsStringSync(text);

  // ------------------------------------------------------------------------------------------------------------ mode

  void toggle() {
    if (on && exiting) {
      exiting = false; // the question was keep or discard: the toggle again is neither, back to work
      notifyListeners();
      return;
    }
    if (on && session.changed) {
      exiting = true; // keep or discard first
      notifyListeners();
      return;
    }
    exiting = false;
    typing = false;
    choosing = false;
    on = !on;
    WidgetsBinding.instance.debugShowWidgetInspectorOverride = on;
    if (on) {
      _selected();
      if (_anchor != null) Timer(const Duration(milliseconds: 300), _reselect);
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
    if (session.showingBefore) _afterWrite(session.toggleBefore());
    session.keep();
    exiting = false;
    toggle();
  }

  void discard() {
    _afterWrite(session.discard());
    exiting = false;
    note = null;
    toggle();
  }

  // ------------------------------------------------------------------------------------------------------------ selection

  bool _keepNext = false;

  void _selected() {
    if (!on) return;
    scheduleMicrotask(() {
      final keep = _keepNext;
      _keepNext = false;
      select(keep: keep);
    });
  }

  // ------------------------------------------------------------------------------------------ the selection survives a remount

  _Anchor? _anchor;

  static List<String> _localChain(Element e) {
    final out = <String>[];
    e.visitAncestorElements((a) {
      if (debugIsLocalCreationLocation(a.widget)) out.add(a.widget.runtimeType.toString());
      return true;
    });
    return out.reversed.toList();
  }

  static bool _same(List<String> a, List<String> b) {
    if (a.length != b.length) return false;
    for (var i = 0; i < a.length; i++) {
      if (a[i] != b[i]) return false;
    }
    return true;
  }

  /// The elements of the live tree with [type] and [sig], in tree order.
  static List<Element> _likes(String type, List<String> sig) {
    final root = WidgetsBinding.instance.rootElement;
    final out = <Element>[];
    if (root == null) return out;
    final stack = <Element>[root];
    while (stack.isNotEmpty) {
      final e = stack.removeLast();
      if (e.widget.runtimeType.toString() == type && _same(_localChain(e), sig)) out.add(e);
      final kids = <Element>[];
      e.visitChildren(kids.add);
      stack.addAll(kids.reversed);
    }
    return out;
  }

  void _remember(Element e, String file) {
    final type = e.widget.runtimeType.toString(), sig = _localChain(e);
    final i = _likes(type, sig).indexOf(e);
    if (i >= 0) _anchor = _Anchor(type, sig, i, file);
  }

  /// After a hot reload (or restart): the same selection when its element lives on, else the element that is the same one now.
  void _reselect([int tries = 0]) {
    if (!on) return;
    final svc = WidgetInspectorService.instance;
    if (svc.selection.currentElement != null) {
      select(keep: true);
      return;
    }
    final a = _anchor;
    if (a == null) return;
    final found = _likes(a.type, a.sig);
    if (found.isNotEmpty) {
      _keepNext = true;
      if (_restoredActive >= 0) active = _restoredActive;
      _restoredActive = -1;
      svc.selection.currentElement = found[a.ordinal < found.length ? a.ordinal : found.length - 1];
      return;
    }
    if (tries < 10) Timer(const Duration(milliseconds: 200), () => _reselect(tries + 1));
  }

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
    'Opacity',
    'ColoredBox',
    'Wrap',
    'ListView',
    'TextSpan',
    'Icon',
    'Image',
    'SingleChildScrollView',
    'GridView',
    'FittedBox',
    'AspectRatio',
    'FractionallySizedBox',
    'OverflowBox',
    'LimitedBox',
    'IntrinsicWidth',
    'IntrinsicHeight',
    'ClipRect',
    'Transform',
    'AnimatedContainer',
    'AnimatedOpacity',
    'AnimatedPositioned',
    'Baseline',
    'Scrollbar',
  };

  /// What Flutter's Inspector selected: the clicked widget and the nearest ancestors that set geometry, each as a place in the source.
  void select({bool keep = false}) {
    final svc = WidgetInspectorService.instance;
    if (svc.selection.currentElement == null) {
      _nodes = [];
      rows = [];
      title = '';
      notifyListeners();
      return;
    }
    const g = 'design';
    // The inspector's object-group calls are @protected for tools that subclass the service; this is the tool, it calls the singleton.
    // ignore: invalid_use_of_protected_member
    svc.disposeGroup(g);
    final sel = jsonDecode(svc.getSelectedSummaryWidget(null, g));
    if (sel is! Map) return;
    // ignore: invalid_use_of_protected_member
    final chainJson = svc.getParentChain(sel['valueId'] as String, g);
    final chain = (jsonDecode(chainJson) as List).map((e) => (e as Map)['node'] as Map).toList().reversed.toList();
    final shown = <Map>[
      sel,
      ...chain
          .skip(1)
          .where((n) => projectPath((n['creationLocation'] as Map?)?['file'] as String?) != null && _geometry.contains(_typeOf(n)))
          .take(12),
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
    _nodes = [];
    for (final n in shown) {
      final loc = n['creationLocation'] as Map?;
      final path = projectPath(loc?['file'] as String?);
      if (loc == null || path == null) continue;
      // ignore: invalid_use_of_protected_member
      final obj = svc.toObject(n['valueId'] as String, g);
      _nodes.add(_Node(_typeOf(n), path, loc['line'] as int, loc['column'] as int, obj is Element ? obj : null));
    }
    title = _nodes.isEmpty ? '' : _nodes.first.head;
    final picked = svc.selection.currentElement;
    if (picked != null && _nodes.isNotEmpty) _remember(picked, _nodes.first.file);
    if (!keep) active = -1;
    _reanalyze();
  }

  static String _typeOf(Map n) => ('${n['description']}').split(RegExp(r'[-<(\s]')).first;

  // ------------------------------------------------------------------------------------------------------------ properties

  Analyzer? _an;

  Analyzer _analyzer() {
    if (_libFiles.isEmpty) {
      _libFiles = [
        for (final f in Directory('$_root/lib').listSync(recursive: true).whereType<File>())
          if (f.path.endsWith('.dart') && !f.path.contains('/lib/dev/')) f.path.substring(_root.length + 1),
      ];
    }
    if (_colors.isEmpty) {
      _colors = parseColorTokens(
        neutral: _read('lib/theme/neutral.dart'),
        identity: _read('lib/theme/identity.dart'),
        metrics: _read('lib/theme/metrics.dart'),
        others: _libFiles.map(_read),
      );
    }
    return _an = Analyzer(
      _read,
      Registry(tokens: parseTokens(_read('lib/theme/metrics.dart')), colors: _colors),
      files: () => _libFiles,
    );
  }

  /// The properties of every node, read again from the source as it is now (after an edit the columns of a call on the same line move).
  void _reanalyze() {
    if (_root.isEmpty) return;
    final an = _analyzer();
    final was = active >= 0 && active < rows.length ? rows[active] : null;
    final next = <_Row>[];
    var shownNodes = 0;
    for (final n in _nodes) {
      _relocate(an, n);
      List<Prop> props;
      try {
        props = an.analyze(n.file, n.line, n.col);
      } catch (_) {
        props = const [];
      }
      // the clicked widget always; of its ancestors those that say something about how it looks, the nearest few
      if (n != _nodes.first && (props.isEmpty || shownNodes >= 9)) continue;
      shownNodes++;
      for (final p in props) {
        final tk = p.token;
        next.add(
          _Row(
            n,
            prop: p,
            scope: p.isToken && p.kind == PropKind.number ? Scope.token : Scope.instance,
            usage: tk == null ? null : _usage(tk),
          ),
        );
      }
      if (n == _nodes.first) next.addAll(_derived(n));
    }
    for (final r in next) {
      final p = r.prop;
      if (p != null) r.line = an.src(p.file).lineOf(p.start);
    }
    // edited: it differs from what the same call said when the session began
    if (session.start.isNotEmpty) {
      final an0 = _startAnalyzer();
      final before = <String, String>{};
      for (final n in _nodes) {
        for (final p in _startProps(n, an0)) {
          before['${n.head}|${p.name}|${p.condition ?? ''}'] = '${p.origin.name}:${p.text}:${p.token ?? ''}';
        }
      }
      for (final r in next) {
        final p = r.prop;
        if (p != null) r.edited = before[r.key] != '${p.origin.name}:${p.text}:${p.token ?? ''}';
      }
    }
    rows = next;
    if (was != null) {
      final i = rows.indexWhere((r) => r.key == was.key);
      if (i >= 0) active = i;
    }
    if (active < 0 || active >= rows.length || !rows[active].editable) active = _firstEditable();
    notifyListeners();
  }

  /// The product's source as it was when the session began (for what each value used to be).
  Analyzer _startAnalyzer() => Analyzer(
    (f) => session.start[f] ?? _read(f),
    Registry(tokens: parseTokens(session.start['lib/theme/metrics.dart'] ?? _read('lib/theme/metrics.dart')), colors: _colors),
    files: () => _libFiles,
  );

  /// The properties of [n]'s call as the session began: the call is found at the line it was at then.
  List<Prop> _startProps(_Node n, Analyzer an0) {
    final sl = session.startLine(n.file, n.line);
    if (sl == null) return const [];
    final then = _Node(n.widget, n.file, sl, n.col, null);
    try {
      _relocate(an0, then);
      return an0.analyze(n.file, sl, then.col);
    } catch (_) {
      return const [];
    }
  }

  /// A call's start moves when an edit earlier on its line changes the line's length: find it again by its name.
  void _relocate(Analyzer an, _Node n) {
    final s = an.src(n.file);
    final at = s.offsetOf(n.line, n.col);
    if (at < s.text.length && RegExp('^${RegExp.escape(n.widget)}\\b').hasMatch(s.text.substring(at))) return;
    final lineStart = s.offsetOf(n.line, 1);
    final lineEnd = s.text.indexOf('\n', lineStart);
    final line = s.text.substring(lineStart, lineEnd < 0 ? s.text.length : lineEnd);
    final found = RegExp('\\b${RegExp.escape(n.widget)}\\b').allMatches(line).map((m) => m.start).toList();
    if (found.isEmpty) return;
    found.sort((a, b) => (a - (n.col - 1)).abs().compareTo((b - (n.col - 1)).abs()));
    n.col = found.first + 1;
  }

  int _firstEditable() {
    final i = rows.indexWhere((r) => r.editable);
    return i < 0 ? 0 : i;
  }

  /// What Flutter decided: the size, the constraints it was given, and the flex that made it (not editable: the cause is).
  List<_Row> _derived(_Node n) {
    final ro = boxOf(n.element);
    final out = <_Row>[];
    if (ro is RenderBox && ro.hasSize) {
      out.add(_Row(n, derivedName: 'size', derivedText: '${fmt(ro.size.width)} × ${fmt(ro.size.height)}'));
      try {
        out.add(_Row(n, derivedName: 'constraints', derivedText: '${ro.constraints}'.replaceFirst('BoxConstraints', '')));
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
        flex.visitChildren((c) => others.add(c.parentData is FlexParentData ? (c.parentData! as FlexParentData).flex ?? 0 : 0));
        out.add(
          _Row(
            n,
            derivedName: 'width',
            derivedText:
                'flex $own of ${others.join(' : ')} in a ${flex.direction == Axis.horizontal ? 'Row' : 'Column'}: edit its Expanded / Flexible flex',
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
    for (final f in _libFiles) {
      n += re.allMatches(_read(f)).length;
    }
    return '$n×';
  });

  /// The colour an expression stands for, through the tokens: `Surface.base` → `N.g10` → `Color(0xFF191919)`.
  Color? colorOf(String expr) {
    var e = expr.trim();
    for (var i = 0; i < 4; i++) {
      final lit = RegExp(r'^(?:const\s+)?Color\(0x([0-9A-Fa-f]{8})\)').firstMatch(e);
      if (lit != null) return Color(int.parse(lit.group(1)!, radix: 16));
      final next = _colors[e];
      if (next == null) return null;
      e = next;
    }
    return null;
  }

  // ------------------------------------------------------------------------------------------------------------ editing

  _Row? get current => rows.isEmpty || active < 0 || active >= rows.length ? null : rows[active];

  void activate(int i) {
    if (i < 0 || i >= rows.length) return;
    active = i;
    typing = false;
    choosing = false;
    _historyAtActivation = session.history.length;
    notifyListeners();
  }

  int _nextEditable(int dir) {
    for (var k = 1; k <= rows.length; k++) {
      final i = ((active + dir * k) % rows.length + rows.length) % rows.length;
      if (rows[i].editable) return i;
    }
    return active;
  }

  /// The row's value one step further (dir = ±1): a number by its step, a choice to the next choice, a switch flipped, a colour to the
  /// next token (or a literal's lightness).
  void step(int dir) {
    final r = current;
    if (r == null) return;
    final kb = HardwareKeyboard.instance;
    final unit = kb.isShiftPressed ? 0.1 : (kb.isAltPressed ? 5.0 : 1.0);
    final p = r.prop;
    if (p == null || !p.editable) {
      note = p?.note ?? 'derived: change what causes it';
      notifyListeners();
      return;
    }
    if (session.showingBefore) {
      note = 'BEFORE is shown: press A to edit';
      notifyListeners();
      return;
    }
    switch (p.kind) {
      case PropKind.number:
        final base = p.number ?? 0;
        _set(r, fmt(math.max(0, double.parse((base + dir * unit).toStringAsFixed(3)))));
      case PropKind.boolean:
        _set(r, p.text == 'true' ? 'false' : 'true');
      case PropKind.option:
      case PropKind.color:
        final opts = p.options;
        if (opts.isEmpty) {
          // a literal colour: its lightness
          final c = colorOf(p.text);
          if (c != null) _set(r, _lighter(c, dir * (kb.isShiftPressed ? 0.005 : 0.02)));
          return;
        }
        final i = opts.indexOf(p.text);
        _set(r, opts[((i < 0 ? 0 : i) + dir + opts.length) % opts.length]);
      case PropKind.readOnly:
        break;
    }
  }

  static String _lighter(Color c, double by) {
    final h = HSLColor.fromColor(c);
    final l = HSLColor.fromAHSL(h.alpha, h.hue, h.saturation, (h.lightness + by).clamp(0.0, 1.0)).toColor();
    return 'Color(0x${l.toARGB32().toRadixString(16).toUpperCase().padLeft(8, '0')})';
  }

  /// Sets the row's property to [value] in the source.
  void _set(_Row r, String value) {
    final p = r.prop!;
    final an = _an ?? _analyzer();
    final ch = edit(an, p, value, scope: r.scope);
    if (ch == null) {
      note = 'cannot write this value here';
      notifyListeners();
      return;
    }
    final changed = <String>[];
    ch.forEach((file, text) => changed.addAll(session.apply(file, text, '${r.node.widget}.${r.name} ${p.text} → $value')));
    if (changed.isEmpty) return;
    _opsSinceAck++;
    _afterWrite(changed);
    _reanalyze();
  }

  /// A value typed or picked: a number as typed, a colour as `#RRGGBB` / `0xAARRGGBB`, a choice as it is.
  void setText(String text) {
    final r = current;
    if (r == null || r.prop == null) return;
    var v = text.trim();
    switch (r.prop!.kind) {
      case PropKind.number:
        if (double.tryParse(v) == null) {
          note = '“$v” is not a number';
          notifyListeners();
          return;
        }
      case PropKind.color:
        final h = RegExp(r'^(?:#|0x)?([0-9A-Fa-f]{6}|[0-9A-Fa-f]{8})$').firstMatch(v);
        if (h != null) {
          final d = h.group(1)!.toUpperCase();
          v = 'Color(0x${d.length == 6 ? 'FF$d' : d})';
        }
      default:
        break;
    }
    typing = false;
    choosing = false;
    _set(r, v);
  }

  /// Enter opens the list of choices when there is one; the TYPE chip always types (a colour as a hex literal).
  // ------------------------------------------------------------------------------------------------------- direct manipulation

  /// The grip on the selected box for the active number: the edge it moves (a width at the right edge, a padding at the inner edge of the
  /// padding area), and which way a drag along [axis] makes it larger. Null when the active row is not such a number.
  ({Offset at, Axis axis, double sign})? get grip {
    final r = current;
    final p = r?.prop;
    if (!on || exiting || typing || choosing || session.showingBefore || r == null || p == null || !p.editable || p.kind != PropKind.number)
      return null;
    final ro = boxOf(r.node.element);
    if (ro == null) return null;
    final box = ro.localToGlobal(Offset.zero) & ro.size;
    final f = UiScale.factor, n = (p.number ?? 0) * f;
    return switch (p.name) {
      'width' || 'dimension' || 'minWidth' || 'maxWidth' => (at: Offset(box.right, box.center.dy), axis: Axis.horizontal, sign: 1.0),
      'height' || 'minHeight' || 'maxHeight' => (at: Offset(box.center.dx, box.bottom), axis: Axis.vertical, sign: 1.0),
      'padding.left' ||
      'padding.horizontal' ||
      'margin.left' => (at: Offset(box.left + n, box.center.dy), axis: Axis.horizontal, sign: 1.0),
      'padding.right' || 'margin.right' => (at: Offset(box.right - n, box.center.dy), axis: Axis.horizontal, sign: -1.0),
      'padding.top' || 'padding.vertical' || 'margin.top' => (at: Offset(box.center.dx, box.top + n), axis: Axis.vertical, sign: 1.0),
      'padding.bottom' || 'margin.bottom' => (at: Offset(box.center.dx, box.bottom - n), axis: Axis.vertical, sign: -1.0),
      _ => null,
    };
  }

  /// How far the grip has been pulled so far (logical pixels along its axis), while a drag is on.
  double? dragged;
  double _dragStart = 0, _dragSign = 1;
  int _dragHistory = 0;

  void dragStart() {
    final g = grip, p = current?.prop;
    if (g == null || p == null) return;
    _dragStart = p.number ?? 0;
    _dragSign = g.sign;
    _dragHistory = session.history.length;
    dragged = 0;
    notifyListeners();
  }

  /// The grip moved by [delta] (along its axis): the number follows in steps of half a unit, written and hot-reloaded as it goes.
  void dragUpdate(double delta) {
    if (dragged == null) return;
    dragged = dragged! + delta;
    final r = current;
    if (r?.prop != null && r!.prop!.editable) {
      final v = math.max(0.0, ((_dragStart + _dragSign * dragged! / UiScale.factor) * 2).roundToDouble() / 2);
      if ((r.prop!.number ?? 0) != v) _set(r, fmt(v));
    }
    notifyListeners();
  }

  void dragEnd() {
    if (dragged == null) return;
    dragged = null;
    session.coalesce(_dragHistory);
    notifyListeners();
  }

  void startTyping({bool list = true}) {
    final r = current;
    if (r == null || !r.editable) return;
    final p = r.prop!;
    if (p.kind == PropKind.option || p.kind == PropKind.boolean || (list && p.kind == PropKind.color && p.options.isNotEmpty)) {
      choosing = !choosing;
      notifyListeners();
      return;
    }
    choosing = false;
    typing = true;
    final c = colorOf(p.text);
    input.text = p.kind == PropKind.color ? (c == null ? '' : '#${c.toARGB32().toRadixString(16).substring(2).toUpperCase()}') : p.text;
    input.selection = TextSelection(baseOffset: 0, extentOffset: input.text.length);
    notifyListeners();
    WidgetsBinding.instance.addPostFrameCallback((_) => inputFocus.requestFocus());
  }

  void stopTyping() {
    typing = false;
    notifyListeners();
  }

  /// This place only <-> the token itself (for a value that is a token).
  void toggleScope() {
    final r = current;
    if (r == null || r.prop == null || !r.prop!.isToken) return;
    r.scope = r.scope == Scope.token ? Scope.instance : Scope.token;
    notifyListeners();
  }

  /// The value goes back to what it was when the session began: a token to its first value, an edited expression to its first text, an
  /// argument this session added is taken out again.
  void resetActive() {
    final r = current;
    final p = r?.prop;
    if (r == null || p == null || session.showingBefore) return;
    final an = _an ?? _analyzer();
    final an0 = _startAnalyzer();
    Changes? ch;
    if (p.isToken && r.scope == Scope.token) {
      final t0 = an0.reg.tokens[p.token];
      if (t0 != null) ch = edit(an, p, fmt(t0.value), scope: Scope.token);
    } else {
      final p0 = _startProps(r.node, an0).where((q) => q.name == p.name && q.condition == p.condition).firstOrNull;
      if (p0 == null) {
        note = 'nothing to reset to';
      } else if (p0.origin == Origin.unset) {
        ch = p.origin == Origin.unset ? null : removeProp(an, p);
      } else if (p0.expr != p.expr) {
        ch = edit(an, p, p0.expr);
      }
    }
    if (ch == null) {
      note ??= 'as it was';
      notifyListeners();
      return;
    }
    final changed = <String>[];
    ch.forEach((file, text) => changed.addAll(session.apply(file, text, 'reset ${r.node.widget}.${r.name}')));
    note = 'reset';
    _opsSinceAck++;
    _afterWrite(changed);
    _reanalyze();
  }

  void cancelActive() {
    if (typing || choosing) {
      typing = false;
      choosing = false;
      notifyListeners();
      return;
    }
    var n = session.history.length - _historyAtActivation;
    final files = <String>[];
    while (n-- > 0) {
      files.addAll(session.undo());
    }
    _afterWrite(files);
    _reanalyze();
  }

  void undo() {
    _afterWrite(session.undo());
    _reanalyze();
  }

  void redo() {
    _afterWrite(session.redo());
    _reanalyze();
  }

  void ab() {
    final changed = session.toggleBefore();
    note = session.showingBefore ? 'BEFORE' : null;
    _afterWrite(changed);
    _reanalyze();
  }

  // ------------------------------------------------------------------------------------------------------------ hot reload

  void _afterWrite(List<String> files) {
    if (files.isEmpty) return;
    _an?.forget();
    // a changed `const` is compiled into its users: only a hot restart brings it to the screen
    for (final f in files) {
      final now = _read(f), was = _seen[f] ?? session.start[f] ?? now;
      _seen[f] = now;
      if (_changesConst(was, now)) _restart = true;
    }
    _reloadTimer?.cancel();
    _reloadTimer = Timer(const Duration(milliseconds: 140), _sendReload);
    notifyListeners();
  }

  final Map<String, String> _seen = {};
  bool _restart = false;

  static bool _changesConst(String a, String b) {
    final x = a.split('\n'), y = b.split('\n');
    if (x.length != y.length) return false;
    for (var i = 0; i < x.length; i++) {
      if (x[i] != y[i] && RegExp(r'\bconst\b').hasMatch(x[i]) && RegExp(r'^\s*(static\s+)?(final|const)\s').hasMatch(x[i])) return true;
    }
    return false;
  }

  File get _sessionFile => File('${_pidFile.parent.path}/design-session.json');

  /// After a hot restart: the session (history, BEFORE / CURRENT) comes back from the file written just before it.
  int _restoredActive = -1;

  bool restore() {
    try {
      final f = _sessionFile;
      if (!f.existsSync() || DateTime.now().difference(f.lastModifiedSync()).inSeconds > 120) return false;
      final j = jsonDecode(f.readAsStringSync()) as Map<String, Object?>;
      f.deleteSync();
      _root = j['root'] as String;
      session.restore((j['session'] as Map).cast<String, Object?>());
      if (j['anchor'] != null) _anchor = _Anchor.fromJson((j['anchor'] as Map).cast<String, Object?>());
      _restoredActive = j['active'] as int? ?? -1;
      note = 'restarted: a changed const needs it. Your session is back';
      return true;
    } catch (_) {
      return false;
    }
  }

  File get _pidFile => File(
    '${_state?.path ?? '${Platform.environment['XDG_STATE_HOME'] ?? '${Platform.environment['HOME']}/.local/state'}/motolii-stage5'}/flutter.pid',
  );

  void _sendReload() {
    if (!_pidFile.existsSync()) {
      note = 'no dev session pid: written, press r in the flutter terminal';
      notifyListeners();
      return;
    }
    if (_restart) {
      _restart = false;
      _sessionFile.writeAsStringSync(
        jsonEncode({'root': _root, 'session': session.toJson(), 'anchor': _anchor?.toJson(), 'active': active}),
      );
      Process.runSync('kill', ['-USR2', _pidFile.readAsStringSync().trim()]);
      note = 'hot restart (a const changed)…';
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
    if (on) Timer(const Duration(milliseconds: 60), _reselect);
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
      _an?.forget();
      if (_pidFile.existsSync()) Process.runSync('kill', ['-USR1', _pidFile.readAsStringSync().trim()]);
      _reanalyze();
    } else {
      note = 'reload did not answer: see the terminal';
    }
    notifyListeners();
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
    if (typing) {
      // the input field has the keys; Esc and Enter end it
      if (k == LogicalKeyboardKey.escape) {
        stopTyping();
        return true;
      }
      if (k == LogicalKeyboardKey.enter || k == LogicalKeyboardKey.numpadEnter) {
        setText(input.text);
        return true;
      }
      return false;
    }
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
    if (k == LogicalKeyboardKey.tab) return _then(() => activate(_nextEditable(kb.isShiftPressed ? -1 : 1)));
    if (e is! KeyDownEvent) return false;
    if (k == LogicalKeyboardKey.enter) return _then(startTyping);
    if (k == LogicalKeyboardKey.keyT) return _then(toggleScope);
    if (k == LogicalKeyboardKey.keyR) return _then(resetActive);
    if (k == LogicalKeyboardKey.keyA) return _then(ab);
    if (k == LogicalKeyboardKey.keyC) return _then(toggleChanges);
    if (k == LogicalKeyboardKey.escape) return _then(cancelActive);
    if (k == LogicalKeyboardKey.bracketLeft || k == LogicalKeyboardKey.bracketRight) {
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
    if (!on || exiting || typing || e is! PointerScrollEvent || hudRect.contains(e.position)) return;
    _wheel += e.scrollDelta.dy;
    final steps = (_wheel / 20).truncate();
    if (steps != 0) {
      _wheel -= steps * 20;
      for (var i = 0; i < math.min(steps.abs(), 3); i++) {
        step(-steps.sign);
      }
    }
  }
}

// ---------------------------------------------------------------------------------------------------------------- what is drawn

/// The handle on the selected box for the active number: pull it and the number follows (width, height, padding).
class _Grip extends StatelessWidget {
  const _Grip(this.c);
  final DesignController c;

  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: c,
    builder: (context, _) {
      final g = c.grip;
      if (g == null) return const SizedBox.shrink();
      final horizontal = g.axis == Axis.horizontal;
      final pull = c.dragged ?? 0;
      final at = g.at + (horizontal ? Offset(pull, 0) : Offset(0, pull));
      return Stack(
        fit: StackFit.expand,
        children: [
          Positioned(
            left: at.dx - 12,
            top: at.dy - 12,
            width: 24,
            height: 24,
            child: MouseRegion(
              cursor: horizontal ? SystemMouseCursors.resizeColumn : SystemMouseCursors.resizeRow,
              child: GestureDetector(
                behavior: HitTestBehavior.opaque,
                onPanStart: (_) => c.dragStart(),
                onPanUpdate: (d) => c.dragUpdate(horizontal ? d.delta.dx : d.delta.dy),
                onPanEnd: (_) => c.dragEnd(),
                onPanCancel: c.dragEnd,
                child: Center(
                  child: Container(
                    width: horizontal ? 6 : 22,
                    height: horizontal ? 22 : 6,
                    decoration: BoxDecoration(
                      color: _Hud.hot,
                      borderRadius: BorderRadius.circular(3),
                      border: Border.all(color: const Color(0xFF000000)),
                    ),
                  ),
                ),
              ),
            ),
          ),
        ],
      );
    },
  );
}

RenderPadding? _paddingBelow(RenderObject from) {
  RenderObject? at = from;
  for (var i = 0; i < 8 && at != null; i++) {
    if (at is RenderPadding) return at;
    RenderObject? next;
    at.visitChildren((c) => next ??= c);
    at = next;
  }
  return null;
}

class _BoundsPainter extends CustomPainter {
  _BoundsPainter(this.c) : super(repaint: c);
  final DesignController c;

  @override
  void paint(Canvas canvas, Size size) {
    final r = c.current;
    final ro = boxOf(r?.node.element);
    if (r == null || r.node.element!.debugIsDefunct || ro is! RenderBox || !ro.attached || !ro.hasSize) return;
    final box = ro.localToGlobal(Offset.zero) & ro.size;
    // the padding of the selected widget: a Container's lies a few render objects down (its Padding child), so look for it there
    final pad = (r.prop?.name.startsWith('padding') ?? false) ? _paddingBelow(ro) : null;
    if (pad != null && pad.attached && pad.hasSize) {
      final pb = pad.localToGlobal(Offset.zero) & pad.size;
      final p = pad.padding.resolve(TextDirection.ltr);
      final inner = Rect.fromLTRB(pb.left + p.left, pb.top + p.top, pb.right - p.right, pb.bottom - p.bottom);
      canvas.drawPath(
        Path.combine(PathOperation.difference, Path()..addRect(pb), Path()..addRect(inner)),
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
    // a GestureDetector's hit bounds, as the render object says them: opaque and translucent listen over their whole box (translucent
    // also lets what is behind it hear); deferToChild listens only where its child is hit, which is not a rectangle of its own
    if (r.node.widget == 'GestureDetector') {
      final hit = _hitListener(ro);
      if (hit != null && hit.attached && hit.hasSize) {
        final hb = hit.localToGlobal(Offset.zero) & hit.size;
        final green = const Color(0xFF4FD1A5);
        if (hit.behavior == HitTestBehavior.opaque) {
          canvas.drawRect(hb.deflate(2), Paint()..color = green.withValues(alpha: 0.18));
        }
        if (hit.behavior != HitTestBehavior.deferToChild) {
          canvas.drawRect(
            hb.deflate(2),
            Paint()
              ..style = PaintingStyle.stroke
              ..strokeWidth = 1.5
              ..color = green,
          );
        }
        final tp = TextPainter(
          text: TextSpan(
            text: 'hit: ${hit.behavior.name}${hit.behavior == HitTestBehavior.deferToChild ? ' (where its child is hit)' : ''}',
            style: const TextStyle(color: Color(0xFF4FD1A5), fontSize: 9, fontFamily: 'Menlo', backgroundColor: Color(0xCC000000)),
          ),
          textDirection: TextDirection.ltr,
        )..layout();
        tp.paint(canvas, hb.bottomLeft + const Offset(0, -12));
      }
    }
  }

  static RenderProxyBoxWithHitTestBehavior? _hitListener(RenderObject from) {
    RenderObject? at = from;
    for (var i = 0; i < 8 && at != null; i++) {
      if (at is RenderProxyBoxWithHitTestBehavior) return at;
      RenderObject? next;
      at.visitChildren((c) => next ??= c);
      at = next;
    }
    return null;
  }

  @override
  bool shouldRepaint(_BoundsPainter old) => true;
}

class _Hud extends StatefulWidget {
  const _Hud(this.c);
  final DesignController c;

  static const bg = Color(0xF2101114), ink = Color(0xFFE6E6E6), dim = Color(0xFF8A8D93), hot = Color(0xFFFFB347), rowBg = Color(0xFF23252B);

  static TextStyle t([Color col = ink, double size = 11, FontWeight w = FontWeight.w400]) =>
      TextStyle(fontFamily: 'Menlo', fontSize: size, color: col, fontWeight: w, height: 1.25, decoration: TextDecoration.none);

  @override
  State<_Hud> createState() => _HudState();
}

class _HudState extends State<_Hud> {
  final _key = GlobalKey();
  final _scroll = ScrollController();
  final _activeKey = GlobalKey();

  DesignController get c => widget.c;
  static final _t = _Hud.t;

  @override
  void dispose() {
    _scroll.dispose();
    super.dispose();
  }

  String _badge(_Row r) {
    final p = r.prop;
    if (p == null) return 'DERIVED';
    if (!p.editable) return 'READ-ONLY';
    if (p.origin == Origin.unset) return 'UNSET';
    if (p.isToken) return r.scope == Scope.token ? 'TOKEN ${r.usage ?? ''}' : 'HERE';
    if (p.origin == Origin.exception) return 'EXCEPTION';
    return p.via != null ? 'VIA ${p.via}' : 'RAW';
  }

  /// The blast radius, said before the change: which scope the next change has.
  String _scopeLine(_Row r) {
    final p = r.prop;
    if (p == null) return '';
    final at = '${p.file.split('/').last}:${r.line}';
    if (p.isToken) {
      return r.scope == Scope.token
          ? 'TOKEN ${p.token} · ${r.usage ?? '?'} uses · T: this place only'
          : 'HERE ${p.widget} $at only · T: edit ${p.token} (${r.usage ?? '?'} uses)';
    }
    if (p.origin == Origin.unset) return 'adds ${p.copyWith ? '.copyWith(' : ''}${p.insert}${p.copyWith ? ')' : ''} at $at';
    if (!p.editable) return p.note ?? '';
    return 'HERE $at${p.via == null ? '' : ' (declaration of ${p.via})'}';
  }

  @override
  Widget build(BuildContext context) {
    final size = MediaQuery.sizeOf(context);
    final ro = boxOf(c.current?.node.element);
    final at = ro is RenderBox && ro.attached && ro.hasSize
        ? ro.localToGlobal(ro.size.center(Offset.zero))
        : Offset(size.width, size.height);
    final hudLeft = at.dx > size.width / 2, hudTop = at.dy > size.height / 2; // the HUD stays out of the way of what is selected
    WidgetsBinding.instance.addPostFrameCallback((_) {
      final b = _key.currentContext?.findRenderObject();
      if (b is RenderBox && b.attached && b.hasSize) c.hudRect = b.localToGlobal(Offset.zero) & b.size;
      final a = _activeKey.currentContext;
      if (a != null && _scroll.hasClients) Scrollable.ensureVisible(a, alignment: 0.5, duration: const Duration(milliseconds: 80));
    });
    final body = <Widget>[];
    if (c.exiting) {
      body.addAll([
        Text('${c.session.changes().length} changed lines', style: _t(_Hud.hot, 12, FontWeight.w600)),
        Text('Enter KEEP: the changes stay in the source', style: _t()),
        Text('Esc   DISCARD: back to the session start', style: _t()),
        Row(children: [_Chip('KEEP', c.keep), _Chip('DISCARD', c.discard), _Chip('CONTINUE', c.toggleOnly)]),
      ]);
    } else if (c.showChanges) {
      body.add(Text('SESSION CHANGES', style: _t(_Hud.hot, 11, FontWeight.w600)));
      final ch = c.session.changes();
      if (ch.isEmpty) body.add(Text('nothing changed', style: _t(_Hud.dim)));
      for (final (f, l, a, b) in ch) {
        body.add(Text('${f.split('/').last}:$l', style: _t(_Hud.dim)));
        body.add(Text('  $a', style: _t(const Color(0xFFD97B7B))));
        body.add(Text('  $b', style: _t(const Color(0xFF7BD98F))));
      }
    } else {
      String? node;
      for (var i = 0; i < c.rows.length; i++) {
        final r = c.rows[i];
        if (r.node.head != node) {
          node = r.node.head;
          body.add(
            Padding(
              padding: const EdgeInsets.only(top: 3),
              child: Text(node, style: _t(_Hud.hot, 10, FontWeight.w600), maxLines: 1, overflow: TextOverflow.ellipsis),
            ),
          );
        }
        body.add(_RowView(c, i, r, _badge(r), i == c.active ? _activeKey : null));
        if (i == c.active && c.choosing && r.prop != null) body.add(_Choices(c, r.prop!));
      }
    }
    final maxH = size.height * 0.7;
    return Positioned(
      left: hudLeft ? 8 : null,
      right: hudLeft ? null : 8,
      top: hudTop ? 8 : null,
      bottom: hudTop ? null : 8,
      width: 400,
      child: Container(
        key: _key,
        padding: const EdgeInsets.all(8),
        decoration: BoxDecoration(
          color: _Hud.bg,
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
                  _Chip('−', () => c.scale(-1)),
                  Text('UI ${UiScale.percent}%', style: _t(_Hud.dim)),
                  _Chip('+', () => c.scale(1)),
                ],
              ),
              const SizedBox(height: 4),
              ConstrainedBox(
                constraints: BoxConstraints(maxHeight: maxH),
                child: SingleChildScrollView(
                  controller: _scroll,
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: body.isEmpty ? [Text('Design Mode: click a thing', style: _t(_Hud.hot, 11, FontWeight.w600))] : body,
                  ),
                ),
              ),
              if (!c.exiting && !c.showChanges && c.current != null)
                Padding(
                  padding: const EdgeInsets.only(top: 4),
                  child: Text(_scopeLine(c.current!), style: _t(_Hud.dim, 10), maxLines: 2),
                ),
              if (c.note != null)
                Padding(
                  padding: const EdgeInsets.only(top: 3),
                  child: Text(c.note!, style: _t(_Hud.hot)),
                ),
              Wrap(
                children: [
                  _Chip('A/B', c.ab),
                  _Chip('UNDO', c.undo),
                  _Chip('REDO', c.redo),
                  _Chip('RESET', c.resetActive),
                  _Chip('TOKEN/HERE', c.toggleScope),
                  _Chip('TYPE', () => c.startTyping(list: false)),
                  _Chip('CHANGES', c.toggleChanges),
                  _Chip('DONE', c.toggle),
                ],
              ),
              Text('Tab · wheel/↑↓ · Enter type/choices · T scope · A before · R reset · Z undo · Esc · C · [ ]', style: _t(_Hud.dim, 9)),
            ],
          ),
        ),
      ),
    );
  }
}

class _Chip extends StatelessWidget {
  const _Chip(this.label, this.onTap);
  final String label;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) => GestureDetector(
    behavior: HitTestBehavior.opaque,
    onTap: onTap,
    child: Container(
      margin: const EdgeInsets.only(right: 4, top: 4),
      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
      decoration: BoxDecoration(color: _Hud.rowBg, borderRadius: BorderRadius.circular(3)),
      child: Text(label, style: _Hud.t(_Hud.ink, 9, FontWeight.w600)),
    ),
  );
}

/// The candidates of a choice or a colour token, to click.
class _Choices extends StatelessWidget {
  const _Choices(this.c, this.p);
  final DesignController c;
  final Prop p;

  @override
  Widget build(BuildContext context) {
    final opts = p.options.isEmpty ? (p.kind == PropKind.boolean ? const ['true', 'false'] : const <String>[]) : p.options;
    return Container(
      margin: const EdgeInsets.only(left: 12, bottom: 2),
      constraints: const BoxConstraints(maxHeight: 150),
      decoration: BoxDecoration(
        color: const Color(0xFF181A1F),
        border: Border.all(color: const Color(0xFF3A3D45)),
      ),
      child: SingleChildScrollView(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            for (final o in opts)
              GestureDetector(
                behavior: HitTestBehavior.opaque,
                onTap: () => c.setText(o),
                child: Container(
                  width: double.infinity,
                  color: o == p.text ? _Hud.rowBg : null,
                  padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 1),
                  child: Row(
                    children: [
                      if (p.kind == PropKind.color && c.colorOf(o) != null)
                        Container(width: 10, height: 10, margin: const EdgeInsets.only(right: 6), color: c.colorOf(o)),
                      Text(o, style: _Hud.t(o == p.text ? _Hud.hot : _Hud.ink)),
                    ],
                  ),
                ),
              ),
          ],
        ),
      ),
    );
  }
}

class _RowView extends StatelessWidget {
  const _RowView(this.c, this.i, this.r, this.badge, this.activeKey);
  final DesignController c;
  final int i;
  final _Row r;
  final String badge;
  final GlobalKey? activeKey;

  @override
  Widget build(BuildContext context) {
    final on = i == c.active;
    final p = r.prop;
    final t = _Hud.t;
    final cond = p?.condition == null ? '' : '${p!.condition} → ';
    final shown = p == null
        ? r.derivedText!
        : cond + (p.kind == PropKind.number && p.isToken ? '${p.token}' : (p.kind == PropKind.number ? '' : p.text));
    final value = p == null ? null : (p.kind == PropKind.number ? p.text : null);
    final swatch = p != null && p.kind == PropKind.color ? c.colorOf(p.text) : null;
    return GestureDetector(
      key: activeKey,
      behavior: HitTestBehavior.opaque,
      onTap: () => on && p != null && p.editable ? c.startTyping() : c.activate(i),
      onHorizontalDragStart: (_) => c.activate(i),
      onHorizontalDragUpdate: (d) {
        if (r.editable && p!.kind == PropKind.number) c.step(d.delta.dx.sign.toInt());
      },
      child: Container(
        color: on ? _Hud.rowBg : null,
        padding: const EdgeInsets.symmetric(vertical: 1, horizontal: 2),
        child: Row(
          children: [
            SizedBox(width: 12, child: Text(r.edited ? '●' : (on ? '▸' : ''), style: t(_Hud.hot))),
            SizedBox(
              width: 118,
              child: Text(r.name, style: t(r.editable ? _Hud.dim : const Color(0xFF5A5D63)), maxLines: 1, overflow: TextOverflow.ellipsis),
            ),
            if (swatch != null) Container(width: 9, height: 9, margin: const EdgeInsets.only(right: 4), color: swatch),
            Expanded(
              child: on && c.typing && p != null && p.editable
                  ? EditableText(
                      controller: c.input,
                      focusNode: c.inputFocus,
                      style: t(_Hud.hot, 11, FontWeight.w600),
                      cursorColor: _Hud.hot,
                      backgroundCursorColor: _Hud.dim,
                      selectionColor: const Color(0x66FFB347),
                      onSubmitted: c.setText,
                      maxLines: 1,
                    )
                  : Text(shown, style: t(r.editable ? _Hud.ink : _Hud.dim), maxLines: 1, overflow: TextOverflow.ellipsis),
            ),
            if (value != null)
              SizedBox(
                width: 46,
                child: Text(value, textAlign: TextAlign.right, style: t(on ? _Hud.hot : _Hud.ink, 11, FontWeight.w600)),
              ),
            SizedBox(
              width: 84,
              child: Text(badge, textAlign: TextAlign.right, style: t(_Hud.dim, 9), maxLines: 1, overflow: TextOverflow.clip),
            ),
          ],
        ),
      ),
    );
  }
}
