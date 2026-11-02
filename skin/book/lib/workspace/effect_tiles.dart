// Every effect on the shelf has a tile that shows what it does, as Alight Motion's do (reports/alight_ui.md, 71 and 72).
// AbsentMotion shows each of its effects by a table written beforehand (tools/effect_showcase.json); here a tile is told
// the same things by the effect itself, so a file put in by its maker needs no line and no developer. Read from it
// (doc_recipes.h, isf_tags): a transition's progress goes from 0 to 1, what runs by the clock or keeps its frame before is
// left to run. Measured (AbsentMotion's change(), reports/sketch_import_2026-11-01.md): the sample stands still, so what
// moves is the effect; its number knob that changes the picture most goes from where it is to a value plainly seen
// (fx_first.sh tries knobs so); nothing seen on the whole photo, it goes on a card going past on AbsentMotion's grid (its
// "move" layout), where what it does at an edge or to what moves shows; what does not move is split into before and after.
// AbsentMotion's own effects keep their line of the table. Sixteen pictures a twelfth of a second apart, as its moving
// tiles, played there and back at 24 a second, so nothing steps or jumps where it starts again. The sample is the balloon
// fx_tiles.sh draws on (fx_sample.jpg). Drawn on a
// document of its own in another isolate (as create_preview.dart reckons the recipes), where the inside says one drawn
// for it goes (its home in ~/Library/Caches/Motolii/tiles, named by the file and when it was written: doc_recipes.h
// doc_tile), then shown at once. Put on as fx_tiles.sh and edit.sh say it: a screen effect (.fs, .wgsl) on the whole
// picture; else on the photo, as a thing's (an AviUtl2 script). Made small with sips (192 px), the moving ones JPEG by
// ffmpeg, one run a tile. Then set beside the sample's own pictures with ffmpeg's psnr (the tool the inside writes films
// with): seen to change nothing in any of them (over 40 dB: helpers of other scripts, PSD ones, scripts that stop on a
// call not made yet) it is marked (home.same), and the shelf says so instead of passing the sample off as the effect.
// The maker's own folder is watched while the window is open (dart:io's watch, FSEvents on a Mac): a file put in or taken
// out there and the shelves are read again (BRIDGE.md rule 8, the second half), and its tile drawn, ahead of the rest.
import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:isolate';

import 'package:flutter/foundation.dart' show ValueNotifier;
import 'package:flutter/painting.dart' show FileImage, ResizeImage;

import '../motolii.dart';

abstract final class EffectTiles {
  static final _asked = <String>{}, _queue = <(String, String)>[];
  static var _drawing = false;
  static void Function()? _told;

  /// How many of the tiles in line are drawn, out of how many, while any are (the shelf's reload mark says it).
  static final progress = ValueNotifier<(int, int)?>(null);

  /// [file]'s tile drawn again, ahead of the line, or, with none, every tile on the shelf (the reload mark).
  static void again(Motolii m, {String? file}) {
    _queue.removeWhere((q) => file == null || q.$1 == file);
    for (final (_, f) in m.effects) {
      final home = m.tileHome(f);
      if (home.isEmpty || (file != null && f != file)) continue;
      _asked.remove(home);
      try {
        File('$home.shows').deleteSync();
      } catch (_) {}
      if (file != null && _asked.add(home)) _queue.insert(0, (f, home));
    }
    if (_told case final changed?) ask(m, changed);
  }

  /// How wide a tile's pictures are read (dock_effects.dart _Moving's Image.file): one drawn again under its own name is
  /// let go of at that width, or Flutter's image cache goes on showing the one drawn before (seen: AbsentMotion's split
  /// tiles came only once the window was opened again).
  static const readWide = 256;

  /// The moments its pictures are drawn at (AbsentMotion's 16 at 12 a second), and the one its still is: well on the way.
  static final moments = [for (var k = 0; k < 16; k++) 0.1 + k / 12];
  static const still = 10;

