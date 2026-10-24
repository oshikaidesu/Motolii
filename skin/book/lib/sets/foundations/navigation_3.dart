// Part of navigation.dart (cut there by size, as it was).
part of 'navigation.dart';

class _Sidebar extends StatefulWidget {
  const _Sidebar({super.key, required this.collapsed, required this.initial, required this.badges, required this.look});
  final bool collapsed, badges;
  final int initial;
  final Look look;
  @override
  State<_Sidebar> createState() => _SidebarState();
}

class _SidebarState extends State<_Sidebar> {
  late int _sel = widget.initial;
  static const _items = [('Browser', G.folder, 0, ''), ('Layers', G.layer, 12, ''), ('Text', G.text, 0, ''), ('Preview', G.eye, 0, ''), ('Search', G.search, 0, '⌘K')];
  @override
  Widget build(BuildContext context) {
    const full = 204.0;
    final col = widget.collapsed;
    return AnimatedContainer(
      duration: kFast,
      curve: kEase,
      width: col ? 44 : 220,
      padding: const EdgeInsets.all(8),
      decoration: BoxDecoration(color: N.g10, borderRadius: BorderRadius.circular(8), border: Border.all(color: N.g20)),
      child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        SizedBox(height: 22, child: ClipRect(child: OverflowBox(alignment: Alignment.centerLeft, minWidth: full, maxWidth: full, child: Padding(padding: const EdgeInsets.only(left: 8), child: Text('PROJECT', style: T.micro(N.g44).copyWith(letterSpacing: .9)))))),
        for (var k = 0; k < _items.length; k++)
          Padding(
            padding: const EdgeInsets.only(bottom: 2),
            child: Hot(
              onTap: () => setState(() => _sel = k),
              builder: (c, h, d) {
                final sel = k == _sel;
                final it = _items[k];
                return RowSurface(
                  sel: sel,
                  hover: h,
                  look: widget.look,
                  height: 28,
                  child: ClipRect(
                    child: OverflowBox(
                      alignment: Alignment.centerLeft,
                      minWidth: full,
                      maxWidth: full,
                      child: Padding(
                        padding: const EdgeInsets.symmetric(horizontal: 8),
                        child: Row(children: [
                          Glyph(it.$2, size: 14, color: sel ? (widget.look == Look.glow ? Role.selected : N.g95) : h ? N.g91 : N.g56),
                          const SizedBox(width: 10),
                          Expanded(child: One(it.$1, T.name(sel ? N.g95 : h ? N.g91 : N.g76))),
                          if (widget.badges && it.$3 > 0) CountChip(it.$3),
                          if (it.$4.isNotEmpty) Kbd(it.$4),
                        ]),
                      ),
                    ),
                  ),
                );
              },
            ),
          ),
      ]),
    );
  }
}

// ---------------------------------------------------------------- accordion

class _Accordion extends StatefulWidget {
  const _Accordion({super.key, required this.multi, required this.first, required this.look});
  final bool multi;
  final int first;
  final Look look;
  @override
  State<_Accordion> createState() => _AccordionState();
}

class _AccordionState extends State<_Accordion> {
  late final Set<int> _open = {widget.first};
  final _vals = <String, double>{'Offset': .4, 'Random': .2, 'Spread': .6, 'Seed': .5, 'Lag': .3, 'Ease': .7};
  static const _secs = [('Stagger', Fam.stagger, ['Offset', 'Random']), ('Scatter', Fam.scatter, ['Spread', 'Seed']), ('Follow', Fam.follow, ['Lag', 'Ease'])];

  void _toggle(int k) => setState(() {
        if (_open.contains(k)) {
          _open.remove(k);
        } else {
          if (!widget.multi) _open.clear();
          _open.add(k);
        }
      });

  @override
  Widget build(BuildContext context) => Container(
        decoration: BoxDecoration(color: N.g10, borderRadius: BorderRadius.circular(6), border: Border.all(color: N.g20)),
        clipBehavior: Clip.antiAlias,
        child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          for (var k = 0; k < _secs.length; k++) ...[
            if (k > 0) const SizedBox(height: 1, child: ColoredBox(color: N.g20)),
            Hot(
              onTap: () => _toggle(k),
              builder: (c, h, d) {
                final open = _open.contains(k);
                return AnimatedContainer(
                  duration: kFast,
                  curve: kEase,
                  height: 32,
                  padding: const EdgeInsets.symmetric(horizontal: 10),
                  color: open ? N.g13 : h ? N.g13 : clear(),
                  child: Row(children: [
                    AnimatedRotation(turns: open ? 0 : -.25, duration: kFast, curve: kEase, child: Glyph(G.caretD, size: 10, color: open && widget.look == Look.glow ? C.mode : open ? N.g95 : N.g56)),
                    const SizedBox(width: 8),
                    Text(_secs[k].$1, style: T.name(open ? N.g95 : N.g76)),
                    const Spacer(),
                    if (!open) Text('${_secs[k].$3.length} values', style: T.label(N.g44)),
                    if (open && widget.look != Look.quiet) Container(width: 6, height: 6, decoration: BoxDecoration(color: _secs[k].$2.c, shape: BoxShape.circle)),
                  ]),
                );
              },
            ),
            AnimatedSize(
              duration: kFast,
              curve: kEase,
              alignment: Alignment.topCenter,
              child: _open.contains(k)
                  ? Padding(
                      padding: const EdgeInsets.fromLTRB(10, 4, 10, 10),
                      child: Column(children: [
                        for (final v in _secs[k].$3) Padding(padding: const EdgeInsets.only(top: 4), child: ValueWell(label: v, value: _vals[v]!, fam: _secs[k].$2, onChanged: (x) => setState(() => _vals[v] = x))),
                      ]),
                    )
                  : const SizedBox(width: double.infinity),
            ),
          ],
        ]),
      );
}

// ---------------------------------------------------------------- dialog

class _Dialog extends StatelessWidget {
  const _Dialog({required this.title, required this.body, required this.destructive, required this.three, required this.enabled});
  final String title, body;
  final bool destructive, three, enabled;
  @override
  Widget build(BuildContext context) => Container(
        decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(8), border: Border.all(color: N.g20), boxShadow: floatShadow()),
        clipBehavior: Clip.antiAlias,
        child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(20, 16, 12, 0),
            child: Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
              Expanded(child: Padding(padding: const EdgeInsets.only(top: 2), child: Text(title, style: T.title()))),
              const IconBtn(G.close),
            ]),
          ),
          Padding(padding: const EdgeInsets.fromLTRB(20, 8, 20, 20), child: Text(body, style: T.name(N.g63).copyWith(fontWeight: FontWeight.w400, height: 1.45))),
          Container(
            height: 56,
            padding: const EdgeInsets.symmetric(horizontal: 16),
            decoration: const BoxDecoration(color: N.g10, border: Border(top: BorderSide(color: N.g20))),
            child: Row(children: [
              if (three) const PBtn('Do not save'),
              const Spacer(),
              const PBtn('Cancel'),
              const SizedBox(width: 8),
              PBtn(destructive ? 'Delete' : 'Continue', primary: true, color: destructive ? Role.error : C.mode, enabled: enabled, onTap: () {}),
            ]),
          ),
        ]),
      );
}
