import 'dart:ui' show lerpDouble;

import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import '../../kit.dart';
import '../../parts/controls.dart';
import '../../tokens.dart';
import 'navigation_parts.dart';

// set: navigation. Floating things (menu, popover, palette, dialog) take the one shadow; everything else is grey level.
// [open] sizes no token names: tab height 33, header height 28, menu row 24, popover arrow 7 x 14, palette width 520.
const _tabH = 33.0, _headH = 28.0, _menuRow = 24.0, _arrowT = 7.0, _arrowW = 14.0; // [open]

WidgetbookComponent navigationSet() => WidgetbookComponent(name: 'navigation', useCases: [
      uc('Tab strip (underline)', (c) {
        final n = c.knobs.int.slider(label: 'Tabs', initialValue: 4, min: 2, max: 6);
        final sel = c.knobs.int.slider(label: 'Selected', initialValue: 0, min: 0, max: 5);
        final look = lookKnob(c);
        return _TabsUnderline(key: ValueKey('$n$sel${look.name}'), count: n, initial: sel.clamp(0, n - 1), look: look);
      }, width: 520),
      uc('Tab strip (pill)', (c) {
        final n = c.knobs.int.slider(label: 'Tabs', initialValue: 4, min: 2, max: 6);
        final sel = c.knobs.int.slider(label: 'Selected', initialValue: 1, min: 0, max: 5);
        final look = lookKnob(c);
        return _TabsPill(key: ValueKey('$n$sel${look.name}'), count: n, initial: sel.clamp(0, n - 1), look: look);
      }, width: 520),
      uc('Panel header', (c) {
        final title = c.knobs.string(label: 'Title', initialValue: 'Layers');
        final count = c.knobs.int.slider(label: 'Count', initialValue: 12, min: 0, max: 99);
        final open = c.knobs.boolean(label: 'Expanded', initialValue: true);
        final actions = c.knobs.boolean(label: 'Actions', initialValue: true);
        final active = c.knobs.boolean(label: 'Focused panel', initialValue: true);
        final look = lookKnob(c);
        return _PanelHeader(key: ValueKey('$open'), title: title, count: count, initialOpen: open, actions: actions, active: active, look: look);
      }, width: 320),
      uc('Dock tab (close + grip)', (c) {
        final sel = c.knobs.int.slider(label: 'Selected', initialValue: 0, min: 0, max: 2);
        final dragging = c.knobs.boolean(label: 'Dragging tab 2', initialValue: false);
        final dirty = c.knobs.boolean(label: 'Modified dot on tab 1', initialValue: true);
        final look = lookKnob(c);
        return _DockTabs(key: ValueKey('$sel$dragging$dirty${look.name}'), initial: sel, dragging: dragging, dirty: dirty, look: look);
      }, width: 560),
      uc('Breadcrumb', (c) {
        final depth = c.knobs.int.slider(label: 'Depth', initialValue: 6, min: 2, max: 7);
        final collapse = c.knobs.boolean(label: 'Collapse middle', initialValue: true);
        return _Breadcrumb(key: ValueKey('$depth$collapse'), depth: depth, collapse: collapse);
      }, width: 560),
      uc('Tree view', (c) {
        final guides = c.knobs.boolean(label: 'Indent guides', initialValue: true);
        final all = c.knobs.boolean(label: 'Expand all', initialValue: true);
        final sel = c.knobs.int.slider(label: 'Selected row', initialValue: 3, min: -1, max: 8);
        final look = lookKnob(c);
        return _Tree(key: ValueKey('$all$sel${look.name}'), guides: guides, expandAll: all, initialSel: sel, look: look);
      }, width: 280),
      uc('List row', (c) {
        final meta = c.knobs.boolean(label: 'Meta', initialValue: true);
        final sel = c.knobs.int.slider(label: 'Selected', initialValue: 1, min: -1, max: 3);
        final disabled = c.knobs.boolean(label: 'Last row disabled', initialValue: true);
        final look = lookKnob(c);
        return _Rows(key: ValueKey('$sel${look.name}'), meta: meta, initial: sel, disabledLast: disabled, look: look);
      }, width: 320),
      uc('Section header', (c) {
        final label = c.knobs.string(label: 'Label', initialValue: 'Transform');
        final collapsed = c.knobs.boolean(label: 'First collapsed', initialValue: false);
        final count = c.knobs.int.slider(label: 'Count', initialValue: 4, min: 0, max: 20);
        final look = lookKnob(c);
        return Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          _Section(key: ValueKey('$collapsed'), label: label, count: count, initialOpen: !collapsed, look: look),
          const SizedBox(height: 12),
          _Section(label: 'Appearance', count: 0, initialOpen: false, look: look),
        ]);
      }, width: 320),
      uc('Context menu', (c) {
        final pasteOff = c.knobs.boolean(label: 'Paste disabled', initialValue: true);
        final shortcuts = c.knobs.boolean(label: 'Shortcuts', initialValue: true);
        final checked = c.knobs.boolean(label: 'Lock layer checked', initialValue: true);
        final sub = c.knobs.boolean(label: 'Submenu open', initialValue: true);
        final look = lookKnob(c);
        return _ContextMenu(pasteOff: pasteOff, shortcuts: shortcuts, checked: checked, subOpen: sub, look: look);
      }, width: 480),
      uc('Popover with arrow', (c) {
        final p = c.knobs.object.dropdown<String>(label: 'Placement', options: const ['top', 'bottom', 'left', 'right'], initialOption: 'top', labelBuilder: (s) => s);
        final a = c.knobs.double.slider(label: 'Arrow position', initialValue: .5, min: 0, max: 1);
        return _PopoverDemo(placement: p, align: a);
      }, width: 400),
      uc('Command palette', (c) {
        final q = c.knobs.string(label: 'Query', initialValue: '');
        final sel = c.knobs.int.slider(label: 'Selected', initialValue: 0, min: 0, max: 7);
        final look = lookKnob(c);
        return _Palette(query: q, initial: sel, look: look);
      }, width: 520),
      uc('Sidebar nav item', (c) {
        final collapsed = c.knobs.boolean(label: 'Collapsed', initialValue: false);
        final sel = c.knobs.int.slider(label: 'Selected', initialValue: 1, min: 0, max: 4);
        final badges = c.knobs.boolean(label: 'Badges', initialValue: true);
        final look = lookKnob(c);
        return Align(alignment: Alignment.centerLeft, child: _Sidebar(key: ValueKey('$sel${look.name}'), collapsed: collapsed, initial: sel, badges: badges, look: look));
      }, width: 300),
      uc('Accordion group', (c) {
        final multi = c.knobs.boolean(label: 'Allow several open', initialValue: false);
        final first = c.knobs.int.slider(label: 'Open section', initialValue: 0, min: 0, max: 2);
        final look = lookKnob(c);
        return _Accordion(key: ValueKey('$multi$first${look.name}'), multi: multi, first: first, look: look);
      }, width: 320),
      uc('Dialog sheet', (c) {
        final title = c.knobs.string(label: 'Title', initialValue: 'Delete 3 layers?');
        final body = c.knobs.string(label: 'Body', initialValue: 'Their keyframes and relations are removed with them. You can undo this with Cmd+Z.');
        final destructive = c.knobs.boolean(label: 'Destructive', initialValue: true);
        final three = c.knobs.boolean(label: 'Third button', initialValue: false);
        final enabled = c.knobs.boolean(label: 'Primary enabled', initialValue: true);
        return _Dialog(title: title, body: body, destructive: destructive, three: three, enabled: enabled);
      }, width: 440),
    ]);

