import 'dart:io';

import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:flutter/widgets.dart';
import 'package:flutter/services.dart';

import '../../foundation/metrics.dart';
import '../../foundation/theme.dart';
import 'media_shelf.dart';
import 'shelf.dart';
import '../../foundation/glyphs.dart';
import '../../foundation/leaves.dart';

/// Files: a window onto real folders, the way AEViewer sits beside AE.
/// Nothing here touches the document; a double-click on a file admits it to
/// Media, a double-click on a folder walks in.
class FilesShelf extends BrowserShelf {
  FilesShelf({this.initialFolder});

  /// Where the shelf opens; the home folder when unset.
  final String? initialFolder;

  @override
  String get name => 'Files';

  String? folder;
  final folderBack = <String>[];
  final folderForward = <String>[];
  List<Map<String, dynamic>> listing = [];
  String? listingError;

  static String get _home => kIsWeb ? '/' : Platform.environment['HOME'] ?? '/';
  static Map<String, String> get _places => kIsWeb
      ? const {}
      : {
          'Home': _home,
          'Desktop': '$_home/Desktop',
          'Downloads': '$_home/Downloads',
          'Pictures': '$_home/Pictures',
          'Movies': '$_home/Movies',
          'Music': '$_home/Music',
        };

  @override
  List<String> rails(BrowserHost host) => _places.keys.toList();

  /// The rail is places, not categories: the entry in force is the place the
  /// folder is, or none.
  @override
  String chosen(BrowserHost host, String category) =>
      _places.entries
          .where((e) => e.value == folder)
          .map((e) => e.key)
          .firstOrNull ??
      '';
  @override
  void rail(BrowserHost host, String entry) => _go(host, _places[entry]!);
  @override
  bool passes(BrowserHost host, Map<String, dynamic> item, String chosen) =>
      true;

  @override
  void enter(BrowserHost host) {
    if (folder != null) return;
    folder = initialFolder ?? _home;
    _read(host);
  }

  @override
  List<Map<String, dynamic>> items(BrowserHost host) => listing;
  @override
  String classification(BrowserHost host, Map<String, dynamic> item) => 'All';

  @override
  bool supported(BrowserHost host, Map<String, dynamic> item) =>
      item['folder'] == true || host.has('import');
  @override
  String identity(BrowserHost host, Map<String, dynamic> item) =>
      mediaFamily(item);
  @override
  String format(BrowserHost host, Map<String, dynamic> item) =>
      mediaFormat(item);

  @override
  Widget preview(BrowserHost host, Map<String, dynamic> item, Color identity) =>
      item['folder'] == true
      ? Center(
          child: Icon(
            Glyph.folder,
            size: EditorMetrics.s32 * host.tileScale,
            color: EditorTheme.of(host.context).tab,
          ),
        )
      : mediaThumbnail(item, host.tileScale);

  @override
  Future<void> apply(BrowserHost host, Map<String, dynamic> item) async {
    if (item['folder'] == true) {
      _go(host, '${item['path']}');
    } else if (host.has('import')) {
      await host.controller.importPaths(['${item['path']}']);
    }
  }

  @override
  String applyLabel(BrowserHost host, Map<String, dynamic> item) =>
      item['folder'] == true ? 'Open' : 'Import to Media';

  @override
  List<String> facts(BrowserHost host, Map<String, dynamic> item) {
    final path = filePath(item);
    return [
      if (item['folder'] == true)
        'Folder'
      else ...[
        [
          if (mediaFormat(item).isNotEmpty) mediaFormat(item),
          if (item['mime'] != null) '${item['mime']}',
        ].join(' · '),
        if (path != null) fileFact(path),
      ],
      if (path != null) homely(fileParent(path)),
    ];
  }

  @override
  List<EditorMenuItem<String>> menu(
    BrowserHost host,
    Map<String, dynamic> item,
  ) => filePath(item) == null
      ? const []
      : [
          EditorMenuItem<String>(value: 'reveal', child: Text(revealLabel)),
          if (item['folder'] != true)
            const EditorMenuItem<String>(
              value: 'open',
              child: Text('Open with default app'),
            ),
          const EditorMenuItem<String>(
            value: 'copyPath',
            child: Text('Copy path'),
          ),
        ];

  @override
  Future<void> act(
    BrowserHost host,
    String action,
    Map<String, dynamic> item,
  ) async {
    final path = filePath(item);
    switch (action) {
      case 'reveal':
        await host.controller.native('reveal', {'path': path});
      case 'open':
        await host.controller.native('openFile', {'path': path});
      case 'copyPath':
        await Clipboard.setData(ClipboardData(text: path ?? ''));
    }
  }

