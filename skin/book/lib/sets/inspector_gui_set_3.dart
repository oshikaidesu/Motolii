// Part of inspector_gui_set.dart (cut there by size, as it was).
part of 'inspector_gui_set.dart';

/// 2D / 2.5D / 3D as three small drawings: choosing is graphic. Changes no value, only what is revealed.
class _SpaceChips extends StatelessWidget {
  const _SpaceChips(this.d);
  final GDoc d;
  static const names = ['2D', '2.5D', '3D'];
  @override
  Widget build(BuildContext context) => Container(
        padding: const EdgeInsets.all(2),
        decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(7)),
        child: Row(children: [
          for (var i = 0; i < 3; i++)
            Expanded(
              child: GestureDetector(
                behavior: HitTestBehavior.opaque,
                onTap: d.locked
                    ? null
                    : () {
                        d.begin();
                        d.set('space', i.toDouble());
                        d.end();
                      },
                child: AnimatedContainer(
                  duration: const Duration(milliseconds: 120),
                  curve: Curves.easeOut,
                  height: Gui.tile,
                  decoration: BoxDecoration(
                    color: d['space'] == i ? N.g20 : null,
                    borderRadius: BorderRadius.circular(5),
                    border: Border(bottom: BorderSide(color: d['space'] == i ? Role.selected : const Color(0x00000000))),
                  ),
                  child: Column(mainAxisAlignment: MainAxisAlignment.center, children: [
                    SizedBox(width: 26, height: 14, child: CustomPaint(painter: _Fn((cv, s) => _spaceIcon(cv, s, i, d.locked ? Role.disabled : (d['space'] == i ? N.g95 : N.g63))))),
                    const SizedBox(height: 4),
                    Text(names[i], style: T.micro(d['space'] == i ? N.g95 : N.g63)),
                  ]),
                ),
              ),
            ),
        ]),
      );
}

void _spaceIcon(Canvas cv, Size s, int i, Color col) {
  final r = Rect.fromCenter(center: s.center(Offset.zero), width: 16, height: 10);
  switch (i) {
    case 0:
      cv.drawRect(r, _ln(col));
    case 1:
      cv.drawRect(r.shift(const Offset(4, -2)), _ln(col.withValues(alpha: .5)));
      cv.drawRect(r.shift(const Offset(-2, 1)), _ln(col));
    default:
      final f = r.shift(const Offset(-3, 2)), b = r.shift(const Offset(3, -2));
      for (final (a, c) in [(f.topLeft, b.topLeft), (f.topRight, b.topRight), (f.bottomRight, b.bottomRight), (f.bottomLeft, b.bottomLeft)]) {
        cv.drawLine(a, c, _ln(col.withValues(alpha: .5)));
      }
      cv.drawRect(b, _ln(col.withValues(alpha: .5)));
      cv.drawRect(f, _ln(col));
  }
}

// ---- the specs shared by the number side and the readouts --------------------------------------------------------------------

List<Spec> _posSpecs(GDoc d) => [
      const Spec('pos.x', 'X', axis: 0, unit: 'px'),
      const Spec('pos.y', 'Y', axis: 1, unit: 'px'),
      if (d['space'] > 0) const Spec('pos.z', 'Z', axis: 2, unit: 'px'),
    ];
List<Spec> _rotSpecs(GDoc d) => d['space'] > 0
    ? const [Spec('rot.x', 'X', axis: 0, unit: '°', per: .5), Spec('rot.y', 'Y', axis: 1, unit: '°', per: .5), Spec('rot.z', 'Z', axis: 2, unit: '°', per: .5)]
    : const [Spec('rot.z', 'Z', axis: 2, unit: '°', per: .5)];
const _scaleSpecs = [Spec('scale.x', 'X', axis: 0, scale: 100, unit: '%', per: .5), Spec('scale.y', 'Y', axis: 1, scale: 100, unit: '%', per: .5)];
const _anchorSpecs = [Spec('anchor.x', 'X', axis: 0, scale: 100, unit: '%', per: .5), Spec('anchor.y', 'Y', axis: 1, scale: 100, unit: '%', per: .5)];

Widget _title(String t, {Widget? trailing}) => SizedBox(
      height: 16,
      child: Row(children: [
        Text(t.toUpperCase(), style: T.micro(N.g56).copyWith(letterSpacing: .4)),
        const SizedBox(width: 6),
        Expanded(child: Container(height: 1, color: N.g15)),
        if (trailing != null) ...[const SizedBox(width: 6), trailing],
      ]),
    );

