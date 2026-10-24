// Part of feedback_parts.dart (cut there by size, as it was).
part of 'feedback_parts.dart';

// ---------------------------------------------------------------- empty state

class EmptyState extends StatelessWidget {
  const EmptyState({super.key, required this.title, required this.body, required this.action, this.illustration = true, this.look = Look.concept});
  final String title, body, action;
  final bool illustration;
  final Look look;
  @override
  Widget build(BuildContext context) => Column(mainAxisSize: MainAxisSize.min, children: [
        if (illustration) ...[CustomPaint(size: const Size(168, 96), painter: _EmptyP(look)), const SizedBox(height: 20)],
        Text(title, style: T.title(N.g95)),
        const SizedBox(height: 8),
        SizedBox(width: 248, child: Text(body, textAlign: TextAlign.center, style: T.label(N.g56).copyWith(fontSize: 11, height: 1.45))),
        const SizedBox(height: 16),
        Btn(action, height: 28),
      ]);
}

class _EmptyP extends CustomPainter {
  _EmptyP(this.look);
  final Look look;
  @override
  void paint(Canvas canvas, Size s) {
    final edge = Paint()..color = N.g26..style = PaintingStyle.stroke..strokeWidth = 1;
    const rows = [(8.0, 22.0, 90.0, 0), (10.0, 44.0, 120.0, 2), (6.0, 66.0, 70.0, 4)];
    for (final (x, y, w, f) in rows) {
      final r = RRect.fromRectAndRadius(Rect.fromLTWH(x, y - 8, s.width - 16, 16), const Radius.circular(3));
      canvas.drawRRect(r, Paint()..color = look == Look.quiet ? N.g00.withValues(alpha: 0) : N.bandA);
      canvas.drawRRect(r, edge);
      if (look != Look.quiet) {
        final bar = RRect.fromRectAndRadius(Rect.fromLTWH(x + 14 + f * 6, y - 5, w - f * 8, 10), const Radius.circular(2));
        canvas.drawRRect(bar, Paint()..color = Fam.all[f].c.withValues(alpha: .28));
      }
    }
    final px = s.width * .62;
    if (look == Look.glow) canvas.drawLine(Offset(px, 6), Offset(px, s.height - 4), Paint()..color = C.playhead.withValues(alpha: .5)..strokeWidth = 5..maskFilter = const MaskFilter.blur(BlurStyle.normal, 4));
    canvas.drawLine(Offset(px, 8), Offset(px, s.height - 4), Paint()..color = C.playhead..strokeWidth = 1);
    canvas.drawPath(Path()..moveTo(px - 4.5, 0)..lineTo(px + 4.5, 0)..lineTo(px, 8)..close(), Paint()..color = C.playhead);
  }

  @override
  bool shouldRepaint(_EmptyP old) => old.look != look;
}

// ---------------------------------------------------------------- skeleton

class Skeleton extends StatelessWidget {
  const Skeleton({super.key, required this.rows, this.animate = true, this.look = Look.concept});
  final int rows;
  final bool animate;
  final Look look;

  Widget _block(double v, {double? w, required double h, double r = 4}) {
    final pulse = look == Look.quiet;
    final hi = look == Look.glow ? N.g26 : N.g20;
    final BoxDecoration d = pulse
        ? BoxDecoration(color: Color.lerp(N.g15, N.g20, animate ? (math.sin(v * math.pi * 2) + 1) / 2 : 0), borderRadius: BorderRadius.circular(r))
        : BoxDecoration(
            borderRadius: BorderRadius.circular(r),
            gradient: LinearGradient(colors: [N.g15, animate ? hi : N.g15, N.g15], stops: const [.35, .5, .65], begin: Alignment(-3 + 6 * v, 0), end: Alignment(-1 + 6 * v, 0)),
          );
    return Container(width: w, height: h, decoration: d);
  }

