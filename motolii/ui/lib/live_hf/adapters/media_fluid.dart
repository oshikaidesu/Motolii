import 'dart:math' as math;

import 'package:flutter/scheduler.dart';
import 'package:flutter/widgets.dart';

import '../../hf/bp/shelf_sections.dart' show PickedRing;
import '../../hf/metrics.dart';
import '../../hf/neutral.dart';
import 'browser_item.dart';
import 'media_library.dart' show materialFace;
import 'media_list.dart' show MediaListHeader;

/// Where every face of a projection stands: faces and the labels by them, in one scrolling content.
typedef Frame = ({Map<String, Rect> faces, Map<String, Rect> labels, Size content});

/// Explore's places: given by the caller (a prototype: there is no similarity backend).
typedef ExploreLayout = Frame Function(List<BrowserItem> items, String? selected, Size viewport);

/// The three projections of one Result Set as one board: every asset is one face, placed by the projection in force, and a
/// change of projection moves the same faces (the selection, the identity and the scroll context stay). List lays them in
/// rows with their facts beside, Thumbnail packs them at their own shapes with a name under each, Explore is the caller's.
class FluidBoard extends StatefulWidget {
  const FluidBoard({super.key, required this.items, required this.selected, required this.view, required this.onTap, this.onOpen, this.explore});
  final List<BrowserItem> items;
  final String? selected;
  final String view; // 'list' | 'thumbnail' | 'explore'
  final ValueChanged<String> onTap;
  final ValueChanged<String>? onOpen;
  final ExploreLayout? explore;

  static const rowHeight = 32.0, caption = 14.0, margin = 6.0, gap = 3.0, minColumn = 44.0;

  /// Thumbnail: the shelf's masonry (the kinds alternate, each face at its own shape, the shortest column next).
  static Frame thumbnail(List<BrowserItem> items, double width) {
    final byKind = <String, List<BrowserItem>>{};
    for (final i in items) {
      (byKind[i.kind] ??= []).add(i);
    }
    final order = [for (var n = 0; byKind.values.any((l) => n < l.length); n++) for (final l in byKind.values) if (n < l.length) l[n]];
    final cols = math.max(1, ((width - margin * 2 + gap) / (minColumn + gap)).floor());
    final colW = (width - margin * 2 - gap * (cols - 1)) / cols;
    final heights = List.filled(cols, 0.0);
    final faces = <String, Rect>{}, labels = <String, Rect>{};
    for (final it in order) {
      var at = 0;
      for (var c = 1; c < cols; c++) {
        if (heights[c] < heights[at] - 1) at = c;
      }
      final x = margin + at * (colW + gap), y = margin + heights[at], h = colW / it.aspect;
      faces[it.id] = Rect.fromLTWH(x, y, colW, h);
      labels[it.id] = Rect.fromLTWH(x, y + h, colW, caption);
      heights[at] += h + caption + gap;
    }
    return (faces: faces, labels: labels, content: Size(width, heights.fold(0.0, math.max) + margin));
  }

  /// List: a row each, a small face at its own shape, the facts beside it.
  static Frame list(List<BrowserItem> items, double width) {
    const boxW = 46.0, boxH = 26.0;
    final faces = <String, Rect>{}, labels = <String, Rect>{};
    for (final (n, it) in items.indexed) {
      final a = it.aspect, fw = a >= boxW / boxH ? boxW : boxH * a, fh = a >= boxW / boxH ? boxW / a : boxH;
      final top = n * rowHeight;
      faces[it.id] = Rect.fromLTWH(8 + (boxW - fw) / 2, top + (rowHeight - fh) / 2, fw, fh);
      labels[it.id] = Rect.fromLTWH(62, top, width - 62, rowHeight);
    }
    return (faces: faces, labels: labels, content: Size(width, items.length * rowHeight));
  }

  @override
  State<FluidBoard> createState() => _FluidBoardState();
}

class _FluidBoardState extends State<FluidBoard> {
  final scroll = ScrollController();

  @override
  void dispose() {
    scroll.dispose();
    super.dispose();
  }

  Frame _frame(Size viewport) => switch (widget.view) {
        'list' => FluidBoard.list(widget.items, viewport.width),
        'explore' when widget.explore != null => widget.explore!(widget.items, widget.selected, viewport),
        _ => FluidBoard.thumbnail(widget.items, viewport.width),
      };

  @override
  void didUpdateWidget(FluidBoard old) {
    super.didUpdateWidget(old);
    if (old.view != widget.view) {
      // the same asset stays in view across the change: bring the chosen one to the middle of the new arrangement
      SchedulerBinding.instance.addPostFrameCallback((_) => _reveal());
    }
  }

  Size _viewport = Size.zero;

  void _reveal() {
    final id = widget.selected;
    if (id == null || !scroll.hasClients) return;
    final rect = _frame(_viewport).faces[id];
    if (rect == null) return;
    final target = (rect.center.dy - _viewport.height / 2).clamp(0.0, math.max(0.0, scroll.position.maxScrollExtent)).toDouble();
    scroll.animateTo(target, duration: const Duration(milliseconds: 380), curve: Curves.easeInOutCubic);
  }

