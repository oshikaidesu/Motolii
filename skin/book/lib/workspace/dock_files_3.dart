// Part of dock_files.dart (cut there by size, as it was).
part of 'dock_files.dart';

String _base(String path) => path.substring(path.lastIndexOf('/') + 1);
String _dir(String path) => path.lastIndexOf('/') <= 0 ? '/' : path.substring(0, path.lastIndexOf('/'));
String _size(int bytes) => bytes >= 1 << 20 ? '${(bytes / (1 << 20)).toStringAsFixed(1)} MB' : '${(bytes / 1024).ceil()} KB';

final _home = Platform.environment['HOME'] ?? '/';

final _lists = <String, List<FileNode>>{};
final _reading = <String>{};

/// Goes up each time a folder of the disk has been read: the shelf then looks again.
final dockRead = ValueNotifier<int>(0);

/// A folder of the disk as it was last read, read again without waiting (the window never stands still for the
/// disk: a folder the system asks leave for is simply empty until leave is given).
List<FileNode> _listed(String path) {
  if (!_lists.containsKey(path)) _read(path);
  return _lists[path] ?? const [];
}

void _read(String path) {
  if (!_reading.add(path)) return;
  _lists.putIfAbsent(path, () => const []);
  () async {
    final out = <FileNode>[];
    try {
      await for (final e in Directory(path).list()) {
        final name = _base(e.path);
        if (name.startsWith('.')) continue;
        if (e is Directory) {
          out.add(FileNode(e.path, name, DockG.folder));
        } else if (_glyphOf(name) case final g?) {
          out.add(FileNode(e.path, name, g, meta: '${name.substring(name.lastIndexOf('.') + 1).toUpperCase()} · ${_size((await e.stat()).size)}'));
        }
      }
    } on FileSystemException {
      out.clear();
    }
    _lists[path] = out..sort((a, b) => a.name.toLowerCase().compareTo(b.name.toLowerCase()));
    _reading.remove(path);
    dockRead.value++;
  }();
}

/// The files the document's things are made of, by whole path: the thing that uses each (the last one, the one in front).
Map<String, String> _used(Ws ws) {
  if (_usedRev == ws.rev) return _usedNow;
  final m = ws.inside!;
  _usedRev = ws.rev;
  return _usedNow = {
    for (final l in ws.layers.reversed) // (the nearest thing is the first row: it is told last, so it is the one kept)
      if (m.word(int.parse(l.id), 'file') case final f when f.isNotEmpty && m.word(int.parse(l.id), 'kind') != 'behaviour') m.whole(f): l.id,   // (a behaviour's file is its kind's name, no file)
  };
}

var _usedNow = <String, String>{};
int? _usedRev;

/// The locations of a joined window: what the document is made of (its files, and the folders they are in), then the disk.
List<FileNode> _rootsOf(DockCtx c) {
  _kindOfFile = c.ws.inside!.kindOf;
  _thumbnailAtOf = c.ws.inside!.thumbnailAt;
  final files = _used(c.ws).keys.toList()..sort();
  return [
    FileNode(
      'Project',
      'Project',
      DockG.folder,
      kids: [
        for (final d in {?c.ws.documentFolder, for (final f in files) _dir(f)}) FileNode(d, _base(d), DockG.folder), // (the document's own folder first: a new document is one step from what lies beside it)
        for (final f in files)
          if (_glyphOf(f) case final g?) FileNode(f, _base(f), g),
      ],
    ),
    FileNode(_home, 'Home', DockG.folder),
  ];
}

/// Root first, [id] last.
List<FileNode> _pathOf(DockCtx c, String id) {
  final roots = _rootsOf(c);
  if (!id.startsWith('/')) return [roots.first];
  final out = <FileNode>[];
  for (var at = id; ; at = _dir(at)) {
    if (at == _home || at == '/') return out..insert(0, FileNode(at, at == _home ? 'Home' : '/', DockG.folder));
    out.insert(0, FileNode(at, _base(at), DockG.folder));
  }
}

/// The thing of the document a file is on.
String? _layerOf(FileNode n, DockCtx c) => _used(c.ws)[n.id];

