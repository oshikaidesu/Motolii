// dock_files_browser.dart: the Media shelf as one browser: where it starts, the way to a folder, the tree, the folder rows, the crumbs, going and opening.
part of 'dock_files.dart';

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

void _go(DockCtx c, String id) => c.set(() {
  if (id.startsWith('/')) _read(id); // (a folder of the disk is read again each time it is gone to)
  c.mem.folder = id;
  // (only that folder opens: the same folder is listed under more than one place, and opening every place above it
  //  moves the rows under the hand)
  c.mem.open.add(id);
});

void _open(FileNode n, DockCtx c) => n.folder ? _go(c, n.id) : c.pick(n.id);

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
