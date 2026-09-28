import 'dart:math' as math;
import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import '../bp/common.dart';
import 'common.dart';
import '../shell/menu.dart' show showHfMenu;
import '../neutral.dart';

class NBlock {
  NBlock(this.kind, this.pos, this.size, this.text, [this.tint = 0, this.id, this.png]);
  final String kind; // note | image | ref | hand
  Offset pos;
  Size size;
  String text;
  final int tint;

  /// The host's id for this block, and an image block's picture.
  final String? id;
  final Uint8List? png;
}

/// A host that keeps the notebook: its pages and blocks, and where a changed block goes. [can] says which kinds of
/// block it holds.
abstract class NotesHost implements Listenable {
  int get pageCount;
  List<NBlock> blocks(int page);
  bool can(String kind);
  void put(int page, NBlock block);
  void patch(int page, NBlock block, Map<String, dynamic> changed);
  void remove(int page, NBlock block);

  /// A new empty page after the last; its index once it exists.
  Future<int?> addPage();
  Future<void> deletePage(int page);

  /// False for the chip an empty notebook still shows: there is no page there to delete.
  bool hasPage(int page);

  /// Cmd+V on the desk: what the clipboard holds becomes a block at [at] (a picture, else its text).
  Future<void> paste(int page, Offset at);

  /// A double click on a reference card: go where it points (its layer, its first frame).
  void follow(int page, NBlock block);

  /// The page the desk shows (where dropped pictures land).
  void showing(int page);
}

/// Notes is a free 2D workbench. Wide shows it at working scale; narrow shows the same canvas fitted small.
/// Blocks are the same objects in both, and stay selectable and movable.
class NotesDesk extends StatefulWidget {
  const NotesDesk({super.key, this.host});
  final NotesHost? host;
  @override
  State<NotesDesk> createState() => _NotesDeskState();
}

class _NotesDeskState extends State<NotesDesk> {
  final _blocks = <NBlock>[
    NBlock('note', const Offset(20, 24), const Size(120, 88), 'Anticipation\nbefore the move,\nthen a soft settle', 0),
    NBlock('note', const Offset(160, 44), const Size(112, 76), 'Try a slower\nease here.', 1),
    NBlock('image', const Offset(30, 136), const Size(122, 90), ''),
    NBlock('ref', const Offset(174, 150), const Size(104, 28), 'Layer 2'),
    NBlock('ref', const Offset(174, 186), const Size(104, 28), 'Camera 1'),
    NBlock('hand', const Offset(36, 244), const Size(220, 46), 'motion is feeling'),
  ];
  int? selected = 0;
  int? editing;
  double zoom = 1;
  Offset pan = Offset.zero;
  Offset npan = Offset.zero;
  Rect? _fitFrom; // the narrow view keeps its frame while blocks move; it re-fits only when asked or a block is added
  int tool = 0;
  int page = 0;
  List<NBlock> _held = const [];
  List<NBlock> get blocks => widget.host == null ? _blocks : _held;
  final _ctl = TextEditingController();

  @override
  void initState() {
    super.initState();
    widget.host?.addListener(_absorb);
    _absorb();
  }

  void _absorb() {
    final h = widget.host;
    if (h == null) return;
    setState(() {
      if (page >= h.pageCount) page = 0;
      _held = h.blocks(page);
      if (selected != null && selected! >= _held.length) selected = null;
      if (editing != null && editing! >= _held.length) editing = null;
    });
  }

  /// Leaving a note being typed into: its text goes to the host.
  void _endEdit() {
    final e = editing;
    if (e != null && e < blocks.length) widget.host?.patch(page, blocks[e], {'text': blocks[e].text});
  }
  final _focus = FocusNode();
  final _canvasFocus = FocusNode();

  @override
  void dispose() {
    widget.host?.removeListener(_absorb);
    _ctl.dispose();
    _focus.dispose();
    _canvasFocus.dispose();
    super.dispose();
  }

  Rect get _bounds {
    if (blocks.isEmpty) return const Rect.fromLTWH(0, 0, 280, 280);
    var r = blocks.first.pos & blocks.first.size;
    for (final b in blocks) { r = r.expandToInclude(b.pos & b.size); }
    return r;
  }

