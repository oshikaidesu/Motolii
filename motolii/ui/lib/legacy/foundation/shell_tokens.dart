import 'package:flutter/widgets.dart';

import '../../theme/editor_theme.dart';

/// The New shell's look: near-black ground, thin rules, and six Flat Pop
/// accents that only ever mean something (on-state, a layer's identity, a
/// family of number). Classic never reads this file; it keeps
/// [EditorTheme.chromatic] and the user's theme JSON.
abstract final class ShellTokens {
  // Accents — one hue, one job.
  static const pink = Color(0xffff6fb5);
  static const mint = Color(0xff54d69a);
  static const sky = Color(0xff4da3ff);
  static const lemon = Color(0xfff5d443);
  static const peach = Color(0xffff9a52);
  static const lavender = Color(0xffa98bff);

  // Surfaces, darkest first, and the rules between them.
  static const ground = Color(0xff0e0f11);
  static const surface = Color(0xff16171a);
  static const raised = Color(0xff1f2024);
  static const hover = Color(0xff2a2b30);
  static const rule = Color(0xff2a2c31);
  static const ruleStrong = Color(0xff3a3c42);
  static const ink = Color(0xffededed);
  static const inkMuted = Color(0xff8e9097);
  static const inkFaint = Color(0xff5e6066);
  static const inkOnAccent = Color(0xff0b0c0e);

  /// The instrument face: labels and readouts.
  static const mono = 'Menlo';

  // Geometry of the machine face.
  static const double ruleWidth = 1, activeRule = 2;
  static const double topBar = 44, header = 26, statusBar = 20;
  static const double key = 28, keyGlyph = 12, readoutGap = 14;

  // The Browser's cells: a square mark tile, a picture tile, the gap
  // between cells, the head of a group, the mark inside a tile.
  static const double groupHead = 28, mark = 22, field = 26;
  static const double browserWidth = 300, inspectorWidth = 320;
  static const double deskWidth = 320, bottomHeight = 300;
  static const double tabGap = 18, gutter = 10;
  static const double kicker = 10, wordmark = 20, readout = 12;
  static const double tracking = 1, wordmarkTracking = -.4;

  static TextStyle readoutStyle(Color color) => TextStyle(
    fontFamily: mono,
    fontSize: readout,
    fontWeight: FontWeight.w500,
    color: color,
    fontFeatures: const [FontFeature.tabularFigures()],
  );

  static TextStyle kickerStyle(Color color) => TextStyle(
    fontFamily: mono,
    fontSize: kicker,
    fontWeight: FontWeight.w600,
    letterSpacing: tracking,
    color: color,
    fontFeatures: const [FontFeature.tabularFigures()],
  );

  static final EditorTheme theme = EditorTheme.chromatic.copyWith(
    name: 'Motolii New',
    colors: const {
      'app': ground,
      'panel': surface,
      'raised': raised,
      'hover': hover,
      'line': rule,
      'border': ruleStrong,
      'ink': ink,
      'muted': inkMuted,
      'disabledInk': inkFaint,
      'accent': sky,
      'animate': pink,
      'tab': sky,
      'tabInk': inkOnAccent,
      'menu': surface,
      'menuEdge': ruleStrong,
      'select': ink,
      'selectInk': inkOnAccent,
      'caret': sky,
      'selection': Color(0x664da3ff),
      'spatial': Color(0xff5ab4ff),
      'amount': peach,
      'time': lavender,
      'count': Color(0xff4fd8e8),
      'seed': lemon,
      'angle': Color(0xff7ee081),
      'timelineWash': Color(0x26000000),
    },
    identityColors: const [pink, sky, mint, lemon, peach, lavender],
    skin: const EditorSkin(
      radius: 2,
      tileRadius: 0,
      labelFamily: mono,
      uppercaseLabels: true,
      identityChip: true,
      clipInset: 4,
      tabular: true,
      outlinedTiles: true,
      flatWells: true,
      newFace: true,
    ),
    drawing: EditorInk.dark.copyWith(
      laneGround: Color(0xff121316),
      lane: Color(0xff15161a),
      laneAlt: Color(0xff191a1e),
      grid: Color(0xff0a0b0d),
      gridMinor: Color(0xff1c1d21),
      tick: Color(0xff9a9ca3),
      tickMinor: Color(0xff6b6d74),
      headerInk: inkOnAccent,
      focusRing: ink,
    ),
  );
}
