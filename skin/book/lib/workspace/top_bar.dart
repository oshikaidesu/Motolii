// The window's top bar: who and what (wordmark, project, comp), the mode, and the one loud action (Render).
import 'dart:async';
import 'dart:io';

import 'package:flutter/widgets.dart';
import '../parts/skin_parts.dart';
import 'zoom.dart';

import '../tokens.dart';
import 'stage_art.dart' show WsArt;
import 'ws.dart';

class WsTopBar extends StatelessWidget {
  const WsTopBar({super.key});

  @override
  Widget build(BuildContext context) {
    final ws = WsScope.of(context);
    return Container(
      color: WsT.body,
      padding: EdgeInsets.symmetric(horizontal: Shape.s(10)),
      child: Row(
        children: [
          const _Wordmark(),
          const SizedBox(width: 14),
          const _UndoRedo(),
          const SizedBox(width: 14),
          Text(ws.document ?? '', style: T.name(Grey.g95).copyWith(fontWeight: FontWeight.w600)),
          const SizedBox(width: 6),
          Text('/', style: T.name(Grey.g44)),
          const SizedBox(width: 6),
          Text(ws.layer?.name ?? 'Stage', style: T.name(Grey.g76)), // (where the hand is: the picked thing, or the stage itself)
          const SizedBox(width: 10),
          // Joined: the document's own size; a tap goes to the next of the usual three (wide, tall, square) and the document is opened again at it.
          GestureDetector(
            key: const ValueKey('ws-top-size'),
            onTap: ws.onSize == null
                ? null
                : () {
                    const usual = [(1920, 1080), (1080, 1920), (1080, 1080)];
                    final now = usual.indexOf((WsArt.comp.width.round(), WsArt.comp.height.round())), to = usual[(now + 1) % usual.length];
                    ws.onSize!(to.$1, to.$2);
                  },
            child: _Chip('${WsArt.comp.width.round()} × ${WsArt.comp.height.round()}'),
          ),
          const SizedBox(width: 4),
          _Chip('${Ws.fps} fps'),
          const SizedBox(width: 4),
          _Chip(ws.timecode(Ws.duration)),
          const Spacer(),
          // (Edit and Animate are whether every change is keyed, the inside's one truth; Export writes the work area as a film, lit while it is written)
          _Modes(
            index: (ws.notice ?? '').startsWith('Writing') ? 2 : (ws.animating ? 1 : 0),
            onPick: (i) => i == 2 ? ws.onRender?.call() : ws.animating = i == 1,
          ),
          const Spacer(),
          const WsZoomReadout(),
          const _ShadeSwitch(),
          const SizedBox(width: 10),
          const _Waited(),
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
        child: Text('M', style: T.title(WsT.onAccent).copyWith(fontSize: T.px(14), fontWeight: FontWeight.w900, height: 1)),
      ),
      const SizedBox(width: 8),
      Text('Motolii', style: T.title(Grey.g95).copyWith(fontSize: T.px(14), fontWeight: FontWeight.w800, letterSpacing: -.2)),
    ],
  );
}

class _Chip extends StatelessWidget {
  const _Chip(this.text);
  final String text;
  @override
  Widget build(BuildContext context) => Container(
    height: 20,
    padding: EdgeInsets.symmetric(horizontal: Shape.s(7)),
    alignment: Alignment.center,
    decoration: BoxDecoration(color: WsT.well, borderRadius: BorderRadius.all(Shape.r(WsT.chipRadius))),
    child: Text(text, style: T.value(Grey.g76).copyWith(fontSize: T.px(10))),
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
    padding: EdgeInsets.all(Shape.s(2)),
    decoration: BoxDecoration(color: WsT.well, borderRadius: BorderRadius.all(Shape.r(WsT.radius))),
    child: Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        for (final (i, s) in items.indexed)
          _Hover(
            onTap: () => onPick(i),
            builder: (h) => Container(
              width: 76,
              alignment: Alignment.center,
              decoration: BoxDecoration(color: i == index ? WsT.accent : (h ? WsT.raised : null), borderRadius: BorderRadius.all(Shape.r(WsT.radius - 2))),
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
          // (a step of the document back, or forward again)
          onTap: () {
            final ws = WsScope.read(context), m = ws.inside;
            if (m == null) return;
            ws.after(m.later(flip ? 'redo' : 'undo')); // (the window does not wait for the step)
          },
          builder: (h) => Container(
            width: 24,
            height: 24,
            alignment: Alignment.center,
            decoration: BoxDecoration(color: h ? WsT.raised : null, borderRadius: BorderRadius.all(Shape.r(WsT.radius))),
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
    if (SkinParts.draw(canvas, s, 'undo', c)) return; // (the skin's own picture, if it brings one)
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
          ..arcToPoint(Offset(8, s.height - 1), radius: Shape.r(3.2))
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
      decoration: BoxDecoration(color: h ? WsT.raised : null, borderRadius: BorderRadius.all(Shape.r(WsT.radius))),
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
    if (SkinParts.draw(canvas, s, 'shade', c)) return; // (the skin's own picture, if it brings one)
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

/// When the document's file was last written (a change is written as it is made).
String _savedAt(Ws ws) {
  final f = File('${ws.documentFolder}/${ws.document}');
  if (ws.document == null || !f.existsSync()) return 'Saved';
  final t = f.lastModifiedSync();
  return 'Saved ${t.hour.toString().padLeft(2, '0')}:${t.minute.toString().padLeft(2, '0')}';
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
        decoration: BoxDecoration(color: WsT.accent, shape: BoxShape.circle),
      ),
      const SizedBox(width: 6),
      ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 260),
        child: Text(WsScope.of(context).notice ?? _savedAt(WsScope.of(context)), maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(Grey.g63)),
      ),
    ],
  );
}

/// The one loud action: a flat lime pill with dark ink.
class _Render extends StatelessWidget {
  const _Render();
  @override
  Widget build(BuildContext context) => _Hover(
    onTap: WsScope.of(context).onRender,
    builder: (h) => Container(
      height: 26,
      padding: EdgeInsets.symmetric(horizontal: Shape.s(14)),
      alignment: Alignment.center,
      decoration: BoxDecoration(color: h ? WsT.accentHot : WsT.accent, borderRadius: BorderRadius.all(Shape.r(13))),
      child: Text('Render', style: T.name(WsT.onAccent).copyWith(fontWeight: FontWeight.w800)),
    ),
  );
}

/// How many times the window waited on the inside (the rule: never), a red dot and the count only when it did; a tap
/// lists each, named and timed.
class _Waited extends StatefulWidget {
  const _Waited();
  @override
  State<_Waited> createState() => _WaitedState();
}

class _WaitedState extends State<_Waited> {
  final _open = OverlayPortalController(), _at = LayerLink();
  late final Timer _look;
  int _count = 0;

