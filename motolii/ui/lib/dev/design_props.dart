// Design Mode's property model: what in the source of a clicked widget's call is a number, a switch, one of a few named choices, or a
// colour, where exactly it is written, where it comes from (a literal, a canonical token, a registered exception, a declaration a
// few lines up), and the one-place text edit that changes it. Plain Dart over source text: no Flutter, no I/O of its own (files come
// in through a reader), so the HUD, tool/ui_inspect.dart and the tests share it. Nothing is changed by pattern over a whole file:
// an edit is the replacement of one expression's span (or the insertion of one named argument) found from the widget's creation location.
import 'design_core.dart';

enum PropKind { number, boolean, option, color, readOnly }

/// Where the value comes from.
enum Origin {
  /// Written here (a number, `true`, `TextAlign.center`, `Color(0xFF…)`).
  literal,

  /// A canonical token (`Surface.sectionGap`, `N.g20`): editing it changes every use, editing here replaces the reference.
  token,

  /// `Surface.px(7)`: a registered custom dimension, local to its line.
  exception,

  /// Not set in the source: the widget's default; editing adds the argument.
  unset,
}

/// A place a value can be read and changed. [start]..[end] is its span in [file] (an unset one has start == end, the insertion point).
class Prop {
  Prop({
    required this.widget,
    required this.name,
    required this.kind,
    required this.origin,
    required this.file,
    required this.start,
    required this.end,
    required this.text,
    required this.expr,
    this.options = const [],
    this.token,
    this.note,
    this.condition,
    this.via,
    this.insert,
    this.copyWith = false,
    this.copyExisting = false,
    this.sizeOf,
  });

  final String widget, name, file, text;

  /// What the source says at [start]..[end] (an edit checks it is still there).
  final String expr;
  final PropKind kind;
  final Origin origin;
  int start, end;
  final List<String> options;

  /// The canonical token this value is (`Surface.sectionGap`, `N.g20`).
  final String? token;

  /// Why it is read-only, or what the default of an unset one is.
  final String? note;

  /// The condition of a branch (`widget.hero`): this value is used `when` it holds, the other when it does not.
  final String? condition;

  /// The local declaration this value was followed through (`size`), edited at its declaration.
  final String? via;

  /// For an unset one: how it is added (the named argument's name, `textAlign`).
  final String? insert;

  /// An unset or token-only style value that an instance changes by `.copyWith(name: …)`.
  final bool copyWith;

  /// The `.copyWith(` it goes into already exists in the source (an argument is added to it, not a new call).
  final bool copyExisting;

  /// A style's size role (`Dn.name`): the size token it is built from.
  final String? sizeOf;

  bool get editable => kind != PropKind.readOnly;
  bool get isToken => origin == Origin.token && token != null;
  double? get number => double.tryParse(text);
  String get label => condition == null ? name : '$name ${condition!}';
}

/// What the analysis knows about the product: numeric tokens, colour tokens, the choices of each enum.
class Registry {
  Registry({required this.tokens, required this.colors, Map<String, List<String>>? enums}) : enums = enums ?? knownEnums;
  final Map<String, Token> tokens;

  /// `Surface.base` -> `N.g10`, `N.g10` -> `Color(0xFF191919)`.
  final Map<String, String> colors;
  final Map<String, List<String>> enums;
}

const knownEnums = <String, List<String>>{
  'Alignment': ['topLeft', 'topCenter', 'topRight', 'centerLeft', 'center', 'centerRight', 'bottomLeft', 'bottomCenter', 'bottomRight'],
  'TextAlign': ['left', 'right', 'center', 'justify', 'start', 'end'],
  'FontWeight': ['w100', 'w200', 'w300', 'w400', 'w500', 'w600', 'w700', 'w800', 'w900'],
  'Axis': ['horizontal', 'vertical'],
  'MainAxisAlignment': ['start', 'end', 'center', 'spaceBetween', 'spaceAround', 'spaceEvenly'],
  'CrossAxisAlignment': ['start', 'end', 'center', 'stretch', 'baseline'],
  'MainAxisSize': ['min', 'max'],
  'FlexFit': ['tight', 'loose'],
  'TextOverflow': ['clip', 'fade', 'ellipsis', 'visible'],
  'TextBaseline': ['alphabetic', 'ideographic'],
  'HitTestBehavior': ['deferToChild', 'opaque', 'translucent'],
  'Clip': ['none', 'hardEdge', 'antiAlias', 'antiAliasWithSaveLayer'],
  'StackFit': ['loose', 'expand', 'passthrough'],
  'BoxFit': ['fill', 'contain', 'cover', 'fitWidth', 'fitHeight', 'none', 'scaleDown'],
  'BoxShape': ['rectangle', 'circle'],
  'VerticalDirection': ['up', 'down'],
  'TextDirection': ['ltr', 'rtl'],
  'WrapAlignment': ['start', 'end', 'center', 'spaceBetween', 'spaceAround', 'spaceEvenly'],
  'WrapCrossAlignment': ['start', 'end', 'center'],
  'BorderStyle': ['none', 'solid'],
  'FilterQuality': ['none', 'low', 'medium', 'high'],
  'TileMode': ['clamp', 'repeated', 'mirror', 'decal'],
};

/// The colour tokens of the product: `N` (neutral.dart) and `H` (identity.dart) declare colours, `Surface` aliases them.
Map<String, String> parseColorTokens({String? neutral, String? identity, String? metrics, Iterable<String> others = const []}) {
  final out = <String, String>{};
  void consts(String? src, String cls) {
    if (src == null) return;
    for (final m in RegExp(r'static const (\w+) = (Color\(0x[0-9A-Fa-f]{8}\))').allMatches(src)) {
      out['$cls.${m.group(1)}'] = m.group(2)!;
    }
  }

  for (final o in others) {
    for (final m in RegExp(
      r'^\\s*(?:static\\s+)?(?:const|final)\\s+(k\\w+)\\s*=\\s*(Color\\(0x[0-9A-Fa-f]{8}\\))',
      multiLine: true,
    ).allMatches(o)) {
      out[m.group(1)!] = m.group(2)!;
    }
  }
  consts(neutral, 'N');
  consts(identity, 'H');
  if (metrics != null) {
    for (final line in metrics.split('\n')) {
      final m = RegExp(r'static const (\w+ = N\.g\d+(?:, \w+ = N\.g\d+)*);').firstMatch(line);
      if (m == null) continue;
      for (final part in m.group(1)!.split(',')) {
        final kv = RegExp(r'(\w+) = (N\.g\d+)').firstMatch(part)!;
        out['Surface.${kv.group(1)}'] = kv.group(2)!;
      }
    }
  }
  return out;
}

