import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import '../../kit.dart';
import '../../tokens.dart';
import '../../parts/controls.dart';
import 'transport_parts.dart';

// set: transport, top bar and global chrome. 14 use cases; every state is a knob.

enum _Kind { play, stop, record }

extension on _Kind {
  Color get accent => this == _Kind.play ? C.play : (this == _Kind.record ? C.record : N.g95);
  G glyph(bool on) => this == _Kind.play ? (on ? G.pause : G.play) : (this == _Kind.stop ? G.stop : G.record);
}

const _fpsOptions = ['23.976', '24', '25', '29.97', '30', '60'];

String _tc(int f, double fps) {
  final r = fps.round();
  String p(int n) => n.toString().padLeft(2, '0');
  final s = f ~/ r;
  return '${p(s ~/ 3600)}:${p((s ~/ 60) % 60)}:${p(s % 60)}:${p(f % r)}';
}

/// Timecode readout: mono, tabular, drag sideways to scrub.
class _Timecode extends StatefulWidget {
  const _Timecode({required this.frame, required this.fps, this.onChanged, this.look = Look.concept, this.force = BtnState.rest});
  final int frame;
  final double fps;
  final Look look;
  final BtnState force;
  final ValueChanged<int>? onChanged;
  @override
  State<_Timecode> createState() => _TimecodeState();
}

class _TimecodeState extends State<_Timecode> {
  bool _h = false, _drag = false;
  double _acc = 0;
  @override
  Widget build(BuildContext context) {
    final hover = _h || _drag || widget.force != BtnState.rest;
    final quiet = widget.look == Look.quiet;
    final parts = _tc(widget.frame, widget.fps).split(':');
    final live = _drag ? C.playhead : N.g95;
    TextStyle s(Color c) => T.value(c).copyWith(fontSize: 14, fontWeight: FontWeight.w500, fontFeatures: const [FontFeature.tabularFigures()]);
    return MouseRegion(
      cursor: SystemMouseCursors.resizeLeftRight,
      onEnter: (_) => setState(() => _h = true),
      onExit: (_) => setState(() => _h = false),
      child: GestureDetector(
        onHorizontalDragStart: (_) => setState(() { _drag = true; _acc = 0; }),
        onHorizontalDragUpdate: (d) {
          _acc += d.delta.dx / 4; // 4 px per frame
          final whole = _acc.truncate();
          if (whole != 0) { _acc -= whole; widget.onChanged?.call((widget.frame + whole).clamp(0, 999999)); }
        },
        onHorizontalDragEnd: (_) => setState(() => _drag = false),
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 120),
          curve: Curves.easeOut,
          height: kCtl,
          padding: const EdgeInsets.symmetric(horizontal: 10),
          decoration: BoxDecoration(
            color: _drag ? N.g07 : (hover ? N.g15 : (quiet ? N.g13.withValues(alpha: 0) : N.g07)),
            borderRadius: BorderRadius.circular(kRad),
            border: Border.all(color: _drag ? C.playhead.withValues(alpha: .55) : (quiet && !hover ? N.g20.withValues(alpha: 0) : (hover ? N.g26 : N.g15))),
          ),
          child: Row(mainAxisSize: MainAxisSize.min, children: [
            Text(parts[0], style: s(N.g38)),
            Text(':', style: s(N.g38)),
            Text(parts[1], style: s(live)),
            Text(':', style: s(N.g56)),
            Text(parts[2], style: s(live)),
            Text(':', style: s(N.g56)),
            Text(parts[3], style: s(_drag ? C.playhead : N.g63)),
          ]),
        ),
      ),
    );
  }
}

