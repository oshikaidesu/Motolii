part of 'timeline.dart';

// The Timeline face's painting: the ruler marks and the rows (bars, keys, lanes, ghosts, markers).

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
        cv.drawPath(
          Path()
            ..moveTo(358, g - 4)
            ..lineTo(368.6, g - 4)
            ..lineTo(363.3, g + 4)
            ..close(),
          p,
        );
      } else {
        cv.drawPath(
          Path()
            ..moveTo(359, g - 5.6)
            ..lineTo(368, g + .1)
            ..lineTo(359, g + 5.8)
            ..close(),
          p,
        );
      }
    }
    for (final (i, r) in m.rows.indexed) {
      if (r.propertiesOpen == null) continue;
      final g = _rowCy(i, r);
      if (r.propertiesOpen!) {
        cv.drawPath(
          Path()
            ..moveTo(474, g - 3)
            ..lineTo(482, g - 3)
            ..lineTo(478, g + 3)
            ..close(),
          p,
        );
      } else {
        cv.drawPath(
          Path()
            ..moveTo(476, g - 4)
            ..lineTo(482, g)
            ..lineTo(476, g + 4)
            ..close(),
          p,
        );
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
    cv.drawRect(
      const Rect.fromLTRB(345, 743, 1521, 993),
      Paint()..color = H.window,
    );
    cv.drawRect(
      const Rect.fromLTRB(tlX0, 743, 1521, 774),
      Paint()..color = H.raised,
    );
    cv.drawRect(
      const Rect.fromLTRB(345, 743, 558, 774),
      Paint()..color = H.raised,
    );
    for (var i = 0; i < 9; i++) {
      final c = (i == 8) ? H.raised : (i.isEven ? H.raisedHi : H.raised);
      final top = i < 8 ? tlTop + tlPitch * i : tlAudioTop;
      cv.drawRect(
        Rect.fromLTWH(345, top, 1521 - 345, i < 8 ? tlRowH : tlAudioH),
        Paint()..color = c,
      );
    }
    for (final (i, r) in m.rows.indexed) {
      if (r.selected)
        cv.drawRect(
          Rect.fromLTWH(345, _rowTop(i, r), 1521 - 345, _rowH(r)),
          Paint()..color = H.sel,
        );
    }
    cv.drawRect(
      const Rect.fromLTRB(345, 774, 1521, 775),
      Paint()..color = H.rule,
    );
    // 3 time grid: major > minor > row gap. Major runs through the ruler as a tick.
    final major = Paint()
      ..color = N.glaze9
      ..strokeWidth = 1;
    final minor = Paint()
      ..color = N.glaze4
      ..strokeWidth = 1;
    final tMaj = Paint()
      ..color = N.glaze28
      ..strokeWidth = 1;
    final tMin = Paint()
      ..color = N.glaze15
      ..strokeWidth = 1;
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
      cv.drawRRect(
        RRect.fromRectAndRadius(
          Rect.fromLTRB(594, cy - 11, 1490, cy + 11),
          const Radius.circular(2),
        ),
        Paint()..color = N.g13,
      );
      final wv = Paint()
        ..color = H.wave.withAlpha(0x80)
        ..strokeWidth = 1;
      for (var k = 0; k < wave.length; k++) {
        final x = 596.0 + 2 * k, a = wave[k];
        cv.drawLine(Offset(x, cy - a), Offset(x, cy + a), wv);
      }
    }
    // 5 temporal bodies: a flat body, one thin connector through the keys, small diamond keys on it.
    // One rule for every relation; a body with a single key has no connector.
    Color mixW(Color c, double t) => Color.lerp(c, N.g100, t)!;
    void node(double x, double cy, Color body) {
      cv.save();
      cv.translate(x, cy);
      cv.rotate(math.pi / 4);
      final rr = Rect.fromCenter(center: Offset.zero, width: 4.8, height: 4.8);
      cv.drawRect(rr, Paint()..color = mixW(body, .32));
      cv.drawRect(
        rr,
        Paint()
          ..color = mixW(body, .58)
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1,
      );
      cv.restore();
    }

    for (final (i, r) in m.rows.indexed) {
      final cy = _rowCy(i, r);
      final b = r.body;
      if (b == null) {
        for (final (x1, x2, linear, chosen) in r.spans) {
          final p = Paint()
            ..color = chosen ? H.playhead : H.text3.withValues(alpha: .6)
            ..strokeWidth = chosen ? 2 : 1;
          if (linear) {
            for (var x = x1; x < x2; x += 6) cv.drawLine(Offset(x, cy), Offset(math.min(x + 3, x2), cy), p);
          } else {
            cv.drawLine(Offset(x1, cy), Offset(x2, cy), p);
          }
        }
        // a key with no body (Camera): the same diamond on the neutral row
        for (final k in r.keys) {
          cv.save();
          cv.translate(k, cy);
          cv.rotate(math.pi / 4);
          final kr = Rect.fromCenter(
            center: Offset.zero,
            width: 4.8,
            height: 4.8,
          );
          cv.drawRect(kr, Paint()..color = mixW(H.raisedHi, .25));
          cv.drawRect(
            kr,
            Paint()
              ..color = mixW(H.raisedHi, .5)
              ..style = PaintingStyle.stroke
              ..strokeWidth = 1,
          );
          cv.restore();
        }
        continue;
      }
      final (a, e, c) = b;
      final top = _rowTop(i, r) + (_rowH(r) - tlBarH) / 2;
      if (r.ghost case (final g0, final g1, final notch)) {
        final g = Rect.fromLTRB(g0, top, g1, top + tlBarH);
        if (g.width > 0) {
          cv.drawRect(g, Paint()..color = H.text3.withValues(alpha: .28));
          cv.drawRect(g.deflate(.5), Paint()..color = c.withValues(alpha: .55)..style = PaintingStyle.stroke..strokeWidth = 1);
        }
        if (notch != null) {
          cv.drawPath(
            Path()
              ..moveTo(notch, top)
              ..lineTo(notch + 5, top + tlBarH / 2)
              ..lineTo(notch, top + tlBarH),
            Paint()..color = c.withValues(alpha: .55)..style = PaintingStyle.stroke..strokeWidth = 1,
          );
        }
      }
      final rc = Rect.fromLTRB(a, top, e, top + tlBarH);
      final clipped = a <= tlX0;
      RRect rrect(Rect q, double rad) => RRect.fromRectAndCorners(
        q,
        topLeft: Radius.circular(clipped ? 0 : rad),
        bottomLeft: Radius.circular(clipped ? 0 : rad),
        topRight: Radius.circular(rad),
        bottomRight: Radius.circular(rad),
      );
      cv.drawRRect(rrect(rc, 4), Paint()..color = c);
      cv.drawRRect(
        rrect(rc.deflate(.5), 3.6),
        Paint()
          ..color = mixW(c, .13)
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1,
      );
      final ks = [...r.keys]..sort();
      if (ks.length > 1)
        cv.drawLine(
          Offset(ks.first, cy),
          Offset(ks.last, cy),
          Paint()
            ..color = N.shade40
            ..strokeWidth = 1,
        );
      for (final k in ks) {
        node(k, cy, r.pickedKeys.contains(k) ? mixW(c, .75) : c);
      }
    }
    for (final (x, _) in m.markers) {
      final markerPaint = Paint()..color = H.scatter.n;
      cv.drawRect(Rect.fromLTWH(x - .5, 754, 1, 239), markerPaint);
      cv.drawPath(
        Path()
          ..moveTo(x - 5, 748)
          ..lineTo(x + 5, 748)
          ..lineTo(x + 5, 755)
          ..lineTo(x, 760)
          ..lineTo(x - 5, 755)
          ..close(),
        markerPaint,
      );
    }
    // 6 playhead: ruler marker + thin line, above bodies and keys
    final ph = m.playhead;
    cv.drawRect(
      Rect.fromLTWH(ph - 0.35, 748, 1.3, 245),
      Paint()..color = H.playhead,
    );
    cv.drawPath(
      Path()
        ..moveTo(ph - 5.75, 748)
        ..lineTo(ph + 7.25, 748)
        ..lineTo(ph + 7.25, 755)
        ..lineTo(ph + 0.75, 762)
        ..lineTo(ph - 5.75, 755)
        ..close(),
      Paint()..color = H.playhead,
    );
  }

  @override
  bool shouldRepaint(_TlPaint o) => o.m != m;
}

/// The tracks' pointer: a press on a key picks it, a drag from a picked key moves the picked keys, a drag on a body
/// or one of its ends retimes it, anything else scrubs; a scroll pans, with Cmd/Ctrl it zooms.
