// The fluid board moves the same faces between projections: the selection and the identity stay, and a face is between its
// two places part-way through the change (motion, not a swap).
import 'dart:ui' show PointerDeviceKind;

import 'package:flutter/gestures.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:motolii_stage5/live_hf/adapters/browser_item.dart';
import 'package:motolii_stage5/live_hf/adapters/media_fluid.dart';
import 'package:motolii_stage5/live_hf/adapters/media_preview.dart' show FaceService;

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
    await t.pump(const Duration(milliseconds: 150)); // the stagger: this face sets off some 127 ms in
    await t.pump(const Duration(milliseconds: 120));
    final part = faceOf(t, 'a3');
    await t.pumpAndSettle();
    final inThumb = faceOf(t, 'a3');
    expect(inList, isNot(inThumb));
    expect(part, isNot(inList), reason: 'it has left the list place');
    expect(part, isNot(inThumb), reason: 'and is not yet at the thumbnail place: it is moving');
    // and the layout it settles into is the projection's own
    final frame = FluidBoard.thumbnail(items, 320);
    final origin = t.getTopLeft(find.byType(FluidBoard));
    expect(inThumb.topLeft - origin, frame.faces['a3']!.topLeft);
  });

  testWidgets('every asset stays one face across the change, the chosen one still ringed', (t) async {
    await t.pumpWidget(board('list', selected: 'a2'));
    await t.pumpAndSettle();
    for (final view in ['thumbnail', 'list', 'thumbnail']) {
      await t.pumpWidget(board(view, selected: 'a2'));
      await t.pumpAndSettle();
      for (final i in items) {
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

  testWidgets('a pointer passing over a clip asks the owner for the frame it is over, without a click', (t) async {
    final asked = <double>[];
    final clip = BrowserItem(id: 'clip', name: 'clip.mp4', path: '/nowhere/clip.mp4', kind: 'video', mime: 'video/mp4', width: 1280, height: 720, seconds: 8);
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
    await t.pump();
    await t.pump(const Duration(milliseconds: 50));
    expect(asked, isNotEmpty);
    expect(asked.last, closeTo(6, .6), reason: 'three quarters across an 8 second clip');
    await mouse.moveTo(const Offset(319, 419));
    await t.pump();
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
