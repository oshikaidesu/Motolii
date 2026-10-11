// R1 "Make a title and play it" (research/experience-first.md section D). A RUN, not a component: an empty Stage -> a text layer + a look + an
// in-animation -> Space. Browser | Stage | Inspector on top, the Timeline below. One knob, Mode: Habit baseline (only AE / Ableton / Blender
// motions) or Habit + additions (Cmd-K quick-add, main values first, try-on-your-own-frame by hover and Q + Up/Down, ease hover preview).
// Decisions already taken (options-browser.md D-1..D-4, D-9; options-inspector.md D-I3, D-I5): a click selects, a double-click or Enter uses, a drag
// places; a trial is shown and never written until used; Esc puts it back; scrub right adds, Shift x10, Alt x0.1.
// Prototype for the Widgetbook, not Motolii's architecture; the shared state is lib/session.dart.
import 'dart:async';
import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import '../../parts/glyphs.dart';
import '../../session.dart';
import '../../tokens.dart';
import '../inspector_gui_set.dart';
import '../inspector/inspector_parts.dart' show Chip, KeyMark, KeyS;
import 'workflow_set.dart';
import 'workflow_timeline.dart';

enum R1Mode { baseline, additions }

WidgetbookComponent runSet() => WidgetbookComponent(name: 'Run', useCases: [
      WidgetbookUseCase(
        name: 'R1 Make a title and play it',
        builder: (c) => ColoredBox(
          color: N.g07,
          child: R1Scene(c.knobs.object.dropdown<R1Mode>(label: 'Mode', options: R1Mode.values, initialOption: R1Mode.additions, labelBuilder: (m) => m == R1Mode.baseline ? 'Habit baseline' : 'Habit + additions')),
        ),
      ),
    ]);

// ---- the shelf -------------------------------------------------------------------------------------------------------------------

enum AKind { object, look, motion, ease }

class Asset {
  const Asset(this.kind, this.idx, this.name, [this.words = '']);
  final AKind kind;
  final int idx;
  final String name, words;
  String get group => switch (kind) { AKind.object => 'Objects', AKind.look => 'Looks', AKind.motion => 'Motion', AKind.ease => 'Eases' };
  bool matches(String q) => q.isEmpty || '$name $words $group'.toLowerCase().contains(q.toLowerCase());
}

const _objects = [Asset(AKind.object, 0, 'Text', 'title type font'), Asset(AKind.object, 1, 'Rectangle', 'shape box'), Asset(AKind.object, 2, 'Ellipse', 'shape circle')];
const _looks = [
  Asset(AKind.look, 0, 'Plain', 'none clean'),
  Asset(AKind.look, 1, 'Neon', 'glow light sign'),
  Asset(AKind.look, 2, 'Shadow', 'drop soft'),
  Asset(AKind.look, 3, 'Outline', 'stroke line'),
  Asset(AKind.look, 4, 'Gold', 'metal shiny'),
  Asset(AKind.look, 5, 'Chrome', 'metal silver'),
  Asset(AKind.look, 6, 'Glitch', 'split rgb'),
  Asset(AKind.look, 7, 'Soft blur', 'dream'),
  Asset(AKind.look, 8, 'Retro', '80s synth'),
  Asset(AKind.look, 9, 'Ice', 'cold frost'),
  Asset(AKind.look, 10, 'Fire', 'hot flame'),
  Asset(AKind.look, 11, 'Emboss', 'relief'),
];
const _motions = [
  Asset(AKind.motion, 0, 'Fade in', 'opacity appear'),
  Asset(AKind.motion, 1, 'Slide up', 'rise move'),
  Asset(AKind.motion, 2, 'Pop', 'scale bounce'),
  Asset(AKind.motion, 3, 'Typewriter', 'type letters'),
  Asset(AKind.motion, 4, 'Blur in', 'focus'),
];
const _easeNames = ['Linear', 'Ease in', 'Ease out', 'Ease in-out', 'Back', 'Bounce'];
const _easeCurves = <Curve>[Curves.linear, Curves.easeIn, Curves.easeOut, Curves.easeInOut, Curves.easeOutBack, Curves.bounceOut];
final _eases = [for (var i = 0; i < 6; i++) Asset(AKind.ease, i, _easeNames[i], 'curve motion')];
final _all = [..._objects, ..._looks, ..._motions, ..._eases];

// ---- drawing a title (the look, the in-animation) -----------------------------------------------------------------------------------

({double alpha, double dy, double scale, double chars, double blur}) _animAt(Session s, LayerM l) {
  final anim = s.val(l, 'anim').round();
  if (anim < 0) return (alpha: 1, dy: 0, scale: 1, chars: 1, blur: 0);
  final t = ((s.frame - l.start) / 30).clamp(0.0, 1.0), p = _easeCurves[s.val(l, 'ease').round().clamp(0, 5)].transform(t);
  return switch (anim) {
    0 => (alpha: p.clamp(0.0, 1.0), dy: 0, scale: 1, chars: 1, blur: 0),
    1 => (alpha: p.clamp(0.0, 1.0), dy: (1 - p) * 26, scale: 1, chars: 1, blur: 0),
    2 => (alpha: (p * 3).clamp(0.0, 1.0), dy: 0, scale: .4 + .6 * p, chars: 1, blur: 0),
    3 => (alpha: 1, dy: 0, scale: 1, chars: p.clamp(0.0, 1.0), blur: 0),
    _ => (alpha: p.clamp(0.0, 1.0), dy: 0, scale: 1, chars: 1, blur: (1 - p.clamp(0.0, 1.0)) * 6),
  };
}

