// Part of media_set.dart (cut there by size, as it was).
part of 'media_set.dart';

class _PresetRow extends StatefulWidget {
  const _PresetRow({required this.name, required this.meta, required this.fam, required this.motion, required this.always, required this.fav, required this.onFav, required this.selected, required this.onTap});
  final String name, meta;
  final Fam fam;
  final _Motion motion;
  final bool always, fav, selected;
  final ValueChanged<bool> onFav;
  final VoidCallback onTap;
  @override
  State<_PresetRow> createState() => _PresetRowState();
}

class _PresetRowState extends State<_PresetRow> with SingleTickerProviderStateMixin {
  static const _rest = .62;
  late final _ctl = AnimationController(vsync: this, duration: const Duration(milliseconds: 1400), value: _rest);
  bool _hover = false;

  void _sync() {
    if (widget.always || _hover) {
      if (!_ctl.isAnimating) _ctl.repeat();
    } else {
      _ctl.stop();
      _ctl.value = _rest;
    }
  }

  @override
  void initState() {
    super.initState();
    _sync();
  }

  @override
  void didUpdateWidget(_PresetRow o) {
    super.didUpdateWidget(o);
    _sync();
  }

  @override
  void dispose() {
    _ctl.dispose();
    super.dispose();
  }

  void _paint(Canvas cv, Size s, double t) {
    final col = widget.fam.c;
    final fill = Paint()..color = col;
    switch (widget.motion) {
      case _Motion.bounce:
        final x = 8 + (s.width - 16) * (.5 - .5 * math.cos(2 * math.pi * Curves.easeInOut.transform(t)));
        cv.drawCircle(Offset(x, s.height / 2), 5, fill);
      case _Motion.fade:
        final a = .15 + .85 * (.5 - .5 * math.cos(2 * math.pi * t));
        cv.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: s.center(Offset.zero), width: 18, height: 12), const Radius.circular(3)), Paint()..color = col.withValues(alpha: a));
      case _Motion.spin:
        cv.save();
        cv.translate(s.width / 2, s.height / 2);
        cv.rotate(2 * math.pi * Curves.easeInOut.transform(t));
        cv.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: Offset.zero, width: 20, height: 6), const Radius.circular(3)), fill);
        cv.restore();
      case _Motion.pop:
        final k = t < .5 ? Curves.easeOutBack.transform(t * 2) : 1 - Curves.easeIn.transform((t - .5) * 2);
        cv.drawCircle(s.center(Offset.zero), 2 + 8 * k.clamp(0.0, 1.3), fill);
    }
  }

  @override
  Widget build(BuildContext context) => MouseRegion(
        onEnter: (_) => setState(() {
          _hover = true;
          _sync();
        }),
        onExit: (_) => setState(() {
          _hover = false;
          _sync();
        }),
        child: Press(
          onTap: widget.onTap,
          builder: (context, hover, focus, _) => AnimatedContainer(
            duration: kFast,
            curve: kEase,
            height: _presetH,
            padding: const EdgeInsets.fromLTRB(6, 0, 2, 0),
            decoration: BoxDecoration(color: widget.selected ? N.g20 : (hover ? N.g15 : const Color(0x00000000)), borderRadius: BorderRadius.circular(4), border: Border.all(color: focus ? N.g56 : const Color(0x00000000))),
            child: Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.center, children: [
              Tick(widget.selected, height: 24),
              Container(
                width: 56,
                height: 28,
                decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(4), border: Border.all(color: N.g20)),
                child: AnimatedBuilder(animation: _ctl, builder: (context, _) => CustomPaint(painter: Draw((cv, s) => _paint(cv, s, _ctl.value), _ctl.value))),
              ),
              const SizedBox(width: 10),
              Expanded(child: Flex(direction: Axis.vertical, mainAxisAlignment: MainAxisAlignment.center, crossAxisAlignment: CrossAxisAlignment.start, children: [
                Text(widget.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(widget.selected ? N.g95 : N.g91)),
                const SizedBox(height: 4),
                Text(widget.meta, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g56)),
              ])),
              StarButton(on: widget.fav, onChanged: widget.onFav),
            ]),
          ),
        ),
      );
}