  @override
  Widget build(BuildContext context) => Loop(
        duration: Duration(milliseconds: look == Look.quiet ? 1600 : 1300),
        running: animate,
        builder: (c, v) => Column(mainAxisSize: MainAxisSize.min, children: [
          for (var i = 0; i < rows; i++)
            Padding(
              padding: const EdgeInsets.symmetric(vertical: 6),
              child: Flex(direction: Axis.horizontal, children: [
                _block(v, w: 48, h: 28, r: 4),
                const SizedBox(width: 12),
                Expanded(child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [_block(v, h: 8, w: 120.0 + (i % 3) * 36), const SizedBox(height: 8), _block(v, h: 8)])),
                const SizedBox(width: 12),
                _block(v, w: 28, h: 8),
              ]),
            ),
        ]),
      );
}

// ---------------------------------------------------------------- kbd & tooltip

class Kbd extends StatelessWidget {
  const Kbd(this.keyLabel, {super.key, this.pressed = false, this.look = Look.concept});
  final String keyLabel;
  final bool pressed;
  final Look look;
  @override
  Widget build(BuildContext context) {
    final border = switch (look) {
      Look.concept => Border.all(color: N.g26),
      Look.quiet => null,
      Look.glow => const Border(top: BorderSide(color: N.g26), left: BorderSide(color: N.g26), right: BorderSide(color: N.g26), bottom: BorderSide(color: N.g38, width: 2)),
    };
    return AnimatedContainer(
      duration: _fast,
      curve: Curves.easeOut,
      constraints: const BoxConstraints(minWidth: 18),
      height: 18,
      padding: const EdgeInsets.symmetric(horizontal: 5),
      alignment: Alignment.center,
      decoration: BoxDecoration(color: pressed ? N.g26 : (look == Look.quiet ? N.g20 : N.g13), borderRadius: BorderRadius.circular(4), border: border),
      child: Text(keyLabel, style: T.micro(pressed ? N.g100 : N.g76).copyWith(fontFamily: T.mono, letterSpacing: 0)),
    );
  }
}

class Combo extends StatelessWidget {
  const Combo(this.keys, {super.key, this.pressed = false, this.look = Look.concept});
  final List<String> keys;
  final bool pressed;
  final Look look;
  @override
  Widget build(BuildContext context) => Flex(direction: Axis.horizontal, mainAxisSize: MainAxisSize.min, children: [
        for (var i = 0; i < keys.length; i++) ...[if (i > 0) const SizedBox(width: 3), Kbd(keys[i], pressed: pressed, look: look)],
      ]);
}

class TipBubble extends StatelessWidget {
  const TipBubble({super.key, required this.label, required this.keys, this.detail = '', this.look = Look.concept});
  final String label, detail;
  final List<String> keys;
  final Look look;
  @override
  Widget build(BuildContext context) => Container(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
        decoration: BoxDecoration(color: N.g15, borderRadius: BorderRadius.circular(6), border: look == Look.quiet ? null : Border.all(color: N.g20), boxShadow: floatShadow(look == Look.glow ? .6 : .4)),
        child: Flex(direction: Axis.horizontal, mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.center, children: [
          Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
            Text(label, style: T.name(N.g95)),
            if (detail.isNotEmpty) ...[const SizedBox(height: 4), Text(detail, style: T.label(N.g56))],
          ]),
          if (keys.isNotEmpty) ...[
            const SizedBox(width: 12),
            if (look == Look.quiet) Text(keys.join(' '), style: T.value(N.g56)) else Combo(keys, look: look == Look.glow ? Look.glow : Look.concept),
          ],
        ]),
      );
}

// ---------------------------------------------------------------- badge & change dot

class CountPill extends StatelessWidget {
  const CountPill(this.count, {super.key, this.kind, this.max = 99});
  final int count;
  final Kind? kind;
  final int max;
  @override
  Widget build(BuildContext context) {
    final col = kind == null ? N.g91 : kindColor(kind!);
    final text = count > max ? '$max+' : '$count';
    return Container(
      constraints: const BoxConstraints(minWidth: 16),
      height: 16,
      padding: const EdgeInsets.symmetric(horizontal: 5),
      alignment: Alignment.center,
      decoration: BoxDecoration(color: kind == null ? N.g20 : col.withValues(alpha: .16), borderRadius: BorderRadius.circular(8)),
      child: Text(text, style: T.micro(N.g91).copyWith(fontFamily: T.mono, letterSpacing: 0)),
    );
  }
}

