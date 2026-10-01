// UI inspection for a running debug Motolii (scripts/motolii-ui.sh dev): click a widget in the real window, read where its size
// comes from, change the value, see it at once. The picking, the widget tree, the source locations and the layout numbers are
// Flutter's own Inspector (the VM service's ext.flutter.inspector.*, the same calls DevTools makes); this file adds only what
// Flutter cannot know: which Surface/Dn token (or literal) a value is, how many places use it, and writing a new value into the
// source so the existing hot reload (scripts/motolii-ui.sh reload) puts it on screen.
//
//   dart run tool/ui_inspect.dart pick            turn on Select Widget mode, wait for a click, print the card, turn it off
//   dart run tool/ui_inspect.dart watch           the same, every click, until Ctrl-C
//   dart run tool/ui_inspect.dart show            the card of what is selected now
//   dart run tool/ui_inspect.dart tokens [text]   the Surface/Dn tokens, value, usage count
//   dart run tool/ui_inspect.dart set Surface.workRow 19      a token's value at 100 %, written to metrics.dart, hot reloaded
//   dart run tool/ui_inspect.dart edit lib/x.dart:244 13=11.5 a literal on one line, written, hot reloaded
//   dart run tool/ui_inspect.dart changed | reset              what this session changed | put it all back
//   dart run tool/ui_inspect.dart scale 78                     the UI Scale, whole percent, in the running window
import 'dart:async';
import 'dart:convert';
import 'dart:io';

/// A named dimension in lib/theme/metrics.dart: `Surface.workRow` is 20 at 100 %.
class Token {
  Token(this.name, this.value, this.file, this.line, this.text);
  final String name, file, text;
  final double value;
  final int line;
}

final _classRe = RegExp(r'^abstract final class (\w+)');
final _getterRe = RegExp(r'static \w+ get (\w+) => (.*?);\s*(//.*)?$');
final _constRe = RegExp(r'static const ([^;]*);');
final _numRe = RegExp(r'-?\d+(?:\.\d+)?');

/// The tokens a metrics.dart source declares, by `Class.name`. A getter's value is the first number in its expression
/// (`px(20)`, `UiScale.derive(24, ...)`); `static const a = 1, b = 2;` declares each.
Map<String, Token> parseTokens(
  String source, {
  String file = 'lib/theme/metrics.dart',
}) {
  final out = <String, Token>{};
  String? cls;
  final lines = source.split('\n');
  for (var i = 0; i < lines.length; i++) {
    final l = lines[i];
    final c = _classRe.firstMatch(l);
    if (c != null) cls = c.group(1);
    if (cls == null) continue;
    final g = _getterRe.firstMatch(l);
    if (g != null) {
      final n = _numRe.firstMatch(g.group(2)!);
      if (n != null)
        out['$cls.${g.group(1)}'] = Token(
          '$cls.${g.group(1)}',
          double.parse(n.group(0)!),
          file,
          i + 1,
          l.trim(),
        );
      continue;
    }
    final k = _constRe.firstMatch(l);
    if (k != null) {
      for (final part in k.group(1)!.split(',')) {
        final m = RegExp(r'^\s*(\w+)\s*=\s*(-?\d+(?:\.\d+)?)\s*$')
            .firstMatch(part);
        if (m != null)
          out['$cls.${m.group(1)}'] = Token(
            '$cls.${m.group(1)}',
            double.parse(m.group(2)!),
            file,
            i + 1,
            l.trim(),
          );
      }
    }
  }
  return out;
}

String _fmt(double v) => v == v.roundToDouble() ? v.toStringAsFixed(0) : '$v';

