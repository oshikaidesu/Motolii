// The Media shelf, AEViewer 2 style: one browser for the project's files and media. A folder tree (locations, then folders only) beside
// or above the open folder's contents, shown as thumbnails or as a list. Wide: tree | contents. Narrow: tree band (height by drag) over contents.
import 'dart:io';

import 'package:flutter/widgets.dart';

import '../tokens.dart';
import 'dock_filters.dart';
import 'dock_glyphs.dart';
import 'dock_media.dart';
import 'dock_parts.dart';
import 'ws.dart';

/// The tree band's height when the shelf is narrow (double-click on its grip comes back here).
const dockTreeHeight = 132.0;

/// From this shelf width the tree stands beside the contents instead of above them.
const _wide = 440.0;

class FileNode {
  // ignore: prefer_initializing_formals
  const FileNode(this.id, this.name, this.g, {this.meta = '', List<FileNode> kids = const []}) : _kids = kids;
  final String id, name, meta;
  final DockG g;
  final List<FileNode> _kids;
  bool get folder => g == DockG.folder;

  /// A file or folder of the disk: its id is its whole path. Its folder is read when it is first opened ([listed]).
  bool get disk => id.startsWith('/');
  bool get listed => !disk || _lists.containsKey(id);
  List<FileNode> get kids => disk && folder ? _listed(id) : _kids;

  /// The catalogue entry behind a media file of the project (same id), if any.
  MediaAsset? get media => _assets[id];
}

final _assets = {for (final a in mediaAssets) a.id: a};

FileNode _m(MediaAsset a) => FileNode(a.id, a.name, a.type.glyph);
List<FileNode> _of(MediaType t) => [for (final a in mediaAssets.where((a) => a.type == t)) _m(a)];

/// The locations of the tree, each a folder: the project first, then the disk.
final fileRoots = <FileNode>[
  FileNode(
    'Project',
    'Project',
    DockG.folder,
    kids: [
      const FileNode(
        'Comps',
        'Comps',
        DockG.folder,
        kids: [
          FileNode('f-main', 'Main', DockG.comp, meta: '0:08 · 1920×1080'),
          FileNode('f-sting', 'Intro Sting', DockG.comp, meta: '0:03 · 1920×1080'),
          FileNode('f-lower', 'Lower Third', DockG.comp, meta: '0:05 · 1920×1080'),
          FileNode('f-square', 'Social 1:1', DockG.comp, meta: '0:08 · 1080×1080'),
        ],
      ),
      FileNode(
        'Footage',
        'Footage',
        DockG.folder,
        kids: [
          ..._of(MediaType.video),
          const FileNode('Plates', 'Plates', DockG.folder, kids: [FileNode('f-p1', 'plate_A_001.exr', DockG.image, meta: '4096×2160 · EXR · 24 MB')]),
        ],
      ),
      FileNode('Stills', 'Stills', DockG.folder, kids: _of(MediaType.image)),
      FileNode('HDR', 'HDR', DockG.folder, kids: _of(MediaType.hdr)),
      FileNode('Audio', 'Audio', DockG.folder, kids: _of(MediaType.audio)),
      const FileNode(
        'Fonts',
        'Fonts',
        DockG.folder,
        kids: [
          FileNode('f-avenir', 'AvenirNext.ttc', DockG.font, meta: 'Font · 12 styles · 9.1 MB'),
          FileNode('f-didot', 'Didot.ttc', DockG.font, meta: 'Font · 3 styles · 1.2 MB'),
        ],
      ),
      const FileNode(
        'Renders',
        'Renders',
        DockG.folder,
        kids: [
          FileNode('f-v12', 'coda_v12.mp4', DockG.video, meta: '1920×1080 · H.264 · 42 MB'),
          FileNode('f-v13', 'coda_v13.mp4', DockG.video, meta: '1920×1080 · H.264 · 43 MB'),
        ],
      ),
      const FileNode('f-notes', 'brief_notes.txt', DockG.doc, meta: 'Text · 4 KB'),
    ],
  ),
  const FileNode(
    'Home',
    'Home',
    DockG.folder,
    kids: [
      FileNode('h-desk', 'Desktop', DockG.folder, kids: [FileNode('h-d1', 'moodboard.png', DockG.image, meta: '2400×1600 · PNG · 2.2 MB')]),
      FileNode('h-docs', 'Documents', DockG.folder, kids: [FileNode('h-d2', 'invoice_0926.pdf', DockG.doc, meta: 'PDF · 88 KB')]),
      FileNode('h-movies', 'Movies', DockG.folder, kids: [FileNode('h-m1', 'reel_2026.mov', DockG.video, meta: '3840×2160 · ProRes · 1.1 GB')]),
      FileNode('h-music', 'Music', DockG.folder, kids: [FileNode('h-a1', 'loops_pack.wav', DockG.audio, meta: '48 kHz · stereo · 64 MB')]),
    ],
  ),
  const FileNode(
    'Desktop',
    'Desktop',
    DockG.folder,
    kids: [
      FileNode('d-1', 'moodboard.png', DockG.image, meta: '2400×1600 · PNG · 2.2 MB'),
      FileNode('d-2', 'ref_poster_04.jpg', DockG.image, meta: '1800×2400 · JPEG · 1.8 MB'),
      FileNode('d-3', 'kick_808.wav', DockG.audio, meta: '48 kHz · mono · 320 KB'),
    ],
  ),
  const FileNode(
    'Downloads',
    'Downloads',
    DockG.folder,
    kids: [
      FileNode('w-1', 'GrotesqueBold.otf', DockG.font, meta: 'Font · 1 style · 210 KB'),
      FileNode('w-2', 'stock_smoke.mov', DockG.video, meta: '1920×1080 · ProRes · 512 MB'),
    ],
  ),
];

