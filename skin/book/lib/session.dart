// The one shared state of the workflow use cases (research/workflows.md section 3): which layers exist, which are selected, which is hovered,
// and one undo list. It wraps the existing inspector-gui document (GDoc, one per layer) and copies none of its values.
// It is not a store, a reducer, a bus or a persistence layer, and it is not the real Motolii's model. Parts keep their own chrome state
// (a button's hover, a popover); this holds only what two panels must agree on.
import 'dart:async';
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import 'sets/inspector_gui_set.dart' show GDoc;
import 'tokens.dart';
part 'session_layers.dart';
part 'session_picking.dart';
part 'session_playhead.dart';
part 'session_carry.dart';
part 'session_timeline.dart';
part 'session_undo.dart';

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

  _Step? _waveRun;

  /// Show a value on a layer without writing it (no history, nothing to undo): the hover try-on and the hot-swap.
  String? trialLayer;
  final trial = <String, double>{};

  /// What the Inspector edits while several layers are selected (W5): the primary's values, a value that differs across the selection marked mixed.
  final group = GDoc();
  Map<String, double> _gPrev = {}, _pPrev = {};
  Set<String> _gMixedPrev = {};
  String? note; // a short readout of the drag in progress (shown under the Stage)

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
  Timer? _timer;
  final _clock = Stopwatch();
  int _from = 0;

  /// true after a key moved the selection, false after a pointer did: only then the focus ring is drawn (the platform's own rule).
  bool keyRing = false;
  LayerM? _moving;
  Offset _moveLast = Offset.zero;

  static bool _sameKeys(Map<int, Offset> a, Map<int, Offset> b) => a.length == b.length && a.entries.every((e) => b[e.key] == e.value);

  @override
  void dispose() {
    _timer?.cancel();
    super.dispose();
  }

  static bool _near(double a, double b) => (a - b).abs() < 1e-9;

  static bool _differs(Map<String, double> a, Map<String, double> b) => a.entries.any((e) => b[e.key] != e.value);

  void _changed() => notifyListeners();   // (told from its calls kept in the extension beside it: session_2.dart)
}

class SessionScope extends InheritedNotifier<Session> {
  const SessionScope({super.key, required Session session, required super.child}) : super(notifier: session);
  static Session of(BuildContext c) => c.dependOnInheritedWidgetOfExactType<SessionScope>()!.notifier!;
}
