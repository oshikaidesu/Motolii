// The fluid board moves the same faces between projections: the selection and the identity stay, and a face is between its
// two places part-way through the change (motion, not a swap).

import 'package:flutter/gestures.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:motolii_ui/browser/item.dart';
import 'package:motolii_ui/browser/media/fluid.dart';
import 'package:motolii_ui/browser/media/preview.dart' show FaceService;

BrowserItem item(int i, String kind, {int? w, int? h}) => BrowserItem(id: 'a$i', name: 'asset$i', path: '/nowhere/asset$i', kind: kind, mime: 'x/y', width: w, height: h, size: 1000 * (i + 1));

void main() {
  final items = [item(0, 'image', w: 400, h: 300), item(1, 'video', w: 1280, h: 720), item(2, 'model'), item(3, 'image', w: 300, h: 500), item(4, 'audio')];

  Widget board(String view, {String? selected}) => Directionality(
        textDirection: TextDirection.ltr,
        child: Align(alignment: Alignment.topLeft, child: SizedBox(width: 320, height: 420, child: FluidBoard(items: items, selected: selected, view: view, onTap: (_) {}))),
      );

  Rect faceOf(WidgetTester t, String id) {
    final finder = find.byKey(ValueKey('face-$id'));
    final box = t.renderObject<RenderBox>(finder);
    return box.localToGlobal(Offset.zero) & box.size;
  }

  testWidgets('a face moves from where the list put it to where the thumbnail puts it', (t) async {
    await t.pumpWidget(board('list', selected: 'a3'));
    await t.pumpAndSettle();
    final inList = faceOf(t, 'a3');
    await t.pumpWidget(board('thumbnail', selected: 'a3'));
    await t.pump(const Duration(milliseconds: 16)); // the very next frame already shows it moving (ease-out, no wind-up)
    final first = faceOf(t, 'a3');
    await t.pump(const Duration(milliseconds: 60));
    final part = faceOf(t, 'a3');
    await t.pumpAndSettle();
    final inThumb = faceOf(t, 'a3');
    expect(inList, isNot(inThumb));
    expect(first, isNot(inList), reason: 'the first frame after the input already moved it: no delayed start');
    expect(part, isNot(inList), reason: 'it has left the list place');
    expect(part, isNot(inThumb), reason: 'and is not yet at the thumbnail place: it is moving');
    // and the layout it settles into is the projection's own
    final frame = FluidBoard.thumbnail(items, 320);
    final origin = t.getTopLeft(find.byType(FluidBoard));
    expect(inThumb.topLeft - origin, frame.faces['a3']!.topLeft);
  });

  testWidgets('an asset is one face across the change (never two, none lost from view), the chosen one still ringed', (t) async {
    await t.pumpWidget(board('list', selected: 'a2'));
    await t.pumpAndSettle();
    for (final view in ['thumbnail', 'list', 'thumbnail']) {
      await t.pumpWidget(board(view, selected: 'a2'));
      await t.pumpAndSettle();
      for (final i in items) {
        // the seat shows all five, so each is one face; what is out of sight would be built where it lands, when it comes into view
        expect(find.byKey(ValueKey('face-${i.id}')), findsOneWidget, reason: '${i.id} in $view');
      }
    }
  });

  test('list rows and thumbnail cards keep every asset and place none on another', () {
    for (final frame in [FluidBoard.list(items, 320), FluidBoard.thumbnail(items, 320)]) {
      expect(frame.faces.keys.toSet(), {for (final i in items) i.id});
      final rects = frame.faces.values.toList();
      for (var a = 0; a < rects.length; a++) {
        for (var b = a + 1; b < rects.length; b++) {
          expect(rects[a].overlaps(rects[b]), isFalse, reason: '$a and $b overlap');
        }
      }
    }
  });

  testWidgets('a pointer merely over a clip changes nothing but a hairline; pressing its scrub track asks for the frame', (t) async {
    final asked = <double>[];
    final clip = const BrowserItem(id: 'clip', name: 'clip.mp4', path: '/nowhere/clip.mp4', kind: 'video', mime: 'video/mp4', width: 1280, height: 720, seconds: 8);
    final faces = _Faces(asked);
    await t.pumpWidget(Directionality(
      textDirection: TextDirection.ltr,
      child: Align(alignment: Alignment.topLeft, child: SizedBox(width: 320, height: 420, child: FluidBoard(items: [clip], selected: null, view: 'thumbnail', onTap: (_) {}, faces: faces))),
    ));
    await t.pumpAndSettle();
    final face = t.getRect(find.byKey(const ValueKey('face-clip')));
    final mouse = await t.createGesture(kind: PointerDeviceKind.mouse);
    await mouse.addPointer(location: Offset.zero);
    await mouse.moveTo(Offset(face.left + face.width * .25, face.center.dy));
    await t.pump();
    await mouse.moveTo(Offset(face.left + face.width * .75, face.center.dy));
    await t.pump(const Duration(milliseconds: 50));
    expect(asked, isEmpty, reason: 'hover alone does not scrub');
    expect(t.getRect(find.byKey(const ValueKey('face-clip'))), face, reason: 'and does not move or resize the face');
    // pressed on the scrub track (the strip along its foot) and dragged: the frame under the pointer
    final foot = face.bottom - 4;
    await mouse.down(Offset(face.left + face.width * .25, foot));
    await mouse.moveTo(Offset(face.left + face.width * .75, foot));
    await t.pump();
    await t.pump(const Duration(milliseconds: 50));
    expect(asked, isNotEmpty);
    expect(asked.last, closeTo(6, .6), reason: 'three quarters across an 8 second clip');
    await mouse.up();
    await mouse.moveTo(const Offset(319, 419));
    await t.pump();
  });

  testWidgets('a still is never scaled or cropped by the pointer', (t) async {
    await t.pumpWidget(board('thumbnail'));
    await t.pumpAndSettle();
    final before = faceOf(t, 'a0');
    final mouse = await t.createGesture(kind: PointerDeviceKind.mouse);
    await mouse.addPointer(location: Offset.zero);
    await mouse.moveTo(before.center);
    await t.pump(const Duration(milliseconds: 200));
    expect(faceOf(t, 'a0'), before);
    expect(find.descendant(of: find.byKey(const ValueKey('face-a0')), matching: find.byType(Transform)), findsNothing, reason: 'nothing under the pointer is scaled');
    await mouse.moveTo(const Offset(319, 419));
    await t.pump();
  });
  group('a large library (what is on screen is built, the rest where it lands)', () {
    final many = [for (var i = 0; i < 1845; i++) item(i, i % 7 == 0 ? 'video' : 'image', w: 100 + (i % 5) * 40, h: 100 + (i % 3) * 30)];
    Widget tall(String view, {String? selected, ScrollController? controller}) => Directionality(
          textDirection: TextDirection.ltr,
          child: Align(alignment: Alignment.topLeft, child: SizedBox(width: 420, height: 640, child: FluidBoard(items: many, selected: selected, view: view, onTap: (_) {}))),
        );
    int built() => find.byWidgetPredicate((w) => w.key is ValueKey<String> && (w.key as ValueKey<String>).value.startsWith('face-')).evaluate().length;

    for (final view in ['thumbnail', 'list']) {
      testWidgets('$view: the first frame builds what is on screen, not the library', (t) async {
        await t.pumpWidget(tall(view, selected: 'a3'));
        await t.pump(const Duration(milliseconds: 16));
        expect(built(), greaterThan(0));
        expect(built(), lessThan(200), reason: '${many.length} assets, built $built');
      });
    }

    testWidgets('scrolling builds the assets that come into view, and they are the ones the layout put there', (t) async {
      await t.pumpWidget(tall('thumbnail', selected: 'a3'));
      await t.pump(const Duration(milliseconds: 16));
      expect(find.byKey(const ValueKey('face-a1500')), findsNothing, reason: 'far below: not built');
      final frame = FluidBoard.thumbnail(many, 420);
      await t.drag(find.byType(CustomScrollView), Offset(0, -(frame.faces['a1500']!.top - 100)));
      await t.pump(const Duration(milliseconds: 50));
      expect(find.byKey(const ValueKey('face-a1500')), findsOneWidget);
      final origin = t.getTopLeft(find.byType(FluidBoard));
      final scrolled = t.getTopLeft(find.byKey(const ValueKey('face-a1500'))) - origin;
      expect((scrolled.dy - (frame.faces['a1500']!.top - t.widget<CustomScrollView>(find.byType(CustomScrollView)).controller!.offset)).abs(), lessThan(1), reason: 'a built face stands where the projection places it, less the scroll');
    });

    testWidgets('the chosen asset keeps its ring when it scrolls out and back', (t) async {
      await t.pumpWidget(tall('thumbnail', selected: 'a3'));
      await t.pump(const Duration(milliseconds: 16));
      expect(find.byKey(const ValueKey('face-a3')), findsOneWidget);
      final frame = FluidBoard.thumbnail(many, 420);
      final controller = t.widget<CustomScrollView>(find.byType(CustomScrollView)).controller!;
      controller.jumpTo(frame.faces['a1500']!.top);
      await t.pump(const Duration(milliseconds: 50));
      expect(find.byKey(const ValueKey('face-a3')), findsNothing);
      controller.jumpTo(0);
      await t.pump(const Duration(milliseconds: 50));
      expect(find.byKey(const ValueKey('face-a3')), findsOneWidget, reason: 'the choice belongs to the browser, not to a built widget');
    });

    testWidgets('a change of view carries what is in sight and never builds the library (an asset is never two faces)', (t) async {
      await t.pumpWidget(tall('list', selected: 'a3'));
      await t.pump(const Duration(milliseconds: 16));
      await t.pumpWidget(tall('thumbnail', selected: 'a3'));
      await t.pump(const Duration(milliseconds: 60));
      expect(built(), lessThan(300), reason: 'mid-move: $built faces');
      final ids = find.byWidgetPredicate((w) => w.key is ValueKey<String> && (w.key as ValueKey<String>).value.startsWith('face-')).evaluate().map((e) => (e.widget.key as ValueKey<String>).value).toList();
      expect(ids.toSet().length, ids.length, reason: 'no asset is two faces while it moves');
      await t.pumpAndSettle();
      expect(built(), lessThan(200));
      expect(find.byKey(const ValueKey('face-a3')), findsOneWidget);
    });
  });
}

class _Faces implements FaceService {
  _Faces(this.asked);
  final List<double> asked;
  @override
  Future<String?> frameAt(BrowserItem item, double seconds, {int edge = 480}) async {
    asked.add(seconds);
    return null;
  }

  @override
  Future<String?> pictureOf(BrowserItem item) async => null;

}
