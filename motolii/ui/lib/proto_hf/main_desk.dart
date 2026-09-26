// Desk panels: five contextual instruments, each with wide, strip and tall-narrow morphologies.
// flutter run -d macos -t lib/proto_hf/main_desk.dart [--dart-define=PROTO_SHOT=/path.png]
import 'dart:io';
import 'dart:ui' as ui;
import 'package:flutter/rendering.dart';
import 'package:flutter/widgets.dart';
import 'bp/common.dart';
import 'desk/blend.dart';
import 'desk/depth.dart';
import 'desk/ease.dart';
import 'desk/history.dart';
import 'desk/notes.dart';

const _shot = String.fromEnvironment('PROTO_SHOT');
const _rows = String.fromEnvironment('PROTO_ROWS', defaultValue: 'all');
final _root = GlobalKey();

Widget housed(Widget child, double w, double h) => Container(
      width: w,
      height: h,
      decoration: BoxDecoration(color: kGround, border: Border.all(color: kRule2)),
      child: child,
    );

Widget label(String t) => Padding(padding: const EdgeInsets.only(bottom: 8, top: 22), child: Text(t, style: mono(11, c: const Color(0xFF8A8A8E), ls: 1)));

void main() {
  const wide = 310.0;
  final desks = <Widget Function()>[() => const EaseDesk(), () => const DepthDesk(), () => const BlendDesk(), () => const HistoryDesk(), () => const NotesDesk()];
  runApp(WidgetsApp(
    color: kGround,
    debugShowCheckedModeBanner: false,
    builder: (_, __) => Align(
      alignment: Alignment.topLeft,
      child: OverflowBox(
        alignment: Alignment.topLeft, minWidth: 1720, maxWidth: 1720, minHeight: _rows == 'wide' ? 760 : 1240, maxHeight: _rows == 'wide' ? 760 : 1240,
        child: RepaintBoundary(
          key: _root,
          child: DefaultTextStyle(
            style: sans(12),
            child: ColoredBox(
              color: const Color(0xFF0C0C0D),
              child: Padding(
                padding: const EdgeInsets.fromLTRB(28, 4, 28, 28),
                child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
                  label('WIDE  310 x 640     Ease / Depth / Blend / History / Notes'),
                  Row(crossAxisAlignment: CrossAxisAlignment.start, children: [for (final d in desks) ...[housed(d(), wide, 640), const SizedBox(width: 20)]]),
                  if (_rows != 'wide') ...[
                  label('NARROW STRIP  310 x 120     only the face is kept'),
                  Row(crossAxisAlignment: CrossAxisAlignment.start, children: [for (final d in desks) ...[housed(d(), wide, 120), const SizedBox(width: 20)]]),
                  label('NARROW TALL  150 x 300'),
                  Row(crossAxisAlignment: CrossAxisAlignment.start, children: [for (final d in desks) ...[housed(d(), 150, 300), const SizedBox(width: 20)]]),
                  ],
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
        for (var i = 0; i < 6; i++) {
          await WidgetsBinding.instance.endOfFrame;
        }
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