// ---------------------------------------------------------------- the disk (a joined window)

/// What a file of the disk is here, by its name's end (null: not shown).
DockG? _glyphOf(String name) => switch (name.substring(name.lastIndexOf('.') + 1).toLowerCase()) {
  'mp4' || 'mov' || 'm4v' || 'webm' || 'mkv' => DockG.video,
  'png' || 'jpg' || 'jpeg' || 'webp' || 'svg' || 'json' || 'lot' => DockG.image,
  'glb' || 'gltf' => DockG.shape,
  'ttf' || 'otf' || 'ttc' => DockG.font,
  'wav' || 'mp3' || 'm4a' || 'aac' => DockG.audio,
  _ => null,
};

/// The inside's kind for a file placed on the stage (null: it is not placed).
String? _kindFor(DockG g) => switch (g) {
  DockG.video => 'film',
  DockG.image => 'picture',
  DockG.shape => 'solid',
  DockG.font => 'text',
  _ => null,
};

String _base(String path) => path.substring(path.lastIndexOf('/') + 1);
String _dir(String path) => path.lastIndexOf('/') <= 0 ? '/' : path.substring(0, path.lastIndexOf('/'));
String _size(int bytes) => bytes >= 1 << 20 ? '${(bytes / (1 << 20)).toStringAsFixed(1)} MB' : '${(bytes / 1024).ceil()} KB';
final _home = Platform.environment['HOME'] ?? '/';

final _lists = <String, List<FileNode>>{};
List<FileNode> _listed(String path) => _lists[path] ??= () {
  final out = <FileNode>[];
  try {
    for (final e in Directory(path).listSync()) {
      final name = _base(e.path);
      if (name.startsWith('.')) continue;
      if (e is Directory) {
        out.add(FileNode(e.path, name, DockG.folder));
      } else if (_glyphOf(name) case final g?) {
        out.add(FileNode(e.path, name, g, meta: '${name.substring(name.lastIndexOf('.') + 1).toUpperCase()} · ${_size(e.statSync().size)}'));
      }
    }
  } on FileSystemException {
    return const <FileNode>[];
  }
  return out..sort((a, b) => a.name.toLowerCase().compareTo(b.name.toLowerCase()));
}();

/// The files the document's things are made of, by whole path: the thing that uses each (the last one, the one in front).
Map<String, String> _used(Ws ws) {
  if (_usedRev == ws.rev) return _usedNow;
  final m = ws.inside!;
  _usedRev = ws.rev;
  return _usedNow = {
    for (final l in ws.layers)
      if (m.word(int.parse(l.id), 'file') case final f when f.isNotEmpty) m.whole(f): l.id,
  };
}

var _usedNow = <String, String>{};
int? _usedRev;

/// The locations of a joined window: what the document is made of (its files, and the folders they are in), then the disk.
List<FileNode> _rootsOf(DockCtx c) {
  if (c.ws.inside == null) return fileRoots;
  final files = _used(c.ws).keys.toList()..sort();
  return [
    FileNode(
      'Project',
      'Project',
      DockG.folder,
      kids: [
        for (final d in {for (final f in files) _dir(f)}) FileNode(d, _base(d), DockG.folder),
        for (final f in files)
          if (_glyphOf(f) case final g?) FileNode(f, _base(f), g),
      ],
    ),
    FileNode(_home, 'Home', DockG.folder),
  ];
}