class _TransportCluster extends StatelessWidget {
  const _TransportCluster({required this.look, required this.playing, required this.recording, this.onPlay, this.onStop, this.onRec, this.size = kCtl, this.round = false, this.disabled = false});
  final Look look;
  final bool playing, recording, round, disabled;
  final double size;
  final VoidCallback? onPlay, onStop, onRec;
  @override
  Widget build(BuildContext context) => Row(mainAxisSize: MainAxisSize.min, children: [
        TpButton(g: _Kind.play.glyph(playing), accent: C.play, on: playing, look: look, size: size, round: round, enabled: !disabled, onTap: onPlay),
        const SizedBox(width: 4),
        TpButton(g: G.stop, look: look, size: size, round: round, enabled: !disabled, onTap: onStop),
        const SizedBox(width: 4),
        TpButton(g: G.record, accent: C.record, on: recording, look: look, size: size, round: round, enabled: !disabled, onTap: onRec),
      ]);
}

class _UndoRedo extends StatelessWidget {
  const _UndoRedo({required this.canUndo, required this.canRedo, this.look = Look.concept});
  final bool canUndo, canRedo;
  final Look look;
  @override
  Widget build(BuildContext context) => Align(widthFactor: 1, heightFactor: 1, child: Container(
        padding: const EdgeInsets.all(2),
        decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(kRad + 1), border: Border.all(color: N.g15)),
        child: Row(mainAxisSize: MainAxisSize.min, children: [
          TpButton(g: G.undo, look: Look.quiet, size: 24, glyph: 18, idle: N.g91, enabled: canUndo),
          const SizedBox(width: 2),
          TpButton(g: G.redo, look: Look.quiet, size: 24, glyph: 18, idle: N.g91, enabled: canRedo),
        ]),
      ));
}

class _Zoom extends StatelessWidget {
  const _Zoom({required this.zoom, this.look = Look.concept, this.fitActive = false, this.onZoom, this.onFit});
  final int zoom;
  final Look look;
  final bool fitActive;
  final ValueChanged<int>? onZoom;
  final VoidCallback? onFit;
  static const steps = [10, 25, 50, 75, 100, 150, 200, 300, 400];
  int _step(int dir) {
    if (dir < 0) return steps.lastWhere((s) => s < zoom, orElse: () => steps.first);
    return steps.firstWhere((s) => s > zoom, orElse: () => steps.last);
  }

  @override
  Widget build(BuildContext context) => Row(mainAxisSize: MainAxisSize.min, children: [
        Container(
          padding: const EdgeInsets.all(2),
          decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(kRad + 1), border: Border.all(color: N.g15)),
          child: Row(mainAxisSize: MainAxisSize.min, children: [
            TpButton(g: G.minus, look: Look.quiet, size: 24, enabled: zoom > steps.first, onTap: () => onZoom?.call(_step(-1))),
            Pressable(
              onTap: () => onZoom?.call(100),
              builder: (h, d) => AnimatedContainer(
                duration: const Duration(milliseconds: 120),
                curve: Curves.easeOut,
                width: 52,
                height: 24,
                alignment: Alignment.center,
                decoration: BoxDecoration(color: d ? N.g10 : (h ? N.g15 : const Color(0x00000000)), borderRadius: BorderRadius.circular(kRad - 1)),
                child: Text('$zoom%', style: T.value(h ? N.g95 : N.g91).copyWith(fontFeatures: const [FontFeature.tabularFigures()])),
              ),
            ),
            TpButton(g: G.plus, look: Look.quiet, size: 24, enabled: zoom < steps.last, onTap: () => onZoom?.call(_step(1))),
            Container(width: 1, height: 14, margin: const EdgeInsets.symmetric(horizontal: 2), color: N.g20),
            TpButton(g: G.fit, look: Look.quiet, size: 24, on: fitActive, accent: C.playhead, onTap: onFit),
          ]),
        ),
      ]);
}

class _CameraButton extends StatelessWidget {
  const _CameraButton({required this.label, required this.open, this.onTap, this.force = BtnState.rest, this.enabled = true});
  final String label;
  final bool open, enabled;
  final BtnState force;
  final VoidCallback? onTap;
  @override
  Widget build(BuildContext context) => TpChip(label: label, icon: G.camera, chevron: true, open: open, force: force, enabled: enabled, onTap: onTap, minWidth: 128);
}

