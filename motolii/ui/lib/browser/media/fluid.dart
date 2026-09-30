import 'dart:async';
import 'dart:convert';
import 'dart:math' as math;
import 'dart:typed_data';

import 'package:flutter/gestures.dart' show kPrimaryButton;
import 'package:flutter/rendering.dart' show ScrollCacheExtent, SliverConstraints, SliverGridGeometry, SliverGridLayout;
import 'package:flutter/scheduler.dart';
import 'package:flutter/widgets.dart';

import '../../hf/metrics.dart';
import '../../hf/neutral.dart';
import '../item.dart';
import 'explore/overlay.dart';
import 'library.dart' show materialFace;
import 'list.dart' show MediaListHeader;
import 'preview.dart' show FaceService;
import 'model_face.dart';

/// Where every face of a projection stands: faces and the labels by them, in one scrolling content.
typedef Frame = ({Map<String, Rect> faces, Map<String, Rect> labels, Size content, List<String> links, GraphOverlay? graph});

/// Explore's places: given by the caller (a prototype: there is no similarity backend).
typedef ExploreLayout = Frame Function(List<BrowserItem> items, String? selected, Size viewport);

/// The three projections of one Result Set as one board: every asset is one face, placed by the projection in force, and a
/// change of projection moves the same faces (the selection, the identity and the scroll context stay). List lays them in
/// rows with their facts beside, Thumbnail packs them at their own shapes with a name under each, Explore is the caller's.
class FluidBoard extends StatefulWidget {
  const FluidBoard({super.key, required this.items, required this.selected, required this.view, required this.onTap, this.onOpen, this.explore, this.faces, this.holding = const {}, this.minColumn = defaultColumn, this.revealOn = false, this.picked = const {}, this.carry, this.onMenu});
  final List<BrowserItem> items;

  /// Everything picked (the chosen one, `selected`, is the last of them): each carries the ring.
  final Set<String> picked;

  /// What dragging a face carries to a drop target (the Timeline), or null when it cannot be carried.
  final Map<String, dynamic>? Function(BrowserItem item)? carry;

  /// A right-click on a face or its row, at a global position.
  final void Function(String id, Offset at)? onMenu;

  /// The narrowest a Thumbnail column may be (the person's size choice: wider columns, bigger faces).
  final double minColumn;

  /// Turning this on (a preview opened, the seat got shorter) brings the chosen face back into view.
  final bool revealOn;
  final String? selected;
  final String view; // 'list' | 'thumbnail' | 'explore'
  final ValueChanged<String> onTap;
  final ValueChanged<String>? onOpen;
  final ExploreLayout? explore;

  /// What a tile may ask the owner for while a pointer passes over it (a clip's frame at a time).
  final FaceService? faces;

  /// Pointers held over faces (asset id -> 0..1 within the face): a story's way to show what a passing pointer does.
  final Map<String, Offset> holding;

  static const rowHeight = 32.0, caption = 14.0, margin = 6.0, gap = 3.0, defaultColumn = 44.0;

  /// Thumbnail: the shelf's masonry (the kinds alternate, each face at its own shape, the shortest column next).
  static Frame thumbnail(List<BrowserItem> items, double width, [double minColumn = defaultColumn]) {
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
    return (faces: faces, labels: labels, links: const <String>[], graph: null, content: Size(width, heights.fold(0.0, math.max) + margin));
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
    return (faces: faces, labels: labels, links: const <String>[], graph: null, content: Size(width, items.length * rowHeight));
  }

  @override
  State<FluidBoard> createState() => _FluidBoardState();
}

class _FluidBoardState extends State<FluidBoard> with TickerProviderStateMixin {
  /// The scroll of the projection in view (List and Thumbnail build only what is on screen; each projection has its own).
  ScrollController _scroll = ScrollController();

  /// Moving between projections: it starts on the input's own frame and only slows at the end (ease-out), and it is short.
  /// The same asset keeps its identity through it; a long, slow move would not make it any more continuous.
  static const _move = Duration(milliseconds: 200);
  static const _curve = Curves.easeOutCubic;
  bool _instant = false;

  /// The node under the pointer in Explore's graph: with the chosen one it decides what stays emphasised.
  String? _hot;
  final _view = TransformationController();
  Object? _fitted;

