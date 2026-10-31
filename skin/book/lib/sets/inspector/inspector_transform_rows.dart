import 'package:flutter/gestures.dart';
import 'package:flutter/widgets.dart';
import 'package:lab_book/forms/forms.dart';
import 'package:lab_book/sets/inspector/inspector_rows_prop_row.dart';
import 'package:lab_book/sets/inspector/inspector_rows_sect.dart';
import '../../tokens.dart';
import 'inspector_direct_transform.dart';
import 'inspector_parts.dart';

// ---- 8. the transform group ------------------------------------------------------------------------------------------
class SpaceRow extends StatelessWidget {
  const SpaceRow({super.key});
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc;
    // contract: 2D / 2.5D / 3D is one switch; every space shows the same parameters; the space only changes how the layer meets the camera (2D held to it), never a value (B2).
    // (only the spaces the document takes for it, by its words: a flat thing's 2d and 2.5d; offered 3D, it was refused
    //  and the plate stayed lit as if taken: golf P05)
    final words = doc.choices['v_space'] ?? const <String>[];
    final offered = [for (final s in Space.values) if (words.isEmpty || words.contains(const {Space.d2: '2d', Space.d25: '2.5d', Space.d3: '3d'}[s])) s];
    return PropRow(
      id: 'v_space',
      label: 'Space',
      keyable: false,
      enabled: !x.cfg.locked,
      cells: [
        Forms.worn.choice( // (as the skin's forms draw one of a few)
          context,
          ChoiceSpec(
            labels: [for (final s in offered) spaceNames[s]!],
            glyphs: [for (final s in offered) (c, st) => _SpacePaint(s, st.color).paint(c, const Size(16, 14))],
            selected: offered.indexOf(doc.space),
            enabled: !x.cfg.locked,
            onSelect: (i) {
              doc.space = offered[i];
              doc.poke();
            },
          ),
        ),
      ],
    );
  }
}

/// A space as a picture and its name: flat square, tilted card, cube.
/// One of a few choices as a plate: its glyph says it, its word only names it (a hand finds it with the words hidden).
class GlyphPlate extends StatelessWidget {
  const GlyphPlate(this.label, this.glyph, {super.key, required this.on, required this.onTap});
  final String label;
  final void Function(Canvas c, Paint stroke) glyph; // (drawn in a 16 x 14 box)
  final bool on;
  final VoidCallback onTap;
  @override
  Widget build(BuildContext context) => Semantics(
    button: true,
    selected: on,
    label: label,
    child: Hov(
      onTap: onTap,
      builder: (_, h) {
        final pop = Pop.on(context);
        final ink = on ? (pop ? Pop.onAccent : Grey.g95) : (h ? Grey.g76 : Grey.g56);
        return Container(
          height: pop ? 22 : 26,
          decoration: pop
              ? BoxDecoration(color: on ? Pop.accent : (h ? Grey.g20 : Pop.well), borderRadius: BorderRadius.all(Shape.r(11)))
              : BoxDecoration(
                  color: on ? Grey.g20 : (h ? Grey.g15 : Grey.g13),
                  borderRadius: BorderRadius.all(Shape.r(5)),
                  border: Border(bottom: BorderSide(color: on ? Role.selected : const Color(0x00000000))),
                ),
          child: Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              CustomPaint(size: const Size(16, 14), painter: _GlyphPaint(glyph, ink)),
              SizedBox(width: Shape.s(6)),
              Flexible( // (narrow, the word gives way; the glyph stays)
                child: Text(label, maxLines: 1, overflow: TextOverflow.clip, softWrap: false, style: pop ? T.micro(on ? Pop.onAccent : Grey.g76).copyWith(fontWeight: FontWeight.w700) : T.label(on ? Grey.g95 : Grey.g63)),
              ),
            ],
          ),
        );
      },
    ),
  );
}

class _GlyphPaint extends CustomPainter {
  const _GlyphPaint(this.glyph, this.ink);
  final void Function(Canvas c, Paint stroke) glyph;
  final Color ink;
  @override
  void paint(Canvas c, Size s) => glyph(
    c,
    Paint()
      ..color = ink
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.2
      ..strokeCap = StrokeCap.round
      ..strokeJoin = StrokeJoin.round,
  );
  @override
  bool shouldRepaint(_GlyphPaint o) => o.ink != ink || o.glyph != glyph;
}

/// Lines of type set left, centred or right (0, .5, 1: ThorVG's own): the glyph of a text's alignment.
void Function(Canvas, Paint) alignGlyph(double at) => (c, st) {
  for (final (y, w) in const [(2.5, 14.0), (7.0, 9.0), (11.5, 12.0)]) {
    final x = 1 + (14 - w) * at;
    c.drawLine(Offset(x, y), Offset(x + w, y), st);
  }
};

/// The spaces' names.
const spaceNames = {Space.d2: '2D', Space.d25: '2.5D', Space.d3: '3D'};

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
    return PropRow(
      id: 'v_position',
      label: 'Position',
      ids: ['v_x', 'v_y', 'v_z'],
      cells: [
        NumField(id: 'v_x', label: 'X', axis: axX, unit: 'px', decimals: 1),
        NumField(id: 'v_y', label: 'Y', axis: axY, unit: 'px', decimals: 1),
        NumField(id: 'v_z', label: 'Z', axis: axZ, unit: 'px', decimals: 1),
      ],
    );
  }
}

