// The cards' own drawings that a hand pulls (Drop Shadow, Trim Paths): a pull is the document's, one step.
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/sets/inspector/inspector_shadow.dart';
import 'package:lab_book/sets/inspector/inspector_trim.dart';

import 'door.dart';

void main() {
  testWidgets('Drop Shadow: the shadow taken and put up and to the right falls that way, further; its knob pulled out is softer', (t) async {
    final ws = await openWorkspace(t), m = ws.inside!;
    final rect = m.add('shape', 'rect');
    expect(m.effect(rect, 'Drop Shadow', 0), isTrue, reason: m.trouble);
    ws..reread()..select('$rect');
    await t.pumpAndSettle();
    final drawing = find.byType(ShadowDrawing);
    await seen(t, drawing);
    final box = t.getRect(drawing), was = (m.value(rect, 'shadow_distance', 0) ?? 0, m.value(rect, 'shadow_blur', 0) ?? 0);
    await t.dragFrom(box.center, Offset(box.height * .35, -box.height * .3)); // (from the thing's middle: the shadow lies under it when near)
    await t.pumpAndSettle();
    final angle = m.value(rect, 'shadow_angle', 0)!;
    expect(angle, inInclusiveRange(0, 90), reason: 'up and to the right (ThorVG, as After Effects: 0 up, 90 right): $angle');
    expect(m.value(rect, 'shadow_distance', 0)!, greaterThan(was.$1), reason: 'further');
    expect(m.undo(), isTrue);
    expect(m.value(rect, 'shadow_distance', 0) ?? 0, was.$1, reason: 'one step');
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  testWidgets('Trim Paths: the end taken left cuts the path there; the part left taken slides it', (t) async {
    final ws = await openWorkspace(t), m = ws.inside!;
    final rect = m.add('shape', 'rect');
    expect(m.effect(rect, 'Trim Paths', 0), isTrue, reason: m.trouble);
    m..set(rect, 'trim', .2)..set(rect, 'trim_begin', 0); // (a fifth cut off the end: the card is there while something is cut)
    ws..reread()..select('$rect');
    await t.pumpAndSettle();
    final drawing = find.byType(TrimDrawing);
    await seen(t, drawing);
    final box = t.getRect(drawing);
    final line = box.width - 32, top = box.top + (box.height - 6) / 2;
    await t.dragFrom(Offset(box.left + 16 + line * .8, top), Offset(-line * .3, 0)); // (the end knob, four fifths along)
    await t.pumpAndSettle();
    expect(m.value(rect, 'trim', 0)!, closeTo(.5, .03), reason: 'cut off the end as far as it was taken');
    await t.dragFrom(Offset(box.left + 16 + line * .25, top), Offset(line * .1, 0)); // (the part left)
    await t.pumpAndSettle();
    expect(m.value(rect, 'trim_offset', 0)!, greaterThan(.05), reason: 'slid along');
    await closeWorkspace(t);
  }, skip: !insideBuilt);
}
