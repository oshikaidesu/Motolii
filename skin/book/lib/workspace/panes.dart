// The window's panes: the seats laid out by the off-the-shelf `docking` package, so a tab can be dragged onto another strip,
// split off to a side, and every divider resized. The lab's version of Motolii's DockWorkspace (ui/lib/workspace/dock_workspace.dart);
// no Detach, since a window of its own needs the native shell. Chrome comes from WsT, so both shades apply.
import 'package:docking/docking.dart';
import 'package:flutter/widgets.dart';

import '../forms/forms.dart';
import '../tokens.dart';
import 'dock_glyphs.dart';
import 'ws.dart';

/// One pane: its name (also its id), the mark on its tab, and how to build its body.
/// [iconOnly] tabs show the mark alone: for panes whose body already names them in its header, so a strip of many stays narrow.
class WsPane {
  const WsPane(this.name, this.glyph, this.build, {this.minSize = 160, this.iconOnly = false});
  final String name;
  final DockG? glyph;   // (none: the pane's first letter)
  final Widget Function() build;
  final double minSize;
  final bool iconOnly;
}

class WsPanes extends StatefulWidget {
  const WsPanes({super.key, required this.panes, required this.preset, this.shelves = const {}});
  final List<WsPane> panes;

  /// Panes that are browser shelves: the front one is [Ws.shelf], both ways (a tab click sets it, [Ws.route] brings its tab forward).
  final Set<String> shelves;

  /// The arrangement a fresh window starts with.
  final DockingArea Function(DockingItem Function(String name, {double? size, double? weight}) item) preset;
  @override
  State<WsPanes> createState() => _WsPanesState();
}

class _WsPanesState extends State<WsPanes> {
  late final _by = {for (final p in widget.panes) p.name: p};
  late final _layout = DockingLayout(root: widget.preset(_item));
  Ws? _ws;
  String? _shelf;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    final ws = WsScope.read(context);
    if (ws == _ws) return;
    _ws?.on[WsAspect.fronted]!.removeListener(_follow);
    _ws?.on[WsAspect.shelf]!.removeListener(_follow);
    _ws = ws..on[WsAspect.fronted]!.addListener(_follow)..on[WsAspect.shelf]!.addListener(_follow);   // (a shelf named by anyone, not only a tab: brought to the front)
    _shelf = ws.shelf;
  }

  @override
  void dispose() {
    _ws?.on[WsAspect.fronted]!.removeListener(_follow);
    _ws?.on[WsAspect.shelf]!.removeListener(_follow);
    super.dispose();
  }

  (String, int)? _fronted;
  void _follow() {
    if (_ws!.fronted case final f? when f != _fronted) {   // (a pane asked to the front)
      _fronted = f;
      _toFront(f.$1);
    }
    final s = _ws!.shelf;
    if (s == _shelf) return;
    _shelf = s;
    _toFront(s);
  }

  void _toFront(String s) {
    final tabs = _layout.findDockingTabsWithItem(s);
    if (tabs == null) return;
    final i = [for (var k = 0; k < tabs.childrenCount; k++) tabs.childAt(k).id].indexOf(s);
    if (i < 0 || tabs.selectedIndex == i) return;
    tabs.selectedIndex = i;
    _layout.rebuild();
  }

  void _picked(DockingItem item) {
    if (!widget.shelves.contains(item.id)) return;
    _shelf = item.id as String;
    _ws?.shelf = _shelf!;
  }

  DockingItem _item(String name, {double? size, double? weight}) {
    final p = _by[name]!;
    return DockingItem(
      id: name,
      name: p.iconOnly ? null : name,
      leading: (context, status) => Semantics(
        label: name,
        button: true,
        child: Padding(
          padding: EdgeInsets.only(right: p.iconOnly ? Shape.s(0) : Shape.s(5)),
          child: Forms.worn.paneMark(
            context,
            selected: status == TabStatus.selected,
            mark: p.glyph == null
                ? Text(p.name.characters.first, style: T.micro(status == TabStatus.selected ? Grey.g95 : Grey.g56))
                : DockGlyph(p.glyph!, size: Shape.z(12), color: status == TabStatus.selected ? Grey.g95 : Grey.g56),
          ),
        ),
      ),
      widget: WsSeat(child: p.build()),
      size: size,
      weight: weight,
      minimalSize: p.minSize,
      keepAlive: true,
      // contract: nothing reopens a closed pane yet, so none can be closed.
      closable: false,
      maximizable: false,
    );
  }

  @override
  Widget build(BuildContext context) => TabbedViewTheme(
    data: _tabs(),
    child: MultiSplitViewTheme(
      data: MultiSplitViewThemeData(
        dividerThickness: WsT.gutter * 2,
        dividerPainter: DividerPainters.background(color: WsT.ground, highlightedColor: WsT.accentInk),
      ),
      child: Docking(layout: _layout, onItemSelection: _picked, maximizableItem: false, maximizableTab: false, maximizableTabsArea: false),
    ),
  );
}

/// Tab strips in the window's manner: a quiet band on the gutter, the front tab wears the seat's body and an accent underline.
/// A form with [Forms.plainTabs] drops the cards, so its own mark is the only thing that says which pane is in front.
TabbedViewThemeData _tabs() {
  final t = Grey.light ? TabbedViewThemeData.minimalist() : TabbedViewThemeData.dark();
  final plain = Forms.worn.plainTabs;
  t.tabsArea
    ..color = WsT.ground
    ..border = null
    ..initialGap = 0
    ..middleGap = plain ? Shape.s(2) : WsT.gutter
    ..gapBottomBorder = BorderSide.none
    ..dropColor = WsT.accent.withValues(alpha: .35)
    ..normalButtonColor = Grey.g56
    ..hoverButtonColor = Grey.g91;
  t.tab
    ..padding = EdgeInsets.fromLTRB(Shape.s(plain ? 4 : 8), Shape.s(5), Shape.s(plain ? 4 : 8), Shape.s(5))   // (seven shelf tabs fit a dock squeezed to 229 by a 1280 window)
    ..margin = EdgeInsets.zero
    ..innerBottomBorder = null
    ..innerTopBorder = null
    ..textStyle = T.micro(Grey.g63).copyWith(fontWeight: FontWeight.w600, letterSpacing: .6)
    ..decoration = plain ? const BoxDecoration() : BoxDecoration(color: WsT.card)
    ..draggingDecoration = BoxDecoration(color: plain ? WsT.body : WsT.raised)
    ..normalButtonColor = Grey.g56
    ..hoverButtonColor = Grey.g91
    ..selectedStatus = (TabStatusThemeData()
      ..decoration = plain
          ? const BoxDecoration()
          : BoxDecoration(
              color: WsT.body,
              border: Border(bottom: BorderSide(color: WsT.accentInk, width: 2)),
            )
      ..fontColor = Grey.g95)
    ..highlightedStatus = (TabStatusThemeData()
      ..decoration = plain ? const BoxDecoration() : BoxDecoration(color: WsT.raised)
      ..fontColor = Grey.g91);
  t.contentArea
    ..decoration = BoxDecoration(color: WsT.body)
    ..padding = EdgeInsets.zero;
  return t;
}