  /// The camera moving to a new fit (another graph, Global to Local): it starts at once and only slows as it arrives.
  late final AnimationController _fly = AnimationController(vsync: this, duration: const Duration(milliseconds: 260))..addListener(_flyStep);
  Matrix4? _flyFrom, _flyTo;
  void _flyStep() {
    final from = _flyFrom, to = _flyTo;
    if (from == null || to == null) return;
    final t = Curves.easeOutCubic.transform(_fly.value);
    _view.value = Matrix4.fromList([for (var i = 0; i < 16; i++) from.storage[i] + (to.storage[i] - from.storage[i]) * t]);
  }

  void _flyTo_(Matrix4 to, {required bool animate}) {
    if (!animate) {
      _fly.stop();
      _view.value = to;
      return;
    }
    _flyFrom = _view.value.clone();
    _flyTo = to;
    _fly.forward(from: 0);
  }

  /// While the faces are on their way the labels stay out (they would run into one another); they return when the faces land.
  bool _moving = false;
  Timer? _landed;
  void _hush() {
    _landed?.cancel();
    setState(() => _moving = true);
    _landed = Timer(_move + const Duration(milliseconds: 40), () {
      if (mounted) setState(() => _moving = false);
    });
  }

  @override
  void dispose() {
    _landed?.cancel();
    _fly.dispose();
    _view.dispose();
    _shiftAnim.dispose();
    _scroll.dispose();
    super.dispose();
  }

  Frame _frame(Size viewport) => switch (widget.view) {
        'list' => FluidBoard.list(widget.items, viewport.width),
        'explore' when widget.explore != null => widget.explore!(widget.items, widget.selected, viewport),
        _ => FluidBoard.thumbnail(widget.items, viewport.width, widget.minColumn),
      };

  static bool _lazy(String view) => view == 'list' || view == 'thumbnail';

  /// The two projections that are built lazily, laid out once per (what is shown, how wide, how big the faces are).
  Frame _placed(String view, List<BrowserItem> items, double width, double minColumn) {
    final key = (view, width, minColumn, items.length, items.isEmpty ? '' : items.first.id, items.isEmpty ? '' : items.last.id, items.fold<int>(0, (a, i) => a ^ i.id.hashCode));
    if (_placedKey == key && _placedFrame != null) return _placedFrame!;
    _placedKey = key;
    return _placedFrame = view == 'list' ? FluidBoard.list(items, width) : FluidBoard.thumbnail(items, width, minColumn);
  }

  Object? _placedKey;
  Frame? _placedFrame;

  /// Where the scroll should stand so that the chosen asset is in the middle of the seat (none chosen: the top).
  double _offsetFor(Frame frame) {
    final rect = frame.faces[widget.selected];
    if (rect == null) return 0;
    return (rect.center.dy - _viewport.height / 2).clamp(0.0, math.max(0.0, frame.content.height - _viewport.height)).toDouble();
  }

  @override
  void didUpdateWidget(FluidBoard old) {
    super.didUpdateWidget(old);
    if (old.revealOn != widget.revealOn) {
      SchedulerBinding.instance.addPostFrameCallback((_) => _reveal());
    }
    // a size being dragged is direct manipulation: the faces follow it 1:1, nothing animates behind the pointer
    _instant = old.view == widget.view && old.minColumn != widget.minColumn;
    if (old.view == widget.view) return;
    if (_lazy(old.view) && _lazy(widget.view) && _viewport != Size.zero && widget.items.isNotEmpty) {
      _startShift(old);
    } else {
      if (widget.view == 'explore') _hush();
      _newScroll(_lazy(widget.view) && _viewport != Size.zero ? _offsetFor(_placed(widget.view, widget.items, _viewport.width, widget.minColumn)) : 0);
    }
  }

  Size _viewport = Size.zero;

  void _newScroll(double at) {
    final old = _scroll;
    _scroll = ScrollController(initialScrollOffset: at);
    SchedulerBinding.instance.addPostFrameCallback((_) => old.dispose());
  }

  // ---- List <-> Thumbnail: the same assets carried from where they stood to where they stand now -------------------------
  //
  // Only what the person can see is carried (the seat and a little round it), by one controller and one overlay; the rest of the
  // library is built where it lands, lazily, when the move is over. A thousand faces out of sight are not kept alive for a
  // motion nobody sees.
  late final AnimationController _shiftAnim = AnimationController(vsync: this, duration: _move)..addStatusListener((status) {
    if (status == AnimationStatus.completed && mounted) setState(() => _shift = null);
  });
  ({Map<String, Rect> from, Map<String, Rect> to, List<BrowserItem> carried})? _shift;