  /// What AbsentMotion shows each of its effects with (its tools/effect_showcase.json, Apache-2.0): the knobs that make
  /// the effect plain to see, and where its picture is split into before (left) and after. By the effect's name in the
  /// table, the file's own `am_<name>.wgsl`; each knob as from_absent.py named it (skslc's R_ before a name WGSL keeps for
  /// itself, origin_x and the like with _ after), a vector or a colour one knob each way; a moving line's list of numbers
  /// for one knob the stops it goes through ([sweeps]), its layer moved the sample's drift ([moves]). Null: a file the
  /// table has not.
  static ({Map<String, double> knobs, double? split, Map<String, List<double>> sweeps, bool moves})? showcase(String file, String head, Map<String, dynamic> table) {
    final name = RegExp(r'/am_([a-z0-9]+)\.wgsl$').firstMatch(file)?.group(1);
    final show = name == null ? null : (table['effects'] as Map<String, dynamic>?)?[name] as Map<String, dynamic>?;
    if (show == null) return null;
    final tracks = {for (final t in RegExp(r'--track@([A-Za-z0-9_]+):').allMatches(head)) t.group(1)!};
    final stem = file.split('/').last.replaceFirst(RegExp(r'\.[^.]*$'), '').replaceAll(RegExp(r'[^A-Za-z0-9_]'), '_');
    final knobs = <String, double>{}, sweeps = <String, List<double>>{};
    String? named(String p, String way) => [for (final n in [p, 'R_$p', '${p}_']) '$n$way'].where(tracks.contains).firstOrNull;
    for (final MapEntry(key: p, :value) in ((show['params'] as Map<String, dynamic>?) ?? const {}).entries) {
      if (show.containsKey('animate') && value is List && value.every((x) => x is num)) {   // (effect_thumbs.py prop: its keys, evenly apart)
        if (named(p, '') case final knob?) {
          sweeps['js_${stem}_$knob'] = [for (final x in value) (x as num).toDouble()];
          continue;
        }
      }
      final parts = switch (value) {
        num v => [v.toDouble()],
        bool v => [v ? 1.0 : 0.0],
        String v when RegExp(r'^#[0-9a-fA-F]{6}$').hasMatch(v) => [for (var i = 1; i < 7; i += 2) int.parse(v.substring(i, i + 2), radix: 16) / 255, 1.0],
        List v when v.length <= 4 && v.every((x) => x is num) => [for (final x in v) (x as num).toDouble()],   // (a vector; a longer list is not one: left as the file says)
        _ => const <double>[],
      };
      for (final (i, v) in parts.indexed) {
        final way = parts.length > 1 ? '_${'xyzw'[i]}' : '';
        if (named(p, way) case final knob?) knobs['js_${stem}_$knob'] = v;
      }
    }
    return (knobs: knobs, split: (show['split'] as num?)?.toDouble(), sweeps: sweeps, moves: show['layout'] == 'move');
  }

  /// AbsentMotion's table, read once (in each isolate that draws): empty where the library has not got it.
  static Map<String, dynamic> table(String dir) {
    try {
      return jsonDecode(File('$dir/third_party/AbsentMotion/tools/effect_showcase.json').readAsStringSync()) as Map<String, dynamic>;
    } catch (_) {
      return const {};
    }
  }

  /// Its moving pictures: home_0.jpg … home_7.jpg beside its still (none for a picture not drawn here).
  static String frame(String home, int k) => '${home.substring(0, home.length - 4)}_$k.jpg';
  static String _drawn(String home, int k) => '${home.substring(0, home.length - 4)}_$k.png';   // (as the inside draws it, before it is made small)

  /// The maker's own folder: MOTOLII_OWN, or ~/Library/Application Support/Motolii (as the inside's doc_folders).
  static String get own => Platform.environment['MOTOLII_OWN'] ?? '${Platform.environment['HOME']}/Library/Application Support/Motolii';

  /// The library's own folders beside the maker's (as the inside's doc_folders: the folder above its out/).
  static List<String> get library => [for (final f in const ['effects', 'aviutl_scripts', 'recipes', 'animators']) '$motoliiDir/$f'];

