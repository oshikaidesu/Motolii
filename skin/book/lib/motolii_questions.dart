// motolii_questions.dart: questions of the inside (skin/BRIDGE.md Q): asked on another isolate or a channel, the answer a Future; the window never waits on them.
part of 'motolii.dart';

extension MotoliiQuestions on Motolii {
  /// The stage's picture as a Flutter texture (the inside draws it by its own clock; nothing is copied). -1: none.
  Future<int> texture() async {
    final (a, _) = await answered(later('live'));   // (opened by the hand: the window never stands still for it)
    if (a == 0) return -1;
    return await Motolii._channel.invokeMethod<int>('start', {'lib': libPath, 'handle': _m.address}) ?? -1;
  }

  /// The desk's picture as a Flutter texture (-1: none).
  Future<int> deskTexture() async => _deskId ??= await Motolii._channel.invokeMethod<int>('start', {'lib': libPath, 'handle': _m.address, 'front': 'motolii_desk_front', 'frames': 'motolii_desk_frames'}) ?? -1;

  /// The whole document written as a film (.mp4, with its films' sound), on another isolate: the window does not wait.
  /// The inside's one hand takes the call in its turn, from whatever thread it comes.
  Future<bool> write(String mp4) {
    final lib = libPath, at = _m.address;
    return Isolate.run(() {
      final ask = c.MotoliiBindings(DynamicLibrary.open(lib)).motolii_write;
      final to = mp4.toNativeUtf8();
      final done = ask(_P.fromAddress(at), to) != 0;
      malloc.free(to);
      return done;
    });
  }

  /// As the stage shows each thing's value at each moment asked (what a relation makes of it, else its own; null: it
  /// has none): another moment than the picture's, a question (skin/BRIDGE.md Q), asked on another isolate.
  Future<List<double?>> shownAt(List<(int, String, double)> asks) {
    final lib = libPath, at = _m.address;
    return Isolate.run(() {
      final l = DynamicLibrary.open(lib), m = _P.fromAddress(at);
      final shows = c.MotoliiBindings(l).motolii_shows;
      final out = malloc<Double>();
      final all = [
        for (final (t, n, moment) in asks)
          () {
            final name = n.toNativeUtf8(), got = shows(m, t, name, moment, out);
            malloc.free(name);
            return got == 0 ? null : out.value;
          }(),
      ];
      malloc.free(out);
      return all;
    });
  }  /// Each thing's own value at each moment asked, as the document says it (not what a relation makes of it; null: it

  /// has none): another moment than the picture's, a question (skin/BRIDGE.md Q), asked on another isolate as [shownAt].
  Future<List<double?>> valuesAt(List<(int, String, double)> asks) {
    final lib = libPath, at = _m.address;
    return Isolate.run(() {
      final l = DynamicLibrary.open(lib), m = _P.fromAddress(at);
      final value = c.MotoliiBindings(l).motolii_value;
      final out = malloc<Double>();
      final all = [
        for (final (t, n, moment) in asks)
          () {
            final name = n.toNativeUtf8(), got = value(m, t, name, moment, out);
            malloc.free(name);
            return got == 0 ? null : out.value;
          }(),
      ];
      malloc.free(out);
      return all;
    });
  }

  /// Of these font files, those a text can be [words] in (asked on another isolate: the window does not wait).
  Future<List<String>> setting(List<String> fonts, String words) {
    final lib = libPath;
    return Isolate.run(() {
      final ask = c.MotoliiBindings(DynamicLibrary.open(lib)).motolii_sets;
      final said = words.toNativeUtf8(), out = <String>[];
      for (final f in fonts) {
        final file = f.toNativeUtf8();
        if (ask(file, said) != 0) out.add(f);
        malloc.free(file);
      }
      malloc.free(said);
      return out;
    });
  }

  /// How a thing moves over a while, as the stage places it (its wave and all): [n] moments from [from] to before [to],
  /// four numbers each: where it is (x, y, units of the stage), how far turned (degrees), how large (a share). null: no
  /// such thing. Asked once when its values change, for a small preview that then plays by itself (motolii_path).
  Future<List<double>?> path(int thing, {int copy = 0, double from = 0, required double to, int n = 60}) => _ask((l, m) {
    final out = malloc<Double>(4 * n);
    final got = c.MotoliiBindings(l).motolii_path(m, thing, copy, from, to, n, out);
    final p = [for (var i = 0; i < 4 * got; i++) out[i]];
    malloc.free(out);
    return got > 0 ? p : null;
  });