// ----------------------------------------------------------------------------------------------------------------- source text

class Arg {
  Arg(this.name, this.start, this.end);
  final String name;

  /// The span of the value (after `name:`), trimmed.
  final int start, end;
}

/// A source file: its text, the same text with comments and strings blanked (brackets in them do not count), and the structure queries.
class Src {
  Src(this.file, this.text) : blank = blankCommentsAndStrings(text);
  final String file, text, blank;

  int offsetOf(int line, int col) {
    var off = 0, l = 1;
    while (l < line) {
      final nl = text.indexOf('\n', off);
      if (nl < 0) return text.length;
      off = nl + 1;
      l++;
    }
    return off + col - 1;
  }

  int lineOf(int offset) => '\n'.allMatches(text.substring(0, offset.clamp(0, text.length))).length + 1;

  /// The offset of the bracket that closes the one at [open].
  int close(int open) {
    var d = 0;
    for (var i = open; i < blank.length; i++) {
      final c = blank[i];
      if (c == '(' || c == '[' || c == '{') d++;
      if (c == ')' || c == ']' || c == '}') {
        d--;
        if (d == 0) return i;
      }
    }
    return blank.length;
  }

  (int, int) trim(int a, int b) {
    while (a < b && ' \n\t'.contains(blank[a])) {
      a++;
    }
    while (b > a && ' \n\t'.contains(blank[b - 1])) {
      b--;
    }
    return (a, b);
  }

  /// The arguments between [open]+1 and [close]: named ones by name, positional ones with name ''.
  List<Arg> args(int open, int closeAt) {
    final out = <Arg>[];
    var d = 0, start = open + 1;
    void add(int s, int e) {
      final (a, b) = trim(s, e);
      if (a >= b) return;
      final m = RegExp(r'^(\w+)\s*:(?!:)').firstMatch(blank.substring(a, b));
      if (m == null) {
        out.add(Arg('', a, b));
      } else {
        final (va, vb) = trim(a + m.end, b);
        out.add(Arg(m.group(1)!, va, vb));
      }
    }

    for (var i = open + 1; i < closeAt; i++) {
      final c = blank[i];
      if (c == '(' || c == '[' || c == '{') d++;
      if (c == ')' || c == ']' || c == '}') d--;
      if (c == ',' && d == 0) {
        add(start, i);
        start = i + 1;
      }
    }
    add(start, closeAt);
    return out;
  }

  /// [a] after a leading `const ` / `new `.
  int skipConst(int a, int b) {
    final m = RegExp(r'^(?:const|new)\s+').firstMatch(blank.substring(a, b));
    return m == null ? a : a + m.end;
  }

  /// A top-level `x ?? fallback`: the offset of the `??`.
  int? coalesce(int a, int b) {
    var d = 0;
    for (var i = a; i < b - 1; i++) {
      final c = blank[i];
      if (c == '(' || c == '[' || c == '{') d++;
      if (c == ')' || c == ']' || c == '}') d--;
      if (d == 0 && c == '?' && blank[i + 1] == '?') return i;
    }
    return null;
  }

  /// A top-level `cond ? a : b` in [a]..[b]: the offsets of the `?` and the `:`.
  (int, int)? ternary(int a, int b) {
    var d = 0;
    int? q;
    for (var i = a; i < b; i++) {
      final c = blank[i];
      if (c == '(' || c == '[' || c == '{') d++;
      if (c == ')' || c == ']' || c == '}') d--;
      if (d != 0) continue;
      if (c == '?' && q == null) {
        final next = i + 1 < b ? blank[i + 1] : ' ', prev = i > a ? blank[i - 1] : ' ';
        if ((next == ' ' || next == '\n') && prev != '?' && prev != '.' && prev != '>') q = i;
      } else if (c == ':' && q != null) {
        return (q, i);
      }
    }
    return null;
  }
}

/// The right-hand side of the declaration of the lower-case name [ident] before [before] in [s]: a `final` / `const` / `var` / typed
/// local, a parameter's default, or a file-level constant. Null when there is none to follow.
(int, int)? declaration(Src s, int before, String ident) {
  final kw = RegExp(
    '(?:^|[;{}(,\\s])(?:(?:final|const|var|late)\\s+(?:[A-Za-z_][\\w<>?]*\\s+)?|(?:double|int|bool|num|Color|FontWeight|TextAlign|Alignment|MainAxisAlignment|CrossAxisAlignment)\\s+)${RegExp.escape(ident)}\\s*=(?!=)\\s*',
  );
  Match? best;
  for (final m in kw.allMatches(s.blank.substring(0, before.clamp(0, s.blank.length)))) {
    best = m;
  }
  if (best == null) {
    for (final m in kw.allMatches(s.blank)) {
      best = m;
      break;
    }
  }
  if (best == null) {
    // a named getter: `static double get rowH => Surface.control;`
    final get = RegExp('(?:^|[;{}\\s])(?:static\\s+)?(?:[A-Za-z_][\\w<>?]*\\s+)get\\s+${RegExp.escape(ident)}\\s*=>\\s*');
    for (final m in get.allMatches(s.blank)) {
      best = m;
      break;
    }
  }
  if (best == null) return null;
  final a = best.end;
  var d = 0;
  for (var i = a; i < s.blank.length; i++) {
    final c = s.blank[i];
    if (c == '(' || c == '[' || c == '{') d++;
    if (c == ')' || c == ']' || c == '}') {
      if (d == 0) return s.trim(a, i);
      d--;
    }
    if (d == 0 && (c == ';' || c == ',')) return s.trim(a, i);
  }
  return null;
}

// ----------------------------------------------------------------------------------------------------------------- analysis

class _Style {
  const _Style({this.positional = const [], this.named = const {}, this.sizeOf});
  final List<String> positional;
  final Map<String, String> named; // argument name -> style property

  /// A role: its size is a token (`Dn.name` -> `Dn.nameSize`), not an argument.
  final String? sizeOf;
}

const _styleCalls = <String, _Style>{
  'sans': _Style(positional: ['fontSize'], named: {'c': 'color', 'w': 'fontWeight', 'ls': 'letterSpacing'}),
  'mono': _Style(positional: ['fontSize'], named: {'c': 'color', 'ls': 'letterSpacing'}),
  'caps': _Style(positional: ['fontSize'], named: {'c': 'color'}),
  'H.s': _Style(positional: ['fontSize'], named: {'color': 'color', 'w': 'fontWeight', 'ls': 'letterSpacing'}),
  'H.m': _Style(positional: ['fontSize'], named: {'color': 'color', 'ls': 'letterSpacing'}),
  'TextStyle': _Style(
    named: {'fontSize': 'fontSize', 'color': 'color', 'fontWeight': 'fontWeight', 'letterSpacing': 'letterSpacing', 'height': 'height'},
  ),
  'Dn.name': _Style(positional: ['color', 'fontWeight'], sizeOf: 'Dn.nameSize'),
  'Dn.label': _Style(positional: ['color', 'fontWeight'], sizeOf: 'Dn.labelSize'),
  'Dn.micro': _Style(positional: ['color'], sizeOf: 'Dn.microSize'),
  'Dn.value': _Style(positional: ['color'], sizeOf: 'Dn.numericSize'),
};