class ChangeDot extends StatelessWidget {
  const ChangeDot(this.change, {super.key, this.size = 6});
  final Change change;
  final double size;
  @override
  Widget build(BuildContext context) => AnimatedContainer(duration: _fast, curve: Curves.easeOut, width: size, height: size, decoration: BoxDecoration(color: changeColor(change), shape: BoxShape.circle));
}

// ---------------------------------------------------------------- error row

class ErrorRow extends StatefulWidget {
  const ErrorRow({super.key, required this.message, required this.detail, this.expanded = false, this.look = Look.concept});
  final String message, detail;
  final bool expanded;
  final Look look;
  @override
  State<ErrorRow> createState() => _ErrorRowState();
}

class _ErrorRowState extends State<ErrorRow> {
  late bool _open = widget.expanded;
  @override
  Widget build(BuildContext context) {
    final glow = widget.look == Look.glow;
    return Container(
      decoration: BoxDecoration(color: glow ? Color.alphaBlend(Role.error.withValues(alpha: .06), N.g13) : N.g13, borderRadius: BorderRadius.circular(4), border: glow ? Border.all(color: Role.error.withValues(alpha: .3)) : null),
      child: ClipRRect(
        borderRadius: BorderRadius.circular(4),
        child: Stack(children: [
          if (widget.look == Look.concept) Positioned(left: 0, top: 0, bottom: 0, width: 2, child: ColoredBox(color: Role.error)),
          Column(mainAxisSize: MainAxisSize.min, children: [
            Hov(
              onTap: () => setState(() => _open = !_open),
              builder: (c, h) => AnimatedContainer(
                duration: _fast,
                height: 28,
                padding: const EdgeInsets.fromLTRB(10, 0, 10, 0),
                color: h ? N.g15 : N.g00.withValues(alpha: 0),
                child: Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.center, children: [
                  const KindGlyph(Kind.error, size: 12),
                  const SizedBox(width: 8),
                  Expanded(child: Text(widget.message, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(N.g91))),
                  const SizedBox(width: 8),
                  Text('Why?', style: T.label(h || _open ? N.g95 : N.g63)),
                  const SizedBox(width: 4),
                  AnimatedRotation(turns: _open ? .25 : 0, duration: _fast, curve: Curves.easeOut, child: Chevron(color: h || _open ? N.g95 : N.g63, size: 10)),
                ]),
              ),
            ),
            AnimatedSize(
              duration: _fast,
              curve: Curves.easeOut,
              alignment: Alignment.topCenter,
              child: _open
                  ? Padding(
                      padding: const EdgeInsets.fromLTRB(30, 2, 10, 10),
                      child: Container(
                        width: double.infinity,
                        padding: const EdgeInsets.all(8),
                        decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(4)),
                        child: Text(widget.detail, style: T.value(N.g63).copyWith(height: 1.5)),
                      ),
                    )
                  : const SizedBox(width: double.infinity),
            ),
          ]),
        ]),
      ),
    );
  }
}

// ---------------------------------------------------------------- render queue

class _ThumbP extends CustomPainter {
  _ThumbP(this.seed);
  final int seed;
  @override
  void paint(Canvas canvas, Size s) {
    canvas.drawRect(Offset.zero & s, Paint()..color = N.g13);
    final f = Fam.all[seed % Fam.all.length].c;
    canvas.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(s.width * .14, s.height * .3, s.width * .5, s.height * .22), const Radius.circular(2)), Paint()..color = f.withValues(alpha: .4));
    canvas.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(s.width * .3, s.height * .6, s.width * .5, s.height * .16), const Radius.circular(2)), Paint()..color = f.withValues(alpha: .22));
    canvas.drawRect((Offset.zero & s).deflate(.5), Paint()..color = N.g20..style = PaintingStyle.stroke);
  }

  @override
  bool shouldRepaint(_ThumbP old) => old.seed != seed;
}