/// Root first, [id] last.
List<FileNode> _pathOf(DockCtx c, String id) {
  if (c.ws.inside == null) return _path(id);
  final roots = _rootsOf(c);
  if (!id.startsWith('/')) return [roots.first];
  final out = <FileNode>[];
  for (var at = id; ; at = _dir(at)) {
    if (at == _home || at == '/') return out..insert(0, FileNode(at, at == _home ? 'Home' : '/', DockG.folder));
    out.insert(0, FileNode(at, _base(at), DockG.folder));
  }
}

/// The thing of the document a file is on (the sample: the layer its catalogue entry names).
String? _layerOf(FileNode n, DockCtx c) => c.ws.inside == null ? n.media?.layer : _used(c.ws)[n.id];

/// A file put on the stage as a new thing, picked. Not done: the foot of the shelf says why.
void _place(DockCtx c, FileNode n) {
  final m = c.ws.inside!, made = m.add(_kindFor(n.g)!, n.id);
  c.set(() => made == 0 ? c.mem.trouble[n.id] = m.trouble : c.mem.trouble.remove(n.id));
  if (made == 0) return;
  c.ws.reread();
  c.ws.select('$made');
}

// ---------------------------------------------------------------- the sample's tree

final _byId = <String, FileNode>{}, _parent = <String, String>{};
void _index() {
  if (_byId.isNotEmpty) return;
  void walk(FileNode n, String? up) {
    _byId[n.id] = n;
    if (up != null) _parent[n.id] = up;
    for (final k in n.kids) {
      walk(k, n.id);
    }
  }

  for (final r in fileRoots) {
    walk(r, null);
  }
}

/// Root first, [id] last.
List<FileNode> _path(String id) {
  final out = <FileNode>[];
  for (String? at = id; at != null; at = _parent[at]) {
    out.insert(0, _byId[at]!);
  }
  return out;
}

String _kindOf(FileNode n) => switch (n.g) {
  DockG.folder => 'Folders',
  DockG.comp => 'Comps',
  DockG.video => 'Video',
  DockG.image => 'Images',
  DockG.light => 'HDR',
  DockG.audio => 'Audio',
  DockG.font => 'Fonts',
  _ => 'Docs',
};

MediaType? _typeOf(DockG g) => switch (g) {
  DockG.video => MediaType.video,
  DockG.image => MediaType.image,
  DockG.light => MediaType.hdr,
  DockG.audio => MediaType.audio,
  _ => null,
};

String _metaOf(FileNode n) {
  final a = n.media;
  if (a != null) return a.dur > 0 ? '${a.clock} · ${a.meta}' : a.meta;
  return n.folder ? (n.listed ? '${n.kids.length} items' : 'Folder') : n.meta;
}

bool _inUse(FileNode n, DockCtx c) => _layerOf(n, c) != null || c.applied(n.id);

void _go(DockCtx c, String id) => c.set(() {
  _lists.remove(id); // (a folder of the disk is read again each time it is gone to)
  c.mem.folder = id;
  c.mem.open.addAll(_pathOf(c, id).map((n) => n.id));
});

class MediaShelf extends StatelessWidget {
  const MediaShelf(this.c, {super.key});
  final DockCtx c;

  @override
  Widget build(BuildContext context) {
    _index();
    final joined = c.ws.inside != null, at = c.mem.folder;
    if (joined ? at != 'Project' && !at.startsWith('/') : !_byId.containsKey(at)) c.mem.folder = _rootsOf(c).first.id;
    return LayoutBuilder(
      builder: (context, box) {
        if (box.maxWidth >= _wide) {
          return Row(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              SizedBox(width: 160, child: _Tree(c)),
              ColoredBox(
                color: WsT.ground,
                child: SizedBox(width: WsT.gutter),
              ),
              Expanded(child: _Contents(c)),
            ],
          );
        }
        return Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            DockResizable(
              height: c.mem.treeHeight,
              min: 44,
              max: box.maxHeight - 160,
              reset: dockTreeHeight,
              gripKey: const ValueKey('browser:tree-grip'),
              onHeight: (h) => c.set(() => c.mem.treeHeight = h),
              child: _Tree(c),
            ),
            Expanded(child: _Contents(c)),
          ],
        );
      },
    );
  }
}

// ---------------------------------------------------------------- tree

