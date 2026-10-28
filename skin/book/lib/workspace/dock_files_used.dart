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

/// The files picked together (⌘ and a press, as Finder adds to what is picked): placed together, one after another.
final _filesPicked = <String>{};

/// The files picked together put on the stage, each a new thing from the present moment, each film as long as it plays
/// (the discoverer's length: a film said no length lasts to the document's end), then, all of them films, laid one after
/// another in the order picked (the inside's Stagger: Alight Motion adds several so); all of them picked. A still lasts
/// as long as chosen under it (End: stills are laid together).
Future<void> _placeAll(DockCtx c, List<String> files) async {
  final m = c.ws.inside!;
  final lengths = [for (final (i, s) in (await Future.wait([for (final f in files) mediaSecondsOf(m.libPath, f)])).indexed) _kindOfFile(files[i]) == 'picture' ? _stillSeconds : s];   // (a still: as long as chosen)
  m.later('together', number: 1);
  final made = <Future<int>>[];
  for (final (i, f) in files.indexed) {
    final asked = Completer<int>();
    c.ws.after(m.later('add', word: f), (done, why) {
      if (done) return;
      c.set(() => c.mem.trouble[f] = why);
      asked.complete(0);
    }, asked.complete);
    if (lengths[i] > 0) m.later('set', thing: -1, name: 'duration', number: lengths[i]);   // (the one just added)
    made.add(asked.future);
  }
  c.ws.after(m.later('together'));
  Future.wait(made).then((ids) {
    final all = [for (final i in ids) if (i > 0) i];
    if (all.length > 1 && lengths.every((s) => s > 0)) c.ws.after(m.later('time_align', word: 'stagger ${all.join(' ')}'), (_, _) => c.ws.reread());
    c.ws.pickMany([for (final i in all) '$i']);
    _filesPicked.clear();
  });
}

/// A file put on the stage as a new thing, picked. Not done: the foot of the shelf says why.
void _place(DockCtx c, FileNode n) {
  final m = c.ws.inside!, still = _kindOfFile(n.id) == 'picture' && _stillSeconds > 0, cut = _kindOfFile(n.id) == 'film' ? _filmRanges[n.id] : null; // (the inside makes of it what the file is; without waiting, picked once made)
  if (still || cut != null) m.later('together', number: 1);   // (made and given its length: one step)
  c.ws.after(m.later('add', word: n.id), (done, why) => c.set(() => done ? c.mem.trouble.remove(n.id) : c.mem.trouble[n.id] = why), (made) => c.ws.select('$made'));
  if (still) m.later('set', thing: -1, name: 'duration', number: _stillSeconds);   // (the one just added: as long as chosen under it)
  if (cut case (final a, final b)) {
    mediaSecondsOf(m.libPath, n.id).then((len) {   // (known already: the band drew it)
      m.later('set', thing: -1, name: 'begin', number: a * len);   // (from where the band's left end is in the film)
      m.later('set', thing: -1, name: 'duration', number: (b - a) * len);
      c.ws.after(m.later('together'), (_, _) => c.ws.reread());
    });
  } else if (still) {
    c.ws.after(m.later('together'), (_, _) => c.ws.reread());
  }
}