class QueueRow extends StatelessWidget {
  const QueueRow({super.key, required this.name, required this.sub, required this.state, required this.progress, required this.eta, this.seed = 0, this.look = Look.concept});
  final String name, sub, eta;
  final QState state;
  final double progress;
  final int seed;
  final Look look;
  @override
  Widget build(BuildContext context) {
    final (String status, Color sc) = switch (state) {
      QState.queued => ('Queued', N.g56),
      QState.rendering => ('${(progress * 100).round()}%', N.g91),
      QState.done => ('Done', N.g63),
      QState.failed => ('Failed', Role.error),
    };
    final barColor = switch (state) { QState.done => N.g63, QState.failed => Role.error, _ => null };
    return Hov(
      onTap: () {},
      builder: (c, h) => AnimatedContainer(
        duration: _fast,
        curve: Curves.easeOut,
        width: 520,
        height: 52,
        padding: const EdgeInsets.fromLTRB(8, 0, 6, 0),
        decoration: BoxDecoration(color: h ? N.g15 : N.g13, borderRadius: BorderRadius.circular(4)),
        child: Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.center, children: [
          ClipRRect(borderRadius: BorderRadius.circular(3), child: Opacity(opacity: state == QState.queued ? .55 : 1, child: CustomPaint(size: const Size(64, 36), painter: _ThumbP(seed)))),
          const SizedBox(width: 12),
          Expanded(
            child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
              Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.center, children: [
                // the name and its detail share ONE Expanded: a Spacer between loose Flexibles does not absorb the slack, so the status cell used to move with the text lengths
                Expanded(child: Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.center, children: [
                  Flexible(child: Text(name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(N.g95))),
                  const SizedBox(width: 8),
                  Flexible(child: Text(sub, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g56))),
                ])),
                SizedBox(width: 56, child: state == QState.failed ? Row(children: [const ErrMark(size: 9, gap: 3), Text(status, softWrap: false, maxLines: 1, style: T.value(sc).copyWith(fontSize: 10.5))]) : Text(status, softWrap: false, maxLines: 1, overflow: TextOverflow.clip, textAlign: TextAlign.left, style: T.value(sc).copyWith(fontSize: 10.5))),
                SizedBox(width: 72, child: state == QState.rendering ? Text.rich(TextSpan(children: [TextSpan(text: 'ETA ', style: T.label(N.g56)), TextSpan(text: eta, style: T.value(N.g76).copyWith(fontSize: 10.5))]), softWrap: false, maxLines: 1, overflow: TextOverflow.clip) : const SizedBox()),
              ]),
              const SizedBox(height: 8),
              ProgressBar(value: switch (state) { QState.queued => 0, QState.done => 1, _ => progress }, height: 3, look: look, color: barColor),
            ]),
          ),
          const SizedBox(width: 8),
          Hov(
            onTap: () {},
            builder: (c, hh) => AnimatedContainer(duration: _fast, width: 24, height: 24, alignment: Alignment.center, decoration: BoxDecoration(color: hh ? N.g20 : N.g00.withValues(alpha: 0), borderRadius: BorderRadius.circular(4)), child: XGlyph(color: hh ? N.g95 : (h ? N.g63 : N.g44), size: 8)),
          ),
        ]),
      ),
    );
  }
}

