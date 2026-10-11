// The dock's shared pieces: what a shelf remembers, hover, chips, section heads, the detail strip at the foot of a shelf.
import 'package:flutter/widgets.dart';

import '../sets/inspector/inspector_parts.dart' show Pop;
import '../sets/panels/media_parts.dart' show drawStar, glyph;
import '../tokens.dart';
import 'dock_glyphs.dart';
import 'ws.dart';

const dockFilterHeight = 168.0;

/// What the dock remembers per shelf while you hop between shelves (selection, chip, folds). Lives in the dock's state, never in [Ws].
class DockMem {
  final sel = <String, String>{'Media': 'coda_intro', 'Effects': 'Glow', 'Fonts': 'Avenir Next', 'Colors': 'pop-2', 'Create': 'Shape'};
  final chip = <String, String>{};
  final folded = <String>{'Time'};
  final open = <String>{'Project'};
  final starred = <String>{'Glow', 'Repeater', 'Trim Paths', 'Futura', 'Didot', 'Pop Poster'};
  final applied = <String>{};
  bool mediaList = false;

  /// The Media shelf: the open folder (a [FileNode] id) and the tree band's height when the shelf is narrow.
  String folder = 'Footage';
  double treeHeight = 132;

  /// The Effects filter band: open or not, which groups are folded, what is chosen per group.
  bool filtersOpen = false;
  double filterHeight = dockFilterHeight;
  final filterFolds = <String>{'Seat', 'Parameters', 'Origin'};
  final filter = <String, Set<String>>{};
}

/// One build's view of the dock for a shelf body: the memory, the search text, the document, and how to change local state.
class DockCtx {
  DockCtx(this.shelf, this.mem, this.query, this.ws, this.set);
  final String shelf;
  final DockMem mem;
  final String query;
  final Ws ws;
  final void Function(VoidCallback) set;

  String? get sel => mem.sel[shelf];
  void pick(String id) => set(() => mem.sel[shelf] = id);
  String get chip => mem.chip[shelf] ?? 'All';
  void pickChip(String c) => set(() => mem.chip[shelf] = c);
  bool hit(String name) => query.isEmpty || name.toLowerCase().contains(query.toLowerCase());
  bool applied(String id) => mem.applied.contains('$shelf:$id');
  void apply(String id) => set(() => mem.applied.add('$shelf:$id'));
  bool starred(String id) => mem.starred.contains(id);
  void star(String id) => set(() => mem.starred.contains(id) ? mem.starred.remove(id) : mem.starred.add(id));
  void toggle(Set<String> s, String id) => set(() => s.contains(id) ? s.remove(id) : s.add(id));
}

/// Hover + tap, nothing else: repaints only when the pointer enters or leaves.
class DockHover extends StatefulWidget {
  const DockHover({super.key, required this.builder, this.onTap});
  final Widget Function(BuildContext context, bool hover) builder;
  final VoidCallback? onTap;
  @override
  State<DockHover> createState() => _DockHoverState();
}

class _DockHoverState extends State<DockHover> {
  bool _h = false;
  @override
  Widget build(BuildContext context) => MouseRegion(
    cursor: widget.onTap == null ? MouseCursor.defer : SystemMouseCursors.click,
    onEnter: (_) => setState(() => _h = true),
    onExit: (_) => setState(() => _h = false),
    child: GestureDetector(behavior: HitTestBehavior.opaque, onTap: widget.onTap, child: widget.builder(context, _h)),
  );
}

/// Pop selection colours: a picked item is the accent with dark ink; hover is one grey step up.
Color dockFill(bool sel, bool hover, [Color? rest]) => sel ? WsT.accent : (hover ? WsT.card : rest ?? WsT.body);
Color dockInk(bool sel, [Color? rest]) => sel ? WsT.onAccent : rest ?? Grey.g91;

