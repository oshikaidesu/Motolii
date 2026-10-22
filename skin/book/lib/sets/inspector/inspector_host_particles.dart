import 'dart:math' as math;

import 'inspector_studies.dart';
import 'package:flutter/widgets.dart';

import '../../tokens.dart';
import 'inspector_parts.dart';

// ---- host: particles ---------------------------------------------------------------------------------------------------
/// The particle layer's table (motolii-doc store/particles.rs ROWS): lengths in comp px, time in seconds, angles in degrees (0 = right, -90 = up).
/// Shares (randomness, opacity, bounce) are shown as percent; colours live in doc.s2 as #RRGGBB.
const ptEmit = [
  Param(id: 'rate', label: 'Rate', look: ParamLook.scrub, type: WType.f32, min: 0, def: 40, unit: '/s'),
  Param(id: 'life', label: 'Life', look: ParamLook.scrub, type: WType.f32, min: .01, def: 2, unit: 's', dec: 2),
  Param(id: 'emitter', label: 'Emitter', look: ParamLook.size, type: WType.vec2, min: 0, unit: 'px'),
];
const ptMotion = [
  Param(id: 'direction', label: 'Direction', look: ParamLook.angle, type: WType.f32, def: -90, unit: '°'),
  Param(id: 'spread', label: 'Spread', look: ParamLook.angle, type: WType.f32, min: 0, max: 360, def: 30, unit: '°'),
  ptSpeed,
];
const ptSpeed = Param(id: 'speed', label: 'Speed', look: ParamLook.scrub, type: WType.f32, def: 240, unit: 'px/s');
const ptLife = [
  Param(id: 'size', label: 'Size', look: ParamLook.scrub, type: WType.f32, min: 0, def: 8, unit: 'px', dec: 1),
  Param(id: 'size_end', label: 'End Size', look: ParamLook.scrub, type: WType.f32, min: 0, def: 2, unit: 'px', dec: 1),
  Param(id: 'opacity_end', label: 'End Opacity', look: ParamLook.percent, type: WType.f32, min: 0, max: 100, unit: '%'),
];
const ptGravity = Param(id: 'gravity', label: 'Gravity', look: ParamLook.bipolar, type: WType.f32, min: -2000, max: 2000, unit: 'px/s²');
const ptWindX = Param(id: 'wind.x', label: 'Wind X', look: ParamLook.bipolar, type: WType.f32, min: -2000, max: 2000, unit: 'px/s²');
const ptWindY = Param(id: 'wind.y', label: 'Wind Y', look: ParamLook.bipolar, type: WType.f32, min: -2000, max: 2000, unit: 'px/s²');
const ptBounce = Param(id: 'bounce', label: 'Bounce', look: ParamLook.percent, type: WType.f32, min: 0, max: 100, unit: '%');
const ptFloor = Param(id: 'floor', label: 'Floor', look: ParamLook.scrub, type: WType.f32, def: 300, unit: 'px');
const ptTurb = Param(id: 'turbulence', label: 'Amount', look: ParamLook.scrub, type: WType.f32, min: 0, unit: 'px');
const ptTurbSize = Param(id: 'turbulence_size', label: 'Scale', look: ParamLook.scrub, type: WType.f32, min: 1, def: 120, unit: 'px');
const ptConnect = Param(id: 'connect', label: 'Distance', look: ParamLook.scrub, type: WType.f32, min: 0, unit: 'px');
const ptLines = [
  Param(id: 'line_width', label: 'Line Width', look: ParamLook.scrub, type: WType.f32, min: 0, max: 100, def: 1, unit: 'px', dec: 1),
  Param(id: 'line_opacity', label: 'Line Opacity', look: ParamLook.percent, type: WType.f32, min: 0, max: 100, def: 60, unit: '%'),
];
const ptRandom = [
  Param(id: 'life_random', label: 'Life Random', look: ParamLook.percent, type: WType.f32, min: 0, max: 100, def: 30, unit: '%'),
  Param(id: 'speed_random', label: 'Speed Random', look: ParamLook.percent, type: WType.f32, min: 0, max: 100, def: 30, unit: '%'),
];
const ptSeed = Param(id: 'seed', label: 'Seed', look: ParamLook.seed, type: WType.f32, min: 0, max: 9999);
const ptAll = [
  ...ptEmit,
  ...ptMotion,
  ...ptLife,
  ptGravity,
  ptWindX,
  ptWindY,
  ptBounce,
  ptFloor,
  ptTurb,
  ptTurbSize,
  ptConnect,
  ...ptLines,
  ...ptRandom,
  ptSeed,
];


