// Generic Inspector, Phase 1: unknown parameters only. Wide panels of the same body at different sizes of declaration.
// flutter run -d macos -t lib/proto_hf/main_inspector.dart [--dart-define=PROTO_SHOT=/path.png]
import 'dart:io';
import 'dart:ui' as ui;
import 'package:flutter/rendering.dart';
import 'package:flutter/widgets.dart';
import '../hf/bp/common.dart';
import '../hf/bp/search.dart';
import 'fixtures.dart';
import '../hf/insp/panel.dart';
import '../hf/insp/rows.dart';

const _shot = String.fromEnvironment('PROTO_SHOT');
const _phase = String.fromEnvironment('PROTO_PHASE', defaultValue: '1');
final _root = GlobalKey();

Widget housed(Widget child, {double w = 310, double h = 640}) => Container(width: w, height: h, decoration: BoxDecoration(color: kGround, border: Border.all(color: kRule2)), child: child);
Widget label(String t) => Padding(padding: const EdgeInsets.only(bottom: 8, top: 22), child: Text(t, style: mono(11, c: const Color(0xFF8A8A8E), ls: 1)));

void main() {
  runApp(WidgetsApp(
    color: kGround,
    debugShowCheckedModeBanner: false,
    builder: (_, __) => Align(
      alignment: Alignment.topLeft,
      child: OverflowBox(
        alignment: Alignment.topLeft, minWidth: _phase == '2' ? 1080 : 1420, maxWidth: _phase == '2' ? 1080 : 1420, minHeight: _phase == '2' ? 760 : 1480, maxHeight: _phase == '2' ? 760 : 1480,
        child: RepaintBoundary(
          key: _root,
          child: DefaultTextStyle(
            style: sans(12),
            child: ColoredBox(
              color: const Color(0xFF0C0C0D),
              child: Padding(
                padding: const EdgeInsets.fromLTRB(28, 4, 28, 28),
                child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
                  if (_phase == '2') ...[
                  label('PHASE 2  declared meaning refines the same Values      semantic values + routes  /  scrolled to the routes  /  frozen'),
                  Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
                    housed(InspectorBody(store: ParamStore(nativeLike()), subject: 'Layer', thingId: 'Layer')), const SizedBox(width: 20),
                    housed(InspectorBody(store: ParamStore(nativeLike()), subject: 'Layer', thingId: 'Layer', initialOffset: 430)), const SizedBox(width: 20),
                    housed(InspectorBody(store: ParamStore(nativeLike(), frozen: true), subject: 'Layer', thingId: 'Layer')),
                  ]),
                  ] else ...[
                  label('UNKNOWN WGSL  declared only by types     1 param  /  18 params, advanced closed  /  advanced open, scrolled  /  filter "e"'),
                  Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
                    housed(InspectorBody(store: ParamStore(stress(1)), subject: 'One parameter')), const SizedBox(width: 20),
                    housed(InspectorBody(store: ParamStore(unknownEffect()), subject: 'Unknown effect')), const SizedBox(width: 20),
                    housed(InspectorBody(store: ParamStore(unknownEffect()), subject: 'Unknown effect', advancedOpen: true, initialOffset: 560)), const SizedBox(width: 20),
                    housed(InspectorBody(store: ParamStore(unknownEffect()), subject: 'Unknown effect', search: SearchCapability(query: 'e'))),
                  ]),
                  label('FROZEN  /  64 parameters  /  256 parameters (scrolled to the middle)'),
                  Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
                    housed(InspectorBody(store: ParamStore(unknownEffect(), frozen: true), subject: 'Unknown effect')), const SizedBox(width: 20),
                    housed(InspectorBody(store: ParamStore(stress(64)), subject: '64 parameters')), const SizedBox(width: 20),
                    housed(InspectorBody(store: ParamStore(stress(256)), subject: '256 parameters', initialOffset: 4200)),
                  ]),
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
