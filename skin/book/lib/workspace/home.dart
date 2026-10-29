// The documents beside the open one, laid out as CapCut's desktop home lays its projects (seen in CapCut for Mac
// 10/29: a wide "Create project" over all, then "Projects" with a search, its pictures in a grid, square and filled,
// each named under it with "size | length"), told as AbsentMotion's home tells them (Apache-2.0, ui/home/
// HomeScreen.kt): its shape and resolution and frames a second too (ProjectFacts.kt aspectName, HomeScreen.kt facts,
// ExportOptions.kt formatBytes), under the day it was last changed, newest first (its listRows and dateHeader, as
// Alight Motion's list), or by name (its ListHeader); opened by a press, duplicated ("Name Copy", its
// edit_layer_copy_name) or put in the Trash, asked first (its delete dialog). The picture and the size and length
// are what the document last kept (Motolii.poster), read without opening it.
import 'dart:io';
import 'dart:math' as math;

import 'package:flutter/services.dart' show HardwareKeyboard;
import 'package:flutter/widgets.dart';

import '../motolii.dart' show docFactsOf;
import '../tokens.dart';
import 'ws.dart';

/// One document listed: its file, and what it kept (null: none yet, or not read yet).
class HomeDoc {
  HomeDoc(this.path, this.changed, this.bytes);
  final String path;
  final DateTime changed;
  final int bytes;
  ({int wide, int high, double seconds, double rate})? facts;
  String? picture;
  String get name => path.split('/').last.replaceFirst(RegExp(r'\.motolii$'), '');
}

class WsHome extends StatefulWidget {
  const WsHome({super.key, required this.folder, required this.lib, required this.open, required this.onOpen, required this.onExport, required this.onNew, required this.onClose});
  final String folder, lib, open;   // (the folder listed, the inside's library (read on another isolate), the document open now)
  final void Function(String doc) onOpen;
  final void Function(List<String> docs) onExport;
  final VoidCallback onNew, onClose;
  @override
  State<WsHome> createState() => _WsHomeState();
}

class _WsHomeState extends State<WsHome> {
  static var byName = false;   // (as it was last asked, as AbsentMotion keeps its sortBy)
  List<HomeDoc> _docs = const [];
  String? _menu, _asking;   // (the card whose actions are open; the one asked whether it goes to the Trash)
  String _find = '';   // (CapCut's search over its projects: by name)
  final _picked = <String>{};   // (Alight Motion's ⋯ › Select, or ⌘ as the Finder's and DaVinci Resolve's project list: while any is, a press picks)
  late final Directory _pictures = Directory.systemTemp.createTempSync('motolii_home');

  @override
  void initState() {
    super.initState();
    _read();
  }

  @override
  void dispose() {
    _findText.dispose();
    _findFocus.dispose();
    try {
      _pictures.deleteSync(recursive: true);
    } catch (_) {}
    super.dispose();
  }

  var _reads = 0;   // (each reading's pictures named apart: one shown is not written over while it is read)
  void _read() {
    final docs = <HomeDoc>[], read = _reads++;
    for (final f in Directory(widget.folder).listSync().whereType<File>()) {
      if (!f.path.endsWith('.motolii')) continue;
      final s = f.statSync();
      docs.add(HomeDoc(f.path, s.modified, s.size));
    }
    setState(() => _docs = docs);
    for (final (i, d) in docs.indexed) {
      final png = '${_pictures.path}/$read-$i.png';
      docFactsOf(widget.lib, d.path, png).then((facts) {
        if (!mounted) return;
        setState(() => d
          ..facts = facts
          ..picture = facts != null && File(png).existsSync() ? png : null);
      });
    }
  }

  /// A copy beside it, named as AbsentMotion names one ("Name Copy", then a number when that is taken).
  void _duplicate(HomeDoc d) {
    var to = '${widget.folder}/${d.name} Copy.motolii';
    for (var n = 2; File(to).existsSync(); n++) {
      to = '${widget.folder}/${d.name} Copy $n.motolii';
    }
    File(d.path).copySync(to);
    _menu = null;
    _read();
  }

  /// Into the Trash, not gone: the user's own ~/.Trash, as the Finder puts a file there (a name taken there: a number).
  void _trash(HomeDoc d) {
    final trash = '${Platform.environment['HOME']}/.Trash';
    var to = '$trash/${d.name}.motolii';
    for (var n = 2; File(to).existsSync(); n++) {
      to = '$trash/${d.name} $n.motolii';
    }
    try {
      File(d.path).renameSync(to);
    } catch (_) {
      return;
    }
    _menu = _asking = null;
    _read();
  }

  static const _months = ['January', 'February', 'March', 'April', 'May', 'June', 'July', 'August', 'September', 'October', 'November', 'December'];
  static String _day(DateTime t) => '${t.day} ${_months[t.month - 1]} ${t.year}';   // (AbsentMotion's dateHeader in English: d MMMM yyyy)

