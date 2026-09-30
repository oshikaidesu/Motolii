// The Transform Instrument, in its situations. flutter run -d macos -t lib/proto_hf/main_transform.dart [--dart-define=PROTO_SHOT=/path.png]
import 'dart:io';
import 'dart:ui' as ui;
import 'package:flutter/rendering.dart';
import 'package:flutter/widgets.dart';
import '../hf/bp/common.dart';
import '../hf/insp/transform.dart';
import '../hf/insp/transform_gizmo.dart';
import '../hf/insp/transform_model.dart';
import '../hf/metrics.dart' show Surface;

const _shot = String.fromEnvironment('PROTO_SHOT');
final _root = GlobalKey();

List<TLayer> layers() => [
      TLayer(1, 'Logo', position: [320, 180, 0], rotation: 32, anchor: [.5, .5])
        ..animated.addAll(['rotation', 'position'])
        ..keyedNow.add('rotation'),
      TLayer(2, 'Title', position: [-120, 40, 60], scale: [1.4, .8, 1], rotation: 12, rotX: 20, rotY: -8, depth: 120, anchor: [0, 1], projection: '2.5D', parent: 1)
        ..animated.add('scale')
        ..keyedNow.add('scale'),
      TLayer(3, 'Shape 3', position: [40, -60, 0], rotation: 90, locked: true),
      TLayer(4, 'Backdrop', position: [0, 0, 0]),
    ];

Widget housed(Widget child, {double w = 310, double h = 640}) => Container(width: w, height: h, decoration: BoxDecoration(color: Surface.base, border: Border.all(color: Surface.dividerFine)), child: child);
Widget label(String t) => Padding(padding: const EdgeInsets.only(bottom: 8, top: 22), child: Text(t, style: mono(11, c: const Color(0xFF8A8A8E), ls: 1)));

void main() {
  TransformStore st({List<int>? sel, int? active, bool unlink = false}) {
    final s = TransformStore(layers(), selection: sel, active: active);
    if (unlink) s.toggleLink('scale');
    return s;
  }

  runApp(WidgetsApp(
    color: Surface.base,
    debugShowCheckedModeBanner: false,
    builder: (_, __) => Align(
      alignment: Alignment.topLeft,
      child: OverflowBox(
        alignment: Alignment.topLeft, minWidth: 1420, maxWidth: 1420, minHeight: 1480, maxHeight: 1480,
        child: RepaintBoundary(
          key: _root,
          child: DefaultTextStyle(
            style: sans(12),
            child: ColoredBox(
              color: const Color(0xFF0C0C0D),
              child: Padding(
                padding: const EdgeInsets.fromLTRB(28, 4, 28, 28),
                child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
                  label('TRANSFORM INSTRUMENT     2D Move  /  2D Scale (kept in shape)  /  2.5D Rotate with axes  /  2.5D Anchor'),
                  Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
                    housed(TransformInstrument(st())), const SizedBox(width: 20),
                    housed(TransformInstrument(st(), initialMode: TMode.scale)), const SizedBox(width: 20),
                    housed(TransformInstrument(st(active: 2), initialMode: TMode.rotate)), const SizedBox(width: 20),
                    housed(TransformInstrument(st(active: 2), initialMode: TMode.anchor)),
                  ]),
                  label('Scale unlinked  /  mixed selection (Logo + Title)  /  locked layer  /  narrow'),
                  Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
                    housed(TransformInstrument(st(unlink: true), initialMode: TMode.scale)), const SizedBox(width: 20),
                    housed(TransformInstrument(st(sel: [1, 2], active: 1))), const SizedBox(width: 20),
                    housed(TransformInstrument(st(active: 3))), const SizedBox(width: 20),
                    housed(TransformInstrument(st()), w: 170, h: 560),
                  ]),
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
    Future<void>.delayed(const Duration(seconds: 40), () { mark('fail watchdog'); exit(2); });
    WidgetsBinding.instance.addPostFrameCallback((_) async {
      mark('first-frame');
      try {
        final b = _root.currentContext!.findRenderObject()! as RenderRepaintBoundary;
        for (var i = 0; i < 6; i++) { await WidgetsBinding.instance.endOfFrame; }
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