void _drawTitle(Canvas c, Offset ctr, String text, double size, {required int look, double a = 50, double b = 50, double alpha = 1, double dy = 0, double scale = 1, double chars = 1, double blur = 0, double rot = 0}) {
  if (text.isEmpty || alpha <= 0) return;
  TextStyle st({Color? color, Paint? fg, List<Shadow>? sh}) => TextStyle(fontFamily: T.sans, fontSize: size, fontWeight: FontWeight.w700, color: fg == null ? color : null, foreground: fg, shadows: sh, height: 1, decoration: TextDecoration.none);
  final m = TextPainter(text: TextSpan(text: text, style: st(color: N.g95)), textDirection: TextDirection.ltr)..layout();
  final w = m.width, h = m.height;
  m.dispose();
  final shown = text.substring(0, (text.length * chars).ceil().clamp(0, text.length)), origin = Offset(-w / 2, -h / 2);
  c.save();
  c.translate(ctr.dx, ctr.dy + dy);
  c.rotate(rot);
  c.scale(scale);
  c.saveLayer(Rect.fromCenter(center: Offset.zero, width: w + 120, height: h + 120), Paint()..color = N.g100.withValues(alpha: alpha.clamp(0.0, 1.0))..imageFilter = blur > 0 ? ui.ImageFilter.blur(sigmaX: blur, sigmaY: blur) : null);
  void put(TextStyle s, [Offset off = Offset.zero]) {
    final tp = TextPainter(text: TextSpan(text: shown, style: s), textDirection: TextDirection.ltr)..layout();
    tp.paint(c, origin + off);
    tp.dispose();
  }

  Paint grad(List<Color> cs) => Paint()..shader = ui.Gradient.linear(origin, origin + Offset(0, h), cs, [for (var i = 0; i < cs.length; i++) i / (cs.length - 1)]);
  switch (look) {
    case 1:
      put(st(color: Fam.scatter.c, sh: [Shadow(color: Fam.scatter.c, blurRadius: a * .4), Shadow(color: Fam.scatter.c, blurRadius: a * .2)]));
      put(st(color: N.g100.withValues(alpha: .85)));
    case 2:
      put(st(color: N.g95, sh: [Shadow(color: N.g00.withValues(alpha: .8), blurRadius: a / 10, offset: Offset(b / 10, b / 10))]));
    case 3:
      put(st(fg: Paint()..style = PaintingStyle.stroke..strokeWidth = 1 + b / 40..color = N.g95));
    case 4:
      put(st(fg: grad([Fam.face.c, Fam.follow.c, Fam.face.c]), sh: [Shadow(color: N.g00.withValues(alpha: .5), blurRadius: 3, offset: const Offset(1, 2))]));
    case 5:
      put(st(fg: grad([N.g100, N.g56, N.g95, N.g38])));
    case 6:
      put(st(color: Fam.scatter.c.withValues(alpha: .85)), Offset(-b / 18, 0));
      put(st(color: Fam.stagger.c.withValues(alpha: .85)), Offset(b / 18, 0));
      put(st(color: N.g95.withValues(alpha: .9)));
    case 7:
      put(st(fg: Paint()..color = N.g95..maskFilter = MaskFilter.blur(BlurStyle.normal, .1 + a / 14)));
    case 8:
      put(st(color: Fam.stagger.c), Offset(b / 12, b / 12));
      put(st(fg: grad([Fam.scatter.c, Fam.face.c])));
    case 9:
      put(st(fg: grad([N.g100, Fam.stagger.c]), sh: [Shadow(color: Fam.stagger.c, blurRadius: a * .2)]));
    case 10:
      put(st(fg: grad([Fam.face.c, Fam.follow.c, Fam.scatter.c]), sh: [Shadow(color: Fam.follow.c, blurRadius: a * .25)]));
    case 11:
      put(st(color: N.g100.withValues(alpha: .6)), const Offset(-1, -1));
      put(st(color: N.g00.withValues(alpha: .6)), const Offset(1, 1));
      put(st(color: N.g76));
    default:
      put(st(color: N.g95));
  }
  c.restore();
  c.restore();
}

class _LookSwatch extends CustomPainter {
  const _LookSwatch(this.look, this.size);
  final int look;
  final double size;
  @override
  void paint(Canvas c, Size s) => _drawTitle(c, s.center(Offset.zero), 'Aa', size, look: look);
  @override
  bool shouldRepaint(_LookSwatch o) => o.look != look;
}

class _CurvePlot extends CustomPainter {
  const _CurvePlot(this.curve, this.color);
  final Curve curve;
  final Color color;
  @override
  void paint(Canvas c, Size s) {
    final p = Path();
    for (var i = 0; i <= 28; i++) {
      final t = i / 28, v = curve.transform(t).clamp(-.3, 1.3), pt = Offset(t * s.width, s.height * (1 - (v + .3) / 1.6));
      i == 0 ? p.moveTo(pt.dx, pt.dy) : p.lineTo(pt.dx, pt.dy);
    }
    c.drawPath(p, Paint()..style = PaintingStyle.stroke..strokeWidth = 1.5..color = color);
  }

  @override
  bool shouldRepaint(_CurvePlot o) => o.curve != curve || o.color != color;
}

// ---- the run: what only this use case owns ---------------------------------------------------------------------------------------------

class Run extends ChangeNotifier {
  Run(this.s);
  final Session s;
  R1Mode mode = R1Mode.additions;
  bool get plus => mode == R1Mode.additions;

  // browser
  int tab = 0;
  String query = '';
  Asset? selAsset;
  Asset? dragging;
  bool cancelDrag = false;
  String? hint;

  // drop feedback on the stage
  String? dropLayer; // the layer outlined
  bool dropStage = false; // the whole stage outlined (a new layer will be added)
  String dropLabel = '';

  // palette and hot-swap (additions)
  bool palette = false, hot = false;
  String pq = '';
  int hotIdx = 0;
  bool played = false;

  // the counter: gestures and surface switches
  int gestures = 0, switches = 0;
  String? lastSurface;

  void touch(String surface, {bool gesture = true}) {
    if (gesture) gestures++;
    if (lastSurface != null && lastSurface != surface) switches++;
    lastSurface = surface;
    notifyListeners();
  }

  void key() {
    gestures++;
    notifyListeners();
  }

  void resetCounter() {
    gestures = 0;
    switches = 0;
    lastSurface = null;
    notifyListeners();
  }

  void poke() => notifyListeners();

  // ---- using an asset: double-click, Enter, a palette pick -------------------------------------------------------------------------

  LayerM? get target => s.primary ?? (s.layers.isEmpty ? null : s.layers.last);

  void use(Asset a) {
    hint = null;
    switch (a.kind) {
      case AKind.object:
        s.addLayer(a.idx == 0 ? 'text' : 'shape', a.idx == 0 ? 'Your title' : a.name, 0, -160.0 + 110 * s.layers.length);
      case AKind.look:
        final l = target;
        if (l == null || l.kind != 'text') {
          hint = 'A look needs a text layer: make one with the Text object first.';
        } else {
          s.setProp(l, {'look': a.idx.toDouble(), 'look.a': 50, 'look.b': 50});
        }
      case AKind.motion:
        final l = target;
        if (l == null) {
          hint = 'Make a layer first, then add the motion to it.';
        } else {
          s.setProp(l, {'anim': a.idx.toDouble()});
          peek(l, once: true);
        }
      case AKind.ease:
        final l = target;
        if (l == null) {
          hint = 'Make a layer first.';
        } else {
          s.setProp(l, {'ease': a.idx.toDouble()});
          peek(l, once: true);
        }
    }
    notifyListeners();
  }