/// A row of round filter chips with counts; the active one is the accent. Scrolls sideways when the shelf is narrow.
class DockChips extends StatelessWidget {
  const DockChips(this.chips, {super.key, required this.active, required this.onPick});
  final List<(String, int?)> chips;
  final String active;
  final ValueChanged<String> onPick;
  @override
  Widget build(BuildContext context) => SizedBox(
    height: 28,
    child: SingleChildScrollView(
      scrollDirection: Axis.horizontal,
      padding: const EdgeInsets.fromLTRB(WsT.inset, 2, WsT.inset, 6),
      child: Row(
        children: [
          for (final (name, n) in chips) ...[
            DockHover(
              onTap: () => onPick(name),
              builder: (context, h) {
                final on = name == active;
                return AnimatedContainer(
                  duration: Mo.dur,
                  curve: Mo.ease,
                  height: 20,
                  padding: const EdgeInsets.symmetric(horizontal: 7),
                  decoration: BoxDecoration(color: on ? WsT.accent : (h ? WsT.raised : WsT.well), borderRadius: BorderRadius.circular(WsT.radius)),
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Text(name, style: T.micro(on ? WsT.onAccent : Grey.g91).copyWith(fontWeight: on ? FontWeight.w700 : FontWeight.w500)),
                      if (n != null) ...[const SizedBox(width: 4), Text('$n', style: T.value(on ? WsT.onAccent : Grey.g56).copyWith(fontSize: 10))],
                    ],
                  ),
                );
              },
            ),
            const SizedBox(width: WsT.gap),
          ],
        ],
      ),
    ),
  );
}

/// A section head inside a shelf body: a flat hue block, the name in caps, a count; optional fold chevron.
class DockSection extends StatelessWidget {
  const DockSection(this.title, {super.key, this.count, this.hue, this.folded, this.onTap, this.trailing});
  final String title;
  final int? count;
  final Color? hue;
  final bool? folded;
  final VoidCallback? onTap;
  final Widget? trailing;
  @override
  Widget build(BuildContext context) => DockHover(
    onTap: onTap,
    builder: (context, h) => Container(
      height: 24,
      color: h && onTap != null ? WsT.raised : WsT.card,
      padding: const EdgeInsets.symmetric(horizontal: WsT.inset),
      child: Row(
        children: [
          if (folded != null) ...[
            AnimatedRotation(
              turns: folded! ? 0 : .25,
              duration: Mo.dur,
              curve: Mo.ease,
              child: DockGlyph(DockG.chevron, size: 8, color: Grey.g63),
            ),
            const SizedBox(width: 6),
          ],
          Container(
            width: 8,
            height: 8,
            decoration: BoxDecoration(color: hue ?? Grey.g56, borderRadius: BorderRadius.circular(2)),
          ),
          const SizedBox(width: 6),
          Expanded(
            child: Text(
              title.toUpperCase(),
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: T.micro(Grey.g91).copyWith(fontWeight: FontWeight.w700, letterSpacing: .9),
            ),
          ),
          ?trailing,
          if (count != null) Text('$count', style: T.value(Grey.g56).copyWith(fontSize: 10)),
        ],
      ),
    ),
  );
}

/// A small round tag: hue fill with dark ink (pop), or a quiet well with grey ink.
class DockTag extends StatelessWidget {
  const DockTag(this.text, {super.key, this.hue, this.edge = false});
  final String text;
  final Color? hue;

  /// A dark keyline, for tags laid on artwork whose colour is unknown.
  final bool edge;
  @override
  Widget build(BuildContext context) => Container(
    height: edge ? 16 : 14,
    padding: const EdgeInsets.symmetric(horizontal: 4),
    alignment: Alignment.center,
    decoration: BoxDecoration(
      color: hue ?? WsT.raised,
      borderRadius: BorderRadius.circular(WsT.chipRadius),
      border: edge ? Border.all(color: WsT.onAccent, width: 1.5) : null,
    ),
    child: Text(text, style: T.micro(hue == null ? Grey.g76 : WsT.onAccent).copyWith(fontWeight: FontWeight.w700, letterSpacing: .4, height: 1)),
  );
}

/// A square hue tile carrying a dark glyph: how an item says what kind it is before you read its name.
class DockBadge extends StatelessWidget {
  const DockBadge(this.g, this.hue, {super.key, this.size = 20, this.inverted = false});
  final DockG g;
  final Color hue;
  final double size;
  final bool inverted;
  @override
  Widget build(BuildContext context) => Container(
    width: size,
    height: size,
    alignment: Alignment.center,
    decoration: BoxDecoration(color: inverted ? WsT.onAccent : hue, borderRadius: BorderRadius.circular(WsT.chipRadius)),
    child: DockGlyph(g, size: size * .62, color: inverted ? WsT.accent : WsT.onAccent),
  );
}

