// Browser-family panels: four independent panels sharing capabilities, not bodies.
// flutter run -d macos -t lib/proto_hf/main_browser.dart [--dart-define=PROTO_SHOT=/path.png]
import 'dart:io';
import 'dart:ui' as ui;
import 'package:flutter/rendering.dart';
import 'package:flutter/widgets.dart';
import '../hf/bp/catalog_io.dart';
import '../hf/bp/classify.dart';
import '../hf/bp/colors.dart';
import '../hf/bp/common.dart';
import '../hf/bp/create.dart';
import '../hf/bp/effects.dart';
import '../hf/bp/fonts.dart';
import 'future.dart';
import '../hf/bp/search.dart';
import '../hf/bp/things.dart';

const _shot = String.fromEnvironment('PROTO_SHOT');
const _thingsDir = String.fromEnvironment('PROTO_THINGS', defaultValue: 'lib/proto_hf/data/things');
final _root = GlobalKey();

const tabs = [tabCreate, tabEffects, tabColors, tabFonts];

/// Preset discovery state for fixtures. The capabilities are the same objects the panels would own.
class Preset {
  const Preset({this.query = '', this.cls, this.stress = false});
  final String query;
  final String? cls;
  final bool stress;
}

late final Catalog baseCatalog, grownCatalog;
final user = UserViews(
  favorites: {'motolii.star', 'motolii.sphere', 'motolii.cube', 'motolii.blur', 'motolii.glow'},
  recent: ['motolii.text', 'motolii.ellipse', 'motolii.halftone'],
  saved: {'Lens glitch': 'family:lens tag:glitch'},
);

Widget panel(int i, EffectScene scene, {Preset p = const Preset()}) {
  final s = p.query.isEmpty ? null : SearchCapability(query: p.query);
  final c = p.cls == null ? null : ClassifyCapability(selected: p.cls);
  final cat = p.stress ? grownCatalog : baseCatalog;
  return switch (i) {
    0 => CreatePanel(catalog: cat, user: user, scene: scene, search: s, classify: c),
    1 => EffectsPanel(scene, catalog: cat, user: user, search: s, classify: c),
    2 => ColorsPanel(search: s, classify: c, items: p.stress ? colorsStress() : null),
    _ => FontsPanel(search: s, classify: c, items: p.stress ? fontsStress() : fontsBase),
  };
}

Widget leaf(int i, double w, double h, EffectScene scene, {List<TabSpec>? stack, int? active, Preset p = const Preset()}) => SizedBox(
      width: w,
      height: h,
      child: Leaf(tabs: stack ?? [tabs[i]], active: active ?? 0, body: panel(stack == null ? i : tabs.indexOf(stack[active ?? 0]), scene, p: p)),
    );

Widget label(String t) => Padding(padding: const EdgeInsets.only(bottom: 8, top: 22), child: Text(t, style: mono(11, c: const Color(0xFF8A8A8E), ls: 1)));
Widget gap(double w) => SizedBox(width: w);

void main() async {
  final scene = await EffectScene.build();
  baseCatalog = loadCatalog(_thingsDir);
  grownCatalog = loadCatalog(_thingsDir, sets: const ['builtin', 'stress']);
  final issues = grownCatalog.validate();
  if (issues.isNotEmpty) stdout.writeln('CAP catalog issues: ${issues.length}');
  const w = 370.0;
  runApp(WidgetsApp(
    color: kGround,
    debugShowCheckedModeBanner: false,
    builder: (_, __) => Align(
      alignment: Alignment.topLeft,
      child: OverflowBox(
        alignment: Alignment.topLeft, minWidth: 1640, maxWidth: 1640, minHeight: 2560, maxHeight: 2560,
        child: RepaintBoundary(
          key: _root,
          child: DefaultTextStyle(
            style: sans(12),
            child: ColoredBox(
              color: const Color(0xFF0C0C0D),
              child: Padding(
                padding: const EdgeInsets.fromLTRB(28, 4, 28, 28),
                child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
                  label('WIDE  370 x 640     class column, content body, search folded into a magnifier'),
                  Row(children: [for (var i = 0; i < 4; i++) ...[leaf(i, w, 640, scene), gap(20)]]),
                  label('STRIP  370 x 100     only the face is kept'),
                  Row(children: [for (var i = 0; i < 4; i++) ...[leaf(i, w, 100, scene), gap(20)]]),
                  label('NARROW  150 x 440   and   TAB-STACKED  340 x 440 (independent panels sharing a seat)'),
                  Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
                    for (var i = 0; i < 4; i++) ...[leaf(i, 150, 440, scene), gap(14)],
                    gap(26),
                    leaf(0, 340, 440, scene, stack: tabs, active: 0), gap(16),
                    leaf(2, 340, 440, scene, stack: tabs, active: 2), gap(16),
                    leaf(1, 190, 440, scene, stack: tabs, active: 1),
                  ]),
                  label('GROWTH  same layout, hundreds to thousands of things.  Create class Physics (40 curves, data only) / Effects query "source:plugin" / Colors 700+ / Fonts 2,000 searched "sans"'),
                  Row(children: [
                    leaf(0, w, 640, scene, p: const Preset(stress: true, cls: 'Physics')), gap(20),
                    leaf(1, w, 640, scene, p: const Preset(stress: true, query: 'source:plugin')), gap(20),
                    leaf(2, w, 640, scene, p: const Preset(stress: true)), gap(20),
                    leaf(3, w, 640, scene, p: const Preset(stress: true, query: 'sans')),
                  ]),
                  label('SAME SHELL, ENDLESS CONTENT  (identity tiles only; nothing behind them)'),
                  Row(children: [for (final n in futureNames) ...[FutureTile(n), gap(10)]]),
                ]),
              ),
            ),
          ),
        ),
      ),
    ),
  ));
  if (_shot.isNotEmpty) {
    void mark(String m) => stdout.writeln('CAP ${DateTime.now().millisecondsSinceEpoch % 100000} $m');
    mark('main-ready');
    Future<void>.delayed(const Duration(seconds: 40), () {
      mark('fail watchdog');
      exit(2);
    });
    WidgetsBinding.instance.addPostFrameCallback((_) async {
      mark('first-frame');
      try {
        final b = _root.currentContext!.findRenderObject()! as RenderRepaintBoundary;
        var n = 0;
        while (b.debugNeedsPaint && n < 90) {
          await WidgetsBinding.instance.endOfFrame;
          n++;
        }
        // post-frame work (scrolling a chosen class into view) runs after the first frame: let it land.
        for (var i = 0; i < 4; i++) {
          await WidgetsBinding.instance.endOfFrame;
        }
        mark('settled after $n frames, needsPaint=${b.debugNeedsPaint}');
        final layer = b.debugLayer as OffsetLayer;
        final img = await layer.toImage(Offset.zero & b.size, pixelRatio: 1.0);
        mark('image ${img.width}x${img.height}');
        File(_shot).writeAsBytesSync((await img.toByteData(format: ui.ImageByteFormat.png))!.buffer.asUint8List());
        mark('done');
        exit(0);
      } catch (e, st) {
        mark('fail $e');
        stdout.writeln(st.toString().split('\n').take(4).join('\n'));
        exit(1);
      }
    });
  }
}
