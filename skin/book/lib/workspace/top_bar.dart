// The window's top bar: who and what (wordmark, project, comp), the mode, and the one loud action (Render).
import 'package:flutter/widgets.dart';

import '../tokens.dart';
import 'ws.dart';

class WsTopBar extends StatefulWidget {
  const WsTopBar({super.key});
  @override
  State<WsTopBar> createState() => _WsTopBarState();
}

class _WsTopBarState extends State<WsTopBar> {
  int _mode = 0;

  @override
  Widget build(BuildContext context) {
    final ws = WsScope.of(context);
    return Container(
      color: WsT.body,
      padding: const EdgeInsets.symmetric(horizontal: 10),
      child: Row(
        children: [
          const _Wordmark(),
          const SizedBox(width: 14),
          const _UndoRedo(),
          const SizedBox(width: 14),
          Text('CODA Teaser', style: T.name(Grey.g95).copyWith(fontWeight: FontWeight.w600)),
          const SizedBox(width: 6),
          Text('/', style: T.name(Grey.g44)),
          const SizedBox(width: 6),
          Text('Main comp', style: T.name(Grey.g76)),
          const SizedBox(width: 10),
          const _Chip('1920 × 1080'),
          const SizedBox(width: 4),
          const _Chip('${Ws.fps} fps'),
          const SizedBox(width: 4),
          _Chip(ws.timecode(Ws.duration)),
          const Spacer(),
          _Modes(index: _mode, onPick: (i) => setState(() => _mode = i)),
          const Spacer(),
          const _ShadeSwitch(),
          const SizedBox(width: 10),
          const _Saved(),
          const SizedBox(width: 12),
          const _Render(),
        ],
      ),
    );
  }
}

/// A square lime block with the wordmark in it: the poster's own manner, used once.
class _Wordmark extends StatelessWidget {
  const _Wordmark();
  @override
  Widget build(BuildContext context) => Row(
    mainAxisSize: MainAxisSize.min,
    children: [
      Container(
        width: 22,
        height: 22,
        alignment: Alignment.center,
        color: WsT.accent,
        child: Text('M', style: T.title(WsT.onAccent).copyWith(fontSize: 14, fontWeight: FontWeight.w900, height: 1)),
      ),
      const SizedBox(width: 8),
      Text('Motolii', style: T.title(Grey.g95).copyWith(fontSize: 14, fontWeight: FontWeight.w800, letterSpacing: -.2)),
    ],
  );
}

class _Chip extends StatelessWidget {
  const _Chip(this.text);
  final String text;
  @override
  Widget build(BuildContext context) => Container(
    height: 20,
    padding: const EdgeInsets.symmetric(horizontal: 7),
    alignment: Alignment.center,
    decoration: BoxDecoration(color: WsT.well, borderRadius: BorderRadius.circular(WsT.chipRadius)),
    child: Text(text, style: T.value(Grey.g76).copyWith(fontSize: 10)),
  );
}

class _Hover extends StatefulWidget {
  const _Hover({super.key, required this.builder, this.onTap});
  final Widget Function(bool hot) builder;
  final VoidCallback? onTap;
  @override
  State<_Hover> createState() => _HoverState();
}

class _HoverState extends State<_Hover> {
  bool _h = false;
  @override
  Widget build(BuildContext context) => MouseRegion(
    cursor: SystemMouseCursors.click,
    onEnter: (_) => setState(() => _h = true),
    onExit: (_) => setState(() => _h = false),
    child: GestureDetector(behavior: HitTestBehavior.opaque, onTap: widget.onTap, child: widget.builder(_h)),
  );
}

/// Edit / Animate / Export: the mode is the one segmented control in the bar, lit in the accent.
class _Modes extends StatelessWidget {
  const _Modes({required this.index, required this.onPick});
  final int index;
  final ValueChanged<int> onPick;
  static const items = ['Edit', 'Animate', 'Export'];
  @override
  Widget build(BuildContext context) => Container(
    height: 26,
    padding: const EdgeInsets.all(2),
    decoration: BoxDecoration(color: WsT.well, borderRadius: BorderRadius.circular(WsT.radius)),
    child: Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        for (final (i, s) in items.indexed)
          _Hover(
            onTap: () => onPick(i),
            builder: (h) => Container(
              width: 76,
              alignment: Alignment.center,
              decoration: BoxDecoration(color: i == index ? WsT.accent : (h ? WsT.raised : null), borderRadius: BorderRadius.circular(WsT.radius - 2)),
              child: Text(
                s,
                style: T.name(i == index ? WsT.onAccent : (h ? Grey.g95 : Grey.g76)).copyWith(fontWeight: i == index ? FontWeight.w700 : FontWeight.w500),
              ),
            ),
          ),
      ],
    ),
  );
}