/// A file put on the stage as a new thing, picked. Not done: the foot of the shelf says why.
void _place(DockCtx c, FileNode n) {
  final m = c.ws.inside!; // (the inside makes of it what the file is; without waiting, picked once made)
  c.ws.after(m.later('add', word: n.id), (done, why) => c.set(() => done ? c.mem.trouble.remove(n.id) : c.mem.trouble[n.id] = why), (made) => c.ws.select('$made'));
}

String _kindOf(FileNode n) => switch (n.g) {
  DockG.folder => 'Folders',
  DockG.comp => 'Comps',
  DockG.video => 'Video',
  DockG.image => 'Images',
  DockG.light => 'HDR',
  DockG.audio => 'Audio',
  DockG.font => 'Fonts',
  DockG.shape => '3D',
  _ => 'Docs',
};

/// The kinds of media a file is, by the shelf's mark: a tag and a hue.
enum MediaType {
  video('Video', 'VID', DockG.video),
  image('Images', 'IMG', DockG.image),
  hdr('HDR', 'HDR', DockG.light),
  audio('Audio', 'AUD', DockG.audio);

  const MediaType(this.label, this.badge, this.glyph);
  final String label, badge;
  final DockG glyph;
  Color get hue => switch (this) {
    video => Pop.toneTime,
    image => WsT.toneImage,
    hdr => Pop.toneTextAnim,
    audio => WsT.toneAudio,
  };
}

MediaType? _typeOf(DockG g) => switch (g) {
  DockG.video => MediaType.video,
  DockG.image => MediaType.image,
  DockG.light => MediaType.hdr,
  DockG.audio => MediaType.audio,
  _ => null,
};

String _metaOf(FileNode n) {
  return n.folder ? (n.listed ? '${n.kids.length} items' : 'Folder') : n.meta;
}

bool _inUse(FileNode n, DockCtx c) => _layerOf(n, c) != null;

void _go(DockCtx c, String id) => c.set(() {
  if (id.startsWith('/')) _read(id); // (a folder of the disk is read again each time it is gone to)
  c.mem.folder = id;
  // (only that folder opens: the same folder is listed under more than one place, and opening every place above it
  //  moves the rows under the hand)
  c.mem.open.add(id);
});

class MediaShelf extends StatelessWidget {
  const MediaShelf(this.c, {super.key});
  final DockCtx c;

  @override
  Widget build(BuildContext context) => ValueListenableBuilder<int>(valueListenable: dockRead, builder: (context, _, _) => _shelf(context));