  int? _told;
  void _tell() {
    if (widget.host != null && _told != page) {
      _told = page;
      widget.host!.showing(page);
    }
  }

  @override
  Widget build(BuildContext context) {
    _tell();
    return _build(context);
  }

  Widget _build(BuildContext context) => DeskShell(
        kind: DeskKind.notes,
        title: 'Notes',
        subtitle: 'PAGES · REFERENCES',
        full: (c, s) => Column(children: [
          _toolbar(),
          Expanded(child: LayoutBuilder(builder: (context, b) => _canvas(Size(b.maxWidth, b.maxHeight), false))),
          _zoomBar(),
        ]),
        strip: (c, s) => Padding(padding: const EdgeInsets.fromLTRB(8, 2, 8, 8), child: LayoutBuilder(builder: (context, b) => _canvas(Size(b.maxWidth, b.maxHeight), true))),
        tall: (c, s) => Padding(padding: const EdgeInsets.fromLTRB(8, 4, 8, 8), child: LayoutBuilder(builder: (context, b) => _canvas(Size(b.maxWidth, b.maxHeight), true))),
      );

  Widget _toolbar() => Container(
        height: 44,
        padding: const EdgeInsets.symmetric(horizontal: 10),
        child: Row(children: [
          for (var i = 0; i < 5; i++) Padding(padding: const EdgeInsets.only(right: 4), child: _tool(i)),
          const Spacer(),
          for (var p = 0; p < (widget.host?.pageCount ?? 3); p++)
            GestureDetector(
              onTap: () {
                _endEdit();
                setState(() { page = p; selected = null; editing = null; });
                _absorb();
              },
              onSecondaryTapDown: widget.host == null ? null : (e) => _pageMenu(p, e.globalPosition),
              child: Container(width: 26, height: 26, margin: const EdgeInsets.only(left: 4), alignment: Alignment.center, decoration: BoxDecoration(border: Border.all(color: p == page ? kAccent : kRule2), borderRadius: BorderRadius.circular(3), color: p == page ? kAccentDim.withValues(alpha: .4) : null), child: Text('${p + 1}', style: sans(11, c: p == page ? kInk : kMuted))),
            ),
        ]),
      );

  /// A page chip's right-click: a new page after the last, or this page gone.
  Future<void> _pageMenu(int p, Offset at) async {
    final h = widget.host!;
    final chosen = await showHfMenu<String>(context, Rect.fromLTWH(at.dx, at.dy, 160, 0), const [('new', 'New page'), ('delete', 'Delete page')], disabled: {if (!h.hasPage(p)) 'delete'});
    if (!mounted || chosen == null) return;
    _endEdit();
    if (chosen == 'new') {
      final added = await h.addPage();
      if (mounted && added != null) setState(() { page = added; selected = null; editing = null; });
    } else {
      await h.deletePage(p);
      // the page being looked at stays in view: one earlier when a page before it went
      if (mounted) setState(() { page = math.max(0, math.min(p < page ? page - 1 : page, h.pageCount - 1)); selected = null; editing = null; });
    }
    _absorb();
  }

  void _add(String kind) {
    final h = widget.host;
    if (h != null && !h.can(kind)) return;
    setState(() {
      final c = (Offset(140, 160) - pan) / zoom - Offset(blocks.length * 3.0 % 24, blocks.length * 3.0 % 24);
      final size = switch (kind) { 'note' => const Size(110, 74), 'image' => const Size(110, 80), _ => const Size(100, 28) };
      final b = NBlock(kind, c - Offset(size.width / 2, size.height / 2), size, kind == 'note' ? 'New note' : (kind == 'ref' ? 'Selection' : ''), blocks.length % 2);
      if (h == null) {
        _blocks.add(b);
      } else {
        h.put(page, b);
      }
      selected = blocks.length - (h == null ? 1 : 0);
      _fitFrom = null;
    });
  }

