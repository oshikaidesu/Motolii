/// Layout measurements shared by Timeline presentations.
class TimelineGeometry {
  static const classicRowHeight = 20.0;
  static const classicRulerHeight = 32.0;
  static const classicRulerTop = 22.0;
  static const classicIndentStep = 8.0;
  static const classicDisclosureWidth = 20.0;
  static const classicNameWidth = 138.0;
  static const classicNameWidthMin = 114.0;
  static const classicNameWidthMax = 162.0;
  const TimelineGeometry({
    required this.rowHeight,
    required this.rulerHeight,
    this.rulerTop = classicRulerTop,
    required this.indentStep,
    required this.disclosureWidth,
    required this.nameWidth,
    required this.nameWidthMin,
    required this.nameWidthMax,
  });

  static const classic = TimelineGeometry(
    rowHeight: classicRowHeight,
    rulerHeight: classicRulerHeight,
    indentStep: classicIndentStep,
    disclosureWidth: classicDisclosureWidth,
    nameWidth: classicNameWidth,
    nameWidthMin: classicNameWidthMin,
    nameWidthMax: classicNameWidthMax,
  );

  static const hf = TimelineGeometry(
    rowHeight: 23, // the hf row pitch (`tlPitch`): rows are hit-tested where they are drawn
    rulerHeight: 31,
    rulerTop: 40, // hf's ruler band, y 743-775 in the reference frame, 703 at the seat top
    indentStep: 14,
    disclosureWidth: 20,
    nameWidth: 133,
    nameWidthMin: 133,
    nameWidthMax: 133,
  );

  final double rowHeight;
  final double rulerHeight;

  /// Where the time ruler starts, from the top of the face: the band where a wheel or a pinch zooms instead of
  /// panning.
  final double rulerTop;
  final double indentStep;
  final double disclosureWidth;
  final double nameWidth;
  final double nameWidthMin;
  final double nameWidthMax;
}