  Widget _shelf(BuildContext context) {
    final at = c.mem.folder;
    if (at != 'Project' && !at.startsWith('/')) c.mem.folder = _rootsOf(c).first.id;
    return LayoutBuilder(
      builder: (context, box) {
        if (switch (c.mem.treeSide) { 1 => true, 2 => false, _ => box.maxWidth >= _wide }) {   // (the tree by the room, or where the hand put it)
          return Row(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              SizedBox(width: Shape.z(160), child: _Tree(c)),
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
      child: ListView(padding: EdgeInsets.symmetric(vertical: Shape.s(2)), children: rows),
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
        height: Shape.z(22),
        padding: EdgeInsets.only(left: Shape.s(4) + depth * Shape.s(12.0), right: WsT.inset),
        color: dockFill(here, h),
        child: Row(
          children: [
            GestureDetector(
              behavior: HitTestBehavior.opaque,
              onTap: hasSubs ? () => c.toggle(c.mem.open, n.id) : null,
              child: SizedBox(
                width: Shape.z(14),
                height: Shape.z(22),
                child: hasSubs
                    ? AnimatedRotation(
                        turns: open ? .25 : 0,
                        duration: Mo.dur,
                        curve: Mo.ease,
                        child: DockGlyph(DockG.chevron, size: Shape.z(7), color: dockInk(here, Grey.g63)),
                      )
                    : null,
              ),
            ),
            DockGlyph(DockG.folder, size: Shape.z(12), color: here ? WsT.onAccent : (depth == 0 ? WsT.accentInk : Grey.g63)),
            SizedBox(width: Shape.s(6)),
            Expanded(
              child: Text(
                n.name,
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                style: T.name(dockInk(here, depth == 0 ? Grey.g95 : Grey.g91)).copyWith(fontWeight: depth == 0 || here ? FontWeight.w600 : FontWeight.w400),
              ),
            ),
            if (files > 0) Text('$files', style: T.value(dockInk(here, Grey.g56)).copyWith(fontSize: T.px(10))),
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
          } else if (DockTags.hit(c.query, k.name, DockTags.of('Media', k.id, {_kindOf(k).toLowerCase(), if (k.id.contains('.')) k.id.split('.').last.toLowerCase()}))) {
            found.add(k);
          }
        }
      }

      walk(root);
      items = found;
    } else {
      if (c.mem.mediaDeep) {   // (the folder and every folder below it, their files together: a folder looked at as one pile)
        final all = <FileNode>[];
        void below(FileNode f, int depth) {
          if (f.disk && !f.listed && depth > 0) _read(f.id);
          for (final k in f.kids) {
            if (k.folder) {
              if (depth < 4) below(k, depth + 1);
            } else {
              all.add(k);
            }
          }
        }

        below(here, 0);
        items = [...here.kids.where((k) => k.folder), ...all];
      } else {
        items = [...here.kids.where((k) => k.folder), ...here.kids.where((k) => !k.folder)];
      }
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
                  children: [for (final n in shown) _Row(n, c, where: searching ? (n.disk ? _base(_dir(n.id)) : '') : null)],
                )
              : LayoutBuilder(
                  builder: (context, box) {
                    final cols = (box.maxWidth / 78).floor().clamp(2, 8), w = (box.maxWidth - WsT.gutter * (cols - 1)) / cols;
                    return ListView(
                      padding: EdgeInsets.only(top: WsT.gutter),
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
    height: Shape.z(24),
    color: WsT.card,
    padding: EdgeInsets.symmetric(horizontal: WsT.inset),
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
                      padding: EdgeInsets.symmetric(horizontal: Shape.s(5)),
                      child: DockGlyph(DockG.chevron, size: Shape.z(6), color: Grey.g56),
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
  const _Art(this.n, {this.step = 0});
  final FileNode n;
  final int step;   // (a film: the picture this far along it, of eight)
  @override
  Widget build(BuildContext context) {
    if (n.disk && (n.g == DockG.video || n.g == DockG.image)) {
      // A file of the disk shows its own picture: a film its first one (the inside makes it), a photo itself (Flutter
      // reads it); what has none to show (a vector picture, until it is made) is a flat tile with its kind's mark.
      final flat = Container(color: WsT.well, alignment: Alignment.center, child: DockBadge(n.g, Grey.g63, size: Shape.z(26)));
      if (n.g == DockG.video) {
        return FutureBuilder<String?>(
          future: _thumbnailAtOf?.call(n.id, step, 8),
          builder: (context, made) => made.data == null ? flat : ColoredBox(color: WsT.well, child: Image.file(File(made.data!), fit: BoxFit.contain, gaplessPlayback: true)),   // (the whole picture, not cut to a square)
        );
      }
      final photo = RegExp(r'\.(png|jpe?g|webp)$', caseSensitive: false).hasMatch(n.id);
      return photo ? ColoredBox(color: WsT.well, child: Image.file(File(n.id), fit: BoxFit.contain, cacheWidth: 240, errorBuilder: (_, _, _) => flat)) : flat;
    }
    return switch (n.g) {
      DockG.font => Container(
        color: WsT.toneText,
        alignment: Alignment.center,
        child: Text('Aa', style: T.title(WsT.onAccent).copyWith(fontSize: T.px(18), fontWeight: FontWeight.w700)),
      ),
      DockG.folder => Container(
        color: WsT.well,
        alignment: Alignment.center,
        child: DockGlyph(DockG.folder, size: Shape.z(26), color: WsT.accentInk),
      ),
      _ => Container(color: WsT.well, alignment: Alignment.center, child: DockBadge(n.g, n.g == DockG.comp ? WsT.accent : Grey.g63, size: Shape.z(26))),
    };
  }
}

void _open(FileNode n, DockCtx c) => n.folder ? _go(c, n.id) : c.pick(n.id);