class ScaleRow extends StatelessWidget {
  const ScaleRow({super.key});
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc;
    final axes = [for (final a in const ['x', 'y', 'z']) if (doc.seat('v_scale_$a')) a]; // (only the axes the thing has: a light none, a film no depth)
    if (axes.isEmpty) return const SizedBox.shrink();
    // contract: with the link on, a factor applies to every axis (2:3, X=4 gives 4:6); unlinked writes only the touched axis; a zero axis gets the typed value (E1). Toggling the link changes nothing.
    void set(String a, double v) {
      final id = 'v_scale_$a';
      final linked = [for (final o in axes) if (o != 'z' || doc.kindWord == 'Solid') o];   // (a flat thing's Z is its thickness, no part of its size)
      if (!doc.link || !linked.contains(a)) {
        doc.set(id, v);
        return;
      }
      final old = doc.get(id) ?? 100;
      if (old == 0) {
        doc.setMany({for (final o in linked) 'v_scale_$o': v});
        return;
      }
      final f = v / old;
      doc.setMany({for (final o in linked) 'v_scale_$o': o == a ? v : (doc.get('v_scale_$o') ?? 100) * f});
    }

    NumField cell(int i) =>
        NumField(id: 'v_scale_${axes[i]}', label: axes[i].toUpperCase(), axis: axesParts[i], unit: '%', decimals: 1, onSet: (v) => set(axes[i], v));
    return PropRow(
      id: 'v_scale',
      label: 'Scale',
      ids: [for (final a in axes) 'v_scale_$a'],
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
    return PropRow(
      id: 'v_rotate',
      label: 'Rotation',
      ids: ['v_rotate_x', 'v_rotate_y', 'v_rotate'],
      cells: [
        NumField(id: 'v_rotate_x', label: 'X', axis: axX, unit: '°', decimals: 1, perPx: .5),
        NumField(id: 'v_rotate_y', label: 'Y', axis: axY, unit: '°', decimals: 1, perPx: .5),
        NumField(id: 'v_rotate', label: 'Z', axis: axZ, unit: '°', decimals: 1, perPx: .5, turns: true),
      ],
    );
  }
}

const anchorNames = ['Top left', 'Top', 'Top right', 'Left', 'Centre', 'Right', 'Bottom left', 'Bottom', 'Bottom right'];

class AnchorRow extends StatelessWidget {
  const AnchorRow({super.key});
  @override
  Widget build(BuildContext context) => PropRow(
    id: 'v_anchor',
    label: 'Anchor',
    ids: ['v_anchor_x', 'v_anchor_y', 'v_anchor_z'],
    lead: AnchorPad(),
    cells: [
      NumField(id: 'v_anchor_x', label: 'X', axis: axX, unit: '%', decimals: 0),
      NumField(id: 'v_anchor_y', label: 'Y', axis: axY, unit: '%', decimals: 0),
      NumField(id: 'v_anchor_z', label: 'Z', axis: axZ, unit: '%', decimals: 0),
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
    _doc!.setMany({'v_anchor_x': to.dx.roundToDouble(), 'v_anchor_y': to.dy.roundToDouble()});
  }

  @override
  void dispose() {
    if (_down) _doc?.cancelGesture();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc;
    return ListenableBuilder(
      listenable: anchorMoved(doc),
      builder: (context, _) {
        final ax = doc.get('v_anchor_x'), ay = doc.get('v_anchor_y');
        final on = !x.cfg.locked && !RowOff.offOf(context) && ax != null && ay != null;
        final at = ax == null || ay == null ? null : Offset(ax, ay);
        final sel = at == null ? null : [for (var i = 0; i < 9; i++) i].where((i) => _snap(i) == at).firstOrNull;
        final hot = doc.directHover == 'v_anchor';
        return Semantics(
          label: 'Anchor: ${at == null ? 'Mixed' : (sel != null ? anchorNames[sel] : 'Custom')}',
          child: MouseRegion(
            cursor: on ? SystemMouseCursors.precise : SystemMouseCursors.basic,
            onHover: (e) {
              if (!on) return;
              final n = _nearest(_pct(e.localPosition));
              if (n != _near) setState(() => _near = n);
              if (!hot) doc.setDirectHover('v_anchor');
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
      },
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
    c.drawRRect(RRect.fromRectAndRadius(box, Shape.r(2)), Paint()..color = on && hot ? Grey.g15 : Grey.g10);
    c.drawRRect(
      RRect.fromRectAndRadius(box, Shape.r(2)),
      Paint()
        ..color = Grey.g44
        ..style = PaintingStyle.stroke,
    );
    for (var i = 0; i < 9; i++) {
      c.drawCircle(point(Offset((i % 3) * 50.0, (i ~/ 3) * 50.0)), (i == near ? 1.6 : 1) * s.height / 18, Paint()..color = i == near ? Grey.g76 : Grey.g38);
    }
    final a = at;
    if (a == null) return;
    final ink = !on ? Grey.g56 : (hot ? TransformZone.pivot.color : Grey.g95);
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
  Widget build(BuildContext context) => PropRow(
    id: 'v_opacity',
    label: 'Opacity',
    cells: [NumField(id: 'v_opacity', unit: '%', decimals: 0, min: 0, max: 100, perPx: .5)],
  );
}

class DepthRow extends StatelessWidget {
  const DepthRow({super.key});
  @override
  // contract: 0 reads as the word "Flat" (E3).
  Widget build(BuildContext context) => PropRow(
    id: 'depth',
    label: 'Depth',
    cells: [NumField(id: 'depth', unit: 'px', decimals: 0, zeroWord: 'Flat')],
  );
}

const layers = [
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
  Widget build(BuildContext context) => PropRow(
    id: 'v_parent',
    label: 'Parent',
    keyable: false,
    cells: [Chooser(id: 'v_parent', options: Ctx.of(context).doc.choices['v_parent'] ?? layers, searchable: true)],
  );
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
