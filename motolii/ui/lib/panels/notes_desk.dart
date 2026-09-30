import 'dart:async';
import 'dart:convert';
import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart';

import '../session/editor_session.dart';
import '../foundation/notes_view.dart';
import '../theme/editor_theme.dart';
import '../controls/panel.dart';
import '../theme/editor_metrics.dart';
import '../theme/material_icons.dart';
import '../controls/leaves.dart';

part 'notes_desk/card.dart';

class NotesPanel extends StatefulWidget {
  const NotesPanel({super.key, required this.controller, this.skin, this.look});
  final EditorSession controller;

  /// Another skin on the same desk: it is given the pages, the canvas and the actions, and places them.
  final Widget Function(BuildContext context, NotesView view)? skin;
  final NoteLook? look;
  @override
  State<NotesPanel> createState() => _NotesPanelState();
}

class _NotesPanelState extends State<NotesPanel> {
  EditorSession get c => widget.controller;
  final _transform = TransformationController();
  final _viewport = GlobalKey();
  final _focus = FocusNode();
  String? _pageId, _autoFocus;

  /// The chosen card, as a signal each card watches for itself.
  final _selection = ValueNotifier<String?>(null);
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
    _selection.dispose();
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
        _selection.value = null;
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
    if (mounted && c.error.value == null) {
      _selection.value = id;
      setState(() => _autoFocus = id);
    }
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
    if (mounted && c.error.value == null) _selection.value = id;
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

