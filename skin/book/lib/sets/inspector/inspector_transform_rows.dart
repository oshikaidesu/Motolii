part of 'inspector_parts.dart';

// ---- 8. the transform group ------------------------------------------------------------------------------------------
class SpaceRow extends StatelessWidget {
  const SpaceRow({super.key});
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc;
    // contract: 2D / 2.5D / 3D is one switch; every space shows the same parameters; the space only changes how the layer meets the camera (2D held to it), never a value (B2).
    return PropRow(
      id: 'space',
      label: 'Space',
      keyable: false,
      enabled: !x.cfg.locked,
      cells: [
        Row(
          children: [
            for (final s in Space.values) ...[
              if (s != Space.d2) const SizedBox(width: 4),
              Expanded(
                child: SpacePlate(
                  s,
                  on: doc.space == s,
                  onTap: () {
                    doc.space = s;
                    doc.poke();
                  },
                ),
              ),
            ],
          ],
        ),
      ],
    );
  }
}

/// A space as a picture and its name: flat square, tilted card, cube.
class SpacePlate extends StatelessWidget {
  const SpacePlate(this.space, {super.key, required this.on, required this.onTap});
  final Space space;
  final bool on;
  final VoidCallback onTap;
  static const names = {Space.d2: '2D', Space.d25: '2.5D', Space.d3: '3D'};
  @override
  Widget build(BuildContext context) => Semantics(
    button: true,
    selected: on,
    label: names[space],
    child: Hov(
      onTap: onTap,
      builder: (_, h) {
        final pop = Pop.on(context);
        final ink = on ? (pop ? Pop.onAccent : Grey.g95) : (h ? Grey.g76 : Grey.g56);
        return Container(
          height: pop ? 22 : 26,
          decoration: pop
              ? BoxDecoration(color: on ? Pop.accent : (h ? Grey.g20 : Pop.well), borderRadius: BorderRadius.circular(11))
              : BoxDecoration(
                  color: on ? Grey.g20 : (h ? Grey.g15 : Grey.g13),
                  borderRadius: BorderRadius.circular(5),
                  border: Border(bottom: BorderSide(color: on ? Role.selected : const Color(0x00000000))),
                ),
          child: Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              CustomPaint(size: const Size(16, 14), painter: _SpacePaint(space, ink)),
              const SizedBox(width: 6),
              Text(names[space]!, style: pop ? T.micro(on ? Pop.onAccent : Grey.g76).copyWith(fontWeight: FontWeight.w700) : T.label(on ? Grey.g95 : Grey.g63)),
            ],
          ),
        );
      },
    ),
  );
}

class _SpacePaint extends CustomPainter {
  const _SpacePaint(this.space, this.ink);
  final Space space;
  final Color ink;
  @override
  void paint(Canvas c, Size s) {
    final st = Paint()
      ..color = ink
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.2
      ..strokeJoin = StrokeJoin.round;
    Path poly(List<Offset> p) => Path()..addPolygon(p, true);
    switch (space) {
      case Space.d2:
        c.drawRect(const Rect.fromLTWH(2, 1.5, 12, 11), st);
      case Space.d25:
        c.drawPath(poly(const [Offset(5, 2), Offset(15, 2), Offset(11, 12), Offset(1, 12)]), st);
      case Space.d3:
        c.drawPath(poly(const [Offset(1, 4.5), Offset(10, 4.5), Offset(10, 13), Offset(1, 13)]), st);
        c.drawPath(poly(const [Offset(1, 4.5), Offset(5, 1), Offset(14, 1), Offset(10, 4.5)]), st);
        c.drawPath(poly(const [Offset(10, 4.5), Offset(14, 1), Offset(14, 9.5), Offset(10, 13)]), st);
    }
  }

  @override
  bool shouldRepaint(_SpacePaint o) => o.space != space || o.ink != ink;
}

class PosRow extends StatelessWidget {
  const PosRow({super.key});
  @override
  Widget build(BuildContext context) {
    // 2D is a layer held to the camera, not a layer without Z: Z shows in every space
    return const PropRow(
      id: 'pos',
      label: 'Position',
      ids: ['pos.x', 'pos.y', 'pos.z'],
      cells: [
        NumField(id: 'pos.x', label: 'X', axis: _axX, unit: 'px', decimals: 1),
        NumField(id: 'pos.y', label: 'Y', axis: _axY, unit: 'px', decimals: 1),
        NumField(id: 'pos.z', label: 'Z', axis: _axZ, unit: 'px', decimals: 1),
      ],
    );
  }
}

