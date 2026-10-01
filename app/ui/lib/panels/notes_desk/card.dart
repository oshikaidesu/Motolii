part of '../notes_desk.dart';

class _NoteCard extends StatefulWidget {
  const _NoteCard({
    required this.controller,
    required this.page,
    required this.block,
    required this.selected,
    required this.autoFocus,
    required this.onSelect,
    this.index = 0,
    this.look,
  });
  final int index;
  final NoteLook? look;
  final EditorSession controller;
  final String page;
  final Map<String, dynamic> block;
  final bool selected, autoFocus;
  final VoidCallback onSelect;
  @override
  State<_NoteCard> createState() => _NoteCardState();
}

class _NoteCardState extends State<_NoteCard> {
  late final TextEditingController _text;
  final _focus = FocusNode();
  Timer? _timer;
  bool _dirty = false;
  Uint8List? _imageBytes;
  void _decodeImage() {
    try {
      _imageBytes = b['kind'] == 'image' ? base64Decode('${b['png']}') : null;
    } catch (_) {
      _imageBytes = null;
    }
  }

  Offset? _delta;
  bool _resizing = false;
  Map<String, dynamic> get b => widget.block;
  @override
  void initState() {
    super.initState();
    _text = TextEditingController(text: '${b['text'] ?? ''}');
    _decodeImage();
    _focus.addListener(_blur);
    widget.controller.pendingEditors.add(_flush);
    if (widget.autoFocus) {
      _editing = true;
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted) _focus.requestFocus();
      });
    }
  }

  @override
  void didUpdateWidget(covariant _NoteCard old) {
    super.didUpdateWidget(old);
    if (old.block['png'] != b['png']) _decodeImage();
    if (!_dirty && '${b['text'] ?? ''}' != _text.text)
      _text.text = '${b['text'] ?? ''}';
  }

  bool _editing = false;

  void _blur() {
    if (!_focus.hasFocus) {
      _flush();
      if (_editing && mounted) setState(() => _editing = false);
    }
  }

  Future<void> _patch(Map<String, dynamic> patch) =>
      widget.controller.command('notes', {
        'action': 'patchBlock',
        'page': widget.page,
        'id': b['id'],
        'patch': patch,
      });
  Future<void> _flush() async {
    _timer?.cancel();
    if (!_dirty) return;
    _dirty = false;
    await _patch({'text': _text.text});
  }

  @override
  void dispose() {
    _flush();
    widget.controller.pendingEditors.remove(_flush);
    _timer?.cancel();
    _focus.removeListener(_blur);
    _focus.dispose();
    _text.dispose();
    super.dispose();
  }

  void _start(bool resize) {
    widget.onSelect();
    _flush();
    setState(() {
      _resizing = resize;
      _delta = Offset.zero;
    });
  }

  void _move(DragUpdateDetails d) {
    setState(() => _delta = (_delta ?? Offset.zero) + d.delta);
  }

  void _end() {
    final delta = _delta;
    if (delta == null) return;
    setState(() => _delta = null);
    _patch(
      _resizing
          ? {
              'width': ((b['width'] as num) + delta.dx).clamp(80, 5000),
              'height': ((b['height'] as num) + delta.dy).clamp(60, 5000),
            }
          : {
              'x': ((b['x'] as num) + delta.dx).clamp(0, 100000),
              'y': ((b['y'] as num) + delta.dy).clamp(0, 100000),
            },
    );
  }

  /// The card in the finished Notes' look: one flat colour, moved by any part of it, written in on a double click,
  /// with a ring, a corner to resize and a cross while it is the chosen one.
  Widget _flat(BuildContext context) {
    final look = widget.look!;
    final delta = _delta ?? Offset.zero;
    final w = ((b['width'] as num) + (_resizing ? delta.dx : 0)).clamp(80, 5000).toDouble();
    final h = ((b['height'] as num) + (_resizing ? delta.dy : 0)).clamp(60, 5000).toDouble();
    final kind = '${b['kind']}';
    final tint = look.tints[widget.index % look.tints.length];
    Widget body = switch (kind) {
      'image' => ClipRRect(
          borderRadius: BorderRadius.circular(3),
          child: _imageBytes == null
              ? ColoredBox(color: look.raised, child: const Center(child: Text('Image unavailable')))
              : Image.memory(_imageBytes!, fit: BoxFit.cover, errorBuilder: (_, __, ___) => ColoredBox(color: look.raised, child: const Center(child: Text('Image unavailable')))),
        ),
      'reference' => GestureDetector(
          onTap: () async {
            widget.onSelect();
            if (b['layer'] != null) await widget.controller.command('select', {'ids': [b['layer']]});
            widget.controller.seek((b['start'] as num).toInt());
          },
          child: Container(
            padding: const EdgeInsets.symmetric(horizontal: 10),
            decoration: BoxDecoration(color: look.reference, borderRadius: BorderRadius.circular(14)),
            child: Row(children: [
              Container(width: 8, height: 8, decoration: BoxDecoration(color: look.ink, shape: BoxShape.circle)),
              const SizedBox(width: 8),
              Expanded(child: Text('${b['label']}', maxLines: 1, overflow: TextOverflow.clip, style: TextStyle(fontSize: 12, fontWeight: FontWeight.w600, color: look.ink))),
            ]),
          ),
        ),
      _ => Container(
          padding: const EdgeInsets.all(10),
          decoration: BoxDecoration(color: tint, borderRadius: BorderRadius.circular(3)),
          child: IgnorePointer(
            ignoring: !_editing,
            child: EditorTextField(
              key: ValueKey('note-text:${b['id']}'),
              controller: _text,
              focusNode: _focus,
              maxLines: null,
              expands: true,
              style: TextStyle(fontSize: 12, fontWeight: FontWeight.w600, color: look.ink),
              cursorColor: look.ink,
              hint: 'Write a note',
              onChanged: (_) {
                _dirty = true;
                _timer?.cancel();
                _timer = Timer(const Duration(milliseconds: 400), _flush);
              },
            ),
          ),
        ),
    };
    return Transform.translate(
      offset: _resizing ? Offset.zero : delta,
      child: OverflowBox(
        alignment: Alignment.topLeft,
        minWidth: 0,
        minHeight: 0,
        maxWidth: EditorMetrics.canvas,
        maxHeight: EditorMetrics.canvas,
        child: SizedBox(
          width: w,
          height: h,
          child: Stack(
            clipBehavior: Clip.none,
            children: [
              Positioned.fill(
                child: GestureDetector(
                  behavior: HitTestBehavior.opaque,
                  dragStartBehavior: DragStartBehavior.down,
                  onTap: widget.onSelect,
                  onDoubleTap: kind == 'image' || kind == 'reference'
                      ? null
                      : () {
                          widget.onSelect();
                          setState(() => _editing = true);
                          _focus.requestFocus();
                        },
                  onPanStart: (_) {
                    if (!_editing) _start(false);
                  },
                  onPanUpdate: (d) {
                    if (!_editing) _move(d);
                  },
                  onPanEnd: (_) => _end(),
                  onPanCancel: () => setState(() => _delta = null),
                  child: body,
                ),
              ),
              if (widget.selected) ...[
                Positioned.fill(child: IgnorePointer(child: DecoratedBox(decoration: BoxDecoration(border: Border.all(color: look.accent, width: 1.6), borderRadius: BorderRadius.circular(3))))),
                Positioned(
                  right: -6,
                  bottom: -6,
                  width: 14,
                  height: 14,
                  child: GestureDetector(
                    key: const ValueKey('note-resize'),
                    dragStartBehavior: DragStartBehavior.down,
                    onPanStart: (_) => _start(true),
                    onPanUpdate: _move,
                    onPanEnd: (_) => _end(),
                    onPanCancel: () => setState(() => _delta = null),
                    child: DecoratedBox(decoration: BoxDecoration(color: look.raised, border: Border.all(color: look.accent, width: 1.4), borderRadius: BorderRadius.circular(2))),
                  ),
                ),
                Positioned(
                  right: -8,
                  top: -8,
                  width: 18,
                  height: 18,
                  child: GestureDetector(
                    key: const ValueKey('note-delete'),
                    onTap: () async {
                      await _flush();
                      await widget.controller.command('notes', {'action': 'deleteBlock', 'page': widget.page, 'id': b['id']});
                    },
                    child: DecoratedBox(decoration: BoxDecoration(color: look.raised, shape: BoxShape.circle, border: Border.all(color: look.accent, width: 1.2)), child: Center(child: Icon(Glyph.close, size: 10, color: look.ink))),
                  ),
                ),
              ],
            ],
          ),
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    if (widget.look != null) return _flat(context);
    final delta = _delta ?? Offset.zero;
    return Transform.translate(
      offset: _resizing ? Offset.zero : delta,
      child: OverflowBox(
        alignment: Alignment.topLeft,
        minWidth: 0,
        minHeight: 0,
        maxWidth: EditorMetrics.canvas,
        maxHeight: EditorMetrics.canvas,
        child: SizedBox(
          width: ((b['width'] as num) + (_resizing ? delta.dx : 0))
              .clamp(80, 5000)
              .toDouble(),
          height: ((b['height'] as num) + (_resizing ? delta.dy : 0))
              .clamp(60, 5000)
              .toDouble(),
          child: ColoredBox(
            color: EditorTheme.of(context).raised,
            child: Container(
              decoration: BoxDecoration(
                border: Border.all(
                  color: widget.selected
                      ? EditorTheme.of(context).accent
                      : EditorTheme.of(context).line,
                ),
              ),
              child: Column(
                children: [
                  SizedBox(
                    height: EditorMetrics.row,
                    child: Row(
                      children: [
                        Expanded(
                          child: GestureDetector(
                            behavior: HitTestBehavior.opaque,
                            dragStartBehavior: DragStartBehavior.down,
                            onPanStart: (_) => _start(false),
                            onPanUpdate: _move,
                            onPanEnd: (_) => _end(),
                            onPanCancel: () => setState(() => _delta = null),
                            onTap: widget.onSelect,
                            child: Center(
                              child: Icon(
                                Glyph.drag_handle,
                                size: EditorMetrics.s14,
                                color: EditorTheme.of(context).muted,
                              ),
                            ),
                          ),
                        ),
                        EditorTooltip(
                          message: 'Delete note',
                          child: EditorIconButton(
                            iconSize: EditorMetrics.s12,
                            onPressed: () async {
                              await _flush();
                              await widget.controller.command('notes', {
                                'action': 'deleteBlock',
                                'page': widget.page,
                                'id': b['id'],
                              });
                            },
                            icon: const Icon(Glyph.close),
                          ),
                        ),
                      ],
                    ),
                  ),
                  Expanded(
                    child: Padding(
                      padding: const EdgeInsets.symmetric(
                        horizontal: EditorMetrics.s6,
                      ),
                      child: switch (b['kind']) {
                        'image' =>
                          _imageBytes == null
                              ? const Text('Image unavailable')
                              : Image.memory(
                                  _imageBytes!,
                                  fit: BoxFit.contain,
                                  errorBuilder: (_, __, ___) =>
                                      const Text('Image unavailable'),
                                ),
                        'reference' => EditorTextButton(
                          onPressed: () async {
                            if (b['layer'] != null)
                              await widget.controller.command('select', {
                                'ids': [b['layer']],
                              });
                            widget.controller.seek((b['start'] as num).toInt());
                          },
                          child: Text(
                            '${b['label']}',
                            style: const TextStyle(
                              fontSize: EditorMetrics.font,
                            ),
                          ),
                        ),
                        _ => EditorTextField(
                          controller: _text,
                          focusNode: _focus,
                          maxLines: null,
                          expands: true,
                          style: TextStyle(
                            fontSize: EditorMetrics.title,
                            color: EditorTheme.of(context).ink,
                          ),
                          hint: 'Write a note',
                          onTap: widget.onSelect,
                          onChanged: (_) {
                            _dirty = true;
                            _timer?.cancel();
                            _timer = Timer(
                              const Duration(milliseconds: 400),
                              _flush,
                            );
                          },
                        ),
                      },
                    ),
                  ),
                  Align(
                    alignment: Alignment.bottomRight,
                    child: GestureDetector(
                      dragStartBehavior: DragStartBehavior.down,
                      onPanStart: (_) => _start(true),
                      onPanUpdate: _move,
                      onPanEnd: (_) => _end(),
                      onPanCancel: () => setState(() => _delta = null),
                      child: SizedBox(
                        width: EditorMetrics.s18,
                        height: EditorMetrics.s16,
                        child: Icon(
                          Glyph.south_east,
                          size: EditorMetrics.s12,
                          color: EditorTheme.of(context).muted,
                        ),
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}

/// The dotted ground of the canvas, in screen space: it slides and scales with the view.
class _NoteDots extends CustomPainter {
  _NoteDots(this.transform, this.look) : super(repaint: transform);
  final ValueNotifier<Matrix4> transform;
  final NoteLook look;
  @override
  void paint(Canvas c, Size s) {
    c.drawRect(Offset.zero & s, Paint()..color = look.ground);
    final m = transform.value;
    final z = m.getMaxScaleOnAxis();
    final step = 16 * z;
    if (step < 5) return;
    final t = m.getTranslation();
    final p = Paint()..color = look.dots;
    for (var x = t.x % step; x < s.width; x += step) {
      for (var y = t.y % step; y < s.height; y += step) {
        c.drawCircle(Offset(x, y), .9, p);
      }
    }
  }

  @override
  bool shouldRepaint(_NoteDots o) => o.look != look;
}
