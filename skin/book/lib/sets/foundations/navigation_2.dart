// Part of navigation.dart (cut there by size, as it was).
part of 'navigation.dart';

class _TreeState extends State<_Tree> {
  late final Set<int> _open = widget.expandAll ? {0, 1, 4, 9} : {0};
  late int _sel = widget.initialSel;

  void _flat(List<_Nd> ns, int depth, List<(_Nd, int)> out) {
    for (final n in ns) {
      out.add((n, depth));
      if (n.kids.isNotEmpty && _open.contains(n.id)) _flat(n.kids, depth + 1, out);
    }
  }

  @override
  Widget build(BuildContext context) {
    final rows = <(_Nd, int)>[];
    _flat(_treeData, 0, rows);
    return Container(
      padding: const EdgeInsets.all(4),
      decoration: BoxDecoration(color: N.g10, borderRadius: BorderRadius.circular(6), border: Border.all(color: N.g20)),
      child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        for (final (n, depth) in rows)
          Hot(
            onTap: () => setState(() => _sel = n.id),
            builder: (c, h, d) {
              final sel = n.id == _sel;
              final has = n.kids.isNotEmpty;
              final open = _open.contains(n.id);
              return SizedBox(
                height: TL.height,
                child: Stack(children: [
                  RowSurface(
                    sel: sel,
                    hover: h,
                    look: widget.look,
                    height: TL.height,
                    padding: EdgeInsets.only(left: 4 + depth * 14.0, right: 8),
                    child: Row(children: [
                      GestureDetector(
                        behavior: HitTestBehavior.opaque,
                        onTap: has ? () => setState(() => open ? _open.remove(n.id) : _open.add(n.id)) : null,
                        child: SizedBox(width: 14, height: TL.height, child: has ? Center(child: AnimatedRotation(turns: open ? 0 : -.25, duration: kFast, curve: kEase, child: Glyph(G.caretD, size: 10, color: sel || h ? N.g95 : N.g56))) : null),
                      ),
                      const SizedBox(width: 2),
                      Glyph(n.g, color: sel ? N.g95 : N.g56),
                      const SizedBox(width: 6),
                      Expanded(child: One(n.name, T.name(sel ? N.g95 : h ? N.g91 : N.g76))),
                      if (n.meta.isNotEmpty) Text(n.meta, style: T.value(N.g44)),
                    ]),
                  ),
                  if (widget.guides)
                    for (var g = 0; g < depth; g++) Positioned(left: 4 + g * 14.0 + 6.5, top: 0, bottom: 0, width: 1, child: ColoredBox(color: N.g20.withValues(alpha: .7))),
                ]),
              );
            },
          ),
      ]),
    );
  }
}

// ---------------------------------------------------------------- list rows

class _Rows extends StatefulWidget {
  const _Rows({super.key, required this.meta, required this.initial, required this.disabledLast, required this.look});
  final bool meta, disabledLast;
  final int initial;
  final Look look;
  @override
  State<_Rows> createState() => _RowsState();
}

class _RowsState extends State<_Rows> {
  late int _sel = widget.initial;
  static const _data = [
    ('Title', G.text, '00:00 - 02:10', Fam.stagger),
    ('Dot field', G.layer, '00:12 - 03:00', Fam.scatter),
    ('Backdrop', G.layer, '00:00 - 05:00', null),
    ('Credits', G.text, '04:00 - 05:00', null),
  ];
  @override
  Widget build(BuildContext context) => Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        for (var k = 0; k < _data.length; k++)
          Builder(builder: (context) {
            final off = widget.disabledLast && k == 3;
            final r = _data[k];
            return Hot(
              enabled: !off,
              onTap: () => setState(() => _sel = k),
              builder: (c, h, d) {
                final sel = k == _sel;
                final show = h || sel;
                return RowSurface(
                  sel: sel,
                  hover: h,
                  look: widget.look,
                  height: 32,
                  padding: const EdgeInsets.only(left: 6, right: 6),
                  child: Row(children: [
                    Container(width: 20, height: 20, alignment: Alignment.center, child: Glyph(r.$2, color: off ? N.g38 : sel ? N.g95 : N.g63)),
                    const SizedBox(width: 8),
                    Expanded(child: One(r.$1, T.name(off ? N.g38 : sel ? N.g95 : N.g91))),
                    if (r.$4 != null) ...[Container(width: 6, height: 6, decoration: const BoxDecoration(color: N.g56, shape: BoxShape.circle)), const SizedBox(width: 8)],
                    if (widget.meta) Text(r.$3, style: T.value(off ? N.g38 : N.g56).copyWith(fontSize: 10)),
                    AnimatedOpacity(
                      duration: kFast,
                      curve: kEase,
                      opacity: show && !off ? 1 : 0,
                      child: const Row(mainAxisSize: MainAxisSize.min, children: [SizedBox(width: 6), IconBtn(G.eye), IconBtn(G.dots)]),
                    ),
                  ]),
                );
              },
            );
          }),
      ]);
}

