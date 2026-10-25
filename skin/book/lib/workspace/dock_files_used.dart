// dock_files_used.dart: which files the document uses (and by which thing), and a file put on the stage.
part of 'dock_files.dart';

var _usedNow = <String, String>{};

int? _usedRev;

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

/// The thing of the document a file is on.
String? _layerOf(FileNode n, DockCtx c) => _used(c.ws)[n.id];

bool _inUse(FileNode n, DockCtx c) => _layerOf(n, c) != null;

/// A file put on the stage as a new thing, picked. Not done: the foot of the shelf says why.
void _place(DockCtx c, FileNode n) {
  final m = c.ws.inside!; // (the inside makes of it what the file is; without waiting, picked once made)
  c.ws.after(m.later('add', word: n.id), (done, why) => c.set(() => done ? c.mem.trouble.remove(n.id) : c.mem.trouble[n.id] = why), (made) => c.ws.select('$made'));
}
