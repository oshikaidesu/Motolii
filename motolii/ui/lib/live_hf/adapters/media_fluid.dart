import 'dart:async';
import 'dart:convert';
import 'dart:math' as math;
import 'dart:typed_data';

import 'package:flutter/scheduler.dart';
import 'package:flutter/widgets.dart';

import '../../hf/metrics.dart';
import '../../hf/neutral.dart';
import 'browser_item.dart';
import 'media_library.dart' show materialFace;
import 'media_list.dart' show MediaListHeader;
import 'media_preview.dart' show FaceService;
import 'model_face.dart';

/// Where every face of a projection stands: faces and the labels by them, in one scrolling content.
typedef Frame = ({Map<String, Rect> faces, Map<String, Rect> labels, Size content, List<String> links});

/// Explore's places: given by the caller (a prototype: there is no similarity backend).
typedef ExploreLayout = Frame Function(List<BrowserItem> items, String? selected, Size viewport);

/// The three projections of one Result Set as one board: every asset is one face, placed by the projection in force, and a
/// change of projection moves the same faces (the selection, the identity and the scroll context stay). List lays them in
/// rows with their facts beside, Thumbnail packs them at their own shapes with a name under each, Explore is the caller's.
class FluidBoard extends StatefulWidget {
  const FluidBoard({super.key, required this.items, required this.selected, required this.view, required this.onTap, this.onOpen, this.explore, this.faces, this.holding = const {}, this.minColumn = defaultColumn, this.revealOn = false});
  final List<BrowserItem> items;

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
    return (faces: faces, labels: labels, links: const <String>[], content: Size(width, heights.fold(0.0, math.max) + margin));
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
    return (faces: faces, labels: labels, links: const <String>[], content: Size(width, items.length * rowHeight));
  }

  @override
  State<FluidBoard> createState() => _FluidBoardState();
}

class _FluidBoardState extends State<FluidBoard> {
  final scroll = ScrollController();

  /// While the faces are on their way the labels stay out (they would run into one another); they return when the faces land.
  bool _moving = false;
  Timer? _landed;
  void _hush() {
    _landed?.cancel();
    setState(() => _moving = true);
    _landed = Timer(const Duration(milliseconds: 420), () {
      if (mounted) setState(() => _moving = false);
    });
  }


  @override
  void dispose() {
    _landed?.cancel();
    scroll.dispose();
    super.dispose();
  }

  Frame _frame(Size viewport) => switch (widget.view) {
        'list' => FluidBoard.list(widget.items, viewport.width),
        'explore' when widget.explore != null => widget.explore!(widget.items, widget.selected, viewport),
        _ => FluidBoard.thumbnail(widget.items, viewport.width, widget.minColumn),
      };