  void _startShift(FluidBoard old) {
    final was = old.view == 'list' ? FluidBoard.list(old.items, _viewport.width) : FluidBoard.thumbnail(old.items, _viewport.width, old.minColumn);
    final now = _placed(widget.view, widget.items, _viewport.width, widget.minColumn);
    final from = _scroll.hasClients ? _scroll.offset : 0.0, to = _offsetFor(now);
    final sight = Rect.fromLTWH(0, -160, _viewport.width, _viewport.height + 320);
    final start = <String, Rect>{}, end = <String, Rect>{};
    final carried = <BrowserItem>[];
    for (final it in widget.items) {
      final a = was.faces[it.id], b = now.faces[it.id];
      if (a == null || b == null) continue;
      final ra = a.shift(Offset(0, -from)), rb = b.shift(Offset(0, -to));
      if (!sight.overlaps(ra) && !sight.overlaps(rb)) continue;
      start[it.id] = ra;
      end[it.id] = rb;
      carried.add(it);
    }
    _newScroll(to);
    _shift = (from: start, to: end, carried: carried);
    _shiftAnim.forward(from: 0); // the first frame after the input already shows movement (ease-out starts fast)
  }

  void _reveal() {
    final id = widget.selected;
    if (id == null || !_scroll.hasClients) return;
    final frame = _lazy(widget.view) ? _placed(widget.view, widget.items, _viewport.width, widget.minColumn) : _frame(_viewport);
    final rect = frame.faces[id];
    if (rect == null) return;
    final target = (rect.center.dy - _viewport.height / 2).clamp(0.0, math.max(0.0, _scroll.position.maxScrollExtent)).toDouble();
    _scroll.animateTo(target, duration: _move, curve: _curve);
  }

  @override
  Widget build(BuildContext context) => LayoutBuilder(builder: (context, box) {
        _viewport = Size(box.maxWidth, box.maxHeight);
        if (_lazy(widget.view)) return _lazyBody(box);
        final frame = _frame(_viewport);
        // Thumbnail and List lay out to the seat's width (responsive layout). Explore's graph is a world with its own size: the
        // seat is only the lens on it, so its size never enters the map.
        final content = frame.graph != null ? frame.content : Size(math.max(frame.content.width, box.maxWidth), math.max(frame.content.height, box.maxHeight));
        final move = _instant ? Duration.zero : _move;
        const curve = _curve;
        final graph = frame.graph;
        final focus = graph == null ? null : (_hot ?? widget.selected);
        final near = focus == null ? const <String>{} : graph!.neighbours(focus);
        bool dim(String id) => focus != null && id != focus && !near.contains(id);
        Offset? centre(String id) => frame.faces[id]?.center ?? graph?.hubs.where((h) => h.id == id).firstOrNull?.rect.center;
        final board = TweenAnimationBuilder<Size>(
            tween: Tween<Size>(end: content),
            duration: move,
            curve: curve,
            builder: (context, size, _) => SizedBox(
              width: size.width,
              height: size.height,
              child: Stack(clipBehavior: Clip.hardEdge, children: [
                // Explore: a line from the chosen asset to each of its nearest (what the layout says is near, nothing more)
                if (widget.view == 'explore' && frame.links.isNotEmpty && frame.faces[widget.selected] != null)
                  Positioned.fill(
                    child: IgnorePointer(
                      child: AnimatedOpacity(
                        opacity: _moving ? 0 : 1,
                        duration: Duration(milliseconds: _moving ? 0 : 120),
                        child: CustomPaint(painter: _Spokes(frame.faces[widget.selected]!.center, [for (final id in frame.links) if (frame.faces[id] != null) frame.faces[id]!.center])),
                      ),
                    ),
                  ),
                if (graph != null && graph.blobs.isNotEmpty)
                  Positioned.fill(child: IgnorePointer(child: CustomPaint(painter: GraphBlobs(graph.blobs)))),
                // Explore's graph: the proven relations as lines (they wait out a move, so they never point at where a face was)
                if (graph != null)
                  Positioned.fill(
                    child: IgnorePointer(
                      child: AnimatedOpacity(
                        opacity: _moving ? 0 : 1,
                        duration: Duration(milliseconds: _moving ? 0 : 120),
                        child: CustomPaint(painter: GraphEdges(edges: graph.edges, at: centre, focus: focus, near: near)),
                      ),
                    ),
                  ),
                if (graph != null)
                  for (final h in graph.hubs) _Glide(key: ValueKey('hub-${h.id}'), rect: h.rect, instant: _instant, child: MouseRegion(onEnter: (_) => setState(() => _hot = h.id), onExit: (_) => setState(() => _hot = null), child: HubPill(hub: h, dim: dim(h.id)))),
                for (final it in widget.items) ...[
                  if (frame.labels[it.id] != null) _Glide(key: ValueKey('label-${it.id}'), rect: frame.labels[it.id]!, instant: _instant, child: _labelWidget(it, frame.labels[it.id]!.width, hush: _moving)),
                  if (frame.faces[it.id] != null) _Glide(key: ValueKey('face-${it.id}'), rect: frame.faces[it.id]!, instant: _instant, child: _faceWidget(it, frame.faces[it.id]!, dim: dim(it.id), tint: graph?.tints[it.id], onHover: graph == null ? null : (on) => setState(() => _hot = on ? it.id : null))),
                ],
              ]),
            ),
          );
        // the graph is a canvas to look round (pan and zoom are the person's own moves); every other projection scrolls
        if (graph != null) {
          // the camera starts with the whole map in view, once per map (a different graph, not a different seat): resizing the
          // Browser only changes how much of the same map is seen, and the person's own pan and zoom are never taken back
          final fit = (frame.content.width, frame.content.height, graph.edges.length, graph.hubs.length);
          if (_fitted != fit) {
            final first = _fitted == null;
            _fitted = fit;
            final scale = graph.initialScale ?? math.min(1.0, math.min(box.maxWidth / content.width, box.maxHeight / content.height));
            // the whole map in view, centred in the seat (what the lens shows beyond the map is empty space, evenly)
            final dx = math.max(0.0, (box.maxWidth - content.width * scale) / 2), dy = math.max(0.0, (box.maxHeight - content.height * scale) / 2);
            final to = Matrix4.identity()..translate(dx, dy)..scale(scale, scale, 1);
            if (first) {
              _view.value = to; // the first picture of a map already has its camera
            } else {
              WidgetsBinding.instance.addPostFrameCallback((_) => mounted ? _flyTo_(to, animate: true) : null);
            }
          }
        }
        return graph != null
            ? InteractiveViewer(transformationController: _view, constrained: false, minScale: .2, maxScale: 3.5, boundaryMargin: const EdgeInsets.all(600), child: board)
            : SingleChildScrollView(controller: _scroll, physics: const ClampingScrollPhysics(), scrollDirection: Axis.vertical, child: board);
      });

