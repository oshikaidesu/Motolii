// The Layout of a Group (or of a child inside one), as Classic's Layout card owns it.
// Kept: Grid gates everything but Columns / Rows; Justify and Align are two rows of one pad (Start / Center / End);
// Hug / Fill / Fixed per axis, and the size number counts under Fixed alone; the child's own lines; the rest behind Advanced.
import 'rows.dart';

const justifyNames = ['Start', 'End', 'Center', 'Between', 'Around', 'Evenly'];
const alignNames = ['Stretch', 'Start', 'End', 'Center'];
const sizingNames = ['Hug', 'Fill', 'Fixed'];

class LayoutStore extends ParamStore {
  LayoutStore({this.child = false, this.children = 6, bool frozen = false}) : super(const [], frozen: frozen) {
    rows.addAll(child ? _childRows() : _groupRows());
  }
  final bool child; // the shown layer is a child inside a laid-out group
  final int children; // how many children the group has (the diagram shows this many boxes)

  static Map<String, dynamic> _r(String id, String label, Object? v, Object? def, {String kind = 'f32', Map<String, dynamic>? more}) => {'id': id, 'label': label, 'kind': kind, 'value': v, 'default': def, ...?more};

  List<Map<String, dynamic>> _groupRows() => [
        _r('layout.display', 'Grid', 2, 0, kind: 'i32'),
        _r('layout.grid_columns', 'Columns', 3, 1, kind: 'i32', more: {'min': 1, 'max': 12}),
        _r('layout.grid_rows', 'Rows', 0, 0, kind: 'i32', more: {'min': 0, 'max': 12, 'zeroWord': 'auto'}),
        _r('layout.gap', 'Gap', 12.0, 0.0, more: {'unit': 'px', 'min': 0.0, 'max': 400.0, 'speed': 1.0}),
        _r('layout.padding', 'Padding', [16.0, 16.0], [0.0, 0.0], kind: 'vec2', more: {'unit': 'px', 'speed': 1.0}),
        _r('layout.justify_content', 'Justify', 0, 0, kind: 'i32', more: {'choices': justifyNames}),
        _r('layout.align_items', 'Align', 1, 0, kind: 'i32', more: {'choices': alignNames}),
        _r('layout.horizontal_sizing', 'Width', 2, 0, kind: 'i32', more: {'choices': sizingNames}),
        _r('layout.vertical_sizing', 'Height', 0, 0, kind: 'i32', more: {'choices': sizingNames}),
        _r('layout.width', 'Width', 360.0, 100.0, more: {'unit': 'px', 'min': 0.0, 'speed': 1.0}),
        _r('layout.height', 'Height', 240.0, 100.0, more: {'unit': 'px', 'min': 0.0, 'speed': 1.0}),
        _r('layout.transition_duration', 'Duration', .3, 0.0, more: {'unit': 's', 'min': 0.0}),
        _r('layout.transition_easing', 'Easing', 3, 0, kind: 'i32', more: {'choices': ['Linear', 'In', 'Out', 'In Out']}),
        // the rest of the layout rows, whatever the document declares, stay reachable behind Advanced
        _r('layout.min_width', 'Min width', 0.0, 0.0, more: {'unit': 'px', 'advanced': true, 'min': 0.0}),
        _r('layout.max_width', 'Max width', 0.0, 0.0, more: {'unit': 'px', 'advanced': true, 'min': 0.0, 'zeroWord': 'none'}),
      ];

  List<Map<String, dynamic>> _childRows() => [
        _r('layout.position_type', 'Ignore layout', 0, 0, kind: 'i32'),
        _r('layout.horizontal_sizing', 'Width', 1, 0, kind: 'i32', more: {'choices': sizingNames}),
        _r('layout.vertical_sizing', 'Height', 0, 0, kind: 'i32', more: {'choices': sizingNames}),
        _r('layout.width', 'Width', 120.0, 100.0, more: {'unit': 'px', 'min': 0.0, 'speed': 1.0}),
        _r('layout.height', 'Height', 80.0, 100.0, more: {'unit': 'px', 'min': 0.0, 'speed': 1.0}),
        _r('layout.column_start', 'Column start', 1, 1, kind: 'i32', more: {'min': 1, 'max': 12}),
        _r('layout.row_start', 'Row start', 1, 1, kind: 'i32', more: {'min': 1, 'max': 12}),
        _r('layout.column_span', 'Column span', 1, 1, kind: 'i32', more: {'min': 1, 'max': 12}),
        _r('layout.row_span', 'Row span', 1, 1, kind: 'i32', more: {'min': 1, 'max': 12}),
      ];

  int gi(String id) => ((row(id)['value'] as num?) ?? 0).round();
  double gd(String id) => ((row(id)['value'] as num?) ?? 0).toDouble();
  double get padX => ((row('layout.padding')['value'] as List)[0] as num).toDouble();
  double get padY => ((row('layout.padding')['value'] as List)[1] as num).toDouble();
  /// The Grid switch: on only for Grid (2). A Flex group (1) shows it off, so turning it off never undoes Flex.
  bool get gridOn => gi('layout.display') == 2;

  /// Arranged at all (Flex or Grid): gap, padding and the diagram's arrangement apply.
  bool get arranged => gi('layout.display') != 0;
  bool fixed(String axis) => gi('layout.${axis == 'w' ? 'horizontal' : 'vertical'}_sizing') == 2;

  /// Several rows as one edit: a gesture that changes Justify and Align together is one commit, as Classic's pad is.
  void applyMany(Map<String, Object?> values) {
    if (frozen) return;
    for (final e in values.entries) { preview(e.key, e.value); }
  }

  void setGrid(bool on) => set('layout.display', on ? 2 : 0);
  void setSizing(String axis, int v) => set('layout.${axis == 'w' ? 'horizontal' : 'vertical'}_sizing', v);
}
