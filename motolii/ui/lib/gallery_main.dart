import 'package:flutter/services.dart';
import 'package:flutter/widget_previews.dart';
import 'package:flutter/widgets.dart';

import 'app/editor_app.dart';
import 'bridge/native_bridge.dart';
import 'session/editor_session.dart';

void main() {
  const stories = {'default', 'dense', 'text'};
  final requested = Uri.base.queryParameters['story'];
  runApp(
    GalleryStory(story: stories.contains(requested) ? requested! : 'default'),
  );
}

@Preview(name: 'Default workspace', group: 'Motolii')
Widget defaultWorkspacePreview() => const GalleryStory(story: 'default');

@Preview(name: 'Dense timeline', group: 'Motolii')
Widget denseWorkspacePreview() => const GalleryStory(story: 'dense');

@Preview(name: 'Text selected', group: 'Motolii')
Widget textWorkspacePreview() => const GalleryStory(story: 'text');

class GalleryStory extends StatefulWidget {
  const GalleryStory({super.key, required this.story});

  final String story;

  @override
  State<GalleryStory> createState() => _GalleryStoryState();
}

class _GalleryStoryState extends State<GalleryStory> {
  late final Map<String, dynamic> status;
  late final _PreviewBridge bridge;
  late final EditorSession session;

  @override
  void initState() {
    super.initState();
    status = _status(widget.story);
    bridge = _PreviewBridge(status);
    session = EditorSession(bridge: bridge)
      ..windowInfo = const {'id': 'preview', 'main': true}
      ..document.value = status
      ..frame.value = (status['frame'] as num? ?? 0).toInt();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) session.deskWork.value = const {'theme': _velvetTheme};
    });
  }

  @override
  void dispose() {
    session.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) =>
      EditorApp(controller: session, initialize: false);
}

class _PreviewBridge extends NativeBridge {
  _PreviewBridge(this.status);

  final Map<String, dynamic> status;
  Future<dynamic> Function(MethodCall)? _handler;

  @override
  void listen(Future<dynamic> Function(MethodCall)? handler) {
    _handler = handler;
  }

  @override
  Future<dynamic> invoke(
    String method, [
    Map<String, dynamic> arguments = const {},
  ]) async {
    switch (method) {
      case 'windowInfo':
        return const {'id': 'preview', 'main': true};
      case 'attach':
      case 'render':
      case 'request':
        return status;
      case 'readSettings':
        return const {
          'deskWork': {'theme': _velvetTheme},
        };
      case 'placePanel':
        await _handler?.call(MethodCall('placePanel', arguments));
        return true;
      case 'openPanelWindow':
        return const {'id': 'preview-panel'};
      case 'pickOpen':
      case 'pickSave':
      case 'pickImport':
      case 'pickExport':
        return null;
      default:
        return const <String, dynamic>{};
    }
  }
}

const _velvetTheme = {
  'schemaVersion': 1,
  'name': 'Velvet',
  'colors': {
    'app': '#24202d',
    'panel': '#332b40',
    'raised': '#453954',
    'hover': '#554564',
    'line': '#17131f',
    'border': '#847294',
    'ink': '#f5ebff',
    'muted': '#c8b9d4',
    'tab': '#dc9bea',
    'tabInk': '#25122e',
    'accent': '#80e2d0',
    'caret': '#80e2d0',
    'selection': '#80e2d066',
    'menu': '#24202d',
    'menuEdge': '#847294',
  },
  'drawing': {
    'laneGround': '#292332',
    'lane': '#2d2637',
    'laneAlt': '#332c40',
    'grid': '#17131f',
    'gridMinor': '#241e2c',
  },
};

Map<String, dynamic> _number(String id, String label, Object value) => {
  'id': id,
  'label': label,
  'kind': 'number',
  'value': value,
  'keys': const [],
  'keyedNow': false,
  'min': null,
  'max': null,
};

