// The New shell's workspace: what it saves and restores, and that the Stage follows the dock.
import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/app/new_shell.dart';
import '../lib/session/editor_session.dart';
import 'support/editor_test_theme.dart';

class Native {
  Map<String, dynamic> settings = {};
  final calls = <(String, Map)>[];
  List<Map> get writes => [for (final c in calls) if (c.$1 == 'writeSettings') c.$2];
  List<Map> get windows => [for (final c in calls) if (c.$1 == 'command' && c.$2['op'] == 'stageWindow') Map.from(c.$2['args'] as Map? ?? c.$2)];

  void install() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(EditorSession.channel, (call) async {
      final args = Map<String, dynamic>.from((call.arguments as Map?) ?? {});
      calls.add((call.method, args));
      switch (call.method) {
        case 'windowInfo':
          return {'id': 'main', 'main': true};
        case 'readSettings':
          return settings;
        case 'writeSettings':
          settings = args;
          return true;
        case 'attach':
        case 'render':
        case 'command':
          return {'layers': [], 'selectedIds': [], 'selectedKeys': [], 'capabilities': ['stageWindow'], 'width': 1920, 'height': 1080, 'durationFrames': 300, 'fps': 30};
        default:
          return <String, dynamic>{};
      }
    });
  }
}

Future<dynamic> open(WidgetTester tester) async {
  await tester.pumpWidget(MaterialApp(theme: editorTestTheme, home: const NewShell()));
  await tester.pumpAndSettle();
  return tester.state(find.byType(NewShell));
}

Future<void> close(WidgetTester tester) async {
  await tester.pumpWidget(const SizedBox());
  await tester.pumpAndSettle();
}

Future<void> drag(WidgetTester tester, Offset from, Offset to) async {
  final g = await tester.startGesture(from);
  await g.moveBy(const Offset(0, -24));
  await tester.pump(const Duration(milliseconds: 50));
  for (var k = 1; k <= 14; k++) {
    await g.moveTo(Offset.lerp(from, to, k / 14)!);
    await tester.pump(const Duration(milliseconds: 16));
  }
  await g.up();
  await tester.pumpAndSettle();
}

/// `tabbed_view` gives a tab that does not fit a minimal, off-screen slot (about 17 px wide) and its label row
/// reports an overflow in debug. Nothing is drawn there; every other error is still reported.
void ignoreSqueezedTabChips() {
  final report = FlutterError.onError;
  FlutterError.onError = (details) {
    final text = details.toString(minLevel: DiagnosticLevel.debug);
    final chip = text.contains('RenderFlex overflowed') && RegExp(r'constraints: BoxConstraints\(w=(\d+\.\d+), h=\d+\.\d+\)').allMatches(text).any((m) => double.parse(m.group(1)!) < 40);
    if (!chip) report?.call(details);
  };
}

