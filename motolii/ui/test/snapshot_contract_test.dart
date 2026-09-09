import 'dart:convert';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import '../lib/session/editor_session.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  testWidgets('selection delta keeps references and updates selection without rendering', (tester) async {
    final calls = <String>[];
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(EditorSession.channel, (call) async {
      calls.add(call.method);
      if (call.method == 'request') {
        final args = jsonDecode(call.arguments['command']);
        expect(call.arguments['knownSnapshotId'], 7);
        expect(args.containsKey('knownSnapshotId'), isFalse);
        expect(call.arguments['knownReferenceId'], 4);
        return {'needsRender': false, 'snapshotId': 7, 'referenceId': 4, 'contentRevision': 'a', 'frame': 0, 'selectedIds': [2], 'selectedId': 2};
      }
      return <String,dynamic>{};
    });
    final c = EditorSession();
    c.document.value = {'snapshotId':7,'referenceId':4,'contentRevision':'a','backgrounds':['kept'],'layers':[{'id':1},{'id':2}],'selectedIds':[1]};
    c.rendered.value = {'contentRevision':'a','frame':0,'selectedIds':[1]};
    await c.command('select', {'ids':[2]});
    expect(calls, ['request']);
    expect(c.state['backgrounds'], ['kept']);
    expect(c.layers.length, 2);
    expect(c.selectedIds, [2]);
    expect(c.rendered.value['selectedIds'], [2]);
    expect(c.renderedIsFresh, isTrue);
    c.dispose();
  });

  testWidgets('pixel change defers the snapshot until render and retains common references', (tester) async {
    final calls = <String>[];
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(EditorSession.channel, (call) async {
      calls.add(call.method);
      if (call.method == 'request') return {'needsRender':true};
      if (call.method == 'render') return {'frameReady':true,'status':{'snapshotId':8,'referenceId':4,'contentRevision':'b','frame':0,'layers':[{'id':2,'x':12.0}],'selectedIds':[2]}};
      return <String,dynamic>{};
    });
    final c = EditorSession();
    c.document.value = {'snapshotId':7,'referenceId':4,'contentRevision':'a','fontFamilies':['kept'],'layers':[{'id':2,'x':0.0}]};
    await c.command('setProperty', {'layer':2,'property':'position_x','value':12.0});
    expect(calls, ['request','render']);
    expect(c.state['fontFamilies'], ['kept']);
    expect(c.state['snapshotId'], 8);
    expect(c.layers.single['x'], 12.0);
    expect(c.renderedIsFresh, isTrue);
    c.document.value = {...c.state,'contentRevision':'new-preview'};
    expect(c.renderedIsFresh, isFalse);
    c.dispose();
  });
}