class _LoopRange extends StatelessWidget {
  const _LoopRange({required this.loop, required this.range, required this.inF, required this.outF, required this.fps, this.look = Look.concept, this.onLoop, this.onRange});
  final bool loop, range;
  final int inF, outF;
  final double fps;
  final Look look;
  final VoidCallback? onLoop, onRange;
  @override
  Widget build(BuildContext context) => Row(mainAxisSize: MainAxisSize.min, children: [
        TpButton(g: G.loop, look: look, on: loop, accent: C.mode, onTap: onLoop),
        const SizedBox(width: 4),
        Pressable(
          onTap: onRange,
          builder: (h, d) {
            final on = range;
            return AnimatedContainer(
              duration: const Duration(milliseconds: 120),
              curve: Curves.easeOut,
              height: kCtl,
              padding: const EdgeInsets.symmetric(horizontal: 10),
              decoration: BoxDecoration(
                color: on ? C.mode.withValues(alpha: d ? .3 : .16) : (d ? N.g10 : (h ? N.g15 : N.g13)),
                borderRadius: BorderRadius.circular(kRad),
                border: Border.all(color: on ? Role.selected.withValues(alpha: .5) : (h ? N.g26 : N.g20)),
              ),
              child: Row(mainAxisSize: MainAxisSize.min, children: [
                Text('RANGE', style: T.micro(on ? C.mode : N.g63).copyWith(letterSpacing: .6)),
                const SizedBox(width: 8),
                Text('${_tc(inF, fps).substring(3)}  →  ${_tc(outF, fps).substring(3)}', style: T.value(on ? N.g95 : N.g56).copyWith(fontSize: 10.5)),
              ]),
            );
          },
        ),
      ]);
}

class _Volume extends StatelessWidget {
  const _Volume({required this.vol, required this.muted, required this.l, required this.r, this.look = Look.concept, this.onVol, this.onMute});
  final double vol, l, r;
  final bool muted;
  final Look look;
  final ValueChanged<double>? onVol;
  final VoidCallback? onMute;
  @override
  Widget build(BuildContext context) => Row(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.center, children: [
        TpButton(g: muted ? G.mute : G.speaker, look: Look.quiet, on: muted, accent: C.record, onTap: onMute),
        const SizedBox(width: 4),
        Column(mainAxisSize: MainAxisSize.min, children: [
          Row(mainAxisSize: MainAxisSize.min, children: [
            TpSlider(value: muted ? 0 : vol, look: look, onChanged: onVol),
            const SizedBox(width: 8),
            SizedBox(width: 28, child: Text(muted ? 'MUTE' : '${(vol * 100).round()}', textAlign: TextAlign.right, maxLines: 1, overflow: TextOverflow.visible, softWrap: false, style: T.value(muted ? N.g56 : N.g91))),
          ]),
          Transform.translate(offset: const Offset(0, -4), child: TpMeter(width: 132, l: muted ? 0 : l * vol, r: muted ? 0 : r * vol, peakL: muted ? null : l * vol + .08, peakR: muted ? null : r * vol + .06)),
        ]),
      ]);
}

class _MenuBar extends StatelessWidget {
  const _MenuBar({required this.open, this.onOpen});
  final int open; // -1 = none
  final ValueChanged<int>? onOpen;
  static const names = ['File', 'Edit', 'Composition', 'Layer', 'Effect', 'View', 'Window', 'Help'];
  @override
  Widget build(BuildContext context) => Container(
        height: 28,
        color: N.g10,
        padding: const EdgeInsets.symmetric(horizontal: 6),
        child: Row(children: [
          for (var i = 0; i < names.length; i++)
            Pressable(
              onTap: () => onOpen?.call(open == i ? -1 : i),
              builder: (h, d) => AnimatedContainer(
                duration: const Duration(milliseconds: 120),
                curve: Curves.easeOut,
                height: 22,
                margin: const EdgeInsets.only(right: 2),
                padding: const EdgeInsets.symmetric(horizontal: 8),
                alignment: Alignment.center,
                decoration: BoxDecoration(color: open == i ? N.g20 : (d ? N.g20 : (h ? N.g15 : const Color(0x00000000))), borderRadius: BorderRadius.circular(4)),
                child: Text(names[i], style: T.name(open == i || h ? N.g95 : N.g76)),
              ),
            ),
        ]),
      );
}

