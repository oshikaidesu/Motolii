// Glass › Live: the Filament glass of motolii/filament, drawn live and shown here as a texture.
// The library (out/libglass_live.dylib, built by filament/build_live.sh) draws every frame on the GPU into a pixel
// buffer; the macOS runner hands that same buffer to Flutter (macos/Runner/MainFlutterWindow.swift, GlassTexture).
// Nothing is copied and nothing is read back. This file only shows the texture and sends numbers: drag turns the
// camera, the wheel moves it nearer or further, the knobs set the sun and the pace.
// macOS only, Debug build only (the Debug entitlements have the sandbox off so that the library can be loaded).
import 'dart:async';
import 'dart:ffi';
import 'dart:math' as math;

import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import '../tokens.dart';

/// Where the glass lives. Override with --dart-define=GLASS_DIR=/path/to/motolii/filament.
const _dir = String.fromEnvironment('GLASS_DIR', defaultValue: '/Users/motolii_maker/motolii/filament');

/// The picture's size in pixels, fixed when the library starts.
const _w = 1920, _h = 1080;

/// What stands on the floor, read once when the library starts (the names are glass_scene's own).
const _scene = {
  'ONLY': 'cube,slab,prism,pillar,hexagon,bar,pyramid,wedge,octahedron,ball,star,washer,letterA,letterB,letterO,letterM,letterR',
  'COUNT': '24',
  'EXNEW': '1',
};

/// The numbers the library takes while it runs (glass_live.mm, glass_set).
abstract final class _Slot {
  static const sunRound = 0, sunDown = 1, camRound = 2, camUp = 3, camFar = 4, pace = 5, frameMs = 6;
}

/// Sizes and limits of this view, in one place.
abstract final class _G {
  static const pad = 16.0, readoutPad = 8.0, radius = 6.0;
  static const turnPerPx = .008, tiltPerPx = .006, zoomPerPx = .0015;
  static const camUpMin = .05, camUpMax = 1.45, camFarMin = 1.2, camFarMax = 8.0;
  static const camRound0 = .5828, camUp0 = .5033, camFar0 = 3.0075;
}

class _Lib {
  _Lib(DynamicLibrary l)
    : set = l.lookupFunction<Void Function(Int32, Float), void Function(int, double)>('glass_set'),
      get = l.lookupFunction<Float Function(Int32), double Function(int)>('glass_get'),
      frames = l.lookupFunction<Uint64 Function(), int Function()>('glass_frames'),
      stop = l.lookupFunction<Void Function(), void Function()>('glass_stop');
  final void Function(int, double) set;
  final double Function(int) get;
  final int Function() frames;
  final void Function() stop;
}

/// The live glass. [sunRound] and [sunDown] in degrees, [pace] 0 (still) to 2.
class GlassLiveView extends StatefulWidget {
  const GlassLiveView({super.key, required this.sunRound, required this.sunDown, required this.pace});
  final double sunRound, sunDown, pace;
  @override
  State<GlassLiveView> createState() => _GlassLiveViewState();
}

class _GlassLiveViewState extends State<GlassLiveView> {
  static const _channel = MethodChannel('motolii/glass');
  int? _texture;
  String? _problem;
  _Lib? _lib;
  double _round = _G.camRound0, _up = _G.camUp0, _far = _G.camFar0;
  Timer? _tick;
  int _seen = 0;
  final _readout = ValueNotifier<String>('');

  @override
  void initState() {
    super.initState();
    _start();
  }

  Future<void> _start() async {
    const lib = '$_dir/out/libglass_live.dylib';
    try {
      final id = await _channel.invokeMethod<int>('start', {'lib': lib, 'base': _dir, 'width': _w, 'height': _h, 'env': _scene});
      if (!mounted) return;
      if (id == null || id < 0) {
        setState(() => _problem = 'Could not load $lib\nBuild it with filament/build_live.sh');
        return;
      }
      _lib = _Lib(DynamicLibrary.open(lib));
      _send();
      _tick = Timer.periodic(const Duration(seconds: 1), (_) {
        final l = _lib;
        if (l == null) return;
        final now = l.frames();
        _readout.value = '${now - _seen} fps   ${l.get(_Slot.frameMs).toStringAsFixed(1)} ms   $_w × $_h';
        _seen = now;
      });
      setState(() => _texture = id);
    } on MissingPluginException {
      if (mounted) setState(() => _problem = 'Live glass runs in the macOS app only');
    } catch (e) {
      if (mounted) setState(() => _problem = '$e');
    }
  }

  void _send() {
    final l = _lib;
    if (l == null) return;
    l.set(_Slot.sunRound, widget.sunRound * math.pi / 180);
    l.set(_Slot.sunDown, widget.sunDown * math.pi / 180);
    l.set(_Slot.pace, widget.pace);
    l.set(_Slot.camRound, _round);
    l.set(_Slot.camUp, _up);
    l.set(_Slot.camFar, _far);
  }

  @override
  void didUpdateWidget(GlassLiveView old) {
    super.didUpdateWidget(old);
    _send();
  }

  @override
  void dispose() {
    _tick?.cancel();
    _lib?.stop(); // a pause: the library keeps its engine and goes on when this view is opened again
    _readout.dispose();
    super.dispose();
  }

  void _drag(PointerMoveEvent e) {
    if (e.buttons == 0) return;
    _round -= e.delta.dx * _G.turnPerPx;
    _up = (_up + e.delta.dy * _G.tiltPerPx).clamp(_G.camUpMin, _G.camUpMax);
    _send();
  }

  void _wheel(PointerSignalEvent e) {
    if (e is! PointerScrollEvent) return;
    _far = (_far * math.exp(e.scrollDelta.dy * _G.zoomPerPx)).clamp(_G.camFarMin, _G.camFarMax);
    _send();
  }

  @override
  Widget build(BuildContext context) {
    final id = _texture;
    return ColoredBox(
      color: N.g07,
      child: Padding(
        padding: const EdgeInsets.all(_G.pad),
        child: Center(
          child: AspectRatio(
            aspectRatio: _w / _h,
            child: id == null
                ? Center(
                    child: Text(_problem ?? 'Starting…', style: T.micro(N.g63), textAlign: TextAlign.center),
                  )
                : ClipRRect(
                    borderRadius: BorderRadius.circular(_G.radius),
                    child: Listener(
                      onPointerMove: _drag,
                      onPointerSignal: _wheel,
                      child: Stack(
                        fit: StackFit.expand,
                        children: [
                          Texture(textureId: id, filterQuality: FilterQuality.medium),
                          Positioned(
                            left: _G.readoutPad,
                            bottom: _G.readoutPad,
                            child: ValueListenableBuilder(
                              valueListenable: _readout,
                              builder: (c, s, _) => Text(s, style: T.micro(N.g63)),
                            ),
                          ),
                        ],
                      ),
                    ),
                  ),
          ),
        ),
      ),
    );
  }
}

WidgetbookComponent glassLiveSet() => WidgetbookComponent(
  name: 'Glass',
  useCases: [
    WidgetbookUseCase(
      name: 'Live',
      builder: (c) => GlassLiveView(
        sunRound: c.knobs.double.slider(label: 'Sun round', initialValue: 13, min: 0, max: 360),
        sunDown: c.knobs.double.slider(label: 'Sun height', initialValue: 24, min: 6, max: 80),
        pace: c.knobs.double.slider(label: 'Pace', initialValue: .5, min: 0, max: 2),
      ),
    ),
  ],
);
