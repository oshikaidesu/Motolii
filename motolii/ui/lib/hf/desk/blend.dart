import 'dart:ui' as ui;
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import '../bp/common.dart';
import 'common.dart';
import '../neutral.dart';
import '../metrics.dart' show Dn, Surface;

/// A blend mode is shown as what it does to one fixed pair of flat shapes: A (base, yellow) and B (top, blue).
/// Every symbol is the same two shapes through a different operator, so what differs is the overlap, nothing else.
class BlendMode {
  const BlendMode(this.name, this.op, {this.alpha = false, this.group = 0, String? key}) : key = key ?? name;
  final String name;

  /// What the document calls the mode.
  final String key;
  final ui.BlendMode op;
  final bool alpha; // uses the top's alpha shape, not its colour
  final int group;
}

const blendModes = <BlendMode>[
  BlendMode('Normal', ui.BlendMode.srcOver),
  BlendMode('Darken', ui.BlendMode.darken, group: 1),
  BlendMode('Multiply', ui.BlendMode.multiply, group: 1),
  BlendMode('Color Burn', ui.BlendMode.colorBurn, group: 1, key: 'ColorBurn'),
  BlendMode('Lighten', ui.BlendMode.lighten, group: 2),
  BlendMode('Screen', ui.BlendMode.screen, group: 2),
  BlendMode('Color Dodge', ui.BlendMode.colorDodge, group: 2, key: 'ColorDodge'),
  BlendMode('Add', ui.BlendMode.plus, group: 2),
  BlendMode('Overlay', ui.BlendMode.overlay, group: 3),
  BlendMode('Soft Light', ui.BlendMode.softLight, group: 3, key: 'SoftLight'),
  BlendMode('Hard Light', ui.BlendMode.hardLight, group: 3, key: 'HardLight'),
  BlendMode('Difference', ui.BlendMode.difference, group: 4),
  BlendMode('Exclusion', ui.BlendMode.exclusion, group: 4),
  BlendMode('Hue', ui.BlendMode.hue, group: 5),
  BlendMode('Saturation', ui.BlendMode.saturation, group: 5),
  BlendMode('Color', ui.BlendMode.color, group: 5),
  BlendMode('Luminosity', ui.BlendMode.luminosity, group: 5),
  BlendMode('Stencil', ui.BlendMode.dstIn, alpha: true, group: 6, key: 'StencilAlpha'),
  BlendMode('Silhouette', ui.BlendMode.dstOut, alpha: true, group: 6, key: 'SilhouetteAlpha'),
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

/// What a host gives the desk: the layers a mode applies to, the mode they wear, the specimens for a mode, and how the
/// hover and the click reach the document. With no host the desk plays with two fixture layers.
abstract class BlendHost implements Listenable {
  /// The mode every target wears (null: they differ), the targets' names, and whether a click may act now.
  String? get current;
  List<String> get names;
  bool get live;
  List<Color> beds(String mode);
  void aim(String? mode);
  void leave();
  Future<void> apply(String mode);
}

class BlendDesk extends StatefulWidget {
  const BlendDesk({super.key, this.host});
  final BlendHost? host;
  @override
  State<BlendDesk> createState() => _BlendDeskState();
}

class _BlendDeskState extends State<BlendDesk> {
  // Two layers can be targeted; each has its own mode. Targets that disagree are a mixed selection.
  final modes = [2, 5];
  final targeted = <int>{0};
  int? hover;
  bool previewOnStage = true;

  BlendHost? get host => widget.host;

  int get sel {
    final h = host;
    if (h != null) return h.current == null ? -1 : blendModes.indexWhere((m) => m.key == h.current);
    final ms = targeted.map((i) => modes[i]).toSet();
    return ms.length == 1 ? ms.first : -1;
  }

  int? get shown => hover != null && previewOnStage ? hover : (sel < 0 ? null : sel);

  void _pick(int i) {
    final h = host;
    if (h != null) {
      h.apply(blendModes[i].key);
      return;
    }
    setState(() { for (final t in targeted) { modes[t] = i; } });
  }

  @override
  Widget build(BuildContext context) => host == null
      ? _shell()
      : Focus(
          focusNode: _keys,
          onKeyEvent: (_, e) {
            if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape && host!.live) {
              host!.leave();
              return KeyEventResult.handled;
            }
            // the keys walk the modes (each shown on the Stage as the pointer shows it) and Enter applies one
            if (e is KeyDownEvent || e is KeyRepeatEvent) {
              final k = e.logicalKey;
              final step = k == LogicalKeyboardKey.arrowRight || k == LogicalKeyboardKey.arrowDown
                  ? 1
                  : (k == LogicalKeyboardKey.arrowLeft || k == LogicalKeyboardKey.arrowUp ? -1 : 0);
              if (step != 0) {
                final from = hover ?? (sel < 0 ? -1 : sel);
                final i = ((from < 0 ? (step > 0 ? -1 : 0) : from) + step) % blendModes.length;
                setState(() => hover = i < 0 ? blendModes.length - 1 : i);
                if (previewOnStage) host!.aim(blendModes[hover!].key);
                return KeyEventResult.handled;
              }
              if ((k == LogicalKeyboardKey.enter || k == LogicalKeyboardKey.numpadEnter) && hover != null) {
                _pick(hover!);
                return KeyEventResult.handled;
              }
            }
            return KeyEventResult.ignored;
          },
          onFocusChange: (focused) {
            if (!focused) host!.leave();
          },
          child: Listener(
            onPointerDown: (_) => _keys.requestFocus(),
            child: ListenableBuilder(listenable: host!, builder: (_, __) => Opacity(opacity: host!.names.isEmpty ? .45 : 1, child: _shell())),
          ),
        );

  final _keys = FocusNode(debugLabel: 'blend');

  @override
  void dispose() {
    _keys.dispose();
    super.dispose();
  }

  Widget _shell() => DeskShell(
        kind: DeskKind.blend,
        title: 'Blend',
        subtitle: 'LAYER · COMPOSITE',
        full: (c, s) => Padding(
          padding: const EdgeInsets.fromLTRB(9, 10.5, 9, 0),
          child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            // the result and who it applies to stay put; only the list of modes scrolls under them
            _equation(),
            const SizedBox(height: 9),
            _targets(),
            const SizedBox(height: 7.5),
            Expanded(
              child: SingleChildScrollView(
                key: const ValueKey('blend-scroll'),
                padding: const EdgeInsets.only(bottom: 12),
                child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
                  Wrap(spacing: 4.5, runSpacing: 6, children: [for (var i = 0; i < blendModes.length; i++) _tile(i, (s.width - 24 - 18) / 4)]),
                  const SizedBox(height: 12),
                  GestureDetector(
                    key: const ValueKey('blend-stage'),
                    behavior: HitTestBehavior.opaque,
                    onTap: () => setState(() => previewOnStage = !previewOnStage),
                    child: Row(children: [
                      Container(width: 13.5, height: 13.5, decoration: BoxDecoration(color: previewOnStage ? kBlue : null, border: previewOnStage ? null : Border.all(color: N.g38, width: 1), borderRadius: BorderRadius.circular(2)), child: previewOnStage ? CustomPaint(painter: _TickP()) : null),
                      const SizedBox(width: 7.5),
                      Text('Preview on Stage', style: sans(Dn.nameSize, c: N.g76)),
                    ]),
                  ),
                ]),
              ),
            ),
          ]),
        ),
        strip: (c, s) => Padding(
          padding: const EdgeInsets.fromLTRB(7.5, 3, 7.5, 6),
          child: LayoutBuilder(builder: (context, b) => SingleChildScrollView(scrollDirection: Axis.horizontal, child: Row(children: [for (var i = 0; i < blendModes.length; i++) Padding(padding: const EdgeInsets.only(right: 3), child: _mark(i, b.maxHeight * 1.5, b.maxHeight))]))),
        ),
        tall: (c, s) => Padding(
          padding: const EdgeInsets.all(6),
          child: SingleChildScrollView(child: Wrap(spacing: 3, runSpacing: 3, children: [for (var i = 0; i < blendModes.length; i++) _mark(i, s.width - 16, 58)])),
        ),
      );

  // The selected operation is the face: A and B are small inputs, the result is the big shape.
  Widget _equation() {
    final m = shown;
    return Row(crossAxisAlignment: CrossAxisAlignment.center, children: [
      Column(mainAxisSize: MainAxisSize.min, children: [
        Container(width: 30, height: 30, decoration: const BoxDecoration(color: kYellow, shape: BoxShape.circle)),
        const SizedBox(height: 3),
        Text('A', style: sans(Dn.labelSize, c: Surface.muted, w: FontWeight.w600)),
        const SizedBox(height: 7.5),
        Container(width: 30, height: 30, decoration: const BoxDecoration(color: kBlue, shape: BoxShape.circle)),
        const SizedBox(height: 3),
        Text('B', style: sans(Dn.labelSize, c: Surface.muted, w: FontWeight.w600)),
      ]),
      SizedBox(width: 19.5, child: Center(child: Text('→', style: sans(16.5, c: Surface.muted)))),
      Expanded(
        child: Column(children: [
          SizedBox(key: const ValueKey('blend-result'), height: 81, width: double.infinity, child: m == null ? Center(child: Text(host != null && host!.names.isEmpty ? 'Nothing' : 'Mixed', style: sans(13.5, c: Surface.muted))) : CustomPaint(painter: ResultPainter(m))),
          const SizedBox(height: 6),
          // The runtime's own specimen of this mode on the layer: its colour over the beds the runtime draws.
          if (m != null && host != null && host!.beds(blendModes[m].key).isNotEmpty)
            Padding(
              padding: const EdgeInsets.only(bottom: 6),
              child: SizedBox(key: const ValueKey('blend-beds'), height: 7.5, child: Row(children: [for (final c in host!.beds(blendModes[m].key)) Expanded(child: ColoredBox(color: c))])),
            ),
          Text(m == null ? (host != null && host!.names.isEmpty ? 'No layer' : 'Targets differ') : blendModes[m].name, style: sans(13, c: Surface.ink, w: FontWeight.w600)),
        ]),
      ),
    ]);
  }

  Widget _targets() => Row(children: [
        Text('TARGETS', style: sans(Dn.microSize, c: Surface.muted, w: FontWeight.w500, ls: 1.4)),
        const SizedBox(width: 9),
        // With a host the selection decides the targets; the chips only say who they are.
        if (host != null) ...[
          for (final n in host!.names.take(2)) Padding(padding: const EdgeInsets.only(right: 4.5), child: Container(height: 19.5, padding: const EdgeInsets.symmetric(horizontal: 7.5), alignment: Alignment.center, decoration: BoxDecoration(color: kYellow, borderRadius: BorderRadius.circular(10)), child: Text(n, softWrap: false, overflow: TextOverflow.clip, style: sans(Dn.nameSize, c: N.g10, w: FontWeight.w600)))),
          if (host!.names.length > 2) Text('+${host!.names.length - 2}', style: sans(Dn.nameSize, c: Surface.muted)),
        ] else
        for (final i in [0, 1]) Padding(
          padding: const EdgeInsets.only(right: 4.5),
          child: GestureDetector(
            key: ValueKey('blend-target-$i'),
            onTap: () => setState(() { if (targeted.contains(i)) { if (targeted.length > 1) targeted.remove(i); } else { targeted.add(i); } }),
            child: Container(height: 19.5, padding: const EdgeInsets.symmetric(horizontal: 7.5), alignment: Alignment.center, decoration: BoxDecoration(color: targeted.contains(i) ? kYellow : null, border: targeted.contains(i) ? null : Border.all(color: N.g26), borderRadius: BorderRadius.circular(10)), child: Text('Layer ${i + 2}', style: sans(Dn.nameSize, c: targeted.contains(i) ? N.g10 : N.g76, w: FontWeight.w600))),
          ),
        ),
      ]);

  Widget _mark(int i, double w, double h) => MouseRegion(
        onEnter: (_) {
          setState(() => hover = i);
          if (previewOnStage) host?.aim(blendModes[i].key);
        },
        onExit: (_) {
          setState(() { if (hover == i) hover = null; });
          if (hover == null) host?.aim(null);
        },
        child: GestureDetector(
          key: ValueKey('blend-mark-$i'),
          onTap: () => _pick(i),
          child: Container(
            width: w < 0 ? 0 : w,
            height: h < 0 ? 0 : h,
            decoration: BoxDecoration(color: i == sel ? Surface.hover : null, border: i == sel ? Border.all(color: kYellow, width: 1.5) : null, borderRadius: BorderRadius.circular(4.5)),
            padding: const EdgeInsets.all(3),
            child: CustomPaint(painter: ResultPainter(i)),
          ),
        ),
      );

  Widget _tile(int i, double w) => SizedBox(
        width: w,
        child: Column(mainAxisSize: MainAxisSize.min, children: [
          _mark(i, w, 40),
          const SizedBox(height: 3),
          Text(blendModes[i].name, softWrap: false, overflow: TextOverflow.clip, textAlign: TextAlign.center, style: sans(Dn.microSize, c: i == sel ? Surface.ink : N.g63, w: i == sel ? FontWeight.w600 : FontWeight.w400)),
        ]),
      );
}

class _TickP extends CustomPainter {
  @override
  void paint(Canvas c, Size s) => c.drawPath(Path()..moveTo(4.5, 9.5)..lineTo(7.8, 12.8)..lineTo(13.5, 5.5), Paint()..color = Surface.ink..style = PaintingStyle.stroke..strokeWidth = 2..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round);
  @override
  bool shouldRepaint(_TickP o) => false;
}
