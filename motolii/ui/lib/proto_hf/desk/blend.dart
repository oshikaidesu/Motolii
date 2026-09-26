import 'dart:ui' as ui;
import 'package:flutter/widgets.dart';
import '../bp/common.dart';
import 'common.dart';

/// A blend mode is shown as what it does to one fixed pair of flat shapes: A (base, yellow) and B (top, blue).
/// Every symbol is the same two shapes through a different operator, so what differs is the overlap, nothing else.
class BlendMode {
  const BlendMode(this.name, this.op, {this.alpha = false, this.group = 0});
  final String name;
  final ui.BlendMode op;
  final bool alpha; // uses the top's alpha shape, not its colour
  final int group;
}

const blendModes = <BlendMode>[
  BlendMode('Normal', ui.BlendMode.srcOver),
  BlendMode('Darken', ui.BlendMode.darken, group: 1),
  BlendMode('Multiply', ui.BlendMode.multiply, group: 1),
  BlendMode('Color Burn', ui.BlendMode.colorBurn, group: 1),
  BlendMode('Lighten', ui.BlendMode.lighten, group: 2),
  BlendMode('Screen', ui.BlendMode.screen, group: 2),
  BlendMode('Color Dodge', ui.BlendMode.colorDodge, group: 2),
  BlendMode('Add', ui.BlendMode.plus, group: 2),
  BlendMode('Overlay', ui.BlendMode.overlay, group: 3),
  BlendMode('Soft Light', ui.BlendMode.softLight, group: 3),
  BlendMode('Hard Light', ui.BlendMode.hardLight, group: 3),
  BlendMode('Difference', ui.BlendMode.difference, group: 4),
  BlendMode('Exclusion', ui.BlendMode.exclusion, group: 4),
  BlendMode('Hue', ui.BlendMode.hue, group: 5),
  BlendMode('Saturation', ui.BlendMode.saturation, group: 5),
  BlendMode('Color', ui.BlendMode.color, group: 5),
  BlendMode('Luminosity', ui.BlendMode.luminosity, group: 5),
  BlendMode('Stencil', ui.BlendMode.dstIn, alpha: true, group: 6),
  BlendMode('Silhouette', ui.BlendMode.dstOut, alpha: true, group: 6),
];

const _a = kYellow;
const _b = kBlue;

/// Two flat discs overlapping. Only the overlap depends on the operator.
void paintResult(Canvas c, Rect r, int m, {double opacity = 1}) {
  final mode = blendModes[m];
  final rad = r.height * .40;
  final dist = rad * 1.05;
  final cy = r.center.dy;
  final ca = Offset(r.center.dx - dist / 2, cy), cb = Offset(r.center.dx + dist / 2, cy);
  c.saveLayer(r.inflate(4), Paint());
  c.drawCircle(ca, rad, Paint()..color = _a);
  c.saveLayer(r.inflate(4), Paint()..blendMode = mode.op);
  c.drawCircle(cb, rad, Paint()..color = _b.withValues(alpha: opacity));
  c.restore();
  c.restore();
}

class ResultPainter extends CustomPainter {
  ResultPainter(this.m, {this.opacity = 1});
  final int m;
  final double opacity;
  @override
  void paint(Canvas c, Size s) => paintResult(c, Offset.zero & s, m, opacity: opacity);
  @override
  bool shouldRepaint(ResultPainter o) => o.m != m || o.opacity != opacity;
}

class BlendDesk extends StatefulWidget {
  const BlendDesk({super.key});
  @override
  State<BlendDesk> createState() => _BlendDeskState();
}

class _BlendDeskState extends State<BlendDesk> {
  // Two layers can be targeted; each has its own mode. Targets that disagree are a mixed selection.
  final modes = [2, 5];
  final targeted = <int>{0};
  int? hover;
  bool previewOnStage = true;

  int get sel {
    final ms = targeted.map((i) => modes[i]).toSet();
    return ms.length == 1 ? ms.first : -1;
  }

  int? get shown => hover != null && previewOnStage ? hover : (sel < 0 ? null : sel);

  void _pick(int i) => setState(() { for (final t in targeted) { modes[t] = i; } });