  /// Watched until the subscription is cancelled; [changed] is told after the folders are read again and after tiles are drawn.
  static StreamSubscription<FileSystemEvent>? watch(Motolii m, void Function() changed, {void Function(String file)? program}) {
    ask(m, changed);
    final dir = Directory('$own/effects');
    try {
      dir.createSync(recursive: true); // (there to put files in, as README says)
    } catch (_) {
      return null;
    }
    Timer? soon;
    void again() {
      m.forgetFolders();
      changed();
      ask(m, changed);
    }
    final inner = <StreamSubscription<FileSystemEvent>>[];
    final merged = StreamController<FileSystemEvent>(onCancel: () => Future.wait([for (final w in inner) w.cancel()]));   // (one watch a folder, told as one)
    inner.addAll([for (final p in [own, ...library.where((p) => Directory(p).existsSync())]) Directory(p).watch(recursive: true).listen(merged.add)]);
    final told = <String, Timer>{};
    return merged.stream.listen((e) {
      if (program != null && e.path.startsWith('$own/programs/')) {   // (a program's file: what is made from it follows, no shelf is read again)
        told[e.path]?.cancel();
        told[e.path] = Timer(const Duration(milliseconds: 300), () => program(e.path));
        return;
      }
      soon?.cancel();
      soon = Timer(const Duration(milliseconds: 300), again); // (a copy is several events: read once it has settled)
    });
  }

  /// The effects on the shelf whose tile is not drawn yet: one drawn before (this run, or another) shown at once; the
  /// rest put in line, the maker's own at its head (what was just put in), drawn a few at a time, each few shown when done.
  static void ask(Motolii m, void Function() changed) {
    final mine = <(String, String)>[], theirs = <(String, String)>[];
    var shown = false;
    _told = changed;
    for (final (_, f) in [...m.effects]..sort((a, b) => a.$1.toLowerCase().compareTo(b.$1.toLowerCase()))) {   // (as the shelf lists them: its top first)
      final home = m.tileHome(f);
      final stale = home.isNotEmpty && !File('$home.shows').existsSync();   // (drawn before a tile showed what its effect does: drawn again)
      if (home.isEmpty || (!stale && m.picture(f) == home)) continue;
      if (!stale && File(home).existsSync()) {
        m.tileDrawn(home, same: File('$home.same').existsSync());
        shown = true;
      } else if (_asked.add(home)) {
        (f.startsWith(own) ? mine : theirs).add((f, home));
      }
    }
    if (shown) changed();
    if (Platform.environment['FLUTTER_TEST'] == 'true') return;   // (flutter test's widget tests: none drawn behind them, seven hundred pictures on the GPU while they time the window; flutter drive's draw them)
    _queue
      ..insertAll(0, mine)
      ..addAll(theirs);
    if (!_drawing) unawaited(_work(m, changed));
  }

  static Future<void> _work(Motolii m, void Function() changed) async {
    _drawing = true;
    final dir = motoliiDir;
    var done = 0;
    try {
      while (_queue.isNotEmpty) {
        progress.value = (done, done + _queue.length);
        final some = _queue.take(12).toList();
        _queue.removeRange(0, some.length);
        done += some.length;
        for (final (home, same) in await Isolate.run(() => draw(dir, some))) {
          for (final p in [home, for (final (k, _) in moments.indexed) frame(home, k)]) {
            ResizeImage(FileImage(File(p)), width: readWide).evict();
          }
          m.tileDrawn(home, same: same);
        }
        changed();
      }
    } finally {
      _drawing = false;
      progress.value = null;
    }
  }

