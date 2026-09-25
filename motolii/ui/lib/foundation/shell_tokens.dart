import 'package:flutter/widgets.dart';

import 'theme.dart';

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

  // Geometry of the machine face.
  static const double ruleWidth = 1, activeRule = 2;
  static const double topBar = 32, header = 26, statusBar = 20;
  static const double browserWidth = 300, inspectorWidth = 320;
  static const double deskWidth = 320, bottomHeight = 300;
  static const double tabGap = 18, gutter = 10;
  static const double kicker = 10, wordmark = 15, tagline = 9;
  static const double tracking = .8, wordmarkTracking = -.2;

  static TextStyle kickerStyle(Color color) => TextStyle(
    fontFamily: EditorTheme.fontFamily,
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
