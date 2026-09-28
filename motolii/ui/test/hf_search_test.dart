// The prototype's discovery capabilities (search, classification) exercised through all four panels.
// Behaviour is written once in bp/search.dart and bp/classify.dart; each panel only places its own faces.
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:motolii_stage5/hf/bp/catalog_io.dart';
import 'package:motolii_stage5/hf/bp/classify.dart';
import 'package:motolii_stage5/hf/bp/colors.dart';
import 'package:motolii_stage5/hf/bp/create.dart';
import 'package:motolii_stage5/hf/bp/effects.dart';
import 'package:motolii_stage5/hf/bp/fonts.dart';
import 'package:motolii_stage5/hf/bp/search.dart';
import 'package:motolii_stage5/hf/bp/shell.dart';
import 'package:motolii_stage5/hf/bp/things.dart';

Widget host(Widget child, double w, double h) => WidgetsApp(
      color: const Color(0xFF000000),
      pageRouteBuilder: <T>(settings, builder) => PageRouteBuilder<T>(settings: settings, pageBuilder: (c, _, __) => builder(c)),
      home: Align(alignment: Alignment.topLeft, child: SizedBox(width: w, height: h, child: child)),
    );

Future<void> openSearch(WidgetTester t) async {
  await t.tap(find.byType(HeaderKey).first);
  await t.pump();
  await t.pump();
}

final catalog = loadCatalog('lib/hf/data/things');

/// A Create key by its name: the board prints no caption; the name is the key's semantics label.
Finder mark(String name) => find.byWidgetPredicate((w) => w is Semantics && w.properties.label == name);

