// The Timeline seat of the reference, at its own coordinates. Layers, bottom to top: panel ground, alternating row
// ground, hierarchy, ruler, major/minor time grid, temporal bodies, keyframes, playhead, waveform (a floor, not a
// body). Row height and bar height are separate tokens; the bar leaves a fixed negative space. What the rows say, where
// bodies and keys sit and where the playhead is come from a [TimelineModel], in the reference's own x.
import 'dart:math' as math;
import 'package:flutter/widgets.dart';
import '../glyphs.dart';
import 'place.dart';

const tlTop = 775.0, tlPitch = 23.0, tlRowH = 22.0, tlBarH = 18.0;
const tlAudioTop = 959.0, tlAudioH = 34.0;
const tlX0 = 559.0, tlUnit = 92.9, tlRight = 1521.0;

/// Time unit -> x.
double tlX(double t) => tlX0 + 0.5 + tlUnit * t;

enum TlKind { group, item, camera, audio }

class TlRow {
  const TlRow(this.name, this.kind, {this.chip, this.body, this.keys = const [], this.nameW, this.open, this.wave});
  final String name;
  final TlKind kind;

  /// The small identity square beside an item's name.
  final Color? chip;

  /// A temporal body: left x, right x, colour.
  final (double, double, Color)? body;

  /// Key x positions (on the body when there is one).
  final List<double> keys;
  final double? nameW;

  /// A disclosure triangle: true open, false closed, null none.
  final bool? open;

  /// Audio: the waveform's half-heights, one per 2 px from x 596.
  final List<double>? wave;
}

class TimelineModel {
  const TimelineModel({required this.rows, required this.ruler, required this.playhead, this.onSeek, this.onRow});

  /// Up to eight rows at the pitch, then audio rows in the floor band.
  final List<TlRow> rows;

  /// Eleven labels, one per major tick.
  final List<String> ruler;
  final double playhead;

  /// A press or drag on the ruler or the tracks, at that x.
  final ValueChanged<double>? onSeek;

  /// A press on a row's name.
  final ValueChanged<int>? onRow;
}

double _rowTop(int i, TlRow r) => r.kind == TlKind.audio ? tlAudioTop : tlTop + tlPitch * i;
double _rowH(TlRow r) => r.kind == TlKind.audio ? tlAudioH : tlRowH;
double _rowCy(int i, TlRow r) => _rowTop(i, r) + _rowH(r) / 2;

List<RI> timeline(TimelineModel m) {
  final items = <RI>[
    Ln(344, 703, 1178, 1, H.rule), Ln(344, 993, 1178, 1, H.rule), Ln(344, 703, 1, 291, H.rule), Ln(1521, 703, 1, 291, H.rule),
    Rc(345, 704, 103, 38, fill: H.sel),
    Tx(370, 727, 'Timeline', H.s(13, w: FontWeight.w500, color: H.text), w: 54),
    Tx(469, 727, 'Graph', H.s(13, color: H.text2), w: 34),
    Tx(546, 727, 'Console', H.s(13, color: H.text2), w: 46),
    Ln(345, 742, 1176, 1, H.rule),
    Pt(_TlPaint(m)),
    Ln(558, 743, 1, 250, H.rule),
  ];
  // ruler numerals: the same technical-readout family as the other numeric text
  for (var i = 0; i <= 10; i++) {
    items.add(Tx(tlX(i.toDouble()) + 3, 762, m.ruler[i], H.m(11, color: const Color(0xFFB0B0B2)), w: 33));
  }
  // hierarchy grid: c0 = disclosure / state, c1 = glyph, c2 = label.
  const c0 = 363.0, c1 = 393.0, c2 = 412.0;
  for (final (i, r) in m.rows.indexed) {
    final cy = _rowCy(i, r);
    final base = cy + 4.5;
    switch (r.kind) {
      case TlKind.group:
        items.add(Tx(386, base, r.name, H.s(12.5, w: FontWeight.w600, color: H.text), w: r.nameW));
        continue;
      case TlKind.item:
        items.add(Rc(c1 - 8, cy - 8, 16, 16, fill: r.chip, r: 3));
        items.add(Hg(c1, cy, 10, HG.diamond, const Color(0xFFF2F2F2)));
      case TlKind.camera:
        items.add(Hg(c1, cy, 15, HG.power, H.text2));
      case TlKind.audio:
        items.add(Hg(c0, cy + 0.5, 16, HG.headphones, H.text2));
        items.add(Hg(c1, cy + 0.5, 16, HG.lock, H.text2, bg: H.raisedHi));
    }
    items.add(Tx(c2, base, r.name, H.s(12.5, color: H.text2), w: r.nameW));
  }
  items.add(Pt(_TlMarks(m)));
  // hit areas over what is drawn above; with no operation they do nothing
  final seek = m.onSeek, row = m.onRow;
  items.add(Wd(tlX0, 743, tlRight - tlX0, 250, GestureDetector(
    behavior: HitTestBehavior.opaque,
    onTapDown: seek == null ? null : (d) => seek(tlX0 + d.localPosition.dx),
    onHorizontalDragUpdate: seek == null ? null : (d) => seek(tlX0 + d.localPosition.dx),
    child: const SizedBox.expand(),
  )));
  for (final (i, r) in m.rows.indexed) {
    items.add(Wd(345, _rowTop(i, r), 213, _rowH(r), GestureDetector(behavior: HitTestBehavior.opaque, onTap: row == null ? null : () => row(i), child: const SizedBox.expand())));
  }
  return items;
}