  static List<(String, bool)> draw(String dir, List<(String, String)> files) {
    final folder = Directory.systemTemp.createTempSync('motolii_tiles'), drawn = <(String, bool)>[];
    final m = Motolii.open('${folder.path}/tiles.motolii', wide: 384, high: 384, dir: dir);   // (AbsentMotion's RENDER, its THUMB below)
    if (m == null) return drawn;
    _ffmpeg(['-f', 'lavfi', '-i', 'color=c=0x2c2c34:s=384x384,drawgrid=w=48:h=48:t=1:c=0x45454f', '-frames:v', '1', '${folder.path}/grid.png']);   // (its grid, where its "move" layout stands)
    final grid = m.add('picture', '${folder.path}/grid.png'), photo = m.add('picture', '$dir/fx_sample.jpg'), card = m.add('picture', '$dir/fx_sample.jpg');
    m
      ..set(grid, 'size', 3.6)
      ..set(photo, 'size', 3.6)
      ..set(card, 'size', 1.8)
      ..key(card, 'x', moments.first, -0.84)
      ..key(card, 'x', moments.last, 0.84);
    void lay(bool moving) {
      for (final (thing, seen) in [(grid, moving), (card, moving), (photo, !moving)]) {
        m.set(thing, 'opacity', seen ? 1 : 0);
      }
    }
    List<String> shoot(String home, {bool plays = false, void Function(double at)? turn}) {   // (its pictures, made small; plays: one that keeps its frame before (ISF's PERSISTENT) drawn every other frame from the start, the sixteen kept as it passes them, as fx_tiles.sh warms one)
      final made = <String>[];
      bool one(double at, String to) {
        turn?.call(at);
        return m.at(at) && m.still(to);
      }
      if (plays) {
        for (var n = 1, k = 0; k < moments.length && n <= (moments.last * 30).round(); n++) {
          final keep = n == (moments[k] * 30).round();
          if (one(n / 30, keep ? _drawn(home, k) : '${folder.path}/warm.png') && keep) made.add(_drawn(home, k));
          if (keep) k++;
        }
      } else {
        made.addAll([for (final (k, at) in moments.indexed) if (one(at, _drawn(home, k))) _drawn(home, k)]);
      }
      if (made.isNotEmpty) Process.runSync('sips', ['-Z', '192', ...made]);
      return made;
    }
    lay(false);
    final plain = shoot('${folder.path}/plain.png');   // (the sample as it is, to tell a tile that shows nothing done)
    lay(true);
    final moved = shoot('${folder.path}/moved.png');
    final enough = plain.length == moments.length && moved.length == moments.length;
    final shows = table(dir), own = {for (final t in [photo, card]) ...m.values(t)};
    for (final (f, home) in files) {
      try {
        Directory(File(home).parent.path).createSync(recursive: true);
      } catch (_) {
        continue;
      }
      final show = f.endsWith('.wgsl') ? showcase(f, File(f).readAsStringSync(), shows) : null;
      final said = m.tags(f), plays = f.endsWith('.fs') && RegExp(r'"PERSISTENT"\s*:\s*(true|1)').hasMatch(File(f).readAsStringSync());
      final runs = said.contains('time') || said.contains('feedback') || plays;
      final first = (show?.moves ?? false) || said.contains('feedback') || plays;   // (an effect of what was has something going past it to see)
      final base = home.substring(0, home.length - 4);
      var same = false, split = false, stands = true;
      var made = const <String>[];
      for (final moving in [first, if (!first) true]) {   // (nothing seen on the whole photo: on a card going past, where what it does at an edge or to what moves shows, as AbsentMotion's "move")
        lay(moving);
        final whole = !moving && (f.endsWith('.fs') || f.endsWith('.wgsl') || f.endsWith('.json')) && m.say(0, 'script', f);   // (.json: Filament's own settings, laid on the stage's view: viewer_settings.h)   // (a screen effect; the stage takes an AviUtl2 script too, but draws none of it: motolii.h, a thing's)
        final thing = moving ? card : photo, on = whole ? 0 : thing, onIt = !whole && m.say(thing, 'fx', f);   // (as a thing's: as edit.sh says it)
        for (final MapEntry(:key, :value) in (show?.knobs ?? const <String, double>{}).entries) {
          m.set(on, key, value);   // (as AbsentMotion shows it: its table's values, not the file's defaults)
        }
        final knobs = [for (final v in m.values(on)) if (v.startsWith('js_') && !own.contains(v)) v];
        final sweeps = <String, List<double>>{...?show?.sweeps};
        if (knobs.where((k) => k.endsWith('_progress')).firstOrNull case final p? when said.contains('transition')) {
          sweeps[p] = const [0, 1];   // (a transition seen going from 0 to 1 across its pictures)
        } else if (show == null && !runs) {
          if (_strongest(m, on, knobs, folder.path) case final knob?) sweeps[knob.$1] = [knob.$2, knob.$3];
        }
        if (!moving && !runs && sweeps.isEmpty && enough && _same('${folder.path}/usual.png', '${folder.path}/plain_$still.png')) {   // (nothing on the photo: no sixteen pictures of nothing)
          if (whole) m.say(0, 'script', 'none');
          if (onIt) m.say(thing, 'fx', 'none');
          same = true;
          continue;
        }
        made = shoot(home, plays: plays, turn: (at) {
          final t = ((at - moments.first) / (moments.last - moments.first)).clamp(0.0, 1.0);
          for (final MapEntry(key: knob, value: stops) in sweeps.entries) {
            m.set(on, knob, _along(stops, t * t * (3 - 2 * t)));   // (smoothstep: there and back, it does not hit its ends)
          }
        });
        if (whole) m.say(0, 'script', 'none');
        if (onIt) m.say(thing, 'fx', 'none');
        if (made.length < moments.length) continue;
        same = enough && _unseen(home, '${folder.path}/${moving ? 'moved' : 'plain'}', over: moving ? 30 : 40);   // (a card through any effect differs at its edge, 36.7 dB on these; what is plainly seen is under 30: fx_first.sh)
        stands = !moving && !runs && sweeps.isEmpty;
        split = stands && !said.contains('generator');
        if (!same) break;
      }
      if (made.length < moments.length || same) {   // (put on but not drawn, a script that stops, or nothing seen: the sample as it is)
        for (final (k, _) in moments.indexed) {
          if (enough) File('${folder.path}/plain_$k.png').copySync(_drawn(home, k));
        }
        stands = true;
        if (!enough) continue;
      }
      final at = show != null ? show.split : split ? 0.4 : null;   // (at AbsentMotion's 0.4, its 22 with a split: what does not move is seen beside what it did it to, golf ⑪)
      if (at != null && !same && enough) {   // (before at its left, as AbsentMotion's effect_thumbs.py lays the photo over it up to there, a white line where they meet)
        if (_ffmpeg(['-i', '${base}_%d.png', '-i', '${folder.path}/plain_%d.png', '-filter_complex', '[1:v]crop=iw*$at:ih:0:0[b];[0:v][b]overlay=0:0,drawbox=x=iw*$at-2:y=0:w=2:h=ih:color=white:t=fill', '-start_number', '0', '${base}_%d.split.png'])) {
          for (final (k, _) in moments.indexed) {
            File('${base}_$k.split.png').renameSync(_drawn(home, k));
          }
        }
      }
      File(_drawn(home, EffectTiles.still)).copySync(home);
      if (stands) {
        for (final (k, _) in moments.indexed) {
          File(frame(home, k)).existsSync() ? File(frame(home, k)).deleteSync() : null;   // (nothing moves: its still alone)
        }
      } else {
        _ffmpeg(['-i', '${base}_%d.png', '-q:v', '6', '-start_number', '0', '${base}_%d.jpg']);
      }
      for (final (k, _) in moments.indexed) {
        File(_drawn(home, k)).deleteSync();
      }
      try {
        same ? File('$home.same').writeAsStringSync('') : File('$home.same').deleteSync();
      } catch (_) {}
      File('$home.shows').writeAsStringSync('');
      drawn.add((home, same));
    }
    m.close();
    try {
      folder.deleteSync(recursive: true);
    } catch (_) {}
    return drawn;
  }

