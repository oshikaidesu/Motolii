// An effect's parameters as generic Toys inside the production Inspector's effect card, on the real session.
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/app/new/inspector/session_effect.dart';
import '../lib/hf/insp/panel.dart';
import 'support/new_inspector_host.dart';
import 'support/window_fixture.dart';

Map<String, dynamic> param(String id, String label, Object v, {double? min, double? max, double? def, List<String>? choices, String kind = 'number', bool keyed = false}) => {
  ...windowNumber(id, label, v),
  'kind': kind,
  'min': min,
  'max': max,
  'default': def ?? (v is num ? v : null),
  if (choices != null) 'choices': choices,
  if (keyed) 'keys': [{'frame': 0}, {'frame': 30}],
  if (keyed) 'keyedNow': true,
};

Map<String, dynamic> withEffect(Map<String, dynamic> layer, {bool placement = false, bool locked = false}) => {
  ...layer,
  'locked': locked,
  'effects': [
    {
      'id': 'motolii.blur',
      'name': 'Blur',
      'enabled': true,
      if (placement) 'placement': true,
      if (placement) 'layout': {'rows': []},
      'params': [
        param('effect.0.param.amount', 'Amount', 2.0, min: 0, max: 10, def: 2.0),
        param('effect.0.param.size', 'Size', 40.0),
        param('effect.0.param.mode', 'Mode', 1, choices: ['Box', 'Gaussian']),
        param('effect.0.param.tint', 'Tint', [1.0, 0.5, 0.0, 1.0], kind: 'color'),
        param('effect.0.param.seed', 'Seed', 7),
        param('effect.0.param.angle', 'Angle', 10.0, keyed: true),
      ],
    },
  ],
};

SessionEffectStore storeOf(WidgetTester tester) => tester.widget<ParamSheet>(find.byType(ParamSheet)).store as SessionEffectStore;