Widget _presetCase(BuildContext c) {
  final fam = c.knobs.object.dropdown<Fam>(label: 'Family', options: Fam.all, initialOption: Fam.stagger, labelBuilder: (f) => f.name);
  final motion = c.knobs.object.dropdown<_Motion>(label: 'Motion', options: _Motion.values, initialOption: _Motion.bounce, labelBuilder: (m) => m.name);
  final always = c.knobs.boolean(label: 'Always animate', initialValue: false);
  final name = c.knobs.string(label: 'Name', initialValue: 'Soft bounce');
  return Hold<({int sel, Set<int> fav})>(
    initial: (sel: 0, fav: {0}),
    builder: (context, st, set) {
      final specs = [(name, '${fam.name} · 0.6 s', fam, motion), ('Slow fade', 'Stagger · 1.2 s', Fam.stagger, _Motion.fade), ('Whip spin', 'Face · 0.4 s', Fam.face, _Motion.spin), ('Pop in', 'Scatter · 0.3 s', Fam.scatter, _Motion.pop)];
      return Flex(direction: Axis.vertical, mainAxisSize: MainAxisSize.min, children: [
        for (var i = 0; i < specs.length; i++)
          _PresetRow(
            name: specs[i].$1,
            meta: specs[i].$2,
            fam: specs[i].$3,
            motion: specs[i].$4,
            always: always,
            fav: st.fav.contains(i),
            selected: st.sel == i,
            onTap: () => set((sel: i, fav: st.fav)),
            onFav: (v) => set((sel: st.sel, fav: v ? {...st.fav, i} : ({...st.fav}..remove(i)))),
          ),
      ]);
    },
  );
}

// ------------------------------------------------------------------- font row

class _FontRow extends StatelessWidget {
  const _FontRow({required this.family, required this.meta, required this.sample, required this.size, required this.selected, required this.onTap});
  final String family, meta, sample;
  final double size;
  final bool selected;
  final VoidCallback onTap;

  TextStyle _style(double s, Color col) => TextStyle(fontFamily: family, fontSize: s, color: T.hidden.value ? const Color(0x00000000) : col, height: 1.1, decoration: TextDecoration.none);

  @override
  Widget build(BuildContext context) => Hold<({double size, bool fav})>(
        key: ValueKey(size),
        initial: (size: size, fav: false),
        builder: (context, st, set) => Press(
          onTap: onTap,
          builder: (context, hover, focus, _) => AnimatedContainer(
            duration: kFast,
            curve: kEase,
            height: _fontH,
            padding: const EdgeInsets.symmetric(horizontal: 8),
            decoration: BoxDecoration(color: selected ? N.g20 : (hover ? N.g15 : const Color(0x00000000)), borderRadius: BorderRadius.circular(4), border: Border.all(color: focus ? N.g56 : const Color(0x00000000))),
            child: Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.center, children: [
              Tick(selected, height: 32),
              Container(width: 40, height: 40, alignment: Alignment.center, decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(4), border: Border.all(color: N.g20)), child: Text('Aa', style: _style(22, N.g95))),
              const SizedBox(width: 12),
              SizedBox(width: 108, child: Flex(direction: Axis.vertical, mainAxisAlignment: MainAxisAlignment.center, crossAxisAlignment: CrossAxisAlignment.start, children: [
                Text(family, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(selected ? N.g95 : N.g91)),
                const SizedBox(height: 4),
                Text(meta, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g56)),
              ])),
              Expanded(child: ClipRect(child: Text(sample, maxLines: 1, softWrap: false, overflow: TextOverflow.clip, style: _style(st.size, N.g95)))),
              const SizedBox(width: 12),
              MiniSlider(value: st.size, min: 10, max: 32, width: 72, track: selected ? N.g38 : N.g26, onChanged: (v) => set((size: v, fav: st.fav))),
              SizedBox(width: 28, child: Text('${st.size.round()}', textAlign: TextAlign.right, style: T.value(N.g56).copyWith(fontSize: 10))),
              StarButton(on: st.fav, onChanged: (v) => set((size: st.size, fav: v))),
            ]),
          ),
        ),
      );
}

