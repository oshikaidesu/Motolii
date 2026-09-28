// Browser-family panels: visual prototype. Real Flutter layout, not reference coordinates.
// Only the dock's tab strip and two tiny controls are shared; every body decides its own folding.
import 'package:flutter/widgets.dart';

import '../metrics.dart';
import '../glyphs.dart';

const kRule = Color(0xFF343434);
const kRule2 = Color(0xFF2A2A2A);
const kGround = Color(0xFF191919);
const kRaised = Color(0xFF202020);
const kRaisedHi = Color(0xFF282828);
const kSel = Color(0xFF2F3034);
const kMuted = Color(0xFF8E8F92);

TextStyle sans(
  double s, {
  Color c = const Color(0xFFDADBDC),
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
TextStyle mono(double s, {Color c = kMuted, double ls = 0}) => TextStyle(
  fontFamily: 'Menlo',
  fontSize: s,
  color: c,
  letterSpacing: ls,
  height: 1.1,
);
TextStyle caps(double s, {Color c = kMuted}) =>
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
  });
  final List<TabSpec> tabs;
  final int active;
  final Widget body;

  /// A tap on a tab. Without it the strip is a picture.
  final ValueChanged<int>? onTab;

  /// What a host adds around each tab (the Dock: drag to move/split, right click for the panel menu). The tab's look
  /// is the Leaf's own either way.
  final Widget Function(int index, Widget tab)? tabWrap;
  static double labelled(String n) => 46 + n.length * 6.6;
  @override
  Widget build(BuildContext context) {
    final stacked = tabs.length > 1;
    return Container(
      decoration: BoxDecoration(
        color: kGround,
        border: Border.all(color: kRule),
        borderRadius: BorderRadius.circular(2),
      ),
      child: Column(
        children: [
          if (stacked)
            LayoutBuilder(
              builder: (_, c) {
                final need = tabs.fold<double>(
                  0,
                  (a, t) => a + labelled(t.name),
                );
                final fits = need <= c.maxWidth - 30 && c.maxWidth >= 110;
                return Container(
                  height: UiMetrics.chromeRow,
                  decoration: const BoxDecoration(
                    border: Border(bottom: BorderSide(color: kRule)),
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
    height: 30,
    padding: EdgeInsets.symmetric(horizontal: compact ? 7 : 10),
    decoration: BoxDecoration(
      color: (selected && !single) ? kSel : null,
      border: const Border(right: BorderSide(color: kRule2)),
    ),
    child: Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        SizedBox(
          width: 15,
          height: 15,
          child: CustomPaint(
            painter: HfTabGlyph(
              t.glyph,
              selected ? const Color(0xFFF0F0F0) : kMuted,
            ),
          ),
        ),
        if (!compact) ...[
          const SizedBox(width: 7),
          Text(
            t.name,
            softWrap: false,
            style: sans(
              12,
              c: selected ? const Color(0xFFF2F2F2) : kMuted,
              w: FontWeight.w500,
            ),
          ),
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
  void paint(Canvas canvas, Size s) => drawHg(canvas, s, g, c, kGround);
  @override
  bool shouldRepaint(HfTabGlyph o) => o.g != g || o.c != c;
}

class SearchBox extends StatelessWidget {
  const SearchBox(this.hint, {super.key});
  final String hint;
  @override
  Widget build(BuildContext context) => Container(
    height: 26,
    padding: const EdgeInsets.symmetric(horizontal: 8),
    decoration: BoxDecoration(
      color: kRaised,
      border: Border.all(color: kRule2),
      borderRadius: BorderRadius.circular(3),
    ),
    child: Row(
      children: [
        SizedBox(
          width: 13,
          height: 13,
          child: CustomPaint(painter: HfTabGlyph(HG.search, kMuted)),
        ),
        const SizedBox(width: 7),
        Expanded(
          child: Text(
            hint,
            softWrap: false,
            overflow: TextOverflow.clip,
            style: sans(11.5, c: kMuted),
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
    width: 26,
    height: 26,
    decoration: BoxDecoration(
      color: kRaised,
      border: Border.all(color: kRule2),
      borderRadius: BorderRadius.circular(3),
    ),
    child: Center(
      child: SizedBox(
        width: 13,
        height: 13,
        child: CustomPaint(painter: HfTabGlyph(HG.search, kMuted)),
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
    height: 24,
    child: Row(
      children: [
        for (var i = 0; i < items.length; i++)
          Container(
            margin: const EdgeInsets.only(right: 12),
            decoration: BoxDecoration(
              border: Border(
                bottom: BorderSide(
                  color: i == active
                      ? const Color(0xFFE8E8E8)
                      : const Color(0x00000000),
                  width: 1.5,
                ),
              ),
            ),
            child: Center(
              child: Text(
                items[i],
                softWrap: false,
                style: caps(
                  10.5,
                  c: i == active ? const Color(0xFFF0F0F0) : kMuted,
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
    padding: EdgeInsets.only(top: compact ? 8 : 12, bottom: 6),
    child: Row(
      children: [
        Flexible(
          child: Text(
            label.toUpperCase(),
            softWrap: false,
            overflow: TextOverflow.clip,
            style: caps(compact ? 9 : 10),
          ),
        ),
        const SizedBox(width: 6),
        const Expanded(
          child: SizedBox(height: 1, child: ColoredBox(color: kRule2)),
        ),
      ],
    ),
  );
}

void drawHg(Canvas c, Size s, HG g, Color col, Color bg) {
  // Reuse the prototype glyph painter through a throwaway widget-less call.
  HgPainter(g, col, bg).paint(c, s);
}