// ---------------------------------------------------------------- section header

class _Section extends StatefulWidget {
  const _Section({super.key, required this.label, required this.count, required this.initialOpen, required this.look});
  final String label;
  final int count;
  final bool initialOpen;
  final Look look;
  @override
  State<_Section> createState() => _SectionState();
}

class _SectionState extends State<_Section> {
  late bool _open = widget.initialOpen;
  @override
  Widget build(BuildContext context) {
    final look = widget.look;
    return Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      Hot(
        onTap: () => setState(() => _open = !_open),
        builder: (c, h, d) => SizedBox(
          height: 24,
          child: Stack(children: [
            Row(children: [
              AnimatedRotation(turns: _open ? 0 : -.25, duration: kFast, curve: kEase, child: Glyph(G.caretD, size: 10, color: h ? N.g95 : N.g56)),
              const SizedBox(width: 6),
              if (look == Look.glow) ...[Container(width: 4, height: 4, decoration: BoxDecoration(color: _open ? C.mode : N.g44, shape: BoxShape.circle)), const SizedBox(width: 6)],
              Text(widget.label.toUpperCase(), style: T.micro(h || _open ? N.g76 : N.g63).copyWith(letterSpacing: .9)),
              if (widget.count > 0) ...[const SizedBox(width: 6), Text('${widget.count}', style: T.value(N.g44).copyWith(fontSize: 10))],
              if (look != Look.quiet) ...[const SizedBox(width: 8), const Expanded(child: SizedBox(height: 1, child: ColoredBox(color: N.g20)))] else const Spacer(),
            ]),
            Positioned(right: 0, top: 2, child: AnimatedOpacity(duration: kFast, curve: kEase, opacity: h ? 1 : 0, child: const ColoredBox(color: N.g07, child: IconBtn(G.plus)))),
          ]),
        ),
      ),
      AnimatedSize(
        duration: kFast,
        curve: kEase,
        alignment: Alignment.topCenter,
        child: _open
            ? Padding(
                padding: const EdgeInsets.only(left: 16, top: 4, bottom: 4),
                child: Column(children: [
                  for (final (k, v) in [('Position', '960, 540'), ('Scale', '100 %'), ('Opacity', '1.00')])
                    SizedBox(height: 22, child: Row(children: [Expanded(child: Text(k, style: T.label(N.g63))), Text(v, style: T.value(N.g91))])),
                ]),
              )
            : const SizedBox(width: double.infinity),
      ),
    ]);
  }
}

// ---------------------------------------------------------------- context menu

class _MI {
  const _MI(this.label, {this.key = '', this.sub = false, this.disabled = false, this.check, this.sep = false});
  final String label, key;
  final bool sub, disabled, sep;
  final bool? check;
}

class _MenuRows extends StatefulWidget {
  const _MenuRows({required this.items, required this.look, required this.shortcuts, this.pinned = -1, this.width = 224});
  final List<_MI> items;
  final Look look;
  final bool shortcuts;
  final int pinned;
  final double width;
  @override
  State<_MenuRows> createState() => _MenuRowsState();
}