const _styleProps = ['fontSize', 'height', 'fontWeight', 'letterSpacing', 'color'];
const _styleKind = {
  'fontSize': PropKind.number,
  'height': PropKind.number,
  'letterSpacing': PropKind.number,
  'fontWeight': PropKind.option,
  'color': PropKind.color,
};

/// What a widget can be given that its source may not say, with the default shown until it is set.
const _catalog = <String, List<(String, PropKind, String, String?)>>{
  // widget: [(name, kind, default, enum type)]
  'Text': [
    ('textAlign', PropKind.option, 'TextAlign.start', 'TextAlign'),
    ('overflow', PropKind.option, 'TextOverflow.clip', 'TextOverflow'),
    ('maxLines', PropKind.number, '1', null),
    ('softWrap', PropKind.boolean, 'true', null),
  ],
  'Container': [
    ('alignment', PropKind.option, 'Alignment.center', 'Alignment'),
    ('width', PropKind.number, '0', null),
    ('height', PropKind.number, '0', null),
  ],
  'Row': [
    ('mainAxisAlignment', PropKind.option, 'MainAxisAlignment.start', 'MainAxisAlignment'),
    ('crossAxisAlignment', PropKind.option, 'CrossAxisAlignment.center', 'CrossAxisAlignment'),
    ('mainAxisSize', PropKind.option, 'MainAxisSize.max', 'MainAxisSize'),
  ],
  'Column': [
    ('mainAxisAlignment', PropKind.option, 'MainAxisAlignment.start', 'MainAxisAlignment'),
    ('crossAxisAlignment', PropKind.option, 'CrossAxisAlignment.center', 'CrossAxisAlignment'),
    ('mainAxisSize', PropKind.option, 'MainAxisSize.max', 'MainAxisSize'),
  ],
  'Expanded': [('flex', PropKind.number, '1', null)],
  'Flexible': [('flex', PropKind.number, '1', null), ('fit', PropKind.option, 'FlexFit.loose', 'FlexFit')],
  'Align': [('alignment', PropKind.option, 'Alignment.center', 'Alignment')],
  'Stack': [('alignment', PropKind.option, 'Alignment.topLeft', 'Alignment'), ('fit', PropKind.option, 'StackFit.loose', 'StackFit')],
  'SizedBox': [('width', PropKind.number, '0', null), ('height', PropKind.number, '0', null)],
  'GestureDetector': [('behavior', PropKind.option, 'HitTestBehavior.deferToChild', 'HitTestBehavior')],
};

const _colorNames = {
  'color',
  'backgroundColor',
  'foregroundColor',
  'borderColor',
  'shadowColor',
  'iconColor',
  'cursorColor',
  'backgroundCursorColor',
  'selectionColor',
  'hoverColor',
  'focusColor',
  'fillColor',
  'thumbColor',
  'trackColor',
};
const _numberNames = {
  'weight',
  'radius',
  'widthFactor',
  'heightFactor',
  'itemExtent',
  'cacheExtent',
  'baseline',
  'aspectRatio',
  'sigmaX',
  'sigmaY',
  'minScale',
  'maxScale',
  'scale',
  'width',
  'height',
  'flex',
  'top',
  'left',
  'right',
  'bottom',
  'spacing',
  'runSpacing',
  'size',
  'thickness',
  'dimension',
  'strokeWidth',
  'minWidth',
  'maxWidth',
  'minHeight',
  'maxHeight',
  'gap',
  'inset',
  'blurRadius',
  'spreadRadius',
  'fontSize',
  'cursorWidth',
  'cursorHeight',
  'letterSpacing',
  'maxLines',
  'elevation',
  'opacity',
};
const _optionNames = {
  'tileMode',
  'textAlign',
  'overflow',
  'mainAxisAlignment',
  'crossAxisAlignment',
  'mainAxisSize',
  'fit',
  'behavior',
  'clipBehavior',
  'alignment',
  'direction',
  'textBaseline',
  'verticalDirection',
  'textDirection',
  'shape',
  'runAlignment',
  'wrapAlignment',
  'wrapCrossAlignment',
  'filterQuality',
};
const _nestedTypes = {
  'BorderSide',
  'Border',
  'BoxDecoration',
  'BoxConstraints',
  'Radius',
  'LinearGradient',
  'RadialGradient',
  'SweepGradient',
  'BoxShadow',
};
const _boolNames = {'softWrap', 'shrinkWrap', 'expands'};

class Analyzer {
  Analyzer(this.read, this.reg, {this.files});
  final String Function(String file) read;

  /// The product's source files, to follow `Class.member` to its declaration.
  final Iterable<String> Function()? files;
  final Registry reg;
  final _srcs = <String, Src>{};

  Src src(String file) => _srcs.putIfAbsent(file, () => Src(file, read(file)));

  /// Forget what was read: the files changed.
  void forget() {
    _srcs.clear();
    _members.clear();
  }

  final _members = <String, (Src, int, int)?>{};

  /// The declaration of `Cls.member` (a `static const` or `static … get`) in the file that declares `class Cls`.
  (Src, int, int)? member(String cls, String name) => _members.putIfAbsent('$cls.$name', () {
    for (final f in files?.call() ?? const <String>[]) {
      final t = src(f);
      final c = RegExp('class\\s+$cls\\b').firstMatch(t.blank);
      if (c == null) continue;
      final d = declaration(t, t.blank.length, name);
      if (d != null) return (t, d.$1, d.$2);
    }
    return null;
  });

