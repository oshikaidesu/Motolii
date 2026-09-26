// Panel chrome shared by the four panels: header row, class column placement, and the three
// morphologies (wide / narrow / strip). Bodies are the panels' own; this only places them.
import 'package:flutter/widgets.dart';
import '../glyphs.dart';
import 'classify.dart';
import 'common.dart';
import 'search.dart';
import 'seat.dart';

const kTile = Color(0xFF212124);
const kLabel = Color(0xFF7481C4);

class GlyphBox extends StatelessWidget {
  const GlyphBox(this.g, {super.key, this.size = 17, this.color = const Color(0xFFCFD0D3)});
  final HG g;
  final double size;
  final Color color;
  @override
  Widget build(BuildContext context) => SizedBox(width: size, height: size, child: CustomPaint(painter: HgPainter(g, color, kGround)));
}

class HeaderKey extends StatelessWidget {
  const HeaderKey(this.g, {super.key, this.on = false, this.onTap});
  final HG g;
  final bool on;
  final VoidCallback? onTap;
  @override
  Widget build(BuildContext context) => GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: onTap,
        child: SizedBox(width: 30, height: 30, child: Center(child: GlyphBox(g, size: 16, color: on ? const Color(0xFFF2F2F4) : const Color(0xFFB4B6BB)))),
      );
}

enum HeadMode { full, compact, stacked }

/// The header: identity and a few small keys. Search is a magnifier until it is used; then it takes the row.
class PanelHeader extends StatefulWidget {
  const PanelHeader({super.key, required this.title, required this.icon, required this.search, required this.hint, this.mode = HeadMode.full, this.count, this.extra});
  final String title, hint;
  final Widget icon;
  final SearchCapability search;
  final HeadMode mode;
  final String? count;
  final Widget? extra;
  @override
  State<PanelHeader> createState() => _PanelHeaderState();
}

class _PanelHeaderState extends State<PanelHeader> {
  bool open = false;
  @override
  void initState() {
    super.initState();
    widget.search.focus.addListener(_focus);
    widget.search.requests.addListener(_asked);
  }

  /// The capability asked for focus (a key press): open the folded face and take it.
  void _asked() {
    if (!open) setState(() => open = true);
    WidgetsBinding.instance.addPostFrameCallback((_) => widget.search.focus.requestFocus());
  }

  void _focus() {
    if (!widget.search.focus.hasFocus && !widget.search.active && open) setState(() => open = false);
  }

  @override
  void dispose() {
    widget.search.focus.removeListener(_focus);
    widget.search.requests.removeListener(_asked);
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final h = switch (widget.mode) { HeadMode.full => 52.0, HeadMode.compact => 40.0, HeadMode.stacked => 34.0 };
    return ListenableBuilder(
      listenable: widget.search,
      builder: (_, __) {
        final searching = open || widget.search.active;
        return Container(
          height: h,
          padding: const EdgeInsets.only(left: 12, right: 4),
          decoration: const BoxDecoration(border: Border(bottom: BorderSide(color: kRule2))),
          child: searching
              ? Row(children: [
                  Expanded(child: SearchField(widget.search, widget.hint, height: 28, trailing: widget.count == null || !widget.search.active ? null : Text(widget.count!, style: mono(10)))),
                  GestureDetector(
                    behavior: HitTestBehavior.opaque,
                    onTap: () {
                      widget.search.clear();
                      widget.search.focus.unfocus();
                      setState(() => open = false);
                    },
                    child: const SizedBox(width: 30, height: 30, child: Center(child: GlyphBox(HG.cross, size: 14, color: kMuted))),
                  ),
                ])
              : Row(children: [
                  if (DockedPanel.of(context)) const Spacer()
                  else if (widget.mode != HeadMode.stacked) ...[
                    widget.icon,
                    SizedBox(width: widget.mode == HeadMode.full ? 12 : 8),
                    Expanded(child: Text(widget.title, softWrap: false, overflow: TextOverflow.clip, style: sans(widget.mode == HeadMode.full ? 17 : 13, c: const Color(0xFFF2F2F4), w: FontWeight.w600, ls: -0.2))),
                  ] else
                    const Spacer(),
                  if (widget.extra != null) widget.extra!,
                  if (widget.count != null && widget.mode == HeadMode.full) Padding(padding: const EdgeInsets.only(right: 6), child: Text(widget.count!, style: mono(10))),
                  HeaderKey(HG.search, onTap: () {
                    setState(() => open = true);
                    WidgetsBinding.instance.addPostFrameCallback((_) => widget.search.request());
                  }),
                  if (widget.mode != HeadMode.compact) const HeaderKey(HG.grid4),
                  const HeaderKey(HG.kebab),
                ]),
        );
      },
    );
  }
}

