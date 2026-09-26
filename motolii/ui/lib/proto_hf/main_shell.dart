// Assembled shell fixture: the latest accepted component of every seat in one fake 1536x1024 window. No Rust, no Document, no new design.
// Seat provenance (see docs/stage5/ui-rebaseline/handoff/shell-provenance.md):
//   Top, Stage, Timeline = hf.dart top() / stage() / timeline() (build-candidate-v5, polish4), placed at their reference rectangles.
//   Browser = bp/*, Inspector = insp/*, Desk = desk/* (their own latest fixtures).
//   flutter run -d macos -t lib/proto_hf/main_shell.dart            (fills the window, hot reload)
//   flutter run -d macos -t lib/proto_hf/main_shell.dart --dart-define=PROTO_SHOT=/path.png   (1536x1024 capture)
// The bottom strip is fixture plumbing (which fixture each seat shows), not product UI.
import 'dart:io';
import 'dart:ui' as ui;
import 'package:flutter/rendering.dart';
import 'package:flutter/widgets.dart';
import 'bp/catalog_io.dart';
import 'bp/common.dart';
import 'bp/effects.dart';
import 'desk/blend.dart';
import 'desk/depth.dart';
import 'desk/ease.dart';
import 'desk/history.dart';
import 'desk/notes.dart';
import 'hf.dart' as hf;
import 'ref.dart' show H, RF, RI;
import 'insp/camera.dart';
import 'insp/camera_model.dart';
import 'insp/fixtures.dart';
import 'insp/layout.dart';
import 'insp/layout_model.dart';
import 'insp/panel.dart';
import 'insp/rows.dart';
import 'insp/transform.dart';
import 'insp/transform_model.dart';
import 'main_browser.dart' as br;
import 'main_camera.dart' as cam;
import 'main_transform.dart' as tf;

const _shot = String.fromEnvironment('PROTO_SHOT');
const _thingsDir = String.fromEnvironment('PROTO_THINGS', defaultValue: '/Users/member_ottoto/rust_ae/Motolii/motolii/ui/lib/proto_hf/data/things');
const _browser = int.fromEnvironment('PROTO_BROWSER', defaultValue: 0);
const _inspector = int.fromEnvironment('PROTO_INSPECTOR', defaultValue: 1);
const _desk = int.fromEnvironment('PROTO_DESK', defaultValue: -1); // -1: the right seat shows an Inspector view
final _root = GlobalKey();