  /// How far along [t] (0 to 1) a line of evenly spaced stops stands.
  static double _along(List<double> stops, double t) {
    final at = t * (stops.length - 1), k = at.floor().clamp(0, stops.length - 2);
    return stops[k] + (stops[k + 1] - stops[k]) * (at - k);
  }

  /// The number knob that changes the picture most, from what it has to a value plainly seen and short of wiping the
  /// picture out, as fx_first.sh tries them and AbsentMotion's change() tells a tile that shows nothing: at most eight,
  /// each half way to the far end of its span (all the way round, an angle's far end is where it began), single numbers
  /// before a vector's parts, what only fades it out left alone; then the one that showed most, in steps. Null: none shows.
  static (String, double, double)? _strongest(Motolii m, int on, List<String> knobs, String folder) {
    if (!m.at(moments[still]) || !m.still('$folder/usual.png')) return null;
    final part = RegExp(r'_[xyzw]$'), plainly = RegExp(r'alpha|opacity', caseSensitive: false);
    final tried = <(String, double, double)>[];
    for (final k in [...knobs.where((k) => !part.hasMatch(k)), ...knobs.where(part.hasMatch)].where((k) => !plainly.hasMatch(k))) {
      final span = m.shown(k), range = m.range(on, k), from = m.value(on, k, 0);
      if (range == null || from == null || const ['onoff', 'seed'].contains(span?.look)) continue;
      final (low, high) = span != null && span.high > span.low ? (span.low, span.high) : range;
      final to = from + (((high - from).abs() >= (from - low).abs() ? high : low) - from) / 2;
      m.set(on, k, to);
      if (m.still('$folder/try_${tried.length}.png')) tried.add((k, from, to));
      m.set(on, k, from);
      if (tried.length == 8) break;
    }
    if (tried.isEmpty) return null;
    final seen = _changes(folder);
    if (seen.isEmpty) return null;
    final most = seen.reduce((a, b) => b.$2 > a.$2 ? b : a);
    if (most.$2 <= _plain || most.$1 >= tried.length) return null;
    final (k, from, half) = tried[most.$1];
    const steps = [1 / 32, 1 / 8, 1 / 2, 2.0];   // (spans go to 4000 in an AviUtl2 script: what shows is often far short of half way)
    final at = [for (final s in steps) from + (half - from) * s];
    for (final (i, v) in at.indexed) {
      m.set(on, k, v);
      m.still('$folder/try_$i.png');
    }
    m.set(on, k, from);
    final fine = _changes(folder), ways = [
      for (final (i, v) in at.indexed) (v, fine.elementAtOrNull(i)?.$2 ?? 0),
    ]..insert(3, (half, most.$2));
    return (k, from, (ways.where((w) => w.$2 >= _clear).firstOrNull ?? ways.reduce((a, b) => b.$2 > a.$2 ? b : a)).$1);
  }