/// [source] with the declaration of [token] set to [value]: the first number of its getter expression, or its `name = n` in a const.
/// Null when the line is not in a form this knows how to write.
String? rewriteToken(String source, Token token, double value) {
  final lines = source.split('\n');
  final l = lines[token.line - 1];
  final short = token.name.split('.').last;
  String? replaced;
  if (_getterRe.hasMatch(l)) {
    final eq = l.indexOf('=>');
    final n = _numRe.firstMatch(l.substring(eq));
    if (n != null)
      replaced = l.replaceRange(eq + n.start, eq + n.end, _fmt(value));
  } else {
    final m = RegExp('\\b$short\\s*=\\s*(-?\\d+(?:\\.\\d+)?)').firstMatch(l);
    if (m != null)
      replaced = l.replaceRange(
        m.start + m.group(0)!.indexOf(m.group(1)!),
        m.end,
        _fmt(value),
      );
  }
  if (replaced == null) return null;
  lines[token.line - 1] = replaced;
  return lines.join('\n');
}

/// [source] with the number [from] on [line] replaced by [to] (the first one, whole number only).
String? rewriteLiteral(String source, int line, String from, String to) {
  final lines = source.split('\n');
  if (line < 1 || line > lines.length) return null;
  final m = RegExp('(?<![\\w.])${RegExp.escape(from)}(?![\\w.])')
      .firstMatch(lines[line - 1]);
  if (m == null) return null;
  lines[line - 1] = lines[line - 1].replaceRange(m.start, m.end, to);
  return lines.join('\n');
}

/// What one widget's own arguments are made of: the arguments of the constructor call at [line]:[col] (1-based) without its
/// child / children, split into tokens (`Surface.sectionGap`), exceptions (`Surface.px(7)`) and raw numbers.
class Provenance {
  Provenance(this.tokens, this.px, this.literals, {this.hasArgs = false});
  final List<String> tokens, px, literals;

  /// The call has arguments of its own (other than its child): when none is a token or a number, they are locals or computed.
  final bool hasArgs;
}

/// [source] with comments and the inside of string literals blanked (same length, same lines): brackets in them do not count.
String blankCommentsAndStrings(String s) {
  final out = StringBuffer();
  var i = 0;
  while (i < s.length) {
    final c = s[i];
    if (s.startsWith('//', i)) {
      while (i < s.length && s[i] != '\n') {
        out.write(' ');
        i++;
      }
    } else if (s.startsWith('/*', i)) {
      while (i < s.length && !s.startsWith('*/', i)) {
        out.write(s[i] == '\n' ? '\n' : ' ');
        i++;
      }
      if (i < s.length) {
        out.write('  ');
        i += 2;
      }
    } else if (c == "'" || c == '"') {
      out.write(c);
      i++;
      while (i < s.length && s[i] != c && s[i] != '\n') {
        if (s[i] == '\\' && i + 1 < s.length) {
          out.write('  ');
          i += 2;
        } else {
          out.write(' ');
          i++;
        }
      }
      if (i < s.length && s[i] == c) {
        out.write(c);
        i++;
      }
    } else {
      out.write(c);
      i++;
    }
  }
  return out.toString();
}

