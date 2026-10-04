// The one shared state of the workflow use cases (research/workflows.md section 3): which layers exist, which are selected, which is hovered,
// and one undo list. It wraps the existing inspector-gui document (GDoc, one per layer) and copies none of its values.
// It is not a store, a reducer, a bus or a persistence layer, and it is not the real Motolii's model. Parts keep their own chrome state
// (a button's hover, a popover); this holds only what two panels must agree on.
import 'dart:async';
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import 'sets/inspector_gui_set.dart' show GDoc;
import 'tokens.dart';

/// A phenomenon attached to a layer (the shelf's Wave): it sways the layer along y, evaluated at the playhead. Amplitude in comp px, frequency in Hz.
class Wave {
  const Wave(this.amp, this.freq);
  final double amp, freq;
}

/// One layer of the mock composition: what the Timeline and the list draw, plus the document the Inspector and the Stage edit.
class LayerM {
  LayerM(this.id, this.name, this.fam, this.start, this.end, {required double x, required double y, double rot = 0, double scale = 1}) {
    doc.v
      ..['space'] = 0 // the workflow is 2D; the 2.5D / 3D axes are not part of this round
      ..['pos.x'] = x
      ..['pos.y'] = y
      ..['rot.z'] = rot
      ..['scale.x'] = scale
      ..['scale.y'] = scale
      ..['look'] = -1 // index into the looks shelf, -1 = none
      ..['look.a'] = 50 // a look's two main values
      ..['look.b'] = 50
      ..['anim'] = -1 // index into the in-animations, -1 = none
      ..['ease'] = 2; // index into the eases
    committed = Map.of(doc.v);
  }
  final String id, name;
  final Fam fam;
  double start, end; // whole frames; a bar is moved and trimmed by the timeline gestures
  final GDoc doc = GDoc();
  String kind = 'shape'; // 'text' or 'shape'
  Wave? wave; // the behaviour attached from the shelf, null = none
  String text = 'Your title';

  /// Position keys: frame -> where the layer is at that frame (comp px, centre origin). Linear between keys, held outside them.
  final Map<int, Offset> pk = {};
  List<int> get keyFrames => pk.keys.toList()..sort();

  Offset posAt(int f) {
    final ks = keyFrames;
    if (ks.isEmpty) return Offset(doc['pos.x'], doc['pos.y']);
    if (f <= ks.first) return pk[ks.first]!;
    if (f >= ks.last) return pk[ks.last]!;
    final b = ks.indexWhere((k) => k > f), a = b - 1, t = (f - ks[a]) / (ks[b] - ks[a]);
    return Offset.lerp(pk[ks[a]], pk[ks[b]], t)!;
  }

  /// The values as of the last finished gesture: what an undo of the next gesture goes back to.
  late Map<String, double> committed;
  Map<int, Offset> committedKeys = {};
}

/// A layer's time data as a gesture found it (bar start / end and key frames).
class TlBase {
  TlBase(this.start, this.end, this.pk, [this.values]);
  final double start, end;
  final Map<int, Offset> pk;
  final Map<String, double>? values; // set by a shared-value edit (W5): the layer's document as it was
}

class _Step {
  const _Step(this.layer, this.before, this.beforeKeys)
      : world = null,
        fn = null;
  const _Step.fn(this.fn)
      : layer = null,
        before = const {},
        beforeKeys = const {},
        world = null;
  const _Step.world(this.world)
      : fn = null,
        layer = null,
        before = const {},
        beforeKeys = const {};
  final LayerM? layer;
  final Map<String, double> before;
  final Map<int, Offset> beforeKeys;
  final VoidCallback? fn; // a step that is its own undo (a layer was added)
  final Map<LayerM, TlBase>? world; // a timeline step: every layer's bar and keys as they were
}

