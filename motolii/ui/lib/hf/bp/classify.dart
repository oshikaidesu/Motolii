// The Classification capability: which class is chosen, and filtering by it. Behaviour only.
// The class column below is one face for it; a panel may show the choice some other way.
import 'package:flutter/widgets.dart';
import 'common.dart';
import 'search.dart';

class ClassifyCapability extends ChangeNotifier {
  ClassifyCapability({this.all = 'All', String? selected}) : _selected = selected ?? all;
  final String all;
  String _selected;
  String get selected => _selected;
  bool get filtering => _selected != all;
  void select(String c) {
    if (c == _selected) return;
    _selected = c;
    notifyListeners();
  }

  List<T> apply<T>(Iterable<T> items, String Function(T) classOf) => filtering ? [for (final it in items) if (classOf(it) == _selected) it] : items.toList();
}

/// Provides search and classification to a panel. Injected ones (fixtures, hosts) are not disposed here.
mixin WithDiscovery<W extends StatefulWidget> on State<W> {
  SearchCapability? get injectedSearch;
  ClassifyCapability? get injectedClassify;
  late final SearchCapability search = injectedSearch ?? SearchCapability();
  late final ClassifyCapability classify = injectedClassify ?? ClassifyCapability();
  Listenable get discovery => Listenable.merge([search, classify]);
  @override
  void dispose() {
    if (injectedSearch == null) search.dispose();
    if (injectedClassify == null) classify.dispose();
    super.dispose();
  }
}

/// A vertical class column: a face for the Classification capability. Groups are separated by space,
/// the chosen class is underlined and is scrolled into view. It scrolls, so hundreds of classes still fit.
class ClassColumn extends StatefulWidget {
  const ClassColumn(this.classify, this.groups, {super.key, this.width = 96, this.trailingPlus = true});
  final ClassifyCapability classify;
  final List<List<String>> groups;
  final double width;
  final bool trailingPlus;
  @override
  State<ClassColumn> createState() => _ClassColumnState();
}

class _ClassColumnState extends State<ClassColumn> {
  final scroll = ScrollController();
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) => _reveal());
    widget.classify.addListener(_reveal);
  }

  /// Rows are built lazily, so the chosen class is found by position, not by key.
  void _reveal() {
    if (!mounted || !scroll.hasClients) return;
    var y = 10.0;
    for (final (gi, g) in widget.groups.indexed) {
      if (gi > 0) y += 14;
      for (final c in g) {
        if (c == widget.classify.selected) {
          final view = scroll.position.viewportDimension;
          final want = (y - view / 2 + 13).clamp(0.0, scroll.position.maxScrollExtent);
          if ((y < scroll.offset) || (y + 27 > scroll.offset + view)) scroll.jumpTo(want);
          return;
        }
        y += 27;
      }
    }
  }

  @override
  void dispose() {
    widget.classify.removeListener(_reveal);
    scroll.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: widget.classify,
        builder: (_, __) => Container(
          width: widget.width,
          decoration: const BoxDecoration(border: Border(right: BorderSide(color: kRule2))),
          child: ListView(
            controller: scroll,
            padding: const EdgeInsets.fromLTRB(12, 10, 6, 10),
            physics: const ClampingScrollPhysics(),
            children: [
              for (final (gi, g) in widget.groups.indexed) ...[
                if (gi > 0) const SizedBox(height: 14),
                for (final c in g) _Row(c, c == widget.classify.selected, () => widget.classify.select(c)),
              ],
              if (widget.trailingPlus) Padding(padding: const EdgeInsets.only(top: 12), child: Text('+', style: sans(15, c: kMuted))),
            ],
          ),
        ),
      );
}

class _Row extends StatelessWidget {
  const _Row(this.label, this.on, this.tap, {super.key});
  final String label;
  final bool on;
  final VoidCallback tap;
  @override
  Widget build(BuildContext context) => GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: tap,
        child: SizedBox(
          height: 27,
          child: Align(
            alignment: Alignment.centerLeft,
            child: Container(
              padding: const EdgeInsets.only(bottom: 2),
              decoration: BoxDecoration(border: Border(bottom: BorderSide(color: on ? const Color(0xFFE8E8EA) : const Color(0x00000000), width: 1.5))),
              child: Text(label, softWrap: false, overflow: TextOverflow.clip, style: sans(11.5, c: on ? const Color(0xFFF2F2F4) : const Color(0xFF9EA0A6), w: on ? FontWeight.w600 : FontWeight.w400, ls: -0.1)),
            ),
          ),
        ),
      );
}

/// A horizontal class strip: the same capability as the column, laid along the top so the body keeps the seat's
/// whole width (Create's board, Media's sheet). Groups are separated by space; the chosen class is underlined.
class ClassStrip extends StatefulWidget {
  const ClassStrip(this.classify, this.groups, {super.key});
  final ClassifyCapability classify;
  final List<List<String>> groups;
  @override
  State<ClassStrip> createState() => _ClassStripState();
}

class _ClassStripState extends State<ClassStrip> {
  final _chosen = GlobalKey();
  @override
  void initState() {
    super.initState();
    widget.classify.addListener(_reveal);
    WidgetsBinding.instance.addPostFrameCallback((_) => _reveal());
  }

  /// The chosen class is never scrolled out of sight (the column does the same).
  void _reveal() => WidgetsBinding.instance.addPostFrameCallback((_) {
        final c = _chosen.currentContext;
        if (mounted && c != null) Scrollable.ensureVisible(c, alignmentPolicy: ScrollPositionAlignmentPolicy.keepVisibleAtEnd);
        if (mounted && c != null) Scrollable.ensureVisible(c, alignmentPolicy: ScrollPositionAlignmentPolicy.keepVisibleAtStart);
      });

  @override
  void dispose() {
    widget.classify.removeListener(_reveal);
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: widget.classify,
        builder: (_, __) => Container(
          height: 28,
          decoration: const BoxDecoration(border: Border(bottom: BorderSide(color: kRule2))),
          child: SingleChildScrollView(
            scrollDirection: Axis.horizontal,
            physics: const ClampingScrollPhysics(),
            padding: const EdgeInsets.symmetric(horizontal: 10),
            child: Row(children: [
              for (final (gi, g) in widget.groups.indexed) ...[
                if (gi > 0) const SizedBox(width: 10),
                for (final c in g)
                  Padding(
                    key: c == widget.classify.selected ? _chosen : null,
                    padding: const EdgeInsets.only(right: 10),
                    child: _Row(c, c == widget.classify.selected, () => widget.classify.select(c)),
                  ),
              ],
            ]),
          ),
        ),
      );
}

/// The small chip a narrow panel shows instead of the column, so a chosen class is never invisible.
class ClassChip extends StatelessWidget {
  const ClassChip(this.classify, {super.key});
  final ClassifyCapability classify;
  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: classify,
        builder: (_, __) => classify.filtering
            ? GestureDetector(
                onTap: () => classify.select(classify.all),
                child: Container(
                  height: 20,
                  padding: const EdgeInsets.symmetric(horizontal: 7),
                  decoration: BoxDecoration(color: kSel, borderRadius: BorderRadius.circular(3)),
                  child: Center(child: Text('${classify.selected}  ×', softWrap: false, style: sans(10.5, c: const Color(0xFFE6E6E8)))),
                ),
              )
            : const SizedBox.shrink(),
      );
}
