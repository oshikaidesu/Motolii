/// Motolii's chrome geometry: the sizes that change together because they are all "how much room the UI takes"
/// (a row of chrome, a control, the least a pointer can hit, the space between). Colours, faces (Create's marks,
/// Media's plates, the Timeline's reference frame) and the work's own geometry have other reasons to change and live
/// with their owners. The whole UI is also scaled as one by LiveUiScale (live_hf/ui_scale.dart); these are the
/// proportions at 100 %.
///
/// Benchmark: Rive's editor, measured from its public screenshots (APPROX): toolbar ~43, tabs / panel headers
/// 29-30, inspector row 31 (label and value on one line), boxed fields 28-29, buttons 30, menu rows 28, text 12-13
/// (scratchpad density/rive-measurements.md, 2026-09-28). Motolii keeps its own character; it takes the discipline.
abstract final class UiMetrics {
  /// A row of chrome: a tab strip, a panel or desk header, a sheet's title bar.
  static const chromeRow = 30.0;

  /// A header that also names its panel (a panel alone in its seat, a desk with its subtitle).
  static const namedHeader = 36.0;

  /// The application bar.
  static const topBar = 44.0;

  /// A control's drawn height: a value well, a key, a field, a segment.
  static const control = 26.0;

  /// The emphasised control of a row (a hero value).
  static const controlHero = 30.0;

  /// The least a pointer target is, however small its drawing (visual size and hit size are separate).
  static const hit = 24.0;

  /// A row of a menu.
  static const menuRow = 28.0;

  /// The Inspector's property cell: its label line, the gap under it, and the gap to the next cell.
  static const labelRow = 12.0, labelGap = 2.0, cellGap = 5.0;

  /// Space: inside a panel, between groups, between neighbours.
  static const pad = 12.0, gap = 8.0, tight = 4.0;
}
