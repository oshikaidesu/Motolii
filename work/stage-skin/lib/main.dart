// Skin only: a window that shows what the renderer made and says which frame to make. No rendering, no timeline model.
// The renderer is The Forge's ray-query sample (a separate process); `seek N` goes in, `ready N` comes out.
import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:flutter/material.dart';

const _forge =
    '/Users/member_ottoto/rust_ae/_ext/The-Forge/Examples_3/Unit_Tests/macOS_Xcode/16_Raytracing/Bin/Release/16_Raytracing.app/Contents/MacOS/16_Raytracing';
const _work = '/Users/member_ottoto/rust_ae/Motolii/work/forge_scene';
const _stage = '/Users/member_ottoto/rust_ae/_ext/stage';
const _world = '/Users/member_ottoto/rust_ae/_ext/rerun-world-target/release/rerun-world';
const _video =
    '/Users/member_ottoto/コラ素材meme/第１回【超底辺】YouTuber（高校生）の財布の中身を、いきなり、チェックしたら、やばすぎ、、。.mp4';
const _frames = 300;
const _fps = 30;
const _videoAt = 12;

void main() => runApp(const StageApp());

class StageApp extends StatelessWidget {
  const StageApp({super.key});
  @override
  Widget build(BuildContext context) => MaterialApp(
        debugShowCheckedModeBanner: false,
        theme: ThemeData.dark(useMaterial3: true),
        home: const Stage(),
      );
}

class Stage extends StatefulWidget {
  const Stage({super.key});
  @override
  State<Stage> createState() => _StageState();
}

class _StageState extends State<Stage> {
  Process? _renderer;
  Uint8List? _image;
  int _shown = -1;
  int _target = 0;
  int _rendering = -1;
  bool _playing = false;
  String _status = 'starting the renderer…';
  Timer? _clock;

  @override
  void initState() {
    super.initState();
    _start();
  }

  @override
  void dispose() {
    _clock?.cancel();
    _renderer?.stdin.writeln('quit');
    _renderer?.kill();
    super.dispose();
  }

  Future<void> _start() async {
    await Directory(_stage).create(recursive: true);
    await Directory('/Users/member_ottoto/rust_ae/_ext/The-Forge/Examples_3/Unit_Tests/macOS_Xcode/16_Raytracing/Bin/Release/Screenshots')
        .create(recursive: true);
    // the world (Rerun) answers per frame: camera, focus, video time, effect parameters
    await _run('/bin/sh', ['-c', '$_world $_frames $_fps $_videoAt > $_stage/params.csv && $_world $_frames $_fps $_videoAt fx > $_stage/fx.csv']);
    final p = await Process.start(
      _forge,
      ['-ApplePersistenceIgnoreState', 'YES', '-w', '960', '-h', '540'],
      workingDirectory: _stage,
      environment: {
        'MOTOLII_NOUI': '1',
        'MOTOLII_SERVE': '1',
        'MOTOLII_SPP': '24',
        'MOTOLII_PARAMS': '$_stage/params.csv',
        'MOTOLII_SVG': '$_work/assets/sunface.svg',
        'MOTOLII_VIDEO': _video,
      },
    );
    _renderer = p;
    p.stdout.transform(utf8.decoder).transform(const LineSplitter()).listen(_onLine);
    p.stderr.drain();
    setState(() => _status = 'renderer up; first frame…');
    Future.delayed(const Duration(seconds: 4), () => _request(_target));
  }

  Future<ProcessResult> _run(String exe, List<String> args) => Process.run(exe, args);

  void _request(int n) {
    if (_renderer == null) return;
    _rendering = n;
    setState(() => _status = 'rendering frame $n…');
    _renderer!.stdin.writeln('seek $n');
  }

  Future<void> _onLine(String line) async {
    if (!line.startsWith('ready ')) return;
    final n = int.parse(line.substring(6).trim());
    final r = await _run('$_work/post.sh', ['$n', _stage]);
    if (r.exitCode != 0) {
      setState(() => _status = 'post failed: ${r.stderr}');
      return;
    }
    final bytes = await File('$_stage/out_$n.png').readAsBytes();
    if (!mounted) return;
    setState(() {
      _image = bytes;
      _shown = n;
      _rendering = -1;
      _status = 'frame $n  (${(n / _fps).toStringAsFixed(2)} s)';
    });
    if (_target != n) {
      _request(_target);
    } else if (_playing) {
      _target = (n + 3) % _frames;
      _request(_target);
    }
  }

  void _seek(double v) {
    _target = v.round();
    if (_rendering < 0) _request(_target);
  }

  void _togglePlay() {
    setState(() => _playing = !_playing);
    if (_playing && _rendering < 0) {
      _target = (_shown + 3) % _frames;
      _request(_target);
    }
  }

  Future<void> _export() async {
    // the export script runs its own renderer instance: let go of the Stage's, then bring it back
    setState(() => _status = 'exporting 3 s with audio…');
    _playing = false;
    _renderer?.stdin.writeln('quit');
    _renderer = null;
    await Future.delayed(const Duration(seconds: 2));
    final out = '$_stage/export.mp4';
    final r = await _run('$_work/export.sh', ['0', '89', '$_fps', '32', out, '960', '540']);
    final note = r.exitCode == 0 ? 'exported $out' : 'export failed: ${r.stderr}';
    await _start();
    setState(() => _status = note);
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: const Color(0xFF0E0E10),
      body: Column(children: [
        Expanded(
          child: Center(
            child: _image == null
                ? const CircularProgressIndicator()
                : AspectRatio(
                    aspectRatio: 16 / 9,
                    child: Image.memory(_image!, gaplessPlayback: true, fit: BoxFit.contain),
                  ),
          ),
        ),
        Padding(
          padding: const EdgeInsets.fromLTRB(16, 4, 16, 12),
          child: Row(children: [
            IconButton(onPressed: _togglePlay, icon: Icon(_playing ? Icons.pause : Icons.play_arrow)),
            Expanded(
              child: Slider(
                value: _target.toDouble().clamp(0, _frames - 1),
                min: 0,
                max: _frames - 1.0,
                onChanged: (v) => setState(() => _seek(v)),
              ),
            ),
            SizedBox(width: 260, child: Text(_status, overflow: TextOverflow.ellipsis)),
            const SizedBox(width: 12),
            FilledButton(onPressed: _export, child: const Text('Export')),
          ]),
        ),
      ]),
    );
  }
}