  // ---- a trial: shown, never written until used ------------------------------------------------------------------------------------

  void tryAsset(Asset a) {
    final l = target;
    if (l == null) return;
    switch (a.kind) {
      case AKind.look:
        if (l.kind == 'text') s.tryOn(l, 'look', a.idx.toDouble());
      case AKind.motion:
        s.tryOn(l, 'anim', a.idx.toDouble());
        peek(l);
      case AKind.ease:
        s.tryOn(l, 'ease', a.idx.toDouble());
        peek(l);
      case AKind.object:
        break;
    }
    if (s.trialLayer != null) s.say('Trying ${a.name}  (not saved: Enter uses it, Esc puts it back)');
  }

  void endTrial() {
    if (s.trialLayer == null && _peek == null) return;
    s.tryOff();
    stopPeek();
    s.say(null);
  }

  void confirmTrial() {
    s.useTrial();
    stopPeek();
    s.say(null);
  }

  // ---- peek: play the layer's first second while a motion or ease is being tried ----------------------------------------------------------

  Timer? _peek;
  int _saved = 0;
  void peek(LayerM l, {bool once = false}) {
    if (s.playing) return;
    if (_peek == null) _saved = s.frame;
    _peek?.cancel();
    var f = l.start.round();
    _peek = Timer.periodic(const Duration(milliseconds: 33), (t) {
      s.seek(f++);
      if (f > l.start + 42) {
        if (once) {
          stopPeek(restore: false);
        } else {
          f = l.start.round();
        }
      }
    });
  }

  void stopPeek({bool restore = true}) {
    if (_peek == null) return;
    _peek!.cancel();
    _peek = null;
    if (restore) s.seek(_saved);
  }

  // ---- hot-swap (Q): the Browser is tied to the selected layer; Up / Down try the next look; Enter uses it; Esc puts it back -------------------------

  void startHot() {
    final l = target;
    if (l == null || l.kind != 'text') {
      hint = 'Q hot-swaps looks on a text layer: select one first.';
      notifyListeners();
      return;
    }
    hot = true;
    tab = 1;
    hotIdx = math.max(0, s.val(l, 'look').round());
    notifyListeners();
  }

  void hotMove(int by) {
    hotIdx = (hotIdx + by) % _looks.length;
    tryAsset(_looks[hotIdx]);
    notifyListeners();
  }

  void endHot({required bool use}) {
    if (use) {
      confirmTrial();
    } else {
      endTrial();
    }
    hot = false;
    notifyListeners();
  }

  // ---- drop (the stage is the target) ------------------------------------------------------------------------------------------------

  LayerM? layerAt(Map<String, List<Offset>> corners, Offset p) {
    for (final l in s.layers.reversed) {
      final c = corners[l.id];
      if (c == null) continue;
      final o = (c[0] + c[2]) / 2, u = c[1] - c[0], v = c[3] - c[0], d = p - o;
      final ux = u / math.max(u.distance, 1), vx = v / math.max(v.distance, 1);
      if ((d.dx * ux.dx + d.dy * ux.dy).abs() <= u.distance / 2 + 6 && (d.dx * vx.dx + d.dy * vx.dy).abs() <= v.distance / 2 + 6) return l;
    }
    return null;
  }

  /// Which result a drop of [a] at [p] would have, for the outline before release. Returns (layer or null, stage?, label); null = invalid, no highlight.
  (LayerM?, bool, String)? aim(Asset a, Offset p, Map<String, List<Offset>> corners) {
    final l = layerAt(corners, p), alt = HardwareKeyboard.instance.isAltPressed;
    switch (a.kind) {
      case AKind.object:
        if (l != null && alt) return (l, false, 'Replace ${l.name} (Alt)');
        return (null, true, 'Add ${a.name} layer here${l != null ? '  (Alt replaces ${l.name})' : ''}');
      case AKind.look:
        if (l == null || l.kind != 'text') return null;
        return (l, false, l.doc['look'] >= 0 ? 'Replace look on ${l.name}' : 'Add look to ${l.name}');
      case AKind.motion:
        if (l == null) return null;
        return (l, false, l.doc['anim'] >= 0 ? 'Replace motion on ${l.name}' : 'Add motion to ${l.name}');
      case AKind.ease:
        if (l == null) return null;
        return (l, false, 'Set ease on ${l.name}');
    }
  }

  void drop(Asset a, Offset p, Size size, double k, Map<String, List<Offset>> corners) {
    final tgt = aim(a, p, corners);
    if (tgt == null) return;
    final l = tgt.$1;
    switch (a.kind) {
      case AKind.object:
        if (l != null) {
          s.select(l.id);
          s.setKind(l, a.idx == 0 ? 'text' : 'shape');
        } else {
          final at = (p - size.center(Offset.zero)) * k;
          s.addLayer(a.idx == 0 ? 'text' : 'shape', a.idx == 0 ? 'Your title' : a.name, at.dx.clamp(-900.0, 900.0), at.dy.clamp(-500.0, 500.0));
        }
      case AKind.look:
        s.select(l!.id);
        s.setProp(l, {'look': a.idx.toDouble(), 'look.a': 50, 'look.b': 50});
      case AKind.motion:
        s.select(l!.id);
        s.setProp(l, {'anim': a.idx.toDouble()});
        peek(l, once: true);
      case AKind.ease:
        s.select(l!.id);
        s.setProp(l, {'ease': a.idx.toDouble()});
        peek(l, once: true);
    }
    touch('Stage', gesture: false);
  }

  @override
  void dispose() {
    _peek?.cancel();
    super.dispose();
  }
}

// ---- the scene --------------------------------------------------------------------------------------------------------------------

class R1Scene extends StatefulWidget {
  const R1Scene(this.mode, {super.key});
  final R1Mode mode;
  @override
  State<R1Scene> createState() => _R1SceneState();
}

class _R1SceneState extends State<R1Scene> {
  late final session = Session(empty: true);
  late final run = Run(session);
  final focus = FocusNode(debugLabel: 'r1');
  final canvasKey = GlobalKey();