  /// How far each try is from the usual picture: (its number, ffmpeg psnr's mean squared error), one run for them all.
  static List<(int, double)> _changes(String folder) {
    final ProcessResult r;
    try {
      r = Process.runSync('ffmpeg', ['-hide_banner', '-loop', '1', '-i', '$folder/usual.png', '-i', '$folder/try_%d.png', '-lavfi', '[0:v][1:v]psnr=shortest=1:stats_file=-', '-f', 'null', '-']);
    } on ProcessException {
      return const [];   // (no ffmpeg where the window was opened from: not measured)
    } finally {
      for (var i = 0; File('$folder/try_$i.png').existsSync(); i++) {
        File('$folder/try_$i.png').deleteSync();   // (the next effect's may be fewer)
      }
    }
    return [for (final l in '${r.stdout}'.split('\n')) if (RegExp(r'n:(\d+) mse_avg:([\d.]+)').firstMatch(l) case final w?) (int.parse(w.group(1)!) - 1, double.parse(w.group(2)!))];
  }

  static final _plain = 255 * 255 / 10000.0, _clear = 255 * 255 / 251.0;   // (mean squared error at 40 dB, where _unseen stops seeing a change; at 24 dB, plainly seen and the picture still there, measured on these tiles)

  /// Whether two pictures are not seen to differ (as [_unseen]).
  static bool _same(String a, String b) => (_psnr(a, b) ?? 0) > 40;

  /// ffmpeg over a tile's pictures; false where it did not run (none where the window was opened from: its still alone).
  static bool _ffmpeg(List<String> also) {
    try {
      return Process.runSync('ffmpeg', ['-hide_banner', '-loglevel', 'error', '-y', ...also]).exitCode == 0;
    } on ProcessException {
      return false;
    }
  }

  /// Whether none of its pictures is seen to differ from the sample's: ffmpeg's psnr, the worst of them over [over] dB
  /// (measured on these tiles: the ones that show nothing all over 45 dB, the ones that show something under 40).
  static bool _unseen(String home, String sample, {required double over}) => (_psnr('${home.substring(0, home.length - 4)}_%d.png', '${sample}_%d.png') ?? 0) > over;

  /// The worst psnr between pictures, 1000 for none differing; null: no ffmpeg where the window was opened from, not said.
  static double? _psnr(String a, String b) {
    final ProcessResult r;
    try {
      r = Process.runSync('ffmpeg', ['-hide_banner', '-i', a, '-i', b, '-lavfi', 'psnr', '-f', 'null', '-']);
    } on ProcessException {
      return null;
    }
    return switch (RegExp(r'min:(inf|[\d.]+)').firstMatch('${r.stderr}')?.group(1)) {
      null => null,
      'inf' => 1000,
      final min => double.parse(min),
    };
  }
}