Widget _fontCase(BuildContext c) {
  const fams = ['Inter', 'Menlo', 'Georgia', 'Courier New', 'Times New Roman', 'Helvetica Neue'];
  final fam = c.knobs.object.dropdown<String>(label: 'Family', options: fams, initialOption: 'Inter', labelBuilder: (s) => s);
  final sample = c.knobs.string(label: 'Specimen', initialValue: 'Quick brown fox jumps');
  final size = c.knobs.double.slider(label: 'Initial size', initialValue: 16, min: 10, max: 32);
  return Hold<int>(
    initial: 0,
    builder: (context, sel, set) {
      final rows = [(fam, 'Sans · 18 styles'), ('Georgia', 'Serif · 8 styles'), ('Menlo', 'Mono · 4 styles')];
      return Flex(direction: Axis.vertical, mainAxisSize: MainAxisSize.min, children: [
        for (var i = 0; i < rows.length; i++) _FontRow(family: rows[i].$1, meta: rows[i].$2, sample: sample, size: size, selected: sel == i, onTap: () => set(i)),
      ]);
    },
  );
}

// -------------------------------------------------------------- palette strip

Widget _paletteCase(BuildContext c) {
  final look = lookKnob(c);
  final size = c.knobs.double.slider(label: 'Swatch size', initialValue: 24, min: 16, max: 36);
  final gap = c.knobs.double.slider(label: 'Gap', initialValue: 4, min: 4, max: 12);
  final title = c.knobs.string(label: 'Title', initialValue: 'Used here');
  final gradients = c.knobs.boolean(label: 'Show gradients', initialValue: true);
  final colours = [for (final f in Fam.all) f.c, N.g95, N.g56, N.g20];
  return Hold<int>(
    initial: 0,
    builder: (context, sel, set) => Flex(direction: Axis.vertical, mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
      Flex(direction: Axis.horizontal, mainAxisAlignment: MainAxisAlignment.spaceBetween, children: [
        Text(title.toUpperCase(), style: T.micro(N.g56).copyWith(letterSpacing: .8)),
        Text(hexOf(colours[sel]), style: T.value(N.g76).copyWith(fontSize: 10)),
      ]),
      const SizedBox(height: 8),
      Wrap(spacing: gap, runSpacing: gap, children: [
        for (var i = 0; i < colours.length; i++)
          Press(
            onTap: () => set(i),
            builder: (context, hover, focus, _) => AnimatedContainer(
              duration: kFast,
              curve: kEase,
              padding: const EdgeInsets.all(2),
              decoration: BoxDecoration(borderRadius: BorderRadius.circular(look == Look.quiet ? 99 : 6), border: Border.all(color: i == sel ? N.g95 : (hover || focus ? N.g56 : const Color(0x00000000)))),
              child: Container(width: size, height: look == Look.glow ? size * 1.5 : size, decoration: BoxDecoration(color: colours[i], borderRadius: BorderRadius.circular(look == Look.quiet ? 99 : 4))),
            ),
          ),
        Press(
          builder: (context, hover, focus, _) => Padding(
            padding: const EdgeInsets.all(2),
            child: SizedBox(width: size, height: look == Look.glow ? size * 1.5 : size, child: DecoratedBox(decoration: BoxDecoration(color: hover ? N.g15 : const Color(0x00000000), borderRadius: BorderRadius.circular(look == Look.quiet ? 99 : 4), border: Border.all(color: focus ? N.g56 : N.g20)), child: Center(child: glyph(10, (cv, s) {
              final p = Paint()..color = hover ? N.g95 : N.g63..strokeWidth = 1.3..strokeCap = StrokeCap.round;
              cv.drawLine(Offset(0, s.height / 2), Offset(s.width, s.height / 2), p);
              cv.drawLine(Offset(s.width / 2, 0), Offset(s.width / 2, s.height), p);
            }, hover)))),
          ),
        ),
      ]),
      if (gradients) ...[
        const SizedBox(height: 14),
        Text('GRADIENTS', style: T.micro(N.g56).copyWith(letterSpacing: .8)),
        const SizedBox(height: 8),
        Wrap(spacing: gap, runSpacing: gap, children: [
          for (var i = 0; i < 2; i++) Container(width: size * 2, height: size * .7, decoration: BoxDecoration(borderRadius: BorderRadius.circular(look == Look.quiet ? 99 : 4), gradient: LinearGradient(colors: [Fam.all[i].c, Fam.all[(i + 2) % 6].c]))),
        ]),
      ],
    ]),
  );
}