Provenance provenanceAt(String original, int line, int col) {
  final source = blankCommentsAndStrings(original);
  final lines = source.split('\n');
  var off = 0;
  for (var i = 0; i < line - 1 && i < lines.length; i++) {
    off += lines[i].length + 1;
  }
  off += col - 1;
  final open = source.indexOf('(', off);
  if (open < 0) return Provenance([], [], []);
  var depth = 0, i = open, end = source.length;
  for (; i < source.length; i++) {
    final c = source[i];
    if (c == '(' || c == '[' || c == '{') depth++;
    if (c == ')' || c == ']' || c == '}') {
      depth--;
      if (depth == 0) {
        end = i;
        break;
      }
    }
  }
  final body = source.substring(open + 1, end);
  // split at depth 0 commas; drop the arguments that are other widgets
  final args = <String>[];
  var d = 0, start = 0;
  for (var k = 0; k < body.length; k++) {
    final c = body[k];
    if (c == '(' || c == '[' || c == '{') d++;
    if (c == ')' || c == ']' || c == '}') d--;
    if (c == ',' && d == 0) {
      args.add(body.substring(start, k));
      start = k + 1;
    }
  }
  args.add(body.substring(start));
  final own = args
      .where(
        (a) => !RegExp(
          r'^\s*(child|children|builder|itemBuilder|key|onTap|onPressed)\s*:',
        ).hasMatch(a),
      )
      .join(', ')
      .trim();
  final px = RegExp(r'Surface\.px\(\s*(-?\d+(?:\.\d+)?)\s*\)')
      .allMatches(own)
      .map((m) => 'Surface.px(${m.group(1)})')
      .toList();
  final tokens = RegExp(r'\b(?:Surface|Dn)\.(?!px\b|fixed\b)\w+')
      .allMatches(own)
      .map((m) => m.group(0)!)
      .toSet()
      .toList();
  final stripped = own
      .replaceAll(RegExp(r'Surface\.px\([^)]*\)'), '')
      .replaceAll(RegExp(r"'[^']*'"), '');
  final literals = RegExp(r'(?<![\w.])\d+(?:\.\d+)?(?![\w.])')
      .allMatches(stripped)
      .map((m) => m.group(0)!)
      .toList();
  return Provenance(tokens, px, literals, hasArgs: own.isNotEmpty);
}

/// `file:///…/lib/a/b.dart` -> `lib/a/b.dart`; null for the framework and packages.
String? projectPath(String? uri) {
  if (uri == null) return null;
  final m = RegExp(r'/motolii/ui/(lib/.*)$').firstMatch(uri);
  return m?.group(1);
}

// ---------------------------------------------------------------------------------------------------------------- the running app

class Vm {
  Vm._(this.ws) {
    ws.listen((m) {
      final j = jsonDecode(m as String);
      if (j is Map && j['id'] != null) _pending.remove(j['id'])?.complete(j);
    });
  }
  final WebSocket ws;
  late final String isolate;
  int _id = 0;
  final _pending = <int, Completer<Map>>{};

  static Future<Vm> connect(String uri) async {
    final vm = Vm._(await WebSocket.connect(uri));
    final isolates = ((await vm.call('getVM'))['isolates'] as List).cast<Map>();
    vm.isolate =
        (isolates.firstWhere(
              (i) => i['name'] == 'main',
              orElse: () => isolates.first,
            ))['id']
            as String;
    return vm;
  }

  Future<Map> call(String method, [Map<String, dynamic>? params]) async {
    final c = Completer<Map>();
    final id = ++_id;
    _pending[id] = c;
    ws.add(
      jsonEncode({
        'jsonrpc': '2.0',
        'id': id,
        'method': method,
        'params': params ?? {},
      }),
    );
    final r = await c.future;
    if (r['error'] != null)
      throw StateError(
        '$method: ${jsonEncode((r['error'] as Map)['data'] ?? (r['error'] as Map)['message'])}',
      );
    return (r['result'] as Map?) ?? {};
  }

  /// One ext.flutter.inspector.* call (Flutter's own Inspector service); the answer's `result`.
  Future<dynamic> ext(
    String method, [
    Map<String, dynamic> params = const {},
  ]) async {
    final r = await call('ext.flutter.inspector.$method', {
      ...params,
      'isolateId': isolate,
    });
    return r.containsKey('result') ? r['result'] : r;
  }

  Future<void> close() => ws.close();
}

// ---------------------------------------------------------------------------------------------------------------- the card

final _root = File.fromUri(Platform.script)
    .parent
    .parent
    .path; // .../motolii/ui
final _state =
    '${Platform.environment['XDG_STATE_HOME'] ?? '${Platform.environment['HOME']}/.local/state'}/motolii-stage5';

Map<String, Token> _tokens() =>
    parseTokens(File('$_root/lib/theme/metrics.dart').readAsStringSync());