class ScaleRow extends StatelessWidget {
  const ScaleRow({super.key});
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc;
    const axes = ['x', 'y', 'z'];
    // contract: with the link on, a factor applies to every axis (2:3, X=4 gives 4:6); unlinked writes only the touched axis; a zero axis gets the typed value (E1). Toggling the link changes nothing.
    void set(String a, double v) {
      final id = 'scale.$a';
      if (!doc.link) {
        doc.set(id, v);
        return;
      }
      final old = doc.get(id) ?? 100;
      if (old == 0) {
        doc.setMany({for (final o in axes) 'scale.$o': v});
        return;
      }
      final f = v / old;
      doc.setMany({for (final o in axes) 'scale.$o': o == a ? v : (doc.get('scale.$o') ?? 100) * f});
    }

    NumField cell(int i) =>
        NumField(id: 'scale.${axes[i]}', label: axes[i].toUpperCase(), axis: _axes[i], unit: '%', decimals: 1, onSet: (v) => set(axes[i], v));
    return PropRow(
      id: 'scale',
      label: 'Scale',
      ids: [for (final a in axes) 'scale.$a'],
      // the link sits beside the label so the X/Y columns stay aligned with every other row (B1)
      trail: LinkMark(
        on: doc.link,
        enabled: !x.cfg.locked,
        onTap: () {
          doc.link = !doc.link;
          doc.poke();
        },
      ),
      cells: [for (var i = 0; i < axes.length; i++) cell(i)],
    );
  }
}

class RotRow extends StatelessWidget {
  const RotRow({super.key});
  @override
  Widget build(BuildContext context) {
    // contract: rotation keeps its turns, typed 450 stays 450 and reads "1x 90" (B5).
    return const PropRow(
      id: 'rot',
      label: 'Rotation',
      ids: ['rot.x', 'rot.y', 'rot.z'],
      cells: [
        NumField(id: 'rot.x', label: 'X', axis: _axX, unit: '°', decimals: 1, perPx: .5),
        NumField(id: 'rot.y', label: 'Y', axis: _axY, unit: '°', decimals: 1, perPx: .5),
        NumField(id: 'rot.z', label: 'Z', axis: _axZ, unit: '°', decimals: 1, perPx: .5, turns: true),
      ],
    );
  }
}

const _anchorNames = ['Top left', 'Top', 'Top right', 'Left', 'Centre', 'Right', 'Bottom left', 'Bottom', 'Bottom right'];

class AnchorRow extends StatelessWidget {
  const AnchorRow({super.key});
  @override
  Widget build(BuildContext context) => const PropRow(
    id: 'anchor',
    label: 'Anchor',
    ids: ['anchor.x', 'anchor.y'],
    lead: AnchorPad(),
    cells: [
      NumField(id: 'anchor.x', label: 'X', axis: _axX, unit: '%', decimals: 0),
      NumField(id: 'anchor.y', label: 'Y', axis: _axY, unit: '%', decimals: 0),
    ],
  );
}

/// The layer as a small box with its pivot as a dot inside. A click snaps the pivot to the nearest of nine points; a drag places it anywhere.
/// contract: a custom pivot is named Custom and typed values still work; the layer does not move visually (B4).
class AnchorPad extends StatefulWidget {
  const AnchorPad({super.key, this.size = const Size(24, 18)});
  final Size size;
  @override
  State<AnchorPad> createState() => _AnchorPadState();
}

class _AnchorPadState extends State<AnchorPad> {
  int? _near;
  bool _down = false, _moved = false;
  Doc? _doc;

  Offset _pct(Offset p) {
    final s = widget.size, inset = 1.5 + (s.height / 18 - 1) * 7;
    return Offset(((p.dx - inset) / (s.width - inset * 2) * 100).clamp(0, 100), ((p.dy - inset) / (s.height - inset * 2) * 100).clamp(0, 100));
  }