  @override
  Widget build(BuildContext context) => DeskShell(
        kind: DeskKind.blend,
        title: 'Blend',
        subtitle: 'LAYER · COMPOSITE',
        full: (c, s) => Padding(
          padding: const EdgeInsets.fromLTRB(12, 14, 12, 0),
          child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            // the result and who it applies to stay put; only the list of modes scrolls under them
            _equation(),
            const SizedBox(height: 12),
            _targets(),
            const SizedBox(height: 10),
            Expanded(
              child: SingleChildScrollView(
                key: const ValueKey('blend-scroll'),
                padding: const EdgeInsets.only(bottom: 16),
                child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
                  Wrap(spacing: 6, runSpacing: 8, children: [for (var i = 0; i < blendModes.length; i++) _tile(i, (s.width - 24 - 18) / 4)]),
                  const SizedBox(height: 16),
                  GestureDetector(
                    key: const ValueKey('blend-stage'),
                    behavior: HitTestBehavior.opaque,
                    onTap: () => setState(() => previewOnStage = !previewOnStage),
                    child: Row(children: [
                      Container(width: 18, height: 18, decoration: BoxDecoration(color: previewOnStage ? kBlue : null, border: previewOnStage ? null : Border.all(color: const Color(0xFF5A5B62), width: 1.6), borderRadius: BorderRadius.circular(3)), child: previewOnStage ? CustomPaint(painter: _TickP()) : null),
                      const SizedBox(width: 10),
                      Text('Preview on Stage', style: sans(11.5, c: const Color(0xFFC4C6CB))),
                    ]),
                  ),
                ]),
              ),
            ),
          ]),
        ),
        strip: (c, s) => Padding(
          padding: const EdgeInsets.fromLTRB(10, 4, 10, 8),
          child: LayoutBuilder(builder: (context, b) => SingleChildScrollView(scrollDirection: Axis.horizontal, child: Row(children: [for (var i = 0; i < blendModes.length; i++) Padding(padding: const EdgeInsets.only(right: 4), child: _mark(i, b.maxHeight * 1.5, b.maxHeight))]))),
        ),
        tall: (c, s) => Padding(
          padding: const EdgeInsets.all(8),
          child: SingleChildScrollView(child: Wrap(spacing: 4, runSpacing: 4, children: [for (var i = 0; i < blendModes.length; i++) _mark(i, s.width - 16, 58)])),
        ),
      );

  // The selected operation is the face: A and B are small inputs, the result is the big shape.
  Widget _equation() {
    final m = shown;
    return Row(crossAxisAlignment: CrossAxisAlignment.center, children: [
      Column(mainAxisSize: MainAxisSize.min, children: [
        Container(width: 40, height: 40, decoration: const BoxDecoration(color: kYellow, shape: BoxShape.circle)),
        const SizedBox(height: 4),
        Text('A', style: sans(10.5, c: kMuted, w: FontWeight.w600)),
        const SizedBox(height: 10),
        Container(width: 40, height: 40, decoration: const BoxDecoration(color: kBlue, shape: BoxShape.circle)),
        const SizedBox(height: 4),
        Text('B', style: sans(10.5, c: kMuted, w: FontWeight.w600)),
      ]),
      SizedBox(width: 26, child: Center(child: Text('→', style: sans(22, c: kMuted)))),
      Expanded(
        child: Column(children: [
          SizedBox(key: const ValueKey('blend-result'), height: 108, width: double.infinity, child: m == null ? Center(child: Text('Mixed', style: sans(18, c: kMuted))) : CustomPaint(painter: ResultPainter(m))),
          const SizedBox(height: 8),
          Text(m == null ? 'Targets differ' : blendModes[m].name, style: sans(17, c: kInk, w: FontWeight.w600)),
        ]),
      ),
    ]);
  }

  Widget _targets() => Row(children: [
        Text('TARGETS', style: sans(9.5, c: kMuted, w: FontWeight.w500, ls: 1.4)),
        const SizedBox(width: 12),
        for (final i in [0, 1]) Padding(
          padding: const EdgeInsets.only(right: 6),
          child: GestureDetector(
            key: ValueKey('blend-target-$i'),
            onTap: () => setState(() { if (targeted.contains(i)) { if (targeted.length > 1) targeted.remove(i); } else { targeted.add(i); } }),
            child: Container(height: 26, padding: const EdgeInsets.symmetric(horizontal: 10), alignment: Alignment.center, decoration: BoxDecoration(color: targeted.contains(i) ? kYellow : null, border: targeted.contains(i) ? null : Border.all(color: const Color(0xFF3A3B40)), borderRadius: BorderRadius.circular(13)), child: Text('Layer ${i + 2}', style: sans(11, c: targeted.contains(i) ? const Color(0xFF1B1B1D) : const Color(0xFFC4C6CB), w: FontWeight.w600))),
          ),
        ),
      ]);

  Widget _mark(int i, double w, double h) => MouseRegion(
        onEnter: (_) => setState(() => hover = i),
        onExit: (_) => setState(() { if (hover == i) hover = null; }),
        child: GestureDetector(
          key: ValueKey('blend-mark-$i'),
          onTap: () => _pick(i),
          child: Container(
            width: w,
            height: h,
            decoration: BoxDecoration(color: i == sel ? kRaisedHi : null, border: i == sel ? Border.all(color: kYellow, width: 2) : null, borderRadius: BorderRadius.circular(6)),
            padding: const EdgeInsets.all(4),
            child: CustomPaint(painter: ResultPainter(i)),
          ),
        ),
      );

  Widget _tile(int i, double w) => SizedBox(
        width: w,
        child: Column(mainAxisSize: MainAxisSize.min, children: [
          _mark(i, w, 40),
          const SizedBox(height: 4),
          Text(blendModes[i].name, softWrap: false, overflow: TextOverflow.clip, textAlign: TextAlign.center, style: sans(9.5, c: i == sel ? kInk : const Color(0xFF9EA0A6), w: i == sel ? FontWeight.w600 : FontWeight.w400)),
        ]),
      );
}

class _TickP extends CustomPainter {
  @override
  void paint(Canvas c, Size s) => c.drawPath(Path()..moveTo(4.5, 9.5)..lineTo(7.8, 12.8)..lineTo(13.5, 5.5), Paint()..color = kInk..style = PaintingStyle.stroke..strokeWidth = 2..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round);
  @override
  bool shouldRepaint(_TickP o) => false;
}