/// Places header, class column and body according to the space. The bodies are the panel's own.
class PanelShell extends StatelessWidget {
  const PanelShell({
    super.key,
    required this.title,
    required this.icon,
    required this.search,
    required this.classify,
    required this.groups,
    required this.hint,
    required this.wide,
    required this.narrow,
    required this.strip,
    this.count,
    this.columnWidth = 96,
    this.columnMinWidth = 300,
  });
  final String title, hint;
  final Widget icon;
  final SearchCapability search;
  final ClassifyCapability classify;
  final List<List<String>> groups;
  final String? count;
  final double columnWidth, columnMinWidth;
  final Widget Function(BuildContext, Size) wide, narrow, strip;
  @override
  Widget build(BuildContext context) {
    final stacked = PanelSeat.stackedOf(context);
    return LayoutBuilder(builder: (context, box) {
      final w = box.maxWidth, h = box.maxHeight;
      final isStrip = h < 170;
      final isWide = !isStrip && w >= columnMinWidth;
      final mode = stacked ? HeadMode.stacked : (isWide ? HeadMode.full : HeadMode.compact);
      final seat = BrowserSeatScope.of(context);
      final tools = seat?.tools(context);
      final header = PanelHeader(
        title: title, icon: icon, search: search, hint: hint, mode: mode, count: count,
        extra: tools == null && isWide ? null : Row(mainAxisSize: MainAxisSize.min, children: [if (tools != null) tools, if (!isWide) ClassChip(classify)]),
      );
      final under = seat?.header(context);
      final hh = switch (mode) { HeadMode.full => 52.0, HeadMode.compact => 40.0, HeadMode.stacked => 34.0 };
      final bodySize = Size(isWide ? w - columnWidth : w, h - hh);
      final shell = search.keys(Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        header,
        if (under != null) under,
        Expanded(
          child: isStrip
              ? strip(context, bodySize)
              : isWide
                  ? Row(crossAxisAlignment: CrossAxisAlignment.stretch, children: [ClassColumn(classify, groups, width: columnWidth), Expanded(child: wide(context, bodySize))])
                  : narrow(context, bodySize),
        ),
      ]));
      if (seat == null) return shell;
      // The panel's keyboard, when a host gives one; a key typed into a field stays that field's.
      return Focus(
        onKeyEvent: (n, e) {
          final r = seat.key(n, e);
          return r == KeyEventResult.ignored ? search.onKey(n, e) : r;
        },
        child: Builder(builder: (context) => Listener(onPointerDown: (_) => Focus.of(context).requestFocus(), child: shell)),
      );
    });
  }
}

class SectionLabel extends StatelessWidget {
  const SectionLabel(this.text, {super.key});
  final String text;
  @override
  Widget build(BuildContext context) => Padding(padding: const EdgeInsets.fromLTRB(2, 14, 0, 8), child: Text(text.toUpperCase(), softWrap: false, style: sans(9.5, c: kLabel, w: FontWeight.w600, ls: 1.1)));
}

Widget emptyBody(String text) => Padding(padding: const EdgeInsets.fromLTRB(14, 18, 14, 4), child: Text(text, style: sans(11.5, c: kMuted)));