/// Quiet segmented control: selected = g20 fill, g95 text, 1px g26 border; no accent.
class Seg extends StatelessWidget {
  const Seg({super.key, required this.items, required this.index, this.onChanged});
  final List<String> items;
  final int index;
  final ValueChanged<int>? onChanged;
  @override
  Widget build(BuildContext context) => Container(
        height: 26,
        padding: const EdgeInsets.all(2),
        decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(6)),
        child: Flex(direction: Axis.horizontal, mainAxisSize: MainAxisSize.min, children: [
          for (var i = 0; i < items.length; i++)
            Hov(
              onTap: () => onChanged?.call(i),
              builder: (c, h) => AnimatedContainer(
                duration: _fast,
                curve: Curves.easeOut,
                padding: const EdgeInsets.symmetric(horizontal: 12),
                alignment: Alignment.center,
                decoration: BoxDecoration(color: i == index ? N.g20 : (h ? N.g15 : N.g00.withValues(alpha: 0)), borderRadius: BorderRadius.circular(4), border: Border.all(color: i == index ? N.g26 : N.g00.withValues(alpha: 0))),
                child: Text(items[i].toUpperCase(), style: T.micro(i == index ? N.g95 : N.g63).copyWith(letterSpacing: .6)),
              ),
            ),
        ]),
      );
}

// ---------------------------------------------------------------- export sheet

class ExportSheet extends StatefulWidget {
  const ExportSheet({super.key, required this.state, required this.progress, required this.fileName, this.look = Look.concept});
  final ExportState state;
  final double progress;
  final String fileName;
  final Look look;
  @override
  State<ExportSheet> createState() => _ExportSheetState();
}

class _ExportSheetState extends State<ExportSheet> {
  int _fmt = 0, _range = 0;
  static const _fmts = ['MP4', 'MOV', 'GIF'], _ranges = ['Work area', 'Full', 'Custom'];
  static const _est = ['48 MB', '212 MB', '9 MB'];
  static const _rangeText = ['00:00:02:00 - 00:00:10:00', '00:00:00:00 - 00:00:12:00', '00:00:04:12 - 00:00:07:00'];

  Widget _field(String label, Widget child) => Padding(
        padding: const EdgeInsets.only(bottom: 12),
        child: Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.start, children: [SizedBox(width: 80, height: 26, child: Align(alignment: Alignment.centerLeft, child: Text(label, style: T.label(N.g56)))), Expanded(child: Align(alignment: Alignment.centerLeft, widthFactor: null, child: child))]),
      );

  @override
  Widget build(BuildContext context) {
    final ext = _fmts[_fmt].toLowerCase();
    final name = '${widget.fileName}.$ext';
    final pct = (widget.progress * 100).round();
    final Widget action = switch (widget.state) {
      ExportState.idle => Btn('Export', primary: true, height: 36, look: widget.look),
      ExportState.exporting => Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.center, children: [
            Text('Exporting', style: T.name(N.g95)),
            const Spacer(),
            Text('$pct%', style: T.value(N.g95)),
          ]),
          const SizedBox(height: 8),
          ProgressBar(value: widget.progress, height: 6, look: widget.look),
          const SizedBox(height: 8),
          Flex(direction: Axis.horizontal, children: [Text('Frame ${(widget.progress * 300).round()} / 300   ETA 00:${(12 * (1 - widget.progress)).round().toString().padLeft(2, '0')}', style: T.value(N.g56).copyWith(fontSize: 10)), const Spacer(), const Btn('Cancel')]),
        ]),
      ExportState.done => Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.center, children: [
          const KindGlyph(Kind.success),
          const SizedBox(width: 8),
          Expanded(child: Text('Saved $name', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(N.g91))),
          const SizedBox(width: 8),
          const Btn('Reveal'),
        ]),
    };
    return Container(
      width: 420,
      padding: const EdgeInsets.all(20),
      decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(8), border: Border.all(color: N.g20), boxShadow: floatShadow()),
      child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        Text('Export', style: T.title()),
        const SizedBox(height: 4),
        Text('Title_v3  -  1920 x 1080  -  30 fps', style: T.label(N.g56)),
        const SizedBox(height: 20),
        _field('Format', Seg(items: _fmts, index: _fmt, onChanged: (i) => setState(() => _fmt = i))),
        _field('Range', Column(crossAxisAlignment: CrossAxisAlignment.start, children: [Seg(items: _ranges, index: _range, onChanged: (i) => setState(() => _range = i)), const SizedBox(height: 8), Text(_rangeText[_range], style: T.value(N.g63).copyWith(fontSize: 10.5))])),
        _field(
          'Destination',
          Flex(direction: Axis.horizontal, children: [
            Expanded(child: Container(height: 26, padding: const EdgeInsets.symmetric(horizontal: 8), alignment: Alignment.centerLeft, decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(4)), child: Text('~/Movies/Motolii/$name', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.value(N.g76)))),
            const SizedBox(width: 8),
            const Btn('Change'),
          ]),
        ),
        Padding(padding: const EdgeInsets.only(left: 80, bottom: 16), child: Text('About ${_est[_fmt]}', style: T.label(N.g44))),
        const SizedBox(height: 4),
        AnimatedSwitcher(duration: _fast, child: KeyedSubtree(key: ValueKey(widget.state), child: action)),
      ]),
    );
  }
}