  /// The properties of the widget call at [file] [line]:[col] (1-based): those its source gives, then those the widget could be given.
  List<Prop> analyze(String file, int line, int col) {
    final s = src(file);
    final at = s.offsetOf(line, col);
    final open = s.blank.indexOf('(', at);
    if (open < 0) return [];
    final head = RegExp(r'([A-Za-z_]\w*(?:\.[A-Za-z_]\w*)?)\s*$').firstMatch(s.blank.substring(at, open));
    final widget = (head?.group(1) ?? s.blank.substring(at, open).trim()).split('.').first;
    final closeAt = s.close(open);
    final out = <Prop>[];
    final given = <String>{};
    for (final a in s.args(open, closeAt)) {
      if (a.name.isEmpty) continue;
      given.add(a.name);
      _arg(s, widget, a.name, a.start, a.end, out, '');
    }
    for (final (name, kind, def, enumType) in _catalog[widget] ?? const <(String, PropKind, String, String?)>[]) {
      if (given.contains(name)) continue;
      out.add(
        Prop(
          widget: widget,
          name: name,
          kind: kind,
          origin: Origin.unset,
          file: file,
          start: closeAt,
          end: closeAt,
          text: def,
          expr: '',
          note: 'default',
          insert: name,
          options: enumType == null ? const [] : [for (final m in reg.enums[enumType] ?? const <String>[]) '$enumType.$m'],
        ),
      );
    }
    // the text style of a Text: unset size / weight / colour / line height / letter spacing are reachable too
    return out;
  }

  // one argument by its name and the shape of its value
  void _arg(Src s, String w, String name, int a, int b, List<Prop> out, String prefix, {String? cond}) {
    final p = '$prefix$name';
    final expr = s.text.substring(a, b);
    final shape = RegExp(r'^(?:const\s+)?([A-Z]\w*)(?:\.\w+)?\(').firstMatch(expr);
    final list = RegExp(r'^(?:const\s+)?\[').firstMatch(s.blank.substring(a, b));
    if ((name == 'colors' || name == 'stops') && list != null) {
      final open = a + list.end - 1;
      var i = 0;
      for (final x in s.args(open, s.close(open))) {
        _value(s, w, '$p[${i++}]', name == 'colors' ? PropKind.color : PropKind.number, x.start, x.end, out, cond: cond);
      }
    } else if (name == 'style') {
      _style(s, w, a, b, out, prefix, cond: cond, depth: 0);
    } else if (name == 'padding' || name == 'margin') {
      _insets(s, w, p, a, b, out, cond: cond);
    } else if (name == 'borderRadius') {
      _radius(s, w, p, a, b, out, cond: cond);
    } else if (name == 'decoration' || (shape != null && _nestedTypes.contains(shape.group(1)))) {
      _nested(s, w, p, a, b, out, cond: cond);
    } else if (name == 'border') {
      _nested(s, w, p, a, b, out, cond: cond);
    } else if (name == 'constraints') {
      _nested(s, w, p, a, b, out, cond: cond);
    } else if (_colorNames.contains(name)) {
      _value(s, w, p, PropKind.color, a, b, out, cond: cond);
    } else if (_numberNames.contains(name)) {
      _value(s, w, p, PropKind.number, a, b, out, cond: cond);
    } else if (_boolNames.contains(name) && RegExp(r'^(true|false)$').hasMatch(expr.trim())) {
      _value(s, w, p, PropKind.boolean, a, b, out, cond: cond);
    } else if (_optionType(expr) != null || _optionNames.contains(name)) {
      _value(s, w, p, PropKind.option, a, b, out, cond: cond);
    }
  }

  String? _optionType(String expr) {
    final m = RegExp(r'^([A-Z]\w*)\.(\w+)$').firstMatch(expr.trim());
    return m != null && reg.enums.containsKey(m.group(1)) ? m.group(1) : null;
  }

  // a nested constructor (BoxDecoration, Border, BoxConstraints, BorderSide, Radius…): each of its arguments, named under [prefix]
  void _nested(Src s, String w, String prefix, int a, int b, List<Prop> out, {String? cond, int depth = 0}) {
    final q = s.coalesce(a, b);
    if (q != null) {
      _nested(s, w, prefix, s.trim(q + 2, b).$1, b, out, cond: cond, depth: depth);
      return;
    }
    a = s.skipConst(a, b);
    final t = s.ternary(a, b);
    if (t != null) {
      final c = s.text.substring(a, t.$1).trim();
      final (a1, b1) = s.trim(t.$1 + 1, t.$2);
      final (a2, b2) = s.trim(t.$2 + 1, b);
      _nested(s, w, prefix, a1, b1, out, cond: 'if $c', depth: depth);
      _nested(s, w, prefix, a2, b2, out, cond: 'else', depth: depth);
      return;
    }
    final m = RegExp(r'^([A-Z]\w*)(?:\.(\w+))?\(').firstMatch(s.blank.substring(a, b));
    if (m == null) return;
    final open = a + m.end - 1;
    final closeAt = s.close(open);
    final ctor = m.group(2);
    final positional = s.args(open, closeAt).where((x) => x.name.isEmpty).toList();
    for (final x in s.args(open, closeAt)) {
      if (x.name.isNotEmpty) {
        _arg(s, w, x.name, x.start, x.end, out, '$prefix.', cond: cond);
      }
    }
    // Border.all(color:…, width:…) and the like name their parts by argument; `BorderSide(...)` inside Border(top: …) recurses
    if (ctor == 'all' && positional.isNotEmpty && m.group(1) == 'Radius') {
      _value(s, w, '$prefix.all', PropKind.number, positional.first.start, positional.first.end, out, cond: cond);
    }
  }

  void _insets(Src s, String w, String name, int a, int b, List<Prop> out, {String? cond}) {
    final q = s.coalesce(a, b);
    if (q != null) {
      _insets(s, w, name, s.trim(q + 2, b).$1, b, out, cond: cond);
      return;
    }
    a = s.skipConst(a, b);
    final t = s.ternary(a, b);
    if (t != null) {
      final c = s.text.substring(a, t.$1).trim();
      final (a1, b1) = s.trim(t.$1 + 1, t.$2);
      final (a2, b2) = s.trim(t.$2 + 1, b);
      _insets(s, w, name, a1, b1, out, cond: 'if $c');
      _insets(s, w, name, a2, b2, out, cond: 'else');
      return;
    }
    final m = RegExp(r'^EdgeInsets(?:Directional)?\.(\w+)\(').firstMatch(s.blank.substring(a, b));
    if (m == null) {
      if (RegExp(r'^[a-z_]\w*$').hasMatch(s.text.substring(a, b))) {
        final d = declaration(s, a, s.text.substring(a, b));
        if (d != null) {
          _insets(s, w, name, d.$1, d.$2, out, cond: cond);
          return;
        }
      }
      out.add(
        Prop(
          widget: w,
          name: name,
          kind: PropKind.readOnly,
          origin: Origin.literal,
          file: s.file,
          start: a,
          end: b,
          text: s.text.substring(a, b),
          expr: s.text.substring(a, b),
          note: 'set by the caller or by a theme: not written in this call',
          condition: cond,
        ),
      );
      return;
    }
    final open = a + m.end - 1;
    final closeAt = s.close(open);
    final args = s.args(open, closeAt);
    final ctor = m.group(1)!;
    if (ctor == 'fromLTRB') {
      const sides = ['left', 'top', 'right', 'bottom'];
      for (var i = 0; i < args.length && i < 4; i++) {
        _value(s, w, '$name.${sides[i]}', PropKind.number, args[i].start, args[i].end, out, cond: cond);
      }
    } else {
      for (final x in args) {
        _value(s, w, '$name.${x.name.isEmpty ? ctor : x.name}', PropKind.number, x.start, x.end, out, cond: cond);
      }
    }
  }

