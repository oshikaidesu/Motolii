// The native channel of a New shell test: what the session asked native, and settings that persist across shells.
import 'dart:convert';

import 'package:flutter/services.dart';
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
  /// A document of the test's own, answered for attach, render, open and every request; a `relate` request writes its
  /// links into it the way the host would, so the shell's projection of relations is exercised.
  Map<String, dynamic>? document;

  List<String> capabilities = ['stageWindow', 'create', 'undo', 'redo', 'save', 'export', 'seek', 'previewProperties', 'commitPreview'];
  final calls = <(String, Map)>[];
  List<Map> get writes => [for (final c in calls) if (c.$1 == 'writeSettings') c.$2];
  /// The operations the session sent to the runtime, in order (each `request` carries one JSON command).
  List<Map<String, dynamic>> get operations => [
    for (final c in calls)
      if (c.$1 == 'request' && c.$2['command'] is String) Map<String, dynamic>.from(jsonDecode(c.$2['command'] as String) as Map),
  ];
  List<String> get ops => [for (final o in operations) '${o['op']}'];

  void _relate(Map<String, dynamic> j) {
    for (final l in (document!['layers'] as List).cast<Map<String, dynamic>>()) {
      if (!(j['members'] as List).contains(l['id'])) continue;
      for (final r in (l['properties'] as List).cast<Map<String, dynamic>>()) {
        if (r['id'] != j['property']) continue;
        r['link'] = {'layer': j['source']['layer'], 'property': j['source']['property'], 'component': j['source']['component'] ?? 0, 'kind': 'motolii.link.remap', 'inMin': j['inMin'], 'inMax': j['inMax'], 'outMin': j['outMin'], 'outMax': j['outMax']};
      }
    }
  }

  void _unrelate(Map<String, dynamic> j) {
    for (final l in (document!['layers'] as List).cast<Map<String, dynamic>>()) {
      if (!(j['layers'] as List).contains(l['id'])) continue;
      for (final r in (l['properties'] as List).cast<Map<String, dynamic>>()) {
        if (r['id'] == j['property']) r.remove('link');
      }
    }
  }

  /// A fresh copy each time, as the host sends fresh JSON: the session's slices fire on new lists, not mutated ones.
  Map<String, dynamic> _reply() => {...Map<String, dynamic>.from(jsonDecode(jsonEncode(document)) as Map), 'capabilities': document!['capabilities'] ?? capabilities};

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
        case 'placePanel':
          // The host answers a placement by asking the shell to place the panel, on the same channel.
          TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.handlePlatformMessage(
            EditorSession.channel.name,
            EditorSession.channel.codec.encodeMethodCall(MethodCall('placePanel', args)),
            (_) {},
          );
          return true;
        case 'request':
          if (document != null && args['command'] is String) {
            final command = Map<String, dynamic>.from(jsonDecode(args['command'] as String) as Map);
            if (command['op'] == 'relate') _relate(command);
            if (command['op'] == 'unrelate') _unrelate(command);
            if (command['op'] == 'select') document!['selectedIds'] = List<int>.from(command['ids'] as List);
            return _reply();
          }
          return <String, dynamic>{};
        case 'attach':
        case 'render':
        case 'open':
        case 'command':
          if (document != null) return _reply();
          return {'createKinds': createKinds, 'layers': [], 'selectedIds': [], 'selectedKeys': [], 'capabilities': capabilities, 'width': 1920, 'height': 1080, 'durationFrames': 300, 'fps': 30};
        default:
          return <String, dynamic>{};
      }
    });
  }
}

