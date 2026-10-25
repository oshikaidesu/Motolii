// motolii_shelves.dart: what the maker has, read once (skin/BRIDGE.md S): the recipes, effects and motions on the shelves, their labels and pictures, a font's kinds, the kinds of way, what a value is shown as.
part of 'motolii.dart';

extension MotoliiShelves on Motolii {
  /// Pictures in one call: each recipe's name and what it makes.
  List<(String, String)> get recipes => _folders.putIfAbsent('recipes', () => [
    for (final line in _recipes(_m).toDartString().split('\n'))
      if (line.indexOf('\t') case final at when at > 0) (line.substring(0, at), line.substring(at + 1)),
  ]);

  /// The whole-screen effects on the shelf: each one's name and file (the stage's script is the files of those on it).
  List<(String, String)> get effects => _folders.putIfAbsent('effects', () => _labelled(_effects(_m).toDartString()));

  /// The motions a thing may be given (Cavalry's Animators): each one's name and file (a thing's script is the file of the one it has).
  List<(String, String)> get animators => _folders.putIfAbsent('animators', () => _labelled(_animators(_m).toDartString()));

  /// What an effect (of a thing or of the whole screen) or a motion is for, as the inside says: the shelf's kind ('' none said).
  String label(String name) {
    _said();
    return _labels[name] ?? '';
  }

  /// An effect's (or a motion's) picture for its tile, as the inside says: what it does to a sample ('' none).
  String picture(String name) {
    _said();
    return _pictures[name] ?? '';
  }

  void _said() => (effects, thingEffects);   // (both lists read: their labels and pictures with them)

  /// "name\tfile\tlabel\tpicture" lines: the names and files, the labels and pictures kept for [label] and [picture].
  List<(String, String)> _labelled(String lines) {
    final out = <(String, String)>[];
    for (final w in [for (final l in lines.split('\n')) l.split('\t')]) {
      if (w.length < 2 || w[0].isEmpty) continue;
      out.add((w[0], w[1]));
      if (w.length > 2) _labels[w[0]] = w[2];
      if (w.length > 3) _pictures[w[0]] = w[3];
    }
    return out;
  }

  /// How a value is shown to a hand, as the inside says (null: as its range says): its look, the unit it is read in,
  /// its name in plain words, the span its knob covers (in the document's units) and the fold it lies in.
  Shown? shown(String name) => _shownOf.putIfAbsent(name, () {
    for (final part in _thing.values) {
      if ((part['shown'] as Map<String, dynamic>?)?[name] case final String said) {
        final f = said.split('\t');
        return f.length < 6 ? null : Shown(f[0], f[1], f[2], double.parse(f[3]), double.parse(f[4]), f[5].isEmpty ? null : f[5]);
      }
    }
    return null;
  });

  /// A font file's kinds as CoreText reads them (serif, sans, script, mono, bold, light…): the system's own, not guessed from a name.
  Set<String> fontTags(String path) => _fontKinds[path] ??= _with([path], (s) => _fontTags(s[0]).toDartString()).split(' ').where((w) => w.isNotEmpty).toSet();

  /// A kind's numbers as a hand takes them: each one's name, lowest and highest (none: a kind without, or one defined).
  List<(String, double, double)> wayNumbers(String kind) {
    final w = _with([kind], (s) => _wayNumbers(_m, s[0]).toDartString()).split(' ').where((x) => x.isNotEmpty).toList();
    return [for (var i = 0; i + 2 < w.length; i += 3) (w[i], double.parse(w[i + 1]), double.parse(w[i + 2]))];
  }

  /// What kind of thing a file makes ('': none).
  String kindOf(String file) => _with([file], (s) => _kindOf(s[0]).toDartString());
}