class _TitleBar extends StatefulWidget {
  const _TitleBar({required this.title, required this.dirty, required this.focused});
  final String title;
  final bool dirty, focused;
  @override
  State<_TitleBar> createState() => _TitleBarState();
}

class _TitleBarState extends State<_TitleBar> {
  bool _hover = false;
  @override
  Widget build(BuildContext context) {
    final title = widget.title, dirty = widget.dirty, focused = widget.focused;
    final lit = focused || _hover; // grey at rest; muted tint only when the window is focused or the dots are hovered
    Widget dot(Color c) => AnimatedContainer(duration: const Duration(milliseconds: 120), curve: Curves.easeOut, width: 12, height: 12, margin: const EdgeInsets.only(right: 8), decoration: BoxDecoration(color: lit ? c.withValues(alpha: .6) : N.g38, shape: BoxShape.circle));
    return Container(
      height: 32,
      color: N.g13,
      padding: const EdgeInsets.symmetric(horizontal: 12),
      child: Stack(alignment: Alignment.center, children: [
        Align(alignment: Alignment.centerLeft, child: MouseRegion(onEnter: (_) => setState(() => _hover = true), onExit: (_) => setState(() => _hover = false), child: Row(mainAxisSize: MainAxisSize.min, children: [dot(C.record), dot(Fam.face.c), dot(C.play)]))),
        Row(mainAxisSize: MainAxisSize.min, children: [
          Text(title, style: T.name(focused ? N.g91 : N.g56)),
          if (dirty) ...[const SizedBox(width: 6), Container(width: 6, height: 6, decoration: BoxDecoration(color: focused ? N.g63 : N.g38, shape: BoxShape.circle))],
          const SizedBox(width: 8),
          Text('— Motolii', style: T.label(N.g56)),
        ]),
      ]),
    );
  }
}

class _TopBar extends StatelessWidget {
  const _TopBar({required this.look, required this.playing, required this.rec, required this.frame, required this.fps, required this.mode, required this.onFrame, required this.onMode, required this.onPlay, required this.onRec, required this.loop, required this.onLoop, required this.vol, required this.onVol, required this.canUndo, this.compact = false});
  final Look look;
  final bool playing, rec, loop, canUndo, compact;
  final int frame, mode;
  final double fps, vol;
  final ValueChanged<int> onFrame, onMode;
  final VoidCallback onPlay, onRec, onLoop;
  final ValueChanged<double> onVol;
  @override
  Widget build(BuildContext context) => Container(
        height: 48,
        padding: const EdgeInsets.symmetric(horizontal: 16),
        decoration: const BoxDecoration(color: N.g10, border: Border(bottom: BorderSide(color: N.g20))),
        child: LayoutBuilder(builder: (context, box) {
          // breakpoints measured from the groups' natural widths; the centre groups never shrink, the outer ones drop.
          final w = box.maxWidth;
          final tagline = !compact && w >= 1060;
          final right = w >= 960;
          return Row(children: [
            TpWordmark(tagline: tagline),
            const Spacer(),
            _TransportCluster(look: look, playing: playing, recording: rec, onPlay: onPlay, onStop: () => onFrame(0), onRec: onRec),
            const SizedBox(width: 12),
            _Timecode(frame: frame, fps: fps, look: look, onChanged: onFrame),
            const SizedBox(width: 4),
            TpChip(label: _trim(fps), mono: true, chevron: true),
            const Spacer(),
            TpMode(items: const ['Edit', 'Play', 'Export'], index: mode, look: look, onChanged: onMode),
            const Spacer(),
            _UndoRedo(canUndo: canUndo, canRedo: false, look: look),
            if (right) ...[
              const SizedBox(width: 8),
              TpButton(g: G.loop, look: look, on: loop, accent: C.mode, onTap: onLoop),
              const TpDivider(),
              _Volume(vol: vol, muted: false, l: .62, r: .5, look: look, onVol: onVol),
            ],
          ]);
        }),
      );
}