Map<String, dynamic> _layer(
  int id,
  String kind,
  double x, {
  String? name,
  int duration = 180,
}) => {
  'id': id,
  'order': id,
  'name': name ?? '$kind $id',
  'kind': kind,
  'locked': false,
  'hidden': false,
  'solo': false,
  'frozen': false,
  'ghostable': true,
  'flatten': false,
  'clipToBelow': false,
  'environment': false,
  'blendMode': 'Normal',
  'projection': '2.5D',
  'start': 0,
  'sourceIn': 0,
  'duration': duration,
  'parent': null,
  'colors': const [],
  'contentKeys': const [],
  'x': x,
  'y': 244.0,
  'properties': [
    {
      'id': 'position',
      'label': 'Position',
      'kind': 'vec2',
      'value': [x, 244.0],
      'keys': const [],
      'keyedNow': false,
      'min': null,
      'max': null,
    },
    _number('position.z', 'Position Z', 0.0),
    {
      'id': 'scale',
      'label': 'Scale',
      'kind': 'vec2',
      'value': const [1.0, 1.0],
      'keys': const [],
      'keyedNow': false,
      'min': null,
      'max': null,
    },
    _number('scale.z', 'Scale Z', 1.0),
    _number('rotation', 'Rotation', 0.0),
    _number('rotation.x', 'Rotation X', 0.0),
    _number('rotation.y', 'Rotation Y', 0.0),
    _number('opacity', 'Opacity', 1.0),
    if (kind == 'Text') ...[
      _number('text_style.0.size', 'Size', 64.0),
      _number('text_style.0.line_height', 'Line height', 72.0),
      _number('text_style.0.tracking', 'Tracking', 0.0),
    ],
  ],
  'effects': [
    {
      'id': 'motolii.blur',
      'name': 'Blur',
      'enabled': true,
      'params': [_number('amount', 'Amount', 2.0)],
    },
  ],
};

Map<String, dynamic> _status(String story) {
  final dense = story == 'dense';
  final text = story == 'text';
  final layers = <Map<String, dynamic>>[
    _layer(
      1,
      text ? 'Text' : 'Rectangle',
      320,
      name: text ? 'Title' : 'Card',
      duration: dense ? 900 : 180,
    ),
    for (var i = 2; i <= (dense ? 36 : 5); i++)
      _layer(
        i,
        i % 5 == 0 ? 'Text' : 'Rectangle',
        320 + i * 12,
        name: i % 5 == 0 ? 'Label $i' : 'Layer $i',
        duration: dense ? 900 : 180,
      ),
  ];
  return {
    'path': '/preview/Motolii Gallery.motolii',
    'layers': layers,
    'selectedId': 1,
    'selectedIds': const [1],
    'selectedKeys': const [],
    'assets': const [
      {
        'id': 'a0',
        'name': 'city-night.mp4',
        'mime': 'video/mp4',
        'path': '/preview/city-night.mp4',
      },
      {
        'id': 'a1',
        'name': 'portrait.png',
        'mime': 'image/png',
        'path': '/preview/portrait.png',
      },
    ],
    'background': const [0.1, 0.1, 0.1, 1.0],
    'backgrounds': const [],
    'catalog': const [
      {'id': 'motolii.isf_bloom', 'name': 'Bloom'},
      {'id': 'motolii.blur', 'name': 'Blur'},
      {'id': 'motolii.glow', 'name': 'Glow'},
      {'id': 'motolii.noise', 'name': 'Noise'},
      {'id': 'motolii.shadow', 'name': 'Shadow'},
      {'id': 'motolii.warp', 'name': 'Warp'},
    ],
    'palette': const [
      {'id': 'p0', 'hex': '#93a5f5'},
      {'id': 'p1', 'hex': '#eedb73'},
      {'id': 'p2', 'hex': '#c18bd3'},
      {'id': 'p3', 'hex': '#79c4ca'},
    ],
    'easeKinds': const ['Linear', 'Hold', 'Ease', 'EaseIn', 'EaseOut'],
    'fontFamilies': const ['Arial', 'Georgia', 'Hiragino Sans'],
    'importExtensions': const ['png', 'jpg', 'mp4', 'obj'],
    'history': const {
      'head': 2,
      'entries': [
        {'head': 0, 'kind': 'open', 'label': 'Open'},
        {'head': 1, 'kind': 'edit', 'label': 'Move Card'},
        {'head': 2, 'kind': 'edit', 'label': 'Blur'},
      ],
    },
    'capabilities': const [
      'create',
      'setFont',
      'preview',
      'animate',
      'placeAsset',
      'previewProperties',
      'commitPreview',
      'cancelPreview',
      'removeAsset',
      'import',
      'setAttrs',
      'previewBlend',
      'focusColor',
      'toggleKey',
      'historyGoto',
    ],
    'durationFrames': dense ? 900 : 180,
    'fps': 30.0,
    'fpsNum': 30,
    'fpsDen': 1,
    'width': 1600,
    'height': 1000,
    'frame': dense ? 310 : 48,
    'stageView': 'free',
    'contentRevision': story,
    'documentRevision': story,
    'snapshotId': 1,
    'referenceId': 1,
  };
}