  Widget _tool(int i) => GestureDetector(
        key: ValueKey('notes-tool-$i'),
        onTap: () {
          setState(() => tool = i);
          if (i == 1) _add('note');
          if (i == 2) _add('image');
          if (i == 3) _add('ref');
        },
        child: Container(
          height: 30,
          width: 30,
          decoration: BoxDecoration(color: i == tool ? kAccentDim.withValues(alpha: .4) : null, border: Border.all(color: i == tool ? kAccent : kRule2), borderRadius: BorderRadius.circular(3)),
          child: CustomPaint(painter: _ToolP(i)),
        ),
      );

  void _fit(Size s) {
    final b = _bounds.inflate(14);
    zoom = math.min(2.0, math.min(s.width / b.width, s.height / b.height));
    pan = Offset(s.width / 2, s.height / 2) - b.center * zoom;
  }

  Widget _zoomBar() => Container(
        height: 40,
        padding: const EdgeInsets.symmetric(horizontal: 12),
        child: Row(children: [
          Text('Page ${page + 1}  ·  ${blocks.length} blocks', style: sans(11, c: kMuted)),
          const Spacer(),
          GestureDetector(key: const ValueKey('zoom-out'), onTap: () => setState(() => zoom = clampD(zoom - .25, .25, 3)), child: Padding(padding: const EdgeInsets.symmetric(horizontal: 6), child: Text('−', style: sans(18, c: N.g82)))),
          SizedBox(width: 48, child: Text('${(zoom * 100).round()}%', key: const ValueKey('zoom-label'), textAlign: TextAlign.center, style: mono(11, c: N.g82))),
          GestureDetector(key: const ValueKey('zoom-in'), onTap: () => setState(() => zoom = clampD(zoom + .25, .25, 3)), child: Padding(padding: const EdgeInsets.symmetric(horizontal: 6), child: Text('+', style: sans(18, c: N.g82)))),
          const SizedBox(width: 10),
          GestureDetector(key: const ValueKey('zoom-fit'), onTap: () => setState(() => _fit(const Size(310, 490))), child: Text('Fit', style: sans(11, c: kAccent))),
        ]),
      );

  Widget _canvas(Size s, bool fit) {
    var z = zoom;
    var p = pan;
    if (fit) {
      final b = _fitFrom ??= _bounds.inflate(14);
      final fz = math.min(2.0, math.min(s.width / b.width, s.height / b.height));
      z = fz < .3 ? .5 : fz; // a short strip shows a working-scale window on the canvas, not a speck
      p = fz < .3 ? Offset(s.width / 2, 6) - Offset(b.center.dx, b.top) * z + npan : Offset(s.width / 2, s.height / 2) - b.center * z + npan;
    }
    return Focus(
      focusNode: _canvasFocus,
      onKeyEvent: (_, e) {
        final cmd = HardwareKeyboard.instance.isMetaPressed || HardwareKeyboard.instance.isControlPressed;
        // Cmd+V on the desk pastes into the note, never into the document
        if (e is KeyDownEvent && editing == null && cmd && e.logicalKey == LogicalKeyboardKey.keyV && widget.host != null) {
          widget.host!.paste(page, (const Offset(140, 160) - pan) / zoom);
          return KeyEventResult.handled;
        }
        if (e is KeyDownEvent && editing == null && selected != null && (e.logicalKey == LogicalKeyboardKey.delete || e.logicalKey == LogicalKeyboardKey.backspace)) {
          final gone = blocks[selected!];
          if (widget.host == null) {
            setState(() { _blocks.removeAt(selected!); selected = null; });
          } else {
            setState(() => selected = null);
            widget.host!.remove(page, gone);
          }
          return KeyEventResult.handled;
        }
        return KeyEventResult.ignored;
      },
      child: Listener(
      onPointerSignal: (e) {
        if (e is PointerScrollEvent && !fit) setState(() => zoom = clampD(zoom * (e.scrollDelta.dy < 0 ? 1.1 : 1 / 1.1), .25, 3));
      },
      child: GestureDetector(
        key: ValueKey(fit ? 'notes-canvas-fit' : 'notes-canvas'),
        dragStartBehavior: DragStartBehavior.down,
        onTap: () { _endEdit(); _canvasFocus.requestFocus(); setState(() { selected = null; editing = null; }); },
        onPanUpdate: (d) => setState(() { if (fit) { npan += d.delta; } else { pan += d.delta; } }),
        child: ClipRect(
          child: CustomPaint(
            size: s,
            painter: _DotsP(p, z),
            child: OverflowBox(
              alignment: Alignment.topLeft,
              minWidth: 3000, maxWidth: 3000, minHeight: 3000, maxHeight: 3000,
              child: Transform(
                transform: Matrix4.translationValues(p.dx, p.dy, 0)..scale(z, z, 1),
                child: Stack(clipBehavior: Clip.none, children: [for (final (i, b) in blocks.indexed) _placed(i, b, z)]),
              ),
            ),
          ),
        ),
      ),
    ),
    );
  }