class _UndoRedo extends StatelessWidget {
  const _UndoRedo();
  @override
  Widget build(BuildContext context) => Row(
    mainAxisSize: MainAxisSize.min,
    children: [
      for (final flip in [false, true])
        _Hover(
          builder: (h) => Container(
            width: 24,
            height: 24,
            alignment: Alignment.center,
            decoration: BoxDecoration(color: h ? WsT.raised : null, borderRadius: BorderRadius.circular(WsT.radius)),
            child: Transform.flip(
              flipX: flip,
              child: CustomPaint(size: const Size(12, 10), painter: _UndoPaint(h ? Grey.g95 : Grey.g63)),
            ),
          ),
        ),
    ],
  );
}

class _UndoPaint extends CustomPainter {
  const _UndoPaint(this.c);
  final Color c;
  @override
  void paint(Canvas canvas, Size s) {
    final p = Paint()
      ..color = c
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.4
      ..strokeCap = StrokeCap.round
      ..strokeJoin = StrokeJoin.round;
    canvas
      ..drawPath(
        Path()
          ..moveTo(1, 4)
          ..lineTo(8, 4)
          ..arcToPoint(Offset(8, s.height - 1), radius: const Radius.circular(3.2))
          ..lineTo(4, s.height - 1),
        p,
      )
      ..drawPath(
        Path()
          ..moveTo(4, 1)
          ..lineTo(1, 4)
          ..lineTo(4, 7),
        p,
      );
  }

  @override
  bool shouldRepaint(_UndoPaint o) => o.c != c;
}

/// Dark / light: flips [Grey.shade] and rebuilds the whole window in place (reassemble keeps every State, so the edit survives).
class _ShadeSwitch extends StatelessWidget {
  const _ShadeSwitch();
  static void flip() {
    Grey.shade.value = Grey.light ? Shade.dark : Shade.light;
    WidgetsBinding.instance.reassembleApplication();
  }

  @override
  Widget build(BuildContext context) => _Hover(
    key: const ValueKey('ws-shade'),
    onTap: flip,
    builder: (h) => Container(
      width: 24,
      height: 24,
      alignment: Alignment.center,
      decoration: BoxDecoration(color: h ? WsT.raised : null, borderRadius: BorderRadius.circular(WsT.radius)),
      child: CustomPaint(size: const Size(12, 12), painter: _ShadePaint(h ? Grey.g95 : Grey.g63)),
    ),
  );
}

/// A ring with its left half filled: the usual sign for "light or dark".
class _ShadePaint extends CustomPainter {
  const _ShadePaint(this.c);
  final Color c;
  @override
  void paint(Canvas canvas, Size s) {
    final r = Rect.fromLTWH(.7, .7, s.width - 1.4, s.height - 1.4);
    canvas
      ..drawOval(
        r,
        Paint()
          ..color = c
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1.4,
      )
      ..drawArc(r, 1.5708, 3.14159, true, Paint()..color = c);
  }

  @override
  bool shouldRepaint(_ShadePaint o) => o.c != c;
}

class _Saved extends StatelessWidget {
  const _Saved();
  @override
  Widget build(BuildContext context) => Row(
    mainAxisSize: MainAxisSize.min,
    children: [
      Container(
        width: 6,
        height: 6,
        decoration: const BoxDecoration(color: WsT.accent, shape: BoxShape.circle),
      ),
      const SizedBox(width: 6),
      Text('Saved', style: T.label(Grey.g63)),
    ],
  );
}

/// The one loud action: a flat lime pill with dark ink.
class _Render extends StatelessWidget {
  const _Render();
  @override
  Widget build(BuildContext context) => _Hover(
    builder: (h) => Container(
      height: 26,
      padding: const EdgeInsets.symmetric(horizontal: 14),
      alignment: Alignment.center,
      decoration: BoxDecoration(color: h ? WsT.accentHot : WsT.accent, borderRadius: BorderRadius.circular(13)),
      child: Text('Render', style: T.name(WsT.onAccent).copyWith(fontWeight: FontWeight.w800)),
    ),
  );
}