  @override
  void initState() {
    super.initState();
    GuiRule.shiftBig = true; // D-I3: scrub right adds, Shift x10, Alt x0.1
    session.addListener(() {
      if (session.playing && !run.played) run.played = true;
    });
  }

  @override
  void dispose() {
    GuiRule.shiftBig = false;
    run.dispose();
    focus.dispose();
    session.dispose();
    super.dispose();
  }

  KeyEventResult _key(FocusNode _, KeyEvent e) {
    if (e is! KeyDownEvent && e is! KeyRepeatEvent) return KeyEventResult.ignored;
    if (FocusManager.instance.primaryFocus?.context?.findAncestorWidgetOfExactType<EditableText>() != null) return KeyEventResult.ignored;
    final k = e.logicalKey, hk = HardwareKeyboard.instance;
    if (k == LogicalKeyboardKey.escape) {
      run.key();
      if (run.dragging != null) {
        run.cancelDrag = true;
        run.dropLayer = null;
        run.dropStage = false;
        run.poke();
        return KeyEventResult.handled;
      }
      if (run.hot) {
        run.endHot(use: false);
        return KeyEventResult.handled;
      }
      if (session.trialLayer != null) {
        run.endTrial();
        return KeyEventResult.handled;
      }
      if (session.cancelMove()) return KeyEventResult.handled;
      if (session.selected.isEmpty) return KeyEventResult.ignored;
      session.clear();
      return KeyEventResult.handled;
    }
    if (run.hot) {
      if (k == LogicalKeyboardKey.arrowDown || k == LogicalKeyboardKey.arrowUp) {
        run.hotMove(k == LogicalKeyboardKey.arrowDown ? 1 : -1);
        return KeyEventResult.handled;
      }
      if (k == LogicalKeyboardKey.enter) {
        run.key();
        run.endHot(use: true);
        return KeyEventResult.handled;
      }
    }
    if (k == LogicalKeyboardKey.space) {
      run.key();
      run.stopPeek(restore: false);
      session.toggle();
      return KeyEventResult.handled;
    }
    if (k == LogicalKeyboardKey.enter && run.selAsset != null) {
      run.key();
      run.use(run.selAsset!);
      return KeyEventResult.handled;
    }
    if (k == LogicalKeyboardKey.keyQ && run.plus) {
      run.key();
      run.startHot();
      return KeyEventResult.handled;
    }
    if (k == LogicalKeyboardKey.delete || k == LogicalKeyboardKey.backspace) {
      session.deleteKey();
      return KeyEventResult.handled;
    }
    if (k == LogicalKeyboardKey.home) {
      session.seek(0);
      return KeyEventResult.handled;
    }
    if (k == LogicalKeyboardKey.tab) {
      session.step(hk.isShiftPressed ? -1 : 1);
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  @override
  Widget build(BuildContext context) {
    run.mode = widget.mode;
    return SessionScope(
      session: session,
      child: ListenableBuilder(
        listenable: run,
        builder: (context, _) => CallbackShortcuts(
          bindings: {
            const SingleActivator(LogicalKeyboardKey.keyZ, meta: true): () => _undo(),
            const SingleActivator(LogicalKeyboardKey.keyZ, control: true): () => _undo(),
            if (run.plus) const SingleActivator(LogicalKeyboardKey.keyK, meta: true): _openPalette,
            if (run.plus) const SingleActivator(LogicalKeyboardKey.keyK, control: true): _openPalette,
          },
          child: Focus(
            focusNode: focus,
            autofocus: true,
            onKeyEvent: _key,
            child: Stack(children: [
              Column(children: [
                _Chrome(run),
                Container(height: 1, color: N.g20),
                Expanded(
                  child: Row(children: [
                    SizedBox(width: 300, child: Listener(onPointerDown: (_) => run.touch('Browser'), child: _Browser(run, focus))),
                    Container(width: 1, color: N.g20),
                    Expanded(child: Listener(onPointerDown: (_) => run.touch('Stage'), child: _stage())),
                    Container(width: 1, color: N.g20),
                    SizedBox(width: 282, child: Listener(onPointerDown: (_) => run.touch('Inspector'), child: _R1Inspector(run))),
                  ]),
                ),
                Container(height: 1, color: N.g20),
                SizedBox(
                  height: TL.ruler + TL.height * 6 + 24,
                  child: Listener(onPointerDown: (_) => run.touch('Timeline'), child: WorkTimeline(focus: focus, knobs: const TlKnobs())),
                ),
              ]),
              if (run.palette) _Palette(run, () => _closePalette()),
            ]),
          ),
        ),
      ),
    );
  }

  void _undo() {
    run.key();
    session.undo();
  }

  void _openPalette() {
    run.key();
    run.palette = true;
    run.pq = '';
    run.poke();
  }

  void _closePalette() {
    run.palette = false;
    run.endTrial();
    focus.requestFocus();
    run.poke();
  }

  Widget _stage() => WorkStage(
        focus,
        wrap: (size, k, canvas) => DragTarget<Asset>(
          onWillAcceptWithDetails: (_) => !run.cancelDrag,
          onMove: (d) {
            final box = canvasKey.currentContext?.findRenderObject() as RenderBox?;
            if (box == null || run.cancelDrag) return;
            final corners = {for (final l in session.layers) l.id: gizmoCorners(l.doc, size)};
            final tgt = run.aim(d.data, box.globalToLocal(d.offset), corners);
            final layer = tgt?.$1?.id, stageOn = tgt?.$2 ?? false, label = tgt?.$3 ?? '';
            if (layer != run.dropLayer || stageOn != run.dropStage || label != run.dropLabel) {
              run.dropLayer = layer;
              run.dropStage = stageOn;
              run.dropLabel = label;
              run.poke();
            }
          },
          onLeave: (_) {
            run.dropLayer = null;
            run.dropStage = false;
            run.poke();
          },
          onAcceptWithDetails: (d) {
            final box = canvasKey.currentContext?.findRenderObject() as RenderBox?;
            if (box == null || run.cancelDrag) return;
            final corners = {for (final l in session.layers) l.id: gizmoCorners(l.doc, size)};
            run.drop(d.data, box.globalToLocal(d.offset), size, k, corners);
            run.dropLayer = null;
            run.dropStage = false;
            run.poke();
          },
          builder: (_, _, _) => KeyedSubtree(key: canvasKey, child: canvas),
        ),
        above: (size, k, corners) => [
          Positioned.fill(child: IgnorePointer(child: CustomPaint(painter: _Content(session, size, k)))),
          Positioned.fill(
            child: IgnorePointer(
              child: TweenAnimationBuilder<double>(
                key: ValueKey('${run.dropLayer}/${run.dropStage}'),
                tween: Tween(begin: 0, end: 1),
                duration: Mo.dur,
                curve: Mo.ease,
                builder: (_, t, _) => CustomPaint(painter: _DropPaint(run.dropLayer == null ? null : corners[run.dropLayer], run.dropStage, run.dropLabel, t)),
              ),
            ),
          ),
        ],
      );
}

/// The work itself: every layer drawn at the playhead, with its look and in-animation (tried values included).
class _Content extends CustomPainter {
  const _Content(this.s, this.size, this.k);
  final Session s;
  final Size size;
  final double k;
  @override
  void paint(Canvas c, Size sz) {
    for (final l in s.layers) {
      final ctr = size.center(Offset.zero) + Offset(l.doc['pos.x'], l.doc['pos.y']) / k, sc = l.doc['scale.x'], rot = l.doc['rot.z'] * math.pi / 180;
      if (l.kind == 'text') {
        final an = _animAt(s, l);
        _drawTitle(c, ctr, l.text, 30 * sc, look: s.val(l, 'look').round(), a: s.val(l, 'look.a'), b: s.val(l, 'look.b'), alpha: an.alpha, dy: an.dy, scale: an.scale, chars: an.chars, blur: an.blur, rot: rot);
      } else {
        final an = _animAt(s, l);
        c.save();
        c.translate(ctr.dx, ctr.dy + an.dy);
        c.rotate(rot);
        c.scale(sc * an.scale);
        final r = Rect.fromCenter(center: Offset.zero, width: 56, height: 36), p = Paint()..color = l.fam.c.withValues(alpha: .85 * an.alpha.clamp(0.0, 1.0));
        l.name == 'Ellipse' ? c.drawOval(r, p) : c.drawRRect(RRect.fromRectAndRadius(r, const Radius.circular(4)), p);
        c.restore();
      }
    }
  }

  @override
  bool shouldRepaint(_Content o) => true;
}

/// The drop target's outline before release, 1 px accent, eased in by the one motion constant; invalid targets draw nothing.
class _DropPaint extends CustomPainter {
  const _DropPaint(this.corners, this.stage, this.label, this.t);
  final List<Offset>? corners;
  final bool stage;
  final String label;
  final double t;
  @override
  void paint(Canvas c, Size s) {
    final line = Paint()..style = PaintingStyle.stroke..strokeWidth = 1..color = Role.selected.withValues(alpha: t);
    Offset? at;
    if (corners != null) {
      c.drawPath(Path()..addPolygon([for (final p in corners!) p], true), line..strokeWidth = 2);
      at = corners![0] + const Offset(0, -18);
    } else if (stage) {
      c.drawRect((Offset.zero & s).deflate(1.5), line..strokeWidth = 1.5);
      at = const Offset(10, 10);
    }
    if (at == null || label.isEmpty) return;
    final tp = TextPainter(text: TextSpan(text: label, style: T.label(N.g95)), textDirection: TextDirection.ltr, maxLines: 1)..layout();
    final o = Offset(at.dx.clamp(4.0, math.max(4.0, s.width - tp.width - 10)), at.dy.clamp(4.0, s.height - tp.height - 6));
    c.drawRRect(RRect.fromRectAndRadius((o & tp.size).inflate(3), const Radius.circular(3)), Paint()..color = N.g10.withValues(alpha: .95 * t));
    tp.paint(c, o);
    tp.dispose();
  }

  @override
  bool shouldRepaint(_DropPaint o) => true;
}

// ---- the story chrome: goal, mode, the counter ------------------------------------------------------------------------------------------

class _Chrome extends StatelessWidget {
  const _Chrome(this.run);
  final Run run;
  @override
  Widget build(BuildContext context) {
    final s = run.s;
    Widget done(String t, bool on) => Padding(
          padding: const EdgeInsets.only(right: 10),
          child: Row(mainAxisSize: MainAxisSize.min, children: [
            Container(width: 8, height: 8, decoration: BoxDecoration(shape: BoxShape.circle, color: on ? C.play : null, border: Border.all(color: on ? C.play : N.g44))),
            const SizedBox(width: 4),
            Text(t, style: T.label(on ? N.g95 : N.g56)),
          ]),
        );
    return Container(
      height: 34,
      color: N.g10,
      padding: const EdgeInsets.symmetric(horizontal: 12),
      child: Row(children: [
        Text('R1  Make a title and play it', style: T.name()),
        const SizedBox(width: 14),
        Text(run.plus ? 'Habit + additions' : 'Habit baseline', style: T.label(run.plus ? C.mode : N.g63)),
        const SizedBox(width: 16),
        done('text layer', s.layers.any((l) => l.kind == 'text')),
        done('look', s.layers.any((l) => l.doc['look'] >= 0)),
        done('in-animation', s.layers.any((l) => l.doc['anim'] >= 0)),
        done('played', run.played),
        const Spacer(),
        Text('${run.gestures} gestures · ${run.switches} surface switches', style: T.value(N.g95)),
        const SizedBox(width: 10),
        GestureDetector(behavior: HitTestBehavior.opaque, onTap: run.resetCounter, child: Padding(padding: const EdgeInsets.symmetric(vertical: 8), child: Text('Reset', style: T.name(N.g76)))),
      ]),
    );
  }
}

// ---- the browser --------------------------------------------------------------------------------------------------------------------------

class _Browser extends StatefulWidget {
  const _Browser(this.run, this.focus);
  final Run run;
  final FocusNode focus;
  @override
  State<_Browser> createState() => _BrowserState();
}

class _BrowserState extends State<_Browser> {
  final ctrl = TextEditingController(), node = FocusNode(debugLabel: 'search');
  Run get run => widget.run;

  @override
  void initState() {
    super.initState();
    ctrl.addListener(() {
      if (ctrl.text != run.query) {
        run.query = ctrl.text;
        run.poke();
      }
    });
  }

  @override
  void dispose() {
    ctrl.dispose();
    node.dispose();
    super.dispose();
  }

  List<Asset> get _items {
    if (run.query.isNotEmpty) return [for (final a in _all) if (a.matches(run.query)) a];
    return switch (run.tab) { 0 => _objects, 1 => _looks, 2 => _motions, _ => _eases };
  }

  @override
  Widget build(BuildContext context) {
    final items = _items, sel = run.selAsset;
    return Container(
      color: N.g10,
      padding: const EdgeInsets.all(12),
      child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        // search: a fixed field at the top, over every shelf (Ableton)
        Focus(
          onKeyEvent: (_, e) {
            if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape) {
              ctrl.clear();
              widget.focus.requestFocus();
              return KeyEventResult.handled;
            }
            return KeyEventResult.ignored;
          },
          child: Container(
            height: 28,
            padding: const EdgeInsets.symmetric(horizontal: 8),
            decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(4), border: Border.all(color: N.g20)),
            child: Stack(alignment: Alignment.centerLeft, children: [
              ValueListenableBuilder<TextEditingValue>(valueListenable: ctrl, builder: (_, v, _) => v.text.isEmpty ? Text('Search objects, looks, motion, eases', style: T.label(N.g56)) : const SizedBox.shrink()),
              EditableText(controller: ctrl, focusNode: node, style: T.name(N.g95), cursorColor: Role.selected, backgroundCursorColor: N.g20, maxLines: 1),
            ]),
          ),
        ),
        const SizedBox(height: 8),
        Wrap(spacing: 2, children: [
          for (final (i, n) in const ['Objects', 'Looks', 'Motion', 'Eases'].indexed)
            Chip(n, on: run.query.isEmpty && run.tab == i, onTap: () {
              ctrl.clear();
              run.tab = i;
              run.poke();
            }),
        ]),
        const SizedBox(height: 8),
        SizedBox(
          height: 18,
          child: run.hot
              ? Text('Hot-swap on the selected layer: Up / Down try, Enter uses, Esc puts it back', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(Role.selected))
              : (run.query.isNotEmpty
                  ? Row(children: [
                      Text('${items.length} results for "${run.query}"', style: T.label(N.g63)),
                      const Spacer(),
                      GestureDetector(behavior: HitTestBehavior.opaque, onTap: ctrl.clear, child: Text('Clear', style: T.name(N.g95))),
                    ])
                  : Text(run.tab == 0 ? 'Drag onto the Stage, or double-click' : 'Click selects, double-click uses, drag onto a layer', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g56))),
        ),
        Expanded(
          child: SingleChildScrollView(
            child: Wrap(spacing: 8, runSpacing: 8, children: [
              for (final a in items) _Tile(run, a, selected: sel == a, hot: run.hot && a.kind == AKind.look && a.idx == run.hotIdx, label: run.query.isNotEmpty),
            ]),
          ),
        ),
        const SizedBox(height: 8),
        Container(height: 1, color: N.g15),
        SizedBox(height: 82, child: _Preview(run)),
      ]),
    );
  }
}