/// How many places use [name] (`Surface.workRow`) in lib/, and in how many files.
(int, int) usage(String name) {
  final re = RegExp('\\b${RegExp.escape(name)}\\b');
  var n = 0, files = 0;
  for (final f
      in Directory('$_root/lib')
          .listSync(recursive: true)
          .whereType<File>()
          .where((f) => f.path.endsWith('.dart'))) {
    final c = re.allMatches(f.readAsStringSync()).length;
    if (c > 0) {
      n += c;
      files++;
    }
  }
  return (n, files);
}

const _geometry = {
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
const _interesting = {
  'padding',
  'margin',
  'constraints',
  'size',
  'alignment',
  'width',
  'height',
  'flex',
  'fit',
  'family',
  'weight',
  'letterSpacing',
  'color',
  'bg',
  'fg',
  'mainAxisAlignment',
  'crossAxisAlignment',
  'mainAxisSize',
  'spacing',
  'borderRadius',
  'left',
  'right',
  'top',
  'bottom',
  'clipBehavior',
  'behavior',
};

String _loc(Map? loc) => loc == null
    ? ''
    : '${projectPath(loc['file'] as String?) ?? loc['file']}:${loc['line']}:${loc['column']}';

Future<void> card(Vm vm, {String group = 'ui_inspect'}) async {
  final sel = await vm.ext('getSelectedSummaryWidget', {'objectGroup': group});
  if (sel is! Map || sel.isEmpty) {
    stdout.writeln(
      'nothing selected: `pick` turns on Select Widget mode, then click the window.',
    );
    return;
  }
  final reg = _tokens();
  final chain =
      ((await vm.ext('getParentChain', {
            'objectGroup': group,
            'arg': sel['valueId'],
          })) as List)
          .map((e) => (e as Map)['node'] as Map)
          .toList()
          .reversed
          .toList(); // nearest first, [0] is the selected widget itself
  final shown = <Map>[
    sel,
    ...chain
        .skip(1)
        .where(
          (n) =>
              projectPath(
                    (n['creationLocation'] as Map?)?['file'] as String?,
                  ) !=
                  null &&
              _geometry.contains(_typeOf(n)),
        )
        .take(7),
  ];
  stdout.writeln('');
  for (final n in shown) {
    final id = n['valueId'];
    final loc = n['creationLocation'] as Map?;
    final path = projectPath(loc?['file'] as String?);
    final props = <String, String>{};
    try {
      final det = await vm.ext('getDetailsSubtree', {
        'objectGroup': group,
        'arg': id,
        'subtreeDepth': '0',
      }) as Map;
      for (final p in (det['properties'] as List? ?? const []).cast<Map>()) {
        final name = p['name'] as String?;
        if (name != null &&
            _interesting.contains(name) &&
            p['description'] != null &&
            !props.containsKey(name))
          props[name] = '${p['description']}';
      }
      final lay = await vm.ext('getLayoutExplorerNode', {
        'objectGroup': group,
        'id': id,
        'subtreeDepth': '0',
      }) as Map;
      for (final p
          in ((lay['renderObject'] as Map?)?['properties'] as List? ?? const [])
              .cast<Map>()) {
        final name = p['name'] as String?;
        if (name == 'size' || name == 'constraints')
          props['render.$name'] = '${p['description']}';
      }
    } on StateError {
      // a node that is gone from the tree: the card says what it has
    }
    stdout.writeln(
      '${n == sel ? '▶' : ' '} ${_typeOf(n).padRight(16)} ${_loc(loc)}',
    );
    for (final e in props.entries) {
      stdout.writeln(
        '      ${e.key.padRight(14)} ${e.value.length > 110 ? '${e.value.substring(0, 107)}...' : e.value}',
      );
    }
    if (path != null && loc != null) {
      final src = File('$_root/$path').readAsStringSync();
      final pv = provenanceAt(src, loc['line'] as int, loc['column'] as int);
      for (final t in pv.tokens) {
        final tk = reg[t];
        if (tk != null) {
          final (c, f) = usage(t);
          stdout.writeln(
            '      ◆ $t = ${_fmt(tk.value)}  (canonical token, $c uses in $f files, ${tk.file}:${tk.line})',
          );
        } else {
          stdout.writeln(
            '      ◆ $t  (named, value not a number in metrics.dart: a colour, a style, a computed value)',
          );
        }
      }
      for (final x in pv.px) {
        stdout.writeln(
          '      ◇ $x  (registered exception: a SCALE-policy custom dimension)',
        );
      }
      if (pv.literals.isNotEmpty)
        stdout.writeln(
          '      ✕ raw number${pv.literals.length > 1 ? 's' : ''} ${pv.literals.join(', ')}  (literal in the source)',
        );
      if (pv.tokens.isEmpty && pv.px.isEmpty && pv.literals.isEmpty)
        stdout.writeln(
          '      · no size of its own here: constraints and children decide it',
        );
    }
    stdout.writeln('');
  }
}

String _typeOf(Map n) =>
    ('${n['description']}').split(RegExp(r'[-<(\s]')).first;

// ---------------------------------------------------------------------------------------------------------------- edits and reload

File get _journal => File('$_state/inspect-edits.json');

List<Map> _readJournal() => _journal.existsSync()
    ? (jsonDecode(_journal.readAsStringSync()) as List).cast<Map>()
    : [];

void _writeJournal(List<Map> j) {
  Directory(_state).createSync(recursive: true);
  _journal.writeAsStringSync(jsonEncode(j));
}

/// Writes one line change and records it (`before`, `after`) so `reset` can put it back.
bool _apply(String rel, String Function(String) change, String what) {
  final f = File('$_root/$rel');
  final before = f.readAsStringSync();
  final after = change(before);
  if (after == before) {
    stderr.writeln('nothing to change for $what');
    return false;
  }
  f.writeAsStringSync(after);
  final j = _readJournal()
    ..add({
      'file': rel,
      'what': what,
      'before': before,
      'at': DateTime.now().toIso8601String(),
    });
  _writeJournal(j);
  return true;
}

/// How long a hot reload takes to land on screen (the flutter tool answers a signal, not a request).
const _reloadWait = Duration(milliseconds: 1800);

Future<void> reload() async {
  final pid = File('$_state/flutter.pid');
  if (!pid.existsSync()) {
    stderr.writeln('No dev session: start one with scripts/motolii-ui.sh dev');
    exit(1);
  }
  Process.runSync('kill', ['-USR1', pid.readAsStringSync().trim()]);
  await Future<void>.delayed(_reloadWait);
}

Future<Vm> _vm() async {
  final f = File('$_state/vmservice');
  if (!f.existsSync()) {
    stderr.writeln('No dev session: start one with scripts/motolii-ui.sh dev');
    exit(1);
  }
  var uri = f.readAsStringSync().trim().replaceFirst('http://', 'ws://');
  if (!uri.endsWith('/ws')) uri = '${uri.endsWith('/') ? uri : '$uri/'}ws';
  return Vm.connect(uri);
}

Future<void> main(List<String> a) async {
  final cmd = a.isEmpty ? 'watch' : a.first;
  switch (cmd) {
    case 'tokens':
      final q = a.length > 1 ? a[1].toLowerCase() : '';
      for (final t in _tokens().values.where(
        (t) => t.name.toLowerCase().contains(q),
      )) {
        final (c, f) = usage(t.name);
        stdout.writeln(
          '${t.name.padRight(26)} ${_fmt(t.value).padLeft(6)}   $c uses in $f files   ${t.file}:${t.line}',
        );
      }
    case 'show':
      final vm = await _vm();
      await card(vm);
      await vm.close();
    case 'pick' || 'watch':
      final vm = await _vm();
      await vm.ext('show', {'enabled': 'true'});
      stdout.writeln(
        'Select Widget mode is on: click a thing in the Motolii window${cmd == 'watch' ? ' (Ctrl-C to stop)' : ''}.',
      );
      String? last;
      Future<String?> current() async {
        final s = await vm.ext('getSelectedSummaryWidget', {
          'objectGroup': 'ui_inspect_poll',
        });
        return s is Map && s.isNotEmpty ? '${s['valueId']}' : null;
      }
      last = await current();
      while (true) {
        await Future<void>.delayed(const Duration(milliseconds: 250));
        final now = await current();
        if (now != null && now != last) {
          last = now;
          await card(vm);
          if (cmd == 'pick') break;
        }
      }
      await vm.ext('show', {'enabled': 'false'});
      await vm.close();
    case 'set':
      if (a.length < 3) {
        stderr.writeln('usage: set Surface.workRow 19');
        exit(2);
      }
      final tk = _tokens()[a[1]];
      final v = double.tryParse(a[2]);
      if (tk == null || v == null) {
        stderr.writeln(
          'no such token (or value is not a number): ${a[1]}; `tokens` lists them',
        );
        exit(2);
      }
      if (_apply(
        tk.file,
        (s) => rewriteToken(s, tk, v) ?? s,
        '${tk.name} ${_fmt(tk.value)} -> ${_fmt(v)}',
      )) {
        await reload();
        final (c, f) = usage(tk.name);
        stdout.writeln(
          '${tk.name}: ${_fmt(tk.value)} -> ${_fmt(v)} (at 100 %; $c uses in $f files), hot reloaded.',
        );
      }
    case 'edit':
      final m = a.length < 3 ? null : RegExp(r'^(.+):(\d+)$').firstMatch(a[1]);
      final ft = a.length < 3 ? null : a[2].split('=');
      if (m == null || ft == null || ft.length != 2) {
        stderr.writeln(
          'usage: edit lib/inspector/value_controls.dart:244 13=11.5',
        );
        exit(2);
      }
      final line = int.parse(m.group(2)!);
      if (_apply(
        m.group(1)!,
        (s) => rewriteLiteral(s, line, ft[0], ft[1]) ?? s,
        '${m.group(1)}:$line ${ft[0]} -> ${ft[1]}',
      )) {
        await reload();
        stdout.writeln(
          '${m.group(1)}:$line  ${ft[0]} -> ${ft[1]}, hot reloaded.',
        );
      }
    case 'changed':
      final j = _readJournal();
      if (j.isEmpty) stdout.writeln('nothing changed in this session.');
      for (final e in j) {
        stdout.writeln('${e['at']}  ${e['what']}');
      }
    case 'reset':
      final j = _readJournal();
      if (j.isEmpty) {
        stdout.writeln('nothing to reset.');
        break;
      }
      final first = <String, String>{}; // a file's oldest recorded content is what it was before the session touched it
      for (final e in j) {
        first.putIfAbsent(e['file'] as String, () => e['before'] as String);
      }
      first.forEach((rel, text) => File('$_root/$rel').writeAsStringSync(text));
      _writeJournal([]);
      await reload();
      stdout.writeln(
        'put back ${first.length} file${first.length == 1 ? '' : 's'}, hot reloaded.',
      );
    case 'scale':
      final pct = a.length > 1 ? int.tryParse(a[1]) : null;
      if (pct == null) {
        stderr.writeln('usage: scale 78');
        exit(2);
      }
      final vm = await _vm();
      final iso = await vm.call('getIsolate', {'isolateId': vm.isolate});
      final lib = (iso['libraries'] as List).cast<Map>().firstWhere(
        (l) => '${l['uri']}'.endsWith('app/ui_scale.dart'),
      );
      final r = await vm.call('evaluate', {
        'isolateId': vm.isolate,
        'targetId': lib['id'],
        'expression': 'LiveUiScale.instance.set($pct, keep: false)',
      });
      stdout.writeln(
        'UI Scale -> $pct % (${r['valueAsString']}: true when it changed; not saved).',
      );
      await vm.close();
    default:
      stderr.writeln(
        'usage: ui_inspect.dart pick|watch|show|tokens [text]|set <Class.token> <value>|edit <file:line> <from>=<to>|changed|reset|scale <pct>',
      );
      exit(2);
  }
  exit(0);
}
