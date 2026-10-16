// The window's first-click test (how a stranger finds a control: Bailey and Wolfson's first click): each task a picture
// of the real window, the task as a goal in plain words (no name the window uses for its controls), and where its right control lies on the picture. A judge who has never
// seen the window says where they would click first; reports/firstclick/grade.py counts the clicks inside.
//   flutter drive --profile -d macos --driver=test_driver/integration_test.dart --target=integration_test/firstclick_test.dart
// Writes reports/firstclick/NN.png and tasks.json (rects in the picture's own pixels).
import 'dart:convert';
import 'dart:io';
import 'dart:ui' show ImageByteFormat;

import 'package:flutter/rendering.dart' show OffsetLayer, RenderBox;
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:lab_book/motolii.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart' show AnchorCube, AnchorGeometry, Hov, InspectorPanel, NumField, OnOff, PanelLink, Space, SpacePlate, TextGroup, Chooser;
import 'package:lab_book/sets/inspector/inspector_rows_prop_row.dart' show PropRow;
import 'package:lab_book/workspace/dock_parts.dart' show DockHover, DockSearch;
import 'package:lab_book/workspace/timeline.dart';
import 'package:lab_book/workspace/workspace.dart' show Workspace;
import 'package:lab_book/workspace/ws.dart';

import '../test/door.dart' show seen;