// ------------------------------------------------------------------ drag ghost

class _DragScene extends StatefulWidget {
  const _DragScene({required this.count, required this.accepts, required this.opacity});
  final int count;
  final bool accepts;
  final double opacity;
  @override
  State<_DragScene> createState() => _DragSceneState();
}

class _DragSceneState extends State<_DragScene> {
  static const _slotX = 300.0, _slotW = 196.0, _slotH = 56.0, _slotY = 24.0, _slotGap = 16.0;
  Offset _pos = const Offset(440, 140);

  Rect _slot(int i) => Rect.fromLTWH(_slotX, _slotY + i * (_slotH + _slotGap), _slotW, _slotH);
  int? get _over {
    for (var i = 0; i < 3; i++) {
      if (_slot(i).contains(_pos)) return i;
    }
    return null;
  }

  @override
  Widget build(BuildContext context) {
    final over = _over;
    final a = sampleAssets[1];
    return GestureDetector(
      behavior: HitTestBehavior.opaque,
      onPanDown: (d) => setState(() => _pos = d.localPosition),
      onPanUpdate: (d) => setState(() => _pos = d.localPosition),
      child: MouseRegion(
        cursor: SystemMouseCursors.grabbing,
        child: Stack(clipBehavior: Clip.none, children: [
          Positioned.fill(child: DecoratedBox(decoration: BoxDecoration(color: N.g10, borderRadius: BorderRadius.circular(6), border: Border.all(color: N.g20)))),
          Positioned(left: 24, top: 24, child: Opacity(opacity: .4, child: _Tile(a, width: _ghostW))),
          Positioned(left: 24, top: 130, child: _Tile(sampleAssets[0], width: _ghostW)),
          Positioned(left: 144, top: 24, child: _Tile(sampleAssets[5], width: _ghostW)),
          for (var i = 0; i < 3; i++)
            Positioned.fromRect(
              rect: _slot(i),
              child: AnimatedContainer(
                duration: kFast,
                curve: kEase,
                alignment: Alignment.topLeft,
                padding: const EdgeInsets.fromLTRB(12, 10, 0, 0),
                decoration: BoxDecoration(color: over == i && widget.accepts ? Role.selected.withValues(alpha: .12) : N.g13, borderRadius: BorderRadius.circular(6), border: Border.all(color: over == i ? (widget.accepts ? Role.selected : N.g44) : N.g20)),
                child: Text(over == i ? (widget.accepts ? 'Drop to add to Layer ${i + 1}' : 'Layer ${i + 1} is locked') : 'Layer ${i + 1}', style: T.label(over == i && widget.accepts ? N.g95 : N.g56)),
              ),
            ),
          Positioned(left: 24, bottom: 16, child: Text('Press and drag anywhere', style: T.label(N.g44))),
          Positioned(
            left: _pos.dx + 8,
            top: _pos.dy + 8,
            child: IgnorePointer(
              child: Opacity(
                opacity: widget.opacity,
                child: Transform.rotate(
                  angle: -.04,
                  child: Stack(clipBehavior: Clip.none, children: [
                    if (widget.count > 2) Positioned(left: 8, top: 8, child: _ghostCard(a, _ghostW, flat: true)),
                    if (widget.count > 1) Positioned(left: 4, top: 4, child: _ghostCard(a, _ghostW, flat: true)),
                    _ghostCard(a, _ghostW),
                    if (widget.count > 1) Positioned(right: -8, top: -8, child: Container(constraints: const BoxConstraints(minWidth: 18), height: 18, padding: const EdgeInsets.symmetric(horizontal: 5), alignment: Alignment.center, decoration: BoxDecoration(color: C.mode, borderRadius: BorderRadius.circular(9)), child: Text('${widget.count}', style: T.value(N.g100).copyWith(fontSize: 10)))),
                  ]),
                ),
              ),
            ),
          ),
        ]),
      ),
    );
  }
}