  void _radius(Src s, String w, String name, int a, int b, List<Prop> out, {String? cond}) {
    final q = s.coalesce(a, b);
    if (q != null) {
      _radius(s, w, name, s.trim(q + 2, b).$1, b, out, cond: cond);
      return;
    }
    a = s.skipConst(a, b);
    final t = s.ternary(a, b);
    if (t != null) {
      final c = s.text.substring(a, t.$1).trim();
      final (a1, b1) = s.trim(t.$1 + 1, t.$2);
      final (a2, b2) = s.trim(t.$2 + 1, b);
      _radius(s, w, name, a1, b1, out, cond: 'if $c');
      _radius(s, w, name, a2, b2, out, cond: 'else');
      return;
    }
    final m = RegExp(r'^BorderRadius\.(\w+)\(').firstMatch(s.blank.substring(a, b));
    if (m == null) return;
    final open = a + m.end - 1;
    final closeAt = s.close(open);
    for (final x in s.args(open, closeAt)) {
      final inner = RegExp(r'^Radius\.circular\(').firstMatch(s.blank.substring(x.start, x.end));
      if (inner != null) {
        final o = x.start + inner.end - 1;
        final c = s.close(o);
        final v = s.args(o, c);
        if (v.isNotEmpty)
          _value(s, w, x.name.isEmpty ? name : '$name.${x.name}', PropKind.number, v.first.start, v.first.end, out, cond: cond);
      } else if (x.name.isEmpty) {
        _value(s, w, name, PropKind.number, x.start, x.end, out, cond: cond);
      }
    }
  }

  // ------------------------------------------------------------------------------------------------------------- text style

  void _style(Src s, String w, int a, int b, List<Prop> out, String prefix, {String? cond, required int depth}) {
    if (depth > 3) return;
    final q = s.coalesce(a, b);
    if (q != null) {
      _style(s, w, s.trim(q + 2, b).$1, b, out, prefix, cond: cond, depth: depth + 1);
      return;
    }
    a = s.skipConst(a, b);
    final t = s.ternary(a, b);
    if (t != null) {
      final c = s.text.substring(a, t.$1).trim();
      final (a1, b1) = s.trim(t.$1 + 1, t.$2);
      final (a2, b2) = s.trim(t.$2 + 1, b);
      _style(s, w, a1, b1, out, prefix, cond: 'if $c', depth: depth + 1);
      _style(s, w, a2, b2, out, prefix, cond: 'else', depth: depth + 1);
      return;
    }
    final expr = s.blank.substring(a, b);
    if (RegExp(r'^[a-z_]\w*$').hasMatch(expr)) {
      final d = declaration(s, a, expr);
      if (d != null) {
        _style(s, w, d.$1, d.$2, out, prefix, cond: cond, depth: depth + 1);
      }
      return;
    }
    // `<base>.copyWith(…)`: the copyWith arguments override the base's
    final cw = RegExp(r'\.copyWith\(').allMatches(expr).lastOrNull;
    var baseEnd = b;
    final over = <String, Arg>{};
    int? cwOpen, cwClose;
    if (cw != null && s.close(a + cw.end - 1) == b - 1) {
      baseEnd = a + cw.start;
      cwOpen = a + cw.end - 1;
      cwClose = b - 1;
      for (final x in s.args(cwOpen, cwClose)) {
        if (x.name.isNotEmpty) over[x.name] = x;
      }
    }
    final m = RegExp(r'^((?:[A-Za-z_]\w*\.)?[A-Za-z_]\w*)\(').firstMatch(s.blank.substring(a, baseEnd));
    final fn = m?.group(1);
    final sig = fn == null ? null : _styleCalls[fn];
    if (m == null || sig == null) {
      out.add(
        Prop(
          widget: w,
          name: '${prefix}style',
          kind: PropKind.readOnly,
          origin: Origin.literal,
          file: s.file,
          start: a,
          end: b,
          text: s.text.substring(a, b),
          expr: s.text.substring(a, b),
          note: 'a text style this does not know how to take apart',
          condition: cond,
        ),
      );
      return;
    }
    final open = a + m.end - 1;
    final closeAt = s.close(open);
    final args = s.args(open, closeAt);
    final byProp = <String, Arg>{};
    var pos = 0;
    for (final x in args) {
      if (x.name.isEmpty) {
        if (pos < sig.positional.length) byProp[sig.positional[pos]] = x;
        pos++;
      } else if (sig.named.containsKey(x.name)) {
        byProp[sig.named[x.name]!] = x;
      }
    }
    for (final pName in _styleProps) {
      final kind = _styleKind[pName]!;
      final override = over[pName == 'fontWeight' ? 'fontWeight' : pName];
      if (override != null) {
        _value(s, w, '$prefix$pName', kind, override.start, override.end, out, cond: cond);
        continue;
      }
      final x = byProp[pName];
      if (x != null) {
        _value(s, w, '$prefix$pName', kind, x.start, x.end, out, cond: cond);
        continue;
      }
      if (pName == 'fontSize' && sig.sizeOf != null) {
        final tk = reg.tokens[sig.sizeOf];
        out.add(
          Prop(
            widget: w,
            name: '${prefix}fontSize',
            kind: PropKind.number,
            origin: Origin.token,
            file: s.file,
            start: cwOpen == null ? baseEnd : cwClose!,
            end: cwOpen == null ? baseEnd : cwClose!,
            text: tk == null ? '' : fmt(tk.value),
            expr: '',
            token: sig.sizeOf,
            sizeOf: fn,
            copyWith: true,
            copyExisting: cwOpen != null,
            condition: cond,
            insert: 'fontSize',
          ),
        );
        continue;
      }
      // not given here: added to the call when it takes the argument by name, else by .copyWith
      final named = sig.named.entries.where((e) => e.value == pName).map((e) => e.key).firstOrNull;
      final viaCopy = named == null;
      out.add(
        Prop(
          widget: w,
          name: '$prefix$pName',
          kind: kind,
          origin: Origin.unset,
          file: s.file,
          start: viaCopy ? (cwOpen == null ? baseEnd : cwClose!) : closeAt,
          end: viaCopy ? (cwOpen == null ? baseEnd : cwClose!) : closeAt,
          text: _styleDefault[pName] ?? '',
          expr: '',
          note: 'not set here',
          insert: viaCopy ? pName : named,
          copyWith: viaCopy,
          copyExisting: viaCopy && cwOpen != null,
          condition: cond,
          options: kind == PropKind.option ? [for (final m in reg.enums['FontWeight']!) 'FontWeight.$m'] : const [],
        ),
      );
    }
  }