// ---------------------------------------------------------------- tabs

class _TabsUnderline extends StatefulWidget {
  const _TabsUnderline({super.key, required this.count, required this.initial, required this.look});
  final int count, initial;
  final Look look;
  @override
  State<_TabsUnderline> createState() => _TabsUnderlineState();
}

class _TabsUnderlineState extends State<_TabsUnderline> {
  late int _i = widget.initial;
  static const _labels = ['Browser', 'Assets', 'Effects', 'Library', 'History', 'Notes'];
  @override
  Widget build(BuildContext context) {
    final acc = accOf(widget.look);
    return SizedBox(
      height: _tabH,
      child: Stack(children: [
        const Positioned(left: 0, right: 0, bottom: 0, height: 1, child: ColoredBox(color: N.g20)),
        Row(children: [
          for (var k = 0; k < widget.count; k++)
            Hot(
              onTap: () => setState(() => _i = k),
              builder: (c, h, d) {
                final sel = k == _i;
                return AnimatedContainer(
                  duration: kFast,
                  curve: kEase,
                  height: _tabH,
                  padding: const EdgeInsets.symmetric(horizontal: 14),
                  decoration: BoxDecoration(gradient: widget.look == Look.glow && sel ? LinearGradient(begin: Alignment.topCenter, end: Alignment.bottomCenter, colors: [clear(), acc.withValues(alpha: .14)]) : null),
                  child: Stack(alignment: Alignment.center, children: [
                    Row(mainAxisSize: MainAxisSize.min, children: [
                      AnimatedDefaultTextStyle(duration: kFast, curve: kEase, style: T.name(sel ? N.g95 : d ? N.g76 : h ? N.g91 : N.g63), child: Text(_labels[k])),
                      if (k == 1) ...[const SizedBox(width: 6), const CountChip(8)],
                    ]),
                    Positioned(left: 0, right: 0, bottom: 0, height: 1, child: AnimatedContainer(duration: kFast, curve: kEase, color: sel ? (widget.look == Look.quiet ? clear() : Role.selected) : (h ? N.g38 : clear()))),
                  ]),
                );
              },
            ),
        ]),
      ]),
    );
  }
}