Widget _ghostCard(Asset a, double w, {bool flat = false}) => Container(
      width: w,
      height: w * .625,
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: flat ? N.g38 : N.g63),
        color: N.g13,
        boxShadow: flat ? null : [BoxShadow(color: N.g00.withValues(alpha: .5), blurRadius: 16, offset: const Offset(0, 6))], // floating: the one shadow
      ),
      child: ClipRRect(borderRadius: BorderRadius.circular(5), child: flat ? null : Stack(fit: StackFit.expand, children: [Art(a), Positioned(left: 4, top: 4, child: TypeBadge(a.kind))])),
    );

Widget _dragCase(BuildContext c) => _DragScene(
      count: c.knobs.int.slider(label: 'Items dragged', initialValue: 3, min: 1, max: 12),
      accepts: c.knobs.boolean(label: 'Target accepts', initialValue: true),
      opacity: c.knobs.double.slider(label: 'Ghost opacity', initialValue: .92, min: .5, max: 1),
    );

// ------------------------------------------------------------------ empty state

enum _Empty { folder, search, favorites }

Widget _emptyCase(BuildContext c) {
  final v = c.knobs.object.dropdown<_Empty>(label: 'Variant', options: _Empty.values, initialOption: _Empty.folder, labelBuilder: (e) => e.name);
  final folder = c.knobs.string(label: 'Folder', initialValue: 'Day 3');
  final action = c.knobs.boolean(label: 'Show action', initialValue: true);
  final (title, hint, button) = switch (v) {
    _Empty.folder => ('"$folder" is empty', 'Drop files here or import them from disk.', 'Import…'),
    _Empty.search => ('No matches', 'Try a shorter search, or clear the filters.', 'Clear filters'),
    _Empty.favorites => ('No favorites yet', 'Star an asset or preset and it will show up here.', 'Browse bundled'),
  };
  return Container(
    decoration: BoxDecoration(color: N.g10, borderRadius: BorderRadius.circular(6), border: Border.all(color: N.g20)),
    alignment: Alignment.center,
    child: Flex(direction: Axis.vertical, mainAxisSize: MainAxisSize.min, children: [
      glyph(40, (cv, s) {
        switch (v) {
          case _Empty.folder:
            drawFolder(cv, s, N.g38);
          case _Empty.search:
            final p = Paint()..color = N.g38..style = PaintingStyle.stroke..strokeWidth = 1.5..strokeCap = StrokeCap.round;
            cv.drawCircle(Offset(s.width * .44, s.height * .44), s.width * .28, p);
            cv.drawLine(Offset(s.width * .66, s.height * .66), Offset(s.width * .88, s.height * .88), p);
          case _Empty.favorites:
            drawStar(cv, s, N.g38, false);
        }
      }, v),
      const SizedBox(height: 14),
      Text(title, style: T.title(N.g91)),
      const SizedBox(height: 8),
      Padding(padding: const EdgeInsets.symmetric(horizontal: 32), child: Text(hint, textAlign: TextAlign.center, style: T.label(N.g56).copyWith(height: 1.4))),
      if (action) ...[const SizedBox(height: 16), QuietButton(button, primary: v == _Empty.folder)],
    ]),
  );
}