  static const _styleDefault = {'height': '1.1', 'letterSpacing': '0', 'fontWeight': 'FontWeight.w400', 'color': 'N.g86', 'fontSize': '12'};

  // ------------------------------------------------------------------------------------------------------------- one value

  void _value(Src s, String w, String name, PropKind kind, int a0, int b0, List<Prop> out, {String? cond, String? via, int depth = 0}) {
    if (depth > 3) return;
    var (a, b) = s.trim(a0, b0);
    if (a >= b) return;
    final q = s.coalesce(a, b);
    if (q != null) {
      _value(s, w, name, kind, q + 2, b, out, cond: cond, via: via, depth: depth + 1);
      return;
    }
    a = s.skipConst(a, b);
    final text = s.text.substring(a, b);
    Prop p(Origin o, PropKind k, int st, int en, String t, {String? token, String? note, List<String> options = const []}) => Prop(
      widget: w,
      name: name,
      kind: k,
      origin: o,
      file: s.file,
      start: st,
      end: en,
      text: t,
      expr: s.text.substring(st, en),
      token: token,
      note: note,
      condition: cond,
      via: via,
      options: options,
    );
    final t = s.ternary(a, b);
    if (t != null) {
      final c = s.text.substring(a, t.$1).trim();
      _value(s, w, name, kind, t.$1 + 1, t.$2, out, cond: 'if $c', via: via, depth: depth + 1);
      _value(s, w, name, kind, t.$2 + 1, b, out, cond: 'else', via: via, depth: depth + 1);
      return;
    }
    switch (kind) {
      case PropKind.boolean:
        if (RegExp(r'^(true|false)$').hasMatch(text)) {
          out.add(p(Origin.literal, PropKind.boolean, a, b, text, options: const ['true', 'false']));
          return;
        }
      case PropKind.option:
        final m = RegExp(r'^([A-Z]\w*)\.(\w+)$').firstMatch(text);
        if (m != null && reg.enums.containsKey(m.group(1))) {
          out.add(p(Origin.literal, PropKind.option, a, b, text, options: [for (final v in reg.enums[m.group(1)]!) '${m.group(1)}.$v']));
          return;
        }
        final ctor = RegExp(r'^Alignment\(').firstMatch(s.blank.substring(a, b));
        if (ctor != null) {
          final open = a + ctor.end - 1;
          final xs = s.args(open, s.close(open));
          for (var i = 0; i < xs.length && i < 2; i++) {
            _value(
              s,
              w,
              '$name.${i == 0 ? 'x' : 'y'}',
              PropKind.number,
              xs[i].start,
              xs[i].end,
              out,
              cond: cond,
              via: via,
              depth: depth + 1,
            );
          }
          return;
        }
      case PropKind.color:
        if (reg.colors.containsKey(text) || RegExp(r'^(Surface|N|H)\.\w+$').hasMatch(text)) {
          out.add(p(Origin.token, PropKind.color, a, b, text, token: text, options: _colorOptions()));
          return;
        }
        final lit = RegExp(r'^Color\(0x([0-9A-Fa-f]{8})\)$').firstMatch(text);
        if (lit != null) {
          out.add(p(Origin.literal, PropKind.color, a, b, text));
          return;
        }
        final wv = RegExp(r'^(.*)\.(withValues|withOpacity|withAlpha)\(').firstMatch(s.blank.substring(a, b));
        if (wv != null) {
          final baseEnd = a + wv.group(1)!.length;
          _value(s, w, name, PropKind.color, a, baseEnd, out, cond: cond, via: via, depth: depth + 1);
          final open = a + wv.end - 1;
          final xs = s.args(open, s.close(open));
          if (xs.isNotEmpty)
            _value(s, w, '$name.opacity', PropKind.number, xs.first.start, xs.first.end, out, cond: cond, via: via, depth: depth + 1);
          return;
        }
      case PropKind.number:
        if (RegExp(r'^-?(\d+\.?\d*|\.\d+)$').hasMatch(text)) {
          out.add(p(Origin.literal, PropKind.number, a, b, text));
          return;
        }
        final px = RegExp(r'^Surface\.px\(\s*(-?\d+(?:\.\d+)?)\s*\)$').firstMatch(text);
        if (px != null) {
          final st = a + text.indexOf(px.group(1)!);
          out.add(p(Origin.exception, PropKind.number, st, st + px.group(1)!.length, px.group(1)!));
          return;
        }
        final tk = reg.tokens[text];
        if (tk != null && tk.sizeToken == null) {
          out.add(
            p(
              tk.isConst ? Origin.literal : Origin.token,
              tk.isConst ? PropKind.readOnly : PropKind.number,
              a,
              b,
              fmt(tk.value),
              token: text,
              note: tk.isConst ? 'const: needs a hot restart' : null,
            ),
          );
          return;
        }
      case PropKind.readOnly:
        break;
    }
    // `Class.member` declared elsewhere in the product: followed to its declaration
    final cm = RegExp(r'^([A-Z]\w*)\.([a-z_]\w*)$').firstMatch(text);
    if (cm != null) {
      final r = member(cm.group(1)!, cm.group(2)!);
      if (r != null) {
        _value(r.$1, w, name, kind, r.$2, r.$3, out, cond: cond, via: text, depth: depth + 1);
        return;
      }
    }
    // a name declared a few lines up: followed to its declaration, which is what is edited
    if (RegExp(r'^[a-z_]\w*$').hasMatch(text)) {
      final d = declaration(s, a, text);
      if (d != null) {
        _value(s, w, name, kind, d.$1, d.$2, out, cond: cond, via: text, depth: depth + 1);
        return;
      }
      out.add(p(Origin.literal, PropKind.readOnly, a, b, text, note: '`$text` is a parameter or field: its value is set by the caller'));
      return;
    }
    // a number made of parts (`edge + Surface.panelInset`): each part that is a number or a token
    if (kind == PropKind.number) {
      var any = false;
      final blank = s.blank.substring(a, b);
      final used = <int>{};
      for (final m in RegExp(r'Surface\.px\(\s*(-?\d+(?:\.\d+)?)\s*\)').allMatches(blank)) {
        final st = a + m.start + m.group(0)!.indexOf(m.group(1)!);
        out.add(p(Origin.exception, PropKind.number, st, st + m.group(1)!.length, m.group(1)!));
        for (var i = m.start; i < m.end; i++) {
          used.add(i);
        }
        any = true;
      }
      for (final m in RegExp(r'\b(?:Surface|Dn)\.\w+').allMatches(blank)) {
        if (used.contains(m.start)) continue;
        final tk = reg.tokens[m.group(0)!];
        if (tk != null && tk.sizeToken == null) {
          out.add(
            p(
              tk.isConst ? Origin.literal : Origin.token,
              tk.isConst ? PropKind.readOnly : PropKind.number,
              a + m.start,
              a + m.end,
              fmt(tk.value),
              token: m.group(0),
              note: tk.isConst ? 'const: needs a hot restart' : null,
            ),
          );
          for (var i = m.start; i < m.end; i++) {
            used.add(i);
          }
          any = true;
        }
      }
      for (final m in RegExp(r'(?<![\w.])(?:\d+\.?\d*|\.\d+)(?![\w.])').allMatches(blank)) {
        if (used.contains(m.start)) continue;
        out.add(p(Origin.literal, PropKind.number, a + m.start, a + m.end, m.group(0)!));
        any = true;
      }
      if (any) return;
    }
    out.add(p(Origin.literal, PropKind.readOnly, a, b, text, note: 'an expression this does not take apart'));
  }