class _Tree extends StatelessWidget {
  const _Tree(this.c);
  final DockCtx c;
  @override
  Widget build(BuildContext context) {
    final rows = <Widget>[];
    void walk(FileNode n, int depth) {
      final open = c.mem.open.contains(n.id), subs = open || n.listed ? n.kids.where((k) => k.folder).toList() : const <FileNode>[];
      rows.add(_FolderRow(n, depth, c, open: open, hasSubs: !n.listed || subs.isNotEmpty));
      if (open) {
        for (final s in subs) {
          walk(s, depth + 1);
        }
      }
    }

    for (final r in _rootsOf(c)) {
      walk(r, 0);
    }
    return ColoredBox(
      color: WsT.body,
      child: ListView(padding: const EdgeInsets.symmetric(vertical: 2), children: rows),
    );
  }
}

class _FolderRow extends StatelessWidget {
  const _FolderRow(this.n, this.depth, this.c, {required this.open, required this.hasSubs});
  final FileNode n;
  final int depth;
  final DockCtx c;
  final bool open, hasSubs;
  @override
  Widget build(BuildContext context) {
    final here = c.mem.folder == n.id, files = n.listed ? n.kids.where((k) => !k.folder).length : 0;
    return DockHover(
      key: ValueKey('browser:folder:${n.id}'),
      onTap: () => _go(c, n.id),
      builder: (context, h) => AnimatedContainer(
        duration: Mo.dur,
        curve: Mo.ease,
        height: 22,
        padding: EdgeInsets.only(left: 4 + depth * 12.0, right: WsT.inset),
        color: dockFill(here, h),
        child: Row(
          children: [
            GestureDetector(
              behavior: HitTestBehavior.opaque,
              onTap: hasSubs ? () => c.toggle(c.mem.open, n.id) : null,
              child: SizedBox(
                width: 14,
                height: 22,
                child: hasSubs
                    ? AnimatedRotation(
                        turns: open ? .25 : 0,
                        duration: Mo.dur,
                        curve: Mo.ease,
                        child: DockGlyph(DockG.chevron, size: 7, color: dockInk(here, Grey.g63)),
                      )
                    : null,
              ),
            ),
            DockGlyph(DockG.folder, size: 12, color: here ? WsT.onAccent : (depth == 0 ? WsT.accentInk : Grey.g63)),
            const SizedBox(width: 6),
            Expanded(
              child: Text(
                n.name,
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                style: T.name(dockInk(here, depth == 0 ? Grey.g95 : Grey.g91)).copyWith(fontWeight: depth == 0 || here ? FontWeight.w600 : FontWeight.w400),
              ),
            ),
            if (files > 0) Text('$files', style: T.value(dockInk(here, Grey.g56)).copyWith(fontSize: 10)),
          ],
        ),
      ),
    );
  }
}

// ---------------------------------------------------------------- contents

class _Contents extends StatelessWidget {
  const _Contents(this.c);
  final DockCtx c;
  @override
  Widget build(BuildContext context) {
    final path = _pathOf(c, c.mem.folder), here = path.last;
    final root = here.disk ? here : path.first; // (on the disk a search stays in the open folder and the folders already read below it)
    final searching = c.query.isNotEmpty;
    final List<FileNode> items;
    if (searching) {
      final found = <FileNode>[];
      void walk(FileNode n) {
        for (final k in n.kids) {
          if (k.folder) {
            if (k.listed) walk(k);
          } else if (c.hit(k.name)) {
            found.add(k);
          }
        }
      }

      walk(root);
      items = found;
    } else {
      items = [...here.kids.where((k) => k.folder), ...here.kids.where((k) => !k.folder)];
    }
    final kinds = <String, int>{};
    for (final n in items) {
      kinds.update(_kindOf(n), (v) => v + 1, ifAbsent: () => 1);
    }
    final chip = kinds.containsKey(c.chip) ? c.chip : 'All';
    final shown = chip == 'All' ? items : items.where((n) => _kindOf(n) == chip).toList();
    final sel = shown.where((n) => n.id == c.sel && !n.folder).firstOrNull;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        _Crumbs(path, c, searching: searching),
        DockChips([('All', items.length), for (final e in kinds.entries) (e.key, e.value)], active: chip, onPick: c.pickChip),
        Expanded(
          child: shown.isEmpty
              ? DockEmpty(c.query)
              : c.mem.mediaList
              ? ListView(
                  padding: EdgeInsets.zero,
                  children: [for (final n in shown) _Row(n, c, where: searching ? (n.disk ? _base(_dir(n.id)) : _byId[_parent[n.id]]!.name) : null)],
                )
              : LayoutBuilder(
                  builder: (context, box) {
                    final cols = (box.maxWidth / 78).floor().clamp(2, 8), w = (box.maxWidth - WsT.gutter * (cols - 1)) / cols;
                    return ListView(
                      padding: const EdgeInsets.only(top: WsT.gutter),
                      children: [
                        Wrap(
                          spacing: WsT.gutter,
                          runSpacing: WsT.gutter,
                          children: [for (final n in shown) SizedBox(width: w, child: _Tile(n, c))],
                        ),
                      ],
                    );
                  },
                ),
        ),
        if (sel != null) _Footer(sel, c),
      ],
    );
  }
}

