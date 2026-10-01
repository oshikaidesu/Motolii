import 'dart:ui' show Size;

Map<String, dynamic> windowNumber(String id, String label, Object v) => {
  'id': id,
  'label': label,
  'kind': 'number',
  'value': v,
  'keys': const [],
  'keyedNow': false,
  'min': null,
  'max': null,
};

/// A layer as the native side sends it: thirteen animatable rows, the flags
/// the Timeline reads, and one effect.
Map<String, dynamic> windowLayer(int id, String kind, double x) => {
  'id': id,
  'order': id,
  'name': '$kind $id',
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
  'duration': 180,
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
    windowNumber('position.z', 'Position Z', 0.0),
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
    windowNumber('scale.z', 'Scale Z', 1.0),
    windowNumber('rotation', 'Rotation', 0.0),
    windowNumber('rotation.x', 'Rotation X', 0.0),
    windowNumber('rotation.y', 'Rotation Y', 0.0),
    windowNumber('opacity', 'Opacity', 1.0),
    windowNumber('text_style.0.size', 'Size', 14.0),
    windowNumber('text_style.0.line_height', 'Line height', 144.0),
    windowNumber('text_style.0.tracking', 'Tracking', 0.0),
  ],
  'effects': [
    {
      'id': 'motolii.blur',
      'name': 'Blur',
      'params': [windowNumber('amount', 'Amount', 2.0)],
    },
  ],
};

/// The two documents of the report: the small one the window is judged on and
/// a fifteen-layer one that says how the cost grows.
Map<String, dynamic> windowStatus(int layers, double x) => {
  'layers': [
    for (var i = 0; i < layers; i++)
      windowLayer(i + 1, i == 0 ? 'Cube' : 'Rectangle', i == 0 ? x : i * 3.0),
  ],
  'selectedId': 1,
  'selectedIds': const [1],
  'selectedKeys': const [],
  'assets': [
    for (var i = 0; i < 6; i++)
      {'id': 'a$i', 'name': 'clip$i.mp4', 'mime': 'video/mp4'},
  ],
  'background': const [0.1, 0.1, 0.1, 1.0],
  'backgrounds': const [],
  'catalog': const [
    {'id': 'motolii.isf_bloom', 'name': 'Bloom'},
    {'id': 'motolii.blur', 'name': 'Blur'},
    {'id': 'motolii.clip', 'name': 'Clip'},
    {'id': 'motolii.colorize', 'name': 'Colorize'},
    {'id': 'motolii.echo', 'name': 'Echo'},
    {'id': 'motolii.glow', 'name': 'Glow'},
    {'id': 'motolii.levels', 'name': 'Levels'},
    {'id': 'motolii.mirror', 'name': 'Mirror'},
    {'id': 'motolii.noise', 'name': 'Noise'},
    {'id': 'motolii.shadow', 'name': 'Shadow'},
    {'id': 'motolii.warp', 'name': 'Warp'},
  ],
  'palette': [
    for (var i = 0; i < 8; i++) {'id': 'p$i', 'hex': '#10101$i'},
  ],
  'easeKinds': const ['Linear', 'Hold', 'Ease', 'EaseIn', 'EaseOut'],
  'fontFamilies': const ['Arial', 'Georgia', 'Hiragino Sans'],
  'importExtensions': const ['png', 'jpg', 'mp4', 'obj'],
  'history': const {
    'head': 0,
    'entries': [
      {'head': 0, 'kind': 'edit', 'label': 'Open', 'detail': ''},
    ],
  },
  'capabilities': const [
    'setFont',
    'preview',
    'animate',
    'placeAsset',
    'previewProperties',
    'commitPreview',
    'cancelPreview',
    'removeAsset',
    'import',
  ],
  'durationFrames': 180,
  'fps': 30.0,
  'width': 1600,
  'height': 1000,
  'frame': 0,
  'stageView': 'free',
  'contentRevision': '$x',
  'documentRevision': '$x',
  'snapshotId': 1,
  'referenceId': 1,
};

/// Panel, and the pixels the default dock gives it inside a 1280x796 window.
const dockPanels = <String, Size>{
  'Create': Size(260, 537),
  'Media': Size(260, 537),
  'Effects': Size(260, 537),
  'Colors': Size(260, 537),
  'Fonts': Size(260, 537),
  'Stage': Size(712, 537),
  'Inspector': Size(300, 333),
  'Notes': Size(712, 537),
  'Desk': Size(300, 200),
  'Ease': Size(300, 200),
  'Depth': Size(300, 200),
  'Blend': Size(300, 200),
  'History': Size(300, 200),
  'Timeline': Size(1280, 255),
};