void main() {
  setUpAll(loadFonts);

  testWidgets('an effect card is the head Classic keeps and the generic Toys for its declared rows', (tester) async {
    await mount(tester, state([withEffect(layerRow(1, 'A'))], [1]), height: 1600);
    expect(find.byType(ParamSheet), findsOneWidget);
    for (final id in ['amount', 'size', 'mode', 'tint', 'seed', 'angle']) {
      expect(key('cell-effect.0.param.$id'), findsOneWidget, reason: id);
    }
    expect(tester.takeException(), isNull);
  });

  testWidgets('an effect that lays out copies keeps the Inspector body until an Instrument owns its grid', (tester) async {
    await mount(tester, state([withEffect(layerRow(1, 'A'), placement: true)], [1]), height: 1600);
    expect(find.byType(ParamSheet), findsNothing);
  });

  testWidgets('a typed number is one preview and one commit, on the row id the Classic Inspector uses', (tester) async {
    final c = await mount(tester, state([withEffect(layerRow(1, 'A'))], [1]), height: 1600);
    final toy = key('toy-effect.0.param.size');
    await tester.ensureVisible(toy);
    await tester.tap(toy);
    await tester.pump();
    await tester.enterText(find.descendant(of: toy, matching: find.byType(EditableText)), '80');
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pump(const Duration(seconds: 1));
    final preview = c.commands.where((v) => v.$1 == 'previewProperties').last;
    expect((preview.$2['edits'] as List).single, {'layer': 1, 'property': 'effect.0.param.size', 'value': 80.0});
    expect(c.ops.where((o) => o == 'commitPreview').length, 1);
  });

  testWidgets('a drag is relative across the selection; a typed number is absolute; a locked layer is skipped', (tester) async {
    final a = withEffect(layerRow(1, 'A'));
    final b = withEffect(layerRow(2, 'B'));
    ((b['effects'] as List).single['params'] as List)[1] = param('effect.0.param.size', 'Size', 100.0);
    final locked = withEffect(layerRow(3, 'Locked'), locked: true);
    // The Inspector shows one effect sheet only for a single layer, so the store is built the way the sheet builds it.
    final c = await mount(tester, state([a, b, locked], [3, 2, 1]), height: 1600);
    final s = SessionEffectStore(c, 1, 'motolii.blur');
    addTearDown(s.dispose);
    s.preview('effect.0.param.size', 50.0); // A: 40 -> 50 (+10)
    var edits = (c.commands.last.$2['edits'] as List).cast<Map>();
    expect(edits.map((e) => e['layer']), [1, 2], reason: 'the locked layer is not a target');
    expect(edits.firstWhere((e) => e['layer'] == 2)['value'], 110.0, reason: 'B keeps its own offset');
    s.commit('effect.0.param.size');
    c.commands.clear();
    s.set('effect.0.param.size', 25.0);
    edits = (c.commands.first.$2['edits'] as List).cast<Map>();
    expect(edits.firstWhere((e) => e['layer'] == 2)['value'], 25.0, reason: 'a typed number is absolute for every target');
  });

  testWidgets('a locked layer refuses every edit', (tester) async {
    final c = await mount(tester, state([withEffect(layerRow(1, 'Locked'), locked: true)], [1]), height: 1600);
    final s = storeOf(tester);
    s.set('effect.0.param.size', 9.0);
    s.toggleKey('effect.0.param.size');
    expect(c.commands, isEmpty);
  });

  testWidgets('the diamond keys the value at this frame; an animated one shows as animated', (tester) async {
    final c = await mount(tester, state([withEffect(layerRow(1, 'A'))], [1]), height: 1600);
    final diamond = key('key-effect.0.param.size');
    await tester.ensureVisible(diamond);
    await tester.tap(diamond);
    await tester.pump();
    expect(c.commands.last.$1, 'toggleKey');
    expect(c.commands.last.$2, {'layer': 1, 'property': 'effect.0.param.size'});
    expect(storeOf(tester).row('effect.0.param.angle')['animated'], isTrue);
    expect(storeOf(tester).row('effect.0.param.size')['animated'], isFalse);
  });

  testWidgets('a colour hands over to the Colors wheel aimed at that property', (tester) async {
    final c = await mount(tester, state([withEffect(layerRow(1, 'A'))], [1]), height: 1600);
    storeOf(tester).route('Colors', 'effect.0.param.tint');
    expect(c.commands.any((v) => v.$1 == 'focusColor' && v.$2['property'] == 'effect.0.param.tint' && v.$2['layer'] == 1), isTrue);
  });

  testWidgets('undo or another edit shows in the sheet; a running gesture keeps its own value', (tester) async {
    final c = await mount(tester, state([withEffect(layerRow(1, 'A'))], [1]), height: 1600);
    final s = storeOf(tester);
    Map<String, dynamic> withSize(double v) {
      final l = withEffect(layerRow(1, 'A'));
      ((l['effects'] as List).single['params'] as List)[1] = param('effect.0.param.size', 'Size', v);
      return l;
    }

    c.document.value = state([withSize(60)], [1]);
    await tester.pump();
    expect(s.row('effect.0.param.size')['value'], 60);
    s.preview('effect.0.param.size', 70.0);
    c.document.value = state([withSize(65)], [1]);
    await tester.pump();
    expect(s.row('effect.0.param.size')['value'], 70, reason: 'the pointer is in charge until it lets go');
    s.commit('effect.0.param.size');
  });

  testWidgets('Esc during a drag of an effect value puts it back; the document hears cancelPreview, not a commit', (tester) async {
    final c = await mount(tester, state([withEffect(layerRow(1, 'A'))], [1]), height: 1600);
    final toy = key('toy-effect.0.param.size');
    await tester.ensureVisible(toy);
    final drag = await tester.startGesture(tester.getCenter(toy));
    await drag.moveBy(const Offset(60, 0));
    await tester.pump();
    expect(storeOf(tester).row('effect.0.param.size')['value'], isNot(40.0));
    await tester.sendKeyEvent(LogicalKeyboardKey.escape);
    await tester.pump();
    expect(storeOf(tester).row('effect.0.param.size')['value'], 40.0);
    await drag.up();
    await tester.pump();
    expect(c.ops.last, 'cancelPreview');
    expect(c.ops, isNot(contains('commitPreview')));
  });
}