  @override
  void initState() {
    super.initState();
    _look = Timer.periodic(const Duration(milliseconds: 500), (_) {   // (read without waiting: the inside's count)
      final n = WsScope.read(context).inside?.brokenCount ?? 0;
      if (n != _count && mounted) setState(() => _count = n);
    });
  }

  @override
  void dispose() {
    _look.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    if (_count == 0) return const SizedBox.shrink();
    final lines = _open.isShowing ? (WsScope.of(context).inside?.broken ?? '').split('\n').reversed.take(40).toList() : const <String>[];
    return CompositedTransformTarget(
      link: _at,
      child: OverlayPortal(
        controller: _open,
        overlayChildBuilder: (_) => CompositedTransformFollower(
          link: _at,
          targetAnchor: Alignment.bottomRight,
          followerAnchor: Alignment.topRight,
          offset: const Offset(0, 6),
          child: Align(
            alignment: Alignment.topRight,
            child: GestureDetector(
              onTap: () => setState(_open.hide),
              child: Container(
                width: 420,
                padding: EdgeInsets.all(Shape.s(10)),
                decoration: BoxDecoration(color: WsT.card, borderRadius: BorderRadius.all(Shape.r(6)), border: Border.all(color: Grey.g26)),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  mainAxisSize: MainAxisSize.min,
                  children: [for (final l in lines) Text(l, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.micro(Grey.g76))],
                ),
              ),
            ),
          ),
        ),
        child: _Hover(
          onTap: () => setState(_open.toggle),
          builder: (h) => Padding(
            padding: EdgeInsets.only(right: Shape.s(10)),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                Container(width: 6, height: 6, decoration: const BoxDecoration(color: Color(0xFFE5484D), shape: BoxShape.circle)),
                const SizedBox(width: 5),
                Text('$_count', style: T.label(h ? Grey.g91 : Grey.g63)),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
