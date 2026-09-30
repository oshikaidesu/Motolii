
import 'rows.dart';

/// One number inside a row: the row's own value, or one axis of a vector. It also knows the little that the
/// row's character adds to a Value (units, whole steps, a finite percent), and which numbers move with it when linked.
class Slot {
  Slot(this.store, this.id, [this.axis, this.linkId]);
  final ParamStore store;
  final String id;
  final int? axis;
  final String? linkId; // set when this number belongs to a linkable group
  List<Slot> peers = const [];
  Map<String, dynamic> get row => store.row(id);
  Character get character => axis == null ? characterOf(row) : Character.none;
  bool get isAngle => character == Character.angle;
  bool get whole => const ['i32', 'u32', 'int'].contains('${row['kind']}') || ((character == Character.count || character == Character.seed) && value == value.roundToDouble());
  // surface: a unit multiplier (percent), not a length
  double get displayScale => character == Character.opacity || row['percent'] == true ? 100 : 1;
  String? get unit => (row['unit'] as String?) ?? (isAngle ? '°' : (displayScale == 100 ? '%' : null));
  bool get mixed => store.mixed(id, axis);

  /// A typed number is absolute for every target, unlike a scrub.
  void typed(double v) {
    store.typing = true;
    try {
      preview(v);
      commit();
    } finally {
      store.typing = false;
    }
  }

  double get value {
    final v = row['value'];
    return ((axis == null ? v : (v as List)[axis!]) as num).toDouble();
  }

  double? get def {
    final d = row['default'];
    if (d == null) return null;
    return ((axis == null ? d : (d as List)[axis!]) as num).toDouble();
  }

  void _raw(double v) {
    if (axis == null) {
      store.preview(id, v);
    } else {
      final l = List<Object?>.of(row['value'] as List);
      l[axis!] = v;
      store.preview(id, l);
    }
  }

  void preview(double v) {
    final old = value;
    _raw(v);
    // linked axes keep their ratio: one drag scales the whole group
    if (linkId != null && store.linked.contains(linkId) && old != 0) {
      final r = value / old;
      for (final p in peers) { p._raw(p.value * r); }
    }
  }

  void commit() => store.commit(id);
  void cancel() => store.cancel(id);
  void reset() {
    final d = def;
    if (d == null) return;
    preview(d);
    commit();
  }

  /// What a scrub lands on: whole degrees (tenths with Shift), whole counts and seeds.
  double quantize(double v, {bool fine = false}) => isAngle ? (fine ? (v * 10).round() / 10 : v.roundToDouble()) : (whole ? v.roundToDouble() : v);

  String format(double v, {int? decimals}) {
    if (row['zeroWord'] is String && v == 0) return row['zeroWord'] as String; // 0 rows means auto
    final shown = v * displayScale;
    if (decimals == 0) return '${shown.round()}'; // a narrow tile keeps its digits whole, as Classic does
    if (isAngle || displayScale != 1) return shown == shown.roundToDouble() ? '${shown.round()}' : shown.toStringAsFixed(1);
    return formatValue(v, whole: whole);
  }
}

String formatValue(double v, {bool whole = false}) {
  if (whole) return '${v.round()}';
  final a = v.abs();
  if (a != 0 && a < .001) return v.toStringAsExponential(2); // a small value must not read as zero
  return v.toStringAsFixed(a < 10 ? 3 : (a < 1000 ? 2 : 1));
}