  static int _nearest(Offset pct) => (pct.dy / 50).round() * 3 + (pct.dx / 50).round();
  static Offset _snap(int i) => Offset((i % 3) * 50.0, (i ~/ 3) * 50.0);

  void _place(Offset local, {required bool snap}) {
    final pct = _pct(local), i = _nearest(pct);
    final to = snap || (pct - _snap(i)).distance < 8 ? _snap(i) : pct;
    _doc!.setMany({'anchor.x': to.dx.roundToDouble(), 'anchor.y': to.dy.roundToDouble()});
  }

  @override
  void dispose() {
    if (_down) _doc?.cancelGesture();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, ax = doc.get('anchor.x'), ay = doc.get('anchor.y');
    final on = !x.cfg.locked && !RowOff.offOf(context) && ax != null && ay != null;
    final at = ax == null || ay == null ? null : Offset(ax, ay);
    final sel = at == null ? null : [for (var i = 0; i < 9; i++) i].where((i) => _snap(i) == at).firstOrNull;
    final hot = doc.directHover == 'anchor';
    return Semantics(
      label: 'Anchor: ${at == null ? 'Mixed' : (sel != null ? _anchorNames[sel] : 'Custom')}',
      child: MouseRegion(
        cursor: on ? SystemMouseCursors.precise : SystemMouseCursors.basic,
        onHover: (e) {
          if (!on) return;
          final n = _nearest(_pct(e.localPosition));
          if (n != _near) setState(() => _near = n);
          if (!hot) doc.setDirectHover('anchor');
        },
        onExit: (_) {
          setState(() => _near = null);
          if (hot && !_down) doc.setDirectHover(null);
        },
        child: Listener(
          onPointerDown: (e) {
            if (!on || e.buttons != kPrimaryButton) return;
            _doc = doc..beginGesture();
            _down = true;
            _moved = false;
          },
          onPointerMove: (e) {
            if (!_down) return;
            _moved = true;
            _place(e.localPosition, snap: false);
          },
          onPointerUp: (e) {
            if (!_down) return;
            if (!_moved) _place(e.localPosition, snap: true);
            _down = false;
            _doc?.endGesture();
          },
          onPointerCancel: (_) {
            if (!_down) return;
            _down = false;
            _doc?.cancelGesture();
          },
          child: CustomPaint(size: widget.size, painter: _AnchorPaint(at, _near, on, hot)),
        ),
      ),
    );
  }
}

class _AnchorPaint extends CustomPainter {
  const _AnchorPaint(this.at, this.near, this.on, this.hot);
  final Offset? at;
  final int? near;
  final bool on, hot;
  @override
  void paint(Canvas c, Size s) {
    final box = (Offset.zero & s).deflate(1.5), inner = box.deflate((s.height / 18 - 1) * 7);
    Offset point(Offset pct) => inner.topLeft + Offset(pct.dx / 100 * inner.width, pct.dy / 100 * inner.height);
    c.drawRRect(RRect.fromRectAndRadius(box, const Radius.circular(2)), Paint()..color = on && hot ? Grey.g15 : Grey.g10);
    c.drawRRect(
      RRect.fromRectAndRadius(box, const Radius.circular(2)),
      Paint()
        ..color = Grey.g44
        ..style = PaintingStyle.stroke,
    );
    for (var i = 0; i < 9; i++) {
      c.drawCircle(point(Offset((i % 3) * 50.0, (i ~/ 3) * 50.0)), (i == near ? 1.6 : 1) * s.height / 18, Paint()..color = i == near ? Grey.g76 : Grey.g38);
    }
    final a = at;
    if (a == null) return;
    final ink = !on ? Grey.g56 : (hot ? const Color(0xFFB57CFF) : Grey.g95);
    final p = point(a), k = s.height / 18;
    c.drawCircle(p, 3.2 * k, Paint()..color = Grey.g07);
    c.drawCircle(
      p,
      3.2 * k,
      Paint()
        ..color = ink
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.4,
    );
    c.drawCircle(p, k, Paint()..color = ink);
  }

  @override
  bool shouldRepaint(_AnchorPaint o) => o.at != at || o.near != near || o.on != on || o.hot != hot;
}

