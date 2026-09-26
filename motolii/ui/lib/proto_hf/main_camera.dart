// The Camera Instrument in its situations. flutter run -d macos -t lib/proto_hf/main_camera.dart [--dart-define=PROTO_SHOT=/path.png]
import 'dart:io';
import 'dart:ui' as ui;
import 'package:flutter/rendering.dart';
import 'package:flutter/widgets.dart';
import 'bp/common.dart';
import 'insp/camera.dart';
import 'insp/camera_model.dart';

const _shot = String.fromEnvironment('PROTO_SHOT');
final _root = GlobalKey();

const layers = [CamLayer(1, 'Logo', 960, 540, 0), CamLayer(2, 'Title', 400, 300, 60), CamLayer(3, 'Backdrop', 1500, 800, -200)];

Widget housed(Widget child, {double w = 310, double h = 640}) => Container(width: w, height: h, decoration: BoxDecoration(color: kGround, border: Border.all(color: kRule2)), child: child);
Widget label(String t) => Padding(padding: const EdgeInsets.only(bottom: 8, top: 22), child: Text(t, style: mono(11, c: const Color(0xFF8A8A8E), ls: 1)));

void main() {
  CameraStore def() => CameraStore(layers: layers);
  CameraStore orbited() => CameraStore(layers: layers)
    ..set('camera.orbit', [22.0, -38.0])
    ..set('camera.distance', 1.6)
    ..set('camera.roll', 14.0)
    ..set('camera.center', [120.0, -40.0]);
  CameraStore behind() => CameraStore(layers: layers)
    ..set('camera.orbit', [-15.0, 140.0])
    ..set('camera.distance', .5)
    ..set('camera.zoom', 2.0);
  runApp(WidgetsApp(
    color: kGround,
    debugShowCheckedModeBanner: false,
    builder: (_, __) => Align(
      alignment: Alignment.topLeft,
      child: OverflowBox(
        alignment: Alignment.topLeft, minWidth: 1900, maxWidth: 1900, minHeight: 760, maxHeight: 760,
        child: RepaintBoundary(
          key: _root,
          child: DefaultTextStyle(
            style: sans(12),
            child: ColoredBox(
              color: const Color(0xFF0C0C0D),
              child: Padding(
                padding: const EdgeInsets.fromLTRB(28, 4, 28, 28),
                child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
                  label('CAMERA INSTRUMENT     default  /  orbited, pulled back, rolled  /  behind the target, close, zoomed  /  Target layer set  /  locked  /  narrow'),
                  Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
                    housed(CameraInstrument(def())), const SizedBox(width: 20),
                    housed(CameraInstrument(orbited())), const SizedBox(width: 20),
                    housed(CameraInstrument(behind())), const SizedBox(width: 20),
                    housed(CameraInstrument(CameraStore(layers: layers, target: 2)..set('camera.orbit', [10.0, 25.0]))), const SizedBox(width: 20),
                    housed(CameraInstrument(CameraStore(layers: layers, frozen: true)), w: 200),
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