  // ---- List and Thumbnail, built lazily -----------------------------------------------------------------------------

  Widget _lazyBody(BoxConstraints box) {
    final items = widget.items;
    if (_shift case final shift?) {
      // the move: the carried faces, one overlay, one controller; nothing else is built until they land
      return AnimatedBuilder(
        animation: _shiftAnim,
        builder: (context, _) {
          final t = _curve.transform(_shiftAnim.value);
          return ClipRect(
            child: Stack(clipBehavior: Clip.hardEdge, children: [
              for (final it in shift.carried)
                Positioned.fromRect(key: ValueKey('face-${it.id}'), rect: Rect.lerp(shift.from[it.id], shift.to[it.id], t)!, child: _faceWidget(it, shift.to[it.id]!)),
            ]),
          );
        },
      );
    }
    final frame = _placed(widget.view, items, box.maxWidth, widget.minColumn);
    if (items.isEmpty) return const SizedBox.shrink();
    if (widget.view == 'list') {
      return ListView.builder(key: const ValueKey('list'), controller: _scroll, physics: const ClampingScrollPhysics(), itemExtent: FluidBoard.rowHeight, itemCount: items.length, scrollCacheExtent: const ScrollCacheExtent.pixels(240), addAutomaticKeepAlives: false, itemBuilder: (context, i) => _tile(items[i], frame));
    }
    return CustomScrollView(
      key: const ValueKey('thumbnail'),
      controller: _scroll,
      physics: const ClampingScrollPhysics(),
      scrollCacheExtent: const ScrollCacheExtent.pixels(320),
      slivers: [
        SliverGrid(
          gridDelegate: _PlacedGrid([for (final it in items) _tileRect(it, frame)], frame.content.height),
          delegate: SliverChildBuilderDelegate((context, i) => _tile(items[i], frame), childCount: items.length, addAutomaticKeepAlives: false),
        ),
      ],
    );
  }

  /// The box an asset's face and its name stand in together (a row of the list, a card of the masonry).
  Rect _tileRect(BrowserItem it, Frame frame) {
    final face = frame.faces[it.id]!, label = frame.labels[it.id];
    if (widget.view == 'list') return Rect.fromLTWH(0, label?.top ?? face.top, _viewport.width, FluidBoard.rowHeight);
    return label == null ? face : face.expandToInclude(label);
  }