class _TabsPill extends StatefulWidget {
  const _TabsPill({super.key, required this.count, required this.initial, required this.look});
  final int count, initial;
  final Look look;
  @override
  State<_TabsPill> createState() => _TabsPillState();
}

class _TabsPillState extends State<_TabsPill> {
  late int _i = widget.initial;
  static const _labels = ['Stage', 'Timeline', 'Graph', 'Mixer', 'Render', 'Notes'];
  @override
  Widget build(BuildContext context) {
    final look = widget.look;
    final selBg = look == Look.glow ? C.mode.withValues(alpha: .22) : look == Look.quiet ? N.g15 : N.g20;
    return Align(
      alignment: Alignment.centerLeft,
      child: Container(
        height: 30,
        padding: const EdgeInsets.all(2),
        decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(8), border: Border.all(color: N.g15)),
        child: Row(mainAxisSize: MainAxisSize.min, children: [
          for (var k = 0; k < widget.count; k++)
            Hot(
              onTap: () => setState(() => _i = k),
              builder: (c, h, d) {
                final sel = k == _i;
                return AnimatedContainer(
                  duration: kFast,
                  curve: kEase,
                  padding: const EdgeInsets.symmetric(horizontal: 14),
                  alignment: Alignment.center,
                  decoration: BoxDecoration(color: sel ? selBg : (d ? N.g15 : h ? N.g13 : clear()), borderRadius: BorderRadius.circular(6)),
                  child: AnimatedDefaultTextStyle(duration: kFast, curve: kEase, style: T.name(sel ? (look == Look.glow ? N.g100 : N.g95) : h ? N.g91 : N.g56), child: Text(_labels[k])),
                );
              },
            ),
        ]),
      ),
    );
  }
}

// ---------------------------------------------------------------- panel header

class _PanelHeader extends StatefulWidget {
  const _PanelHeader({super.key, required this.title, required this.count, required this.initialOpen, required this.actions, required this.active, required this.look});
  final String title;
  final int count;
  final bool initialOpen, actions, active;
  final Look look;
  @override
  State<_PanelHeader> createState() => _PanelHeaderState();
}