  /// Where we are: back, forward, up, and the path as crumbs you can press.
  @override
  Widget header(BrowserHost host) {
    final here = folder ?? _home;
    final crumbs = homely(here).split('/').where((c) => c.isNotEmpty).toList();
    String pathTo(int i) {
      final head = crumbs.first == '~' ? _home : '';
      final rest = crumbs.sublist(crumbs.first == '~' ? 1 : 0, i + 1);
      return rest.isEmpty ? head : '$head/${rest.join('/')}';
    }

    Widget step(IconData icon, String label, VoidCallback? press) =>
        EditorTooltip(
          message: label,
          child: EditorIconButton(
            iconSize: EditorMetrics.s14,
            color: press == null
                ? EditorTheme.of(host.context).disabledInk
                : EditorTheme.of(host.context).muted,
            onPressed: press,
            icon: Icon(icon),
          ),
        );
    return Container(
      key: const ValueKey('browser:path'),
      height: EditorMetrics.control,
      padding: const EdgeInsets.symmetric(horizontal: EditorMetrics.s4),
      decoration: BoxDecoration(
        border: Border(
          bottom: BorderSide(color: EditorTheme.of(host.context).line),
        ),
      ),
      child: Row(
        children: [
          step(
            Glyph.arrow_back,
            'Back',
            folderBack.isEmpty ? null : () => _back(host),
          ),
          step(
            Glyph.arrow_forward,
            'Forward',
            folderForward.isEmpty ? null : () => _forward(host),
          ),
          step(Glyph.arrow_upward, 'Up', kIsWeb ? null : () => _up(host)),
          const SizedBox(width: EditorMetrics.s4),
          Expanded(
            child: SingleChildScrollView(
              scrollDirection: Axis.horizontal,
              reverse: true,
              child: Row(
                children: [
                  for (var i = 0; i < crumbs.length; i++) ...[
                    if (i > 0)
                      Text(
                        '›',
                        style: TextStyle(
                          color: EditorTheme.of(host.context).muted,
                        ),
                      ),
                    EditorPress(
                      onTap: i == crumbs.length - 1
                          ? null
                          : () => _go(host, pathTo(i)),
                      child: Padding(
                        padding: const EdgeInsets.symmetric(
                          horizontal: EditorMetrics.s3,
                        ),
                        child: Text(
                          crumbs[i],
                          style: TextStyle(
                            fontSize: EditorMetrics.font,
                            color: i == crumbs.length - 1
                                ? EditorTheme.of(host.context).ink
                                : EditorTheme.of(host.context).muted,
                          ),
                        ),
                      ),
                    ),
                  ],
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  /// Walk to a folder, remembering where we came from.
  void _go(BrowserHost host, String path, {bool remember = true}) {
    if (remember && folder != null && folder != path) {
      folderBack.add(folder!);
      folderForward.clear();
    }
    folder = path;
    host.clearSelection();
    _read(host);
    host.relist();
  }

  void _back(BrowserHost host) {
    if (folderBack.isEmpty) return;
    folderForward.add(folder!);
    _go(host, folderBack.removeLast(), remember: false);
  }

  void _forward(BrowserHost host) {
    if (folderForward.isEmpty) return;
    folderBack.add(folder!);
    _go(host, folderForward.removeLast(), remember: false);
  }

  void _up(BrowserHost host) {
    if (kIsWeb) return;
    final parent = Directory(folder!).parent.path;
    if (parent != folder) _go(host, parent);
  }

  /// Read the folder: folders first, then files the shelf can take, by name.
  /// Hidden entries and everything else stay out of sight. The read is
  /// synchronous: a folder of a few hundred entries lists in a millisecond,
  /// and the answer is on screen in the same frame as the press.
  void _read(BrowserHost host) {
    if (kIsWeb) {
      listing = const [];
      listingError = 'Files are unavailable in browser';
      return;
    }
    final path = folder;
    if (path == null) return;
    final allowed = (host.controller.state['importExtensions'] as List? ?? [])
        .map((e) => '$e'.toLowerCase())
        .toSet();
    final folders = <Map<String, dynamic>>[];
    final files = <Map<String, dynamic>>[];
    String? error;
    try {
      for (final entry in Directory(path).listSync(followLinks: false)) {
        final name = entry.path.split('/').last;
        if (name.startsWith('.')) continue;
        if (entry is Directory) {
          folders.add({
            'id': entry.path,
            'name': name,
            'path': entry.path,
            'folder': true,
          });
        } else if (entry is File) {
          final dot = name.lastIndexOf('.');
          final ext = dot > 0 ? name.substring(dot + 1).toLowerCase() : '';
          if (allowed.isNotEmpty && !allowed.contains(ext)) continue;
          final mime = _mimeOf(ext);
          files.add({
            'id': entry.path,
            'name': name,
            'path': entry.path,
            'mime': mime,
            if (mime.startsWith('image/') && ext != 'hdr' && ext != 'exr')
              'thumbnail': entry.path,
          });
        }
      }
    } catch (e) {
      error = 'Cannot read this folder';
    }
    int byName(Map<String, dynamic> a, Map<String, dynamic> b) =>
        '${a['name']}'.toLowerCase().compareTo('${b['name']}'.toLowerCase());
    folders.sort(byName);
    files.sort(byName);
    listing = [...folders, ...files];
    listingError = error;
  }

  /// A rough MIME from the extension, enough to colour the badge and pick
  /// the family; the native side decides for real once a file is admitted.
  static String _mimeOf(String ext) => switch (ext) {
    'mp4' || 'mov' || 'mkv' || 'webm' || 'm4v' => 'video/$ext',
    'wav' || 'mp3' || 'flac' || 'aac' || 'aiff' || 'm4a' => 'audio/$ext',
    'obj' || 'glb' || 'gltf' || 'ply' => 'model/$ext',
    'hdr' || 'exr' => 'image/$ext',
    '' => 'application/octet-stream',
    _ => 'image/$ext',
  };
}