  Widget _tile(BrowserItem it, Frame frame) {
    final box = _tileRect(it, frame), face = frame.faces[it.id]!, label = frame.labels[it.id];
    return Stack(clipBehavior: Clip.none, children: [
      if (label != null) Positioned.fromRect(key: ValueKey('label-${it.id}'), rect: label.shift(-box.topLeft), child: _labelWidget(it, label.width)),
      Positioned.fromRect(key: ValueKey('face-${it.id}'), rect: face.shift(-box.topLeft), child: _faceWidget(it, face)),
    ]);
  }

  /// A face: what a press chooses, a double click places, a right click asks, a drag carries, a pointer only draws a hairline on.
  Widget _faceWidget(BrowserItem it, Rect rect, {bool dim = false, Color? tint, void Function(bool on)? onHover}) => GestureDetector(
        behavior: HitTestBehavior.opaque,
        onDoubleTap: widget.onOpen == null ? null : () => widget.onOpen!(it.id),
        onSecondaryTapDown: widget.onMenu == null ? null : (d) => widget.onMenu!(it.id, d.globalPosition),
        child: _press(it.id, MouseRegion(
          onEnter: onHover == null ? null : (_) => onHover(true),
          onExit: onHover == null ? null : (_) => onHover(false),
          child: Opacity(
            opacity: dim ? .28 : 1,
            child: _tinted(tint, _carried(it, rect, _Face(it, selected: widget.picked.contains(it.id) || it.id == widget.selected, marked: widget.view != 'list', faces: widget.faces, held: widget.holding[it.id], short: widget.view == 'explore' && it.id != widget.selected))),
          ),
        )),
      );

  /// An asset's name beside or under its face. Each label is laid out at the width it was made for (the box may still be
  /// growing or shrinking, and an outgoing label keeps its own): clipped, never squeezed.
  Widget _labelWidget(BrowserItem it, double width, {bool hush = false}) => GestureDetector(
        behavior: HitTestBehavior.opaque,
        onDoubleTap: widget.onOpen == null ? null : () => widget.onOpen!(it.id),
        onSecondaryTapDown: widget.onMenu == null ? null : (d) => widget.onMenu!(it.id, d.globalPosition),
        child: _press(it.id, ClipRect(
          child: AnimatedSwitcher(
            duration: const Duration(milliseconds: 120),
            reverseDuration: Duration.zero,
            child: KeyedSubtree(
              key: ValueKey('${widget.view}-${it.id}'),
              child: OverflowBox(
                alignment: Alignment.topLeft,
                minWidth: width,
                maxWidth: width,
                minHeight: 0,
                maxHeight: double.infinity,
                child: AnimatedOpacity(opacity: hush ? 0 : 1, duration: Duration(milliseconds: hush ? 0 : 120), child: _label(it, width)),
              ),
            ),
          ),
        )),
      );

  /// Choosing happens on the press itself, whatever else the face answers to: a tap handler beside a double-tap one waits out
  /// the double-tap window, and a click that lands late feels like lag. Only the primary button chooses (the other opens a menu).
  Widget _press(String id, Widget child) => Listener(onPointerDown: (e) {
        if (e.buttons & kPrimaryButton != 0) widget.onTap(id);
      }, child: child);

  /// A face with a thin colour round it (metadata such as its type; the picture is untouched).
  Widget _tinted(Color? tint, Widget face) => tint == null ? face : Stack(fit: StackFit.expand, children: [face, IgnorePointer(child: DecoratedBox(decoration: BoxDecoration(borderRadius: BorderRadius.circular(3), border: Border.all(color: tint, width: 1.5))))]);

  /// A face that can be carried (to the Timeline) is dragged as itself; one that cannot is only the face.
  Widget _carried(BrowserItem it, Rect rect, Widget face) {
    final data = widget.carry?.call(it);
    if (data == null) return face;
    return Draggable<Map<String, dynamic>>(
      data: data,
      dragAnchorStrategy: pointerDragAnchorStrategy,
      feedback: IgnorePointer(child: Opacity(opacity: .85, child: SizedBox(width: math.max(48.0, math.min(rect.width, 120.0)), height: math.max(32.0, math.min(rect.height, 90.0)), child: ClipRRect(borderRadius: BorderRadius.circular(3), child: materialFace(it.shelf))))),
      childWhenDragging: Opacity(opacity: .4, child: face),
      child: face,
    );
  }

