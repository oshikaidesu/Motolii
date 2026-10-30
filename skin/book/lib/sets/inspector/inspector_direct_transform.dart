import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_direct_transform_canvas.dart';
import 'package:lab_book/sets/inspector/inspector_direct_transform_value_glyph.dart';
import 'package:lab_book/sets/inspector/inspector_rows_sect.dart';

import '../../theme.dart';
import '../../tokens.dart';
import 'inspector_parts.dart';

/// What a point on the canvas grabs. Each zone edits one Transform row.
/// The colour is the zone's name: the canvas part and its row wear the same one.
enum TransformZone {
  move('v_position', 'HandleMove', Color(0xFF3D6BFF)),
  scale('v_scale', 'HandleScale', Color(0xFFB3C66B)),
  rotate('v_rotate', 'HandleRotate', Color(0xFFFF6A2B)),
  pivot('v_anchor', 'HandlePivot', Color(0xFFB57CFF)),
  lift('v_position', 'HandleLift', Color(0xFF8FB0FF));

  const TransformZone(this.prop, this.role, this.usual);
  final String prop, role;
  final Color usual;
  Color get color => ColorTheme.of(role, usual); // (the skin's handle colours)
}


abstract final class DirectSurface {
  // surface: one canvas for the whole Transform block, sized to keep the rows below it on screen.
  static double get height => Shape.z(108.0);
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
        'v_position': ValueGlyph(ValueShape.position),
        'v_scale': ValueGlyph(ValueShape.scale),
        'v_rotate': ValueGlyph(ValueShape.rotation),
        'v_opacity': ValueGlyph(ValueShape.opacity),
        'depth': ValueGlyph(ValueShape.depth),
        'v_parent': ValueGlyph(ValueShape.parent),
      },
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        mainAxisSize: MainAxisSize.min,
        children: [
          Sect(
            title: 'Transform',
            first: first,
            tone: Pop.toneTransform,
            keyIds: const ['v_position', 'v_scale', 'v_rotate'],
            mark: CardMark.transform,
            trailing: CardWords('Transform', [for (final n in const ['anchor_x', 'anchor_y', 'anchor_z', 'x', 'y', 'z', 'scale_x', 'scale_y', 'scale_z', 'skew_x', 'skew_y', 'rotate_x', 'rotate_y', 'rotate', 'opacity']) if (doc.seat('v_$n')) n], more: [
              if (doc.onArrange case final to?)   // (Alight Motion's ⋯ on Move & Transform; they lay in the menu bar alone: golf_noise.md 【入口】)
                for (final (how, word) in const [('flipX', 'Flip Horizontal'), ('flipY', 'Flip Vertical'), ('fit', 'Fit to Picture'), ('fill', 'Fill Picture'), ('stretch', 'Stretch to Picture')]) (how, word, () => to(how)),
            ]),
            brief: ['${brief(doc.get('v_x'))}, ${brief(doc.get('v_y'))}', '${brief(doc.get('v_scale_x'))}%', '${brief(doc.get('v_rotate'))}°'],
            children: [
              SpaceRow(),
              TransformCanvas(),
              SizedBox(height: Pop.titleGap),
              AxisColumns(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  mainAxisSize: MainAxisSize.min,
                  children: [AxisHeader(), PosRow(), ScaleRow(), RotRow(), SkewRow()],
                ),
              ),
            ],
          ),
          const AnchorCard(),
          Sect(
            title: 'Layer',
            tone: Pop.toneLayer,
            keyIds: const ['v_opacity'],
            mark: CardMark.layer,
            brief: [if (doc.seat('v_opacity')) '${brief(doc.get('v_opacity'), 0)}%', if (doc.s2['v_parent'] case final p? when p != 'None') p],
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
