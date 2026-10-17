// The desk's Audio tool: a film's sound as how loud it is through its length (the inside reads it with GStreamer's own
// level, as After Effects' Convert Audio to Keyframes does, on another isolate), and where it strikes: grows so many dB
// louder at once (a drum, a clap), ticks on the same time. Marks puts a mark named beat at each, those it put before
// taken away first: one step. The film is the picked one, else the first that has sound.
import 'package:flutter/widgets.dart';

import '../tokens.dart';
import 'desk_kit.dart';
import 'ws.dart';

class DeskAudio extends StatefulWidget {
  const DeskAudio({super.key});
  @override
  State<DeskAudio> createState() => _DeskAudioState();
}

class _DeskAudioState extends State<DeskAudio> {
  double _rise = 9;
  final _loud = <int, List<double>>{};   // (each film's loudness, once read: read on another isolate)
  final _asked = <int>{};

  List<double>? _of(Ws ws, int t) {
    if (_loud[t] case final l?) return l;
    if (_asked.add(t)) ws.inside!.loudness(t).then((l) => mounted ? setState(() => _loud[t] = l) : null);
    return null;
  }

  @override
  Widget build(BuildContext context) {
    final ws = WsScope.of(context), m = ws.inside;
    if (m == null) return const SizedBox.shrink();
    bool film(String id) => m.word(int.parse(id), 'kind') == 'film';
    // (the picked film, else the first film: its sound read once it is shown here)
    final layer = ws.layers.where((l) => l.id == ws.selected && film(l.id)).firstOrNull ?? ws.layers.where((l) => film(l.id)).firstOrNull;
    final t = layer == null ? null : int.parse(layer.id), loud = t == null ? null : _of(ws, t);
    final ready = t != null && loud != null && loud.isNotEmpty;
    final own = ready ? m.strikes(t, _rise, own: true) : const <double>[];
    void mark() {
      final at = m.strikes(t!, _rise);
      final ok = m.together(() => [
        for (final (at, name) in m.marks) if (name == 'beat') m.mark(at, ''),
        for (final s in at) m.mark((s * Ws.fps).round() / Ws.fps, 'beat'),   // (on a frame, as the timeline counts)
      ].every((ok) => ok));
      if (ok) ws.reread();
    }

    return ListView(
      padding: const EdgeInsets.all(WsT.gap),
      children: [
        DeskStrip(
          left: [
            if (layer != null) DeskLayerChip(layer.name, wsTone(layer.kind)),
            const SizedBox(width: 8),
            SizedBox(
              width: 130,
              child: DeskScrub(key: const ValueKey('audio.rise'), label: 'Rise', value: _rise, min: 3, max: 30, perPx: .1, enabled: ready, onChanged: (v) => setState(() => _rise = v.roundToDouble()), format: (v) => '${v.round()} dB'),
            ),
          ],
          right: [DeskIconButton(key: const ValueKey('audio.marks'), size: 28, glyph: (ink) => _Beats(ink), onTap: () => ready ? mark() : null)],
        ),
        const SizedBox(height: 8),
        Opacity(opacity: t == null || (loud != null && loud.isEmpty) ? .35 : 1, child: SizedBox(height: 78, child: CustomPaint(painter: _Loud(loud ?? const [], own)))),   // (no film, or one with no sound)
      ],
    );
  }
}

/// How loud, through the film's own length (-60 dB at the bottom, full at the top), and where it strikes, ticks on
/// the same time.
class _Loud extends CustomPainter {
  _Loud(this.db, this.strikes);
  final List<double> db, strikes;
  @override
  void paint(Canvas c, Size s) {
    c.drawRRect(RRect.fromRectAndRadius(Offset.zero & s, const Radius.circular(5)), Paint()..color = WsT.well);
    if (db.length < 2) return;
    final step = (db.length / s.width).ceil().clamp(1, 1 << 20);   // (no more points than the strip is wide: the loudest of each)
    final p = Path()..moveTo(0, s.height);
    for (var i = 0; i < db.length; i += step) {
      var v = -100.0;
      for (var k = i; k < i + step && k < db.length; k++) {
        if (db[k] > v) v = db[k];
      }
      p.lineTo(s.width * i / (db.length - 1), s.height * (1 - ((v + 60) / 60).clamp(0.0, 1.0)));
    }
    p
      ..lineTo(s.width, s.height)
      ..close();
    c.drawPath(p, Paint()..color = WsT.toneAudio.withValues(alpha: .75));
    final tick = Paint()
      ..color = Grey.g95
      ..strokeWidth = 1.2;
    for (final a in strikes) {
      final x = s.width * a * 100 / (db.length - 1);
      c.drawLine(Offset(x, s.height - 8), Offset(x, s.height), tick);
    }
  }

  @override
  bool shouldRepaint(_Loud o) => o.db != db || o.strikes.length != strikes.length;
}

/// Marks at the strikes: three ticks under a drum's stroke.
class _Beats extends CustomPainter {
  _Beats(this.ink);
  final Color ink;
  @override
  void paint(Canvas c, Size s) {
    final p = Paint()
      ..color = ink
      ..strokeWidth = 1.5
      ..strokeCap = StrokeCap.round;
    for (final k in [.28, .5, .72]) {
      c.drawLine(Offset(s.width * k, s.height * .3), Offset(s.width * k, s.height * .7), p);
    }
  }

  @override
  bool shouldRepaint(_Beats o) => o.ink != ink;
}
