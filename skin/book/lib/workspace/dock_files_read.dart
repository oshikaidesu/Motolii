// dock_files_read.dart: the folders read (once each, off the window's thread) and what a file is: its name, folder, size, kind, the media it is.
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
