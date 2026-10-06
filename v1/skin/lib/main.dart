// SKIN. Shows the Stage (a GPU texture The Forge draws) and writes what the hand does into the Core (Rerun). Nothing else.
import 'dart:async';
import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter/scheduler.dart';
import 'package:flutter/services.dart';

const _ch = MethodChannel('motolii');
const _frames = 300;

void main() => runApp(MaterialApp(debugShowCheckedModeBanner: false, theme: ThemeData.dark(useMaterial3: true), home: const Motolii()));

class Motolii extends StatefulWidget {
  const Motolii({super.key});
  @override
  State<Motolii> createState() => _S();
}

class _S extends State<Motolii> with SingleTickerProviderStateMixin {
  int? stage;
  int frame = 0;
  bool playing = true;
  double orbit = 0.4, height = 48, distance = 52;
  String stats = '';
  late final Ticker _ticker;
  Duration _last = Duration.zero;
  double _acc = 0;

  @override
  void initState() {
    super.initState();
    _ch.invokeMethod<int>('stage').then((v) => setState(() => stage = v));
    // the camera move, as values over time, lives in the Core
    for (var f = 0; f < _frames; f++) {
      final t = f / 60.0;
      _ch.invokeMethod('set', ['/world/camera/orbit', f, 0.4 + 0.9 * math.sin(t * 0.9)]);
      _ch.invokeMethod('set', ['/world/camera/height', f, 48 + 14 * math.sin(t * 0.7)]);
      _ch.invokeMethod('set', ['/world/camera/distance', f, 52 + 8 * math.cos(t * 0.6)]);
      _ch.invokeMethod('set', ['/world/video/seconds', f, 5.0 + (f ~/ 2) / 30.0]);
      _ch.invokeMethod('set', ['/world/svg/scale', f, 0.75 + 0.25 * math.sin(t * 2.0)]);
    }
    _ticker = createTicker((now) {
      final dt = (now - _last).inMicroseconds / 1e6;
      _last = now;
      if (!playing) return;
      _acc += dt * 60;
      if (_acc >= 1) {
        final n = _acc.floor();
        _acc -= n;
        _go((frame + n) % _frames);
      }
    })..start();
    Timer.periodic(const Duration(seconds: 2), (_) async {
      final s = await _ch.invokeMapMethod<String, dynamic>('stats');
      if (s == null || !mounted) return;
      final line = 'stage ${((s['frames'] as int) / (s['secs'] as double)).toStringAsFixed(1)} fps   set->frame ${(s['setToFrameMs'] as double).toStringAsFixed(1)} ms   video ${((s['videoFrames'] as int) / (s['secs'] as double)).toStringAsFixed(1)} f/s   svg ${(s['svgMs'] as double).toStringAsFixed(2)} ms';
      debugPrint('MOTOLII $line');
      setState(() => stats = line);
    });
  }

  void _go(int f) {
    _ch.invokeMethod('playhead', f);
    setState(() => frame = f);
  }

  void _set(String path, double v) => _ch.invokeMethod('set', [path, frame, v]);

  @override
  Widget build(BuildContext context) => Scaffold(
        body: Column(children: [
          Expanded(
            child: GestureDetector(
              onPanUpdate: (d) {
                setState(() {
                  playing = false;
                  orbit -= d.delta.dx * 0.01;
                  height += d.delta.dy * 0.3;
                });
                // from here on in time the hand's value holds (latest-at): overwrite the rest of the move
                for (var f = frame; f < _frames; f++) {
                  _ch.invokeMethod('set', ['/world/camera/orbit', f, orbit]);
                  _ch.invokeMethod('set', ['/world/camera/height', f, height]);
                }
              },
              child: Center(child: stage == null ? const SizedBox() : AspectRatio(aspectRatio: 16 / 9, child: Texture(textureId: stage!))),
            ),
          ),
          Row(children: [
            IconButton(icon: Icon(playing ? Icons.pause : Icons.play_arrow), onPressed: () => setState(() => playing = !playing)),
            Expanded(child: Slider(value: frame.toDouble(), max: _frames - 1.0, onChanged: (v) { playing = false; _go(v.round()); })),
            SizedBox(width: 56, child: Text('$frame')),
            const Text('distance'),
            SizedBox(width: 180, child: Slider(value: distance, min: 20, max: 120, onChanged: (v) { setState(() => distance = v); for (var f = frame; f < _frames; f++) { _ch.invokeMethod('set', ['/world/camera/distance', f, v]); } })),
            Padding(padding: const EdgeInsets.only(right: 12), child: Text(stats, style: const TextStyle(fontFeatures: [FontFeature.tabularFigures()]))),
          ]),
        ]),
      );
}