class Session extends ChangeNotifier {
  Session({bool seedKeys = false, bool empty = false}) {
    if (empty) layers.clear();
    for (final l in layers) {
      l.doc.addListener(() => _onDoc(l));
    }
    group.addListener(_onGroup);
    if (seedKeys) {
      // a few keys to retime, marquee and snap to (the W4 use case); positions are near where the layer sits
      void k(int i, List<(int, double, double)> ks) {
        final l = layers[i];
        for (final (f, dx, dy) in ks) {
          l.pk[f] = Offset(l.doc['pos.x'] + dx, l.doc['pos.y'] + dy);
        }
        l.committedKeys = Map.of(l.pk);
      }

      k(0, [(10, 0, 0), (40, 160, 60), (70, 300, -40)]);
      k(1, [(4, 0, 0), (36, 120, -140), (60, 260, 0)]);
      k(3, [(0, 0, 0), (34, -200, -120)]);
      _applyAll();
    }
  }

  // ---- R1: layers are made on the Stage; a look, an animation or an ease can be TRIED (shown, not written) and then used (one step) ----

  int _made = 0;

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
    return w == null ? Offset.zero : Offset(0, w.amp * math.sin(2 * math.pi * w.freq * frame / fps));
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
    notifyListeners();
  }

  _Step? _waveRun;

  /// Edit the attached wave's numbers; one scrub is one undo step.
  void setWave(LayerM l, {double? amp, double? freq}) {
    final old = l.wave;
    if (old == null) return;
    if (_undo.isEmpty || !identical(_undo.last, _waveRun)) {
      _waveRun = _Step.fn(() => l.wave = old);
      _undo.add(_waveRun!);
    }
    l.wave = Wave(amp ?? old.amp, freq ?? old.freq);
    notifyListeners();
  }

  void removeWave(LayerM l) {
    final old = l.wave;
    if (old == null) return;
    l.wave = null;
    _undo.add(_Step.fn(() => l.wave = old));
    notifyListeners();
  }

  /// Change what kind of layer this is (text / shape): one undo step.
  void setKind(LayerM l, String kind) {
    final old = l.kind;
    if (old == kind) return;
    l.kind = kind;
    _undo.add(_Step.fn(() => l.kind = old));
    notifyListeners();
  }

  /// Show a value on a layer without writing it (no history, nothing to undo): the hover try-on and the hot-swap.
  String? trialLayer;
  final trial = <String, double>{};
  double val(LayerM l, String id) => trialLayer == l.id && trial.containsKey(id) ? trial[id]! : l.doc[id];

  void tryOn(LayerM l, String id, double v) {
    if (trialLayer != null && trialLayer != l.id) trial.clear();
    trialLayer = l.id;
    trial[id] = v;
    notifyListeners();
  }

  void tryOff() {
    if (trialLayer == null) return;
    trialLayer = null;
    trial.clear();
    notifyListeners();
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

  /// What the Inspector edits while several layers are selected (W5): the primary's values, a value that differs across the selection marked mixed.
  final group = GDoc();
  Map<String, double> _gPrev = {}, _pPrev = {};
  Set<String> _gMixedPrev = {};
  String? note; // a short readout of the drag in progress (shown under the Stage)
  void say(String? t) {
    if (note == t) return;
    note = t;
    notifyListeners();
  }

  final layers = <LayerM>[
    LayerM('l0', 'Scatter', Fam.scatter, 10, 92, x: -620, y: -180),
    LayerM('l1', 'Stagger', Fam.stagger, 4, 118, x: -300, y: 200, rot: 15),
    LayerM('l2', 'Along Path', Fam.along, 14, 118, x: 60, y: -160),
    LayerM('l3', 'Face', Fam.face, 0, 76, x: 380, y: 120, rot: -20),
    LayerM('l4', 'Follow', Fam.follow, 23, 106, x: 640, y: -220),
    LayerM('l5', 'Attach', Fam.attach, 18, 118, x: -80, y: 300, scale: 1.2),
  ];

  final _sel = <String>[]; // in the order picked; the last is the primary (the one the Inspector and the gizmo edit)
  final _undo = <_Step>[];
  String? hovered;

  // ---- time: the playhead is shared, and everything keyed is evaluated at it ----------------------------------------------------
  static const fps = 30, lastFrame = 120;
  int frame = 0;
  bool playing = false;
  final keySel = <String>{}; // picked keys, 'layerId:frame'
  final markers = const [30, 90]; // frames; snap targets and flags on the ruler
  double ppf = 4, scroll = 0; // the timeline's view: pixels per frame, the frame at its left edge (not undoable)
  int? snapFrame; // the frame a drag is snapped to right now (drawn as a line)
  Map<LayerM, TlBase>? _tlBase;
  Set<String> _tlKeySel = {};
  static String kid(LayerM l, int f) => '${l.id}:$f';
  bool keyOn(LayerM l, int f) => keySel.contains(kid(l, f));
  Timer? _timer;
  final _clock = Stopwatch();
  int _from = 0;

  /// true after a key moved the selection, false after a pointer did: only then the focus ring is drawn (the platform's own rule).
  bool keyRing = false;
  LayerM? _moving;
  Offset _moveLast = Offset.zero;

  List<String> get selected => List.unmodifiable(_sel);
  bool isSelected(String id) => _sel.contains(id);
  LayerM? get primary => _sel.isEmpty ? null : byId(_sel.last);
  int get undoSteps => _undo.length;
  LayerM byId(String id) => layers.firstWhere((l) => l.id == id);
  int indexOf(String id) => layers.indexWhere((l) => l.id == id);

  // ---- verbs ---------------------------------------------------------------------------------------------------------------

  /// A plain click picks one; with [add] (Shift / Cmd / Ctrl) it adds or removes one.
  void select(String id, {bool add = false, bool key = false}) {
    if (!add) keySel.clear();
    if (add) {
      _sel.contains(id) ? _sel.remove(id) : _sel.add(id);
    } else {
      _sel
        ..clear()
        ..add(id);
    }
    keyRing = key;
    _syncGroup();
    notifyListeners();
  }

  /// Esc or a click on empty ground.
  void clear() {
    keySel.clear();
    if (_sel.isEmpty) {
      notifyListeners();
      return;
    }
    _sel.clear();
    keyRing = false;
    _syncGroup();
    notifyListeners();
  }

  void hover(String? id) {
    if (id == hovered) return;
    hovered = id;
    notifyListeners();
  }

  /// Tab / Shift-Tab / arrows: the next layer, wrapping; with nothing selected the first (or the last going back).
  void step(int by) {
    final i = primary == null ? (by > 0 ? -1 : 0) : indexOf(primary!.id), n = layers.length;
    select(layers[(i + by) % n].id, key: true);
  }

  /// A press on a box that is not selected yet: select it and carry on as a drag, one gesture (the doc keeps the snapshot).
  void beginMove(LayerM l, Offset at) {
    if (isSelected(l.id) && _sel.length > 1) {
      // a press on a box that is already in the selection drags the whole selection (it becomes the primary, the others follow)
      _sel
        ..remove(l.id)
        ..add(l.id);
      keyRing = false;
      _syncGroup();
      notifyListeners();
    } else {
      select(l.id);
    }
    _moving = l;
    _moveLast = at;
    l.doc.begin();
  }

  /// [k] = composition px per canvas px; Shift is fine (x0.1), the same rule as every other hand.
  void moveTo(Offset at, double k, bool fine) {
    final l = _moving;
    if (l == null) return;
    final dd = (at - _moveLast) * k * (fine ? Gui.fine : 1);
    _moveLast = at;
    l.doc
      ..add('pos.x', dd.dx)
      ..add('pos.y', dd.dy);
  }

  void endMove() {
    _moving?.doc.end();
    _moving = null;
  }

  /// Esc during a drag: everything is where the gesture found it.
  bool cancelMove() {
    if (tlCancel()) return true;
    final l = _moving;
    if (l == null) return false;
    l.doc.cancel();
    _moving = null;
    return true;
  }

  bool get moving => _moving != null;

  // ---- W3: playhead, play, keys ---------------------------------------------------------------------------------------------------

  void seek(int f) {
    frame = f.clamp(0, lastFrame);
    _applyAll();
    notifyListeners();
  }

  void toggle() {
    playing = !playing;
    _timer?.cancel();
    if (playing) {
      _from = frame;
      _clock
        ..reset()
        ..start();
      _timer = Timer.periodic(const Duration(milliseconds: 16), (_) {
        frame = (_from + _clock.elapsedMilliseconds * fps ~/ 1000) % (lastFrame + 1); // loops
        _applyAll();
        notifyListeners();
      });
    } else {
      _clock.stop();
    }
    notifyListeners();
  }

  bool get keyAtHead => primary?.pk.containsKey(frame) ?? false;

  /// The key mark: no key here adds one at the playhead (the layer's current position); a key here removes it. One undo step.
  void toggleKey() {
    final l = primary;
    if (l == null) return;
    _edit(l, () {
      if (l.pk.containsKey(frame)) {
        l.pk.remove(frame);
      } else {
        l.pk[frame] = Offset(l.doc['pos.x'], l.doc['pos.y']);
      }
    });
  }

  /// Delete: every picked key, or else the key under the playhead. One undo step.
  void deleteKey() {
    final doomed = <LayerM, List<int>>{};
    for (final id in keySel) {
      final i = id.indexOf(':');
      doomed.putIfAbsent(byId(id.substring(0, i)), () => []).add(int.parse(id.substring(i + 1)));
    }
    final p = primary;
    if (doomed.isEmpty && p != null && keyAtHead) doomed[p] = [frame];
    if (doomed.isEmpty) return;
    _world(() {
      for (final e in doomed.entries) {
        e.value.forEach(e.key.pk.remove);
      }
      keySel.clear();
    });
  }

  /// A press on a key: it becomes the picked key (with [add]: toggled in the picked set); its layer is selected.
  void pickKey(String id, int f, {bool add = false}) {
    final l = byId(id);
    if (add) {
      if (!isSelected(id)) select(id, add: true);
      final k = kid(l, f);
      keySel.contains(k) ? keySel.remove(k) : keySel.add(k);
    } else {
      select(id);
      keySel.add(kid(l, f));
    }
    notifyListeners();
  }

  // ---- W4: one timeline gesture = begin, any number of sets, then commit (one undo step) or cancel (everything as it was) --------

  bool get tlActive => _tlBase != null;
  Map<LayerM, TlBase> get tlBase => _tlBase!;
  Set<String> get tlKeySel => _tlKeySel;

  void tlBegin() {
    _tlBase = {for (final l in layers) l: TlBase(l.start, l.end, Map.of(l.pk))};
    _tlKeySel = Set.of(keySel);
  }

  void tlSet(LayerM l, {double? start, double? end, Map<int, Offset>? pk, Set<String>? picked}) {
    if (_tlBase == null) return;
    if (start != null) l.start = start;
    if (end != null) l.end = end;
    if (pk != null) {
      l.pk
        ..clear()
        ..addAll(pk);
    }
    if (picked != null) {
      keySel
        ..clear()
        ..addAll(picked);
    }
    _applyAll();
    notifyListeners();
  }

  void snap(int? f) {
    if (snapFrame == f) return;
    snapFrame = f;
    notifyListeners();
  }

  void tlCommit() {
    final b = _tlBase;
    _tlBase = null;
    snapFrame = null;
    note = null;
    if (b != null && _timeChanged(b)) {
      _undo.add(_Step.world(b));
      for (final l in layers) {
        l.committedKeys = Map.of(l.pk);
      }
    }
    notifyListeners();
  }

  bool tlCancel() {
    final b = _tlBase;
    if (b == null) return false;
    _restore(b);
    keySel
      ..clear()
      ..addAll(_tlKeySel);
    _tlBase = null;
    snapFrame = null;
    note = null;
    notifyListeners();
    return true;
  }

  /// The marquee: the layers and keys inside it become the selection (with [add], they join it).
  void marquee(Set<String> layerIds, Set<String> keys, {required bool add}) {
    if (!add) {
      _sel.clear();
      keySel.clear();
    }
    for (final id in layerIds) {
      if (!_sel.contains(id)) _sel.add(id);
    }
    keySel.addAll(keys);
    keyRing = false;
    _syncGroup();
    notifyListeners();
  }

  void view({double? ppf, double? scroll}) {
    if (ppf != null) this.ppf = ppf;
    if (scroll != null) this.scroll = scroll;
    notifyListeners();
  }

  bool _timeChanged(Map<LayerM, TlBase> b) => b.entries.any((e) => e.key.start != e.value.start || e.key.end != e.value.end || !_sameKeys(e.key.pk, e.value.pk));
  static bool _sameKeys(Map<int, Offset> a, Map<int, Offset> b) => a.length == b.length && a.entries.every((e) => b[e.key] == e.value);

  void _restore(Map<LayerM, TlBase> b) {
    for (final e in b.entries) {
      e.key.start = e.value.start;
      e.key.end = e.value.end;
      e.key.pk
        ..clear()
        ..addAll(e.value.pk);
      e.key.committedKeys = Map.of(e.value.pk);
      final v = e.value.values;
      if (v != null) {
        e.key.doc.v
          ..clear()
          ..addAll(v);
        e.key.committed = Map.of(v);
      }
    }
    _applyAll();
    _syncGroup();
  }

  /// A change to bars or keys made outside a drag (Delete): snapshot, change, one step.
  void _world(VoidCallback change) {
    final b = {for (final l in layers) l: TlBase(l.start, l.end, Map.of(l.pk))};
    change();
    if (_timeChanged(b)) {
      _undo.add(_Step.world(b));
      for (final l in layers) {
        l.committedKeys = Map.of(l.pk);
      }
      _applyAll();
    }
    notifyListeners();
  }

  void _edit(LayerM l, VoidCallback change) {
    final before = Map.of(l.committed), beforeKeys = Map.of(l.committedKeys);
    change();
    _undo.add(_Step(l, before, beforeKeys));
    l.committedKeys = Map.of(l.pk);
    _applyAll();
    notifyListeners();
  }

  /// Every keyed layer takes its value at the playhead (not a user edit: no undo step, no key written).
  void _applyAll() {
    for (final l in layers) {
      if (l.pk.isEmpty || l.doc.live) continue;
      final p = l.posAt(frame);
      l.doc.v
        ..['pos.x'] = p.dx
        ..['pos.y'] = p.dy;
      l.committed
        ..['pos.x'] = p.dx
        ..['pos.y'] = p.dy;
    }
    if (_sel.length > 1 && !group.live) _syncGroup();
  }

  @override
  void dispose() {
    _timer?.cancel();
    super.dispose();
  }

  /// One step back: the last finished gesture, whichever panel it came from.
  bool undo() {
    if (_undo.isEmpty) return false;
    final s = _undo.removeLast();
    keySel.clear();
    if (s.fn != null) {
      s.fn!();
      _syncGroup();
      notifyListeners();
      return true;
    }
    if (s.world != null) {
      _restore(s.world!);
      notifyListeners();
      return true;
    }
    final layer = s.layer!;
    layer.doc.v
      ..clear()
      ..addAll(s.before);
    layer.pk
      ..clear()
      ..addAll(s.beforeKeys);
    layer.committed = Map.of(s.before);
    layer.committedKeys = Map.of(s.beforeKeys);
    if (layer == primary) _pPrev = Map.of(s.before);
    layer.doc.poke();
    _syncGroup();
    return true;
  }

  // ---- the one undo list: any finished change of a layer's document is a step, however it was made ----------------------------

  // ---- W5: several layers selected. A scrub adds the same delta to each, a typed value sets the absolute value on all, one gesture is one step ----

  List<LayerM> get _selLayers => [for (final id in _sel) byId(id)];
  static bool _near(double a, double b) => (a - b).abs() < 1e-9;

  void _syncGroup() {
    final p = primary;
    if (_sel.length < 2 || p == null || group.live) return;
    final ts = _selLayers;
    group.v
      ..clear()
      ..addAll(p.doc.v);
    group.mixed
      ..clear()
      ..addAll([for (final id in p.doc.v.keys) if (ts.any((l) => !_near(l.doc.v[id] ?? 0, p.doc.v[id]!))) id]);
    _gPrev = Map.of(group.v);
    _gMixedPrev = Set.of(group.mixed);
    _pPrev = Map.of(p.doc.v);
  }

  /// An edit made in the Inspector's fields (or its pad, dial, handles) while several layers are selected.
  void _onGroup() {
    final ts = _selLayers;
    if (ts.length < 2) return;
    for (final e in group.v.entries) {
      final old = _gPrev[e.key] ?? e.value, was = _gMixedPrev.contains(e.key), now = group.mixed.contains(e.key);
      if (_near(old, e.value) && was == now) continue;
      for (final l in ts) {
        l.doc.v[e.key] = (was && !now) ? e.value : (l.doc.v[e.key] ?? 0) + (e.value - old);
      }
    }
    _gPrev = Map.of(group.v);
    _gMixedPrev = Set.of(group.mixed);
    if (!group.live) {
      _commitGroup(ts);
      _syncGroup();
    }
    notifyListeners();
  }

  /// An edit made on the Stage gizmo, which edits the primary's own document: every other selected layer receives the same delta.
  void _fromPrimary(LayerM p) {
    final ts = _selLayers;
    for (final e in p.doc.v.entries) {
      final old = _pPrev[e.key] ?? e.value;
      if (_near(old, e.value)) continue;
      for (final l in ts) {
        if (l != p) l.doc.v[e.key] = (l.doc.v[e.key] ?? 0) + (e.value - old);
      }
    }
    _pPrev = Map.of(p.doc.v);
    if (!p.doc.live) {
      _commitGroup(ts);
      _syncGroup();
    }
    notifyListeners();
  }

  void _commitGroup(List<LayerM> ts) {
    final changed = ts.any((l) => l.doc.v.entries.any((e) => !_near(e.value, l.committed[e.key] ?? e.value)));
    if (!changed) {
      for (final l in ts) {
        l.doc.v
          ..clear()
          ..addAll(l.committed); // an Esc leaves exactly what was there, not what arithmetic gives back
      }
      return;
    }
    final before = {for (final l in ts) l: TlBase(l.start, l.end, Map.of(l.committedKeys), Map.of(l.committed))};
    for (final l in ts) {
      if (l.pk.isNotEmpty && (!_near(l.doc['pos.x'], l.committed['pos.x']!) || !_near(l.doc['pos.y'], l.committed['pos.y']!))) l.pk[frame] = Offset(l.doc['pos.x'], l.doc['pos.y']);
    }
    _undo.add(_Step.world(before));
    for (final l in ts) {
      l.committed = Map.of(l.doc.v);
      l.committedKeys = Map.of(l.pk);
    }
  }

  void _onDoc(LayerM l) {
    if (_sel.length > 1 && l == primary) {
      _fromPrimary(l);
      return;
    }
    if (!l.doc.live && _differs(l.doc.v, l.committed)) {
      // a keyed layer edited at the playhead writes its key there (the first key is the key mark's job): the gesture and its key are one step
      if (l.pk.isNotEmpty && (l.doc['pos.x'] != l.committed['pos.x'] || l.doc['pos.y'] != l.committed['pos.y'])) l.pk[frame] = Offset(l.doc['pos.x'], l.doc['pos.y']);
      _undo.add(_Step(l, l.committed, l.committedKeys));
      l.committed = Map.of(l.doc.v);
      l.committedKeys = Map.of(l.pk);
    }
    notifyListeners();
  }

  static bool _differs(Map<String, double> a, Map<String, double> b) => a.entries.any((e) => b[e.key] != e.value);
}

class SessionScope extends InheritedNotifier<Session> {
  const SessionScope({super.key, required Session session, required super.child}) : super(notifier: session);
  static Session of(BuildContext c) => c.dependOnInheritedWidgetOfExactType<SessionScope>()!.notifier!;
}
