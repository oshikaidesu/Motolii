// The workspace: one window that gathers the lab's panels (browser dock, stage, inspector, timeline, desk).
// This file is what every seat shares: the mock document they all read (Ws), and the window's own tokens (WsT).
// Tone and manner come from the Inspector's Pop rules (sets/inspector_parts.dart): containers square and tiled edge to edge,
// what you touch or read round, colour flat and opaque, one lime accent, each kind of thing wearing its own hue.
import 'dart:developer' show Timeline, TimelineTask;
import 'dart:io' show Platform;
import 'package:flutter/scheduler.dart';
import 'package:flutter/foundation.dart' show mapEquals;
import 'package:flutter/widgets.dart';

import '../motolii.dart';
import '../theme.dart';
import '../tokens.dart';
import 'desk_kit.dart' show DeskTool, deskTools;
import 'stage_art.dart' show WsArt;
part 'ws_2.dart';
part 'ws_3.dart';

/// The window's own sizes and colours. A seat names one of these; it never types a number that is the window's business.
abstract final class WsT {
  /// The gutter between seats: the dark you see between tiles.
  static const gutter = 2.0;

  /// A seat's header strip (title, tabs, small actions).
  static double get header => Shape.z(28);

  /// The stage's tool column.
  static double get rail => Shape.z(40);

  /// Seat widths and the lower band's height, at the reference window (1600 x 960).
  static const dock = 300.0, inspector = 372.0, lower = 300.0;
  static double get topBar => Shape.z(40);
  static const minWindow = Size(1280, 800), refWindow = Size(1600, 960);

  /// Grounds: the gutter, a seat's body, a card or row inside it, a well (field) inside that.
  static Color get ground => Grey.g00;
  static Color get body => Grey.g07;
  static Color get card => Grey.g13;
  static Color get well => Grey.g15;
  static Color get raised => Grey.g20;
  static Color get line => Grey.g20;

  /// The one accent (same as the Inspector's) and the ink drawn on it.
  static Color get accent => Accent.fill;
  static Color get accentHot => Color.lerp(accent, const Color(0xFFFFFFFF), .14)!;
  static Color get onAccent => Accent.on;
  static Color get keyDot => Accent.key;

  /// The accent drawn as a line or text on a ground. Lime reads on dark but vanishes on white, so the light shade inks it darker; fills stay [accent].
  static Color get accentInk => Accent.ink;

  /// Round things you touch: fields, chips, tabs, buttons.
  static const radius = 6.0, chipRadius = 4.0;

  /// Paddings inside a seat.
  static double get inset => Shape.s(8);
  static double get gap => Shape.s(4);

  /// Hues per kind of layer / asset, shared by timeline bars, browser tiles and the stage's artwork, so a thing keeps its colour everywhere.
  static Color get toneShape => ColorTheme.of('Clip1', const Color(0xFFF08A5D));   // (a layer's kind: Ableton's clip colours)
  static Color get toneText => ColorTheme.of('Clip2', const Color(0xFFD9D2C3));
  static Color get toneImage => ColorTheme.of('Clip3', const Color(0xFF7FB8B0));
  static Color get toneCamera => ColorTheme.of('Clip4', const Color(0xFFA7B4C8));
  static Color get toneGroup => ColorTheme.of('Clip5', const Color(0xFFB0A6F0));
  static Color get toneParticles => ColorTheme.of('Clip6', const Color(0xFFF2A65A));
  static Color get toneAudio => ColorTheme.of('Clip7', const Color(0xFF5CC8E8));

  /// The playhead line: the accent, so "now" is the loudest line in the window. Its head is an [accent] fill.
  static Color get playhead => accentInk;
}

enum WsKind { shape, text, image, camera, group, particles, audio }

/// A thing's label colour, as Ableton gives each clip its own: one of sixteen, five apart from the last made so
/// neighbours differ, the same each time the document is opened; its row, its dot on the Relations map, its frame on
/// the stage and its name in a list wear it, so a photo or a model is told apart as well as a red dot. A skin may say
/// each (Label1 ... Label16).
Color wsLabel(String id) {
  const live = [0xFFFF94A6, 0xFFFFA529, 0xFFF7F47C, 0xFFBFFB00, 0xFF1AFF2F, 0xFF25FFA8, 0xFF5CFFE8, 0xFF8BC5FF, 0xFF5480E4, 0xFF92A7FF, 0xFFD86CE4, 0xFFE553A0, 0xFFFF3636, 0xFFF66C03, 0xFF10A4EE, 0xFF00BFAF];   // (Ableton Live's clip colours)
  final k = ((int.tryParse(id) ?? 0) * 5) % live.length;
  return ColorTheme.of('Label${k + 1}', Color(live[k]));
}

Color wsTone(WsKind k) => switch (k) {
  WsKind.shape => WsT.toneShape,
  WsKind.text => WsT.toneText,
  WsKind.image => WsT.toneImage,
  WsKind.camera => WsT.toneCamera,
  WsKind.group => WsT.toneGroup,
  WsKind.particles => WsT.toneParticles,
  WsKind.audio => WsT.toneAudio,
};

/// One layer of the mock composition: where it lives in time, and where its keys are (frames).
class WsLayer {
  WsLayer(this.id, this.name, this.kind, this.inF, this.outF, this.keys, {this.parent, this.hidden = false, this.solo = false, this.locked = false, this.stretches = const []});
  final String id, name;
  final WsKind kind;
  int inF, outF;
  final List<int> keys;
  final List<WsStretch> stretches; // (how it goes between its keys, as the inside says: drawn on its bar)
  final String? parent;
  final bool hidden, solo, locked; // (as the inside said when the layers were last read: a row asks nothing of it while the stage plays)
  Color get tone => wsLabel(id);   // (its label colour)
}

/// A stretch between two keys: its frames and how it goes there (the way's shape, 0 to 1, at 17 points along it).
typedef WsStretch = ({int from, int to, String way, List<double> shape});

/// A key the user picked: which layer, which frame.
typedef WsKey = ({String layer, int frame});

/// The window's state: the document's things as rows (read again from the inside), what is picked, the moment, the
/// view. Every seat listens to this and nothing else; a seat never keeps a copy of what is here.
/// contract: changes notify once per user action; nothing here ticks while idle (play advances only while [playing]).
/// What of the window changed: a part of it is built again only when one it reads did (Flutter's InheritedModel
/// aspects: [WsScope.of]). [WsAspect.document]: the inside's picture of the document was read again.
enum WsAspect { document, picked, playing, shelf, view, fronted, route, animating, notice, stage }
