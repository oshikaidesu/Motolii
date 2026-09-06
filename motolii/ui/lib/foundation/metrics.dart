/// The metric scale. Every size, gap, radius and font size in the UI is one
/// of these names; `raw_dimension` (tool/motolii_lints) refuses a bare number
/// in the IDE, `dart run bin/check.dart` refuses it in tests, and the fix
/// swaps in the token of the same value from this class.
///
/// Retune the UI here, not in panels.
abstract final class EditorMetrics {
  // Rows, bars and type come first so a matching value takes the meaning.
  static const double row = 20, control = 24, section = 26, bar = 28, tall = 30;
  static const double micro = 9, dense = 10, font = 11, title = 13;
  static const double field = 52, thumb = 128, sheet = 320, sheetWide = 420;
  static const double canvas = 5000;

  // Spacing and size steps.
  static const double s2 = 2, s3 = 3, s4 = 4, s6 = 6, s8 = 8, s12 = 12;
  static const double s16 = 16, s32 = 32, s48 = 48;

  // Off-scale, lifted as-is from the panels on 2026-09-06. Each is a snap
  // candidate: change the value here to try, delete the name once unused.
  static const double s5 = 5, s7 = 7, s14 = 14, s15 = 15, s17 = 17, s18 = 18;
  static const double s19 = 19, s22 = 22, s23 = 23, s34 = 34, s36 = 36;
  static const double s44 = 44, s60 = 60, s70 = 70, s76 = 76, s78 = 78;
  static const double s85 = 85, s90 = 90, s96 = 96, s155 = 155, s160 = 160;
  static const double s200 = 200, s244 = 244, s280 = 280;
}