Widget _lockedWord(GDoc d) => d.locked ? Text('Locked', style: T.label(N.g76)) : const SizedBox.shrink();
Widget _gap(double h) => SizedBox(height: h);

/// (b) and the base of (c): the instruments, each with its numbers beside it as a quiet readout.
class TransformGui extends StatelessWidget {
  const TransformGui(this.d, {super.key});
  final GDoc d;
  @override
  Widget build(BuildContext context) {
    final three = d['space'] > 0;
    const w = Gui.content, colW = (w - 16) / 3;
    final axisIdx = d['rot.axis'].round().clamp(0, 2);
    final anchorName = _anchorIndex(d) == null ? 'Custom' : _anchorNames[_anchorIndex(d)!];
    Widget col(String t, Widget hand, Widget below, {Widget? trailing}) => SizedBox(
          width: colW,
          child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            SizedBox(height: 20, child: Row(children: [Text(t, style: T.label(N.g63)), const Spacer(), ?trailing])),
            hand,
            _gap(4),
            below,
          ]),
        );
    return Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      _title('Space', trailing: _lockedWord(d)),
      _gap(6),
      _SpaceChips(d),
      _gap(12),
      _title('Position'),
      _gap(6),
      Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
        _padHand(d, Size(three ? w - Gui.rail - 6 : w, Gui.padH)),
        if (three) ...[const SizedBox(width: 6), _railHand(d, const Size(Gui.rail, Gui.padH))],
      ]),
      _gap(4),
      Row(children: [for (final (i, s) in _posSpecs(d).indexed) ...[if (i > 0) const SizedBox(width: 4), Expanded(child: XyzStack(d, [s]))]]),
      _gap(12),
      Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
        col(
          'Rotation',
          _dialHand(d, const Size(colW, colW)),
          Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            XyzStack(d, [
              Spec(const ['rot.z', 'rot.x', 'rot.y'][axisIdx], const ['Z', 'X', 'Y'][axisIdx], axis: const [2, 0, 1][axisIdx], unit: '°', per: .5),
            ]),
            if (three) ...[
              _gap(4),
              Row(children: [
                for (var i = 0; i < 3; i++)
                  Chip(const ['Z', 'X', 'Y'][i], on: axisIdx == i, enabled: !d.locked, onTap: () {
                    d.set('rot.axis', i.toDouble());
                  }),
              ]),
            ],
          ]),
        ),
        const SizedBox(width: 8),
        col(
          'Scale',
          _scaleHand(d, Size(colW, colW)),
          XyzStack(d, _scaleSpecs, gap: Gui.rowGap),
          trailing: Chip('Link', on: d['link'] > .5, enabled: !d.locked, onTap: () {
            d.set('link', d['link'] > .5 ? 0 : 1); // changes no artwork, only how the next drag behaves
          }),
        ),
        const SizedBox(width: 8),
        col(
          'Anchor',
          _anchorHand(d, Size(colW, colW)),
          Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            XyzStack(d, _anchorSpecs, gap: Gui.rowGap),
            _gap(4),
            Text(anchorName, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g56)),
          ]),
        ),
      ]),
    ]);
  }
}

/// (a) Blender-style: three XYZ stacks, nothing to grab.
class TransformFields extends StatelessWidget {
  const TransformFields(this.d, {super.key});
  final GDoc d;
  @override
  Widget build(BuildContext context) {
    Widget grp(String t, List<Spec> s, {Widget? trailing}) => Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [_title(t, trailing: trailing), _gap(6), XyzStack(d, s), _gap(12)]);
    return Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      _title('Space', trailing: _lockedWord(d)),
      _gap(6),
      _SpaceChips(d),
      _gap(12),
      grp('Position', _posSpecs(d)),
      grp('Rotation', _rotSpecs(d)),
      grp('Scale', _scaleSpecs, trailing: Chip('Link', on: d['link'] > .5, enabled: !d.locked, onTap: () => d.set('link', d['link'] > .5 ? 0 : 1))),
      grp('Anchor', _anchorSpecs),
    ]);
  }
}

