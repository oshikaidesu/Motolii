// The Timeline seat of the reference, at its own coordinates. Layers, bottom to top: panel ground, alternating row
// ground, hierarchy, ruler, major/minor time grid, temporal bodies, keyframes, playhead, waveform (a floor, not a
// body). Row height and bar height are separate tokens; the bar leaves a fixed negative space. What the rows say, where
// bodies and keys sit and where the playhead is come from a [TimelineModel], in the reference's own x.
import 'dart:math' as math;

import 'package:flutter/gestures.dart'
    show
        PointerCancelEvent,
        PointerDownEvent,
        PointerMoveEvent,
        PointerScrollEvent,
        PointerUpEvent;
import 'package:flutter/services.dart' show HardwareKeyboard;
import 'package:flutter/widgets.dart';

import '../glyphs.dart';
import 'place.dart';

part 'timeline_paint.dart';

const tlTop = 775.0, tlPitch = 23.0, tlRowH = 22.0, tlBarH = 18.0;
const tlAudioTop = 959.0, tlAudioH = 34.0;
const tlX0 = 559.0, tlUnit = 92.9, tlRight = 1521.0;

/// Time unit -> x.
double tlX(double t) => tlX0 + 0.5 + tlUnit * t;

enum TlKind { group, item, camera, audio }

class TlRow {
  const TlRow(
    this.name,
    this.kind, {
    this.chip,
    this.body,
    this.keys = const [],
    this.nameW,
    this.indent = 0,
    this.open,
    this.propertiesOpen,
    this.wave,
    this.selected = false,
    this.pickedKeys = const [],
    this.hidden = false,
    this.solo = false,
    this.locked = false,
    this.clipToBelow = false,
    this.lane = false,
    this.ghost,
    this.spans = const [],
  });

  /// Where only the layer's ghost plays (outside its own bar): from–to x, and whether it starts before frame 0
  /// (then a notch at [notchX] says so).
  final (double from, double to, double? notchX)? ghost;

  /// A lane's spans between consecutive keys: dashed while linear, solid once shaped, lit when both ends are picked.
  final List<(double from, double to, bool linear, bool chosen)> spans;

  /// A property lane under its layer: it has no layer switches; its ◆ adds or removes a key at the playhead.
  final bool lane;
  final String name;
  final TlKind kind;

  /// The small identity square beside an item's name.
  final Color? chip;

  /// A temporal body: left x, right x, colour.
  final (double, double, Color)? body;

  /// Key x positions (on the body when there is one).
  final List<double> keys;
  final double? nameW;
  final int indent;

  /// A disclosure triangle: true open, false closed, null none.
  final bool? open;
  final bool? propertiesOpen;

  /// Audio: the waveform's half-heights, one per 2 px from x 596.
  final List<double>? wave;

  /// The row is selected; these of its keys are.
  final bool selected;
  final List<double> pickedKeys;
  final bool hidden;
  final bool solo;
  final bool locked;
  final bool clipToBelow;
}

/// What a drag on a body changes: where it sits, or one of its ends.
enum TlGrip { body, start, end }

class TimelineModel {
  const TimelineModel({
    required this.rows,
    required this.ruler,
    required this.playhead,
    this.markers = const [],
    this.onAddMarker,
    this.onSplit,
    this.headerKeys = true,
    this.onMarkerContext,
    this.onMarkerDrag,
    this.onSeek,
    this.onRow,
    this.onFold,
    this.onProperties,
    this.onKey,
    this.onGrip,
    this.onKeysDrag,
    this.onScroll,
    this.onCoreDown,
    this.onCoreMove,
    this.onCoreUp,
    this.onCoreCancel,
    this.onCoreLabelDown,
    this.onCoreLabelMove,
    this.onCoreLabelUp,
    this.onCoreLabelCancel,
    this.tabs = true,
    this.onContext,
    this.frameWidth = 1,
    this.marquee,
    this.dropGuide,
    this.dropInside = false,
    this.marqueeInk = const Color(0xFFFFBC53),
    this.guideInk = const Color(0xFFB0E3EF),
  });

  /// How wide one frame is drawn (a key at the playhead is within half of it).
  final double frameWidth;

  /// A right press at [at] (the frame's coordinates) over a row's label or the tracks.
  final void Function(Offset at, Offset global)? onContext;

  /// A box being dragged over empty tracks to pick keys, in the frame's coordinates.
  final Rect? marquee;

  /// Where dragged rows would land: a line at the rect's top from its left, or the rect itself when they go inside.
  final Rect? dropGuide;
  final bool dropInside;

  /// The window's selection feedback colours (the Stage's marquee and drop guide use the same).
  final Color marqueeInk, guideInk;

