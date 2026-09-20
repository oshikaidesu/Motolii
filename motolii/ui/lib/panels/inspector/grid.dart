part of '../inspector.dart';

/// How wide everything is: the panel's columns, the cell the zoom strip
/// sets, and the two value classes a line and a card grid are laid out by.
mixin _InspectorGrid on _InspectorReading {
  /// Two wells side by side in one cell, and the wells beside a pad.
  double get _half => (_cellWidth - EditorMetrics.s4) / 2;

  double get _beside => _cellWidth - EditorMetrics.s60 - EditorMetrics.s4;

  _InspectorColumns _columns = _InspectorColumns(
    EditorMetrics.sheet,
    EditorMetrics.cell,
  );

  double get _wellWidth => _columns.division(3);

  double get _wordWidth => _columns.named ? EditorMetrics.s48 : 0;

  double get _cardWidth => _columns.content;

  void _fit(double width) {
    if (_columns.width != width || _columns.target != _preferredCellWidth) {
      _columns = _InspectorColumns(width, _preferredCellWidth);
    }
  }

  /// The cell width the user set at the panel's foot (the zoom strip);
  /// EditorMetrics.cell until touched.
  double get _preferredCellWidth =>
      (c.deskWork.value['inspectorCell'] as num? ?? EditorMetrics.cell)
          .toDouble()
          .clamp(InspectorCell.min, InspectorCell.max);

  double get _cellWidth => _columns.tile;
}

/// How wide a card cell may be: narrow packs three columns into a dock,
/// wide gives one number a whole row.
abstract final class InspectorCell {
  static const double min = 88, max = 200;
}

class _InspectorColumns {
  _InspectorColumns(this.width, this.target) {
    content = math.max(0, width - EditorCard.contentInset * 2);
    named =
        content >=
        EditorMetrics.s18 +
            EditorMetrics.s48 +
            accessory +
            gap * 3 +
            EditorMetrics.s44 * 3;
    label = named ? EditorMetrics.s18 + EditorMetrics.s48 : EditorMetrics.s18;
    values = math.max(0, content - label - gap - accessory);
    final count = math.max(1, ((content + gap) / (target + gap)).floor());
    tile = math.max(0, (content - gap * (count - 1)) / count);
  }
  static const gap = EditorMetrics.s4;
  static const accessory = EditorMetrics.s22;
  static const rowHeight = EditorMetrics.s22;
  final double width, target;
  late final double content, label, values, tile;
  late final bool named;
  double division(int count) =>
      math.max(0, (values - gap * (count - 1)) / count);
  double span(int count, {int divisions = 3}) =>
      division(divisions) * count + gap * (count - 1);
}

class _RowCell {
  const _RowCell(
    this.child, {
    this.span = 1,
    this.center = false,
    this.tall = false,
  });
  final Widget? child;
  final int span;
  final bool center, tall;
}

/// One cell of a card's grid: a control, a tall one (a pad), or a wide
/// section label.
class _Cell {
  const _Cell(this.child, {this.tall = false, this.wide = false});
  final Widget child;
  final bool tall, wide;
}
