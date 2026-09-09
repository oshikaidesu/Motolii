import 'dart:async';
import 'dart:convert';
import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart';

import '../session/editor_session.dart';
import '../foundation/theme.dart';
import '../foundation/panel_controls.dart';
import '../foundation/metrics.dart';

class NotesPanel extends StatefulWidget {
  const NotesPanel({super.key, required this.controller});
  final EditorSession controller;
  @override
  State<NotesPanel> createState() => _NotesPanelState();
}

class _NotesPanelState extends State<NotesPanel> {
  EditorSession get c => widget.controller;
  final _transform = TransformationController();
  final _viewport = GlobalKey();
  final _focus = FocusNode();
  String? _pageId, _selected, _autoFocus;
  Offset _insertion = const Offset(24, 24);
  int _sequence = 0;
  List<Map<String, dynamic>> get _pages =>
      EditorSession.maps(EditorSession.map(c.state['notebook'])['pages']);
  Map<String, dynamic>? get _page =>
      _pages.where((p) => p['id'] == _pageId).firstOrNull ?? _pages.firstOrNull;
  String _id() => '${DateTime.now().microsecondsSinceEpoch}-${_sequence++}';
  @override
  void initState() {
    super.initState();
    c.fileDropTarget = _drop;
  }

  @override
  void dispose() {
    if (c.fileDropTarget == _drop) c.fileDropTarget = null;
    _transform.dispose();
    _focus.dispose();
    super.dispose();
  }

  Future<void> _action(
    String action,
    Map<String, dynamic> data, {
    String? page,
  }) => c.command('notes', {
    'action': action,
    'page': page ?? _page?['id'],
    ...data,
  });
  Future<String?> _ensurePage() async {
    if (_page != null) return '${_page!['id']}';
    final id = _id();
    await _action('addPage', {'title': 'Untitled page'}, page: id);
    if (c.error.value != null) return null;
    if (mounted) setState(() => _pageId = id);
    return id;
  }

  Future<void> _newPage() async {
    await c.flushEditors();
    final id = _id();
    await _action('addPage', {'title': 'Untitled page'}, page: id);
    if (mounted && c.error.value == null)
      setState(() {
        _pageId = id;
        _selected = null;
        _transform.value = Matrix4.identity();
      });
  }

  Map<String, dynamic> _block(String id, Offset p) => {
    'id': id,
    'x': p.dx.clamp(0, 100000),
    'y': p.dy.clamp(0, 100000),
    'width': 220.0,
    'height': 120.0,
  };
  Future<void> _text(Offset p, [String text = '']) async {
    final page = await _ensurePage();
    if (page == null) return;
    final id = _id();
    await _action('putBlock', {
      'block': {..._block(id, p), 'kind': 'text', 'text': text},
    }, page: page);
    if (mounted && c.error.value == null)
      setState(() {
        _selected = id;
        _autoFocus = id;
      });
  }

  Future<void> _image(Offset point, {String? path, String? png}) async {
    final page = await _ensurePage();
    if (page == null) return;
    final id = _id();
    await _action('image', {
      'block': _block(id, point),
      if (path != null) 'path': path,
      if (png != null) 'png': png,
    }, page: page);
    if (mounted && c.error.value == null) setState(() => _selected = id);
  }

  Future<void> _paste() async {
    final data = EditorSession.map(await c.native('noteClipboard'));
    if (data['png'] is String) {
      await _image(_insertion, png: data['png']);
    } else if ('${data['text'] ?? ''}'.isNotEmpty) {
      await _text(_insertion, '${data['text']}');
    }
  }

  bool _drop(Map<String, dynamic> event) {
    final box = _viewport.currentContext?.findRenderObject();
    final point = event['point'];
    if (box is! RenderBox || point is! List) return false;
    final global = Offset(
      (point[0] as num).toDouble(),
      (point[1] as num).toDouble(),
    );
    final hits = HitTestResult();
    GestureBinding.instance.hitTestInView(
      hits,
      global,
      View.of(context).viewId,
    );
    if (!hits.path.any((hit) => identical(hit.target, box))) return false;
    final p = _transform.toScene(box.globalToLocal(global));
    final paths = (event['paths'] as List? ?? []).whereType<String>().toList();
    () async {
      for (var i = 0; i < paths.length; i++) {
        await _image(p + Offset(i * 24, i * 24), path: paths[i]);
      }
    }();
    return true;
  }

