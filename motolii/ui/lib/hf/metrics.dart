import 'package:flutter/widgets.dart';

import 'neutral.dart';

/// Motolii's chrome geometry: the sizes that change together because they are all "how much room the UI takes"
/// (a row of chrome, a control, the least a pointer can hit, the space between). Colours, faces (Create's marks,
/// Media's plates, the Timeline's reference frame) and the work's own geometry have other reasons to change and live
/// with their owners. These are real pixels at the new 100 % (2026-09-29, the density pass): the whole UI is NOT
/// scaled to reach them (LiveUiScale is the user's own choice, on top). The pass was made on a MacBook's physical
/// size; the rules it follows, from Material and Apple density practice and Inter's own metrics:
///  * density shrinks heights and space, not text; text has a floor by role (below);
///  * small text gets more tracking (Inter's dynamic metrics), a sturdier weight, and a colour that is not dim;
///  * a small drawing keeps its pointer target ([hit]): visual size and interaction size are separate.
abstract final class UiMetrics {
  /// A row of chrome: a tab strip, a panel or desk header, a sheet's title bar.
  static const chromeRow = 24.0;

  /// A header that also names its panel (a panel alone in its seat, a desk with its subtitle).
  static const namedHeader = 28.0;

  /// The application bar.
  static const topBar = 32.0;

  /// A control's drawn height: a value well, a key, a field, a segment.
  static const control = 18.0;

  /// The emphasised control of a row (a hero value).
  static const controlHero = 22.0;

  /// The least a pointer target is, however small its drawing (visual size and hit size are separate).
  static const hit = 24.0;

  /// A row of a menu.
  static const menuRow = 22.0;

  /// The Inspector's property cell: its label line, the gap under it, and the gap to the next cell.
  static const labelRow = 11.0, labelGap = 2.0, cellGap = 3.0;

  /// Space: inside a panel, between groups, between neighbours.
  static const pad = 9.0, gap = 6.0, tight = 3.0;

  /// A row of a list (a layer, a property): the standard, the packed one, and a header.
  static const rowStd = 20.0, rowTight = 18.0, rowHead = 24.0;
}

/// The four text roles of the whole tool (px at the new 100 %). A surface names a role; it does not pick a size.
abstract final class Dn {
  static TextStyle _t(String f, double s, Color c, FontWeight w, double ls) => TextStyle(fontFamily: f, fontSize: s, color: c, fontWeight: w, letterSpacing: ls, height: 1);

  /// A name (a layer, an item, a title): 11, Medium.
  static TextStyle name([Color c = N.g91, FontWeight w = FontWeight.w500]) => _t('Inter', 11, c, w, .05);

  /// A value (a number, a time): 11, mono, so its width does not move.
  static TextStyle value([Color c = N.g91]) => _t('Menlo', 11, c, FontWeight.w400, 0);

  /// A label or secondary line: 10, and not dim (grey 69, not 56).
  static TextStyle label([Color c = N.g69, FontWeight w = FontWeight.w400]) => _t('Inter', 10, c, w, .12);

  /// A badge, a mark, a section's small caps: 9.5, Medium, the loosest tracking.
  static TextStyle micro([Color c = N.g76]) => _t('Inter', 9.5, c, FontWeight.w500, .15);

}