  List<String> _colorOptions() => reg.colors.keys.where((k) => !k.startsWith('H.')).toList()..sort();
}

// ----------------------------------------------------------------------------------------------------------------- edits

/// The files an edit changes (a path -> its new text).
typedef Changes = Map<String, String>;

/// How a numeric value is edited: the token itself, or this place only.
enum Scope { token, instance }

String _insertNamed(Src s, int closeAt, String arg) {
  var i = closeAt;
  while (i > 0 && ' \n\t'.contains(s.blank[i - 1])) {
    i--;
  }
  final comma = i > 0 && s.blank[i - 1] == ',';
  final multiline = s.text.substring(i, closeAt).contains('\n');
  if (multiline && comma) {
    final lineStart = s.text.lastIndexOf('\n', i - 1) + 1;
    final indent = RegExp(r'^[ \t]*').firstMatch(s.text.substring(lineStart))!.group(0)!;
    return s.text.replaceRange(i, i, '\n$indent$arg,');
  }
  return s.text.replaceRange(i, i, comma ? ' $arg,' : ', $arg');
}

/// The text change that sets [p] to [value] (a number, `true`, `TextAlign.center`, `Color(0x…)`, `N.g20` …). [scope] only matters for a value that
/// is a canonical token: `token` rewrites the token's own declaration, `instance` replaces the reference here. Null when it cannot be written.
Changes? edit(Analyzer an, Prop p, String value, {Scope scope = Scope.instance}) {
  if (!p.editable) return null;
  final s = an.src(p.file);
  if (p.isToken && scope == Scope.token) return _editToken(an, p, value);
  if (p.copyWith && p.start == p.end) {
    return {
      p.file: p.copyExisting
          ? _insertNamed(s, p.start, '${p.insert}: $value')
          : s.text.replaceRange(p.start, p.start, '.copyWith(${p.insert}: $value)'),
    };
  }
  if (p.origin == Origin.unset) return {p.file: _insertNamed(s, p.start, '${p.insert}: $value')};
  if (p.start < 0 || p.end > s.text.length || s.text.substring(p.start, p.end) != p.expr) return null;
  return {p.file: s.text.replaceRange(p.start, p.end, value)};
}

Changes? _editToken(Analyzer an, Prop p, String value) {
  final tk = an.reg.tokens[p.token];
  if (p.kind == PropKind.color) {
    final cs = an.src('lib/theme/metrics.dart');
    final name = p.token!.split('.').last;
    final cls = p.token!.split('.').first;
    if (cls == 'Surface') {
      final m = RegExp('\\b$name = (N\\.g\\d+)').firstMatch(cs.text);
      if (m == null) return null;
      final st = m.start + m.group(0)!.indexOf(m.group(1)!);
      return {'lib/theme/metrics.dart': cs.text.replaceRange(st, st + m.group(1)!.length, value)};
    }
    final file = cls == 'N' ? 'lib/theme/neutral.dart' : (cls == 'H' ? 'lib/theme/identity.dart' : null);
    if (file == null) return null;
    final m = RegExp('static const $name = Color\\((0x[0-9A-Fa-f]{8})\\)').firstMatch(an.src(file).text);
    final hex = RegExp(r'0x[0-9A-Fa-f]{8}').firstMatch(value)?.group(0);
    if (m == null || hex == null) return null;
    final st = m.start + m.group(0)!.indexOf(m.group(1)!);
    return {file: an.src(file).text.replaceRange(st, st + 10, hex)};
  }
  if (tk == null) return null;
  final v = double.tryParse(value);
  if (v == null) return null;
  final text = an.src(tk.file).text;
  final out = rewriteToken(text, tk, v);
  return out == null ? null : {tk.file: out};
}

/// The text change that takes the argument [p] out of its call (a property this session added): its `name: value`, its comma, its line, and
/// a `.copyWith()` that is left empty. Null when [p] is not a named argument.
Changes? removeProp(Analyzer an, Prop p) {
  final s = an.src(p.file);
  var a = p.start;
  while (a > 0 && ' \n\t'.contains(s.blank[a - 1])) {
    a--;
  }
  if (a == 0 || s.blank[a - 1] != ':') return null;
  a--;
  while (a > 0 && ' \n\t'.contains(s.blank[a - 1])) {
    a--;
  }
  final nameEnd = a;
  while (a > 0 && RegExp(r'\w').hasMatch(s.blank[a - 1])) {
    a--;
  }
  if (a == nameEnd) return null;
  var b = p.end;
  var text = s.text;
  var i = b;
  while (i < s.blank.length && ' \t'.contains(s.blank[i])) {
    i++;
  }
  int from = a, to = b;
  if (i < s.blank.length && s.blank[i] == ',') {
    to = i + 1;
    // the whole line when the argument is alone on it
    final ls = text.lastIndexOf('\n', a - 1) + 1;
    final le = text.indexOf('\n', to);
    if (text.substring(ls, a).trim().isEmpty && le > 0 && text.substring(to, le).trim().isEmpty) {
      from = ls;
      to = le + 1;
    } else {
      while (to < text.length && ' \t'.contains(text[to])) {
        to++;
      }
    }
  } else {
    // the last argument: take the comma before it
    var k = a;
    while (k > 0 && ' \n\t'.contains(s.blank[k - 1])) {
      k--;
    }
    if (k > 0 && s.blank[k - 1] == ',') from = k - 1;
  }
  text = text.replaceRange(from, to, '');
  final empty = RegExp(r'\.copyWith\(\s*\)').firstMatch(text.substring(from > 12 ? from - 12 : 0));
  if (empty != null) {
    final base = from > 12 ? from - 12 : 0;
    text = text.replaceRange(base + empty.start, base + empty.end, '');
  }
  return {p.file: text};
}

