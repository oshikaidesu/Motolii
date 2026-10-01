// Thing descriptors: static validation, queries, derived views, and panels fed only by data.
import 'dart:convert';
import 'dart:io';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:motolii_stage5/hf/bp/catalog_io.dart';
import 'package:motolii_stage5/hf/bp/create.dart';
import 'package:motolii_stage5/hf/bp/effects.dart';
import 'package:motolii_stage5/hf/bp/faces.dart';
import 'package:motolii_stage5/hf/bp/shell.dart';
import 'package:motolii_stage5/hf/bp/things.dart';

const dir = 'lib/proto_hf/data/things';

Map<String, dynamic> good() => {
      'id': 'acme.demo_thing',
      'name': 'Demo',
      'kind': 'effect',
      'family': 'blur',
      'tags': ['blur'],
      'capabilities': ['apply'],
      'source': 'plugin:acme.pack',
      'searchTerms': ['demo'],
      'face': {'type': 'fx', 'base': 'Blur', 'hue': 0},
    };

List<Issue> check(Registry r, Map<String, dynamic> j, {Set<String> implemented = const {}}) => validateThings(r, [('t.json', j)], implemented: implemented);
Set<String> rules(List<Issue> l) => {for (final i in l) i.rule};

void main() {
  final reg = loadCatalog(dir).registry;

  setUpAll(() async {
    final inter = FontLoader('Inter')
      ..addFont(rootBundle.load('assets/fonts/Inter-Regular.ttf'))
      ..addFont(rootBundle.load('assets/fonts/Inter-Medium.ttf'))
      ..addFont(rootBundle.load('assets/fonts/Inter-SemiBold.ttf'));
    await inter.load();
  });

  group('the committed catalog', () {
    test('has no validation issue, including the growth set', () {
      final c = loadCatalog(dir, sets: const ['builtin', 'stress']);
      expect(c.validate(), isEmpty, reason: c.validate().join('\n'));
    });
    test('every panel kind and family is registered', () {
      expect(validateRegistry(reg), isEmpty);
    });
  });

  group('static rules catch each kind of drift', () {
    test('a valid descriptor passes', () => expect(check(reg, good()), isEmpty));
    test('id format', () => expect(rules(check(reg, {...good(), 'id': 'Bad Id'})), contains('THING-ID-FORMAT')));
    test('duplicate id', () {
      final l = validateThings(reg, [('a.json', good()), ('b.json', good())]);
      expect(rules(l), contains('THING-ID-DUPLICATE'));
    });
    test('name required', () => expect(rules(check(reg, {...good(), 'name': ' '})), contains('THING-NAME')));
    test('unknown kind is rejected with a suggestion', () {
      final l = check(reg, {...good(), 'kind': 'efect'});
      expect(rules(l), contains('THING-KIND'));
      expect(l.first.message, contains('effect'));
    });
    test('an invented family is rejected', () => expect(rules(check(reg, {...good(), 'family': 'spaceships'})), contains('THING-FAMILY')));
    test('a family must accept the kind', () => expect(rules(check(reg, {...good(), 'family': 'physics'})), contains('THING-FAMILY-KIND')));
    test('a tag typo is caught and corrected', () {
      final l = check(reg, {...good(), 'tags': ['blurr']});
      expect(rules(l), contains('THING-TAG-UNKNOWN'));
      expect(l.first.message, contains('blur'));
    });
    test('x- tags are the extension lane', () => expect(check(reg, {...good(), 'tags': ['blur', 'x-acme-fast']}), isEmpty));
    test('unknown capability', () => expect(rules(check(reg, {...good(), 'capabilities': ['aply']})), contains('THING-CAPABILITY-UNKNOWN')));
    test('source shape', () => expect(rules(check(reg, {...good(), 'source': 'somewhere'})), contains('THING-SOURCE')));
    test('must be searchable beyond its name', () => expect(rules(check(reg, {...good(), 'tags': <String>[], 'searchTerms': <String>[]})), contains('THING-NOT-SEARCHABLE')));
    test('a face is required', () {
      final j = good()..remove('face');
      expect(rules(check(reg, j)), contains('THING-FACE-MISSING'));
    });
    test('face type and parameters are checked', () {
      expect(rules(check(reg, {...good(), 'face': {'type': 'hologram'}})), contains('THING-FACE-TYPE'));
      expect(rules(check(reg, {...good(), 'face': {'type': 'fx', 'base': 'Blurr', 'hue': 0}})), contains('THING-FACE-PARAM'));
      expect(rules(check(reg, {...good(), 'face': {'type': 'mark', 'mark': 'cube', 'color': 'red'}})), contains('THING-FACE-PARAM'));
    });
    test('a declared capability needs an implementation', () {
      expect(rules(check(reg, good(), implemented: {'effect:apply'})), isEmpty);
      expect(rules(check(reg, good(), implemented: {'body:apply'})), contains('THING-CAPABILITY-UNIMPLEMENTED'));
    });
    test('a kind with no panel would never appear', () {
      final j = jsonDecode(File('$dir/registry.json').readAsStringSync()) as Map<String, dynamic>;
      (j['kinds'] as Map)['ghost'] = {'label': 'Ghost'};
      expect(rules(validateRegistry(Registry.fromJson(j))), contains('REG-KIND-UNBOUND'));
    });
  });

  group('query language', () {
    final things = loadCatalog(dir, sets: const ['builtin', 'stress']).things;
    List<String> ids(String q) => [for (final t in things) if (ThingQuery.parse(q).matches(t, reg)) t.id];
    test('plain words search name, terms, tags and family', () {
      expect(ids('sphere'), contains('motolii.sphere'));
      expect(ids('spring').length, greaterThan(0));
    });
    test('key:value filters combine', () {
      final a = ids('kind:effect source:plugin');
      expect(a, isNotEmpty);
      expect(a.every((id) => id.startsWith('acme.')), isTrue);
      expect(ids('family:physics kind:relation').length, greaterThan(20));
    });
    test('a parent family also matches its children', () {
      final rigid = things.where((t) => t.family == 'physics/rigid').length;
      expect(rigid, greaterThan(0));
      expect(ids('family:physics').length, greaterThanOrEqualTo(rigid));
    });
    test('unknown keys are ordinary words', () => expect(ThingQuery.parse('foo:bar').words, ['foo:bar']));
  });

  group('derived views', () {
    final cat = loadCatalog(dir, sets: const ['builtin', 'stress']);
    test('system views come from families present, in registry order', () {
      final v = ThingViews(cat.registry, cat.things, cat.registry.panels['create']!, UserViews());
      final g = v.groups(v.scope);
      expect(g.first.first, 'All');
      expect(g.first, contains('Physics'));
      expect(g.first, isNot(contains('Blur')));
    });
    test('search narrows the class list to families with matches, as After Effects keeps context', () {
      final v = ThingViews(cat.registry, cat.things, cat.registry.panels['create']!, UserViews());
      final found = [for (final t in v.scope) if (ThingQuery.parse('spring').matches(t, cat.registry)) t];
      final g = v.groups(found).first;
      expect(g, contains('Physics'));
      expect(g.length, lessThan(v.groups(v.scope).first.length));
    });
    test('user views are separate from metadata', () {
      final user = UserViews(favorites: {'motolii.star'}, saved: {'Glitchy lens': 'family:lens tag:glitch'});
      final v = ThingViews(cat.registry, cat.things, cat.registry.panels['effects']!, user);
      expect(v.groups(v.scope)[1], containsAll(['Favorites', 'Recent', 'Glitchy lens']));
      expect(v.contains('Favorites', cat.things.firstWhere((t) => t.id == 'motolii.star')), isTrue);
      expect(v.scope.where((t) => v.contains('Glitchy lens', t)), isNotEmpty);
    });
  });

  group('faces', faceDrawsTests);

  group('panels are fed by data only', () {
    Widget host(Widget c) => WidgetsApp(color: const Color(0xFF000000), pageRouteBuilder: <T>(s, b) => PageRouteBuilder<T>(settings: s, pageBuilder: (x, _, __) => b(x)), home: Align(alignment: Alignment.topLeft, child: SizedBox(width: 370, height: 640, child: c)));
    testWidgets('Create shows a registered physics thing with no physics code', (t) async {
      final cat = loadCatalog(dir, sets: const ['builtin', 'stress']);
      await t.pumpWidget(host(CreatePanel(catalog: cat)));
      expect(find.text('PHYSICS'), findsNothing); // a class in the column, not a section, until chosen
      await t.tap(find.text('Physics').first);
      await t.pump();
      expect(find.text('Spring'), findsOneWidget);
    });
    testWidgets('a structured query reaches the Effects panel through the shared search', (t) async {
      final cat = loadCatalog(dir, sets: const ['builtin', 'stress']);
      final scene = (await t.runAsync(EffectScene.build))!;
      await t.pumpWidget(host(EffectsPanel(scene, catalog: cat)));
      await t.tap(find.byType(HeaderKey).first);
      await t.pump();
      await t.pump();
      await t.enterText(find.byType(EditableText), 'source:plugin tag:glitch');
      await t.pump();
      expect(find.text('Glow'), findsNothing);
      expect(find.byType(GridView), findsOneWidget);
    });
  });
}