class DockIconButton extends StatelessWidget {
  const DockIconButton(this.g, {super.key, this.onTap, this.on = false});
  final DockG g;
  final VoidCallback? onTap;
  final bool on;
  @override
  Widget build(BuildContext context) => DockHover(
    onTap: onTap,
    builder: (context, h) => AnimatedContainer(
      duration: Mo.dur,
      curve: Mo.ease,
      width: 20,
      height: 20,
      margin: const EdgeInsets.only(left: 2),
      alignment: Alignment.center,
      decoration: BoxDecoration(color: on ? WsT.raised : (h ? WsT.card : null), borderRadius: BorderRadius.circular(WsT.chipRadius)),
      child: DockGlyph(g, size: 11, color: on || h ? Grey.g95 : Grey.g63),
    ),
  );
}

/// The foot of a shelf: what is picked (preview, name, facts), where it would go (the selected layer), and the one action, in the accent.
class DockFooter extends StatelessWidget {
  const DockFooter({
    super.key,
    this.preview,
    required this.title,
    required this.meta,
    required this.action,
    this.onAction,
    this.done = false,
    this.doneLabel = 'Added',
  });
  final Widget? preview;
  final String title, meta, action, doneLabel;
  final VoidCallback? onAction;
  final bool done;
  @override
  Widget build(BuildContext context) {
    final ws = WsScope.of(context), l = ws.layer;
    return Container(
      height: 52,
      padding: const EdgeInsets.symmetric(horizontal: WsT.inset),
      decoration: BoxDecoration(
        color: WsT.card,
        border: Border(
          top: BorderSide(color: WsT.ground, width: WsT.gutter),
        ),
      ),
      child: Row(
        children: [
          if (preview case final p?) ...[SizedBox.square(dimension: 34, child: p), const SizedBox(width: 8)],
          Expanded(
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  title,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: T.name(Grey.g95).copyWith(fontWeight: FontWeight.w600),
                ),
                const SizedBox(height: 4),
                Text(meta, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(Grey.g63)),
                const SizedBox(height: 4),
                Row(
                  children: [
                    Text('to ', style: T.label(Grey.g56)),
                    if (l != null) ...[
                      Container(
                        width: 6,
                        height: 6,
                        decoration: BoxDecoration(color: wsTone(l.kind), borderRadius: BorderRadius.circular(1)),
                      ),
                      const SizedBox(width: 4),
                      Flexible(
                        child: Text(l.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(Grey.g91)),
                      ),
                    ] else
                      Text('new layer', style: T.label(Grey.g91)),
                  ],
                ),
              ],
            ),
          ),
          const SizedBox(width: 6),
          DockHover(
            onTap: done ? null : onAction,
            builder: (context, h) => AnimatedContainer(
              duration: Mo.dur,
              curve: Mo.ease,
              height: 24,
              padding: const EdgeInsets.symmetric(horizontal: 10),
              alignment: Alignment.center,
              decoration: BoxDecoration(
                color: done ? WsT.raised : WsT.accent,
                borderRadius: BorderRadius.circular(WsT.radius),
                border: Border.all(color: h && !done ? Grey.g95 : const Color(0x00000000)),
              ),
              child: Text(done ? doneLabel : action, style: T.name(done ? Grey.g76 : WsT.onAccent).copyWith(fontWeight: FontWeight.w700)),
            ),
          ),
        ],
      ),
    );
  }
}

/// The search well under the header: magnifier, editable text, clear.
class DockSearch extends StatefulWidget {
  const DockSearch({super.key, required this.controller, required this.hint, this.trailing});
  final TextEditingController controller;
  final String hint;

  /// Sits beside the well, e.g. the filter toggle.
  final Widget? trailing;
  @override
  State<DockSearch> createState() => _DockSearchState();
}

class _DockSearchState extends State<DockSearch> {
  final _focus = FocusNode();
  @override
  void initState() {
    super.initState();
    _focus.addListener(_r);
  }

