// The document's beats, what the timeline's grid goes by when it has a tempo (Live's Tempo field and Editing Grid: its
// manual, 6.10). Four to a bar, Live's first time signature. Alight Motion has no such grid.
import '../tokens.dart' show halfWidth;

/// A number as it was typed (null: none).
double? wsNumber(String said) => double.tryParse(halfWidth(said).trim());

class WsBeats {
  const WsBeats(this.first, this.every);
  final double first, every;   // (in frames: where a bar begins, and from one beat to the next)

  static WsBeats? of(double bpm, double from, int fps) => bpm > 0 ? WsBeats(from * fps, 60 / bpm * fps) : null;

  /// The beats from a frame to a frame, each with whether a bar begins there.
  Iterable<(double, bool)> between(double a, double b) sync* {
    for (var n = ((a - first) / every).ceil(); first + n * every <= b; n++) {
      yield (first + n * every, n % 4 == 0);
    }
  }

  /// The frame of the beat nearest a frame.
  int nearest(int frame) => (first + ((frame - first) / every).round() * every).round();
}