// ------------------------------------------------------------ import drop zone

class _DropZone extends StatelessWidget {
  const _DropZone({required this.dragging, required this.look, required this.files});
  final bool dragging;
  final Look look;
  final int files;

  @override
  Widget build(BuildContext context) => Press(
        builder: (context, hover, focus, _) {
          final active = dragging;
          final lit = active || hover || focus;
          final col = active ? C.mode : (lit ? N.g56 : N.g38);
          return AnimatedContainer(
            duration: kFast,
            curve: kEase,
            decoration: BoxDecoration(color: active ? Role.selected.withValues(alpha: .08) : (hover ? N.g13 : const Color(0x00000000)), borderRadius: BorderRadius.circular(6), border: Border.all(color: active && look != Look.quiet ? const Color(0x00000000) : (active ? Role.selected : (focus ? N.g56 : (hover ? N.g26 : N.g20))))),
            child: CustomPaint(
              painter: !active ? null : (look == Look.concept ? DashedBox(col) : (look == Look.glow ? _Corners(col) : null)),
              child: Center(
                child: Flex(direction: Axis.vertical, mainAxisSize: MainAxisSize.min, children: [
                  glyph(24, (cv, s) {
                    final p = Paint()..color = active ? Role.selected : (lit ? N.g76 : N.g56)..style = PaintingStyle.stroke..strokeWidth = 1.5..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round;
                    final x = s.width / 2;
                    cv.drawLine(Offset(x, 2), Offset(x, s.height * .62), p);
                    cv.drawPath(Path()..moveTo(x - 5, s.height * .46)..lineTo(x, s.height * .64)..lineTo(x + 5, s.height * .46), p);
                    cv.drawPath(Path()..moveTo(3, s.height * .74)..lineTo(3, s.height - 2)..lineTo(s.width - 3, s.height - 2)..lineTo(s.width - 3, s.height * .74), p);
                  }, [active, lit]),
                  const SizedBox(height: 10),
                  Text(active ? 'Release to import' : 'Drop files to import', style: T.name(active ? N.g100 : N.g91)),
                  const SizedBox(height: 6),
                  Text(active ? '$files files · ${files * 42} MB' : 'or click to browse · MP4  MOV  WAV  GLB  EXR', style: T.value(N.g56).copyWith(fontSize: 10)),
                ]),
              ),
            ),
          );
        },
      );
}

class _Corners extends CustomPainter {
  const _Corners(this.color);
  final Color color;
  @override
  void paint(Canvas canvas, Size size) {
    final p = Paint()..color = color..style = PaintingStyle.stroke..strokeWidth = 1.5..strokeCap = StrokeCap.round;
    const l = 14.0, r = 6.0, o = 1.0;
    final w = size.width, h = size.height;
    canvas.drawPath(Path()..moveTo(o, l)..lineTo(o, r)..quadraticBezierTo(o, o, r, o)..lineTo(l, o), p);
    canvas.drawPath(Path()..moveTo(w - l, o)..lineTo(w - r, o)..quadraticBezierTo(w - o, o, w - o, r)..lineTo(w - o, l), p);
    canvas.drawPath(Path()..moveTo(w - o, h - l)..lineTo(w - o, h - r)..quadraticBezierTo(w - o, h - o, w - r, h - o)..lineTo(w - l, h - o), p);
    canvas.drawPath(Path()..moveTo(l, h - o)..lineTo(r, h - o)..quadraticBezierTo(o, h - o, o, h - r)..lineTo(o, h - l), p);
  }

  @override
  bool shouldRepaint(_Corners o) => o.color != color;
}

Widget _dropCase(BuildContext c) => _DropZone(
      dragging: c.knobs.boolean(label: 'Dragging over', initialValue: false),
      look: lookKnob(c),
      files: c.knobs.int.slider(label: 'Files', initialValue: 3, min: 1, max: 40),
    );

// --------------------------------------------------------------- details card