class OpacityRow extends StatelessWidget {
  const OpacityRow({super.key});
  @override
  // contract: opacity is 0-100 %, display only (0.5 stored reads 50 %); the fill shows where it sits in its range (B6, E2).
  Widget build(BuildContext context) => const PropRow(
    id: 'op',
    label: 'Opacity',
    cells: [NumField(id: 'op', unit: '%', decimals: 0, min: 0, max: 100, perPx: .5)],
  );
}

class DepthRow extends StatelessWidget {
  const DepthRow({super.key});
  @override
  // contract: 0 reads as the word "Flat" (E3).
  Widget build(BuildContext context) => const PropRow(
    id: 'depth',
    label: 'Depth',
    cells: [NumField(id: 'depth', unit: 'px', decimals: 0, zeroWord: 'Flat')],
  );
}

const _layers = [
  'None',
  'Backdrop',
  'Hero title',
  'Jewel Field',
  'Jewel 01',
  'Jewel 02',
  'Jewel 03',
  'Jewel 04',
  'Jewel 05',
  'Jewel 06',
  'Glow plate',
  'Lower third',
  'Logo',
  'Camera 1',
  'Audio bars',
  'Vignette',
  'Null 1',
  'Null 2',
];

class ParentRow extends StatelessWidget {
  const ParentRow({super.key});
  @override
  // contract: shows the parent's name or None; a list with search, never prev/next arrows; changing it does not move the layer (G3).
  Widget build(BuildContext context) => const PropRow(
    id: 'parent',
    label: 'Parent',
    keyable: false,
    cells: [Chooser(id: 'parent', options: _layers, searchable: true)],
  );
}

/// Opacity, Depth and Parent as blocks in the same flowing grid as an effect's parameters: three across in a normal panel.
/// The picture in each block's corner is the row mark the Transform rows use.
class LayerTiles extends StatelessWidget {
  const LayerTiles({super.key});
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc, on = !Ctx.of(context).cfg.locked;
    Widget num(String id, String caption, Widget glyph, {required String unit, double? min, double? max, double perPx = 1, String? zeroWord}) => _Revealed(
      id: id,
      child: _RowInfo(
        unit: null,
        keys: doc.keys[id] ?? KeyS.off,
        child: NumField(id: id, caption: caption, glyph: glyph, unit: unit, min: min, max: max, perPx: perPx, zeroWord: zeroWord, enabled: on),
      ),
    );
    return _LdGrid([
      _LdCell(num('op', 'Opacity', const ValueGlyph(ValueShape.opacity), unit: '%', min: 0, max: 100, perPx: .5)),
      _LdCell(num('depth', 'Depth', const ValueGlyph(ValueShape.depth), unit: 'px', zeroWord: 'Flat')),
      _LdCell(
        _Revealed(
          id: 'parent',
          child: _LdTile(
            label: 'Parent',
            glyph: const ValueGlyph(ValueShape.parent),
            child: SizedBox(
              height: 22,
              child: Chooser(id: 'parent', options: _layers, searchable: true, enabled: on),
            ),
          ),
        ),
      ),
    ]);
  }
}

/// Brings a block on screen when its property shortcut fires (I2), as [PropRow] does for a row.
class _Revealed extends StatefulWidget {
  const _Revealed({required this.id, required this.child});
  final String id;
  final Widget child;
  @override
  State<_Revealed> createState() => _RevealedState();
}

class _RevealedState extends State<_Revealed> {
  bool _was = false;
  @override
  Widget build(BuildContext context) {
    final revealed = Ctx.of(context).doc.revealed == widget.id;
    if (revealed && !_was) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted) Scrollable.ensureVisible(context, alignment: .2, duration: const Duration(milliseconds: 160), curve: Curves.easeOut);
      });
    }
    _was = revealed;
    return widget.child;
  }
}

class TransformGroup extends StatelessWidget {
  const TransformGroup({super.key, this.titled = true, this.first = true});
  final bool titled, first;
  @override
  Widget build(BuildContext context) => Sect(
    title: titled ? 'Transform' : null,
    first: first,
    children: const [SpaceRow(), PosRow(), ScaleRow(), RotRow(), AnchorRow(), OpacityRow(), DepthRow(), ParentRow()],
  );
}
