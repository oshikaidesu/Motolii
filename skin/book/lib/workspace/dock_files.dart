// The Media shelf, AEViewer 2 style: one browser for the project's files and media. A folder tree (locations, then folders only) beside
// or above the open folder's contents, shown as thumbnails or as a list. Wide: tree | contents. Narrow: tree band (height by drag) over contents.
import 'dart:async';
import 'dart:io';
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../motolii.dart' show mediaInfoOf, mediaSecondsOf;
import '../sets/inspector/inspector_parts.dart' show Pop;
import '../tokens.dart';
import 'dock_resizable.dart';
import 'dock_glyphs.dart';
import 'dock_parts.dart';
import 'dock_tags.dart';
import 'ws.dart';
part 'dock_files_read.dart';
part 'dock_files_used.dart';
part 'dock_files_browser.dart';
part 'dock_files_contents.dart';

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

}

// ---------------------------------------------------------------- the disk

/// What a file of the disk is here: the inside says what kind of thing it makes (null: none, not shown); the mark for
/// each kind is the shelf's own.
String Function(String file) _kindOfFile = (_) => '';
Future<String?> Function(String film, int step, int steps)? _thumbnailAtOf;
DockG? _glyphOf(String file) => switch (_kindOfFile(file)) {
  'film' => DockG.video,
  'picture' => DockG.image,
  'solid' => DockG.shape,
  'text' => DockG.font,
  _ => null,
};
