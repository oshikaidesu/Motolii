// Panel chrome shared by the four panels: header row, class column placement, and the three
// morphologies (wide / narrow / strip). Bodies are the panels' own; this only places them.
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import '../glyphs.dart';
import 'classify.dart';
import 'common.dart';
import 'search.dart';
import '../metrics.dart';
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
  const PanelHeader({super.key, required this.title, required this.icon, required this.search, required this.hint, this.mode = HeadMode.full, this.count, this.extra, this.lead, this.height});
  final String title, hint;
  final Widget icon;
  final SearchCapability search;
  final HeadMode mode;
  final String? count;
  final Widget? extra;

  /// Takes the row's leading space (the title's) — a panel's class strip, when classes share the header row.
  final Widget? lead;

  /// A row lower than the mode's own (the compact one-row header of Create and Media).
  final double? height;

  /// A header's height by its mode: one chrome row, a little more when it names its panel.
  static double heightOf(HeadMode mode) => mode == HeadMode.full ? UiMetrics.namedHeader : UiMetrics.chromeRow;
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
    final h = widget.height ?? PanelHeader.heightOf(widget.mode);
    return ListenableBuilder(
      listenable: widget.search,
      builder: (_, __) {
        final searching = open || widget.search.active;
        return Container(
          height: h,
          padding: EdgeInsets.only(left: widget.lead == null ? 12 : 0, right: 4),
          decoration: const BoxDecoration(border: Border(bottom: BorderSide(color: kRule2))),
          child: searching
              ? Row(children: [
                  Expanded(child: SearchField(widget.search, widget.hint, height: 28, trailing: widget.count == null || !widget.search.active ? null : Text(widget.count!, style: mono(10)))),
                  Builder(builder: (context) => HeaderKey(HG.kebab, onTap: () {
                    final seat = BrowserSeatScope.of(context);
                    final box = context.findRenderObject() as RenderBox?;
                    if (seat != null && box != null) seat.more(context, box.localToGlobal(Offset(box.size.width, box.size.height)));
                  })),
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
                  if (widget.lead != null) Expanded(child: widget.lead!)
                  else if (DockedPanel.of(context)) const Spacer()
                  else if (widget.mode != HeadMode.stacked) ...[
                    widget.icon,
                    SizedBox(width: widget.mode == HeadMode.full ? 12 : 8),
                    Expanded(child: Text(widget.title, softWrap: false, overflow: TextOverflow.clip, style: sans(widget.mode == HeadMode.full ? 14 : 12.5, c: const Color(0xFFF2F2F4), w: FontWeight.w600, ls: -0.2))),
                  ] else
                    const Spacer(),
                  if (widget.extra != null) widget.extra!,
                  if (widget.count != null && widget.mode == HeadMode.full) Padding(padding: const EdgeInsets.only(right: 6), child: Text(widget.count!, style: mono(10))),
                  HeaderKey(HG.search, onTap: () {
                    setState(() => open = true);
                    WidgetsBinding.instance.addPostFrameCallback((_) => widget.search.request());
                  }),
                  if (widget.mode != HeadMode.compact && widget.lead == null) const HeaderKey(HG.grid4),
                  Builder(builder: (context) => HeaderKey(HG.kebab, onTap: () {
                    final seat = BrowserSeatScope.of(context);
                    final box = context.findRenderObject() as RenderBox?;
                    if (seat != null && box != null) seat.more(context, box.localToGlobal(Offset(box.size.width, box.size.height)));
                  })),
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
    // The rail is the Browser family's primary navigation, not spare room: it should survive at the
    // Dock's own default seat width (the REFERENCE's 324px seat, a few px narrower once its own
    // border/padding are taken out - measured ~260-290px live). Only drop to the narrow layout's bare
    // filter chip below that, and only as that fallback already does.
    this.columnMinWidth = 260,
    this.classStrip = false,
  });
  final String title, hint;
  final Widget icon;
  final SearchCapability search;
  final ClassifyCapability classify;
  final List<List<String>> groups;
  final String? count;
  final double columnWidth, columnMinWidth;

  /// Classes along the top (a [ClassStrip]) instead of the column: the body keeps the seat's whole width.
  final bool classStrip;
  final Widget Function(BuildContext, Size) wide, narrow, strip;
  @override
  Widget build(BuildContext context) {
    final stacked = PanelSeat.stackedOf(context);
    return LayoutBuilder(builder: (context, box) {
      final w = box.maxWidth, h = box.maxHeight;
      final isStrip = h < 170;
      final isWide = !isStrip && !classStrip && w >= columnMinWidth;
      final stripped = classStrip && !isStrip;
      final mode = stacked ? HeadMode.stacked : (isWide || (stripped && w >= columnMinWidth) ? HeadMode.full : HeadMode.compact);
      final seat = BrowserSeatScope.of(context);
      final tools = seat?.tools(context);
      final header = PanelHeader(
        title: title, icon: icon, search: search, hint: hint, mode: mode, count: count,
        lead: stripped ? ClassStrip(classify, groups, bare: true) : null,
        height: stripped ? UiMetrics.chromeRow : null,
        extra: tools == null && (isWide || stripped) ? null : Row(mainAxisSize: MainAxisSize.min, children: [if (tools != null) tools, if (!isWide && !stripped) ClassChip(classify)]),
      );
      final under = seat?.header(context);
      final hh = PanelHeader.heightOf(mode);
      final bodySize = Size(isWide ? w - columnWidth : w, h - (stripped ? UiMetrics.chromeRow : hh));
      final shell = search.keys(Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        header,
        if (under != null) under,
        Expanded(
          child: isStrip
              ? strip(context, bodySize)
              : stripped
                  ? wide(context, bodySize)
              : isWide
                  ? Row(crossAxisAlignment: CrossAxisAlignment.stretch, children: [ClassColumn(classify, groups, width: columnWidth), Expanded(child: wide(context, bodySize))])
                  : narrow(context, bodySize),
        ),
      ]));
      if (seat == null) return shell;
      // The panel's keyboard, when a host gives one; a key typed into a field stays that field's.
      return Focus(
        onKeyEvent: (n, e) {
          // Escape clears a search before it drops the pick (Classic's order); other keys are the seat's first.
          if (e.logicalKey == LogicalKeyboardKey.escape) {
            final r = search.onKey(n, e);
            return r == KeyEventResult.ignored ? seat.key(n, e) : r;
          }
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
