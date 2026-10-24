import 'dart:ui' show lerpDouble;

import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import '../../kit.dart';
import '../../parts/controls.dart';
import '../../tokens.dart';
import 'navigation_parts.dart';
part 'navigation_2.dart';
part 'navigation_3.dart';

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
