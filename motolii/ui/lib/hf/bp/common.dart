// Browser-family panels: visual prototype. Real Flutter layout, not reference coordinates.
// Only the dock's tab strip and two tiny controls are shared; every body decides its own folding.
import 'package:flutter/widgets.dart';

import '../metrics.dart';
import '../glyphs.dart';
import '../neutral.dart';

TextStyle sans(
  double s, {
  Color c = N.g86,
  FontWeight w = FontWeight.w400,
  double ls = 0,
}) => TextStyle(
  fontFamily: 'Inter',
  fontSize: s,
  color: c,
  fontWeight: w,
  letterSpacing: ls,
  height: 1.1,
);
TextStyle mono(double s, {Color c = Surface.muted, double ls = 0}) => TextStyle(
  fontFamily: 'Menlo',
  fontSize: s,
  color: c,
  letterSpacing: ls,
  height: 1.1,
);
TextStyle caps(double s, {Color c = Surface.muted}) =>
    sans(s, c: c, w: FontWeight.w500, ls: 0.8);

/// Present above a panel body that sits in a dock tab: the tab already names the panel,
/// so the panel's own header drops its name and icon and keeps its search, view and menu.
class DockedPanel extends InheritedWidget {
  const DockedPanel({super.key, required super.child});
  static bool of(BuildContext context) =>
      context.getInheritedWidgetOfExactType<DockedPanel>() != null;
  @override
  bool updateShouldNotify(DockedPanel oldWidget) => false;
}

class TabSpec {
  const TabSpec(this.name, this.glyph);
  final String name;
  final HG glyph;
}

const tabCreate = TabSpec('Create', HG.plus);
const tabEffects = TabSpec('Effects', HG.glow);
const tabColors = TabSpec('Colors', HG.color);
const tabFonts = TabSpec('Fonts', HG.text);
const tabMedia = TabSpec('Media', HG.image);

/// Tells a panel whether it shares its seat with others (a tab strip above it names it) or sits alone.
class PanelSeat extends InheritedWidget {
  const PanelSeat({super.key, required this.stacked, required super.child});
  final bool stacked;
  static bool stackedOf(BuildContext c) =>
      c.dependOnInheritedWidgetOfExactType<PanelSeat>()?.stacked ?? false;
  @override
  bool updateShouldNotify(PanelSeat o) => o.stacked != stacked;
}

/// A dock leaf as a fixture. Tabs are independent panels sharing a seat, not navigation: the strip only
/// exists when there is more than one, and then only the active tab keeps its word when crowded.
class Leaf extends StatelessWidget {
  const Leaf({
    super.key,
    required this.tabs,
    required this.active,
    required this.body,
    this.onTab,
    this.tabWrap,
    this.trailing,
  });
  final List<TabSpec> tabs;

  /// The front panel's own tools at the strip's right end (the Timeline's Split and Marker), where every editor puts
  /// them, instead of a band of the panel's own.
  final Widget? trailing;
  final int active;
  final Widget body;

  /// A tap on a tab. Without it the strip is a picture.
  final ValueChanged<int>? onTab;

  /// What a host adds around each tab (the Dock: drag to move/split, right click for the panel menu). The tab's look
  /// is the Leaf's own either way.
  final Widget Function(int index, Widget tab)? tabWrap;
  static double labelled(String n) => 34 + n.length * 6.1;
  @override
  Widget build(BuildContext context) {
    final stacked = tabs.length > 1;
    return Container(
      decoration: BoxDecoration(
        color: Surface.base,
        border: Border.all(color: Surface.divider),
        borderRadius: BorderRadius.circular(2),
      ),
      child: Column(
        children: [
          if (stacked || trailing != null)
            LayoutBuilder(
              builder: (_, c) {
                final need = tabs.fold<double>(
                  0,
                  (a, t) => a + labelled(t.name),
                );
                final fits = need <= c.maxWidth - 30 && c.maxWidth >= 110;
                return Container(
                  height: Surface.chromeRow,
                  decoration: const BoxDecoration(
                    border: Border(bottom: BorderSide(color: Surface.divider)),
                  ),
                  child: Row(
                    children: [
                      Expanded(
                        child: ClipRect(
                          child: SingleChildScrollView(
                            scrollDirection: Axis.horizontal,
                            physics: const NeverScrollableScrollPhysics(),
                            child: Row(
                              children: [
                                for (var i = 0; i < tabs.length; i++)
                                  (tabWrap ?? (_, t) => t)(i, GestureDetector(
                                    behavior: HitTestBehavior.opaque,
                                    onTap: onTab == null
                                        ? null
                                        : () => onTab!(i),
                                    child: _Tab(
                                      tabs[i],
                                      selected: i == active,
                                      single: false,
                                      compact:
                                          !fits &&
                                          (i != active || c.maxWidth < 110),
                                    ),
                                  )),
                              ],
                            ),
                          ),
                        ),
                      ),
                      if (trailing != null) Padding(padding: const EdgeInsets.only(right: Surface.inlineGap), child: trailing),
                      // (no ✕ here: it was painted with no gesture; a seat's panel is closed from its tab's menu)
                    ],
                  ),
                );
              },
            ),
          Expanded(
            child: PanelSeat(
              stacked: stacked,
              child: ClipRect(child: body),
            ),
          ),
        ],
      ),
    );
  }
}

