import 'dart:convert';
import 'dart:math' as math;

import 'package:flutter/gestures.dart' show HitTestResult;
import 'package:flutter/widgets.dart';

import 'face.dart';
import '../../session/editor_session.dart';
import '../hosts.dart';

/// The Notes desk over the document's notebook (`notebook` in the status, `notes` to change it). Text, image and
/// reference blocks are the host's; a kind it does not hold (a hand line) is not added. An image comes from the
/// picture on the pasteboard; a reference names the selected layer and its span.
class LiveNotesHost extends ChangeNotifier implements NotesHost {
  LiveNotesHost(this.c) {
    c.slice('liveNotes', const ['notebook']).addListener(notifyListeners);
  }
  final EditorSession c;

  List<Map<String, dynamic>> get _pages => EditorSession.maps(EditorSession.map(c.state['notebook'])['pages']);

  @override
  int get pageCount => math.max(1, _pages.length);

  @override
  List<NBlock> blocks(int page) {
    final pages = _pages;
    if (page >= pages.length) return [];
    return [
      for (final (i, b) in EditorSession.maps(pages[page]['blocks']).indexed)
        if (const {'text': 'note', 'image': 'image', 'reference': 'ref'}[b['kind']] case final kind?)
          NBlock(
            kind,
            Offset((b['x'] as num).toDouble(), (b['y'] as num).toDouble()),
            Size((b['width'] as num).toDouble(), (b['height'] as num).toDouble()),
            '${b['text'] ?? b['label'] ?? ''}',
            i % 2,
            '${b['id']}',
            b['png'] is String ? base64Decode(b['png'] as String) : null,
          ),
    ];
  }

  @override
  bool can(String kind) => const {'note', 'image', 'ref'}.contains(kind);

  /// The page shown, by id; null before the notebook has one (the host then makes the first page in the same edit).
  Future<String?> _page(int page) async {
    final pages = _pages;
    return page < pages.length ? '${pages[page]['id']}' : null;
  }

  @override
  Future<int?> addPage() async {
    // the host names the page; it goes last
    await c.command('notes', {'action': 'addPage', 'title': 'Untitled page'});
    if (c.error.value != null || _pages.isEmpty) return null;
    return _pages.length - 1;
  }

  @override
  bool hasPage(int page) => page < _pages.length;

  @override
  void follow(int page, NBlock block) {
    if (page >= _pages.length) return;
    final b = EditorSession.maps(_pages[page]['blocks']).where((x) => '${x['id']}' == block.id).firstOrNull;
    if (b == null) return;
    if (b['layer'] != null) c.command('select', {'ids': [b['layer']]});
    if (b['start'] is num) c.seek((b['start'] as num).toInt());
  }

  int _shown = 0;
  @override
  void showing(int page) => _shown = page;

  /// Pictures dropped on the desk become image cards on the page shown (Classic DK-062), fanned out, in one step.
  Future<void> dropImages(List<String> paths) async {
    final pageId = await _page(_shown);
    final stamp = DateTime.now().microsecondsSinceEpoch;
    await c.command('notes', {
      'page': pageId,
      'action': 'images',
      'images': [
        for (var i = 0; i < paths.length; i++)
          {'path': paths[i], 'block': {'id': 'b$stamp$i', ..._frame(NBlock('image', Offset(40.0 + i * 24, 40.0 + i * 24), const Size(110, 80), '', 0))}},
      ],
    });
  }

  @override
  Future<void> paste(int page, Offset at) async {
    final clip = EditorSession.map(await c.native('noteClipboard'));
    final id = 'b${DateTime.now().microsecondsSinceEpoch}';
    if (clip['png'] is String) {
      final pageId = await _page(page);
      await c.command('notes', {'page': pageId, 'action': 'image', 'png': clip['png'], 'block': {'id': id, ..._frame(NBlock('image', at, const Size(110, 80), '', 0))}});
    } else if ('${clip['text'] ?? ''}'.isNotEmpty) {
      final pageId = await _page(page);
      await c.command('notes', {'page': pageId, 'action': 'putBlock', 'block': {'id': id, ..._frame(NBlock('note', at, const Size(110, 74), '', 0)), 'kind': 'text', 'text': '${clip['text']}'}});
    }
  }

