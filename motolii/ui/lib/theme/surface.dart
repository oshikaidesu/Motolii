import 'package:flutter/widgets.dart';

import 'neutral.dart';

/// The Surface Grammar: the one place Motolii says how much room the UI takes and what a surface is made of. A panel names a
/// token; it does not pick a number. (Flutter does layout, input, text, focus, scroll and rendering; Motolii owns this grammar.)
///
/// It is read off the real window and Ableton Live as a density oracle, not copied: work surfaces are maximised; sections are
/// separated by a line or a level change, never a card; a label is structure (inline, left of its control), not a header over a
/// gap over a control; colour means identity, state or selection. The pass was made on a MacBook's physical size at the new 100 %
/// (2026-09-29; LiveUiScale is the user's own choice, on top): the whole UI is NOT scaled to reach these numbers.
///
/// Two densities, never one global scale:
///  * Work density  (Timeline, Inspector, property rows, toolbars, menus, lists, numeric and relation editing): the rows below.
///  * Face density  (Media Browser, Effects, Create, Colors, Fonts, presets): a face gets the area it needs to be read; it is
///    sized by its own owner (Media's plates, Create's marks), not by [workRow].
///
/// Geometry that belongs to the work, not the chrome (the Stage, a video frame, a thumbnail's real size, the Timeline's reference
/// coordinates) is not a Surface token.
abstract final class Surface {
  // ---- Work density: geometry (real pixels)
  /// The application bar.
  static const topBar = 32.0;

  /// A header that also names its panel (a panel alone in its seat, a desk with its subtitle).
  static const namedHeader = 28.0;

  /// A row of chrome: a tab strip, a toolbar, a panel or desk header, a sheet's title bar.
  static const chromeRow = 24.0;

  /// A property row, a list row: one line of the work.
  static const workRow = 20.0;

  /// A control's drawn height when it sits in a packed row (a value well, a key, a segment, a timeline lane's bar).
  static const control = 18.0;

  /// The emphasised control of a row (a hero value).
  static const controlHero = 22.0;

  /// The least a pointer target is, however small its drawing (visual size and hit size are separate).
  static const hit = 24.0;

  /// A row of a menu.
  static const menuRow = 22.0;

  /// A state mark (the keyed diamond, the off-default dot) and a small glyph (power, menu, grip).
  static const mark = 5.0, glyph = 10.0;

  /// A divider / hairline and a focus stroke.
  static const hair = 1.0, focusStroke = 1.5;

  /// The stacked Inspector cell (label line, gap, control): the form layout that Work density retires in favour of the inline
  /// row. Still read by the cells that have not moved.
  static const labelRow = 11.0, labelGap = 2.0, cellGap = 3.0;

  // ---- Spacing: three steps. Between neighbours in a row, between groups, and inside a panel.
  static const inlineGap = 3.0, sectionGap = 6.0, panelInset = 9.0;

  // ---- Shape: a control and a face are rounded; a panel and a section are not (no cards).
  static const controlRadius = 3.0, faceRadius = 4.5;

  // ---- Surface levels (darkest first): what a region is, by level, not by a box around it.
  static const base = N.g10, raised = N.g13, hover = N.g15, selected = N.g20, divider = N.g20, dividerFine = N.g15, well = N.g07;
  static const disabled = N.g44, muted = N.g56, ink = N.g95;
}

/// The four text roles of the whole tool (px at the new 100 %). A surface names a role; it does not pick a size.
abstract final class Dn {
  /// The sizes, for the helpers that take one (`sans(Dn.nameSize, …)`): name 11, label 10, micro 9.5.
  static const nameSize = 11.0, labelSize = 10.0, microSize = 9.5;

  static TextStyle _t(String f, double s, Color c, FontWeight w, double ls) => TextStyle(fontFamily: f, fontSize: s, color: c, fontWeight: w, letterSpacing: ls, height: 1);

  /// A name (a layer, an item, a title): 11, Medium.
  static TextStyle name([Color c = N.g91, FontWeight w = FontWeight.w500]) => _t('Inter', nameSize, c, w, .05);

  /// A value (a number, a time): 11, mono, so its width does not move.
  static TextStyle value([Color c = N.g91]) => _t('Menlo', nameSize, c, FontWeight.w400, 0);

  /// A label or secondary line: 10, and not dim (grey 69, not 56).
  static TextStyle label([Color c = N.g69, FontWeight w = FontWeight.w400]) => _t('Inter', labelSize, c, w, .12);

  /// A badge, a mark, a section's small caps: 9.5, Medium, the loosest tracking.
  static TextStyle micro([Color c = N.g76]) => _t('Inter', microSize, c, FontWeight.w500, .15);
}