/// A group's name inside the card; given [onTap] it is a fold with its state summarised on the right, like Advanced.
class PtHead extends StatelessWidget {
  const PtHead(this.title, {super.key, this.open, this.summary, this.onTap});
  final String title;
  final bool? open;
  final String? summary;
  final VoidCallback? onTap;
  @override
  Widget build(BuildContext context) {
    final fold = onTap != null;
    return Hov(
      cursor: fold ? SystemMouseCursors.click : SystemMouseCursors.basic,
      onTap: onTap,
      builder: (_, h) => Container(
        height: fold ? 24 : 20,
        padding: EdgeInsets.only(top: fold ? 0 : 6),
        decoration: fold
            ? BoxDecoration(
                border: Border(top: BorderSide(color: Grey.g20)),
              )
            : null,
        child: Row(
          children: [
            if (fold) ...[FoldMark(open: open ?? false, hot: h), const SizedBox(width: 6)],
            Text(title, style: T.micro(fold && (h || (open ?? false)) ? Grey.g95 : Grey.g63).copyWith(fontWeight: FontWeight.w700)),
            const Spacer(),
            if (summary != null) Text(summary!, style: T.label(Grey.g56)),
          ],
        ),
      ),
    );
  }
}

/// A colour handle as a block: name on top, the floating palette chooser under it, the swatch in the corner.
class PtColour extends StatelessWidget {
  const PtColour(this.id, this.label, {super.key});
  final String id, label;
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), on = !x.cfg.locked;
    return Container(
      height: Pop.tile,
      decoration: BoxDecoration(color: Pop.well, borderRadius: BorderRadius.circular(6)),
      child: Stack(
        children: [
          Positioned(right: Pop.tilePad - 3, top: 4, width: Pop.glyph, height: Pop.glyph, child: Swatch(x.doc.s2[id])),
          Padding(
            padding: EdgeInsets.fromLTRB(Pop.tilePad, 4, Pop.glyph + Pop.tilePad, 3),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(label, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.micro(on ? Grey.g63 : Grey.g44).copyWith(height: 1.1)),
                LinkLine(to: LinkTo.colors, value: x.doc.s2[id] ?? '—', id: id),
              ],
            ),
          ),
        ],
      ),
    );
  }
}



/// Birth to death on one strip: each dot's size, opacity and colour at that share of its life.
class PtRamp extends StatelessWidget {
  const PtRamp({super.key});
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    return Padding(
      padding: const EdgeInsets.only(top: 2),
      child: SizedBox(
        height: 18,
        child: Row(
          children: [
            Text('Birth', style: T.label(Grey.g44)),
            const SizedBox(width: 6),
            Expanded(
              child: CustomPaint(
                painter: _RampPaint(
                  doc.get('pt_size') ?? 0,
                  doc.get('pt_size_end') ?? 0,
                  doc.get('pt_opacity_end') ?? 0,
                  hexControls(doc.s2['pt_color']) ?? Grey.g95,
                  hexControls(doc.s2['pt_color_end']) ?? Grey.g95,
                ),
              ),
            ),
            const SizedBox(width: 6),
            Text('Death', style: T.label(Grey.g44)),
          ],
        ),
      ),
    );
  }
}

Paint get _ptRing => Paint()
  ..color = Grey.g38
  ..style = PaintingStyle.stroke
  ..strokeWidth = .8;

class _RampPaint extends CustomPainter {
  const _RampPaint(this.s0, this.s1, this.op1, this.c0, this.c1);
  final double s0, s1, op1;
  final Color c0, c1;
  @override
  void paint(Canvas c, Size s) {
    const n = 11;
    final mx = math.max(math.max(s0, s1), 1e-6), y = s.height / 2;
    c.drawLine(Offset(0, y), Offset(s.width, y), Paint()..color = Grey.g20);
    for (var i = 0; i < n; i++) {
      final t = i / (n - 1), x = 6 + (s.width - 12) * t;
      final a = (1 + (op1 / 100 - 1) * t).clamp(0.0, 1.0);
      final col = Color.lerp(c0, c1, t)!;
      c.drawCircle(Offset(x, y), .8 + 5.2 * (s0 + (s1 - s0) * t) / mx, Paint()..color = col.withValues(alpha: a));
      if (a < .15) c.drawCircle(Offset(x, y), 2.5, _ptRing);
    }
  }

  @override
  bool shouldRepaint(_RampPaint o) => o.s0 != s0 || o.s1 != s1 || o.op1 != op1 || o.c0 != c0 || o.c1 != c1;
}
