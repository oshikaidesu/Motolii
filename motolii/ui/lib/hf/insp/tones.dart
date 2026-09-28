// Colour identifies a local thing (a group of parameters); it is not a widget taxonomy and carries no meaning.
// A small Flat Pop palette is dealt to groups in order, starting from a point fixed by the thing's id,
// so the same effect opens with the same colours and neighbouring groups never share one.
import 'package:flutter/painting.dart';
import '../desk/common.dart' show kYellow, kMint, kBlue, kPink, kViolet;
import '../neutral.dart';

const kOrange = Color(0xFFF08A3C);
const kTonePalette = [kYellow, kMint, kBlue, kPink, kViolet, kOrange];

int _fnv(String s) {
  var h = 0x811c9dc5;
  for (final c in s.codeUnits) { h ^= c; h = (h * 0x01000193) & 0xFFFFFFFF; }
  return h;
}

String groupKey(Map<String, dynamic> r) => r['advanced'] == true ? '\u0000advanced' : '${r['section'] ?? ''}';

class Tones {
  Tones(this.thing, List<Map<String, dynamic>> rows) {
    final base = _fnv(thing) % kTonePalette.length;
    for (final r in rows) {
      _by.putIfAbsent(groupKey(r), () => kTonePalette[(base + _by.length) % kTonePalette.length]);
    }
  }
  final String thing;
  final _by = <String, Color>{};
  Color ofGroup(String key) => _by[key] ?? kTonePalette.first;
  Color of(Map<String, dynamic> r) => ofGroup(groupKey(r));
  Color get advanced => ofGroup('\u0000advanced');
}

/// A tone that has been asked to step back (frozen): still recognisable, no longer a colour to touch.
Color dimTone(Color c) => Color.lerp(c, N.g26, .72)!;