  Widget _placed(int i, NBlock b, double z) => Positioned(
        left: b.pos.dx,
        top: b.pos.dy,
        width: b.size.width,
        height: b.size.height,
        child: GestureDetector(
          key: ValueKey('blk-$i'),
          dragStartBehavior: DragStartBehavior.down,
          onTap: () { if (editing != i) { _endEdit(); _canvasFocus.requestFocus(); } setState(() { selected = i; if (editing != i) editing = null; }); },
          onDoubleTap: b.kind == 'note' || b.kind == 'hand'
              ? () => setState(() { selected = i; editing = i; _ctl.text = b.text; _focus.requestFocus(); })
              : (b.kind == 'ref' && widget.host != null ? () => widget.host!.follow(page, b) : null),
          onPanStart: (d) => setState(() => selected = i),
          // delta arrives in the block's own (canvas) units: the transform's scale is already undone
          onPanUpdate: (d) {
            if (editing == i) return;
            setState(() => b.pos += d.delta);
          },
          onPanEnd: (_) {
            if (editing != i) widget.host?.patch(page, b, {'x': b.pos.dx, 'y': b.pos.dy});
          },
          child: Stack(clipBehavior: Clip.none, children: [
            Positioned.fill(child: _body(b, i)),
            if (selected == i) ...[
              Positioned.fill(child: IgnorePointer(child: DecoratedBox(decoration: BoxDecoration(border: Border.all(color: kAccent, width: 1.6 / z), borderRadius: BorderRadius.circular(2))))),
              Positioned(
                right: -6 / z,
                bottom: -6 / z,
                width: 14 / z,
                height: 14 / z,
                child: GestureDetector(
                  key: const ValueKey('resize'),
                  dragStartBehavior: DragStartBehavior.down,
                  onPanUpdate: (d) => setState(() => b.size = Size(math.max(56, b.size.width + d.delta.dx), math.max(28, b.size.height + d.delta.dy))),
                  onPanEnd: (_) => widget.host?.patch(page, b, {'width': b.size.width, 'height': b.size.height}),
                  child: DecoratedBox(decoration: BoxDecoration(color: kInk, border: Border.all(color: kAccent, width: 1.4 / z), borderRadius: BorderRadius.circular(2 / z))),
                ),
              ),
            ],
          ]),
        ),
      );

  Widget _body(NBlock b, int i) {
    switch (b.kind) {
      case 'note':
        final col = const [kYellow, kPink, kBlue, kMint][b.tint % 4];
        final ink = N.g10;
        return Container(
          padding: const EdgeInsets.all(10),
          decoration: BoxDecoration(color: col, borderRadius: BorderRadius.circular(3)),
          child: editing == i
              ? EditableText(key: const ValueKey('note-edit'), controller: _ctl, focusNode: _focus, autofocus: true, maxLines: null, style: sans(12, c: ink, w: FontWeight.w600), cursorColor: ink, backgroundCursorColor: ink, onChanged: (t) => b.text = t)
              : Text(b.text, style: sans(12, c: ink, w: FontWeight.w600)),
        );
      case 'image':
        final png = b.png;
        return ClipRRect(borderRadius: BorderRadius.circular(3), child: png == null ? CustomPaint(painter: _PicP()) : Image.memory(png, fit: BoxFit.cover, gaplessPlayback: true));
      case 'ref':
        return Container(
          padding: const EdgeInsets.symmetric(horizontal: 10),
          decoration: BoxDecoration(color: kViolet, borderRadius: BorderRadius.circular(14)),
          child: Row(children: [
            Container(width: 8, height: 8, decoration: const BoxDecoration(color: N.g10, shape: BoxShape.circle)),
            const SizedBox(width: 8),
            Expanded(child: Text(b.text, softWrap: false, overflow: TextOverflow.clip, style: sans(12, c: N.g10, w: FontWeight.w600))),
          ]),
        );
      default:
        return Align(alignment: Alignment.centerLeft, child: Text(b.text, softWrap: false, style: const TextStyle(fontFamily: 'Snell Roundhand', fontSize: 26, color: N.g95)));
    }
  }
}