class _Tile extends StatefulWidget {
  const _Tile(this.run, this.a, {required this.selected, required this.hot, required this.label});
  final Run run;
  final Asset a;
  final bool selected, hot, label;
  @override
  State<_Tile> createState() => _TileState();
}

class _TileState extends State<_Tile> {
  bool hover = false;
  Timer? dwell;
  Run get run => widget.run;
  Asset get a => widget.a;

  @override
  void dispose() {
    dwell?.cancel();
    super.dispose();
  }

  Widget _face({double w = 86}) {
    final inner = switch (a.kind) {
      AKind.object => Glyph(const [G.text, G.rect, G.ellipse][a.idx], size: 22, color: N.g76),
      AKind.look => SizedBox(width: 40, height: 28, child: CustomPaint(painter: _LookSwatch(a.idx, 22))),
      AKind.motion => Glyph(G.effect, size: 22, color: N.g76),
      AKind.ease => SizedBox(width: 40, height: 24, child: CustomPaint(painter: _CurvePlot(_easeCurves[a.idx], N.g95))),
    };
    final on = widget.selected || widget.hot;
    return AnimatedContainer(
      duration: Mo.dur,
      curve: Mo.ease,
      width: w,
      height: 64,
      decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(8), border: Border.all(color: on ? Role.selected : (hover ? N.g38 : N.g20))),
      child: Column(mainAxisAlignment: MainAxisAlignment.center, children: [
        inner,
        const SizedBox(height: 5),
        Text(widget.label ? '${a.name} · ${a.group}' : a.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g76)),
      ]),
    );
  }

  @override
  Widget build(BuildContext context) => MouseRegion(
        cursor: SystemMouseCursors.click,
        onEnter: (_) {
          setState(() => hover = true);
          if (run.plus && run.dragging == null && (a.kind == AKind.look || a.kind == AKind.motion || a.kind == AKind.ease)) {
            dwell?.cancel();
            dwell = Timer(const Duration(milliseconds: 180), () => run.tryAsset(a)); // try it on the selected layer's own frame; nothing is written
          }
        },
        onExit: (_) {
          setState(() => hover = false);
          dwell?.cancel();
          if (run.plus && !run.hot && run.dragging == null) run.endTrial();
        },
        child: Draggable<Asset>(
          data: a,
          dragAnchorStrategy: pointerDragAnchorStrategy,
          feedback: ListenableBuilder(
            listenable: run,
            builder: (_, _) => run.cancelDrag ? const SizedBox.shrink() : Transform.translate(offset: const Offset(-35, -24), child: Opacity(opacity: .7, child: _face(w: 70))),
          ),
          childWhenDragging: Opacity(opacity: .4, child: _face()),
          onDragStarted: () {
            dwell?.cancel();
            run.endTrial();
            run.dragging = a;
            run.cancelDrag = false;
            run.poke();
          },
          onDragEnd: (_) {
            run.dragging = null;
            run.cancelDrag = false;
            run.dropLayer = null;
            run.dropStage = false;
            run.poke();
          },
          child: GestureDetector(
            behavior: HitTestBehavior.opaque,
            onTap: () {
              run.selAsset = a;
              run.poke();
            },
            onDoubleTap: () {
              run.selAsset = a;
              dwell?.cancel();
              run.endTrial();
              run.use(a);
            },
            child: _face(),
          ),
        ),
      );
}