  @override
  void didUpdateWidget(FluidBoard old) {
    super.didUpdateWidget(old);
    if (old.revealOn != widget.revealOn) {
      SchedulerBinding.instance.addPostFrameCallback((_) => _reveal());
    }
    if (old.view != widget.view || old.minColumn != widget.minColumn) {
      _hush();
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
                // Explore: a line from the chosen asset to each of its nearest (what the layout says is near, nothing more)
                if (widget.view == 'explore' && frame.links.isNotEmpty && frame.faces[widget.selected] != null)
                  Positioned.fill(
                    child: IgnorePointer(
                      child: AnimatedOpacity(
                        opacity: _moving ? 0 : 1,
                        duration: Duration(milliseconds: _moving ? 60 : 300),
                        child: CustomPaint(painter: _Spokes(frame.faces[widget.selected]!.center, [for (final id in frame.links) if (frame.faces[id] != null) frame.faces[id]!.center])),
                      ),
                    ),
                  ),
                for (final it in widget.items) ...[
                  if (frame.labels[it.id] != null)
                    _Glide(
                      key: ValueKey('label-${it.id}'),
                      rect: frame.labels[it.id]!,
                      delay: _delay(it),
                      child: GestureDetector(
                        behavior: HitTestBehavior.opaque,
                        onTap: () => widget.onTap(it.id),
                        onDoubleTap: widget.onOpen == null ? null : () => widget.onOpen!(it.id),
                        // each label is laid out at the width it was made for (the box is still growing or shrinking, and an
                        // outgoing label keeps its own): clipped, never squeezed
                        child: ClipRect(
                          child: AnimatedSwitcher(
                            duration: const Duration(milliseconds: 220),
                            reverseDuration: const Duration(milliseconds: 70),
                            child: KeyedSubtree(
                              key: ValueKey('${widget.view}-${it.id}'),
                              child: OverflowBox(
                                alignment: Alignment.topLeft,
                                minWidth: frame.labels[it.id]!.width,
                                maxWidth: frame.labels[it.id]!.width,
                                minHeight: frame.labels[it.id]!.height,
                                maxHeight: frame.labels[it.id]!.height,
                                child: AnimatedOpacity(opacity: _moving ? 0 : 1, duration: Duration(milliseconds: _moving ? 60 : 180), child: _label(it, frame.labels[it.id]!.width)),
                              ),
                            ),
                          ),
                        ),
                      ),
                    ),
                  if (frame.faces[it.id] != null)
                    _Glide(
                      key: ValueKey('face-${it.id}'),
                      rect: frame.faces[it.id]!,
                      delay: _delay(it),
                      child: GestureDetector(
                        behavior: HitTestBehavior.opaque,
                        onTap: () => widget.onTap(it.id),
                        onDoubleTap: widget.onOpen == null ? null : () => widget.onOpen!(it.id),
                        child: _Face(it, selected: it.id == widget.selected, marked: widget.view != 'list', faces: widget.faces, held: widget.holding[it.id], short: widget.view == 'explore' && it.id != widget.selected),
                      ),
                    ),
                ],
              ]),
            ),
          ),
        );
      });

  /// Faces set off one after another in the order the list shows them (a ripple, not everything at once), so a path can be
  /// followed: the first leaves at once, the last some 170 ms later.
  Duration _delay(BrowserItem it) {
    final i = widget.items.indexOf(it);
    final n = math.max(1, widget.items.length - 1);
    return Duration(milliseconds: (170 * i / n).round());
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

/// A face is a small instrument for what it is, under a passing pointer (no click, so choosing and carrying stay as they
/// are): a clip scrubs to where the pointer is, a sound shows a position, a model turns, a panorama pans and a still zooms.
/// Moves a child from where it stood to a new rectangle, after a delay, along one smooth path (position and size together).
class _Glide extends StatefulWidget {
  const _Glide({super.key, required this.rect, required this.delay, required this.child});
  final Rect rect;
  final Duration delay;
  final Widget child;
  @override
  State<_Glide> createState() => _GlideState();
}

class _GlideState extends State<_Glide> with SingleTickerProviderStateMixin {
  late final AnimationController _c = AnimationController(vsync: this, duration: const Duration(milliseconds: 360));
  late Rect _from = widget.rect, _to = widget.rect;
  Timer? _wait;

  @override
  void didUpdateWidget(_Glide old) {
    super.didUpdateWidget(old);
    if (old.rect == widget.rect) return;
    // start from wherever it is now (a change of mind mid-flight continues from there)
    _from = _now;
    _to = widget.rect;
    _wait?.cancel();
    if (widget.delay == Duration.zero) {
      _c.forward(from: 0);
    } else {
      _c.value = 0;
      _wait = Timer(widget.delay, () => mounted ? _c.forward(from: 0) : null);
    }
  }

  Rect get _now => Rect.lerp(_from, _to, Curves.easeInOutCubic.transform(_c.value))!;

  @override
  void dispose() {
    _wait?.cancel();
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
  Offset? at; // the pointer, 0..1 across the tile
  String? frame; // a clip's frame at the pointer
  bool _busy = false;
  double? _wanted;

  BrowserItem get item => widget.item;

  @override
  void initState() {
    super.initState();
    if (widget.held != null) _at(widget.held!);
  }

  void _over(PointerEvent e, Size size) {
    if (size.width <= 0 || size.height <= 0) return;
    _at(Offset((e.localPosition.dx / size.width).clamp(0.0, 1.0), (e.localPosition.dy / size.height).clamp(0.0, 1.0)));
  }

  void _at(Offset p) {
    setState(() => at = p);
    final faces = widget.faces, seconds = item.seconds;
    if (item.kind == 'video' && faces != null && seconds != null) {
      _wanted = p.dx * seconds;
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

  void _left() => setState(() {
        at = null;
        frame = null;
        _wanted = null;
      });

  Widget _content() {
    final p = at;
    final base = materialFace(item.shelf);
    if (p == null) {
      if (item.kind == 'model' && widget.faces != null) return ModelFace(path: item.path, fallback: base);
      // a clip carries its scrub track at rest (faint), so it reads as something to run a pointer along
      if (item.kind == 'video') return Stack(fit: StackFit.expand, children: [base, Align(alignment: Alignment.bottomCenter, child: Container(height: 2, color: const Color(0x40FFFFFF)))]);
      return base;
    }
    switch (item.kind) {
      case 'video':
        final bytes = frame == null ? null : _decode(frame!);
        return Stack(fit: StackFit.expand, children: [
          bytes == null ? base : Image.memory(bytes, fit: BoxFit.cover, gaplessPlayback: true),
          Align(alignment: Alignment.bottomLeft, child: Container(height: 4, color: const Color(0x99000000), alignment: Alignment.centerLeft, child: FractionallySizedBox(widthFactor: p.dx, child: Container(height: 3, color: N.g100)))),
        ]);
      case 'audio':
        return Stack(fit: StackFit.expand, children: [base, Align(alignment: Alignment(p.dx * 2 - 1, 0), child: Container(width: 1.5, color: N.g100))]);
      case 'model':
        return ModelFace(path: item.path, fallback: base, hoverYaw: p.dx);
      case 'environment':
      case 'image':
        return Transform.scale(scale: item.kind == 'environment' ? 2.4 : 2.6, alignment: FractionalOffset(p.dx, p.dy), child: base);
      default:
        return base;
    }
  }

  @override
  Widget build(BuildContext context) => LayoutBuilder(builder: (context, box) => MouseRegion(
        onHover: (e) => _over(e, box.biggest),
        onExit: (_) => _left(),
        child: Stack(fit: StackFit.expand, children: [
          ClipRRect(borderRadius: BorderRadius.circular(3), child: ColoredBox(color: N.g13, child: _content())),
          if (widget.marked && item.mark.isNotEmpty && at == null) Positioned(left: 2.5, top: 2.5, child: Container(padding: const EdgeInsets.symmetric(horizontal: 3, vertical: 1.5), decoration: BoxDecoration(color: N.veil, borderRadius: BorderRadius.circular(2)), child: Text(widget.short ? item.mark.split(' ').first : item.mark, softWrap: false, style: Dn.micro(N.g95).copyWith(fontSize: 8.5)))),
          if (widget.selected) const IgnorePointer(child: _Ring()),
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
