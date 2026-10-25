// motolii_watch.dart: how the inside is doing, for a check or a person looking: why the last call was refused, what it holds, what its calls cost, when the window waited on it, how many pictures it drew.
part of 'motolii.dart';

extension MotoliiWatch on Motolii {
  String get trouble => _trouble(_m).toDartString();

  String get list => _list(_m).toDartString();

  /// How many pictures a second a film written of the document has: what a frame is.
  int get rate => _rate(_m);

  int get frames => _frames(_m); // (how many pictures the live stage has drawn: motolii_frames)

  /// The stage's pictures kept on disk for playing, as stretches of frames [from, to) (motolii_kept).
  List<(int, int)> get kept {
    final n = _keptRuns(_m).toDartString().split(' ').where((w) => w.isNotEmpty).map(int.parse).toList();
    return [for (var i = 0; i + 1 < n.length; i += 2) (n[i], n[i + 1])];
  }

  /// What the inside's hand has spent its time on (for measuring).
  String get spent => _spent(_m).toDartString();

  void watch(bool on) => _watch(_m, on ? 1 : 0);

  String get broken => _broken(_m).toDartString();

  int get brokenCount => _brokenCount(_m);   // (how many times since watched: read without waiting)
}