class _PanelHeaderState extends State<_PanelHeader> {
  late bool _open = widget.initialOpen;
  @override
  Widget build(BuildContext context) {
    final w = widget;
    final glow = w.look == Look.glow && w.active;
    return SizedBox(width: double.infinity, child: Container(
      decoration: BoxDecoration(color: N.g10, borderRadius: BorderRadius.circular(6), border: Border.all(color: N.g20)),
      clipBehavior: Clip.antiAlias,
      child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        Hot(
          onTap: () => setState(() => _open = !_open),
          builder: (c, h, d) => AnimatedContainer(
            duration: kFast,
            curve: kEase,
            width: double.infinity,
            height: _headH,
            padding: const EdgeInsets.only(left: 8, right: 8),
            decoration: BoxDecoration(
              color: h ? N.g15 : N.g13,
              border: Border(top: BorderSide(color: glow ? Role.selected : clear(), width: 2)),
            ),
            child: Row(children: [
              AnimatedRotation(turns: _open ? 0 : -.25, duration: kFast, curve: kEase, child: Glyph(G.caretD, color: h ? N.g95 : N.g56)),
              const SizedBox(width: 8),
              Expanded(child: Row(children: [
                Flexible(child: One(w.title.toUpperCase(), T.micro(w.look == Look.quiet && !w.active ? N.g56 : w.active ? N.g95 : N.g76).copyWith(letterSpacing: .6))),
                if (w.count > 0) ...[const SizedBox(width: 8), CountChip(w.count, color: glow ? C.mode : N.g56)],
              ])),
              Text('00:12', style: T.value(N.g56).copyWith(fontSize: 10)),
              if (w.actions) ...[const SizedBox(width: 8), const IconBtn(G.plus), const SizedBox(width: 2), const IconBtn(G.dots)],
            ]),
          ),
        ),
        const SizedBox(height: 1, child: ColoredBox(color: N.g20)),
        AnimatedSize(
          duration: kFast,
          curve: kEase,
          alignment: Alignment.topCenter,
          child: _open
              ? Container(height: 32, padding: const EdgeInsets.symmetric(horizontal: 12), alignment: Alignment.centerLeft, child: Row(children: [Expanded(child: Text('Dot field', style: T.name(N.g91))), Text('Layer', style: T.label(N.g56))]))
              : const SizedBox(width: double.infinity),
        ),
      ]),
    ));
  }
}

// ---------------------------------------------------------------- dock tab

class _DockTabs extends StatefulWidget {
  const _DockTabs({super.key, required this.initial, required this.dragging, required this.dirty, required this.look});
  final int initial;
  final bool dragging, dirty;
  final Look look;
  @override
  State<_DockTabs> createState() => _DockTabsState();
}

class _DockTabsState extends State<_DockTabs> {
  late final _tabs = ['Timeline', 'Graph', 'Mixer'];
  late int _sel = widget.initial;

  Widget _tab(int k) {
    final look = widget.look;
    final acc = accOf(look);
    final lifted = widget.dragging && k == 1;
    return Hot(
      onTap: () => setState(() => _sel = k),
      builder: (c, h, d) {
        final sel = k == _sel;
        final modified = widget.dirty && k == 0;
        final showClose = h || sel;
        return Transform.translate(
          offset: Offset(0, lifted ? -3 : 0),
          child: AnimatedContainer(
            duration: kFast,
            curve: kEase,
            height: 30,
            padding: const EdgeInsets.only(left: 4, right: 6),
            decoration: BoxDecoration(
              color: lifted ? N.g20 : clear(),
              borderRadius: lifted ? BorderRadius.circular(6) : null,
              gradient: look == Look.glow && sel && !lifted ? LinearGradient(begin: Alignment.topCenter, end: Alignment.bottomCenter, colors: [clear(), acc.withValues(alpha: .14)]) : null,
              border: Border(bottom: BorderSide(color: sel ? (look == Look.quiet ? clear() : Role.selected) : (h ? N.g38 : clear()), width: 1)),
              boxShadow: lifted ? floatShadow() : null,
            ),
            child: Row(mainAxisSize: MainAxisSize.min, children: [
              AnimatedOpacity(duration: kFast, curve: kEase, opacity: h || lifted || sel ? 1 : 0, child: Glyph(G.grip, color: N.g56)),
              const SizedBox(width: 2),
              AnimatedDefaultTextStyle(duration: kFast, curve: kEase, style: T.name(sel || lifted ? N.g95 : h ? N.g91 : N.g56), child: Text(_tabs[k])),
              if (modified) ...[const SizedBox(width: 6), const DecoratedBox(decoration: BoxDecoration(color: N.g63, shape: BoxShape.circle), child: SizedBox(width: 6, height: 6))],
              const SizedBox(width: 6),
              SizedBox(
                width: 16,
                height: 16,
                child: AnimatedOpacity(
                        duration: kFast,
                        curve: kEase,
                        opacity: showClose ? 1 : 0,
                        child: Hot(
                          onTap: () => setState(() {
                            _tabs.removeAt(k);
                            _sel = _sel.clamp(0, _tabs.length - 1);
                          }),
                          builder: (c, h2, d2) => Container(alignment: Alignment.center, decoration: BoxDecoration(color: d2 ? N.g26 : h2 ? N.g20 : clear(), borderRadius: BorderRadius.circular(4)), child: Glyph(G.close, size: 10, color: h2 ? N.g95 : N.g56)),
                        ),
                      ),
              ),
            ]),
          ),
        );
      },
    );
  }