  /// The seat's Timeline / Graph / Console labels, drawn here in proto_hf's fixed frame. Off in the Dock, where the
  /// seat strip (the shared tab primitive) names them.
  final bool tabs;

  /// Up to eight rows at the pitch, then audio rows in the floor band.
  final List<TlRow> rows;

  /// Eleven labels, one per major tick.
  final List<String> ruler;
  final double playhead;
  final List<(double x, String id)> markers;
  final VoidCallback? onAddMarker;
  final VoidCallback? onSplit;

  /// Split and Marker drawn at the reference's right end; a host that anchors them to its own edge turns this off.
  final bool headerKeys;
  final void Function(String id, Offset globalPosition)? onMarkerContext;

  /// A marker dragged along the ruler: how far it moved since the last call, and whether the drag ended.
  final void Function(String id, double dx, bool done)? onMarkerDrag;

  /// A press or drag on the ruler or the tracks, at that x.
  final ValueChanged<double>? onSeek;

  /// A press on a row's name.
  final ValueChanged<int>? onRow;
  final ValueChanged<int>? onFold;
  final ValueChanged<int>? onProperties;

  /// A press on a key of row [row] at x (with Shift: add to the picked keys).
  final void Function(int row, double x, bool add)? onKey;

  /// A drag on a body or one of its ends: dx so far; [done] true when released, null when cancelled.
  final void Function(int row, TlGrip grip, double dx, bool? done)? onGrip;

  /// A drag starting on a picked key: dx so far; [done] as above.
  final void Function(double dx, bool? done)? onKeysDrag;

  /// A scroll over the tracks: dx/dy, and whether it zooms (Cmd/Ctrl held).
  final void Function(double dx, double dy, bool zoom)? onScroll;
  final void Function(PointerDownEvent event, Offset at)? onCoreDown;
  final void Function(PointerMoveEvent event, Offset at)? onCoreMove;
  final void Function(PointerUpEvent event, Offset at)? onCoreUp;
  final void Function(PointerCancelEvent event, Offset at)? onCoreCancel;
  final void Function(PointerDownEvent event, int row, Offset at)?
  onCoreLabelDown;
  final void Function(PointerMoveEvent event, int row, Offset at)?
  onCoreLabelMove;
  final void Function(PointerUpEvent event, int row, Offset at)? onCoreLabelUp;
  final void Function(PointerCancelEvent event, int row, Offset at)?
  onCoreLabelCancel;
}

double _rowTop(int i, TlRow r) =>
    r.kind == TlKind.audio ? tlAudioTop : tlTop + tlPitch * i;
double _rowH(TlRow r) => r.kind == TlKind.audio ? tlAudioH : tlRowH;
double _rowCy(int i, TlRow r) => _rowTop(i, r) + _rowH(r) / 2;