  Widget _label(BrowserItem it, double width) {
    final on = it.id == widget.selected;
    switch (widget.view) {
      case 'list':
        return Row(children: [
          Expanded(child: Text(it.name, maxLines: 1, softWrap: false, overflow: TextOverflow.ellipsis, style: Dn.name(on ? N.g100 : N.g91))),
          const SizedBox(width: 8),
          SizedBox(width: 42, child: Text(it.typeWord, softWrap: false, style: Dn.label(N.g63))),
          SizedBox(width: 40, child: Text(clockText(it.seconds), softWrap: false, style: Dn.value(N.g69).copyWith(fontSize: 10))),
          if (MediaListHeader.wide(width + 62)) SizedBox(width: 54, child: Text(sizeText(it.size), softWrap: false, textAlign: TextAlign.right, style: Dn.value(N.g69).copyWith(fontSize: 10))),
          if (MediaListHeader.roomy(width + 62)) SizedBox(width: 74, child: Text(dateText(it.mtimeNs), softWrap: false, textAlign: TextAlign.right, style: Dn.value(N.g56).copyWith(fontSize: 10))),
          const SizedBox(width: 8),
        ]);
      case 'thumbnail':
      case 'explore':
        return Align(alignment: widget.view == 'explore' ? Alignment.topCenter : Alignment.bottomLeft, child: Text(it.name, maxLines: 1, softWrap: false, overflow: TextOverflow.ellipsis, textAlign: widget.view == 'explore' ? TextAlign.center : TextAlign.start, style: Dn.label(on ? N.g100 : N.g82, FontWeight.w500)));
      default:
        return const SizedBox.shrink();
    }
  }
}

/// The rectangles the masonry already decided, given to Flutter's own SliverGrid: it builds only the children whose rectangles reach
/// the viewport. The children are in the library's order, which is not quite top-to-bottom (the shortest column takes the next one), so
/// the index bounds come from running maxima / minima, which are exact.
class _PlacedGrid extends SliverGridDelegate {
  _PlacedGrid(this.rects, this.height);
  final List<Rect> rects;
  final double height;

  @override
  SliverGridLayout getLayout(SliverConstraints constraints) => _PlacedLayout(rects, height);

  @override
  bool shouldRelayout(_PlacedGrid old) => !identical(old.rects, rects) && (old.rects.length != rects.length || old.height != height || !_same(old.rects, rects));

  static bool _same(List<Rect> a, List<Rect> b) {
    for (var i = 0; i < a.length; i++) {
      if (a[i] != b[i]) return false;
    }
    return true;
  }
}

class _PlacedLayout extends SliverGridLayout {
  _PlacedLayout(this.rects, this.height)
      : _reach = _running(rects, (r) => r.bottom, math.max),
        _start = _suffix(rects, (r) => r.top, math.min);
  final List<Rect> rects;
  final double height;

  /// `_reach[i]`: how far down the first i+1 children go. `_start[i]`: how far up the children from i on begin.
  final List<double> _reach, _start;

  static List<double> _running(List<Rect> r, double Function(Rect) f, double Function(double, double) pick) {
    final out = List<double>.filled(r.length, 0);
    for (var i = 0; i < r.length; i++) {
      out[i] = i == 0 ? f(r[i]) : pick(out[i - 1], f(r[i]));
    }
    return out;
  }

  static List<double> _suffix(List<Rect> r, double Function(Rect) f, double Function(double, double) pick) {
    final out = List<double>.filled(r.length, 0);
    for (var i = r.length - 1; i >= 0; i--) {
      out[i] = i == r.length - 1 ? f(r[i]) : pick(out[i + 1], f(r[i]));
    }
    return out;
  }

  @override
  int getMinChildIndexForScrollOffset(double scrollOffset) {
    var lo = 0, hi = rects.length;
    while (lo < hi) {
      final mid = (lo + hi) ~/ 2;
      _reach[mid] > scrollOffset ? hi = mid : lo = mid + 1;
    }
    return math.min(lo, math.max(0, rects.length - 1));
  }

  @override
  int getMaxChildIndexForScrollOffset(double scrollOffset) {
    var lo = 0, hi = rects.length;
    while (lo < hi) {
      final mid = (lo + hi) ~/ 2;
      _start[mid] <= scrollOffset ? lo = mid + 1 : hi = mid;
    }
    return math.max(0, lo - 1);
  }