/// What the selected tile is, bigger: the look on a title, the curve, the motion's name.
class _Preview extends StatelessWidget {
  const _Preview(this.run);
  final Run run;
  @override
  Widget build(BuildContext context) {
    final a = run.selAsset;
    if (a == null) {
      return Align(alignment: Alignment.centerLeft, child: Text(run.hint ?? 'Select a tile to see it here.', style: T.label(run.hint == null ? N.g56 : N.g95).copyWith(height: 1.4)));
    }
    return Row(children: [
      Container(
        width: 96,
        height: 62,
        decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(4), border: Border.all(color: N.g20)),
        child: switch (a.kind) {
          AKind.look => CustomPaint(painter: _LookSwatch(a.idx, 28)),
          AKind.ease => Padding(padding: const EdgeInsets.all(10), child: CustomPaint(painter: _CurvePlot(_easeCurves[a.idx], N.g95))),
          AKind.motion => Center(child: Text(a.name, style: T.name())),
          AKind.object => Center(child: Glyph(const [G.text, G.rect, G.ellipse][a.idx], size: 28, color: N.g76)),
        },
      ),
      const SizedBox(width: 10),
      Expanded(child: Text('${a.name}\n${a.group}  ·  double-click or Enter uses it${run.hint != null ? '\n${run.hint}' : ''}', maxLines: 4, overflow: TextOverflow.ellipsis, style: T.label(N.g76).copyWith(height: 1.4))),
    ]);
  }
}