class _TlMarks extends CustomPainter {
  _TlMarks(this.m);
  final TimelineModel m;
  @override
  void paint(Canvas cv, Size s) {
    final p = Paint()..color = H.text;
    // disclosure triangles sit on c0, optically centred on the row
    for (final (i, r) in m.rows.indexed) {
      if (r.open == null) continue;
      final g = _rowCy(i, r);
      if (r.open!) {
        cv.drawPath(Path()..moveTo(358, g - 4)..lineTo(368.6, g - 4)..lineTo(363.3, g + 4)..close(), p);
      } else {
        cv.drawPath(Path()..moveTo(359, g - 5.6)..lineTo(368, g + .1)..lineTo(359, g + 5.8)..close(), p);
      }
    }
  }

  @override
  bool shouldRepaint(_TlMarks o) => o.m != m;
}

class _TlPaint extends CustomPainter {
  _TlPaint(this.m);
  final TimelineModel m;
  @override
  void paint(Canvas cv, Size s) {
    // 1 panel ground is the window colour; 2 row ground alternates and runs continuously across both columns
    cv.drawRect(const Rect.fromLTRB(345, 743, 1521, 993), Paint()..color = H.window);
    cv.drawRect(const Rect.fromLTRB(tlX0, 743, 1521, 774), Paint()..color = H.raised);
    cv.drawRect(const Rect.fromLTRB(345, 743, 558, 774), Paint()..color = H.raised);
    for (var i = 0; i < 9; i++) {
      final c = (i == 8) ? H.raised : (i.isEven ? H.raisedHi : H.raised);
      final top = i < 8 ? tlTop + tlPitch * i : tlAudioTop;
      cv.drawRect(Rect.fromLTWH(345, top, 1521 - 345, i < 8 ? tlRowH : tlAudioH), Paint()..color = c);
    }
    cv.drawRect(const Rect.fromLTRB(345, 774, 1521, 775), Paint()..color = H.rule);
    // 3 time grid: major > minor > row gap. Major runs through the ruler as a tick.
    final major = Paint()..color = const Color(0x17FFFFFF)..strokeWidth = 1;
    final minor = Paint()..color = const Color(0x0AFFFFFF)..strokeWidth = 1;
    final tMaj = Paint()..color = const Color(0x47FFFFFF)..strokeWidth = 1;
    final tMin = Paint()..color = const Color(0x29FFFFFF)..strokeWidth = 1;
    for (var i = 0; i <= 10; i++) {
      final x = tlX(i.toDouble());
      cv.drawLine(Offset(x, 766), Offset(x, 774), tMaj);
      cv.drawLine(Offset(x, 775), Offset(x, 993), major);
      for (var k = 1; k < 4; k++) {
        final xm = x + tlUnit * k / 4;
        cv.drawLine(Offset(xm, 770), Offset(xm, 774), tMin);
        cv.drawLine(Offset(xm, 775), Offset(xm, 993), minor);
      }
    }
    // 4 waveform: a quiet floor
    for (final (i, r) in m.rows.indexed) {
      final wave = r.wave;
      if (wave == null) continue;
      final cy = _rowCy(i, r);
      cv.drawRRect(RRect.fromRectAndRadius(Rect.fromLTRB(594, cy - 11, 1490, cy + 11), const Radius.circular(2)), Paint()..color = const Color(0xFF1E2622));
      final wv = Paint()..color = const Color(0x803B6D5F)..strokeWidth = 1;
      for (var k = 0; k < wave.length; k++) {
        final x = 596.0 + 2 * k, a = wave[k];
        cv.drawLine(Offset(x, cy - a), Offset(x, cy + a), wv);
      }
    }
    // 5 temporal bodies: a flat body, one thin connector through the keys, small diamond keys on it.
    // One rule for every relation; a body with a single key has no connector.
    Color mixW(Color c, double t) => Color.lerp(c, const Color(0xFFFFFFFF), t)!;
    void node(double x, double cy, Color body) {
      cv.save();
      cv.translate(x, cy);
      cv.rotate(math.pi / 4);
      final rr = Rect.fromCenter(center: Offset.zero, width: 4.8, height: 4.8);
      cv.drawRect(rr, Paint()..color = mixW(body, .32));
      cv.drawRect(rr, Paint()..color = mixW(body, .58)..style = PaintingStyle.stroke..strokeWidth = 1);
      cv.restore();
    }

    for (final (i, r) in m.rows.indexed) {
      final cy = _rowCy(i, r);
      final b = r.body;
      if (b == null) {
        // a key with no body (Camera): the same diamond on the neutral row
        for (final k in r.keys) {
          cv.save();
          cv.translate(k, cy);
          cv.rotate(math.pi / 4);
          final kr = Rect.fromCenter(center: Offset.zero, width: 4.8, height: 4.8);
          cv.drawRect(kr, Paint()..color = mixW(H.raisedHi, .25));
          cv.drawRect(kr, Paint()..color = mixW(H.raisedHi, .5)..style = PaintingStyle.stroke..strokeWidth = 1);
          cv.restore();
        }
        continue;
      }
      final (a, e, c) = b;
      final top = _rowTop(i, r) + (_rowH(r) - tlBarH) / 2;
      final rc = Rect.fromLTRB(a, top, e, top + tlBarH);
      final clipped = a <= tlX0;
      RRect rrect(Rect q, double rad) => RRect.fromRectAndCorners(q, topLeft: Radius.circular(clipped ? 0 : rad), bottomLeft: Radius.circular(clipped ? 0 : rad), topRight: Radius.circular(rad), bottomRight: Radius.circular(rad));
      cv.drawRRect(rrect(rc, 4), Paint()..color = c);
      cv.drawRRect(rrect(rc.deflate(.5), 3.6), Paint()..color = mixW(c, .13)..style = PaintingStyle.stroke..strokeWidth = 1);
      final ks = [...r.keys]..sort();
      if (ks.length > 1) cv.drawLine(Offset(ks.first, cy), Offset(ks.last, cy), Paint()..color = const Color(0x5C000000)..strokeWidth = 1);
      for (final k in ks) {
        node(k, cy, c);
      }
    }
    // 6 playhead: ruler marker + thin line, above bodies and keys
    final ph = m.playhead;
    cv.drawRect(Rect.fromLTWH(ph - 0.35, 748, 1.3, 245), Paint()..color = H.playhead);
    cv.drawPath(Path()..moveTo(ph - 5.75, 748)..lineTo(ph + 7.25, 748)..lineTo(ph + 7.25, 755)..lineTo(ph + 0.75, 762)..lineTo(ph - 5.75, 755)..close(), Paint()..color = H.playhead);
  }

  @override
  bool shouldRepaint(_TlPaint o) => o.m != m;
}