class _MenuRowsState extends State<_MenuRows> {
  @override
  Widget build(BuildContext context) {
    final look = widget.look;
    return Container(
      width: widget.width,
      padding: const EdgeInsets.symmetric(vertical: 4, horizontal: 4),
      decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(6), border: Border.all(color: N.g20), boxShadow: floatShadow()),
      child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        for (var k = 0; k < widget.items.length; k++)
          widget.items[k].sep
              ? const Padding(padding: EdgeInsets.symmetric(vertical: 4, horizontal: 4), child: SizedBox(height: 1, child: ColoredBox(color: N.g20)))
              : Hot(
                  enabled: !widget.items[k].disabled,
                  onTap: () {},
                  builder: (c, h, d) {
                    final it = widget.items[k];
                    final on = (h || k == widget.pinned) && !it.disabled;
                    final fg = it.disabled ? N.g38 : on ? N.g95 : N.g91;
                    return Stack(children: [
                      AnimatedContainer(
                        duration: kFast,
                        curve: kEase,
                        height: _menuRow,
                        padding: const EdgeInsets.symmetric(horizontal: 6),
                        decoration: BoxDecoration(color: d ? N.g26 : on ? N.g20 : clear(), borderRadius: BorderRadius.circular(4)),
                        child: Row(children: [
                          SizedBox(width: 16, child: it.check == true ? Glyph(G.check, color: fg) : null),
                          Expanded(child: One(it.label, T.name(fg))),
                          if (it.sub)
                            RotatedBox(quarterTurns: 3, child: Glyph(G.sub, size: 10, color: fg))
                          else if (widget.shortcuts && it.key.isNotEmpty)
                            Text(it.key, style: T.value(it.disabled ? N.g38 : N.g56).copyWith(fontSize: 10)),
                        ]),
                      ),
                      Positioned(left: 0, top: 5, bottom: 5, width: 2, child: AnimatedOpacity(duration: kFast, curve: kEase, opacity: on ? 1 : 0, child: DecoratedBox(decoration: BoxDecoration(color: accOf(look), borderRadius: BorderRadius.circular(1))))),
                    ]);
                  },
                ),
      ]),
    );
  }
}

class _ContextMenu extends StatelessWidget {
  const _ContextMenu({required this.pasteOff, required this.shortcuts, required this.checked, required this.subOpen, required this.look});
  final bool pasteOff, shortcuts, checked, subOpen;
  final Look look;
  @override
  Widget build(BuildContext context) {
    final items = [
      const _MI('Cut', key: '⌘X'),
      const _MI('Copy', key: '⌘C'),
      _MI('Paste', key: '⌘V', disabled: pasteOff),
      const _MI('', sep: true),
      const _MI('Duplicate', key: '⌘D'),
      const _MI('Rename', key: '↵'),
      _MI('Lock layer', key: '⌘L', check: checked),
      const _MI('', sep: true),
      const _MI('Arrange', sub: true),
      const _MI('Add relation', sub: true),
      const _MI('', sep: true),
      const _MI('Delete', key: '⌫'),
    ];
    final arrange = items.indexWhere((e) => e.label == 'Arrange');
    var top = 0.0;
    for (var k = 0; k < arrange; k++) {
      top += items[k].sep ? 9 : _menuRow;
    }
    const subItems = [_MI('Bring to front', key: '⌘]'), _MI('Forward', key: '⌥⌘]'), _MI('Backward', key: '⌥⌘['), _MI('Send to back', key: '⌘[')];
    return Row(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
      _MenuRows(items: items, look: look, shortcuts: shortcuts, pinned: subOpen ? arrange : -1),
      if (subOpen) Padding(padding: EdgeInsets.only(top: top), child: Padding(padding: const EdgeInsets.only(left: 2), child: _MenuRows(items: subItems, look: look, shortcuts: shortcuts, width: 176))),
    ]);
  }
}

// ---------------------------------------------------------------- popover

class _PopPainter extends CustomPainter {
  _PopPainter(this.placement, this.align);
  final String placement;
  final double align;
  @override
  void paint(Canvas canvas, Size size) {
    final left = placement == 'right' ? _arrowT : 0.0;
    final top = placement == 'bottom' ? _arrowT : 0.0;
    final right = size.width - (placement == 'left' ? _arrowT : 0.0);
    final bottom = size.height - (placement == 'top' ? _arrowT : 0.0);
    final body = RRect.fromLTRBR(left, top, right, bottom, const Radius.circular(8));
    final hw = _arrowW / 2;
    final cx = lerpDouble(left + 8 + hw, right - 8 - hw, align)!;
    final cy = lerpDouble(top + 8 + hw, bottom - 8 - hw, align)!;
    final tri = Path();
    switch (placement) {
      case 'top':
        tri.addPolygon([Offset(cx - hw, bottom - 1), Offset(cx, bottom + _arrowT), Offset(cx + hw, bottom - 1)], true);
      case 'bottom':
        tri.addPolygon([Offset(cx - hw, top + 1), Offset(cx, top - _arrowT), Offset(cx + hw, top + 1)], true);
      case 'left':
        tri.addPolygon([Offset(right - 1, cy - hw), Offset(right + _arrowT, cy), Offset(right - 1, cy + hw)], true);
      default:
        tri.addPolygon([Offset(left + 1, cy - hw), Offset(left - _arrowT, cy), Offset(left + 1, cy + hw)], true);
    }
    final path = Path.combine(PathOperation.union, Path()..addRRect(body), tri);
    canvas.drawPath(path.shift(const Offset(0, 8)), Paint()..color = N.g00.withValues(alpha: .5)..maskFilter = const MaskFilter.blur(BlurStyle.normal, 12));
    canvas.drawPath(path, Paint()..color = N.g13);
    canvas.drawPath(path, Paint()..color = N.g20..style = PaintingStyle.stroke..strokeWidth = 1);
  }