class _Tab extends StatelessWidget {
  const _Tab(
    this.t, {
    required this.selected,
    required this.single,
    required this.compact,
  });
  final TabSpec t;
  final bool selected, single;
  final bool compact;
  @override
  Widget build(BuildContext context) => Container(
    height: Surface.chromeRow,
    padding: EdgeInsets.symmetric(horizontal: compact ? 6 : 8),
    decoration: BoxDecoration(
      color: (selected && !single) ? Surface.selected : null,
      border: const Border(right: BorderSide(color: Surface.dividerFine)),
    ),
    child: Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        SizedBox(
          width: 12,
          height: 12,
          child: CustomPaint(
            painter: HfTabGlyph(
              t.glyph,
              selected ? N.g95 : Surface.muted,
            ),
          ),
        ),
        if (!compact) ...[
          const SizedBox(width: 6),
          Text(t.name, softWrap: false, style: Dn.name(selected ? N.g95 : N.g63)),
        ],
      ],
    ),
  );
}

class HfTabGlyph extends CustomPainter {
  HfTabGlyph(this.g, this.c);
  final HG g;
  final Color c;
  @override
  void paint(Canvas canvas, Size s) => drawHg(canvas, s, g, c, Surface.base);
  @override
  bool shouldRepaint(HfTabGlyph o) => o.g != g || o.c != c;
}

class SearchBox extends StatelessWidget {
  const SearchBox(this.hint, {super.key});
  final String hint;
  @override
  Widget build(BuildContext context) => Container(
    height: Surface.control,
    padding: const EdgeInsets.symmetric(horizontal: 6),
    decoration: BoxDecoration(
      color: Surface.raised,
      border: Border.all(color: Surface.dividerFine),
      borderRadius: BorderRadius.circular(3),
    ),
    child: Row(
      children: [
        SizedBox(
          width: 11,
          height: 11,
          child: CustomPaint(painter: HfTabGlyph(HG.search, Surface.muted)),
        ),
        const SizedBox(width: 5),
        Expanded(
          child: Text(
            hint,
            softWrap: false,
            overflow: TextOverflow.clip,
            style: Dn.label(N.g63),
          ),
        ),
      ],
    ),
  );
}

class SearchKey extends StatelessWidget {
  const SearchKey({super.key});
  @override
  Widget build(BuildContext context) => Container(
    width: Surface.control,
    height: Surface.control,
    decoration: BoxDecoration(
      color: Surface.raised,
      border: Border.all(color: Surface.dividerFine),
      borderRadius: BorderRadius.circular(3),
    ),
    child: Center(
      child: SizedBox(
        width: 11,
        height: 11,
        child: CustomPaint(painter: HfTabGlyph(HG.search, Surface.muted)),
      ),
    ),
  );
}

/// Small text tabs with an underline: quiet housing for categories.
class Chips extends StatelessWidget {
  const Chips(this.items, this.active, {super.key});
  final List<String> items;
  final int active;
  @override
  Widget build(BuildContext context) => SizedBox(
    height: 18,
    child: Row(
      children: [
        for (var i = 0; i < items.length; i++)
          Container(
            margin: const EdgeInsets.only(right: 9),
            decoration: BoxDecoration(
              border: Border(
                bottom: BorderSide(
                  color: i == active
                      ? N.g91
                      : N.clear,
                  width: 1.5,
                ),
              ),
            ),
            child: Center(
              child: Text(
                items[i],
                softWrap: false,
                style: caps(
                  Dn.labelSize,
                  c: i == active ? N.g95 : Surface.muted,
                ),
              ),
            ),
          ),
      ],
    ),
  );
}

class Section extends StatelessWidget {
  const Section(this.label, {super.key, this.compact = false});
  final String label;
  final bool compact;
  @override
  Widget build(BuildContext context) => Padding(
    padding: EdgeInsets.only(top: compact ? 6 : 9, bottom: 5),
    child: Row(
      children: [
        Flexible(
          child: Text(
            label.toUpperCase(),
            softWrap: false,
            overflow: TextOverflow.clip,
            style: caps(compact ? 9.5 : 10),
          ),
        ),
        const SizedBox(width: 5),
        const Expanded(
          child: SizedBox(height: 1, child: ColoredBox(color: Surface.dividerFine)),
        ),
      ],
    ),
  );
}

void drawHg(Canvas c, Size s, HG g, Color col, Color bg) {
  // Reuse the prototype glyph painter through a throwaway widget-less call.
  HgPainter(g, col, bg).paint(c, s);
}
