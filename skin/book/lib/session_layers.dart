// session_layers.dart: the layers themselves: one added, its wave, its kind, a value tried on and kept, a value set, a readout said.
part of 'session.dart';

extension SessionLayers on Session {
  /// A new layer at a position on the stage (comp px, centre origin), selected. One undo step (undo removes it). null when the six rows are full.
  LayerM? addLayer(String kind, String name, double x, double y, {int start = 0, int end = 90}) {
    if (layers.length >= 6) return null;
    final l = LayerM('n${_made++}', name, Fam.all[(_made + 1) % Fam.all.length], start.toDouble(), end.toDouble(), x: x, y: y)..kind = kind;
    if (kind == 'text') l.text = name;
    l.doc.addListener(() => _onDoc(l));
    layers.add(l);
    _undo.add(_Step.fn(() {
      layers.remove(l);
      _sel.remove(l.id);
    }));
    select(l.id);
    return l;
  }

  // ---- behaviour shelf: a phenomenon is picked from the shelf, attached to a layer, and its numbers are read and edited in the Inspector ----

  /// Where the layer is drawn relative to where it sits (comp px): the attached wave at the playhead.
  Offset waveShift(LayerM l) {
    final w = l.wave;
    return w == null ? Offset.zero : Offset(0, w.amp * math.sin(2 * math.pi * w.freq * frame / Session.fps));
  }

  /// Attach a Wave to a layer (one undo step). Attaching again keeps the numbers it has.
  void attachWave(LayerM l) {
    if (l.wave != null) {
      select(l.id);
      return;
    }
    l.wave = const Wave(80, 1);
    _undo.add(_Step.fn(() => l.wave = null));
    select(l.id);
    _changed();
  }

  /// Edit the attached wave's numbers; one scrub is one undo step.
  void setWave(LayerM l, {double? amp, double? freq}) {
    final old = l.wave;
    if (old == null) return;
    if (_undo.isEmpty || !identical(_undo.last, _waveRun)) {
      _waveRun = _Step.fn(() => l.wave = old);
      _undo.add(_waveRun!);
    }
    l.wave = Wave(amp ?? old.amp, freq ?? old.freq);
    _changed();
  }

  void removeWave(LayerM l) {
    final old = l.wave;
    if (old == null) return;
    l.wave = null;
    _undo.add(_Step.fn(() => l.wave = old));
    _changed();
  }

  /// Change what kind of layer this is (text / shape): one undo step.
  void setKind(LayerM l, String kind) {
    final old = l.kind;
    if (old == kind) return;
    l.kind = kind;
    _undo.add(_Step.fn(() => l.kind = old));
    _changed();
  }

  double val(LayerM l, String id) => trialLayer == l.id && trial.containsKey(id) ? trial[id]! : l.doc[id];

  void tryOn(LayerM l, String id, double v) {
    if (trialLayer != null && trialLayer != l.id) trial.clear();
    trialLayer = l.id;
    trial[id] = v;
    _changed();
  }

  void tryOff() {
    if (trialLayer == null) return;
    trialLayer = null;
    trial.clear();
    _changed();
  }

  /// Write what is being tried: one undo step.
  void useTrial() {
    final l = trialLayer == null ? null : byId(trialLayer!);
    if (l == null) return;
    final t = Map.of(trial);
    tryOff();
    setProp(l, t);
  }

  /// Write values on a layer outside a drag: one undo step (the layer's document does the bookkeeping).
  void setProp(LayerM l, Map<String, double> vals) {
    l.doc.v.addAll(vals);
    l.doc.poke();
  }

  void say(String? t) {
    if (note == t) return;
    note = t;
    _changed();
  }

  LayerM byId(String id) => layers.firstWhere((l) => l.id == id);

  int indexOf(String id) => layers.indexWhere((l) => l.id == id);
}