void main() {
  void size(WidgetTester tester, [Size s = const Size(1600, 1000)]) {
    tester.view.physicalSize = s;
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
  }

  testWidgets('a layout change is written beside the keys already there, and read back', (tester) async {
    size(tester);
    final native = Native()..settings = {'deskWork': {'animateFrom': false}, 'dock': {'classic': 'layout'}};
    native.install();
    dynamic shell = await open(tester);
    expect(native.writes, isEmpty, reason: 'opening writes nothing');

    // A tab in front, a panel closed, and a divider dragged.
    await tester.tap(find.text('CAMERA'));
    await tester.pumpAndSettle();
    shell.dock.close('Notes');
    await tester.pumpAndSettle();
    final browser = shell.dock.rectOf('Create')!;
    await drag(tester, Offset(browser.right + 2, 400), Offset(browser.right + 122, 400));
    await tester.pump(const Duration(seconds: 1));
    final stageLeft = shell.dock.rectOf('Stage') ?? shell.dock.rectOf('Camera');
    print('WS writes=${native.writes.length}');
    expect(native.writes, isNotEmpty);
    final saved = native.settings;
    expect(saved['deskWork'], {'animateFrom': false}, reason: "Classic's keys are kept");
    expect(saved['dock'], {'classic': 'layout'});
    final ws = saved['newWorkspace'] as Map;
    expect(ws['version'], 1);
    expect((ws['shown'] as List), contains('Camera'));

    await close(tester);
    final again = native.writes.length;
    shell = await open(tester);
    expect(native.writes.length, again, reason: 'reading it back writes nothing');
    expect(shell.dock.isOpen('Notes'), isFalse);
    expect(shell.dock.isShown('Camera'), isTrue);
    final restored = shell.dock.rectOf('Camera')!;
    print('WS camera left before ${stageLeft?.left} after restart ${restored.left}');
    expect((restored.left - (stageLeft?.left ?? 0)).abs(), lessThan(3), reason: 'the dragged width came back');
    await close(tester);
  });

  testWidgets('a state this build cannot read leaves the default layout', (tester) async {
    size(tester);
    for (final bad in <Object?>[
      {'version': 99, 'layout': 'x', 'shown': []},
      {'version': 1, 'layout': 'V1:2:1(R;0;;;2,3),2(I;1;NoSuchPanel;0.5;F),3(I;1;Timeline;0.5;F)', 'shown': []},
      {'version': 1, 'layout': 'garbage', 'shown': []},
      'not a map',
    ]) {
      final native = Native()..settings = {'newWorkspace': bad};
      native.install();
      final shell = await open(tester);
      for (final id in ['Create', 'Stage', 'Inspector', 'Timeline', 'Desk']) {
        expect(shell.dock.isOpen(id), isTrue, reason: '$id with $bad');
      }
      await close(tester);
    }
  });

  testWidgets('every panel can be named back onto the face, and showing one never rearranges the others', (tester) async {
    size(tester);
    Native().install();
    ignoreSqueezedTabChips();
    final shell = await open(tester);
    final ids = (shell.dock.defs as Map<String, dynamic>).keys.toList();
    String structure() => (shell.dock.snapshot() as Map)['layout'] as String;
    // Showing a docked panel picks its tab and changes no split, size or membership.
    final before = structure();
    for (final id in ids) {
      await (shell.c as EditorSession).panelPlacementRequested!(id, 'show');
      await tester.pumpAndSettle();
      expect(shell.dock.isShown(id), isTrue, reason: id);
      expect(structure(), before, reason: 'showing $id rearranged the workspace');
    }
    // A closed one comes back, and the panels that stayed are where they were.
    for (final id in ['Inspector', 'Timeline', 'Desk', 'Fonts', 'Web']) {
      shell.dock.close(id);
      await tester.pumpAndSettle();
      expect(shell.dock.isOpen(id), isFalse, reason: id);
      await (shell.c as EditorSession).panelPlacementRequested!(id, 'show');
      await tester.pumpAndSettle();
      expect(shell.dock.isShown(id), isTrue, reason: '$id reopened in front');
    }
    expect(tester.takeException(), isNull);
    await close(tester);
  });

  testWidgets('Reset layout brings the default arrangement back, and the View menu names every panel', (tester) async {
    size(tester);
    final native = Native()..settings = {};
    native.install();
    final shell = await open(tester);
    String structure() => (shell.dock.snapshot() as Map)['layout'] as String;
    final fresh = structure();
    shell.dock.close('Timeline');
    shell.dock.close('Desk');
    await tester.pumpAndSettle();
    final browser = shell.dock.rectOf('Create')!;
    await drag(tester, Offset(browser.right + 2, 400), Offset(browser.right + 122, 400));
    expect(structure(), isNot(fresh));
    await shell.menu('Reset layout');
    await tester.pumpAndSettle();
    expect(structure(), fresh, reason: 'the default arrangement, sizes included');
    for (final id in ['Create', 'Stage', 'Inspector', 'Timeline', 'Desk']) {
      expect(shell.dock.isOpen(id), isTrue, reason: id);
    }
    await tester.pump(const Duration(seconds: 1));
    expect((native.settings['newWorkspace'] as Map)['layout'], fresh, reason: 'and it is what is saved');
    await close(tester);
  });

  testWidgets('the Stage asks native for the surface it is showing, and only that one', (tester) async {
    size(tester);
    final native = Native();
    native.install();
    final shell = await open(tester);
    var seen = 0;
    // What the session asked native since the last look: which view was attached, and the last Stage window size.
    ({List<String> attached, List<(int, int, String)> windows}) look() {
      final fresh = native.calls.skip(seen).toList();
      seen = native.calls.length;
      final attached = [for (final c in fresh) if (c.$1 == 'attach') '${c.$2['view']}'];
      final windows = <(int, int, String)>[];
      for (final c in fresh) {
        if (c.$1 != 'request' || c.$2['command'] is! String) continue;
        final op = jsonDecode(c.$2['command'] as String) as Map;
        if (op['op'] == 'stageWindow') windows.add((op['width'] as int, op['height'] as int, jsonEncode(op['roi'])));
      }
      return (attached: attached, windows: windows);
    }

    var now = look();
    expect(now.attached, contains('User'));
    expect(now.windows, isNotEmpty);
    var (w0, h0, roi0) = now.windows.last;
    expect(w0 > 0 && h0 > 0, isTrue, reason: 'the Stage on the face asks for a real window');

    // Another tab in front of the Stage: the Stage withdraws its window and the Camera takes its own view.
    await tester.tap(find.text('CAMERA'));
    await tester.pumpAndSettle();
    now = look();
    expect(now.attached, contains('Camera'));
    expect((now.windows.last.$1, now.windows.last.$2), (0, 0), reason: 'a hidden Stage hands its window back');

    // Back: it asks again, at the same size.
    await tester.tap(find.text('STAGE'));
    await tester.pumpAndSettle();
    now = look();
    expect(now.attached, contains('User'));
    expect(now.windows.last, (w0, h0, roi0));

    // The dock resizes it: it asks for the part of the picture it now shows.
    final browser = shell.dock.rectOf('Create')!;
    final wide = shell.dock.rectOf('Stage')!;
    await drag(tester, Offset(browser.right + 2, 400), Offset(browser.right + 122, 400));
    now = look();
    final narrow = shell.dock.rectOf('Stage')!;
    print('SURF resize: rect ${wide.size} -> ${narrow.size}; window $w0 x $h0 ${roi0.substring(0, 12)}.. -> ${now.windows.isEmpty ? 'no request' : now.windows.last}');
    expect(narrow.width, lessThan(wide.width - 60));
    expect(now.windows, isNotEmpty, reason: 'a resized Stage says so');
    expect(now.windows.last.$3, isNot(roi0), reason: 'and the region it shows changed');
    expect(now.windows.last.$1 > 0 && now.windows.last.$2 > 0, isTrue);
    roi0 = now.windows.last.$3;

    // The dock moves it above the Timeline: smaller and lower, still in front, still asking for its picture.
    final tl = shell.dock.rectOf('Timeline')!;
    await drag(tester, tester.getCenter(find.text('STAGE').first), Offset(tl.center.dx, tl.top + tl.height * .12));
    now = look();
    final after = shell.dock.rectOf('Stage')!;
    print('SURF move: rect ${narrow.size} -> ${after.size}; windows ${now.windows.length}, last ${now.windows.isEmpty ? '-' : '${now.windows.last.$1}x${now.windows.last.$2}'}; attached ${now.attached}');
    expect(after.height, lessThan(narrow.height));
    expect(shell.dock.isShown('Stage'), isTrue);
    expect(now.windows, isNotEmpty);
    expect(now.windows.last.$3, isNot(roi0), reason: 'the new shape changes what it shows');
    expect(now.windows.where((w) => w.$1 == 0 && w.$2 == 0), isEmpty, reason: 'no withdrawal while it is in front');
    expect(now.windows.last.$1 > 0 && now.windows.last.$2 > 0, isTrue);
    expect(tester.takeException(), isNull);
    await close(tester);
  });
}