List<RI> timeline(TimelineModel m) {
  final items = <RI>[
    Ln(344, 703, 1178, 1, H.rule),
    Ln(344, 993, 1178, 1, H.rule),
    Ln(344, 703, 1, 291, H.rule),
    Ln(1521, 703, 1, 291, H.rule),
    if (m.tabs) ...[
      Rc(345, 704, 103, 38, fill: H.sel),
      Tx(370, 727, 'Timeline', H.s(13, w: FontWeight.w500, color: H.text), w: 54),
      Tx(469, 727, 'Graph', H.s(13, color: H.text2), w: 34),
      Tx(546, 727, 'Console', H.s(13, color: H.text2), w: 46),
    ],
    Ln(345, 742, 1176, 1, H.rule),
    Pt(_TlPaint(m)),
    Ln(558, 743, 1, 250, H.rule),
    if (m.headerKeys) ...[
    Wd(
      1362,
      708,
      66,
      24,
      GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: m.onSplit,
        child: Center(
          child: Text('Split', style: H.s(11, color: H.text2)),
        ),
      ),
    ),
    Wd(
      1436,
      708,
      70,
      24,
      GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: m.onAddMarker,
        child: Center(
          child: Text('Marker', style: H.s(11, color: H.text2)),
        ),
      ),
    ),
    ],
  ];
  // ruler numerals: the same technical-readout family as the other numeric text
  for (var i = 0; i <= 10; i++) {
    items.add(
      Tx(
        tlX(i.toDouble()) + 3,
        762,
        m.ruler[i],
        H.m(11, color: const Color(0xFFB0B0B2)),
        w: 33,
      ),
    );
  }
  // hierarchy grid: c0 = disclosure / state, c1 = glyph, c2 = label.
  const c0 = 363.0, c1 = 393.0, c2 = 412.0;
  for (final (i, r) in m.rows.indexed) {
    final cy = _rowCy(i, r);
    final base = cy + 4.5;
    switch (r.kind) {
      case TlKind.group:
        items.add(
          Tx(
            386 + r.indent * 14,
            base,
            r.name,
            H.s(12.5, w: FontWeight.w600, color: H.text),
            w: r.nameW,
          ),
        );
        break;
      case TlKind.item:
        items.add(Rc(c1 - 8, cy - 8, 16, 16, fill: r.chip, r: 3));
        items.add(Hg(c1, cy, 10, HG.diamond, const Color(0xFFF2F2F2)));
      case TlKind.camera:
        items.add(Hg(c1, cy, 15, HG.power, H.text2));
      case TlKind.audio:
        items.add(Hg(c0, cy + 0.5, 16, HG.headphones, H.text2));
        items.add(Hg(c1, cy + 0.5, 16, HG.lock, H.text2, bg: H.raisedHi));
    }
    if (r.kind != TlKind.group) {
      items.add(
        Tx(
          c2 + r.indent * 14,
          base,
          r.name,
          H.s(12.5, color: H.text2),
          w: r.nameW,
        ),
      );
    }
    if (r.lane) {
      // keyed at the playhead: a solid mark; otherwise an outline one says a key can be added here
      final keyed = r.keys.any((k) => (k - m.playhead).abs() < m.frameWidth / 2 + 1e-6);
      items.add(Hg(547, cy, 10, HG.diamond, keyed ? H.text : H.text3));
      continue;
    }
    for (final (column, state) in [
      r.hidden,
      r.solo,
      r.locked,
      r.clipToBelow,
    ].indexed) {
      items.add(
        Tx(
          495.0 + 16 * column,
          base,
          ['M', 'S', 'L', 'C'][column],
          H.s(9, color: state ? H.text : H.text2, w: FontWeight.w600),
          w: 10,
        ),
      );
    }
  }
  items.add(Pt(_TlMarks(m)));
  if (m.marquee case final Rect box) {
    final r = box.intersect(const Rect.fromLTRB(345, 775, tlRight, 993));
    if (r.width > 0 && r.height > 0)
      items.add(Rc(r.left, r.top, r.width, r.height, fill: m.marqueeInk.withValues(alpha: .12), border: m.marqueeInk));
  }
  if (m.dropGuide case final Rect g) {
    if (m.dropInside) {
      items.add(Rc(g.left + 1, g.top + 1, tlRight - g.left - 2, g.height - 2, border: m.guideInk, bw: 2));
    } else {
      items.add(Ln(g.left, g.top - 1, tlRight - g.left, 2, m.guideInk));
      items.add(Rc(g.left, g.top - 3, 6, 6, border: m.guideInk, bw: 2, r: 3));
    }
  }
  // hit areas over what is drawn above; with no operation they do nothing
  items.add(Wd(tlX0, 775, tlRight - tlX0, 218, _Tracks(m)));
  final seek = m.onSeek;
  items.add(
    Wd(
      tlX0,
      743,
      tlRight - tlX0,
      32,
      GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTapDown: seek == null ? null : (e) => seek(tlX0 + e.localPosition.dx),
        onHorizontalDragUpdate: seek == null
            ? null
            : (e) => seek(tlX0 + e.localPosition.dx),
        child: const SizedBox.expand(),
      ),
    ),
  );
  for (final (x, id) in m.markers) {
    items.add(
      Wd(
        x - 7,
        743,
        14,
        31,
        Builder(
          builder: (context) => GestureDetector(
            behavior: HitTestBehavior.opaque,
            onSecondaryTapDown: m.onMarkerContext == null
                ? null
                : (event) => m.onMarkerContext!(id, event.globalPosition),
            onHorizontalDragUpdate: m.onMarkerDrag == null
                ? null
                : (e) => m.onMarkerDrag!(id, e.delta.dx, false),
            onHorizontalDragEnd: m.onMarkerDrag == null
                ? null
                : (e) => m.onMarkerDrag!(id, 0, true),
            child: const SizedBox.expand(),
          ),
        ),
      ),
    );
  }
  final row = m.onRow;
  for (final (i, r) in m.rows.indexed) {
    items.add(
      Wd(
        345,
        _rowTop(i, r),
        213,
        _rowH(r),
        Listener(
          behavior: HitTestBehavior.opaque,
          onPointerDown: m.onCoreLabelDown == null
              ? null
              : (e) => m.onCoreLabelDown!(e, i, e.localPosition),
          onPointerMove: m.onCoreLabelMove == null
              ? null
              : (e) => m.onCoreLabelMove!(e, i, e.localPosition),
          onPointerUp: m.onCoreLabelUp == null
              ? null
              : (e) => m.onCoreLabelUp!(e, i, e.localPosition),
          onPointerCancel: m.onCoreLabelCancel == null
              ? null
              : (e) => m.onCoreLabelCancel!(e, i, e.localPosition),
          child: GestureDetector(
            behavior: HitTestBehavior.opaque,
            onSecondaryTapDown: m.onContext == null
                ? null
                : (e) => m.onContext!(
                    e.localPosition + Offset(345, _rowTop(i, r)),
                    e.globalPosition,
                  ),
            onTapDown: row == null || m.onCoreLabelDown != null
                ? null
                : (e) {
                    if (r.open != null && e.localPosition.dx < 30) {
                      m.onFold?.call(i);
                    } else if (r.propertiesOpen != null &&
                        e.localPosition.dx >= 30 &&
                        e.localPosition.dx < 48) {
                      m.onProperties?.call(i);
                    } else {
                      row(i);
                    }
                  },
            child: const SizedBox.expand(),
          ),
        ),
      ),
    );
  }
  return items;
}