  void _r() => setState(() {});

  @override
  void dispose() {
    _focus.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final c = widget.controller;
    final well = GestureDetector(
      onTap: _focus.requestFocus,
      child: AnimatedContainer(
        duration: Mo.dur,
        curve: Mo.ease,
        height: 24,
        padding: const EdgeInsets.symmetric(horizontal: 7),
        decoration: BoxDecoration(
          color: WsT.well,
          borderRadius: BorderRadius.circular(WsT.radius),
          border: Border.all(color: _focus.hasFocus ? WsT.accentInk : WsT.well),
        ),
        child: Row(
          children: [
            DockGlyph(DockG.search, size: 11, color: _focus.hasFocus ? WsT.accentInk : Grey.g56),
            const SizedBox(width: 6),
            Expanded(
              child: Stack(
                alignment: Alignment.centerLeft,
                children: [
                  if (c.text.isEmpty)
                    IgnorePointer(
                      child: Text(widget.hint, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(Grey.g56)),
                    ),
                  EditableText(
                    controller: c,
                    focusNode: _focus,
                    style: T.name(Grey.g95),
                    cursorColor: WsT.accent,
                    backgroundCursorColor: Grey.g44,
                    selectionColor: Grey.g38,
                    cursorWidth: 1.5,
                    maxLines: 1,
                  ),
                ],
              ),
            ),
            if (c.text.isNotEmpty)
              DockHover(
                onTap: c.clear,
                builder: (context, h) => DockGlyph(DockG.close, size: 10, color: h ? Grey.g95 : Grey.g63),
              ),
          ],
        ),
      ),
    );
    return Padding(
      padding: const EdgeInsets.fromLTRB(WsT.inset, 6, WsT.inset, 4),
      child: Row(
        children: [
          Expanded(child: well),
          if (widget.trailing case final t?) ...[const SizedBox(width: WsT.gap), t],
        ],
      ),
    );
  }
}

/// What a shelf shows when the search finds nothing.
class DockEmpty extends StatelessWidget {
  const DockEmpty(this.query, {super.key});
  final String query;
  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.all(24),
    child: Text('Nothing matches “$query”', textAlign: TextAlign.center, style: T.label(Grey.g63)),
  );
}

/// The star that keeps an item in Starred; shown on hover, on the picked row, or when set.
class DockStar extends StatelessWidget {
  const DockStar({super.key, required this.on, required this.onTap, this.ink});
  final bool on;
  final VoidCallback onTap;
  final Color? ink;
  @override
  Widget build(BuildContext context) => DockHover(
    onTap: onTap,
    builder: (context, h) {
      final col = ink ?? (on ? Pop.keyDot : (h ? Grey.g91 : Grey.g56));
      return SizedBox(width: 20, height: 20, child: Center(child: glyph(11, (cv, s) => drawStar(cv, s, col, on), [col, on])));
    },
  );
}

/// A shelf's sections as cards: one column while the shelf is narrow; as it widens, columns of at least [minColumn] px stand side by side
/// and each section goes to the shortest column so far ([weights] estimate the heights, in rows). A shelf made wider shows more at once.
class DockMasonry extends StatelessWidget {
  const DockMasonry({super.key, required this.children, required this.weights, this.minColumn = 280});
  final List<Widget> children;
  final List<double> weights;
  final double minColumn;
  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, box) {
      final n = wsCols(box.maxWidth, minColumn, max: 4);
      if (n == 1) return ListView(padding: EdgeInsets.zero, children: children);
      final cols = List.generate(n, (_) => <Widget>[]), h = List.filled(n, 0.0);
      for (final (i, w) in children.indexed) {
        var k = 0;
        for (var j = 1; j < n; j++) {
          if (h[j] < h[k]) k = j;
        }
        cols[k].add(w);
        h[k] += weights[i];
      }
      return ListView(
        padding: EdgeInsets.zero,
        children: [
          Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              for (final (i, col) in cols.indexed) ...[
                if (i > 0) const SizedBox(width: WsT.gutter),
                Expanded(
                  child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: col),
                ),
              ],
            ],
          ),
        ],
      );
    },
  );
}
