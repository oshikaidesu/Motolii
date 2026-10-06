// Probe: Flutter Texture widget backed by an IOSurface video frame (no pixels cross into Dart).
import 'dart:async';
import 'package:flutter/material.dart';
import 'package:flutter/scheduler.dart';
import 'package:flutter/services.dart';

void main() => runApp(const MaterialApp(debugShowCheckedModeBanner: false, home: TexProbe()));

class TexProbe extends StatefulWidget {
  const TexProbe({super.key});
  @override
  State<TexProbe> createState() => _S();
}

class _S extends State<TexProbe> {
  static const ch = MethodChannel('skin/tex');
  int? id;
  int frames = 0;
  String line = '';
  @override
  void initState() {
    super.initState();
    ch.invokeMethod<int>('create', const String.fromEnvironment('CLIP')).then((v) => setState(() => id = v));
    void tick(Duration _) {
      frames++;
      SchedulerBinding.instance.scheduleFrameCallback(tick);
    }
    SchedulerBinding.instance.scheduleFrameCallback(tick);
    Timer.periodic(const Duration(seconds: 5), (_) async {
      final s = await ch.invokeMapMethod<String, dynamic>('stats');
      if (s == null) return;
      final secs = s['secs'] as double;
      print('PROBE flutter frames/s ${(frames / secs).toStringAsFixed(1)}  textures handed to engine/s ${((s['handed'] as int) / secs).toStringAsFixed(1)}');
    });
  }
  @override
  Widget build(BuildContext context) => Scaffold(body: Center(child: id == null ? const Text('...') : AspectRatio(aspectRatio: 16 / 9, child: Texture(textureId: id!))));
}
