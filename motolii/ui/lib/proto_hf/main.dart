// flutter run -d macos -t lib/proto_hf/main.dart [--dart-define=PROTO_SHOT=/path.png]
import 'dart:io';
import 'dart:ui' as ui;
import 'package:flutter/rendering.dart';
import 'package:flutter/widgets.dart';
import 'hf.dart';
import 'ref.dart';

const _shot = String.fromEnvironment('PROTO_SHOT');
final _root = GlobalKey();

void main() {
  runApp(WidgetsApp(
    color: H.window,
    debugShowCheckedModeBanner: false,
    builder: (_, __) => Align(
      alignment: Alignment.topLeft,
      child: OverflowBox(
        alignment: Alignment.topLeft, minWidth: 1536, maxWidth: 1536, minHeight: 1024, maxHeight: 1024,
        child: RepaintBoundary(key: _root, child: DefaultTextStyle(style: H.s(13), child: RF(all()))),
      ),
    ),
  ));
  if (_shot.isNotEmpty) {
    Future<void>.delayed(const Duration(seconds: 2), () async {
      final b = _root.currentContext!.findRenderObject()! as RenderRepaintBoundary;
      final img = await b.toImage(pixelRatio: 1);
      File(_shot).writeAsBytesSync((await img.toByteData(format: ui.ImageByteFormat.png))!.buffer.asUint8List());
      exit(0);
    });
  }
}
