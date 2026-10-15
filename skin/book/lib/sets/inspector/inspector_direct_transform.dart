import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_direct_transform_canvas.dart';
import 'package:lab_book/sets/inspector/inspector_direct_transform_value_glyph.dart';

import 'inspector_parts.dart';

/// What a point on the canvas grabs. Each zone edits one Transform row.
/// The colour is the zone's name: the canvas part and its row wear the same one.
enum TransformZone {
  move('pos', Color(0xFF3D6BFF)),
  scale('scale', Color(0xFFB3C66B)),
  rotate('rot', Color(0xFFFF6A2B)),
  pivot('anchor', Color(0xFFB57CFF)),
  lift('pos', Color(0xFF8FB0FF));

  const TransformZone(this.prop, this.color);
  final String prop;
  final Color color;
}


abstract final class DirectSurface {
  // surface: one canvas for the whole Transform block, sized to keep the rows below it on screen.
  static const height = 124.0;
  static const objectWidth = 64.0, objectHeight = 40.0, handle = 7.0;
  static const stem = 20.0, knob = 6.0, turnArc = 11.0;
  static const tiltStem = 16.0, rail = 12.0, zReach = 1000.0;
}

class DirectTransform extends StatelessWidget {
  const DirectTransform({super.key, this.titled = false, this.first = true});
  final bool titled, first;

  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    return RowMark(
      marks: const {
        'pos': ValueGlyph(ValueShape.position),
        'scale': ValueGlyph(ValueShape.scale),
        'rot': ValueGlyph(ValueShape.rotation),
        'op': ValueGlyph(ValueShape.opacity),
        'depth': ValueGlyph(ValueShape.depth),
        'parent': ValueGlyph(ValueShape.parent),
      },
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        mainAxisSize: MainAxisSize.min,
        children: [
          Sect(
            title: 'Transform',
            first: first,
            tone: Pop.toneTransform,
            keyIds: const ['pos', 'scale', 'rot'],
            mark: CardMark.transform,
            brief: ['${brief(doc.get('pos.x'))}, ${brief(doc.get('pos.y'))}', '${brief(doc.get('scale.x'))}%', '${brief(doc.get('rot.z'))}°'],
            children: const [
              SpaceRow(),
              TransformCanvas(),
              SizedBox(height: Pop.titleGap),
              AxisColumns(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  mainAxisSize: MainAxisSize.min,
                  children: [AxisHeader(), PosRow(), ScaleRow(), RotRow()],
                ),
              ),
            ],
          ),
          const AnchorCard(),
          Sect(
            title: 'Layer',
            tone: Pop.toneLayer,
            keyIds: const ['op'],
            mark: CardMark.layer,
            brief: [if (doc.seat('op')) '${brief(doc.get('op'), 0)}%', if (doc.s2['parent'] case final p? when p != 'None') p],
            children: const [LayerTiles()],
          ),
        ],
      ),
    );
  }
}

enum ValueShape {
  position(TransformZone.move),
  scale(TransformZone.scale),
  rotation(TransformZone.rotate),
  opacity(null),
  depth(null),
  parent(null);

  const ValueShape(this.zone);
  final TransformZone? zone;
}

Offset rotate(Offset p, double angle) => Offset(p.dx * math.cos(angle) - p.dy * math.sin(angle), p.dx * math.sin(angle) + p.dy * math.cos(angle));