// ---- Cmd-K: quick-add (additions) ----------------------------------------------------------------------------------------------------------

class _Palette extends StatefulWidget {
  const _Palette(this.run, this.close);
  final Run run;
  final VoidCallback close;
  @override
  State<_Palette> createState() => _PaletteState();
}

class _PaletteState extends State<_Palette> {
  final ctrl = TextEditingController(), node = FocusNode(debugLabel: 'palette');
  int idx = 0;
  Run get run => widget.run;

  List<Asset> get _res => [for (final a in _all) if (a.matches(ctrl.text)) a].take(8).toList();

  @override
  void initState() {
    super.initState();
    ctrl.addListener(() {
      idx = 0;
      _try();
      setState(() {});
    });
    WidgetsBinding.instance.addPostFrameCallback((_) => node.requestFocus());
  }

  void _try() {
    final r = _res;
    if (r.isEmpty) {
      run.endTrial();
    } else {
      run.tryAsset(r[idx.clamp(0, r.length - 1)]);
    }
  }

  @override
  void dispose() {
    ctrl.dispose();
    node.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final r = _res;
    return Positioned(
      top: 44,
      left: 0,
      right: 0,
      child: Center(
        child: Focus(
          onKeyEvent: (_, e) {
            if (e is! KeyDownEvent && e is! KeyRepeatEvent) return KeyEventResult.ignored;
            final k = e.logicalKey;
            if (k == LogicalKeyboardKey.escape) {
              widget.close();
              return KeyEventResult.handled;
            }
            if (k == LogicalKeyboardKey.arrowDown || k == LogicalKeyboardKey.arrowUp) {
              setState(() => idx = r.isEmpty ? 0 : (idx + (k == LogicalKeyboardKey.arrowDown ? 1 : r.length - 1)) % r.length);
              _try();
              return KeyEventResult.handled;
            }
            if (k == LogicalKeyboardKey.enter && r.isNotEmpty) {
              final a = r[idx.clamp(0, r.length - 1)];
              run.key();
              run.palette = false;
              run.endTrial();
              run.use(a);
              widget.close();
              return KeyEventResult.handled;
            }
            return KeyEventResult.ignored;
          },
          child: Container(
            width: 380,
            padding: const EdgeInsets.all(8),
            decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(8), border: Border.all(color: N.g26), boxShadow: [BoxShadow(color: N.g00.withValues(alpha: .5), blurRadius: 16)]),
            child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
              Container(
                height: 28,
                padding: const EdgeInsets.symmetric(horizontal: 8),
                alignment: Alignment.centerLeft,
                decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(4), border: Border.all(color: N.g20)),
                child: Stack(alignment: Alignment.centerLeft, children: [
                  if (ctrl.text.isEmpty) Text('Type a look, motion, ease or object', style: T.label(N.g56)),
                  EditableText(controller: ctrl, focusNode: node, style: T.name(N.g95), cursorColor: Role.selected, backgroundCursorColor: N.g20, maxLines: 1),
                ]),
              ),
              const SizedBox(height: 6),
              for (final (i, a) in r.indexed)
                Container(
                  height: 24,
                  padding: const EdgeInsets.symmetric(horizontal: 8),
                  color: i == idx ? N.g20 : null,
                  child: Row(children: [
                    Expanded(child: Text(a.name, style: T.name(i == idx ? N.g100 : N.g91))),
                    Text(a.group, style: T.label(N.g56)),
                  ]),
                ),
              const SizedBox(height: 4),
              Text('Up / Down try it on your layer (not saved) · Enter uses · Esc puts it back', style: T.label(N.g56)),
            ]),
          ),
        ),
      ),
    );
  }
}

// ---- the inspector ------------------------------------------------------------------------------------------------------------------------

class _R1Inspector extends StatelessWidget {
  const _R1Inspector(this.run);
  final Run run;

  Widget _head(String t) => Padding(padding: const EdgeInsets.only(bottom: 6), child: Text(t, style: T.micro(N.g56).copyWith(letterSpacing: .8)));

  Widget _value(String label, GDoc d, String id) => Padding(
        padding: const EdgeInsets.only(bottom: 6),
        child: Row(children: [
          SizedBox(width: 70, child: Text(label, style: T.label(N.g63))),
          Expanded(child: XyzStack(d, [Spec(id, '·', per: .5)])),
        ]),
      );

