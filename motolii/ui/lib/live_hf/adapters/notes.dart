import 'dart:convert';
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../../hf/desk/notes.dart';
import '../../session/editor_session.dart';

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

  Future<String> _page(int page) async {
    final pages = _pages;
    if (page < pages.length) return '${pages[page]['id']}';
    await c.command('notes', {'page': 'p1', 'action': 'addPage', 'title': 'Page 1'});
    return 'p1';
  }

  @override
  Future<int?> addPage() async {
    final ids = {for (final p in _pages) '${p['id']}'};
    var n = _pages.length + 1;
    while (ids.contains('p$n')) n++;
    await c.command('notes', {'page': 'p$n', 'action': 'addPage', 'title': 'Untitled page'});
    if (c.error.value != null) return null;
    final at = _pages.indexWhere((p) => p['id'] == 'p$n');
    return at < 0 ? null : at;
  }

  @override
  bool hasPage(int page) => page < _pages.length;

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
        final layer = c.activeLayer;
        final start = (layer?['start'] as num? ?? c.frame.value).round();
        await c.command('notes', {
          'page': pageId,
          'action': 'putBlock',
          'block': {'id': id, ..._frame(b), 'kind': 'reference', 'label': '${layer?['name'] ?? 'Frame ${c.frame.value}'}', 'layer': layer?['id'], 'start': start, 'end': (start + (layer?['duration'] as num? ?? 0)).round()},
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
  late final host = LiveNotesHost(widget.c);
  @override
  void dispose() {
    host.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => NotesDesk(host: host);
}