/// (c) Both: the stacks are the primary, the instruments sit beside them as helpers and edit the same numbers.
class TransformBoth extends StatelessWidget {
  const TransformBoth(this.d, {super.key});
  final GDoc d;
  @override
  Widget build(BuildContext context) => LayoutBuilder(builder: (context, box) => _build(math.min(box.maxWidth, Gui.content)));

  // helpers take whatever width the column really has, minus the stacks: they shrink, they are never cut
  Widget _build(double w) {
    const stackW = 112.0, h = 74.0;
    final helperW = w - stackW - 8;
    Widget row(String t, List<Spec> s, Widget helper, {Widget? trailing}) => Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          _title(t, trailing: trailing),
          _gap(6),
          Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
            SizedBox(width: stackW, child: XyzStack(d, s)),
            const SizedBox(width: 8),
            helper,
          ]),
          _gap(12),
        ]);
    final axisIdx = d['rot.axis'].round().clamp(0, 2);
    // with the stacks primary, the dial follows the field you last touched; here it follows the axis chip
    final rotHelper = Column(children: [
      _dialHand(d, Size(helperW, h - 22)),
      if (d['space'] > 0)
        Row(children: [
          for (var i = 0; i < 3; i++)
            Chip(const ['Z', 'X', 'Y'][i], on: axisIdx == i, enabled: !d.locked, onTap: () => d.set('rot.axis', i.toDouble())),
        ]),
    ]);
    return Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      _title('Space', trailing: _lockedWord(d)),
      _gap(6),
      _SpaceChips(d),
      _gap(12),
      row('Position', _posSpecs(d), _padHand(d, Size(helperW, h))),
      row('Rotation', _rotSpecs(d), rotHelper),
      row('Scale', _scaleSpecs, _scaleHand(d, Size(helperW, h)), trailing: Chip('Link', on: d['link'] > .5, enabled: !d.locked, onTap: () => d.set('link', d['link'] > .5 ? 0 : 1))),
      row('Anchor', _anchorSpecs, _anchorHand(d, Size(helperW, h))),
    ]);
  }
}

// ---- 2. the Camera face ---------------------------------------------------------------------------------------------------------

List<double> _eye(GDoc d) {
  final p = d['cam.pitch'] * math.pi / 180, y = d['cam.yaw'] * math.pi / 180;
  return [-math.sin(y) * math.cos(p), math.sin(p), -math.cos(y) * math.cos(p)];
}

class _CamGeo {
  _CamGeo(this.size, this.d);
  final Size size;
  final GDoc d;
  Offset get c => size.center(Offset.zero);
  // an oblique view of the unit sphere, so a camera straight in front of the target still stands apart from it
  Offset get eye {
    final e = _eye(d);
    var v = Offset(e[0] - .62 * e[2], e[1] + .46 * e[2]);
    if (v.distance < .6) {
      v = v.distance < 1e-3 ? const Offset(.6, 0) : v / v.distance * .6;
    }
    return c + v * (Gui.orbitR * .95);
  }

  bool get behind => _eye(d)[2] > 0;
  Offset get ray {
    final v = eye - c;
    return v.distance < 1e-3 ? const Offset(-.8, .6) : v / v.distance;
  }

  Offset get framing {
    final t = (.5 + .28 * (math.log(d['cam.dist']) / math.ln10)).clamp(.22, .9);
    return c + (eye - c) * t;
  }

  Offset get roll {
    final a = (-90 + d['cam.roll']) * math.pi / 180;
    return c + Offset(math.cos(a), math.sin(a)) * Gui.ringR;
  }
}

