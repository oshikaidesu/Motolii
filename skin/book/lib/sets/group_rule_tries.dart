// Tries of how a group's one rule is touched, on the real window (the stage the inside draws, the timeline to seek):
// each a pane beside the inspector that changes the document's own relations. Borrowed forms (research 2026-10-22):
// one example's before and after (Viscuit's glasses, PhET's arrow tip), two states (PowerPoint's Morph, Live2D).
//   flutter run -d macos --dart-define=LAB_ROUTE='/?path=relation/group-rule-tries/1-before-and-after' --dart-define=MOTOLII_DOC=<stage.motolii>
import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import '../motolii.dart';
import '../tokens.dart';
import '../workspace/dock_glyphs.dart';
import '../workspace/panes.dart';
import '../workspace/workspace.dart';
import '../workspace/ws.dart';

const _doc = String.fromEnvironment('MOTOLII_DOC', defaultValue: '$motoliiDir/out/skin.motolii');

/// The document's first relation of a kind (its number), or none.
int? _relation(Motolii m, String kind) => m.things.where((t) => m.word(t, 'kind') == 'behaviour' && m.word(t, 'file') == kind).firstOrNull;

void _set(Ws ws, int thing, String name, double v) => ws.after(ws.inside!.later('set', thing: thing, name: name, number: v), (_, _) => ws.reread());

/// A number pulled sideways, as the inspector's.
class _Knob extends StatelessWidget {
  const _Knob(this.label, this.text, this.pull);
  final String label, text;
  final ValueChanged<double> pull;
  @override
  Widget build(BuildContext context) => GestureDetector(
    onHorizontalDragUpdate: (d) => pull(d.delta.dx),
    child: Container(
      margin: const EdgeInsets.only(bottom: 6),
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
      decoration: BoxDecoration(color: Grey.g20, borderRadius: BorderRadius.circular(6)),
      child: Row(children: [Expanded(child: Text(label, style: T.label(Grey.g63))), Text(text, style: T.value(Grey.g95))]),
    ),
  );
}

/// 1. Before and after: the group's first thing at home and, as a ghost, where every thing goes; pull the ghost to
/// move them all, its corner to turn and size them; each goes in turn, lag apart.
class _BeforeAfter extends StatelessWidget {
  const _BeforeAfter();
  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: WsScope.read(context),
    builder: (context, _) {
      final ws = WsScope.read(context), m = ws.inside, g = m == null ? null : _relation(m, 'group');
      if (m == null || g == null) return Center(child: Text('No group in this document', style: T.label(Grey.g56)));
      double v(String n, double or) => m.value(g, n, 0) ?? or;
      final move = Offset(v('x', 0), v('y', 0)), turn = v('rotate', 0), size = v('size', 1), opacity = v('opacity', 1), lag = v('lag', 0);
      return Padding(
        padding: const EdgeInsets.all(10),
        child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          Expanded(   // (as tall as the pane leaves it)
            child: LayoutBuilder(builder: (context, box) {
              final s = box.biggest, k = s.width * .4, home = Offset(s.width / 2, s.height / 2), ghost = home + Offset(move.dx * k, -move.dy * k);
              final corner = ghost + Offset.fromDirection(-turn * math.pi / 180 - math.pi / 4, 26 * size);
              return Stack(children: [
                Positioned.fill(child: CustomPaint(painter: _Glasses(home, ghost, turn, size, opacity))),
                Positioned(
                  left: ghost.dx - 24,
                  top: ghost.dy - 20,
                  child: GestureDetector(
                    onPanUpdate: (d) {
                      _set(ws, g, 'x', move.dx + d.delta.dx / k);
                      _set(ws, g, 'y', move.dy - d.delta.dy / k);
                    },
                    child: const SizedBox(width: 48, height: 40),
                  ),
                ),
                Positioned(
                  left: corner.dx - 7,
                  top: corner.dy - 7,
                  child: GestureDetector(
                    onPanUpdate: (d) {
                      final box = context.findRenderObject() as RenderBox, p = box.globalToLocal(d.globalPosition) - ghost;
                      _set(ws, g, 'rotate', -(p.direction + math.pi / 4) * 180 / math.pi);
                      _set(ws, g, 'size', (p.distance / 26).clamp(.05, 3.0));
                    },
                    child: Container(width: 14, height: 14, decoration: BoxDecoration(color: const Color(0xFFFFFFFF), shape: BoxShape.circle, border: Border.all(color: WsT.toneGroup, width: 2))),
                  ),
                ),
              ]);
            }),
          ),
          const SizedBox(height: 8),
          _Knob('Opacity away', '${(opacity * 100).round()}%', (dx) => _set(ws, g, 'opacity', (opacity + dx * .01).clamp(0, 1))),
          _Knob('Lag', '${lag.toStringAsFixed(2)} s', (dx) => _set(ws, g, 'lag', (lag + dx * .002).clamp(0, 1))),
        ]),
      );
    },
  );
}

/// Viscuit's glasses: before on the left of the line, after on the right; here home, an arrow, and the ghost.
class _Glasses extends CustomPainter {
  const _Glasses(this.home, this.ghost, this.turn, this.size, this.opacity);
  final Offset home, ghost;
  final double turn, size, opacity;
  @override
  void paint(Canvas c, Size s) {
    c.drawRect(Offset.zero & s, Paint()..color = const Color(0xFFF1ECE2));
    final card = Rect.fromCenter(center: Offset.zero, width: 44, height: 32), ink = Paint()..color = WsT.toneGroup..style = PaintingStyle.stroke..strokeWidth = 1.5;
    c.drawRRect(RRect.fromRectAndRadius(card.shift(home), const Radius.circular(4)), Paint()..color = wsLabel('2'));
    c.drawLine(home, ghost, ink);
    c.save();
    c.translate(ghost.dx, ghost.dy);
    c.rotate(turn * math.pi / 180);
    c.scale(size);
    c.drawRRect(RRect.fromRectAndRadius(card, const Radius.circular(4)), Paint()..color = wsLabel('2').withValues(alpha: .2 + .6 * opacity));
    c.drawRRect(RRect.fromRectAndRadius(card, const Radius.circular(4)), ink);
    c.restore();
  }

