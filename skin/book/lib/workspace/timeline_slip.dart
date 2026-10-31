// timeline_slip.dart: a film cut shorter than it is. What is left of it lies pale along its row, and that carried changes
// what the bar shows, the bar staying where it is (After Effects' slip edit: the pale slip-edit bar dragged, In and Out
// stay; Premiere Pro's Slip, its Program Monitor showing the clip meanwhile: so the hand comes to the bar if it was off it).
part of 'timeline.dart';

extension _WsTimelineSlip on _WsTimelineState {
  /// The frames a film lies over whole, as it is now cut (null: no film, or its length not read yet).
  (int, int)? _wholeOf(Ws ws, WsLayer l) {
    final m = ws.inside, t = int.tryParse(l.id);
    if (m == null || t == null || m.word(t, 'kind') != 'film') return null;
    final said = m.word(t, 'file'), file = said.startsWith('/') ? said : '${m.home}/$said';
    final seconds = _filmSeconds[file];
    if (seconds == null) {
      _filmSeconds[file] = 0;
      mediaSecondsOf(m.libPath, file).then((s) => mounted ? setState(() => _filmSeconds[file] = s) : null);
    }
    if (seconds == null || seconds <= 0) return null;
    final rate = Ws.fps / _speedOf(m, t), from = l.inF - (m.value(t, 'begin', 0) ?? 0) * rate;
    return (from.round(), (from + seconds * rate).round());
  }

  double _speedOf(Motolii m, int t) => switch ((m.value(t, 'speed', 0) ?? 1).abs()) { 0 => 1, final s => s };

  void _slip(Ws ws, int t, int by) {
    final whole = _barWhole, m = ws.inside!;
    if (whole == null) return;
    final to = by.clamp(math.min(0, _barOut - whole.$2), math.max(0, _barIn - whole.$1));
    m.carry('begin', (_barIn - whole.$1 - to) * _speedOf(m, t) / Ws.fps);
    setState(() {});
  }
}
