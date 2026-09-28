import 'dart:ui' show Color;

/// Motolii's neutral ramp: every grey, black and white the hf faces draw, named by lightness (g10 = 10 %). The base
/// is achromatic; colour is for identity (things, relations, modes) and lives with its owners, never in the ramp.
/// Semantic tokens (H.window, H.rule, …) are steps of this ramp; a face asks for a role first and a step only when
/// no role says what it is. Nothing else in hf / live_hf writes a grey as a hex.
abstract final class N {
  static const g00 = Color(0xFF000000);
  static const g07 = Color(0xFF131313); // wells: darker than the ground
  static const g10 = Color(0xFF191919); // ground
  static const g13 = Color(0xFF202020); // raised
  static const g15 = Color(0xFF262626); // raised, hover
  static const g20 = Color(0xFF343434); // rules, selected fills
  static const g26 = Color(0xFF424242);
  static const g33 = Color(0xFF555555);
  static const g38 = Color(0xFF616161);
  static const g44 = Color(0xFF717171);
  static const g51 = Color(0xFF818181);
  static const g56 = Color(0xFF8E8E8E); // muted
  static const g63 = Color(0xFFA1A1A1); // tertiary text
  static const g69 = Color(0xFFB1B1B1);
  static const g76 = Color(0xFFC1C1C1);
  static const g82 = Color(0xFFD0D0D0); // secondary text
  static const g86 = Color(0xFFDBDBDB);
  static const g91 = Color(0xFFE7E7E7);
  static const g95 = Color(0xFFF2F2F2); // primary text
  static const g100 = Color(0xFFFFFFFF);

  // the ramp seen through: nothing, shades of black over the work, glazes of white, the ground as a veil
  static const clear = Color(0x00000000);
  static const shade40 = Color(0x66000000);
  static const shade55 = Color(0x88000000);
  static const shade90 = Color(0xE6000000);
  static const glaze0 = Color(0x00FFFFFF); // the end of a white fade
  static const glaze4 = Color(0x0AFFFFFF), glaze9 = Color(0x17FFFFFF); // grid lines over the tracks
  static const glaze15 = Color(0x29FFFFFF);
  static const glaze28 = Color(0x47FFFFFF);
  static const glaze80 = Color(0xCCFFFFFF);
  static const veil = Color(0xCC191919); // a label over a picture
  static const veilHi = Color(0xE6202020); // a readout over the work
  static const inkSoft = Color(0xB3191919); // dark ink at 70 % on a colour
}