/// One visual / layout argument of one widget across the product: how often it is written and whether Design Mode can edit it.
class CoverageRow {
  CoverageRow(this.widget, this.arg);
  final String widget, arg;
  int uses = 0, editable = 0, readOnly = 0, unsupported = 0;
}

const _notVisual = {
  'child',
  'children',
  'key',
  'builder',
  'itemBuilder',
  'listenable',
  'painter',
  'foregroundPainter',
  'semanticsLabel',
  'cursor',
  'debugLabel',
  'autofocus',
  'focusNode',
  'controller',
  'scrollController',
  'physics',
  'value',
  'text',
  'data',
  'icon',
  'name',
  'id',
  'label',
  'tooltip',
  'duration',
  'curve',
  'animation',
  'vsync',
  'restorationId',
  'semanticLabel',
  'excludeFromSemantics',
  'canRequestFocus',
  'skipTraversal',
  'descendantsAreFocusable',
  'descendantsAreTraversable',
  'includeSemantics',
  'maintainState',
  'group',
  'blendMode',
  'isAntiAlias',
  'fontFamily',
  'package',
  'milliseconds',
  'seconds',
  'microseconds',
};

/// Constructors whose arguments are already taken apart as part of the argument that holds them (`padding: EdgeInsets.…`), or that are
/// coordinates and data, not the look of a widget.
const _valueTypes = {
  'EdgeInsets',
  'EdgeInsetsDirectional',
  'BorderRadius',
  'Radius',
  'BorderSide',
  'Border',
  'BoxDecoration',
  'BoxConstraints',
  'TextStyle',
  'Alignment',
  'Offset',
  'Size',
  'Rect',
  'RRect',
  'Tx',
  'PanelDef',
  'Fam',
  'Duration',
  'IconData',
  'Color',
  'ColorFilter',
  'Paint',
  'Path',
  'Matrix4',
  'Curve',
  'Random',
  'DateTime',
  'Uri',
  'File',
  'TextPainter',
  'TextSpan',
  'RegExp',
  'ValueKey',
  'GlobalKey',
  'UniqueKey',
  'ObjectKey',
  'Key',
};

/// Arguments that configure behaviour, data or accessibility, not how something looks or is laid out: counted by the census, but
/// reported apart from VISUAL / LAYOUT coverage.
const nonVisualArgs = {
  'itemCount',
  'gaplessPlayback',
  'dragStartBehavior',
  'enabled',
  'button',
  'selected',
  'actions',
  'ignoring',
  'errorBuilder',
  'maximizableItem',
  'maximizableTab',
  'maximizableTabsArea',
  'layout',
  'supportedDevices',
  'overlayChildBuilder',
  'baseOffset',
  'extentOffset',
  'initialScrollOffset',
  'alignmentPolicy',
  'constrained',
  'keyboardType',
  'cacheWidth',
  'cacheHeight',
  'readOnly',
  'obscureText',
  'textInputAction',
  'inputFormatters',
  'selectionControls',
  'onGenerateRoute',
  'hitTestBehavior',
  'behavior',
  'excludeSemantics',
  'container',
  'header',
  'liveRegion',
  'checked',
  'toggled',
  'focusable',
  'focused',
  'textStyle',
  'scrollDirection',
  'reverse',
  'primary',
};

/// The visual / layout arguments written in [files] (file -> text), by widget and name, and what Design Mode can do with each.
List<CoverageRow> census(
  Analyzer an,
  Iterable<String> files, {
  void Function(String widget, String arg, String where, String what)? sample,
}) {
  final rows = <String, CoverageRow>{};
  final list = files.toList();
  // the product's own widgets take product arguments (a title, a kind, a layer): those are not the look of a Flutter widget
  final own = {
    for (final f in list)
      ...RegExp(r'^(?:abstract |final |sealed )*class (\w+)', multiLine: true).allMatches(an.src(f).blank).map((m) => m.group(1)!),
  };
  for (final f in list) {
    final s = an.src(f);
    for (final m in RegExp(r'\b([A-Z]\w*)(?:\.\w+)?\(').allMatches(s.blank)) {
      if (_valueTypes.contains(m.group(1)) || own.contains(m.group(1))) continue;
      final open = m.end - 1;
      final closeAt = s.close(open);
      final args = s
          .args(open, closeAt)
          .where((a) => a.name.isNotEmpty && !_notVisual.contains(a.name) && !a.name.startsWith('on'))
          .toList();
      if (args.isEmpty) continue;
      final line = s.lineOf(m.start);
      final col = m.start - (s.text.lastIndexOf('\n', m.start > 0 ? m.start - 1 : 0) + 1) + 1;
      List<Prop> props;
      try {
        props = an.analyze(f, line, col);
      } catch (_) {
        props = const [];
      }
      for (final a in args) {
        final r = rows.putIfAbsent('${m.group(1)}.${a.name}', () => CoverageRow(m.group(1)!, a.name));
        r.uses++;
        final mine = props
            .where(
              (p) => a.name == 'style'
                  ? _styleProps.contains(p.name.split('.').last) || p.name == 'style'
                  : (p.name == a.name || p.name.startsWith('${a.name}.') || p.name.startsWith('${a.name}[')),
            )
            .toList();
        if (sample != null)
          sample(
            m.group(1)!,
            a.name,
            '$f:$line',
            mine.isEmpty
                ? 'UNSUPPORTED  ${s.text.substring(a.start, a.end).replaceAll(RegExp(r'\s+'), ' ')}'
                : (mine.any((p) => p.editable) ? 'ok' : 'READ-ONLY  ${mine.first.note ?? ''}  ${mine.first.expr}'),
          );
        if (mine.isEmpty) {
          r.unsupported++;
        } else if (mine.any((p) => p.editable)) {
          r.editable++;
        } else {
          r.readOnly++;
        }
      }
    }
  }
  return rows.values.toList()..sort((a, b) => b.uses.compareTo(a.uses));
}