  Future<void> _reference() async {
    final layer = c.activeLayer;
    final keys = EditorSession.maps(c.state['selectedKeys']);
    final frames = keys.map((k) => (k['frame'] as num).toInt()).toList()
      ..sort();
    final start = frames.firstOrNull ?? c.frame.value,
        end = frames.lastOrNull ?? c.frame.value;
    final page = await _ensurePage();
    if (page == null) return;
    await _action('putBlock', {
      'block': {
        ..._block(_id(), _insertion),
        'height': 64.0,
        'kind': 'reference',
        'label': '${layer?['name'] ?? 'Timeline'} · $start–$end',
        'layer': layer?['id'],
        'start': start,
        'end': end,
      },
    }, page: page);
  }

  Future<void> _legacy() async {
    final notes = <String>[
      if ('${c.deskWork.value['note'] ?? ''}'.isNotEmpty)
        '${c.deskWork.value['note']}',
      ...(c.deskWork.value['notes'] as List? ?? []).whereType<String>(),
    ];
    for (var i = 0; i < notes.length; i++) {
      await _text(Offset(24, 24 + i * 150), notes[i]);
    }
    final references = EditorSession.maps(c.state['assets'])
        .where((a) => a['role'] == 'reference' && a['path'] is String);
    var i = 0;
    for (final asset in references) {
      await _image(Offset(280, 24 + i++ * 180), path: asset['path']);
    }
  }