// ---------------------------------------------------------------- undo history

class HistoryList extends StatefulWidget {
  const HistoryList({super.key, required this.count, required this.current, this.look = Look.concept});
  final int count, current;
  final Look look;
  @override
  State<HistoryList> createState() => _HistoryListState();
}

class _HistoryListState extends State<HistoryList> {
  late int _cur = widget.current;
  static const _items = <(String, Change, String)>[
    ('Open composition', Change.none, '12:01'),
    ('Add layer "Hero_BG"', Change.added, '12:02'),
    ('Set Position', Change.modified, '12:02'),
    ('Move keyframe', Change.modified, '12:04'),
    ('Delete layer "Spark"', Change.removed, '12:05'),
    ('Add Stagger relation', Change.added, '12:07'),
    ('Set Opacity', Change.modified, '12:08'),
    ('Rename "Hero_BG"', Change.modified, '12:09'),
    ('Trim layer end', Change.modified, '12:10'),
    ('Add layer "Title"', Change.added, '12:11'),
  ];
  @override
  Widget build(BuildContext context) {
    final n = widget.count.clamp(1, _items.length);
    final cur = _cur.clamp(0, n - 1);
    return Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      Padding(padding: const EdgeInsets.fromLTRB(8, 0, 8, 8), child: Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.center, children: [Text('HISTORY', style: T.micro(N.g56).copyWith(letterSpacing: .8)), const Spacer(), CountPill(n - 1 - cur, kind: n - 1 - cur > 0 ? Kind.info : null)])),
      for (var i = 0; i < n; i++)
        Padding(
          padding: const EdgeInsets.only(bottom: 2),
          child: Hov(
            onTap: () => setState(() => _cur = i),
            builder: (c, h) {
              final isCur = i == cur, undone = i > cur;
              final glow = widget.look == Look.glow, quiet = widget.look == Look.quiet;
              final bg = isCur && !quiet ? (glow ? N.g15 : N.g20) : (h ? N.g15 : N.g00.withValues(alpha: 0));
              return AnimatedContainer(
                duration: _fast,
                curve: Curves.easeOut,
                height: 26,
                padding: const EdgeInsets.symmetric(horizontal: 8),
                decoration: BoxDecoration(color: bg, borderRadius: BorderRadius.circular(4), border: glow && isCur ? Border(left: BorderSide(color: Role.selected, width: 2)) : null),
                child: Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.center, children: [
                  Opacity(opacity: undone ? .35 : 1, child: ChangeDot(_items[i].$2 == Change.none ? Change.modified : _items[i].$2)),
                  const SizedBox(width: 10),
                  Expanded(child: Text(_items[i].$1, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(undone ? N.g44 : (isCur ? N.g100 : N.g91)))),
                  if (isCur && quiet) ...[Container(width: 5, height: 5, decoration: BoxDecoration(color: C.mode, shape: BoxShape.circle)), const SizedBox(width: 8)],
                  Text(_items[i].$3, style: T.value(N.g44).copyWith(fontSize: 10)),
                ]),
              );
            },
          ),
        ),
    ]);
  }
}
