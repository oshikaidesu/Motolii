// The part of Design Mode that is only text: which named dimension a value in the source is, what a widget's own arguments are made
// of, the one-line source rewrites, and the edit session (history, before/after, discard). Plain Dart: no Flutter, no I/O of its own
// (files come in through [DesignSession.read] / [write]), so the tool/ui_inspect.dart CLI, the in-window HUD (design_mode.dart) and the
// tests share it. Debug builds only: nothing in the release app imports this.

/// A named dimension in lib/theme/metrics.dart: `Surface.workRow` is 20 at 100 %.
class Token {
  Token(this.name, this.value, this.file, this.line, this.text, {this.isConst = false, this.sizeToken});
  final String name, file, text;
  final double value;
  final int line;

  /// `static const`: a hot reload does not re-fold a constant into the code that uses it, so the HUD shows it read-only.
  final bool isConst;

  /// A text style (`Dn.name`): the size token it is built from (`Dn.nameSize`).
  final String? sizeToken;
}

final _classRe = RegExp(r'^abstract final class (\w+)');
final _getterRe = RegExp(r'static \w+ get (\w+) => (.*?);\s*(//.*)?$');
final _styleRe = RegExp(r"static TextStyle (\w+)\(.*\) => _t\('[^']*', (\w+),");
final _constRe = RegExp(r'static const ([^;]*);');
final _numRe = RegExp(r'-?\d+(?:\.\d+)?');

/// The tokens a metrics.dart source declares, by `Class.name`. A getter's value is the first number in its expression
/// (`px(20)`, `UiScale.derive(24, ...)`); `static const a = 1, b = 2;` declares each; a `TextStyle` method points at its size token.
Map<String, Token> parseTokens(String source, {String file = 'lib/theme/metrics.dart'}) {
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
      if (n != null) out['$cls.${g.group(1)}'] = Token('$cls.${g.group(1)}', double.parse(n.group(0)!), file, i + 1, l.trim());
      continue;
    }
    final s = _styleRe.firstMatch(l);
    if (s != null) {
      final size = out['$cls.${s.group(2)}'];
      out['$cls.${s.group(1)}'] = Token(
        '$cls.${s.group(1)}',
        size?.value ?? 0,
        file,
        i + 1,
        l.trim(),
        sizeToken: size == null ? null : '$cls.${s.group(2)}',
      );
      continue;
    }
    final k = _constRe.firstMatch(l);
    if (k != null) {
      for (final part in k.group(1)!.split(',')) {
        final m = RegExp(r'^\s*(\w+)\s*=\s*(-?\d+(?:\.\d+)?)\s*$').firstMatch(part);
        if (m != null)
          out['$cls.${m.group(1)}'] = Token('$cls.${m.group(1)}', double.parse(m.group(2)!), file, i + 1, l.trim(), isConst: true);
      }
    }
  }
  return out;
}

String fmt(double v) => v == v.roundToDouble() ? v.toStringAsFixed(0) : '${(v * 1000).round() / 1000}';

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
    if (n != null) replaced = l.replaceRange(eq + n.start, eq + n.end, fmt(value));
  } else {
    final m = RegExp('\\b$short\\s*=\\s*(-?\\d+(?:\\.\\d+)?)').firstMatch(l);
    if (m != null) replaced = l.replaceRange(m.start + m.group(0)!.indexOf(m.group(1)!), m.end, fmt(value));
  }
  if (replaced == null) return null;
  lines[token.line - 1] = replaced;
  return lines.join('\n');
}

