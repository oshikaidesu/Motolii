// The Layout Instrument in its situations. flutter run -d macos -t lib/proto_hf/main_layout.dart [--dart-define=PROTO_SHOT=/path.png]
import 'dart:io';
import 'dart:ui' as ui;
import 'package:flutter/rendering.dart';
import 'package:flutter/widgets.dart';
import '../hf/bp/common.dart';
import '../hf/insp/layout.dart';
import '../hf/insp/layout_model.dart';
import '../hf/metrics.dart' show Surface;

const _shot = String.fromEnvironment('PROTO_SHOT');
final _root = GlobalKey();

Widget housed(Widget child, {double w = 310, double h = 660}) => Container(width: w, height: h, decoration: BoxDecoration(color: Surface.base, border: Border.all(color: Surface.dividerFine)), child: child);
Widget label(String t) => Padding(padding: const EdgeInsets.only(bottom: 8, top: 22), child: Text(t, style: mono(11, c: const Color(0xFF8A8A8E), ls: 1)));

void main() {
  LayoutStore base() => LayoutStore();
  LayoutStore centered() => LayoutStore()
    ..set('layout.horizontal_sizing', 1)
    ..set('layout.vertical_sizing', 2)
    ..set('layout.justify_content', 2)
    ..set('layout.align_items', 3)
    ..set('layout.grid_columns', 2);
  LayoutStore spread() => LayoutStore()
    ..set('layout.horizontal_sizing', 1)
    ..set('layout.vertical_sizing', 1)
    ..set('layout.justify_content', 3)
    ..set('layout.align_items', 0)
    ..set('layout.gap', 6.0)
    ..set('layout.grid_columns', 3)
    ..set('layout.grid_rows', 2);
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
                  label('LAYOUT INSTRUMENT     3 columns, Fixed width, Hug height  /  Fill width, Fixed height, centred  /  Fill both, spread across, Stretch  /  Grid off'),
                  Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
                    housed(LayoutInstrument(base(), advancedOpen: true)), const SizedBox(width: 20),
                    housed(LayoutInstrument(centered())), const SizedBox(width: 20),
                    housed(LayoutInstrument(spread())), const SizedBox(width: 20),
                    housed(LayoutInstrument(LayoutStore()..set('layout.display', 0))),
                  ]),
                  label('Child in a laid-out group  /  locked  /  narrow'),
                  Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
                    housed(LayoutInstrument(LayoutStore(child: true)), h: 420), const SizedBox(width: 20),
                    housed(LayoutInstrument(LayoutStore(frozen: true))), const SizedBox(width: 20),
                    housed(LayoutInstrument(base()), w: 170, h: 660),
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