const _font = '$motoliiDir/out/arial_black.ttf';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized().framePolicy = LiveTestWidgetsFlutterBindingFramePolicy.fullyLive; // (the stage's picture is the inside's, drawn as the window goes)

  testWidgets('first clicks: the window as a stranger sees it, task by task', (t) async {
    t.binding.defaultBinaryMessenger.setMockMethodCallHandler(SystemChannels.menu, (_) async => null);
    final doc = '${Directory.systemTemp.createTempSync('firstclick').path}/firstclick.motolii';
    await t.pumpWidget(WidgetsApp(color: const Color(0xFF000000), builder: (c, _) => Workspace(document: doc)));
    await t.pumpAndSettle();
    final ws = WsScope.read(t.element(find.byType(WsTimeline))), m = ws.inside!;
    final rect = m.add('shape', 'rect'), text = m.add('text', _font), cube = m.add('solid', 'cube'), sun = m.add('light', 'sun'), camera = m.add('camera', 'camera');
    m
      ..say(rect, 'name', 'Card')
      ..say(rect, 'fill', 'ff3366')
      ..say(text, 'text', 'Hello')
      ..say(cube, 'name', 'Box')
      ..set(cube, 'x', 2.5) // (the things side by side, as a stranger's stage would have them: none hides another)
      ..set(text, 'y', -1.6);
    ws
      ..reread()
      ..shelf = 'Effects';
    await t.pumpAndSettle();

    Finder num(String id) => find.byWidgetPredicate((w) => w is NumField && w.id == id);
    Finder row(String id) => find.byWidgetPredicate((w) => w is PropRow && w.id == id);
    Finder chooser(String id) => find.byWidgetPredicate((w) => w is Chooser && w.id == id);
    // (the task, the thing picked, what to do first, the right control; for a point of a control (the anchor box's corner), where in it)
    final tasks = <(String, int, void Function()?, Finder, Rect Function(Rect)?)>[
      ('Let what is behind the selected card show through it a little.', rect, null, num('v_opacity'), null),
      ('Tilt the selected card, as you would tilt a picture on a wall.', rect, null, num('v_rotate'), null),
      ('Make the selected card turn and grow around its bottom-left corner instead of its middle.', rect, null, find.byType(AnchorCube),
          (box) => Rect.fromCenter(center: box.topLeft + AnchorGeometry(box.size).at(0, 100, 0), width: 20, height: 20)),
      ('Make the selected card go along with the box whenever the box moves.', rect, null, chooser('v_parent'), null),
      ('Make the selected card lighten what lies under it, like a projector\'s light, instead of covering it.', rect, null, chooser('v_blend'), null),
      ('Let the selected card be seen only through the thing just beneath it.', rect, null, find.descendant(of: row('v_clip'), matching: find.byType(OnOff)), null),
      ('Let the selected card sit deeper in the scene, while it stays a flat card.', rect, null, find.byWidgetPredicate((w) => w is SpacePlate && w.space == Space.d25), null),
      ('Make the selected card blue.', rect, null, row('v_fill'), null),
      ('Draw a line all round the edge of the selected card.', rect, null, row('v_stroke'), null),
      ('Make the line round the selected card heavier.', rect, () => m.say(rect, 'stroke', 'ffffff'), num('v_stroke_width'), null),
      ('Turn the selected card into ten cards.', rect, null, num('v_copies'), null),
      ('Push the repeated cards further apart, left to right.', rect, () => m.set(rect, 'copies', 5), num('v_copy_x'), null),
      ('Make the selected card look out of focus.', rect, null, find.ancestor(of: find.text('Gaussian Blur'), matching: find.byType(DockHover)), null),
      ('Change the words that are written.', text, null, find.descendant(of: find.byType(TextGroup), matching: find.byType(EditableText)), null),
      ('Put each line of the writing in the middle.', text, null, find.descendant(of: row('v_align'), matching: find.byWidgetPredicate((w) => w is Text && w.data?.toLowerCase() == 'centre')), null),
      ('Make the letters look different, in another style of lettering.', text, null, row('v_file'), null),
      ('Make the writing larger.', text, null, num('v_size'), null),
      ('Make the selected box look like it is made of glass.', cube, null, row('v_fill'), null),
      ('Make the light shine stronger.', sun, null, num('v_intensity'), null),
      ('Get a closer view of the scene through the camera without moving it.', camera, null, num('v_zoom'), null),
    ];

    // (another right first click: with the words hidden, a blur is found by typing its name in the shelf's search)
    final also = <String, Finder>{'Make the selected card look out of focus.': find.byType(DockSearch)};
    final out = Directory('$motoliiDir/../reports/firstclick')..createSync(recursive: true);
    final dpr = t.view.devicePixelRatio, written = <Map<String, Object>>[];
    for (final (i, (task, thing, before, control, point)) in tasks.indexed) {
      before?.call();
      ws
        ..reread()
        ..select('$thing');
      await t.pumpAndSettle();
      // (the window as a hand finds it: every list at its top; a control below the fold brought to the middle of its
      //  list, not to its top edge, so where it lies tells nothing of the answer)
      for (final e in find.byType(Scrollable).evaluate()) {
        if ((e as StatefulElement).state case final ScrollableState l when l.position.hasPixels) l.position.jumpTo(l.position.minScrollExtent);
      }
      await t.pumpAndSettle();
      final view = t.getRect(find.byType(InspectorPanel));
      if (control.evaluate().isEmpty || !view.contains(t.getRect(control.first).center)) {
        await seen(t, control);
        await Scrollable.ensureVisible(t.element(control.first), alignment: .5);
        await t.pumpAndSettle();
      }
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 300))); // (the inside's picture of what was just said, drawn)
      await t.pump();
      final box = t.getRect(control.first), r = point?.call(box) ?? box;
      // ignore: invalid_use_of_protected_member
      final root = t.binding.renderViews.first.layer! as OffsetLayer;
      final png = await (await root.toImage(Offset.zero & t.view.physicalSize)).toByteData(format: ImageByteFormat.png);
      final name = '${(i + 1).toString().padLeft(2, '0')}.png';
      File('${out.path}/$name').writeAsBytesSync(png!.buffer.asUint8List());
      // (every word on the window, where it lies: the test with the words hidden asks whether a control speaks by its glyph)
      final words = [
        for (final e in [...find.byType(RichText).evaluate(), ...find.byType(EditableText).evaluate()])
          if (e.renderObject case final RenderBox b when b.hasSize && b.attached)
            if (MatrixUtils.transformRect(b.getTransformTo(null), Offset.zero & b.size) case final w when !w.isEmpty) [w.left * dpr, w.top * dpr, w.right * dpr, w.bottom * dpr],
      ];
      // (every part a hand may press, the innermost of those nested: a random click's chance of the right one is the baseline)
      Rect? partOf(Element e) {
        final o = e.renderObject;
        return o is RenderBox && o.hasSize && o.attached ? MatrixUtils.transformRect(o.getTransformTo(null), Offset.zero & o.size) : null;
      }
      final all = {
        for (final type in const [NumField, Chooser, OnOff, SpacePlate, PanelLink, EditableText, DockHover, DockSearch, AnchorCube, Hov])
          for (final e in find.byType(type).evaluate())
            if (partOf(e) case final b? when !b.isEmpty && (Offset.zero & t.view.physicalSize / dpr).overlaps(b)) b,
      };
      final parts = [
        for (final b in all)
          if (!all.any((o) => o != b && b.contains(o.topLeft) && b.contains(o.bottomRight - const Offset(.01, .01)))) [b.left * dpr, b.top * dpr, b.right * dpr, b.bottom * dpr],
      ];
      written.add({'parts': parts, 'n': i + 1, 'task': task, 'picture': name, 'rect': [r.left * dpr, r.top * dpr, r.right * dpr, r.bottom * dpr], 'words': words, if (also[task] case final a?) 'also': [for (final x in [t.getRect(a.first)]) [x.left * dpr, x.top * dpr, x.right * dpr, x.bottom * dpr]]});
    }
    File('${out.path}/tasks.json').writeAsStringSync(const JsonEncoder.withIndent('  ').convert(written));
  });
}