  @override
  Future<void> deletePage(int page) async {
    final pages = _pages;
    if (page >= pages.length) return;
    await c.flushEditors();
    await c.command('notes', {'page': '${pages[page]['id']}', 'action': 'deletePage'});
  }

  Map<String, dynamic> _frame(NBlock b) => {
        'x': math.max(0.0, b.pos.dx),
        'y': math.max(0.0, b.pos.dy),
        'width': math.max(40.0, b.size.width),
        'height': math.max(32.0, b.size.height),
      };

  @override
  Future<void> put(int page, NBlock b) async {
    final id = 'b${DateTime.now().microsecondsSinceEpoch}';
    final pageId = await _page(page);
    switch (b.kind) {
      case 'note':
        await c.command('notes', {'page': pageId, 'action': 'putBlock', 'block': {'id': id, ..._frame(b), 'kind': 'text', 'text': b.text}});
      case 'image':
        final clip = EditorSession.map(await c.native('noteClipboard'));
        if (clip['png'] is! String) return;
        await c.command('notes', {'page': pageId, 'action': 'image', 'png': clip['png'], 'block': {'id': id, ..._frame(b)}});
      case 'ref':
        // Classic's link: the active layer, from the picked keys' first to last frame, else the playhead
        final layer = c.activeLayer;
        final frames = [for (final k in EditorSession.maps(c.state['selectedKeys'])) (k['frame'] as num).toInt()]..sort();
        final start = frames.firstOrNull ?? c.frame.value, end = frames.lastOrNull ?? c.frame.value;
        await c.command('notes', {
          'page': pageId,
          'action': 'putBlock',
          'block': {'id': id, ..._frame(b), 'height': 64.0, 'kind': 'reference', 'label': '${layer?['name'] ?? 'Timeline'} · $start–$end', 'layer': layer?['id'], 'start': start, 'end': end},
        });
    }
  }

  @override
  Future<void> patch(int page, NBlock b, Map<String, dynamic> changed) async {
    if (b.id == null || page >= _pages.length) return;
    final f = _frame(b);
    await c.command('notes', {
      'page': '${_pages[page]['id']}',
      'action': 'patchBlock',
      'id': b.id,
      'patch': {for (final e in changed.entries) e.key: f[e.key] ?? e.value},
    });
  }

  @override
  Future<void> remove(int page, NBlock b) async {
    if (b.id == null || page >= _pages.length) return;
    await c.command('notes', {'page': '${_pages[page]['id']}', 'action': 'deleteBlock', 'id': b.id});
  }

  @override
  void dispose() {
    c.slice('liveNotes', const ['notebook']).removeListener(notifyListeners);
    super.dispose();
  }
}

/// The Notes desk on the session.
class LiveNotes extends StatefulWidget {
  const LiveNotes({super.key, required this.c});
  final EditorSession c;
  @override
  State<LiveNotes> createState() => _LiveNotesState();
}

class _LiveNotesState extends State<LiveNotes> {
  LiveNotesHost get host => DeskSession.of(widget.c).notes;
  final _desk = GlobalKey();

  @override
  void initState() {
    super.initState();
    widget.c.fileDropTarget = _drop;
  }

  /// Files dropped over the desk are pictures for the page, not imports (Classic DK-062); elsewhere they import.
  bool _drop(Map<String, dynamic> event) {
    final box = _desk.currentContext?.findRenderObject();
    final point = event['point'];
    if (box is! RenderBox || !box.attached || point is! List) return false;
    final global = Offset((point[0] as num).toDouble(), (point[1] as num).toDouble());
    // only where the desk is actually hit (a hidden tab stays mounted but is not under the pointer), as Classic does
    final hits = HitTestResult();
    WidgetsBinding.instance.hitTestInView(hits, global, View.of(context).viewId);
    // anywhere inside the desk counts, an empty part of it too
    bool inside(Object target) {
      for (RenderObject? o = target is RenderObject ? target : null; o != null; o = o.parent) {
        if (identical(o, box)) return true;
      }
      return false;
    }
    if (!hits.path.any((h) => inside(h.target))) return false;
    host.dropImages([for (final p in event['paths'] as List? ?? const []) if (p is String) p]);
    return true;
  }

  @override
  void dispose() {
    if (widget.c.fileDropTarget == _drop) widget.c.fileDropTarget = null;
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => KeyedSubtree(key: _desk, child: NotesDesk(host: host));
}