Widget _faceHand(GDoc d, Size size) {
  final g = _CamGeo(size, d);
  final tone = [Fam.face.c, Fam.attach.c, Fam.stagger.c, Fam.scatter.c];
  final byLayer = d['cam.layer'] > .5;
  return _Hand(
    d: d,
    size: size,
    pick: (p) {
      if ((p - g.c).distance < 9 && !byLayer) {
        return 'target';
      }
      if ((p - g.framing).distance < 10) {
        return 'framing';
      }
      if ((p - g.eye).distance < 12) {
        return 'eye';
      }
      if (((p - g.c).distance - Gui.ringR).abs() < 9) {
        return 'roll';
      }
      return null;
    },
    drag: (grab, p, last, fine) {
      final k = fine ? Gui.fine : 1.0, dd = (p - last) * k;
      switch (grab) {
        case 'target':
          d.add('cam.tx', dd.dx * 4);
          d.add('cam.ty', dd.dy * 4);
        case 'eye': // the camera follows the pointer around the target
          d.set('cam.pitch', (d['cam.pitch'] + dd.dy * .8).clamp(-89.0, 89.0));
          d.add('cam.yaw', -dd.dx * .8);
        case 'framing': // outward along the ray is farther
          d.set('cam.dist', (d['cam.dist'] * math.exp((dd.dx * g.ray.dx + dd.dy * g.ray.dy) * .02)).clamp(.1, 50.0));
        case 'roll':
          d.add('cam.roll', _wrap((p - g.c).direction - (last - g.c).direction) * 180 / math.pi * k);
      }
    },
    paint: (cv, sz, hot, live) {
      _well(cv, sz);
      final a = d.locked ? .4 : 1.0, c = g.c, eye = g.eye;
      final dots = _fl(N.g15); // a quiet field that slides under the target as it moves: the world moves, the face stays
      for (var x = (-d['cam.tx'] / 4) % 16; x < sz.width; x += 16) {
        for (var y = (-d['cam.ty'] / 4) % 16; y < sz.height; y += 16) {
          cv.drawCircle(Offset(x, y), .7, dots);
        }
      }
      cv.drawCircle(c, Gui.ringR, _ln(tone[3].withValues(alpha: (hot == 'roll' || live && hot == 'roll' ? .9 : .35) * a)));
      for (var i = 0; i < 12; i++) {
        final t = i * math.pi / 6, u = Offset(math.cos(t), math.sin(t));
        cv.drawLine(c + u * (Gui.ringR - 3), c + u * (Gui.ringR + 3), _ln(tone[3].withValues(alpha: .3 * a)));
      }
      cv.drawCircle(c, Gui.orbitR, _ln(tone[1].withValues(alpha: .5 * a)));
      cv.drawOval(Rect.fromCenter(center: c, width: Gui.orbitR * 2, height: Gui.orbitR * .7), _ln(tone[1].withValues(alpha: .25 * a)));
      cv.drawLine(eye, c, _ln(tone[2].withValues(alpha: .9 * a)));
      final fh = g.framing;
      cv.save();
      cv.translate(fh.dx, fh.dy);
      cv.rotate(g.ray.direction + math.pi / 2);
      final bar = Rect.fromCenter(center: Offset.zero, width: 12, height: 4);
      cv.drawRect(bar, _fl(N.g07));
      cv.drawRect(bar, _ln(tone[2].withValues(alpha: a)));
      cv.restore();
      cv.save();
      cv.translate(eye.dx, eye.dy);
      cv.rotate((c - eye).direction);
      final body = const Rect.fromLTRB(-9, -6, 5, 6), lens = Path()..moveTo(5, -3.5)..lineTo(11, -6.5)..lineTo(11, 6.5)..lineTo(5, 3.5)..close();
      cv.drawRect(body, _fl(g.behind ? N.g07 : tone[1].withValues(alpha: .25 * a))); // hollow when the eye is on the far side
      cv.drawRect(body, _ln(tone[1].withValues(alpha: a)));
      cv.drawPath(lens, _ln(tone[1].withValues(alpha: a)));
      cv.restore();
      cv.drawCircle(c, 5, _fl(byLayer ? N.g07 : tone[0].withValues(alpha: .25 * a)));
      cv.drawCircle(c, 5, _ln(tone[0].withValues(alpha: a)));
      final cross = _ln(tone[0].withValues(alpha: .8 * a));
      for (final o in const [Offset(-1, 0), Offset(1, 0), Offset(0, -1), Offset(0, 1)]) {
        cv.drawLine(c + o * 8, c + o * 11, cross);
      }
      final rh = g.roll;
      cv.drawCircle(rh, 4.5, _fl(N.g07));
      cv.drawCircle(rh, 4.5, _ln(tone[3].withValues(alpha: a)));
    },
  );
}

