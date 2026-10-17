// The desk's Audio tool: a film's sound as how loud it is through its length (the inside reads it with GStreamer's own
// level, as After Effects' Convert Audio to Keyframes does), and where it strikes: grows so many dB louder at once (a
// drum, a clap), shown as ticks along the document's time. Marks puts a mark named beat at each, those it put before
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
  (int, int, double, List<double>, List<double>)? _kept;   // (thing, document change, rise: its loudness and strikes)

  @override
  Widget build(BuildContext context) {
    final ws = WsScope.of(context), m = ws.inside;
    if (m == null) return const SizedBox.shrink();
    bool film(String id) => m.word(int.parse(id), 'kind') == 'film';
    final picked = ws.selected != null && film(ws.selected!) ? ws.selected : null;
    final layer = ws.layers.where((l) => l.id == picked).firstOrNull ??
        ws.layers.where((l) => film(l.id) && (_kept?.$1 == int.parse(l.id) ? _kept!.$4.isNotEmpty : m.loudness(int.parse(l.id)).isNotEmpty)).firstOrNull;
    final t = layer == null ? null : int.parse(layer.id);
    if (t != null && (_kept?.$1 != t || _kept?.$2 != ws.rev || _kept?.$3 != _rise)) _kept = (t, ws.rev, _rise, m.loudness(t), m.strikes(t, _rise));
    final (loud, strikes) = t == null ? (const <double>[], const <double>[]) : (_kept!.$4, _kept!.$5);
    void mark() {
      final ok = m.together(() => [
        for (final (at, name) in m.marks) if (name == 'beat') m.mark(at, ''),
        for (final at in strikes) m.mark(at, 'beat'),
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
              child: DeskScrub(key: const ValueKey('audio.rise'), label: 'Rise', value: _rise, min: 3, max: 30, perPx: .1, enabled: t != null, onChanged: (v) => setState(() => _rise = v.roundToDouble()), format: (v) => '${v.round()} dB'),
            ),
          ],
          right: [DeskIconButton(key: const ValueKey('audio.marks'), size: 28, glyph: (ink) => _Beats(ink), onTap: () => strikes.isEmpty ? null : mark())],
        ),
        const SizedBox(height: 8),
        SizedBox(height: 70, child: CustomPaint(painter: _Loud(loud))),
        const SizedBox(height: 4),
        SizedBox(height: 18, child: CustomPaint(painter: _Ticks(strikes, m.seconds, ws.frame / Ws.fps))),
      ],
    );
  }
}

/// How loud, through the film's own length: -60 dB at the bottom, full at the top.
class _Loud extends CustomPainter {
  _Loud(this.db);
  final List<double> db;
  @override
  void paint(Canvas c, Size s) {
    c.drawRRect(RRect.fromRectAndRadius(Offset.zero & s, const Radius.circular(5)), Paint()..color = WsT.well);
    if (db.length < 2) return;
    final p = Path()..moveTo(0, s.height);
    for (final (i, v) in db.indexed) {
      p.lineTo(s.width * i / (db.length - 1), s.height * (1 - ((v + 60) / 60).clamp(0.0, 1.0)));
    }
    p
      ..lineTo(s.width, s.height)
      ..close();
    c.drawPath(p, Paint()..color = WsT.toneAudio.withValues(alpha: .75));
  }

  @override
  bool shouldRepaint(_Loud o) => o.db != db;
}

/// Where it strikes, along the document's time; the playhead a line.
class _Ticks extends CustomPainter {
  _Ticks(this.at, this.seconds, this.now);
  final List<double> at;
  final double seconds, now;
  @override
  void paint(Canvas c, Size s) {
    c.drawRRect(RRect.fromRectAndRadius(Offset.zero & s, const Radius.circular(4)), Paint()..color = WsT.well);
    final tick = Paint()
      ..color = Grey.g91
      ..strokeWidth = 1.2;
    for (final a in at) {
      final x = s.width * a / seconds;
      c.drawLine(Offset(x, 3), Offset(x, s.height - 3), tick);
    }
    final x = s.width * now / seconds;
    c.drawLine(Offset(x, 0), Offset(x, s.height), Paint()
      ..color = WsT.accent
      ..strokeWidth = 1);
  }

  @override
  bool shouldRepaint(_Ticks o) => true;
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