// A face that declares a known effect must actually draw something. A shadowed name once made every
// effect thumbnail blank without any error, so this checks pixels, not just that a widget exists.
void faceDrawsTests() {
  testWidgets('an fx face paints pixels', (t) async {
    final cat = loadCatalog(dir);
    final scene = (await t.runAsync(EffectScene.build))!;
    for (final id in ['motolii.blur', 'motolii.halftone', 'motolii.invert']) {
      final thing = cat.things.firstWhere((x) => x.id == id);
      final key = GlobalKey();
      await t.pumpWidget(Directionality(textDirection: TextDirection.ltr, child: Align(alignment: Alignment.topLeft, child: RepaintBoundary(key: key, child: SizedBox(width: 80, height: 67, child: ThingFace(thing, scene: scene))))));
      final img = await t.runAsync(() async {
        final b = key.currentContext!.findRenderObject()! as RenderRepaintBoundary;
        return b.toImage();
      });
      final data = (await t.runAsync(() => img!.toByteData()))!;
      var lit = 0;
      for (var i = 0; i < data.lengthInBytes; i += 4) {
        if (data.getUint8(i) + data.getUint8(i + 1) + data.getUint8(i + 2) > 30) lit++;
      }
      expect(lit, greaterThan(500), reason: '$id drew almost nothing');
    }
  });
}