List<Spec> _camSpecs(GDoc d, String part) {
  final byLayer = d['cam.layer'] > .5;
  final role = [Fam.face.c, Fam.attach.c, Fam.stagger.c, Fam.scatter.c];
  switch (part) {
    case 'target':
      return [
        Spec('cam.tx', 'X', color: role[0], unit: 'px', per: 4, on: !byLayer, word: byLayer ? 'Jewel 02' : null),
        Spec('cam.ty', 'Y', color: role[0], unit: 'px', per: 4, on: !byLayer, word: byLayer ? 'its layer' : null),
      ];
    case 'orbit':
      return [Spec('cam.pitch', 'P', color: role[1], unit: '°', per: .5), Spec('cam.yaw', 'Y', color: role[1], unit: '°', per: .5)];
    case 'framing':
      return [Spec('cam.dist', 'F', color: role[2], unit: '×', dec: 2, per: .01)];
    default:
      return [Spec('cam.roll', 'R', color: role[3], unit: '°', per: .5)];
  }
}

Widget _camNumbers(GDoc d) {
  Widget part(String t, String key) => Expanded(child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [Text(t, style: T.label(N.g63)), _gap(4), XyzStack(d, _camSpecs(d, key))]));
  return Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
    Row(crossAxisAlignment: CrossAxisAlignment.start, children: [part('Target', 'target'), const SizedBox(width: 8), part('Orbit', 'orbit')]),
    _gap(8),
    Row(crossAxisAlignment: CrossAxisAlignment.start, children: [part('Framing', 'framing'), const SizedBox(width: 8), part('Roll', 'roll')]),
  ]);
}

class CameraGui extends StatelessWidget {
  const CameraGui(this.d, {super.key, this.mode = Mode.gui});
  final GDoc d;
  final Mode mode;
  @override
  Widget build(BuildContext context) {
    final face = _faceHand(d, const Size(Gui.content, Gui.faceH));
    return Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      _title('Camera', trailing: _lockedWord(d)),
      _gap(6),
      if (mode == Mode.fields) _camNumbers(d) else if (mode == Mode.gui) ...[face, _gap(6), _camNumbers(d)] else ...[_camNumbers(d), _gap(8), face],
      if (mode != Mode.fields && d['cam.layer'] > .5) Cap('Target follows Jewel 02: the point fields disable and say why.'),
    ]);
  }
}

// ---- 3. the Layout mini-diagram ------------------------------------------------------------------------------------------------

class _LayGeo {
  _LayGeo(this.size, this.d);
  final Size size;
  final GDoc d;
  bool get on => d['lay.on'] > .5;
  int get cols => d['lay.cols'].round().clamp(1, Gui.layN);
  int get rows => (Gui.layN / cols).ceil();
  double get gap => on ? d['lay.gap'] * Gui.layK : 3;
  double get pad => on ? d['lay.pad'] * Gui.layK : 0;
  int get hs => on ? d['lay.hs'].round() : 0;
  int get vs => on ? d['lay.vs'].round() : 0;
  Rect get avail => Rect.fromLTRB(8, 8, size.width - 8, size.height - 8);
  Size get content => Size(cols * Gui.layCw + (cols - 1) * gap, rows * Gui.layCh + (rows - 1) * gap);
  Rect get parent {
    final w = hs == 0 ? content.width + 2 * pad : (hs == 1 ? avail.width : (d['lay.w'] * Gui.layK).clamp(40.0, avail.width));
    final h = vs == 0 ? content.height + 2 * pad : (vs == 1 ? avail.height : (d['lay.h'] * Gui.layK).clamp(30.0, avail.height));
    return Rect.fromLTWH(avail.left, avail.top, math.min(w, avail.width), math.min(h, avail.height));
  }

  Rect get inner => Rect.fromLTRB(parent.left + pad, parent.top + pad, math.max(parent.left + pad, parent.right - pad), math.max(parent.top + pad, parent.bottom - pad));
  List<Rect> get boxes => [for (var i = 0; i < Gui.layN; i++) Rect.fromLTWH(inner.left + (i % cols) * (Gui.layCw + gap), inner.top + (i ~/ cols) * (Gui.layCh + gap), Gui.layCw, Gui.layCh)];
  Offset get gapAt {
    final b = boxes;
    return cols > 1 ? Offset((b[0].right + b[1].left) / 2, b[0].center.dy) : Offset(b[0].center.dx, (b[0].bottom + b[1].top) / 2);
  }

  Offset get colAt => Offset(boxes[cols - 1].right + 7, boxes[0].center.dy);
  Offset get padAt => Offset(parent.left + pad, parent.top + pad);
  Offset get rightAt => Offset(parent.right, parent.center.dy);
  Offset get bottomAt => Offset(parent.center.dx, parent.bottom);
}