void main() async {
  final scene = await EffectScene.build();
  br.baseCatalog = loadCatalog(_thingsDir);
  br.grownCatalog = loadCatalog(_thingsDir, sets: const ['builtin', 'stress']);
  runApp(WidgetsApp(
    color: kGround,
    debugShowCheckedModeBanner: false,
    builder: (_, __) => _shot.isEmpty
        ? DefaultTextStyle(style: sans(12), child: Shell(scene: scene))
        : Align(
            alignment: Alignment.topLeft,
            child: OverflowBox(
              alignment: Alignment.topLeft, minWidth: 1536, maxWidth: 1536, minHeight: 1024, maxHeight: 1024,
              child: RepaintBoundary(key: _root, child: DefaultTextStyle(style: sans(12), child: Shell(scene: scene, strip: false))),
            ),
          ),
  ));
  if (_shot.isNotEmpty) {
    void mark(String m) => stdout.writeln('CAP ${DateTime.now().millisecondsSinceEpoch % 100000} $m');
    mark('main-ready');
    Future<void>.delayed(const Duration(seconds: 40), () { mark('fail watchdog'); exit(2); });
    WidgetsBinding.instance.addPostFrameCallback((_) async {
      mark('first-frame');
      try {
        final b = _root.currentContext!.findRenderObject()! as RenderRepaintBoundary;
        for (var i = 0; i < 8; i++) { await WidgetsBinding.instance.endOfFrame; }
        final img = await (b.debugLayer as OffsetLayer).toImage(Offset.zero & b.size, pixelRatio: 1.0);
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

class Shell extends StatefulWidget {
  const Shell({super.key, required this.scene, this.strip = true});
  final EffectScene scene;
  final bool strip;
  @override
  State<Shell> createState() => _ShellState();
}

class _ShellState extends State<Shell> {
  int browserTab = _browser;
  // The right seat shows one Inspector view or one Desk. The concept has no Desk seat: this is a placeholder, not a placement.
  int seat = _desk >= 0 ? 4 + _desk : _inspector;
  // One store per inspector seat view, kept for the session so edits survive switching.
  late final ParamStore layerStore = ParamStore(nativeLike());
  late final TransformStore transformStore = TransformStore(tf.layers());
  late final LayoutStore layoutStore = LayoutStore();
  late final CameraStore cameraStore = CameraStore(layers: cam.layers);

  Widget _rightSeat() => switch (seat) {
        0 => InspectorBody(store: layerStore, subject: 'Layer', thingId: 'Layer'),
        1 => TransformInstrument(transformStore),
        2 => LayoutInstrument(layoutStore),
        3 => CameraInstrument(cameraStore),
        4 => const EaseDesk(),
        5 => const DepthDesk(),
        6 => const BlendDesk(),
        7 => const HistoryDesk(),
        _ => const NotesDesk(),
      };

  /// A region of the reference canvas, drawn by the accepted harness lists at their own coordinates.
  Widget _ref(double x, double y, double w, double h, List<RI> items) =>
      Positioned(left: x, top: y, width: w, height: h, child: RF(items, ox: x, oy: y));

  Widget _seatBox(double x, double y, double w, double h, Widget child) => Positioned(
        left: x, top: y, width: w, height: h,
        child: DecoratedBox(decoration: BoxDecoration(color: kGround, border: Border.all(color: kRule)), child: child),
      );

  Widget _canvas() => SizedBox(
        width: 1536,
        height: 1024,
        child: Stack(children: [
          const Positioned.fill(child: ColoredBox(color: H.window)),
          _ref(0, 0, 1536, 62, hf.top()),
          _seatBox(10, 62, 324, 953, br.leaf(browserTab, 324, double.infinity, widget.scene, stack: br.tabs, active: browserTab)),
          _ref(344, 62, 779, 632, hf.stage()),
          _seatBox(1135, 62, 387, 631, _rightSeat()),
          _ref(344, 703, 1178, 291, hf.timeline()),
        ]),
      );

  @override
  Widget build(BuildContext context) => !widget.strip
      ? _canvas()
      : ColoredBox(
          color: kGround,
          child: Column(children: [
            Expanded(child: Center(child: FittedBox(fit: BoxFit.scaleDown, child: _canvas()))),
            _FixtureStrip(
              browser: browserTab, seat: seat,
              onBrowser: (i) => setState(() => browserTab = i),
              onSeat: (i) => setState(() => seat = i),
            ),
          ]),
        );
}

class _FixtureStrip extends StatelessWidget {
  const _FixtureStrip({required this.browser, required this.seat, required this.onBrowser, required this.onSeat});
  final int browser, seat;
  final ValueChanged<int> onBrowser, onSeat;

  Widget _group(String label, List<String> names, int on, ValueChanged<int> pick) => Row(mainAxisSize: MainAxisSize.min, children: [
        Text(label, style: mono(9.5, c: const Color(0xFF6C6D70), ls: 1)),
        const SizedBox(width: 6),
        for (var i = 0; i < names.length; i++)
          GestureDetector(
            behavior: HitTestBehavior.opaque,
            onTap: () => pick(i),
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 5, vertical: 4),
              child: Text(names[i], style: mono(9.5, c: i == on ? const Color(0xFFE6E6E8) : const Color(0xFF6C6D70))),
            ),
          ),
      ]);

  @override
  Widget build(BuildContext context) => Container(
        height: 22,
        padding: const EdgeInsets.symmetric(horizontal: 10),
        decoration: const BoxDecoration(color: Color(0xFF111112), border: Border(top: BorderSide(color: kRule))),
        child: Row(children: [
          Text('FIXTURE', style: mono(9.5, c: const Color(0xFF55565A), ls: 1.4)),
          const SizedBox(width: 18),
          _group('BROWSER', const ['Create', 'Effects', 'Colors', 'Fonts'], browser, onBrowser),
          const SizedBox(width: 18),
          _group('RIGHT SEAT', const ['Layer', 'Transform', 'Layout', 'Camera', 'Ease', 'Depth', 'Blend', 'History', 'Notes'], seat, onSeat),
        ]),
      );
}