String _trim(double f) => f == f.roundToDouble() ? '${f.round()}' : f.toString();

WidgetbookComponent transportSet() => WidgetbookComponent(name: 'transport', useCases: [
      uc('Transport buttons / states', (c) {
        final look = lookKnob(c);
        final round = c.knobs.boolean(label: 'Round', initialValue: false);
        const states = ['default', 'hover', 'pressed', 'active', 'disabled'];
        const forces = [BtnState.rest, BtnState.hover, BtnState.pressed, BtnState.rest, BtnState.rest];
        return Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
          for (final k in _Kind.values) ...[
            Row(mainAxisSize: MainAxisSize.min, children: [
              SizedBox(width: 56, child: Text(k.name, style: T.label(N.g56))),
              for (var i = 0; i < 5; i++) Padding(
                padding: const EdgeInsets.only(right: 12),
                child: Column(mainAxisSize: MainAxisSize.min, children: [
                  TpButton(g: k.glyph(false), accent: k.accent, look: look, round: round, size: 36, force: forces[i], on: i == 3 && k != _Kind.stop, enabled: i != 4),
                  const SizedBox(height: 6),
                  Text(states[i], style: T.micro(N.g56)),
                ]),
              ),
            ]),
            const SizedBox(height: 12),
          ],
        ]);
      }, width: 480, height: 280),
      uc('Transport cluster', (c) {
        final look = lookKnob(c);
        return Live<bool>(value: c.knobs.boolean(label: 'Playing', initialValue: false), builder: (play, setPlay) => Live<bool>(
          value: c.knobs.boolean(label: 'Recording', initialValue: false),
          builder: (rec, setRec) => _TransportCluster(
            look: look, playing: play, recording: rec,
            round: c.knobs.boolean(label: 'Round', initialValue: false),
            disabled: c.knobs.boolean(label: 'Disabled', initialValue: false),
            size: c.knobs.int.slider(label: 'Size', initialValue: 32, min: 24, max: 48).toDouble(),
            onPlay: () => setPlay(!play), onStop: () { setPlay(false); setRec(false); }, onRec: () => setRec(!rec),
          ),
        ));
      }, width: 240, height: 96),
      uc('Timecode readout', (c) {
        final fps = double.parse(c.knobs.list<String>(label: 'FPS', options: _fpsOptions, initialOption: '24'));
        return Live<int>(
          value: c.knobs.int.slider(label: 'Frame', initialValue: 61, min: 0, max: 20000),
          builder: (f, set) => Column(mainAxisSize: MainAxisSize.min, children: [
            _Timecode(frame: f, fps: fps, look: lookKnob(c), onChanged: set, force: c.knobs.boolean(label: 'Force hover', initialValue: false) ? BtnState.hover : BtnState.rest),
            const SizedBox(height: 12),
            Text('drag sideways to scrub, 4 px = 1 frame', style: T.label(N.g56)),
          ]),
        );
      }, width: 300, height: 100),
      uc('FPS chip', (c) {
        final opt = c.knobs.list<String>(label: 'FPS', options: _fpsOptions, initialOption: '24');
        final open = c.knobs.boolean(label: 'Open', initialValue: false);
        return Live<bool>(value: open, builder: (o, setO) => Live<String>(value: opt, builder: (v, setV) => Stack(clipBehavior: Clip.none, children: [
          Row(mainAxisSize: MainAxisSize.min, children: [
            TpChip(label: v == '29.97' ? '29.97 DF' : v, mono: true, chevron: true, open: o, onTap: () => setO(!o)),
            const SizedBox(width: 8),
            Text('fps', style: T.label(N.g56)),
          ]),
          if (o) Positioned(left: 0, top: 32, child: TpMenu(
            width: 120, items: _fpsOptions, selected: _fpsOptions.indexOf(v),
            onPick: (i) { setV(_fpsOptions[i]); setO(false); },
          )),
        ])));
      }, width: 200, height: 220),
      uc('Mode switch', (c) => Live<int>(
        value: c.knobs.int.slider(label: 'Mode', initialValue: 0, min: 0, max: 2),
        builder: (i, set) => TpMode(items: const ['Edit', 'Play', 'Export'], index: i, look: lookKnob(c), onChanged: set),
      ), width: 280, height: 80),
      uc('Wordmark block', (c) => Container(
        padding: const EdgeInsets.all(16),
        decoration: BoxDecoration(color: N.g10, borderRadius: BorderRadius.circular(8), border: Border.all(color: N.g20)),
        child: TpWordmark(tagline: c.knobs.boolean(label: 'Tagline', initialValue: true), version: c.knobs.boolean(label: 'Version', initialValue: false) ? 'v0.5.0' : ''),
      ), width: 360, height: 80),
      uc('Menu bar', (c) => Live<int>(
        value: c.knobs.int.slider(label: 'Open menu (-1 none)', initialValue: 0, min: -1, max: 7),
        builder: (o, set) => ClipRect(
          clipBehavior: Clip.none,
          child: Stack(clipBehavior: Clip.none, children: [
            Container(
              decoration: BoxDecoration(color: N.g10, border: Border.all(color: N.g20), borderRadius: BorderRadius.circular(6)),
              child: _MenuBar(open: o, onOpen: set),
            ),
            if (o == 0) Positioned(left: 6, top: 32, child: const TpMenu(
              width: 220,
              items: ['New Composition', 'Open…', 'Open Recent', 'Save', 'Save As…', 'Export…'],
              keys: ['⌘N', '⌘O', '', '⌘S', '⇧⌘S', '⌘E'],
              selected: 3,
            )),
          ]),
        ),
      ), width: 560, height: 240),
      uc('Window title bar', (c) => ClipRRect(
        borderRadius: BorderRadius.circular(8),
        child: _TitleBar(
          title: c.knobs.string(label: 'Title', initialValue: 'Jewel Field'),
          dirty: c.knobs.boolean(label: 'Unsaved', initialValue: true),
          focused: c.knobs.boolean(label: 'Window focused', initialValue: true),
        ),
      ), width: 560, height: 40),
      uc('Zoom control', (c) => Live<int>(
        value: c.knobs.int.slider(label: 'Zoom %', initialValue: 100, min: 10, max: 400),
        builder: (z, set) => _Zoom(zoom: z, look: lookKnob(c), fitActive: c.knobs.boolean(label: 'Fit active', initialValue: false), onZoom: set, onFit: () => set(100)),
      ), width: 240, height: 64),
      uc('Camera view dropdown', (c) {
        const views = ['Camera View', 'Top', 'Front', 'Right', 'Free'];
        return Live<bool>(value: c.knobs.boolean(label: 'Open', initialValue: false), builder: (o, setO) => Live<int>(
          value: c.knobs.int.slider(label: 'View', initialValue: 0, min: 0, max: 4),
          builder: (v, setV) => Stack(clipBehavior: Clip.none, children: [
            _CameraButton(
              label: views[v], open: o, onTap: () => setO(!o),
              enabled: !c.knobs.boolean(label: 'Disabled', initialValue: false),
              force: c.knobs.boolean(label: 'Force hover', initialValue: false) ? BtnState.hover : BtnState.rest,
            ),
            if (o) Positioned(left: 0, top: 32, child: TpMenu(width: 160, items: views, selected: v, onPick: (i) { setV(i); setO(false); })),
          ]),
        ));
      }, width: 220, height: 220),
      uc('Undo / redo pair', (c) => _UndoRedo(canUndo: c.knobs.boolean(label: 'Can undo', initialValue: true), canRedo: c.knobs.boolean(label: 'Can redo', initialValue: false), look: lookKnob(c)), width: 160, height: 56),
      uc('Loop / range toggle', (c) {
        final fps = double.parse(c.knobs.list<String>(label: 'FPS', options: _fpsOptions, initialOption: '24'));
        return Live<bool>(value: c.knobs.boolean(label: 'Loop', initialValue: true), builder: (l, setL) => Live<bool>(
          value: c.knobs.boolean(label: 'Range', initialValue: false),
          builder: (r, setR) => _LoopRange(loop: l, range: r, inF: c.knobs.int.slider(label: 'In (frame)', initialValue: 0, min: 0, max: 600), outF: c.knobs.int.slider(label: 'Out (frame)', initialValue: 96, min: 1, max: 1200), fps: fps, look: lookKnob(c), onLoop: () => setL(!l), onRange: () => setR(!r)),
        ));
      }, width: 320, height: 64),
      uc('Master volume + meter', (c) => Live<double>(
        value: c.knobs.double.slider(label: 'Volume', initialValue: .72, min: 0, max: 1),
        builder: (v, set) => Live<bool>(
          value: c.knobs.boolean(label: 'Muted', initialValue: false),
          builder: (m, setM) => _Volume(vol: v, muted: m, l: c.knobs.double.slider(label: 'Level L', initialValue: .8, min: 0, max: 1), r: c.knobs.double.slider(label: 'Level R', initialValue: .6, min: 0, max: 1), look: lookKnob(c), onVol: set, onMute: () => setM(!m)),
        ),
      ), width: 260, height: 72),
      uc('Top bar (assembled)', (c) {
        final look = lookKnob(c);
        final chrome = c.knobs.boolean(label: 'With title + menu bar', initialValue: false);
        final fps = double.parse(c.knobs.list<String>(label: 'FPS', options: _fpsOptions, initialOption: '24'));
        return Live<bool>(value: c.knobs.boolean(label: 'Playing', initialValue: false), builder: (p, setP) => Live<bool>(
          value: c.knobs.boolean(label: 'Recording', initialValue: false),
          builder: (r, setR) => Live<int>(
            value: c.knobs.int.slider(label: 'Frame', initialValue: 61, min: 0, max: 20000),
            builder: (f, setF) => Live<int>(
              value: c.knobs.int.slider(label: 'Mode', initialValue: 0, min: 0, max: 2),
              builder: (m, setMode) => Live<bool>(
                value: c.knobs.boolean(label: 'Loop', initialValue: false),
                builder: (l, setL) => Live<double>(
                  value: c.knobs.double.slider(label: 'Volume', initialValue: .72, min: 0, max: 1),
                  builder: (v, setV) => ClipRRect(
                    borderRadius: BorderRadius.circular(8),
                    child: Column(mainAxisSize: MainAxisSize.min, children: [
                      if (chrome) ...[
                        const _TitleBar(title: 'Jewel Field', dirty: true, focused: true),
                        const _MenuBar(open: -1),
                      ],
                      _TopBar(
                        look: look, playing: p, rec: r, frame: f, fps: fps, mode: m, loop: l, vol: v,
                        canUndo: c.knobs.boolean(label: 'Can undo', initialValue: true),
                        compact: c.knobs.boolean(label: 'Compact (no tagline)', initialValue: false),
                        onFrame: (n) { setF(n); if (n == 0) { setP(false); setR(false); } },
                        onMode: setMode, onPlay: () => setP(!p), onRec: () => setR(!r), onLoop: () => setL(!l), onVol: setV,
                      ),
                    ]),
                  ),
                ),
              ),
            ),
          ),
        ));
      }, width: double.infinity, height: 140),
    ]);
