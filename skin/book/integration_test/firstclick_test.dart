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
import 'package:lab_book/sets/inspector/inspector_parts.dart' show AnchorCube, AnchorGeometry, ValuesCard, CopiesDrawing, LdTile, copiesRowOf, Ctx, Hov, LightDrawing, NumField, OnOff, PanelLink, Space, TextGroup, Chooser, GlyphPlate;
import 'package:lab_book/sets/inspector/inspector_rows_prop_row.dart' show PropRow;
import 'package:lab_book/workspace/dock_parts.dart' show DockHover, DockSearch;
import 'package:lab_book/workspace/timeline.dart';
import 'package:lab_book/workspace/workspace.dart' show Workspace;
import 'package:lab_book/workspace/ws.dart';

import 'package:lab_book/sets/inspector/inspector_direct_transform.dart' show DirectSurface;
import 'package:lab_book/sets/inspector/inspector_direct_transform_canvas.dart' show TransformCanvas;
import 'package:lab_book/sets/inspector/inspector_direct_transform_layout.dart' show TransformLayout;
import '../test/door.dart' show seen;

final _font = '$motoliiDir/out/arial_black.ttf';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized().framePolicy = LiveTestWidgetsFlutterBindingFramePolicy.fullyLive; // (the stage's picture is the inside's, drawn as the window goes)

  testWidgets('first clicks: the window as a stranger sees it, task by task', (t) async {
    t.binding.defaultBinaryMessenger.setMockMethodCallHandler(SystemChannels.menu, (_) async => null);
    // (a window tall enough that the inspector shows every card of a thing unscrolled: where a list is scrolled to would
    //  tell where the answer lies)
    t.view.physicalSize = Size(t.view.physicalSize.width, t.view.physicalSize.width * (const String.fromEnvironment('SET', defaultValue: 'a') == 'a' ? 1.6 : 3.2)); // (set B reaches cards further down)
    addTearDown(t.view.resetPhysicalSize);
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
      ..say(rect, 'stroke', 'ffffff') // (every state a task needs, set once before any picture: pictures of one thing are the same, so none tells by a changed pixel)
      ..set(rect, 'copies', 5)
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
      ('Make the selected card go along with the box whenever the box moves.', rect, null, find.ancestor(of: chooser('v_parent'), matching: find.byType(LdTile)).first, null),
      ('Make the selected card lighten what lies under it, like a projector\'s light, instead of covering it.', rect, null, chooser('v_blend'), null),
      ('Let the selected card be seen only through the thing just beneath it.', rect, null, find.descendant(of: row('v_clip'), matching: find.byType(OnOff)), null),
      ('Let the selected card sit deeper in the scene, while it stays a flat card.', rect, null, find.byWidgetPredicate((w) => w is GlyphPlate && w.label == '2.5D'), null),
      ('Make the selected card blue.', rect, null, row('v_fill'), null),
      ('Draw a line all round the edge of the selected card.', rect, null, row('v_stroke'), null),
      ('Make the line round the selected card heavier.', rect, null, num('v_stroke_width'), null),
      ('Turn the selected card into ten cards.', rect, null, num('v_copies'), null),
      ('Push the repeated cards further apart, left to right.', rect, null, num('v_copy_x'), null),
      ('Make the selected card look out of focus.', rect, null, find.ancestor(of: find.text('Gaussian Blur'), matching: find.byType(DockHover)), null),
      ('Change the words that are written.', text, null, find.descendant(of: find.byType(TextGroup), matching: find.byType(EditableText)), null),
      ('Put each line of the writing in the middle.', text, null, find.descendant(of: row('v_align'), matching: find.byWidgetPredicate((w) => w is Text && w.data?.toLowerCase() == 'centre')), null),
      ('Make the letters look different, in another style of lettering.', text, null, row('v_file'), null),
      ('Make the writing larger.', text, null, num('v_size'), null),
      ('Make the selected box look like it is made of glass.', cube, null, row('v_fill'), null),
      ('Make the light shine stronger.', sun, null, num('v_intensity'), null),
      ('Get a closer view of the scene through the camera without moving it.', camera, null, num('v_zoom'), null),
    ];
    // (set B: tasks the window was never tuned on, to tell whether it is understood or only taught to these twenty)
    final b = <(String, int, void Function()?, Finder, Rect Function(Rect)?)>[
      ('Move the selected card a little to the right.', rect, null, num('v_x'), null),
      ('Make the selected card twice as big.', rect, null, num('v_scale_x'), null),
      ('Lay the repeated cards out in rows of three.', rect, null, num('v_copy_columns'), null),
      ('Make the corners of the selected card round.', rect, null, num('v_round'), null),
      ('Make the selected card sway like a flag in the wind.', rect, null, find.byWidgetPredicate((w) => w is ValuesCard && w.card.name == 'Wave'), null),
      ('Make the selected card a solid block that pokes out toward you.', rect, null, find.byWidgetPredicate((w) => w is GlyphPlate && w.label == '3D'), null),
      ('Spread the letters of the writing further apart.', text, null, num('v_letter_spacing'), null),
      ('Make the letters of the writing lean forward.', text, null, num('v_italic'), null),
      ('Line the writing up on the right.', text, null, find.descendant(of: row('v_align'), matching: find.byWidgetPredicate((w) => w is Text && w.data?.toLowerCase() == 'right')), null),
      ('Make the shadows the light casts softer.', sun, null, num('v_shadow_softness'), null),
      ('Make the camera keep looking at the box.', camera, null, chooser('v_parent'), null),
      ('Tilt the camera so it looks down on the scene.', camera, null, num('v_rotate_x'), null),
      ('Move the camera further back from what it looks at.', camera, null, num('v_distance'), null),
      ('Make the selected box look like polished metal.', cube, null, num('v_metallic'), null),
      ('Make the selected box bounce when it falls.', cube, null, num('v_bounce'), null),
    ];
    final set = const String.fromEnvironment('SET', defaultValue: 'a');

    // (another right first click: with the words hidden, a blur is found by typing its name in the shelf's search)
    // (and a card is tilted by the round knob above it on the Transform card's own drawing, as well as by its number)
    Rect knob() {
      final box = t.getRect(find.byType(TransformCanvas)), doc = Ctx.read(t.element(find.byType(TransformCanvas))).doc;
      final at = TransformLayout(Size(box.width, DirectSurface.height), Map.of(doc.v), space: doc.space).knob;
      return Rect.fromCenter(center: box.topLeft + const Offset(0, 4) + at, width: 16, height: 16);
    }
    Rect copiesKnob(bool count) {
      final box = t.getRect(find.byType(CopiesDrawing)), g = copiesRowOf(Ctx.read(t.element(find.byType(CopiesDrawing))).doc, Size(box.width, CopiesDrawing.height));
      return Rect.fromCenter(center: box.topLeft + (count ? g.countKnob : g.gapKnob), width: 22, height: 22);
    }
    final also = <String, List<Rect> Function()>{
      'Make the selected card look out of focus.': () => [t.getRect(find.byType(DockSearch))],
      'Tilt the selected card, as you would tilt a picture on a wall.': () => [knob()],
      // (and the Depth desk's lens cone, pulled across, is the camera's zoom too)
      'Get a closer view of the scene through the camera without moving it.': () => [t.getRect(find.byWidgetPredicate((w) => w.runtimeType.toString() == '_Diagram').first)],
      // (and the Copies card's drawing: its + for how many, its second copy's knob for how far apart)
      'Turn the selected card into ten cards.': () => [copiesKnob(true)],
      'Push the repeated cards further apart, left to right.': () => [copiesKnob(false)],
      // (and a card's turning point is any bottom-left point of the box (front, middle or back: a flat card has no depth),
      //  or the point on the Transform card's drawing, pulled there)
      'Make the selected card turn and grow around its bottom-left corner instead of its middle.': () {
        final box = t.getRect(find.byType(AnchorCube)), g = AnchorGeometry(box.size);
        final canvas = t.getRect(find.byType(TransformCanvas)), doc = Ctx.read(t.element(find.byType(TransformCanvas))).doc;
        final pivot = TransformLayout(Size(canvas.width, DirectSurface.height), Map.of(doc.v), space: doc.space).pivot;
        return [Rect.fromPoints(box.topLeft + g.at(0, 100, -50), box.topLeft + g.at(0, 100, 50)).inflate(10), Rect.fromCenter(center: canvas.topLeft + const Offset(0, 4) + pivot, width: 16, height: 16)];
      },
      'Make the light shine stronger.': () { // (the strength rail at the right of the Light card's drawing)
        final box = t.getRect(find.byType(LightDrawing));
        return [Rect.fromLTRB(box.right - 28, box.top + 8, box.right, box.top + LightDrawing.height - 6)];
      },
    };
    final out = Directory('$motoliiDir/../reports/firstclick${set == 'a' ? '' : '_$set'}')..createSync(recursive: true);
    final dpr = t.view.devicePixelRatio, written = <Map<String, Object>>[];
    for (final (i, (task, thing, before, control, point)) in (set == 'a' ? tasks : b).indexed) {
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
      final view = Offset.zero & t.view.physicalSize / dpr; // (only a control off the window is scrolled to)
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
        for (final type in const [NumField, Chooser, OnOff, GlyphPlate, PanelLink, EditableText, DockHover, DockSearch, AnchorCube, Hov])
          for (final e in find.byType(type).evaluate())
            if (partOf(e) case final b? when !b.isEmpty && (Offset.zero & t.view.physicalSize / dpr).overlaps(b)) b,
      };
      final parts = [
        for (final b in all)
          if (!all.any((o) => o != b && b.contains(o.topLeft) && b.contains(o.bottomRight - const Offset(.01, .01)))) [b.left * dpr, b.top * dpr, b.right * dpr, b.bottom * dpr],
      ];
      // (a row's mark scrubs the row's own number: a click on it is a right first click too)
      final w = control.evaluate().first.widget;
      final scrub = w is NumField ? find.descendant(of: row(w.id), matching: find.byWidgetPredicate((x) => x.runtimeType.toString() == '_Scrub')) : null;
      final more = [...?also[task]?.call(), if (scrub != null && scrub.evaluate().isNotEmpty) t.getRect(scrub.first)];
      written.add({'parts': parts, 'n': i + 1, 'task': task, 'picture': name, 'rect': [r.left * dpr, r.top * dpr, r.right * dpr, r.bottom * dpr], 'words': words, if (more.isNotEmpty) 'also': [for (final x in more) [x.left * dpr, x.top * dpr, x.right * dpr, x.bottom * dpr]]});
    }
    File('${out.path}/tasks.json').writeAsStringSync(const JsonEncoder.withIndent('  ').convert(written));
  });
}