  @override
  bool shouldRepaint(_PopPainter o) => o.placement != placement || o.align != align;
}

class _PopoverDemo extends StatefulWidget {
  const _PopoverDemo({required this.placement, required this.align});
  final String placement;
  final double align;
  @override
  State<_PopoverDemo> createState() => _PopoverDemoState();
}

class _PopoverDemoState extends State<_PopoverDemo> {
  double _v = .4;
  static const _w = 232.0, _h = 132.0;
  @override
  Widget build(BuildContext context) {
    final p = widget.placement;
    final horiz = p == 'top' || p == 'bottom';
    final pad = EdgeInsets.fromLTRB(12 + (p == 'right' ? _arrowT : 0), 12 + (p == 'bottom' ? _arrowT : 0), 12 + (p == 'left' ? _arrowT : 0), 12 + (p == 'top' ? _arrowT : 0));
    final pop = CustomPaint(
      painter: _PopPainter(p, widget.align),
      child: SizedBox(
        width: _w + (horiz ? 0 : _arrowT),
        height: _h + (horiz ? _arrowT : 0),
        child: Padding(
          padding: pad,
          child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            Text('Stagger', style: T.title()),
            const SizedBox(height: 6),
            Text('Offsets each copy in time, in order.', style: T.label(N.g63).copyWith(height: 1.3)),
            const SizedBox(height: 10),
            ValueWell(label: 'Delay', value: _v, fam: Fam.stagger, onChanged: (v) => setState(() => _v = v)),
            const Spacer(),
            const Row(mainAxisAlignment: MainAxisAlignment.end, children: [PBtn('Reset'), SizedBox(width: 8), PBtn('Apply', primary: true)]),
          ]),
        ),
      ),
    );
    final along = lerpDouble(8 + _arrowW / 2, (horiz ? _w : _h) - 8 - _arrowW / 2, widget.align)! - 14;
    final anchor = Container(width: 28, height: 28, alignment: Alignment.center, decoration: BoxDecoration(color: N.g20, borderRadius: BorderRadius.circular(6), border: Border.all(color: N.g44)), child: const Glyph(G.plus, color: N.g95));
    final gap = const SizedBox(width: 4, height: 4);
    return switch (p) {
      'top' => Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [pop, gap, Padding(padding: EdgeInsets.only(left: along), child: anchor)]),
      'bottom' => Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [Padding(padding: EdgeInsets.only(left: along), child: anchor), gap, pop]),
      'left' => Row(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [pop, gap, Padding(padding: EdgeInsets.only(top: along), child: anchor)]),
      _ => Row(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [Padding(padding: EdgeInsets.only(top: along), child: anchor), gap, pop]),
    };
  }
}

// ---------------------------------------------------------------- command palette

class _Cmd {
  const _Cmd(this.group, this.name, this.g, [this.keys = '', this.fam]);
  final String group, name, keys;
  final G g;
  final Fam? fam;
}

const _cmds = [
  _Cmd('Actions', 'New composition', G.plus, '⌘N'),
  _Cmd('Actions', 'Import footage', G.folder, '⌘I'),
  _Cmd('Actions', 'Add relation: Scatter', G.layer, '⌘R', Fam.scatter),
  _Cmd('Actions', 'Add relation: Stagger', G.layer, '', Fam.stagger),
  _Cmd('Actions', 'Split layer at playhead', G.layer, 'S'),
  _Cmd('Layers', 'Title', G.text),
  _Cmd('Layers', 'Dot field', G.layer),
  _Cmd('Layers', 'Backdrop', G.layer),
];

class _Palette extends StatefulWidget {
  const _Palette({required this.query, required this.initial, required this.look});
  final String query;
  final int initial;
  final Look look;
  @override
  State<_Palette> createState() => _PaletteState();
}