/// Where the contents come from: each step of the path opens that folder. While searching it names the location searched.
class _Crumbs extends StatelessWidget {
  const _Crumbs(this.path, this.c, {required this.searching});
  final List<FileNode> path;
  final DockCtx c;
  final bool searching;
  @override
  Widget build(BuildContext context) => Container(
    height: 24,
    color: WsT.card,
    padding: const EdgeInsets.symmetric(horizontal: WsT.inset),
    child: Row(
      children: [
        if (searching)
          Expanded(
            child: Text('Search in ${path.first.name}', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.micro(Grey.g76)),
          )
        else
          Expanded(
            child: Row(
              children: [
                for (final (i, n) in path.indexed) ...[
                  if (i > 0)
                    Padding(
                      padding: const EdgeInsets.symmetric(horizontal: 5),
                      child: DockGlyph(DockG.chevron, size: 6, color: Grey.g56),
                    ),
                  Flexible(
                    child: GestureDetector(
                      key: ValueKey('browser:crumb:${n.id}'),
                      onTap: () => _go(c, n.id),
                      child: Text(
                        n.name,
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: T
                            .micro(i == path.length - 1 ? Grey.g95 : Grey.g63)
                            .copyWith(fontWeight: i == path.length - 1 ? FontWeight.w700 : FontWeight.w500),
                      ),
                    ),
                  ),
                ],
              ],
            ),
          ),
      ],
    ),
  );
}

/// A file's picture: its thumbnail if it is media, otherwise a flat tile with its kind's mark.
class _Art extends StatelessWidget {
  const _Art(this.n);
  final FileNode n;
  @override
  Widget build(BuildContext context) {
    if (n.media case final a?) return MediaThumb(a);
    if (_typeOf(n.g) case final t?) {
      return RepaintBoundary(
        child: CustomPaint(painter: PopArt(n.name.codeUnits.fold(0, (s, u) => s + u), t), size: Size.infinite),
      );
    }
    return switch (n.g) {
      DockG.font => Container(
        color: WsT.toneText,
        alignment: Alignment.center,
        child: Text('Aa', style: T.title(WsT.onAccent).copyWith(fontSize: 18, fontWeight: FontWeight.w700)),
      ),
      DockG.folder => Container(
        color: WsT.well,
        alignment: Alignment.center,
        child: DockGlyph(DockG.folder, size: 26, color: WsT.accentInk),
      ),
      _ => Container(color: WsT.well, alignment: Alignment.center, child: DockBadge(n.g, n.g == DockG.comp ? WsT.accent : Grey.g63, size: 26)),
    };
  }
}

void _open(FileNode n, DockCtx c) => n.folder ? _go(c, n.id) : c.pick(n.id);

