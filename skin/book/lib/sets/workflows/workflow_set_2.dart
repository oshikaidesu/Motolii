// Part of workflow_set.dart (cut there by size, as it was).
part of 'workflow_set.dart';

class _Inspector extends StatelessWidget {
  const _Inspector();
  @override
  Widget build(BuildContext context) {
    final s = SessionScope.of(context), p = s.primary, n = s.selected.length;
    final (kicker, title) = n == 0 ? ('NOTHING SELECTED', '') : (n == 1 ? ('LAYER', p!.name) : ('SEVERAL SELECTED', '$n layers'));
    return Container(
      color: N.g10,
      padding: const EdgeInsets.all(12),
      child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        // pinned: the header and, for one layer, the position key; only the fields below scroll
        Text(kicker, style: T.micro(N.g56).copyWith(letterSpacing: .8)),
        if (title.isNotEmpty) ...[const SizedBox(height: 4), Text(title, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.title())],
        const SizedBox(height: 10),
        Container(height: 1, color: N.g15),
        if (n == 1) ...[
          const SizedBox(height: 8),
          Row(children: [
            Text('POSITION KEY', style: T.micro(N.g56).copyWith(letterSpacing: .8)),
            const Spacer(),
            Text(_time(s.frame), style: T.value(N.g76)),
            const SizedBox(width: 6),
            KeyMark(state: p!.pk.isEmpty ? KeyS.off : (s.keyAtHead ? KeyS.at : KeyS.anim), onTap: s.toggleKey),
          ]),
        ],
        const SizedBox(height: 10),
        Expanded(
          child: SingleChildScrollView(
            child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
              if (n == 0)
                Text('Pick a layer on the Stage, in the list or on the Timeline.', style: T.label(N.g56).copyWith(height: 1.4))
              else if (n == 1)
                Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
                  // the attached behaviour is the thing being worked on: it sits above the Transform, not below it
                  if (p!.wave != null) _WaveSection(p),
                  KeyedSubtree(key: ValueKey(p.id), child: TransformBoth(p.doc)),
                ])
              else ...[
                Wrap(spacing: 10, runSpacing: 4, children: [
                  for (final id in s.selected)
                    Row(mainAxisSize: MainAxisSize.min, children: [
                      Container(width: 8, height: 8, decoration: BoxDecoration(color: s.byId(id).fam.c, borderRadius: BorderRadius.circular(2))),
                      const SizedBox(width: 4),
                      Text(s.byId(id).name, style: T.label(N.g76)),
                    ]),
                ]),
                const SizedBox(height: 6),
                Text('— = the layers differ. A scrub moves each by the same amount; a typed value sets them all.', style: T.label(N.g56).copyWith(height: 1.4)),
                const SizedBox(height: 10),
                TransformBoth(s.group),
              ],
              const SizedBox(height: 12),
              Text('Click · Shift add · Tab · Esc · Space · Delete', style: T.label(N.g56).copyWith(height: 1.4)),
            ]),
          ),
        ),
        // pinned: the undo list is always in view
        Container(height: 1, color: N.g15),
        Row(children: [
          Text(s.undoSteps == 1 ? '1 step' : '${s.undoSteps} steps', style: T.value(N.g76)),
          const SizedBox(width: 12),
          MouseRegion(
            cursor: s.undoSteps == 0 ? SystemMouseCursors.basic : SystemMouseCursors.click,
            child: GestureDetector(
              behavior: HitTestBehavior.opaque,
              onTap: s.undo,
              child: Padding(padding: const EdgeInsets.symmetric(vertical: 8), child: Text('Undo', style: T.name(s.undoSteps == 0 ? N.g56 : N.g95))),
            ),
          ),
        ]),
      ]),
    );
  }
}

/// One box around every selected layer (W5): the selection as a whole.
class _UnionPaint extends CustomPainter {
  const _UnionPaint(this.pts);
  final List<Offset> pts;
  @override
  void paint(Canvas cv, Size sz) {
    final r = Rect.fromLTRB(pts.map((p) => p.dx).reduce(math.min), pts.map((p) => p.dy).reduce(math.min), pts.map((p) => p.dx).reduce(math.max), pts.map((p) => p.dy).reduce(math.max)).inflate(6);
    cv.drawRect(r, Paint()..style = PaintingStyle.stroke..strokeWidth = 1..color = N.g76.withValues(alpha: .8));
  }

  @override
  bool shouldRepaint(_UnionPaint o) => true;
}

/// The selected layer's motion on the stage: a line through its keys, a diamond at each (the one under the playhead blue, the picked one accent).
class _KeysPaint extends CustomPainter {
  const _KeysPaint(this.l, this.size, this.head, this.picked);
  final LayerM l;
  final Size size;
  final int head;
  final Set<int> picked;
  @override
  void paint(Canvas cv, Size sz) {
    final k = Gui.span / size.width, ks = l.keyFrames, pts = [for (final f in ks) size.center(Offset.zero) + l.pk[f]! / k];
    if (pts.length > 1) cv.drawPath(Path()..addPolygon(pts, false), Paint()..style = PaintingStyle.stroke..strokeWidth = 1..color = N.g100.withValues(alpha: .7));
    for (var i = 0; i < pts.length; i++) {
      final p = pts[i], r = 4.5, d = Path()..moveTo(p.dx, p.dy - r)..lineTo(p.dx + r, p.dy)..lineTo(p.dx, p.dy + r)..lineTo(p.dx - r, p.dy)..close();
      cv.drawPath(d, Paint()..color = picked.contains(ks[i]) ? Role.selected : (ks[i] == head ? C.playhead : Role.of(N.g95, Role.key).withValues(alpha: .9)));
      cv.drawPath(d, Paint()..style = PaintingStyle.stroke..strokeWidth = picked.contains(ks[i]) ? 1.5 : (Role.keyEdge != null ? 1 : .75)..color = picked.contains(ks[i]) ? N.g100 : (Role.keyEdge ?? N.g100.withValues(alpha: .6)));
    }
  }