  @override
  Widget build(BuildContext context) => Container(
        height: 34,
        padding: const EdgeInsets.only(left: 4, top: 4),
        decoration: const BoxDecoration(color: N.g07, border: Border(bottom: BorderSide(color: N.g20))),
        child: Row(crossAxisAlignment: CrossAxisAlignment.end, children: [
          for (var k = 0; k < _tabs.length; k++) ...[
            _tab(k),
            if (k < _tabs.length - 1) const SizedBox(width: 8),
            if (widget.dragging && k == 2) Container(width: 2, height: 22, margin: const EdgeInsets.symmetric(horizontal: 1), decoration: BoxDecoration(color: accOf(widget.look), borderRadius: BorderRadius.circular(1))),
          ],
          const SizedBox(width: 4),
          const Padding(padding: EdgeInsets.only(bottom: 4), child: IconBtn(G.plus)),
        ]),
      );
}

// ---------------------------------------------------------------- breadcrumb

class _Breadcrumb extends StatefulWidget {
  const _Breadcrumb({super.key, required this.depth, required this.collapse});
  final int depth;
  final bool collapse;
  @override
  State<_Breadcrumb> createState() => _BreadcrumbState();
}

class _BreadcrumbState extends State<_Breadcrumb> {
  static const _all = ['Project', 'Compositions', 'Intro', 'Layers', 'Title', 'Text', 'Fill'];
  late int _depth = widget.depth;
  Widget _sep() => const Padding(padding: EdgeInsets.symmetric(horizontal: 2), child: Glyph(G.chevR, size: 10, color: N.g44));
  Widget _seg(int k, {String? text}) {
    final last = k == _depth - 1;
    return Hot(
      enabled: !last,
      onTap: () => setState(() => _depth = k + 1),
      builder: (c, h, d) => AnimatedContainer(
        duration: kFast,
        curve: kEase,
        height: 22,
        padding: const EdgeInsets.symmetric(horizontal: 6),
        decoration: BoxDecoration(color: d ? N.g20 : h ? N.g15 : clear(), borderRadius: BorderRadius.circular(4)),
        child: Row(mainAxisSize: MainAxisSize.min, children: [
          if (k == 0 && text == null) ...[Glyph(G.folder, color: h ? N.g95 : N.g56), const SizedBox(width: 6)],
          Text(text ?? _all[k], style: T.name(last ? N.g95 : h ? N.g95 : N.g56)),
        ]),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final ids = [for (var k = 0; k < _depth; k++) k];
    final fold = widget.collapse && _depth > 4;
    final shown = fold ? [0, -1, _depth - 2, _depth - 1] : ids;
    return SingleChildScrollView(
      scrollDirection: Axis.horizontal,
      child: Row(mainAxisSize: MainAxisSize.min, children: [
        for (var j = 0; j < shown.length; j++) ...[
          if (j > 0) _sep(),
          shown[j] == -1 ? _seg(_depth - 3, text: '…') : _seg(shown[j]),
        ],
      ]),
    );
  }
}

// ---------------------------------------------------------------- tree

class _Nd {
  const _Nd(this.id, this.name, this.g, [this.kids = const [], this.meta = '']);
  final int id;
  final String name, meta;
  final G g;
  final List<_Nd> kids;
}

const _treeData = [
  _Nd(0, 'Intro', G.folder, [
    _Nd(1, 'Title', G.text, [_Nd(2, 'Fill', G.layer, [], 'rgb'), _Nd(3, 'Stagger', G.layer, [], '0.4')], 'T'),
    _Nd(4, 'Shapes', G.folder, [_Nd(5, 'Dot A', G.layer), _Nd(6, 'Dot B', G.layer), _Nd(7, 'Dot C', G.layer)]),
    _Nd(8, 'Background', G.layer),
  ]),
  _Nd(9, 'Outro', G.folder, [_Nd(10, 'Credits', G.text)]),
];

class _Tree extends StatefulWidget {
  const _Tree({super.key, required this.guides, required this.expandAll, required this.initialSel, required this.look});
  final bool guides, expandAll;
  final int initialSel;
  final Look look;
  @override
  State<_Tree> createState() => _TreeState();
}

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