  @override
  SliverGridGeometry getGeometryForChildIndex(int index) {
    final r = rects[index];
    return SliverGridGeometry(scrollOffset: r.top, crossAxisOffset: r.left, mainAxisExtent: r.height, crossAxisExtent: r.width);
  }

  @override
  double computeMaxScrollOffset(int childCount) => height;
}

/// Moves a child from where it stood to a new rectangle, along one path (position and size together): at once, slowing
/// only as it lands. A small change is a jump, and `instant` (a size being dragged) is 1:1 with no motion at all.
class _Glide extends StatefulWidget {
  const _Glide({super.key, required this.rect, this.instant = false, required this.child});
  final Rect rect;
  final bool instant;
  final Widget child;
  @override
  State<_Glide> createState() => _GlideState();
}

class _GlideState extends State<_Glide> with SingleTickerProviderStateMixin {
  late final AnimationController _c = AnimationController(vsync: this, duration: _FluidBoardState._move);
  late Rect _from = widget.rect, _to = widget.rect;

  @override
  void didUpdateWidget(_Glide old) {
    super.didUpdateWidget(old);
    if (old.rect == widget.rect) return;
    // start from wherever it is now (a change of mind mid-flight continues from there)
    final here = _now;
    final far = (here.topLeft - widget.rect.topLeft).distance > 6 || math.max((here.width - widget.rect.width).abs(), (here.height - widget.rect.height).abs()) > 6;
    _from = here;
    _to = widget.rect;
    if (widget.instant || !far) {
      _from = _to;
      _c.value = 1;
    } else {
      _c.forward(from: 0); // the first frame after the input already shows movement (ease-out starts fast)
    }
  }

  Rect get _now => Rect.lerp(_from, _to, _FluidBoardState._curve.transform(_c.value))!;

  @override
  void dispose() {
    _c.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => AnimatedBuilder(
        animation: _c,
        child: widget.child,
        builder: (context, child) {
          final r = _now;
          return Positioned(left: r.left, top: r.top, width: r.width, height: r.height, child: child!);
        },
      );
}

/// A face keeps its whole picture whatever the pointer does: a pointer over it only draws a hairline. A clip, a sound or a
/// model answers a drag across it (scrub, position, turn); a click still chooses. A still has nothing to drag.
class _Face extends StatefulWidget {
  const _Face(this.item, {required this.selected, required this.marked, this.faces, this.held, this.short = false});
  final bool short;
  final Offset? held;
  final BrowserItem item;
  final bool selected, marked;
  final FaceService? faces;
  @override
  State<_Face> createState() => _FaceState();
}

class _FaceState extends State<_Face> {
  bool hover = false; // a pointer is over it: a hairline only, nothing moves or is cropped
  double? at; // 0..1 along a clip's scrub track while a pointer is on it
  String? frame; // the clip's frame at that place
  bool _busy = false;
  double? _wanted;

  BrowserItem get item => widget.item;

  @override
  void initState() {
    super.initState();
    if (widget.held != null) _scrub(widget.held!.dx);
  }

  /// A clip answers a pointer pressed on its scrub track (the strip along its foot) by showing the frame there; the rest of
  /// the face is for choosing and carrying, so dragging it never scrubs.
  void _scrub(double p) {
    setState(() => at = p.clamp(0.0, 1.0));
    final faces = widget.faces, seconds = item.seconds;
    if (faces != null && seconds != null) {
      _wanted = at! * seconds;
      if (_busy) return;
      _busy = true;
      () async {
        while (_wanted != null && mounted && at != null) {
          final t = _wanted!;
          _wanted = null;
          final got = await faces.frameAt(item, t, edge: 240);
          if (mounted && at != null && got != null) setState(() => frame = got);
        }
        _busy = false;
      }();
    }
  }

  void _release() {
    if (at == null) return;
    setState(() {
      at = null;
      frame = null;
      _wanted = null;
    });
  }

  Widget _content() {
    final base = materialFace(item.shelf);
    if (item.kind == 'model' && widget.faces != null) return ModelFace(path: item.path, fallback: base);
    if (item.kind != 'video') return base;
    final bytes = frame == null ? null : _decode(frame!);
    return Stack(fit: StackFit.expand, children: [
      bytes == null ? base : Image.memory(bytes, fit: BoxFit.cover, gaplessPlayback: true),
      // the scrub track at rest (faint), and its position while a pointer is on it
      Align(alignment: Alignment.bottomLeft, child: Container(height: at == null ? 2 : 4, color: at == null ? const Color(0x40FFFFFF) : const Color(0x99000000), alignment: Alignment.centerLeft, child: at == null ? null : FractionallySizedBox(widthFactor: at, child: Container(height: 3, color: N.g100)))),
    ]);
  }