/// [source] with the number [from] on [line] replaced by [to] (the first one, whole number only).
String? rewriteLiteral(String source, int line, String from, String to) {
  final lines = source.split('\n');
  if (line < 1 || line > lines.length) return null;
  final m = RegExp('(?<![\\w.])${RegExp.escape(from)}(?![\\w.])').firstMatch(lines[line - 1]);
  if (m == null) return null;
  lines[line - 1] = lines[line - 1].replaceRange(m.start, m.end, to);
  return lines.join('\n');
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

enum RefKind {
  /// A canonical token (`Surface.workRow`): editing it changes every place that uses it.
  token,

  /// A token that is a colour, a style or another non-number (`Surface.ink`, `Dn.name`).
  named,

  /// `Surface.px(7)`: a registered custom dimension.
  exception,

  /// A number written in the call.
  raw,
}

/// One thing a widget's own argument is made of. A token has no place of its own to edit (its declaration is the place);
/// an exception or a raw number is the number at [start]..[end] of the file.
class Ref {
  Ref(this.arg, this.kind, this.text, {this.value, this.start = -1, this.end = -1});
  final String arg;
  final RefKind kind;
  final String text;
  final double? value;
  int start, end;
}

/// What the call at [line]:[col] (1-based) is made of, argument by argument, without its child / children. Only the widget's own
/// arguments: another widget passed as an argument is not read.
List<Ref> refsAt(String original, int line, int col) {
  final source = blankCommentsAndStrings(original);
  final lines = source.split('\n');
  var off = 0;
  for (var i = 0; i < line - 1 && i < lines.length; i++) {
    off += lines[i].length + 1;
  }
  off += col - 1;
  final open = source.indexOf('(', off);
  if (open < 0) return [];
  var depth = 0, end = source.length;
  for (var i = open; i < source.length; i++) {
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
  final args = <(int, String)>[];
  var d = 0, start = open + 1;
  for (var k = open + 1; k < end; k++) {
    final c = source[k];
    if (c == '(' || c == '[' || c == '{') d++;
    if (c == ')' || c == ']' || c == '}') d--;
    if (c == ',' && d == 0) {
      args.add((start, source.substring(start, k)));
      start = k + 1;
    }
  }
  args.add((start, source.substring(start, end)));
  final out = <Ref>[];
  final head = RegExp(r'^\s*(\w+)\s*:');
  final skip = RegExp(r'^(child|children|builder|itemBuilder|key|onTap|onPressed)$');
  for (final (at, text) in args) {
    final h = head.firstMatch(text);
    final name = h?.group(1) ?? '';
    if (skip.hasMatch(name)) continue;
    final px = RegExp(r'Surface\.px\(\s*(-?\d+(?:\.\d+)?)\s*\)').allMatches(text).toList();
    for (final m in px) {
      final num = m.group(1)!;
      final s = at + m.start + m.group(0)!.indexOf(num);
      out.add(Ref(name, RefKind.exception, 'Surface.px($num)', value: double.parse(num), start: s, end: s + num.length));
    }
    for (final m in RegExp(r'\b(?:Surface|Dn)\.(?!px\b|fixed\b)\w+').allMatches(text)) {
      if (px.any((p) => m.start >= p.start && m.start < p.end)) continue;
      out.add(Ref(name, RefKind.token, m.group(0)!));
    }
    final stripped = text.replaceAllMapped(RegExp(r'Surface\.px\([^)]*\)'), (m) => ' ' * m.group(0)!.length);
    for (final m in RegExp(r'(?<![\w.])\d+(?:\.\d+)?(?![\w.])').allMatches(stripped)) {
      out.add(Ref(name, RefKind.raw, m.group(0)!, value: double.parse(m.group(0)!), start: at + m.start, end: at + m.end));
    }
  }
  return out;
}

/// `file:///…/motolii/ui/lib/a/b.dart` -> `lib/a/b.dart`; null for the framework and packages.
String? projectPath(String? uri) {
  if (uri == null) return null;
  return RegExp(r'/motolii/ui/(lib/.*)$').firstMatch(uri)?.group(1);
}

// ------------------------------------------------------------------------------------------------------------------ the session

/// One change to one file: the whole text before and after (the files are small, and an edit never adds or removes a line).
class Edit {
  Edit(this.file, this.before, this.after, this.label);
  final String file, before, after, label;
}

/// The history of one design session: what was edited, undo / redo, the session's start (BEFORE) against now (CURRENT), reset one
/// line, keep or discard. It writes through [write] and says which files changed so the caller reloads; it keeps no other state.
class DesignSession {
  DesignSession(this.read, this.write);
  final String Function(String file) read;
  final void Function(String file, String text) write;

  /// Each edited file's text before the session first touched it.
  final Map<String, String> start = {};
  final List<Edit> _done = [], _undone = [];

  /// Showing BEFORE (the session's start) now; [_current] is what CURRENT was when BEFORE was shown.
  bool showingBefore = false;
  final Map<String, String> _current = {};

  List<Edit> get history => List.unmodifiable(_done);

  /// One drag is one undo: the edits made from history position [from] on, all of one file, become a single edit.
  void coalesce(int from) {
    if (from < 0 || _done.length - from < 2) return;
    final part = _done.sublist(from);
    if (part.any((e) => e.file != part.first.file)) return;
    String head(String l) => l.contains(' → ') ? l.substring(0, l.lastIndexOf(' → ')) : l;
    String tail(String l) => l.contains(' → ') ? l.substring(l.lastIndexOf(' → ') + 3) : l;
    final first = head(part.first.label), last = tail(part.last.label);
    _done.removeRange(from, _done.length);
    _done.add(Edit(part.first.file, part.first.before, part.last.after, first.contains(' → ') ? first : '$first → $last'));
  }

  /// The whole session as data, for a hot restart (which starts the app, and so this object, again).
  Map<String, Object?> toJson() => {
    'start': start,
    'current': _current,
    'before': showingBefore,
    'done': [
      for (final e in _done) [e.file, e.before, e.after, e.label],
    ],
    'undone': [
      for (final e in _undone) [e.file, e.before, e.after, e.label],
    ],
  };

  void restore(Map<String, Object?> j) {
    Edit edit(Object? x) {
      final l = (x as List).cast<String>();
      return Edit(l[0], l[1], l[2], l[3]);
    }

    start
      ..clear()
      ..addAll((j['start'] as Map).cast<String, String>());
    _current
      ..clear()
      ..addAll((j['current'] as Map).cast<String, String>());
    showingBefore = j['before'] == true;
    _done
      ..clear()
      ..addAll((j['done'] as List).map(edit));
    _undone
      ..clear()
      ..addAll((j['undone'] as List).map(edit));
  }

  bool get canUndo => _done.isNotEmpty && !showingBefore;
  bool get canRedo => _undone.isNotEmpty && !showingBefore;
  bool get changed => start.entries.any((e) => read(e.key) != e.value) || showingBefore;

  /// Edits [file] to [after]; the files that changed. Not while BEFORE is shown.
  List<String> apply(String file, String after, String label) {
    if (showingBefore) return const [];
    final before = read(file);
    if (before == after) return const [];
    start.putIfAbsent(file, () => before);
    _done.add(Edit(file, before, after, label));
    _undone.clear();
    write(file, after);
    return [file];
  }

  List<String> undo() {
    if (!canUndo) return const [];
    final e = _done.removeLast();
    _undone.add(e);
    write(e.file, e.before);
    return [e.file];
  }

  List<String> redo() {
    if (!canRedo) return const [];
    final e = _undone.removeLast();
    _done.add(e);
    write(e.file, e.after);
    return [e.file];
  }

  /// BEFORE <-> CURRENT: every edited file takes its session-start text, or the text it had when BEFORE was shown.
  List<String> toggleBefore() {
    if (start.isEmpty) return const [];
    if (!showingBefore) {
      _current
        ..clear()
        ..addEntries(start.keys.map((f) => MapEntry(f, read(f))));
      start.forEach(write);
    } else {
      _current.forEach(write);
    }
    showingBefore = !showingBefore;
    return start.keys.toList();
  }

  /// For line [nowLine] of [file] as it is now, the line it was at when the session began: an edit can add a line (an argument put on its
  /// own line), so numbers shift. Null for a line written in this session.
  int? startLine(String file, int nowLine) {
    final s = start[file];
    if (s == null) return nowLine;
    final a = s.split('\n'), b = read(file).split('\n');
    if (a.length == b.length) return nowLine;
    // longest common subsequence of the two files' lines: the matched ones keep their place, a changed line sits between two matches
    final n = a.length, m = b.length;
    final dp = List.generate(n + 1, (_) => List<int>.filled(m + 1, 0));
    for (var i = n - 1; i >= 0; i--) {
      for (var j = m - 1; j >= 0; j--) {
        dp[i][j] = a[i] == b[j] ? dp[i + 1][j + 1] + 1 : (dp[i + 1][j] >= dp[i][j + 1] ? dp[i + 1][j] : dp[i][j + 1]);
      }
    }
    final map = List<int?>.filled(m, null);
    var i = 0, j = 0;
    while (i < n && j < m) {
      if (a[i] == b[j]) {
        map[j] = i;
        i++;
        j++;
      } else if (dp[i + 1][j] >= dp[i][j + 1]) {
        i++;
      } else {
        j++;
      }
    }
    final k = nowLine - 1;
    if (k < 0 || k >= m) return null;
    if (map[k] != null) return map[k]! + 1;
    var p = k - 1, q = k + 1;
    while (p >= 0 && map[p] == null) {
      p--;
    }
    while (q < m && map[q] == null) {
      q++;
    }
    if (p >= 0 && q < m && map[q]! - map[p]! == q - p) return map[p]! + (k - p) + 1;
    return null;
  }

  /// Puts one line of [file] back as it was at the session start (an edit never changes the line count). Recorded, so undo works.
  List<String> resetLine(String file, int line) {
    final s = start[file];
    if (s == null || showingBefore) return const [];
    final from = s.split('\n'), now = read(file).split('\n');
    if (from.length != now.length || line < 1 || line > now.length || from[line - 1] == now[line - 1]) return const [];
    now[line - 1] = from[line - 1];
    return apply(file, now.join('\n'), 'reset $file:$line');
  }

  /// The session's lines that differ from its start: file, line (now), before, after. A line added or removed shifts the rest, so the
  /// lines are matched (longest common subsequence) and only the ones that really changed are listed; an added line has no "before".
  List<(String, int, String, String)> changes() {
    final out = <(String, int, String, String)>[];
    start.forEach((file, s) {
      final a = s.split('\n'), b = read(file).split('\n');
      final n = a.length, m = b.length;
      final dp = List.generate(n + 1, (_) => List<int>.filled(m + 1, 0));
      for (var i = n - 1; i >= 0; i--) {
        for (var j = m - 1; j >= 0; j--) {
          dp[i][j] = a[i] == b[j] ? dp[i + 1][j + 1] + 1 : (dp[i + 1][j] >= dp[i][j + 1] ? dp[i + 1][j] : dp[i][j + 1]);
        }
      }
      var i = 0, j = 0;
      final gone = <String>[], came = <String>[];
      var at = 0;
      void flush() {
        for (var k = 0; k < gone.length || k < came.length; k++) {
          out.add((
            file,
            at + 1 + (k < came.length ? k : came.length - 1).clamp(0, m),
            k < gone.length ? gone[k].trim() : '',
            k < came.length ? came[k].trim() : '',
          ));
        }
        gone.clear();
        came.clear();
      }

      while (i < n || j < m) {
        if (i < n && j < m && a[i] == b[j]) {
          flush();
          i++;
          j++;
        } else {
          if (gone.isEmpty && came.isEmpty) at = j;
          if (j >= m || (i < n && dp[i + 1][j] >= dp[i][j + 1])) {
            gone.add(a[i++]);
          } else {
            came.add(b[j++]);
          }
        }
      }
      flush();
    });
    return out;
  }

  /// Back to the session start; the files that changed.
  List<String> discard() {
    final files = [
      for (final e in start.entries)
        if (read(e.key) != e.value) e.key,
    ];
    start.forEach(write);
    _clear();
    return files;
  }

  /// Keeps what is on screen as the new baseline.
  void keep() => _clear();

  void _clear() {
    start.clear();
    _done.clear();
    _undone.clear();
    _current.clear();
    showingBefore = false;
  }
}