  @override
  Widget build(BuildContext context) => LayoutBuilder(builder: (context, box) {
        _viewport = Size(box.maxWidth, box.maxHeight);
        final frame = _frame(_viewport);
        final content = Size(math.max(frame.content.width, box.maxWidth), math.max(frame.content.height, box.maxHeight));
        const move = Duration(milliseconds: 380);
        const curve = Curves.easeInOutCubic;
        return SingleChildScrollView(
          controller: scroll,
          physics: const ClampingScrollPhysics(),
          scrollDirection: Axis.vertical,
          child: TweenAnimationBuilder<Size>(
            tween: Tween<Size>(end: content),
            duration: move,
            curve: curve,
            builder: (context, size, _) => SizedBox(
              width: size.width,
              height: size.height,
              child: Stack(clipBehavior: Clip.hardEdge, children: [
                for (final it in widget.items) ...[
                  if (frame.labels[it.id] != null)
                    AnimatedPositioned(
                      key: ValueKey('label-${it.id}'),
                      duration: move,
                      curve: curve,
                      left: frame.labels[it.id]!.left,
                      top: frame.labels[it.id]!.top,
                      width: frame.labels[it.id]!.width,
                      height: frame.labels[it.id]!.height,
                      child: GestureDetector(
                        behavior: HitTestBehavior.opaque,
                        onTap: () => widget.onTap(it.id),
                        onDoubleTap: widget.onOpen == null ? null : () => widget.onOpen!(it.id),
                        // each label is laid out at the width it was made for (the box is still growing or shrinking, and an
                        // outgoing label keeps its own): clipped, never squeezed
                        child: ClipRect(
                          child: AnimatedSwitcher(
                            duration: const Duration(milliseconds: 220),
                            child: KeyedSubtree(
                              key: ValueKey('${widget.view}-${it.id}'),
                              child: OverflowBox(
                                alignment: Alignment.topLeft,
                                minWidth: frame.labels[it.id]!.width,
                                maxWidth: frame.labels[it.id]!.width,
                                minHeight: frame.labels[it.id]!.height,
                                maxHeight: frame.labels[it.id]!.height,
                                child: _label(it, frame.labels[it.id]!.width),
                              ),
                            ),
                          ),
                        ),
                      ),
                    ),
                  if (frame.faces[it.id] != null)
                    AnimatedPositioned(
                      key: ValueKey('face-${it.id}'),
                      duration: move,
                      curve: curve,
                      left: frame.faces[it.id]!.left,
                      top: frame.faces[it.id]!.top,
                      width: frame.faces[it.id]!.width,
                      height: frame.faces[it.id]!.height,
                      child: GestureDetector(
                        behavior: HitTestBehavior.opaque,
                        onTap: () => widget.onTap(it.id),
                        onDoubleTap: widget.onOpen == null ? null : () => widget.onOpen!(it.id),
                        child: _Face(it, selected: it.id == widget.selected, marked: widget.view != 'list'),
                      ),
                    ),
                ],
              ]),
            ),
          ),
        );
      });

  Widget _label(BrowserItem it, double width) {
    final on = it.id == widget.selected;
    switch (widget.view) {
      case 'list':
        return Row(children: [
          Expanded(child: Text(it.name, maxLines: 1, softWrap: false, overflow: TextOverflow.ellipsis, style: Dn.name(on ? N.g100 : N.g91))),
          SizedBox(width: 42, child: Text(it.typeWord, softWrap: false, style: Dn.label(N.g63))),
          SizedBox(width: 40, child: Text(clockText(it.seconds), softWrap: false, style: Dn.value(N.g69).copyWith(fontSize: 10))),
          if (MediaListHeader.wide(width + 62)) SizedBox(width: 54, child: Text(sizeText(it.size), softWrap: false, textAlign: TextAlign.right, style: Dn.value(N.g69).copyWith(fontSize: 10))),
          if (MediaListHeader.roomy(width + 62)) SizedBox(width: 74, child: Text(dateText(it.mtimeNs), softWrap: false, textAlign: TextAlign.right, style: Dn.value(N.g56).copyWith(fontSize: 10))),
          const SizedBox(width: 8),
        ]);
      case 'thumbnail':
        return Align(alignment: Alignment.bottomLeft, child: Text(it.name, maxLines: 1, softWrap: false, overflow: TextOverflow.ellipsis, style: Dn.label(on ? N.g100 : N.g82, FontWeight.w500)));
      default:
        return const SizedBox.shrink();
    }
  }
}

class _Face extends StatelessWidget {
  const _Face(this.item, {required this.selected, required this.marked});
  final BrowserItem item;
  final bool selected, marked;
  @override
  Widget build(BuildContext context) => Stack(fit: StackFit.expand, children: [
        ClipRRect(borderRadius: BorderRadius.circular(3), child: ColoredBox(color: N.g13, child: materialFace(item.shelf))),
        if (marked && item.mark.isNotEmpty) Positioned(left: 2.5, top: 2.5, child: Container(padding: const EdgeInsets.symmetric(horizontal: 3, vertical: 1.5), decoration: BoxDecoration(color: N.veil, borderRadius: BorderRadius.circular(2)), child: Text(item.mark, softWrap: false, style: Dn.micro(N.g95).copyWith(fontSize: 8.5)))),
        if (selected) const IgnorePointer(child: PickedRing(radius: 3)),
      ]);
}