class _PicP extends CustomPainter {
  @override
  void paint(Canvas c, Size s) {
    c.drawRect(Offset.zero & s, Paint()..color = kBlue);
    c.drawCircle(Offset(s.width * .72, s.height * .28), s.height * .14, Paint()..color = kYellow);
    c.drawPath(Path()..moveTo(0, s.height)..lineTo(s.width * .3, s.height * .42)..lineTo(s.width * .58, s.height)..close(), Paint()..color = kViolet);
    c.drawPath(Path()..moveTo(s.width * .35, s.height)..lineTo(s.width * .68, s.height * .55)..lineTo(s.width, s.height)..close(), Paint()..color = kMint);
  }
  @override
  bool shouldRepaint(_PicP o) => false;
}

class _DotsP extends CustomPainter {
  _DotsP(this.pan, this.z);
  final Offset pan;
  final double z;
  @override
  void paint(Canvas c, Size s) {
    c.drawRect(Offset.zero & s, Paint()..color = kGround);
    final p = Paint()..color = N.g15;
    final step = 16 * z;
    if (step < 5) return;
    final ox = pan.dx % step, oy = pan.dy % step;
    for (var x = ox; x < s.width; x += step) {
      for (var y = oy; y < s.height; y += step) { c.drawCircle(Offset(x, y), .9, p); }
    }
  }
  @override
  bool shouldRepaint(_DotsP o) => o.pan != pan || o.z != z;
}

class _ToolP extends CustomPainter {
  _ToolP(this.i);
  final int i;
  @override
  void paint(Canvas c, Size s) {
    final p = Paint()..color = N.g82..style = PaintingStyle.stroke..strokeWidth = 1.4..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round;
    final m = s.center(Offset.zero);
    switch (i) {
      case 0:
        c.drawPath(Path()..moveTo(m.dx - 4, m.dy - 6)..lineTo(m.dx - 4, m.dy + 6)..lineTo(m.dx - 1, m.dy + 3)..lineTo(m.dx + 3, m.dy + 7)..lineTo(m.dx + 5, m.dy + 5)..lineTo(m.dx + 1, m.dy + 1)..lineTo(m.dx + 5, m.dy - 1)..close(), p);
      case 1:
        c.drawRect(Rect.fromCenter(center: m, width: 12, height: 12), p);
        c.drawLine(m + const Offset(-3, -1), m + const Offset(3, -1), p);
        c.drawLine(m + const Offset(-3, 2), m + const Offset(1, 2), p);
      case 2:
        c.drawRect(Rect.fromCenter(center: m, width: 14, height: 11), p);
        c.drawPath(Path()..moveTo(m.dx - 6, m.dy + 4)..lineTo(m.dx - 2, m.dy)..lineTo(m.dx + 1, m.dy + 3)..lineTo(m.dx + 3, m.dy + 1)..lineTo(m.dx + 6, m.dy + 4), p);
      case 3:
        c.drawCircle(m + const Offset(-3, 0), 3.5, p);
        c.drawCircle(m + const Offset(3, 0), 3.5, p);
      default:
        c.drawPath(Path()..moveTo(m.dx - 6, m.dy + 5)..cubicTo(m.dx - 3, m.dy - 8, m.dx + 1, m.dy + 8, m.dx + 6, m.dy - 4), p);
    }
  }
  @override
  bool shouldRepaint(_ToolP o) => false;
}
