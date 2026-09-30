// THE visual canon of the product window: every size, gap, radius, text size and grey is named, and a panel names a token; it never
// picks a number. Where each lives:
//   typography       Dn (this file): body 11 (nameSize), label 10, tiny 9.5 (microSize), numeric 11 mono; leading Dn.leading
//   density          Surface.topBar/namedHeader/chromeRow/workRow/control/controlHero/ruler/menuRow (rows and bars), Surface.hit (floor)
//   spacing, shape   Surface.hair, labelGap, inlineGap, sectionGap, panelInset; controlRadius, faceRadius; mark, glyph; faceRow, faceTile
//   surface colours  Surface.base/raised/hover/selected/divider/well/ink/muted/disabled, steps of the grey ramp N (neutral.dart)
//   semantic colours H (identity.dart), desks/parts.dart and inspector/tones.dart palettes
//   shared controls  EditorTheme (editor_theme.dart) is the ThemeExtension the controls read; liveEditorTheme is this window's instance
// Reach in this order: Surface role, Dn role, N step or Surface level, Surface.px(n) (a SCALE-policy custom dimension: an exception,
// registered in tool/motolii_lints/baseline.txt and ratcheted down, each new one with `// surface: <reason>`), a new token HERE after
// asking the owner. Geometry that belongs to the work (Stage, frames, thumbnails, painter and timeline reference coordinates,
// animation) is not a token: say so with `// surface: <reason>`. Retune here, not in panels.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import 'neutral.dart';

/// How a token follows the user's UI Scale. Every dimension token carries one; its value at 100 % is its base.
enum ScalePolicy {
  /// base x scale: the layout takes real, fractional logical pixels.
  scale,

  /// a hairline, border or divider: scaled, then snapped to whole physical pixels of the view (never thinner than one).
  snap,

  /// a hit bound: scaled, never below its floor.
  minimum,

  /// a face's size: scaled, kept between a fraction and a multiple of its base so the face stays readable.
  clamp,

  /// outside the UI Scale: the work (Stage, artwork, document, export) and time zoom. Says so; returns the value unchanged.
  fixed,
}

/// The one user scale, and the display it is read against. A token is a derived value: `f(base, scale, policy)` computed when it is
/// read (at build time), so a change of scale is a rebuild of the tree, not a transform of it. The setting (integer percent, 1 % steps,
/// an application preference, never part of a project) is kept by `app/ui_scale.dart`; this is only its value.
abstract final class UiScale {
  /// The provisional range of the setting (the scale sweep fixes it); the one place that says it.
  static const minPercent = 50, maxPercent = 200, resetPercent = 100;

  static int _percent = resetPercent;
  static double _dpr = 1;

  /// Notifies when the percent changes (the root rebuilds the whole tree on it).
  static final changes = ValueNotifier<int>(0);

  static int get percent => _percent;

  /// What a SCALE token is multiplied by: percent / 100.
  static double get factor => _percent / 100;
  static double get devicePixelRatio => _dpr;

  /// Sets the percent (clamped to the range, whole). True when it changed.
  static bool setPercent(int value) {
    final next = value.clamp(minPercent, maxPercent);
    if (next == _percent) return false;
    _percent = next;
    changes.value++;
    return true;
  }

  /// The view's pixel ratio, read by SNAP tokens. True when it changed (the root then builds the tree again: no notification here,
  /// it is called while building).
  static bool adoptDevicePixelRatio(double value) {
    if (value <= 0 || value == _dpr) return false;
    _dpr = value;
    return true;
  }

  /// [base] at 100 %, derived by [policy]. `floor` is MINIMUM's least; `lo` and `hi` are CLAMP's fractions and multiples of [base].
  static double derive(double base, ScalePolicy policy, {double floor = 0, double lo = 1, double hi = 1}) => switch (policy) {
        ScalePolicy.scale => base * factor,
        ScalePolicy.snap => base == 0 ? 0 : math.max(1, (base * factor * _dpr).round()) / _dpr,
        ScalePolicy.minimum => math.max(base * factor, floor),
        ScalePolicy.clamp => (base * factor).clamp(base * lo, base * hi).toDouble(),
        ScalePolicy.fixed => base,
      };
}

/// The Surface Grammar: the one place Motolii says how much room the UI takes and what a surface is made of. A panel names a
/// token; it does not pick a number. (Flutter does layout, input, text, focus, scroll and rendering; Motolii owns this grammar.)
///
/// Each dimension is a derived value (see [UiScale], [ScalePolicy]): the number written here is its base at UI Scale 100 %.
///
/// Two densities, never one global scale:
///  * Work density  (Timeline, Inspector, property rows, toolbars, menus, lists, numeric and relation editing): the rows below.
///  * Face density  (Media Browser, Effects, Create, Colors, Fonts, presets): a face gets the area it needs to be read; it is
///    sized by its own owner (Media's plates, Create's marks) and CLAMPed, not by [workRow].
///
/// Geometry that belongs to the work, not the chrome (the Stage, a video frame, a thumbnail's real size, the Timeline's reference
/// coordinates) is not a Surface token.
abstract final class Surface {
  /// A custom dimension: [base] pixels at 100 %, SCALE policy. The registered escape hatch for a size no token names: each use
  /// is counted per file by tool/motolii_lints (ratcheted down; a new one says why with `// surface: <reason>`).
  static double px(double base) => base * UiScale.factor;