  @override
  Widget build(BuildContext context) => AnimatedBuilder(
    animation: c.document,
    builder: (context, _) {
      final page = _page;
      final blocks = EditorSession.maps(page?['blocks']);
      final width = blocks.fold<double>(
        1600,
        (v, b) => math.max(
          v,
          (b['x'] as num).toDouble() + (b['width'] as num).toDouble() + 300,
        ),
      );
      final height = blocks.fold<double>(
        1200,
        (v, b) => math.max(
          v,
          (b['y'] as num).toDouble() + (b['height'] as num).toDouble() + 300,
        ),
      );
      return Focus(
        focusNode: _focus,
        onKeyEvent: (_, event) {
          if (event is! KeyDownEvent) return KeyEventResult.ignored;
          final typing =
              FocusManager.instance.primaryFocus?.context
                  ?.findAncestorWidgetOfExactType<EditableText>() !=
              null;
          if (typing) return KeyEventResult.ignored;
          final cmd =
              HardwareKeyboard.instance.isMetaPressed ||
              HardwareKeyboard.instance.isControlPressed;
          if (cmd && event.logicalKey == LogicalKeyboardKey.keyV) {
            _paste();
            return KeyEventResult.handled;
          }
          if ((event.logicalKey == LogicalKeyboardKey.delete ||
                  event.logicalKey == LogicalKeyboardKey.backspace) &&
              _selected != null) {
            _action('deleteBlock', {'id': _selected});
            setState(() => _selected = null);
            return KeyEventResult.handled;
          }
          return KeyEventResult.ignored;
        },
        child: Column(
          children: [
            SizedBox(
              height: EditorMetrics.tall,
              child: Row(
                children: [
                  Expanded(
                    child: ListView(
                      scrollDirection: Axis.horizontal,
                      children: [
                        for (final p in _pages)
                          TextButton(
                            onPressed: () async {
                              await c.flushEditors();
                              if (mounted)
                                setState(() {
                                  _pageId = p['id'];
                                  _selected = null;
                                  _transform.value = Matrix4.identity();
                                });
                            },
                            style: TextButton.styleFrom(
                              foregroundColor: p['id'] == page?['id']
                                  ? EditorTheme.accent
                                  : EditorTheme.muted,
                            ),
                            child: Text(
                              '${p['title']}',
                              style: const TextStyle(
                                fontSize: EditorMetrics.font,
                              ),
                            ),
                          ),
                      ],
                    ),
                  ),
                  EditorIconButton(
                    tooltip: 'New page',
                    constraints: EditorTheme.iconConstraints,
                    padding: EdgeInsets.zero,
                    iconSize: EditorMetrics.s16,
                    onPressed: _newPage,
                    icon: const Icon(Icons.add),
                  ),
                ],
              ),
            ),
            if (page != null)
              Padding(
                padding: const EdgeInsets.symmetric(
                  horizontal: EditorMetrics.s8,
                ),
                child: EditorDraftField(
                  key: ValueKey('page:${page['id']}'),
                  value: '${page['title']}',
                  label: 'Page title',
                  onCommit: (value) => _action('renamePage', {
                    'title': value,
                  }, page: '${page['id']}'),
                ),
              ),
            SizedBox(
              height: EditorMetrics.s32,
              child: Row(
                children: [
                  EditorIconButton(
                    padding: EdgeInsets.zero,
                    constraints: const BoxConstraints.tightFor(
                      width: EditorMetrics.tall,
                      height: EditorMetrics.tall,
                    ),
                    tooltip: 'Paste',
                    iconSize: EditorMetrics.s16,
                    onPressed: _paste,
                    icon: const Icon(Icons.content_paste),
                  ),
                  EditorIconButton(
                    padding: EdgeInsets.zero,
                    constraints: const BoxConstraints.tightFor(
                      width: EditorMetrics.tall,
                      height: EditorMetrics.tall,
                    ),
                    tooltip: 'Insert image',
                    iconSize: EditorMetrics.s16,
                    onPressed: () async {
                      final paths = await c.native('pickImport');
                      if (paths is List)
                        for (var i = 0; i < paths.length; i++) {
                          await _image(
                            _insertion + Offset(i * 24, i * 24),
                            path: '${paths[i]}',
                          );
                        }
                    },
                    icon: const Icon(Icons.image_outlined),
                  ),
                  EditorIconButton(
                    padding: EdgeInsets.zero,
                    constraints: const BoxConstraints.tightFor(
                      width: EditorMetrics.tall,
                      height: EditorMetrics.tall,
                    ),
                    tooltip: 'Link selection',
                    iconSize: EditorMetrics.s16,
                    onPressed: _reference,
                    icon: const Icon(Icons.link),
                  ),
                  EditorIconButton(
                    padding: EdgeInsets.zero,
                    constraints: const BoxConstraints.tightFor(
                      width: EditorMetrics.tall,
                      height: EditorMetrics.tall,
                    ),
                    tooltip: 'Reset view',
                    iconSize: EditorMetrics.s16,
                    onPressed: () =>
                        setState(() => _transform.value = Matrix4.identity()),
                    icon: const Icon(Icons.center_focus_strong),
                  ),
                  if (page != null)
                    EditorIconButton(
                      padding: EdgeInsets.zero,
                      constraints: const BoxConstraints.tightFor(
                        width: EditorMetrics.tall,
                        height: EditorMetrics.tall,
                      ),
                      tooltip: 'Delete page',
                      iconSize: EditorMetrics.s16,
                      onPressed: () async {
                        await c.flushEditors();
                        await _action('deletePage', {}, page: '${page['id']}');
                      },
                      icon: const Icon(Icons.delete_outline),
                    ),
                ],
              ),
            ),
            if (_pages.isEmpty)
              Padding(
                padding: const EdgeInsets.all(EditorMetrics.s8),
                child: Column(
                  children: [
                    const Text(
                      'Click anywhere to write',
                      style: TextStyle(
                        fontSize: EditorMetrics.font,
                        color: EditorTheme.muted,
                      ),
                    ),
                    if (c.deskWork.value['note'] != null ||
                        c.deskWork.value['notes'] != null ||
                        EditorSession.maps(c.state['assets'])
                            .any((a) => a['role'] == 'reference'))
                      TextButton(
                        onPressed: _legacy,
                        child: const Text('Import previous text / references'),
                      ),
                  ],
                ),
              ),
            Expanded(
              child: Listener(
                key: _viewport,
                behavior: HitTestBehavior.opaque,
                child: InteractiveViewer(
                  transformationController: _transform,
                  constrained: false,
                  minScale: .25,
                  maxScale: 2.0,
                  boundaryMargin: const EdgeInsets.all(EditorMetrics.s200),
                  child: SizedBox(
                    width: width,
                    height: height,
                    child: Stack(
                      children: [
                        Positioned.fill(
                          child: GestureDetector(
                            behavior: HitTestBehavior.opaque,
                            onTapUp: (e) {
                              _focus.requestFocus();
                              _insertion = e.localPosition;
                              _text(_insertion);
                            },
                            child: const ColoredBox(color: EditorTheme.panel),
                          ),
                        ),
                        for (final b in blocks)
                          Positioned(
                            left: (b['x'] as num).toDouble(),
                            top: (b['y'] as num).toDouble(),
                            width: (b['width'] as num).toDouble(),
                            height: (b['height'] as num).toDouble(),
                            child: _NoteCard(
                              key: ValueKey('${page!['id']}:${b['id']}'),
                              controller: c,
                              page: '${page['id']}',
                              block: b,
                              selected: _selected == b['id'],
                              autoFocus: _autoFocus == b['id'],
                              onSelect: () =>
                                  setState(() => _selected = b['id']),
                            ),
                          ),
                      ],
                    ),
                  ),
                ),
              ),
            ),
          ],
        ),
      );
    },
  );
}

