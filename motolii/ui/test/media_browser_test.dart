// The Browser keeps the old Media shelf's operations: pick one, pick several (Cmd / Shift), select all, walk with the arrows,
// Enter places, Delete removes what the caller lets it, Escape lets go, and a right click offers the caller's menu.
import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:motolii_ui/browser/item.dart';
import 'package:motolii_ui/browser/media/media_browser.dart';

class _Source extends ChangeNotifier implements ResultSource {
  _Source(this.items);
  @override
  final List<BrowserItem> items;
}

BrowserItem item(int i) => BrowserItem(id: 'a$i', name: 'asset$i', path: '/nowhere/asset$i.png', kind: 'image', mime: 'image/png', width: 400, height: 300, size: 1000);

void main() {
  final source = _Source([for (var i = 0; i < 6; i++) item(i)]);
  final placed = <String>[];
  final removed = <List<String>>[];

  Widget browser({BrowserView view = BrowserView.list}) => Directionality(
        textDirection: TextDirection.ltr,
        child: Overlay(initialEntries: [OverlayEntry(builder: (_) => Align(
          alignment: Alignment.topLeft,
          child: SizedBox(
            width: 320,
            height: 520,
            child: MediaBrowser(
              key: ValueKey(view),
              source: source,
              initial: view,
              onPlace: (i) => placed.add(i.id),
              onRemove: (p) => removed.add([for (final i in p) i.id]),
              menuOf: (i, p) => [(value: 'place', label: 'Place', enabled: true)],
              onMenu: (a, i, p) {},
            ),
          ),
        ))]),
      );

  MediaBrowserState state(WidgetTester t) => t.state<MediaBrowserState>(find.byType(MediaBrowser));

  Future<void> tapFace(WidgetTester t, int i) async {
    await t.tap(find.byKey(ValueKey('face-a$i')));
    await t.pump(const Duration(milliseconds: 400)); // past the double-tap window: separate clicks
  }

  setUp(() {
    placed.clear();
    removed.clear();
  });

  testWidgets('a click picks one; Cmd toggles; Shift takes the run from the anchor', (t) async {
    await t.pumpWidget(browser());
    await t.pumpAndSettle();
    await tapFace(t, 1);
    expect(state(t).picked, {'a1'});
    await t.sendKeyDownEvent(LogicalKeyboardKey.metaLeft, platform: 'macos');
    await tapFace(t, 3);
    await t.sendKeyUpEvent(LogicalKeyboardKey.metaLeft, platform: 'macos');
    expect(state(t).picked, {'a1', 'a3'});
    await t.sendKeyDownEvent(LogicalKeyboardKey.metaLeft, platform: 'macos');
    await tapFace(t, 1);
    await t.sendKeyUpEvent(LogicalKeyboardKey.metaLeft, platform: 'macos');
    expect(state(t).picked, {'a3'}, reason: 'Cmd on a picked one lets it go');
    await tapFace(t, 2);
    await t.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft, platform: 'macos');
    await tapFace(t, 5);
    await t.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft, platform: 'macos');
    expect(state(t).picked, {'a2', 'a3', 'a4', 'a5'});
  });

  testWidgets('Cmd-A picks all, Escape lets go, Enter places the chosen, Delete hands over what is picked', (t) async {
    await t.pumpWidget(browser());
    await t.pumpAndSettle();
    await tapFace(t, 0);
    await t.sendKeyDownEvent(LogicalKeyboardKey.metaLeft, platform: 'macos');
    await t.sendKeyEvent(LogicalKeyboardKey.keyA);
    await t.sendKeyUpEvent(LogicalKeyboardKey.metaLeft, platform: 'macos');
    expect(state(t).picked.length, 6);
    await t.sendKeyEvent(LogicalKeyboardKey.delete);
    expect(removed.single.length, 6);
    await t.sendKeyEvent(LogicalKeyboardKey.escape);
    expect(state(t).picked, isEmpty);
    await tapFace(t, 2);
    await t.sendKeyEvent(LogicalKeyboardKey.enter);
    expect(placed, ['a2']);
  });

  testWidgets('the arrows walk the list; Home and End', (t) async {
    await t.pumpWidget(browser());
    await t.pumpAndSettle();
    await tapFace(t, 1);
    await t.sendKeyEvent(LogicalKeyboardKey.arrowDown);
    expect(state(t).selected, 'a2');
    await t.sendKeyEvent(LogicalKeyboardKey.end);
    expect(state(t).selected, 'a5');
    await t.sendKeyEvent(LogicalKeyboardKey.home);
    expect(state(t).selected, 'a0');
  });

  testWidgets('in Thumbnail the arrows go by where the faces are', (t) async {
    await t.pumpWidget(browser(view: BrowserView.thumbnail));
    await t.pumpAndSettle();
    await tapFace(t, 0);
    await t.sendKeyEvent(LogicalKeyboardKey.arrowRight);
    expect(state(t).selected, 'a1');
    await t.sendKeyEvent(LogicalKeyboardKey.arrowDown);
    expect(state(t).selected, isNot('a1'), reason: 'down goes to a face below');
    expect(state(t).picked.length, 1);
  });

  testWidgets('a right click on something not picked picks it alone and offers the caller\'s menu', (t) async {
    await t.pumpWidget(browser());
    await t.pumpAndSettle();
    await tapFace(t, 0);
    await t.tap(find.byKey(const ValueKey('face-a3')), buttons: kSecondaryButton);
    await t.pump(const Duration(milliseconds: 50));
    expect(state(t).picked, {'a3'});
    expect(find.text('Place'), findsOneWidget);
    expect(find.text('asset3'), findsWidgets, reason: 'the menu starts with the name');
  });
}