  /// A size of the work, outside the UI Scale (FIXED policy): it says so where it is written.
  static double fixed(double value) => value;

  // ---- Density: rows and bars (logical px at 100 %)
  /// The application bar.
  static double get topBar => px(32);

  /// A header that also names its panel (a panel alone in its seat, a desk with its subtitle).
  static double get namedHeader => px(28);

  /// A row of chrome: a tab strip, a toolbar, a panel or desk header, a sheet's title bar.
  static double get chromeRow => px(24);

  /// A property row, a list row: one line of the work.
  static double get workRow => px(20);

  /// A control's drawn height when it sits in a packed row (a value well, a key, a segment, a timeline lane's bar).
  static double get control => px(18);

  /// The emphasised control of a row (a hero value).
  static double get controlHero => px(22);

  /// The Timeline's ruler.
  static double get ruler => px(20);

  /// A row of a menu.
  static double get menuRow => px(22);

  /// The least a pointer target is, however small its drawing (visual size and hit size are separate): MINIMUM, never below [hitFloor].
  static double get hit => UiScale.derive(24, ScalePolicy.minimum, floor: hitFloor);
  static const hitFloor = 18.0;

  // ---- Glyphs
  /// A state mark (the keyed diamond, the off-default dot) and a small glyph (power, menu, grip).
  static double get mark => px(5);
  static double get glyph => px(10);

  /// A divider / hairline and a focus stroke: SNAP to whole physical pixels.
  static double get hair => UiScale.derive(1, ScalePolicy.snap);
  static double get focusStroke => UiScale.derive(1.5, ScalePolicy.snap);

  /// The stacked Inspector cell (label line, gap, control): the form layout that Work density retires in favour of the inline
  /// row. Still read by the cells that have not moved.
  static double get labelRow => px(11);
  static double get cellGap => px(3);

  // ---- Spacing: between a label and its control, between neighbours in a row, between groups, and inside a panel.
  static double get labelGap => px(2);
  static double get inlineGap => px(3);
  static double get sectionGap => px(6);
  static double get panelInset => px(9);

  // ---- Shape: a control and a face are rounded; a panel and a section are not (no cards).
  static double get controlRadius => px(3);
  static double get faceRadius => px(4.5);

  // ---- Face density: CLAMP (a face is scaled, never below 3/4 of its base nor above 1.5x of it).
  /// A Media row and a Create tile's height.
  static double get faceRow => UiScale.derive(32, ScalePolicy.clamp, lo: .75, hi: 1.5);
  static double get faceTile => UiScale.derive(40, ScalePolicy.clamp, lo: .75, hi: 1.5);

  // ---- Surface levels (darkest first): what a region is, by level, not by a box around it.
  static const base = N.g10, raised = N.g13, hover = N.g15, selected = N.g20, divider = N.g20, dividerFine = N.g15, well = N.g07;
  static const disabled = N.g44, muted = N.g56, ink = N.g95;
}

/// The text roles of the whole tool (px at UI Scale 100 %). A surface names a role; it does not pick a size. The size scales
/// with the UI; the line box follows it (a constant [leading] ratio), so the baseline and the paddings stay in their measured relation.
abstract final class Dn {
  /// The sizes, for the helpers that take one (`sans(Dn.nameSize, …)`): body 11, label 10, tiny 9.5, numeric 11.
  static double get nameSize => Surface.px(11);
  static double get labelSize => Surface.px(10);
  static double get microSize => Surface.px(9.5);
  static double get numericSize => Surface.px(11);

  /// The line box as a multiple of the font size: Dn's and H's styles, and the Browser family's ([leadingText]). Ratios do not scale.
  static const leading = 1.0, leadingText = 1.1;

  static TextStyle _t(String f, double s, Color c, FontWeight w, double ls) => TextStyle(fontFamily: f, fontSize: s, color: c, fontWeight: w, letterSpacing: ls, height: leading);

  /// A name (a layer, an item, a title): body, Medium.
  static TextStyle name([Color c = N.g91, FontWeight w = FontWeight.w500]) => _t('Inter', nameSize, c, w, .05);

  /// A value (a number, a time): numeric, mono, so its width does not move.
  static TextStyle value([Color c = N.g91]) => _t('Menlo', numericSize, c, FontWeight.w400, 0);

  /// A label or secondary line: label, and not dim (grey 69, not 56).
  static TextStyle label([Color c = N.g69, FontWeight w = FontWeight.w400]) => _t('Inter', labelSize, c, w, .12);

  /// A badge, a mark, a section's small caps: tiny, Medium, the loosest tracking.
  static TextStyle micro([Color c = N.g76]) => _t('Inter', microSize, c, FontWeight.w500, .15);
}