  @override
  Widget build(BuildContext context) => LayoutBuilder(builder: (context, box) => MouseRegion(
        onEnter: (_) => setState(() => hover = true),
        onExit: (_) => setState(() => hover = false),
        child: Stack(fit: StackFit.expand, children: [
          ClipRRect(borderRadius: BorderRadius.circular(3), child: ColoredBox(color: N.g13, child: _content())),
          if (widget.marked && item.mark.isNotEmpty && at == null) Positioned(left: 2.5, top: 2.5, child: Container(padding: const EdgeInsets.symmetric(horizontal: 3, vertical: 1.5), decoration: BoxDecoration(color: N.veil, borderRadius: BorderRadius.circular(2)), child: Text(widget.short ? item.mark.split(' ').first : item.mark, softWrap: false, style: Dn.micro(N.g95).copyWith(fontSize: 8.5)))),
          if (item.used && !item.missing && box.maxWidth > 30) Positioned(right: 3, top: 3, child: Container(width: 5, height: 5, decoration: BoxDecoration(color: N.g95, shape: BoxShape.circle, border: Border.all(color: N.g07.withValues(alpha: .5))))),
          if (item.favorite && box.maxWidth > 30) Positioned(left: 3, top: 3, child: Text('★', style: Dn.micro(N.g95).copyWith(fontSize: 10, height: 1, shadows: const [Shadow(color: Color(0xCC000000), blurRadius: 2)]))),
          if (item.missing) Positioned(right: 3, top: 3, child: Container(padding: const EdgeInsets.symmetric(horizontal: 3), decoration: BoxDecoration(color: const Color(0xFFFFD166), borderRadius: BorderRadius.circular(2)), child: Text('!', style: Dn.micro(N.g00).copyWith(fontSize: 9, fontWeight: FontWeight.w700)))),
          // hover: a hairline only (the picture is neither scaled nor cropped, and nothing moves)
          if (hover && !widget.selected) IgnorePointer(child: DecoratedBox(decoration: BoxDecoration(borderRadius: BorderRadius.circular(3), border: Border.all(color: N.g69, width: 1)))),
          if (widget.selected) const IgnorePointer(child: _Ring()),
          if (item.kind == 'video' && box.maxWidth > 40 && box.maxHeight > 24)
            Positioned(
              left: 0,
              right: 0,
              bottom: 0,
              height: 12,
              child: Listener(
                behavior: HitTestBehavior.opaque,
                onPointerDown: (e) => _scrub(e.localPosition.dx / box.maxWidth),
                onPointerMove: (e) => _scrub(e.localPosition.dx / box.maxWidth),
                onPointerUp: (_) => _release(),
                onPointerCancel: (_) => _release(),
                child: const MouseRegion(cursor: SystemMouseCursors.resizeLeftRight),
              ),
            ),
        ]),
      ));
}

/// The chosen face's ring: a light line with a dark line outside it, so it reads on a bright picture and on a black tile,
/// and the picture inside is untouched.
class _Ring extends StatelessWidget {
  const _Ring();
  @override
  Widget build(BuildContext context) => Stack(clipBehavior: Clip.none, fit: StackFit.expand, children: [
        Positioned(left: -1.5, top: -1.5, right: -1.5, bottom: -1.5, child: DecoratedBox(decoration: BoxDecoration(borderRadius: BorderRadius.circular(4.5), border: Border.all(color: const Color(0xB3000000), width: 1.5)))),
        DecoratedBox(decoration: BoxDecoration(borderRadius: BorderRadius.circular(3), border: Border.all(color: N.g100, width: 2.5))),
      ]);
}

Uint8List? _decode(String dataUri) {
  final comma = dataUri.indexOf(',');
  try {
    return comma < 0 ? null : base64Decode(dataUri.substring(comma + 1));
  } catch (_) {
    return null;
  }
}

class _Spokes extends CustomPainter {
  _Spokes(this.from, this.to);
  final Offset from;
  final List<Offset> to;
  @override
  void paint(Canvas canvas, Size size) {
    final p = Paint()
      ..color = N.g51
      ..strokeWidth = 1;
    for (final t in to) {
      canvas.drawLine(from, t, p);
    }
  }

  @override
  bool shouldRepaint(_Spokes o) => o.from != from || o.to.length != to.length || [for (var i = 0; i < to.length; i++) if (to[i] != o.to[i]) 1].isNotEmpty;
}