  Future<void> _pickImages() async {
    final paths = await c.native('pickImport');
    if (paths is List)
      for (var i = 0; i < paths.length; i++) {
        await _image(_insertion + Offset(i * 24, i * 24), path: '${paths[i]}');
      }
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
    animation: c.slice('notes', const [
      'notebook',
      'assets',
      'selectedKeys',
      'capabilities',
    ]),
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
      final canvas = Stack(
        children: [
          if (widget.look != null) Positioned.fill(child: IgnorePointer(child: CustomPaint(painter: _NoteDots(_transform, widget.look!)))),
          Listener(
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
                            child: ColoredBox(
                              color: widget.look != null ? const Color(0x00000000) : EditorTheme.of(context).panel,
                            ),
                          ),
                        ),
                        for (final (i, b) in blocks.indexed)
                          Positioned(
                            left: (b['x'] as num).toDouble(),
                            top: (b['y'] as num).toDouble(),
                            width: (b['width'] as num).toDouble(),
                            height: (b['height'] as num).toDouble(),
                            child: Picked<String?>(
                              key: ValueKey('${page!['id']}:${b['id']}'),
                              of: _selection,
                              test: (chosen) => chosen == b['id'],
                              builder: (selected) => _NoteCard(
                                controller: c,
                                page: '${page['id']}',
                                block: b,
                                selected: selected,
                                autoFocus: _autoFocus == b['id'],
                                index: i,
                                look: widget.look,
                                onSelect: () => _selection.value = b['id'],
                              ),
                            ),
                          ),
                      ],
                    ),
                  ),
                ),
              ),
        ],
      );
      KeyEventResult keys(FocusNode _, KeyEvent event) {
        {
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
              _selection.value != null) {
            _action('deleteBlock', {'id': _selection.value});
            _selection.value = null;
            return KeyEventResult.handled;
          }
          return KeyEventResult.ignored;
        }
      }

      if (widget.skin != null) {
        return Focus(
          focusNode: _focus,
          onKeyEvent: keys,
          child: widget.skin!(
            context,
            NotesView(
              pages: [for (final p in _pages) NotesPage('${p['id']}', '${p['title']}', active: p['id'] == page?['id'])],
              pageTitle: page == null ? null : '${page['title']}',
              blocks: blocks.length,
              canvas: canvas,
              legacy: _pages.isEmpty &&
                  (c.deskWork.value['note'] != null ||
                      c.deskWork.value['notes'] != null ||
                      EditorSession.maps(c.state['assets']).any((a) => a['role'] == 'reference')),
              transform: _transform,
              selectPage: (id) async {
                await c.flushEditors();
                if (mounted)
                  setState(() {
                    _pageId = id;
                    _autoFocus = null;
                    _selection.value = null;
                    _transform.value = Matrix4.identity();
                  });
              },
              newPage: _newPage,
              renamePage: (title) => _action('renamePage', {'title': title}, page: page == null ? null : '${page['id']}'),
              deletePage: () async {
                if (page == null) return;
                await c.flushEditors();
                await _action('deletePage', {}, page: '${page['id']}');
              },
              paste: _paste,
              insertImage: _pickImages,
              linkSelection: _reference,
              resetView: () => setState(() => _transform.value = Matrix4.identity()),
              importLegacy: _legacy,
              setZoom: (z) {
                final t = _transform.value.clone();
                final k = z / t.getMaxScaleOnAxis();
                _transform.value = Matrix4.identity()
                  ..translate(t.getTranslation().x, t.getTranslation().y)
                  ..scale(z.clamp(.25, 2.0));
                // keeping the view where it is: only the scale changes
                if (k.isNaN) _transform.value = Matrix4.identity();
              },
            ),
          ),
        );
      }
      return Focus(
        focusNode: _focus,
        onKeyEvent: keys,
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
                          EditorTextButton(
                            onPressed: () async {
                              await c.flushEditors();
                              if (mounted)
                                setState(() {
                                  _pageId = p['id'];
                                  _selection.value = null;
                                  _transform.value = Matrix4.identity();
                                });
                            },
                            foreground: p['id'] == page?['id']
                                ? EditorTheme.of(context).accent
                                : EditorTheme.of(context).muted,
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
                  EditorTooltip(
                    message: 'New page',
                    child: EditorIconButton(
                      iconSize: EditorMetrics.s16,
                      onPressed: _newPage,
                      icon: const Icon(Glyph.add),
                    ),
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
                spacing: EditorMetrics.s14,
                children: [
                  EditorTooltip(
                    message: 'Paste',
                    child: EditorIconButton(
                      iconSize: EditorMetrics.s16,
                      onPressed: _paste,
                      icon: const Icon(Glyph.content_paste),
                    ),
                  ),
                  EditorTooltip(
                    message: 'Insert image',
                    child: EditorIconButton(
                      iconSize: EditorMetrics.s16,
                      onPressed: _pickImages,
                      icon: const Icon(Glyph.image_outlined),
                    ),
                  ),
                  EditorTooltip(
                    message: 'Link selection',
                    child: EditorIconButton(
                      iconSize: EditorMetrics.s16,
                      onPressed: _reference,
                      icon: const Icon(Glyph.link),
                    ),
                  ),
                  EditorTooltip(
                    message: 'Reset view',
                    child: EditorIconButton(
                      iconSize: EditorMetrics.s16,
                      onPressed: () =>
                          setState(() => _transform.value = Matrix4.identity()),
                      icon: const Icon(Glyph.center_focus_strong),
                    ),
                  ),
                  if (page != null)
                    EditorTooltip(
                      message: 'Delete page',
                      child: EditorIconButton(
                        iconSize: EditorMetrics.s16,
                        onPressed: () async {
                          await c.flushEditors();
                          await _action(
                            'deletePage',
                            {},
                            page: '${page['id']}',
                          );
                        },
                        icon: const Icon(Glyph.delete_outline),
                      ),
                    ),
                ],
              ),
            ),
            if (_pages.isEmpty)
              Padding(
                padding: const EdgeInsets.all(EditorMetrics.s8),
                child: Column(
                  children: [
                    Text(
                      'Click anywhere to write',
                      style: TextStyle(
                        fontSize: EditorMetrics.font,
                        color: EditorTheme.of(context).muted,
                      ),
                    ),
                    if (c.deskWork.value['note'] != null ||
                        c.deskWork.value['notes'] != null ||
                        EditorSession.maps(c.state['assets'])
                            .any((a) => a['role'] == 'reference'))
                      EditorTextButton(
                        onPressed: _legacy,
                        child: const Text('Import previous text / references'),
                      ),
                  ],
                ),
              ),
            Expanded(child: canvas),
          ],
        ),
      );
    },
  );
}