class _PaletteState extends State<_Palette> {
  int? _hover;
  @override
  Widget build(BuildContext context) {
    final q = widget.query.trim().toLowerCase();
    final res = [for (final c in _cmds) if (q.isEmpty || c.name.toLowerCase().contains(q)) c];
    final sel = res.isEmpty ? -1 : (_hover ?? widget.initial).clamp(0, res.length - 1);
    final rows = <Widget>[];
    String? group;
    for (var k = 0; k < res.length; k++) {
      final r = res[k];
      if (r.group != group) {
        group = r.group;
        rows.add(Padding(padding: EdgeInsets.fromLTRB(10, rows.isEmpty ? 4 : 10, 10, 4), child: Text(r.group.toUpperCase(), style: T.micro(N.g44).copyWith(letterSpacing: .9))));
      }
      final on = k == sel;
      rows.add(Hot(
        onTap: () {},
        onHover: (h) => setState(() => _hover = h ? k : null),
        builder: (c, h, d) => RowSurface(
          sel: on,
          hover: false,
          look: widget.look,
          height: 30,
          padding: const EdgeInsets.symmetric(horizontal: 8),
          child: Row(children: [
            Glyph(r.g, size: 14, color: r.fam?.c ?? (on ? N.g95 : N.g56)),
            const SizedBox(width: 10),
            Expanded(child: _Match(r.name, q, on)),
            for (final ch in r.keys.split('').where((e) => e.isNotEmpty)) ...[const SizedBox(width: 3), Kbd(ch)],
          ]),
        ),
      ));
    }
    Widget hint(String k, String t) => Row(mainAxisSize: MainAxisSize.min, children: [Kbd(k), const SizedBox(width: 6), Text(t, style: T.label(N.g56))]);
    return Container(
      decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(8), border: Border.all(color: N.g20), boxShadow: floatShadow()),
      clipBehavior: Clip.antiAlias,
      child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        Container(
          height: 44,
          padding: const EdgeInsets.symmetric(horizontal: 14),
          decoration: const BoxDecoration(border: Border(bottom: BorderSide(color: N.g20))),
          child: Row(children: [
            Glyph(G.search, size: 14, color: widget.look == Look.quiet ? N.g63 : C.mode),
            const SizedBox(width: 10),
            if (q.isEmpty) Text('Type a command or a layer name', style: T.name(N.g44).copyWith(fontSize: 13, fontWeight: FontWeight.w400)) else Text(widget.query, style: T.name(N.g95).copyWith(fontSize: 13, fontWeight: FontWeight.w400)),
            if (q.isNotEmpty) Container(width: 1, height: 16, margin: const EdgeInsets.only(left: 1), color: accOf(widget.look)),
            const Spacer(),
            const Kbd('esc'),
          ]),
        ),
        Padding(
          padding: const EdgeInsets.fromLTRB(6, 0, 6, 6),
          child: res.isEmpty ? SizedBox(height: 96, child: Center(child: Text('No results for "${widget.query}"', style: T.label(N.g56)))) : Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: rows),
        ),
        Container(
          height: 32,
          padding: const EdgeInsets.symmetric(horizontal: 14),
          decoration: const BoxDecoration(color: N.g10, border: Border(top: BorderSide(color: N.g20))),
          child: Row(children: [hint('↑↓', 'Navigate'), const SizedBox(width: 16), hint('↵', 'Run'), const Spacer(), Text('${res.length} results', style: T.label(N.g44))]),
        ),
      ]),
    );
  }
}

class _Match extends StatelessWidget {
  const _Match(this.text, this.q, this.on);
  final String text, q;
  final bool on;
  @override
  Widget build(BuildContext context) {
    final base = T.name(on ? N.g100 : N.g76).copyWith(fontWeight: FontWeight.w400);
    final hi = T.name(N.g100);
    final i = q.isEmpty ? -1 : text.toLowerCase().indexOf(q);
    if (i < 0) return One(text, T.name(on ? N.g95 : N.g91));
    return RichText(
      maxLines: 1,
      overflow: TextOverflow.ellipsis,
      text: TextSpan(children: [
        TextSpan(text: text.substring(0, i), style: base),
        TextSpan(text: text.substring(i, i + q.length), style: hi),
        TextSpan(text: text.substring(i + q.length), style: base),
      ]),
    );
  }
}

// ---------------------------------------------------------------- sidebar