  @override
  bool shouldRepaint(_Glasses o) => true;
}

/// 2. Two states: A where they were put, B the ring round the cube; one slider between them (the ring's strength),
/// its lag apart.
class _TwoStates extends StatelessWidget {
  const _TwoStates();
  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: WsScope.read(context),
    builder: (context, _) {
      final ws = WsScope.read(context), m = ws.inside, b = m == null ? null : _relation(m, 'around');
      if (m == null || b == null) return Center(child: Text('No ring in this document', style: T.label(Grey.g56)));
      final s = m.value(b, 'strength', ws.frame / Ws.fps) ?? 0, lag = m.value(b, 'lag', 0) ?? 0;
      return Padding(
        padding: const EdgeInsets.all(10),
        child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          Row(children: [
            Expanded(child: _State('A', false, s < .5, () => _set(ws, b, 'strength', 0))),
            const SizedBox(width: 6),
            Expanded(child: _State('B', true, s >= .5, () => _set(ws, b, 'strength', 1))),
          ]),
          const SizedBox(height: 10),
          LayoutBuilder(builder: (context, box) => GestureDetector(
            onHorizontalDragUpdate: (d) => _set(ws, b, 'strength', (d.localPosition.dx / box.maxWidth).clamp(0, 1)),
            onTapDown: (d) => _set(ws, b, 'strength', (d.localPosition.dx / box.maxWidth).clamp(0, 1)),
            child: CustomPaint(size: Size(box.maxWidth, 26), painter: _Slider(s.clamp(0, 1))),
          )),
          const SizedBox(height: 10),
          _Knob('Lag', '${lag.toStringAsFixed(2)} s', (dx) => _set(ws, b, 'lag', (lag + dx * .002).clamp(0, 1))),
        ]),
      );
    },
  );
}

class _State extends StatelessWidget {
  const _State(this.name, this.ring, this.on, this.tap);
  final String name;
  final bool ring, on;
  final VoidCallback tap;
  @override
  Widget build(BuildContext context) => GestureDetector(
    onTap: tap,
    child: Container(
      height: 86,
      decoration: BoxDecoration(color: const Color(0xFFF1ECE2), borderRadius: BorderRadius.circular(6), border: Border.all(color: on ? WsT.toneGroup : Grey.g38, width: on ? 2 : 1)),
      child: Stack(children: [
        Positioned.fill(child: CustomPaint(painter: _Thumb(ring))),
        Positioned(left: 6, top: 4, child: Text(name, style: T.label(Grey.g07))),
      ]),
    ),
  );
}

class _Thumb extends CustomPainter {
  const _Thumb(this.ring);
  final bool ring;
  @override
  void paint(Canvas c, Size s) {
    final mid = s.center(Offset.zero);
    c.drawRect(Rect.fromCenter(center: mid, width: 14, height: 14), Paint()..color = const Color(0xFFE5432D));
    for (var i = 0; i < 8; i++) {
      final a = i / 8 * 2 * math.pi;
      final p = ring ? mid + Offset(math.sin(a) * s.width * .32, math.cos(a) * s.height * .14) : mid + Offset(math.sin(a) * s.width * .38, math.cos(a) * s.height * .3);
      c.drawRect(Rect.fromCenter(center: p, width: 9, height: 9), Paint()..color = wsLabel('${i + 2}'));
    }
  }

  @override
  bool shouldRepaint(_Thumb o) => false;
}

class _Slider extends CustomPainter {
  const _Slider(this.u);
  final double u;
  @override
  void paint(Canvas c, Size s) {
    c.drawRRect(RRect.fromLTRBR(0, s.height / 2 - 2, s.width, s.height / 2 + 2, const Radius.circular(2)), Paint()..color = Grey.g26);
    c.drawRRect(RRect.fromLTRBR(0, s.height / 2 - 2, u * s.width, s.height / 2 + 2, const Radius.circular(2)), Paint()..color = WsT.toneGroup);
    c.drawCircle(Offset(u * s.width, s.height / 2), 7, Paint()..color = const Color(0xFFFFFFFF));
  }

  @override
  bool shouldRepaint(_Slider o) => o.u != u;
}

/// The tries as panes beside the inspector, for the window alone (--dart-define=MOTOLII_TRIES=true with MOTOLII_BARE).
List<WsPane> groupRuleTryPanes() => [WsPane('Before & after', DockG.desk, () => const _BeforeAfter()), WsPane('Two states', DockG.desk, () => const _TwoStates())];

WidgetbookComponent groupRuleTriesSet() => WidgetbookComponent(
  name: 'Group rule tries',
  useCases: [
    WidgetbookUseCase(name: '1 Before and after', builder: (c) => Workspace(document: _doc, extra: [WsPane('Before & after', DockG.desk, () => const _BeforeAfter())])),
    WidgetbookUseCase(name: '2 Two states', builder: (c) => Workspace(document: _doc, extra: [WsPane('Two states', DockG.desk, () => const _TwoStates())])),
  ],
);
