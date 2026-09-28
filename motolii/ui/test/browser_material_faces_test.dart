import 'dart:convert';

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/hf/bp/common.dart' show sans;
import '../lib/hf/bp/catalog_io.dart';
import '../lib/hf/bp/classify.dart' show ClassStrip;
import '../lib/hf/bp/create.dart';
import '../lib/hf/bp/create_board.dart';
import '../lib/hf/bp/shell.dart' show PanelHeader;
import '../lib/hf/bp/faces.dart';
import '../lib/hf/bp/shelf_sections.dart';
import '../lib/hf/bp/things.dart';
import '../lib/live_hf/adapters/browser_shelf.dart';
import '../lib/live_hf/adapters/browser_user.dart';
import '../lib/session/editor_session.dart';

/// Create is a toybox (the object is the face, the name a caption) and Media is material (each family drawn as what it
/// is). The shelf's mechanics — picking, keys, placing — are unchanged underneath.
/// A Create key by its name: the board prints no caption; the name is the key's semantics label.
Finder mark(String name) => find.byWidgetPredicate((w) => w is Semantics && w.properties.label == name);

void main() {
  final sent = <Map<String, dynamic>>[];
  setUp(() {
    sent.clear();
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(EditorSession.channel, (call) async {
      if (call.method == 'request') {
        sent.add(jsonDecode('${EditorSession.map(call.arguments)['command']}') as Map<String, dynamic>);
        return {'ok': true, 'needsRender': false};
      }
      if (call.method == 'readSettings') return <String, dynamic>{'deskWork': {}};
      return <String, dynamic>{'ok': true};
    });
  });

  Widget host(WidgetTester t, Widget child, double w, double h) {
    t.view.physicalSize = Size(w, h);
    t.view.devicePixelRatio = 1;
    addTearDown(t.view.reset);
    return Directionality(
        textDirection: TextDirection.ltr,
        child: DefaultTextStyle(style: sans(12), child: Overlay(initialEntries: [OverlayEntry(builder: (_) => Align(alignment: Alignment.topLeft, child: SizedBox(width: w, height: h, child: child)))])),
    );
  }

  testWidgets('Media shows each family as itself: stills with their size, clips with motion and length, sounds as their waveform', (t) async {
    final c = EditorSession()
      ..document.value = {
        'capabilities': ['placeAsset'],
        'assets': [
          {'id': 1, 'name': 'Dusk', 'mime': 'image/png', 'facts': {'width': 1600, 'height': 900}},
          {'id': 2, 'name': 'Zoom', 'mime': 'video/mp4', 'facts': {'width': 1280, 'height': 720, 'fps': 30.0, 'seconds': 6.0}},
          {'id': 3, 'name': 'Pulse', 'mime': 'audio/wav', 'facts': {'sampleRate': 48000, 'channels': 2, 'seconds': 8.0}, 'peaks': [for (var i = 0; i < 96; i++) [-0.5, 0.5]]},
        ],
      };
    final user = LiveBrowserUser(c, 'Media');
    await t.pumpWidget(host(t, LiveBrowserShelf(controller: c, name: 'Media', user: user), 520, 900));
    await t.pump();
    // sections read at a glance, with how many each holds
    expect(find.byType(ShelfHeading), findsNWidgets(3));
    for (final name in ['Images', 'Video', 'Audio']) {
      expect(find.text(name), findsWidgets, reason: name);
    }
    // a contact sheet: names only; lengths sit on the faces of clips and sounds; size and rate are not always on show
    for (final name in ['Dusk', 'Zoom', 'Pulse']) {
      expect(find.text(name), findsOneWidget, reason: name);
    }
    expect(find.text('0:06'), findsOneWidget);
    expect(find.text('0:08'), findsOneWidget);
    expect(find.text('1600×900'), findsNothing);
    // the classes run along the top, so the sheet keeps the seat's width
    expect(find.byType(ClassStrip), findsOneWidget);
    expect(find.byWidgetPredicate((w) => w is CustomPaint && w.painter.runtimeType.toString() == '_Wave'), findsOneWidget);

    // keys walk the tiles in the order they are drawn: Images, then Video, then Audio
    await t.tap(find.text('Dusk'));
    await t.pump(const Duration(milliseconds: 400));
    await t.sendKeyEvent(LogicalKeyboardKey.arrowRight);
    await t.pump();
    await t.sendKeyEvent(LogicalKeyboardKey.enter);
    await t.pump();
    expect(sent.lastWhere((m) => m['op'] == 'placeAsset')['id'], '2', reason: 'right of the only still is the first clip');
    await t.pumpWidget(const SizedBox());
    user.dispose();
    c.dispose();
  });

  testWidgets('Create: a Swiss symbol matrix — identifiers, uncaptioned marks, one header row, a Recent row', (t) async {
    final catalog = loadCatalog('lib/hf/data/things');
    final user = UserViews(recent: ['motolii.sphere']);
    await t.pumpWidget(host(t, CreatePanel(catalog: catalog, user: user), 420, 3000));
    await t.pump();
    // a symbol matrix: section identifiers, keys without captions (the name is the key's label), marks of 24-30 px
    expect(find.text('PRIMITIVES'), findsOneWidget);
    expect(find.text('RECENT'), findsOneWidget, reason: 'what was taken last is the last row');
    expect(find.text('Sphere'), findsNothing, reason: 'no caption under every mark');
    for (final name in ['Text', 'Cube', 'Camera', 'Stage', 'Sphere']) {
      expect(mark(name), findsWidgets, reason: name);
    }
    final face = t.getSize(find.byType(ThingFace).first);
    expect(face.width, inInclusiveRange(24, 30));
    // one compact header row: the classes share it with search and the menu
    expect(find.byType(ClassStrip), findsOneWidget);
    expect(t.getSize(find.ancestor(of: find.byType(ClassStrip), matching: find.byType(PanelHeader))).height, lessThanOrEqualTo(32));
    // at the default seat width a row holds at least six marks
    expect(SymbolMatrix.columns(324 - 0), greaterThanOrEqualTo(6));
  });
}