  @override
  Widget build(BuildContext context) {
    final s = SessionScope.of(context), l = s.primary, n = s.selected.length;
    Widget body;
    if (l == null) {
      body = Text(s.layers.isEmpty ? 'Empty. Drag Text from the Browser onto the Stage (or double-click it).' : 'Pick a layer on the Stage or in the Timeline.', style: T.label(N.g56).copyWith(height: 1.4));
    } else if (n > 1) {
      body = Text('$n layers selected. Select one to edit it.', style: T.label(N.g56).copyWith(height: 1.4));
    } else {
      final isText = l.kind == 'text';
      final transform = KeyedSubtree(key: ValueKey('t${l.id}'), child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [_head('TRANSFORM'), TransformBoth(l.doc), const SizedBox(height: 12)]));
      final text = isText ? KeyedSubtree(key: ValueKey('x${l.id}'), child: _TextBox(l, s)) : const SizedBox.shrink();
      final look = !isText
          ? const SizedBox.shrink()
          : Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
              _head('LOOK'),
              Padding(
                padding: const EdgeInsets.only(bottom: 8),
                child: Text(s.val(l, 'look') < 0 ? 'None. Double-click or drag a look from the Browser.' : _looks[s.val(l, 'look').round()].name, style: T.name(s.val(l, 'look') < 0 ? N.g56 : N.g95)),
              ),
              if (s.val(l, 'look') >= 0) ...[_value('Amount', l.doc, 'look.a'), _value('Spread', l.doc, 'look.b')],
              const SizedBox(height: 10),
            ]);
      final anim = Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        _head('IN ANIMATION'),
        for (var i = -1; i < _motions.length; i++)
          _PickRow(
            name: i < 0 ? 'None' : _motions[i].name,
            on: l.doc['anim'].round() == i,
            onTap: () => s.setProp(l, {'anim': i.toDouble()}),
            onHover: run.plus && i >= 0 ? (h) => h ? run.tryAsset(_motions[i]) : run.endTrial() : null,
          ),
        const SizedBox(height: 8),
        _head('EASE'),
        for (var i = 0; i < _easeNames.length; i++)
          _PickRow(
            name: _easeNames[i],
            on: l.doc['ease'].round() == i,
            curve: _easeCurves[i],
            onTap: () {
              run.endTrial();
              s.setProp(l, {'ease': i.toDouble()});
              run.peek(l, once: true);
            },
            onHover: run.plus ? (h) => h ? run.tryAsset(_eases[i]) : run.endTrial() : null,
          ),
      ]);
      // the one difference in the Inspector between the modes: where the main values sit
      body = Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: run.plus ? [text, look, anim, const SizedBox(height: 12), transform] : [transform, text, look, anim]);
    }
    return Container(
      color: N.g10,
      padding: const EdgeInsets.all(12),
      child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        Text(l == null ? 'NOTHING SELECTED' : (n > 1 ? 'SEVERAL SELECTED' : (l.kind == 'text' ? 'TEXT LAYER' : 'SHAPE LAYER')), style: T.micro(N.g56).copyWith(letterSpacing: .8)),
        if (l != null) ...[const SizedBox(height: 4), Text(n > 1 ? '$n layers' : l.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.title())],
        const SizedBox(height: 10),
        Container(height: 1, color: N.g15),
        if (l != null && n == 1) ...[
          const SizedBox(height: 8),
          Row(children: [
            Text('POSITION KEY', style: T.micro(N.g56).copyWith(letterSpacing: .8)),
            const Spacer(),
            KeyMark(state: l.pk.isEmpty ? KeyS.off : (s.keyAtHead ? KeyS.at : KeyS.anim), onTap: s.toggleKey),
          ]),
        ],
        const SizedBox(height: 10),
        Expanded(child: SingleChildScrollView(child: body)),
        Container(height: 1, color: N.g15),
        Row(children: [
          Text(s.undoSteps == 1 ? '1 step' : '${s.undoSteps} steps', style: T.value(N.g76)),
          const SizedBox(width: 12),
          GestureDetector(behavior: HitTestBehavior.opaque, onTap: s.undo, child: Padding(padding: const EdgeInsets.symmetric(vertical: 8), child: Text('Undo', style: T.name(s.undoSteps == 0 ? N.g56 : N.g95)))),
        ]),
      ]),
    );
  }
}

class _PickRow extends StatefulWidget {
  const _PickRow({required this.name, required this.on, required this.onTap, this.curve, this.onHover});
  final String name;
  final bool on;
  final VoidCallback onTap;
  final Curve? curve;
  final ValueChanged<bool>? onHover;
  @override
  State<_PickRow> createState() => _PickRowState();
}

class _PickRowState extends State<_PickRow> {
  bool h = false;
  @override
  Widget build(BuildContext context) => MouseRegion(
        cursor: SystemMouseCursors.click,
        onEnter: (_) {
          setState(() => h = true);
          widget.onHover?.call(true);
        },
        onExit: (_) {
          setState(() => h = false);
          widget.onHover?.call(false);
        },
        child: GestureDetector(
          behavior: HitTestBehavior.opaque,
          onTap: widget.onTap,
          child: AnimatedContainer(
            duration: Mo.dur,
            curve: Mo.ease,
            height: 24,
            padding: const EdgeInsets.symmetric(horizontal: 8),
            decoration: BoxDecoration(color: widget.on ? N.g20 : (h ? N.g15 : const Color(0x00262626)), border: Border(left: BorderSide(color: widget.on ? N.g95 : const Color(0x00F2F2F2), width: 2))),
            child: Row(children: [
              Expanded(child: Text(widget.name, style: T.name(widget.on ? N.g100 : N.g91))),
              if (widget.curve != null) SizedBox(width: 36, height: 16, child: CustomPaint(painter: _CurvePlot(widget.curve!, widget.on || h ? N.g95 : N.g63))),
            ]),
          ),
        ),
      );
}

class _TextBox extends StatefulWidget {
  const _TextBox(this.l, this.s);
  final LayerM l;
  final Session s;
  @override
  State<_TextBox> createState() => _TextBoxState();
}

class _TextBoxState extends State<_TextBox> {
  late final ctrl = TextEditingController(text: widget.l.text);
  final node = FocusNode(debugLabel: 'title text');

  @override
  void initState() {
    super.initState();
    ctrl.addListener(() {
      if (ctrl.text != widget.l.text) {
        widget.l.text = ctrl.text;
        widget.l.doc.poke();
      }
    });
  }

  @override
  void dispose() {
    ctrl.dispose();
    node.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Padding(
        padding: const EdgeInsets.only(bottom: 12),
        child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          Padding(padding: const EdgeInsets.only(bottom: 6), child: Text('TEXT', style: T.micro(N.g56).copyWith(letterSpacing: .8))),
          Container(
            height: 28,
            padding: const EdgeInsets.symmetric(horizontal: 8),
            alignment: Alignment.centerLeft,
            decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(4), border: Border.all(color: N.g20)),
            child: EditableText(controller: ctrl, focusNode: node, style: T.name(N.g95), cursorColor: Role.selected, backgroundCursorColor: N.g20, maxLines: 1),
          ),
        ]),
      );
}