class _Tracks extends StatefulWidget {
  const _Tracks(this.m);
  final TimelineModel m;
  @override
  State<_Tracks> createState() => _TracksState();
}

class _TracksState extends State<_Tracks> {
  TimelineModel get m => widget.m;
  (int, TlGrip)? _grip;
  bool _keys = false;
  double _from = 0;

  int? _rowAt(double y) {
    for (final (i, r) in m.rows.indexed) {
      final t = _rowTop(i, r) - tlTop;
      if (y >= t && y < t + _rowH(r)) return i;
    }
    return null;
  }

  double? _keyAt(int row, double x) {
    for (final k in m.rows[row].keys) {
      if ((k - x).abs() <= 5) return k;
    }
    return null;
  }

  void _down(Offset p) {
    final x = tlX0 + p.dx, row = _rowAt(p.dy);
    _grip = null;
    _keys = false;
    _from = x;
    if (row != null) {
      final k = _keyAt(row, x);
      if (k != null) {
        final shift = HardwareKeyboard.instance.isShiftPressed;
        if (!m.rows[row].pickedKeys.contains(k)) m.onKey?.call(row, k, shift);
        _keys = m.onKeysDrag != null;
        return;
      }
      final b = m.rows[row].body;
      if (b != null && x >= b.$1 - 4 && x <= b.$2 + 4 && m.onGrip != null) {
        _grip = (
          row,
          (x - b.$1).abs() <= 5
              ? TlGrip.start
              : ((x - b.$2).abs() <= 5 ? TlGrip.end : TlGrip.body),
        );
        m.onRow?.call(row);
        return;
      }
    }
    m.onSeek?.call(x);
  }

  void _move(Offset p) {
    final x = tlX0 + p.dx;
    if (_keys) {
      m.onKeysDrag?.call(x - _from, false);
    } else if (_grip != null) {
      m.onGrip?.call(_grip!.$1, _grip!.$2, x - _from, false);
    } else {
      m.onSeek?.call(x);
    }
  }

  void _up(double x, {bool cancel = false}) {
    if (_keys) m.onKeysDrag?.call(x - _from, cancel ? null : true);
    if (_grip != null)
      m.onGrip?.call(_grip!.$1, _grip!.$2, x - _from, cancel ? null : true);
    _grip = null;
    _keys = false;
  }

  double _last = 0;

  @override
  Widget build(BuildContext context) => Listener(
    behavior: HitTestBehavior.opaque,
    onPointerDown: (e) {
      _last = tlX0 + e.localPosition.dx;
      if (m.onCoreDown != null) {
        m.onCoreDown!(e, e.localPosition);
      } else {
        _down(e.localPosition);
      }
    },
    onPointerMove: (e) {
      _last = tlX0 + e.localPosition.dx;
      if (m.onCoreMove != null) {
        m.onCoreMove!(e, e.localPosition);
      } else {
        _move(e.localPosition);
      }
    },
    onPointerUp: (e) {
      if (m.onCoreUp != null) {
        m.onCoreUp!(e, e.localPosition);
      } else {
        _up(_last);
      }
    },
    onPointerCancel: (e) {
      if (m.onCoreCancel != null) {
        m.onCoreCancel!(e, e.localPosition);
      } else {
        _up(_last, cancel: true);
      }
    },
    onPointerSignal: (e) {
      if (e is PointerScrollEvent && m.onScroll != null) {
        final k = HardwareKeyboard.instance;
        m.onScroll!(
          e.scrollDelta.dx,
          e.scrollDelta.dy,
          k.isMetaPressed || k.isControlPressed,
        );
      }
    },
    child: GestureDetector(
      behavior: HitTestBehavior.opaque,
      onSecondaryTapDown: m.onContext == null
          ? null
          : (e) => m.onContext!(
              e.localPosition + const Offset(tlX0, tlTop),
              e.globalPosition,
            ),
      child: const SizedBox.expand(),
    ),
  );
}
