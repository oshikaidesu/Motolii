// The native channel of a New shell test: what the session asked native, and settings that persist across shells.
import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';

import '../../lib/session/editor_session.dart';

/// The host's table of what can be created (Rust: editor::create::kinds_json), a few of it for the doubles.
const createKinds = [
  {'id': 'text', 'name': 'Text', 'detail': 'Adds a text layer', 'rail': 'Text'},
  {'id': 'rectangle', 'name': 'Rectangle', 'detail': 'Adds a shape layer', 'rail': 'Shapes'},
  {'id': 'ellipse', 'name': 'Ellipse', 'detail': 'Adds a shape layer', 'rail': 'Shapes'},
  {'id': 'null', 'name': 'Null', 'detail': 'Adds an empty layer to parent others to', 'rail': 'Helpers'},
  {'id': 'camera', 'name': 'Camera', 'detail': 'Adds a camera layer', 'rail': '3D'},
  {'id': 'cube', 'name': 'Cube', 'detail': 'Adds a 3D cube', 'rail': '3D'},
];

class Native {
  Map<String, dynamic> settings = {};

  /// Which window this session is: the main one, or a window that holds only the panels it was given.
  Map<String, dynamic> windowInfo = {'id': 'main', 'main': true};
  int windows = 0;

  /// What the runtime says it can do; a test that needs an operation adds it.
  List<String> capabilities = ['stageWindow', 'create', 'undo', 'redo', 'save', 'export', 'seek', 'previewProperties', 'commitPreview'];
  final calls = <(String, Map)>[];
  List<Map> get writes => [for (final c in calls) if (c.$1 == 'writeSettings') c.$2];
  /// The operations the session sent to the runtime, in order (each `request` carries one JSON command).
  List<Map<String, dynamic>> get operations => [
    for (final c in calls)
      if (c.$1 == 'request' && c.$2['command'] is String) Map<String, dynamic>.from(jsonDecode(c.$2['command'] as String) as Map),
  ];
  List<String> get ops => [for (final o in operations) '${o['op']}'];

  void install() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(EditorSession.channel, (call) async {
      final args = Map<String, dynamic>.from((call.arguments as Map?) ?? {});
      calls.add((call.method, args));
      switch (call.method) {
        case 'windowInfo':
          return windowInfo;
        case 'openPanelWindow':
          return {'id': 'w${++windows}', 'panels': args['panels']};
        case 'readSettings':
          return settings;
        case 'writeSettings':
          settings = args;
          return true;
        case 'pickSave':
        case 'pickExport':
          return '/tmp/motolii-test/${call.method == 'pickExport' ? 'out.mp4' : 'work.rrd'}';
        case 'pickOpen':
          return '/tmp/motolii-test/work.rrd';
        case 'attach':
        case 'render':
        case 'open':
        case 'command':
          return {'createKinds': createKinds, 'layers': [], 'selectedIds': [], 'selectedKeys': [], 'capabilities': capabilities, 'width': 1920, 'height': 1080, 'durationFrames': 300, 'fps': 30};
        default:
          return <String, dynamic>{};
      }
    });
  }
}