  @override
  Widget build(BuildContext context) {
    final rows = <Widget>[], shown = [for (final d in _docs) if (d.name.toLowerCase().contains(_find.toLowerCase())) d];
    Widget grid(List<HomeDoc> docs) => Wrap(spacing: 20, runSpacing: 20, children: [for (final d in docs) _card(d)]);
    if (byName) {
      rows.add(grid(shown..sort((a, b) => a.name.toLowerCase().compareTo(b.name.toLowerCase()))));
    } else {
      final byDay = <String, List<HomeDoc>>{};
      for (final d in shown..sort((a, b) => b.changed.compareTo(a.changed))) {
        (byDay[_day(d.changed)] ??= []).add(d);
      }
      for (final MapEntry(key: day, value: docs) in byDay.entries) {
        rows
          ..add(Padding(padding: EdgeInsets.only(top: rows.isEmpty ? 0 : 24, bottom: 12), child: Text(day, style: T.label(Grey.g76))))
          ..add(grid(docs));
      }
    }
    return ColoredBox(
      key: const ValueKey('ws-home'),
      color: WsT.ground,
      child: Padding(
        padding: const EdgeInsets.fromLTRB(32, 24, 32, 0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Row(children: [const Spacer(), _button('home-close', '×', false, widget.onClose)]),
            const SizedBox(height: 8),
            GestureDetector(   // (CapCut's "Create project", as wide as all)
              key: const ValueKey('home-new'),
              onTap: widget.onNew,
              child: Container(
                height: 120,
                alignment: Alignment.center,
                decoration: BoxDecoration(color: WsT.card, borderRadius: BorderRadius.circular(12), border: Border.all(color: WsT.raised)),
                child: Text('+  New Project', style: T.title(Grey.g91)),
              ),
            ),
            const SizedBox(height: 28),
            Row(
              children: [
                Text('Projects', style: T.title(Grey.g91)),
                const Spacer(),
                SizedBox(
                  width: 180,
                  child: Container(
                    padding: EdgeInsets.symmetric(horizontal: WsT.inset, vertical: 4),
                    decoration: BoxDecoration(color: WsT.card, borderRadius: BorderRadius.circular(6)),
                    child: EditableText(
                      key: const ValueKey('home-find'),
                      controller: _findText,
                      focusNode: _findFocus,
                      style: T.label(Grey.g91),
                      cursorColor: WsT.accent,
                      backgroundCursorColor: Grey.g63,
                      onChanged: (v) => setState(() => _find = v),
                    ),
                  ),
                ),
                const SizedBox(width: 8),
                if (_picked.isNotEmpty) ...[
                  _button('home-export', 'Export ${_picked.length}', true, () => widget.onExport([..._picked])),
                  _button('home-unpick', 'Cancel', false, () => setState(_picked.clear)),
                  const SizedBox(width: 8),
                ],
                _button('home-by-date', 'Date', !byName, () => setState(() => byName = false)),
                _button('home-by-name', 'Name', byName, () => setState(() => byName = true)),
              ],
            ),
            const SizedBox(height: 16),
            Expanded(child: ListView(children: rows)),
          ],
        ),
      ),
    );
  }

  final _findText = TextEditingController();
  final _findFocus = FocusNode();

  Widget _button(String key, String text, bool on, VoidCallback tap) => Padding(
    padding: const EdgeInsets.only(left: 4),
    child: GestureDetector(
      key: ValueKey(key),
      onTap: tap,
      child: Container(
        padding: EdgeInsets.symmetric(horizontal: WsT.inset, vertical: 4),
        decoration: BoxDecoration(color: on ? WsT.accent : WsT.card, borderRadius: BorderRadius.circular(6)),
        child: Text(text, style: T.label(on ? WsT.onAccent : Grey.g91)),
      ),
    ),
  );

  static const _tile = 150.0;   // (a tile: its picture square above its name, as CapCut's desktop grid)

  Widget _card(HomeDoc d) {
    final f = d.facts, open = d.path == widget.open;
    return SizedBox(
      width: _tile,
      child: GestureDetector(
        key: ValueKey('home-doc:${d.path}'),
        onTap: () => _picked.isNotEmpty || HardwareKeyboard.instance.isMetaPressed
            ? setState(() {
                if (!_picked.remove(d.path)) _picked.add(d.path);
              })
            : widget.onOpen(d.path),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Container(
              height: _tile,
              decoration: BoxDecoration(color: WsT.card, borderRadius: BorderRadius.circular(8), border: open ? Border.all(color: WsT.accent, width: 2) : null),
              clipBehavior: Clip.antiAlias,
              child: Stack(
                fit: StackFit.expand,
                children: [
                  if (d.picture != null) Image.file(File(d.picture!), fit: BoxFit.cover),
                  if (_picked.contains(d.path))
                    Positioned(
                      left: 0,
                      top: 0,
                      child: Container(
                        key: ValueKey('home-picked:${d.path}'),
                        margin: const EdgeInsets.all(6),
                        padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                        decoration: BoxDecoration(color: WsT.accent, borderRadius: BorderRadius.circular(4)),
                        child: Text('✓', style: T.label(WsT.onAccent)),
                      ),
                    ),
                  Positioned(
                    right: 0,
                    top: 0,
                    child: GestureDetector(
                      key: ValueKey('home-more:${d.path}'),
                      onTap: () => setState(() => _menu = _menu == d.path ? null : d.path),
                      child: Container(
                        margin: const EdgeInsets.all(6),
                        padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                        decoration: BoxDecoration(color: WsT.ground, borderRadius: BorderRadius.circular(4)),
                        child: Text('•••', style: T.label(Grey.g76)),
                      ),
                    ),
                  ),
                ],
              ),
            ),
            const SizedBox(height: 8),
            Text(d.name, style: T.name(Grey.g91).copyWith(fontWeight: FontWeight.w700), maxLines: 1, overflow: TextOverflow.ellipsis),
            const SizedBox(height: 2),
            Text([_bytes(d.bytes), if (f != null) _length(f.seconds)].join(' | '), style: T.value(Grey.g63), maxLines: 1),   // (CapCut's "254.3M | 02:13")
            Text(_facts(d).join(' · '), key: ValueKey('home-facts:${d.path}'), style: T.value(Grey.g56), maxLines: 1, overflow: TextOverflow.ellipsis),
            if (_menu == d.path)
              Padding(
                padding: const EdgeInsets.only(top: 8),
                child: Wrap(
                  spacing: 4,
                  runSpacing: 4,
                  crossAxisAlignment: WrapCrossAlignment.center,
                  children: _asking == d.path
                      ? [
                          Text('Move to the Trash?', style: T.label(Grey.g91)),
                          _button('home-trash-yes:${d.path}', 'Move to Trash', true, () => _trash(d)),
                          _button('home-trash-no:${d.path}', 'Cancel', false, () => setState(() => _asking = null)),
                        ]
                      : [
                          _button('home-open:${d.path}', 'Open', false, () => widget.onOpen(d.path)),
                          _button('home-select:${d.path}', 'Select', false, () => setState(() {
                            _picked.add(d.path);
                            _menu = null;
                          })),
                          _button('home-export:${d.path}', 'Export…', false, () => widget.onExport([d.path])),
                          _button('home-duplicate:${d.path}', 'Duplicate', false, () => _duplicate(d)),
                          if (!open) _button('home-trash:${d.path}', 'Move to Trash', false, () => setState(() => _asking = d.path)),
                        ],
                ),
              ),
          ],
        ),
      ),
    );
  }

  /// Its shape and resolution, then its size and frames a second: AbsentMotion's facts (a named shape: "9:16" and the
  /// shorter side "1080p"; else "1080x1350").
  static List<String> _facts(HomeDoc d) {
    final f = d.facts;
    if (f == null) return const [];
    String? ratio;
    for (final (a, b) in const [(16, 9), (9, 16), (4, 5), (1, 1), (4, 3)]) {
      if ((f.wide * b - f.high * a).abs() <= math.max(a, b)) {
        ratio = '$a:$b';
        break;
      }
    }
    return [if (ratio != null) ...[ratio, '${math.min(f.wide, f.high)}p'] else '${f.wide}x${f.high}', '${f.rate == f.rate.roundToDouble() ? f.rate.round() : f.rate}fps'];   // (its own, the size box's frame_rate)
  }

  static String _bytes(int bytes) {   // (ExportOptions.kt formatBytes)
    String f(double v, String unit) => (v >= 10 ? v.toStringAsFixed(0) : v.toStringAsFixed(1).replaceFirst(RegExp(r'\.0$'), '')) + unit;
    if (bytes < 1024) return '${math.max(bytes, 0)}B';
    if (bytes < 1024 * 1024 - 512) return f(bytes / 1024, 'KB');
    if (bytes < 1024 * 1024 * 1024 - 512 * 1024) return f(bytes / (1024 * 1024), 'MB');
    return f(bytes / (1024 * 1024 * 1024), 'GB');
  }

  static String _length(double seconds) {   // (CapCut's "02:13"; an hour or more as AbsentMotion's 01:02:13)
    final s = seconds.round();
    String two(int n) => n.toString().padLeft(2, '0');
    return s >= 3600 ? '${two(s ~/ 3600)}:${two(s ~/ 60 % 60)}:${two(s % 60)}' : '${two(s ~/ 60)}:${two(s % 60)}';
  }
}