  /// A question of the inside (skin/BRIDGE.md Q): asked on another isolate, the window does not wait for its answer.
  Future<T> _ask<T>(T Function(DynamicLibrary lib, _P m) ask) {
    final lib = libPath, at = _m.address;
    return Isolate.run(() => ask(DynamicLibrary.open(lib), _P.fromAddress(at)));
  }

  /// The same in the round: four numbers a moment, where it is (x, y, z, units of the stage) and how large (motolii_path3).
  List<double>? path3(int thing, {int copy = 0, double from = 0, required double to, int n = 24}) {
    final out = malloc<Double>(4 * n);
    final got = _path3(_m, thing, copy, from, to, n, out), p = [for (var i = 0; i < 4 * got; i++) out[i]];
    malloc.free(out);
    return got > 0 ? p : null;
  }

  /// A film's sound: its loudness (dB, 0 at full) 100 times a second of its own (none: no sound, or not a film). Read
  /// on another isolate, the file decoded once: the window does not wait.
  Future<List<double>> loudness(int thing) {
    final lib = libPath, at = _m.address;
    return Isolate.run(() {
      final ask = c.MotoliiBindings(DynamicLibrary.open(lib)).motolii_loudness;
      final m = _P.fromAddress(at), n = ask(m, thing, nullptr, 0), out = malloc<Float>(n < 1 ? 1 : n);
      final got = ask(m, thing, out, n), all = [for (var i = 0; i < got && i < n; i++) out[i].toDouble()];
      malloc.free(out);
      return all;
    });
  }

  /// How loud a film's sound is at [n] moments of the document from [from] to before [to], where its film shows them
  /// (dB; -100 silent or not shown). null: its sound not read yet ([loudness] reads it); empty: no sound.
  Future<List<double>?> loudnessOver(int thing, double from, double to, int n) => _ask((l, m) {
    final out = malloc<Float>(n < 1 ? 1 : n);
    final got = c.MotoliiBindings(l).motolii_loudness_over(m, thing, from, to, n, out);
    final all = got < 0 ? null : [for (var i = 0; i < got; i++) out[i].toDouble()];
    malloc.free(out);
    return all;
  });

  /// Where a film's sound strikes (grows [rise] dB louder at once): the document's moments where it shows them, or
  /// [own] the film's own seconds. Asked once its loudness has been read ([loudness]).
  Future<List<double>> strikes(int thing, double rise, {bool own = false}) => _ask((l, m) {
    final ask = c.MotoliiBindings(l).motolii_strikes;
    final n = ask(m, thing, rise, own ? 1 : 0, nullptr, 0), out = malloc<Double>(n < 1 ? 1 : n);
    final got = ask(m, thing, rise, own ? 1 : 0, out, n), all = [for (var i = 0; i < got && i < n; i++) out[i]];
    malloc.free(out);
    return all;
  });

  /// A film's first picture, small, as a .png file (null: none could be made). Made once and on another isolate: the
  /// window does not wait.
  Future<String?> thumbnail(String film) => _thumbnails[film] ??= _thumbnail(libPath, film);

  /// A picture [step] of [steps] along the film (a tile looked along by the hand's place on it, or played).
  Future<String?> thumbnailAt(String film, int step, int steps) => step == 0 ? thumbnail(film) : _thumbnails['$film#$step/$steps'] ??= _thumbnail(libPath, film, step, steps);

  /// A ticket's answer when it comes, and why not if refused: Thermion's VoidCallbackRegistry, as it is (a Completer under
  /// each request's number, completed when the inside says an answer is there: motolii_tell 8). Nothing looks again and again.
  Future<(int, String)> answered(int ticket) {
    final a = answer(ticket);
    if (a >= 0) return Future.value((a, a >= 1 ? '' : answerWhy));
    return (_waiting[ticket] ??= Completer<(int, String)>()).future;
  }

  /// A ticket's answer: -1 not done yet, 0 refused ([answerWhy] says why), 1 done; said once.
  int answer(int ticket) {
    final a = _answer(_m, ticket);
    if (a >= 0) _take(3);   // (done: the pictures as the inside made them before it answered)
    return a;
  }

  String get answerWhy => _answerWhy(_m).toDartString();
}