void main() {
  // Layout in tests needs the real UI face: the default test font is far wider and would overflow.
  setUpAll(() async {
    final inter = FontLoader('Inter')
      ..addFont(rootBundle.load('assets/fonts/Inter-Regular.ttf'))
      ..addFont(rootBundle.load('assets/fonts/Inter-Medium.ttf'))
      ..addFont(rootBundle.load('assets/fonts/Inter-SemiBold.ttf'));
    await inter.load();
  });

  group('capabilities, no widgets', () {
    test('search matches tokens case-insensitively across fields', () {
      final s = SearchCapability();
      final items = ['Blur', 'Color Shift', 'Chromatic', 'Halftone'];
      List<String> run(String q) {
        s.controller.text = q;
        return s.apply(items, (e) => [e]);
      }
      expect(run(''), items);
      expect(run('COLOR'), ['Color Shift']);
      expect(run('zzz'), isEmpty);
      s.dispose();
    });
    test('search notifies once per change and clears', () {
      final s = SearchCapability();
      var n = 0;
      s.addListener(() => n++);
      s.controller.text = 'a';
      s.controller.text = 'a';
      s.controller.text = 'ab';
      expect(n, 2);
      s.clear();
      expect(s.active, isFalse);
      s.dispose();
    });
    test('classification filters and combines with search', () {
      final c = ClassifyCapability();
      final items = [('Cube', '3D'), ('Star', 'Shapes'), ('Cone', '3D')];
      expect(c.apply(items, (e) => e.$2).length, 3);
      c.select('3D');
      expect(c.filtering, isTrue);
      expect(c.apply(items, (e) => e.$2).map((e) => e.$1), ['Cube', 'Cone']);
      c.select('All');
      expect(c.filtering, isFalse);
      c.dispose();
    });
  });

  group('one capability set, four panels', () {
    testWidgets('Create: header magnifier opens a field; results and count follow', (t) async {
      await t.pumpWidget(host(CreatePanel(catalog: catalog), 370, 640));
      expect(find.byType(EditableText), findsNothing);
      await openSearch(t);
      expect(find.byType(EditableText), findsOneWidget);
      await t.enterText(find.byType(EditableText), 'sphere');
      await t.pump();
      expect(mark('Sphere'), findsOneWidget);
      expect(mark('Cube'), findsNothing);
      await t.enterText(find.byType(EditableText), 'nothing here');
      await t.pump();
      expect(find.textContaining('No mark matches'), findsOneWidget);
    });

    testWidgets('Create: choosing a class in the column filters the body', (t) async {
      // the class decides what the body holds, whichever way it is chosen (not what fits on the first screen)
      await t.pumpWidget(host(CreatePanel(catalog: catalog), 370, 640));
      await t.tap(find.text('Shapes').first);
      await t.pump();
      expect(mark('Rectangle'), findsOneWidget);
      expect(mark('Cube'), findsNothing);
      await t.tap(find.text('3D').first);
      await t.pump();
      expect(mark('Cube'), findsOneWidget);
      expect(mark('Rectangle'), findsNothing);
    });

    testWidgets('Create: narrow keeps its classes along the top, and the chosen one on screen', (t) async {
      // Create lays its classes out as a strip (the board keeps the seat's width); a chosen class is never scrolled away
      final c = ClassifyCapability(selected: 'Helpers');
      await t.pumpWidget(host(CreatePanel(catalog: catalog, classify: c), 150, 440));
      await t.pump();
      expect(find.byType(ClassColumn), findsNothing);
      expect(find.byType(ClassStrip), findsOneWidget);
      final chosen = t.getRect(find.descendant(of: find.byType(ClassStrip), matching: find.text('Helpers')));
      expect(chosen.left >= 0 && chosen.right <= 150, isTrue, reason: 'the chosen class is in view: $chosen');
      await openSearch(t);
      await t.enterText(find.byType(EditableText), 'torus');
      await t.pump();
      expect(find.byType(EditableText), findsOneWidget);
    });

    testWidgets('Fonts: search by name at wide and narrow', (t) async {
      await t.pumpWidget(host(const FontsPanel(), 370, 640));
      await openSearch(t);
      await t.enterText(find.byType(EditableText), 'ne');
      await t.pump();
      expect(find.text('Helvetica Neue'), findsOneWidget);
      expect(find.text('Courier New'), findsOneWidget);
      expect(find.text('Georgia'), findsNothing);
      await t.pumpWidget(host(const FontsPanel(key: ValueKey('narrow')), 150, 440));
      await openSearch(t);
      await t.enterText(find.byType(EditableText), 'geo');
      await t.pump();
      expect(find.text('Georgia'), findsOneWidget);
    });

    testWidgets('Colors: names and hex, and the instrument yields to results', (t) async {
      await t.pumpWidget(host(const ColorsPanel(), 370, 640));
      await openSearch(t);
      await t.enterText(find.byType(EditableText), 'pink');
      await t.pump();
      expect(find.text('USED HERE'), findsOneWidget);
      await t.enterText(find.byType(EditableText), '#000000');
      await t.pump();
      expect(find.text('USED HERE'), findsNothing);
      expect(find.text('MONOCHROME'), findsOneWidget);
    });

    testWidgets('Effects: same capabilities, own faces', (t) async {
      final scene = (await t.runAsync(EffectScene.build))!;
      await t.pumpWidget(host(EffectsPanel(scene, catalog: catalog), 370, 640));
      await openSearch(t);
      await t.enterText(find.byType(EditableText), 'pixel');
      await t.pump();
      expect(find.text('Pixelate'), findsOneWidget);
      expect(find.text('Glow'), findsNothing);
    });

    testWidgets('growth: thousands of items, one layout, class plus search', (t) async {
      await t.pumpWidget(host(FontsPanel(items: fontsStress(), classify: ClassifyCapability(selected: 'JP')), 370, 640));
      expect(find.byType(ClassColumn), findsOneWidget);
      await openSearch(t);
      await t.enterText(find.byType(EditableText), 'mincho');
      await t.pump();
      expect(find.textContaining('Hiragino Mincho'), findsWidgets);
    });

    testWidgets('keys: slash focuses, escape clears then leaves', (t) async {
      final s = SearchCapability();
      await t.pumpWidget(host(FontsPanel(search: s), 370, 640));
      s.panel.requestFocus();
      await t.pump();
      await t.sendKeyEvent(LogicalKeyboardKey.slash);
      await t.pump();
      expect(s.focus.hasFocus, isTrue);
      await t.enterText(find.byType(EditableText), 'geo');
      await t.pump();
      expect(s.active, isTrue);
      await t.sendKeyEvent(LogicalKeyboardKey.escape);
      await t.pump();
      expect(s.active, isFalse);
      expect(find.text('Georgia'), findsOneWidget);
      s.dispose();
    });
  });
}
