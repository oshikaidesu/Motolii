// A production Inspector on a recording session, for the Instrument tests.
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import '../../lib/live_hf/adapters/effect.dart';
import '../../lib/live_hf/adapters/layout.dart';
import '../../lib/live_hf/adapters/transform.dart';
import '../../lib/panels/inspector.dart';
import '../../lib/session/editor_session.dart';
import 'editor_test_theme.dart';
import 'window_fixture.dart';

class Recording extends EditorSession {
  final commands = <(String, Map<String, dynamic>)>[];
  List<String> get ops => [for (final c in commands) c.$1];
  @override
  Future<void> refreshPreview() async {}
  @override
  Future<void> command(String op, [Map<String, dynamic> args = const {}]) async {
    commands.add((op, args));
  }
}

const capabilities = ['previewProperties', 'commitPreview', 'toggleKey', 'anchor', 'setAttrs', 'animate', 'focusColor', 'enableEffect', 'moveEffect', 'removeEffect'];

Map<String, dynamic> layerRow(int id, String name, {List<double>? position, bool locked = false, String projection = '2D', int? parent, List<Map<String, dynamic>>? extra, bool keyed = false}) {
  final l = windowLayer(id, 'Shape', 0);
  return {
    ...l,
    'name': name,
    'locked': locked,
    'projection': projection,
    'parent': parent,
    'anchorFraction': [.5, .5],
    'effects': [],
    'properties': [
      for (final p in l['properties'] as List)
        if ((p as Map)['id'] == 'position')
          <String, dynamic>{...p, 'value': position ?? [320.0, 180.0], if (keyed) 'keys': [{'frame': 0}, {'frame': 30}], if (keyed) 'keyedNow': true}
        else
          p,
      ...?extra,
    ],
  };
}

Map<String, dynamic> state(List<Map<String, dynamic>> layers, List<int> selected) => {
  ...windowStatus(1, 0),
  'layers': layers,
  'selectedId': selected.first,
  'selectedIds': selected,
  'capabilities': capabilities,
  'animate': false,
};

Future<Recording> mount(WidgetTester tester, Map<String, dynamic> doc, {double height = 960}) async {
  tester.view.physicalSize = const Size(700, 1000);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  final c = Recording()..document.value = doc;
  await tester.pumpWidget(MaterialApp(
    theme: editorTestTheme,
    home: Align(
      alignment: Alignment.topLeft,
      child: SizedBox(
        width: 340,
        height: height,
        child: InspectorPanel(
          controller: c,
          instruments: InspectorInstruments(
            transform: (context, controller) => NewTransform(controller: controller),
            layout: (context, controller, layer) => NewLayout(controller: controller, layer: layer),
            effectParams: (context, controller, layerId, effect) => NewEffectParams(
              key: ValueKey('new-effect:$layerId:${effect['id']}'),
              controller: controller,
              layerId: layerId,
              effectId: effect['id'] as Object,
            ),
          ),
        ),
      ),
    ),
  ));
  await tester.pump();
  return c;
}


Future<void> loadFonts() async {
  final inter = FontLoader('Inter')
    ..addFont(rootBundle.load('assets/fonts/Inter-Regular.ttf'))
    ..addFont(rootBundle.load('assets/fonts/Inter-Medium.ttf'))
    ..addFont(rootBundle.load('assets/fonts/Inter-SemiBold.ttf'));
  await inter.load();
}

Finder key(String k) => find.byKey(ValueKey(k));