class _Tile extends StatelessWidget {
  const _Tile(this.n, this.c);
  final FileNode n;
  final DockCtx c;
  @override
  Widget build(BuildContext context) {
    final sel = c.sel == n.id, a = n.media, t = _typeOf(n.g);
    return DockHover(
      key: ValueKey('browser:file:${n.id}'),
      onTap: () => _open(n, c),
      builder: (context, h) => Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          AspectRatio(
            aspectRatio: 1.25,
            child: Stack(
              fit: StackFit.expand,
              children: [
                _Art(n),
                if (t != null) Positioned(left: 3, top: 3, child: DockTag(t.badge, hue: t.hue, edge: true)),
                if (n.folder && n.listed) Positioned(right: 4, bottom: 3, child: Text('${n.kids.length}', style: T.value(Grey.g63).copyWith(fontSize: 10))),
                if (a != null && a.dur > 0)
                  Positioned(
                    right: 3,
                    bottom: 3,
                    child: Container(
                      padding: const EdgeInsets.symmetric(horizontal: 3, vertical: 2),
                      decoration: BoxDecoration(color: Grey.g07, borderRadius: BorderRadius.circular(WsT.chipRadius)),
                      child: Text(a.clock, style: T.value(Grey.g91).copyWith(fontSize: 10)),
                    ),
                  ),
                if (_inUse(n, c)) const Positioned(right: 3, top: 3, child: _InUse()),
                if (sel || h)
                  DecoratedBox(
                    decoration: BoxDecoration(
                      border: Border.all(color: sel ? WsT.accentInk : Grey.g76, width: sel ? 2 : 1),
                    ),
                  ),
              ],
            ),
          ),
          AnimatedContainer(
            duration: Mo.dur,
            curve: Mo.ease,
            height: 18,
            padding: const EdgeInsets.symmetric(horizontal: 4),
            alignment: Alignment.centerLeft,
            color: dockFill(sel, h, WsT.card),
            child: Text(
              n.name,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: T.micro(dockInk(sel, n.folder ? Grey.g91 : Grey.g76)).copyWith(fontWeight: sel || n.folder ? FontWeight.w700 : FontWeight.w500),
            ),
          ),
        ],
      ),
    );
  }
}

class _Row extends StatelessWidget {
  const _Row(this.n, this.c, {this.where});
  final FileNode n;
  final DockCtx c;

  /// The folder a search result sits in.
  final String? where;
  @override
  Widget build(BuildContext context) {
    final sel = c.sel == n.id, a = n.media;
    final meta = where == null ? _metaOf(n) : '$where · ${_metaOf(n)}';
    return DockHover(
      key: ValueKey('browser:file:${n.id}'),
      onTap: () => _open(n, c),
      builder: (context, h) => AnimatedContainer(
        duration: Mo.dur,
        curve: Mo.ease,
        height: 34,
        margin: const EdgeInsets.only(top: WsT.gutter),
        padding: const EdgeInsets.only(left: 4, right: WsT.inset),
        color: dockFill(sel, h, WsT.body),
        child: Row(
          children: [
            SizedBox(width: 40, height: 28, child: _Art(n)),
            const SizedBox(width: 8),
            Expanded(
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(n.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(dockInk(sel))),
                  const SizedBox(height: 4),
                  Text(meta, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(dockInk(sel, Grey.g63))),
                ],
              ),
            ),
            if (_inUse(n, c)) ...[const _InUse(), const SizedBox(width: 6)],
            if (a != null && a.dur > 0) Text(a.clock, style: T.value(dockInk(sel, Grey.g76)).copyWith(fontSize: 10)),
          ],
        ),
      ),
    );
  }
}

class _InUse extends StatelessWidget {
  const _InUse();
  @override
  Widget build(BuildContext context) => Container(
    width: 14,
    height: 14,
    alignment: Alignment.center,
    decoration: BoxDecoration(color: WsT.accent, borderRadius: BorderRadius.circular(WsT.chipRadius)),
    child: const DockGlyph(DockG.link, size: 10, color: WsT.onAccent),
  );
}

/// What the picked file does here: media already on a layer selects it, other media is placed, a comp opens, the rest is imported.
class _Footer extends StatelessWidget {
  const _Footer(this.n, this.c);
  final FileNode n;
  final DockCtx c;
  @override
  Widget build(BuildContext context) {
    if (c.ws.inside != null) {
      // Joined: a file the inside can show is placed (as often as wanted, each a new thing from the present moment); one in use says so.
      final layer = _layerOf(n, c), can = _kindFor(n.g) != null;
      return DockFooter(
        preview: _Art(n),
        title: n.name,
        meta: c.mem.trouble[n.id] ?? (layer != null ? 'In use · ${_metaOf(n)}' : _metaOf(n)),
        action: can ? 'Place' : 'Select',
        done: !can && layer == null,
        doneLabel: 'No seat yet',
        onAction: () => can ? _place(c, n) : c.ws.select(layer),
      );
    }
    final layer = n.media?.layer, comp = n.g == DockG.comp, media = _typeOf(n.g) != null;
    return DockFooter(
      preview: _Art(n),
      title: n.name,
      meta: _metaOf(n),
      action: layer != null ? 'Select' : (comp ? 'Open' : (media ? 'Place' : 'Import')),
      done: layer == null && c.applied(n.id),
      doneLabel: comp ? 'Open' : (media ? 'Placed' : 'Imported'),
      onAction: () => layer != null ? c.ws.select(layer) : c.apply(n.id),
    );
  }
}