  @override
  bool shouldRepaint(_KeysPaint o) => true;
}


// ---- the behaviour shelf (B1) -----------------------------------------------------------------------------------------------------------

/// One period of a sine, one line: the shelf's Wave and the mark on a layer that has it.
class WaveGlyph extends StatelessWidget {
  const WaveGlyph({super.key, this.color = N.g76});
  final Color color;
  @override
  Widget build(BuildContext context) => SizedBox(width: 18, height: 10, child: CustomPaint(painter: _WavePaint(color)));
}

class _WavePaint extends CustomPainter {
  _WavePaint(this.c);
  final Color c;
  @override
  void paint(Canvas canvas, Size z) {
    final path = Path();
    for (var i = 0; i <= 24; i++) {
      final x = z.width * i / 24, y = z.height / 2 - math.sin(i / 24 * 2 * math.pi) * (z.height / 2 - 1);
      i == 0 ? path.moveTo(x, y) : path.lineTo(x, y);
    }
    canvas.drawPath(path, Paint()..color = c..style = PaintingStyle.stroke..strokeWidth = 1);
  }

  @override
  bool shouldRepaint(_WavePaint o) => o.c != c;
}

class _Shelf extends StatelessWidget {
  const _Shelf();
  @override
  Widget build(BuildContext context) {
    final s = SessionScope.of(context);
    Widget row(bool hot) => Container(
          height: TL.height,
          padding: const EdgeInsets.symmetric(horizontal: 12),
          color: hot ? N.g15 : N.g10,
          child: Row(children: [
            const WaveGlyph(color: N.g95),
            const SizedBox(width: 8),
            Expanded(child: Text('Wave', style: T.name(N.g91))),
          ]),
        );
    return Container(
      color: N.g10,
      padding: const EdgeInsets.fromLTRB(0, 12, 0, 8),
      child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        Padding(padding: const EdgeInsets.fromLTRB(12, 0, 12, 8), child: Text('SHELF', style: T.micro(N.g56).copyWith(letterSpacing: .8))),
        _ShelfItem(
          // click attaches to the selected layer; a drag carries it to any layer
          dragChild: SizedBox(width: _listW, child: row(true)),
          onTap: () {
            final p = s.primary;
            if (p != null) s.attachWave(p);
          },
          child: row(false),
        ),
      ]),
    );
  }
}

class _ShelfItem extends StatefulWidget {
  const _ShelfItem({required this.child, required this.dragChild, required this.onTap});
  final Widget child, dragChild;
  final VoidCallback onTap;
  @override
  State<_ShelfItem> createState() => _ShelfItemState();
}

class _ShelfItemState extends State<_ShelfItem> {
  bool hov = false;
  @override
  Widget build(BuildContext context) => MouseRegion(
        cursor: SystemMouseCursors.grab,
        onEnter: (_) => setState(() => hov = true),
        onExit: (_) => setState(() => hov = false),
        child: Draggable<String>(
          data: 'wave',
          feedback: Opacity(opacity: .92, child: DefaultTextStyle(style: const TextStyle(decoration: TextDecoration.none), child: widget.dragChild)),
          childWhenDragging: Opacity(opacity: .4, child: widget.child),
          child: GestureDetector(
            behavior: HitTestBehavior.opaque,
            onTap: widget.onTap,
            child: AnimatedContainer(duration: Mo.dur, curve: Mo.ease, color: hov ? N.g15 : const Color(0x00262626), child: widget.child),
          ),
        ),
      );
}

/// What the attached wave reads as in the Inspector: its two numbers, edited like any other number, and a way to take it off.
class _WaveSection extends StatelessWidget {
  const _WaveSection(this.l);
  final LayerM l;
  @override
  Widget build(BuildContext context) {
    final s = SessionScope.of(context), w = l.wave!;
    return Padding(
      padding: const EdgeInsets.only(bottom: 12),
      child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        Row(children: [
          const WaveGlyph(color: N.g95),
          const SizedBox(width: 8),
          Text('WAVE', style: T.micro(N.g56).copyWith(letterSpacing: .8)),
          const Spacer(),
          MouseRegion(
            cursor: SystemMouseCursors.click,
            child: GestureDetector(behavior: HitTestBehavior.opaque, onTap: () => s.removeWave(l), child: Padding(padding: const EdgeInsets.all(4), child: Text('Remove', style: T.name(N.g56)))),
          ),
        ]),
        const SizedBox(height: 8),
        ScrubField(label: 'Amplitude', unit: ' px', value: w.amp, min: 0, max: 400, perPx: 1, decimals: 0, onChanged: (v) => s.setWave(l, amp: v)),
        const SizedBox(height: 4),
        ScrubField(label: 'Frequency', unit: ' Hz', value: w.freq, min: .1, max: 8, perPx: .02, decimals: 1, onChanged: (v) => s.setWave(l, freq: v)),
        const SizedBox(height: 12),
        Container(height: 1, color: N.g15),
      ]),
    );
  }
}