class _NoteCard extends StatefulWidget {
  const _NoteCard({
    super.key,
    required this.controller,
    required this.page,
    required this.block,
    required this.selected,
    required this.autoFocus,
    required this.onSelect,
  });
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
    if (widget.autoFocus)
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted) _focus.requestFocus();
      });
  }

  @override
  void didUpdateWidget(covariant _NoteCard old) {
    super.didUpdateWidget(old);
    if (old.block['png'] != b['png']) _decodeImage();
    if (!_dirty && '${b['text'] ?? ''}' != _text.text)
      _text.text = '${b['text'] ?? ''}';
  }

  void _blur() {
    if (!_focus.hasFocus) _flush();
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

  @override
  Widget build(BuildContext context) {
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
          child: Material(
            color: EditorTheme.raised,
            child: Container(
              decoration: BoxDecoration(
                border: Border.all(
                  color: widget.selected
                      ? EditorTheme.accent
                      : EditorTheme.line,
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
                            child: const Center(
                              child: Icon(
                                Icons.drag_handle,
                                size: EditorMetrics.s14,
                                color: EditorTheme.muted,
                              ),
                            ),
                          ),
                        ),
                        EditorIconButton(
                          tooltip: 'Delete note',
                          padding: EdgeInsets.zero,
                          constraints: const BoxConstraints.tightFor(
                            width: EditorMetrics.row,
                            height: EditorMetrics.row,
                          ),
                          iconSize: EditorMetrics.s12,
                          onPressed: () async {
                            await _flush();
                            await widget.controller.command('notes', {
                              'action': 'deleteBlock',
                              'page': widget.page,
                              'id': b['id'],
                            });
                          },
                          icon: const Icon(Icons.close),
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
                        'reference' => TextButton(
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
                        _ => TextField(
                          controller: _text,
                          focusNode: _focus,
                          maxLines: null,
                          expands: true,
                          style: const TextStyle(
                            fontSize: EditorMetrics.title,
                            color: EditorTheme.ink,
                          ),
                          decoration: const InputDecoration(
                            border: InputBorder.none,
                            hintText: 'Write a note',
                            isDense: true,
                          ),
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
                      child: const SizedBox(
                        width: EditorMetrics.s18,
                        height: EditorMetrics.s16,
                        child: Icon(
                          Icons.south_east,
                          size: EditorMetrics.s12,
                          color: EditorTheme.muted,
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
