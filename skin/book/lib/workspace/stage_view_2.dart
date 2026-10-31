// Part of stage_view.dart (cut there by size, as it was).
part of 'stage_view.dart';

class _StageViewState extends State<StageView> implements StageHandles {
  Offset _pan = Offset.zero;
  String? _hover;
  _View? _shown; // (the view as last laid out: where the handles are now)

  @override
  Map<String, Offset> get handles {
    final v = _shown, scene = widget.scene, b = WsArt.boxOf(scene, widget.selected), l = scene.layer(widget.selected);
    if (v == null || b == null || l == null) return const {};
    final (form, round) = _formOf();
    return _handles((d) => v.toScreen(WsArt.toView(scene, d)), b, knob: l.kind != WsKind.camera && l.id != 'bg', form: form, round: round, crop: _cropOf());
  }

  static const _crops = ['crop_left', 'crop_top', 'crop_right', 'crop_bottom'];
  /// The picked photo's crop, each side's share cut in (the inside's): none for anything else.
  List<double> _cropOf() {
    final m = WsScope.read(context).inside, t = int.tryParse(widget.selected ?? '');
    if (m == null || t == null || m.word(t, 'kind') != 'picture' || !RegExp(r'\.(jpe?g|png|webp|bmp)$', caseSensitive: false).hasMatch(m.word(t, 'file'))) return const [];
    return [for (final n in _crops) m.value(t, n, 0) ?? 0];
  }

  /// The picked shape's form and how round its corners are (the inside's), for its own handles.
  (String, double) _formOf() {
    final m = WsScope.read(context).inside, t = int.tryParse(widget.selected ?? '');
    if (m == null || t == null || m.word(t, 'kind') != 'shape') return ('', 0);
    return (m.word(t, 'shape'), m.value(t, 'round', 0) ?? 0);
  }

  @override
  void didUpdateWidget(StageView o) {
    super.didUpdateWidget(o);
    if (o.zoom != widget.zoom) _pan = Offset.zero;
    if (o.make != widget.make) _penning = false; // (another tool: the pen's next point begins a new shape)
  }

  // Joined: the inside draws the whole pane (the desk), the stage lying in it where this view puts it; told again when
  // that changes (after the frame: the inside is not asked while building).
  int? _desk;
  _View? _toldDesk;
  Motolii? _deskOf;
  double? _toldDpr;
  void _deskTo(_View v) {
    final dpr = MediaQuery.devicePixelRatioOf(context), m = WsScope.read(context).inside;
    if (m != _deskOf) {   // (another document opened in its place: its own desk told and its picture asked; the last one's was kept, and the stage stood still on it: golf_noise.md (73))
      _desk = null;
      _toldDesk = null;
      _toldPick = null;
    }
    if (widget.texture == null || (v == _toldDesk && dpr == _toldDpr)) return; // (told again when the window's zoom changes its pixels too)
    _toldDesk = v;
    _toldDpr = dpr;
    if ((_deskOf = m) == null) return;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted || _toldDesk != v || _toldDpr != dpr) return;
      final m = _deskOf!;
      final ticket = m.desk((v.area.width * dpr).round(), (v.area.height * dpr).round(), (v.o.dx - v.area.left) * dpr, (v.o.dy - v.area.top) * dpr, v.s * dpr, dpr);
      void look(Duration _) {   // (a picture at a time till the inside has made the desk: the window never stands still for it)
        if (!mounted) return;
        final a = m.answer(ticket);
        if (a < 0) {
          SchedulerBinding.instance.scheduleFrameCallback(look);
          return;
        }
        if (a == 0) return;
        _toldPick = null;
        if (_desk == null) m.deskTexture().then((id) => mounted && id >= 0 ? setState(() => _desk = id) : null);
      }
      SchedulerBinding.instance..scheduleFrameCallback(look)..ensureVisualUpdate();
    });
  }

  // Joined, the desk on: which thing is picked and which is under the hand, told when it changes (the inside marks them).
  (int, int)? _toldPick;
  void _pickTo() {
    final m = _deskOf;
    if (_desk == null || m == null) return;
    final now = (int.tryParse(widget.selected ?? '') ?? 0, int.tryParse(_hover ?? '') ?? 0);
    if (now == _toldPick) return;
    _toldPick = now;
    m.pick(now.$1, now.$2);
  }

  @override
  void dispose() {
    _deskOf?.desk(0, 0, 0, 0, 0, 0); // (the stage's own picture again)
    _dollyRest?.cancel();
    super.dispose();
  }

  _View _view(Size size) {
    final area = Rect.fromLTRB(_V.ruler, _V.ruler, size.width, size.height);
    final fit = math.max(.02, math.min((area.width - _V.fitMargin * 2) / WsArt.comp.width, (area.height - _V.fitMargin * 2) / WsArt.comp.height));
    final s = widget.zoom == 0 ? fit : _fixed[widget.zoom];
    final half = Offset(WsArt.comp.width * s, WsArt.comp.height * s) / 2;
    return _View(area.center - half + _pan, s, area);
  }

  Offset _doc(_View v, Offset local) => WsArt.toDoc(widget.scene, v.toComp(local));

  String? _hit(_View v, Offset local) {
    // (a thing that lies off the picture can still be held by its frame and carried back)
    final d = _doc(v, local), id = WsArt.hit(widget.scene, d), m = WsScope.read(context).inside;
    if (widget.group && m != null && id != null) { // (Group: the outermost of the things it goes with, by the document's parent)
      var t = int.parse(id);
      for (var p = (m.value(t, 'parent', 0) ?? 0).round(), n = 0; p > 0 && n < 64; p = (m.value(t, 'parent', 0) ?? 0).round(), n++) {
        t = p;
      }
      return '$t';
    }
    return id;
  }

  // Joined: a thing held on the stage, carried, and let go. The stage only says how far the hand has gone, in points of
  // the picture; where that puts the thing (at its own depth), and what it is to the document, is the inside's.
  ({Offset from, bool moved, String? by})? _held;   // (by: a handle's way of carrying it, scale, turn or pivot; null: placed)
  String? _grip;   // (the handle in hand, by its name: it alone shown, lit, while it is carried: Alight Motion's)

  // Joined, Snap (the past repository's, After Effects' snapping): the held thing's left, middle and right (top, middle,
  // bottom) stick to the nearest of the same three lines of every other thing it does not go with and of the picture,
  // or to a guide, within six points of the screen. What it holds to, and the lines it met.
  ({Rect box, List<(double, double)> xs, List<(double, double)> ys})? _snapTo;
  (double?, double?) _met = (null, null);

  Rect _bounds(ArtBox b) {
    final p = [for (final c in b.corners) WsArt.toView(widget.scene, c)];
    return p.skip(1).fold(Rect.fromPoints(p[0], p[0]), (r, q) => r.expandToInclude(Rect.fromPoints(q, q)));
  }

  void _snapFrom(String id) {
    final boxes = widget.scene.boxes, m = WsScope.read(context).inside;
    final held = boxes[id];
    if (!widget.snap || held == null || m == null) return _snapTo = null;
    int top(int t) {
      for (var p = (m.value(t, 'parent', 0) ?? 0).round(), n = 0; p > 0 && n < 64; p = (m.value(t, 'parent', 0) ?? 0).round(), n++) {
        t = p;
      }
      return t;
    }
    final mine = top(int.parse(id)), c = WsArt.comp;
    final xs = <(double, double)>[(0, c.width)], ys = <(double, double)>[(0, c.height)];
    for (final g in widget.scene.guidesX) {
      xs.add((g, g));
    }
    for (final g in widget.scene.guidesY) {
      ys.add((g, g));
    }
    for (final MapEntry(:key, :value) in boxes.entries) {
      if (top(int.parse(key)) == mine) continue;
      final r = _bounds(value);
      xs.add((r.left, r.right));
      ys.add((r.top, r.bottom));
    }
    _snapTo = (box: _bounds(held), xs: xs, ys: ys);
  }

  Offset _snapped(Offset by, double s) {
    final to = _snapTo;
    if (to == null) return by;
    (double, double?) one(double lo, double size, double d, List<(double, double)> lines) {
      (double, double)? best;
      for (final (a, b) in lines) {
        for (final line in [a, (a + b) / 2, b]) {
          for (final mine in [lo + d, lo + d + size / 2, lo + d + size]) {
            final diff = line - mine;
            if (diff.abs() <= 6 / s && (best == null || diff.abs() < best.$1.abs())) best = (diff, line);
          }
        }
      }
      return best == null ? (d, null) : (d + best.$1, best.$2);
    }
    final (x, mx) = widget.axis == 2 ? (by.dx, null) : one(to.box.left, to.box.width, by.dx, to.xs);
    final (y, my) = widget.axis == 1 ? (by.dy, null) : one(to.box.top, to.box.height, by.dy, to.ys);
    _met = (mx, my);
    return Offset(x, y);
  }

  // Joined: a guide pulled out of a ruler (from the top one, a level line; from the left one, an upright one) or taken
  // hold of where it lies; let go on the picture it is the document's there, let go back over its ruler it is gone.
  ({String axis, double? was})? _pulling;

  bool _pull(_View v, Offset local) {
    final ws = WsScope.read(context), m = ws.inside;
    if (m == null) return false;
    final at = v.toComp(local), tol = 4 / v.s;
    if (local.dy < _V.ruler || local.dx < _V.ruler) {
      _pulling = (axis: local.dy < _V.ruler ? 'y' : 'x', was: null);
      return true;
    }
    for (final (axis, g) in m.guides) {
      if (((axis == 'x' ? at.dx : at.dy) - g).abs() > tol) continue;
      _pulling = (axis: axis, was: g);
      return true;
    }
    return false;
  }

  void _pullLetGo(_View v, Offset local) {
    final p = _pulling, ws = WsScope.read(context), m = ws.inside;
    _pulling = null;
    if (p == null || m == null) return;
    final over = p.axis == 'y' ? local.dy < _V.ruler : local.dx < _V.ruler, at = v.toComp(local);
    m.later('together', number: 1);   // (one step, without waiting)
    if (p.was != null) m.later('guide', name: p.axis, number: p.was!, word: 'off');
    if (!over) m.later('guide', name: p.axis, number: ((p.axis == 'x' ? at.dx : at.dy)).roundToDouble());
    ws.after(m.later('together'));
  }

  Offset? _last;

  // Joined: a thing drawn with the tool in hand: from where the hand came down to where it is let go (a rect or a
  // circle that box, a line from the one point to the other, a text where it came down); only come down, one as it
  // comes. One step; it is picked.
  Offset? _drawFrom;

  // Joined: drawn round on the stage where no thing is (no tool in hand): the things whose middles it goes round are
  // picked together (Ws.many), and what they may do together is offered beside them (relate.dart).
  List<Offset>? _lasso;

  void _lassoLetGo(_View v) {
    final round = _lasso, ws = WsScope.read(context);
    setState(() => _lasso = null);
    if (round == null || round.length < 3) return;
    // (drawn straight, it is a box from where the hand came down to where it let go (Figma's, AE's); else the line it drew round)
    final a = round.first, z = round.last, along = z - a;
    final straight = along.distance > 4 && round.every((p) => ((p - a).dx * along.dy - (p - a).dy * along.dx).abs() / along.distance < 12);
    final path = straight ? (Path()..addRect(Rect.fromPoints(a, z))) : (Path()..addPolygon(round, true));
    ws.pickMany([for (final l in widget.scene.layers) if (widget.scene.box(l.id) case final b? when path.contains(v.toScreen(WsArt.toView(widget.scene, b.c)))) l.id]);
  }

  void _draw(_View v, Offset to) {
    final from = _drawFrom, ws = WsScope.read(context), m = ws.inside, kind = widget.make;
    _drawFrom = null;
    setState(() {});
    if (from == null || m == null || kind == null) return;
    final a = v.toComp(from), b = v.toComp(to), p = m.point, c = WsArt.comp;
    final centred = kind == 'circle' && widget.fromCentre; // (drawn out from its middle: twice as large about where the hand came down)
    final mid = centred ? a : (a + b) / 2, d = centred ? (b - a) * 2 : b - a, tiny = d.distance < 6 / v.s;
    final x = ((kind == 'text' || tiny ? a.dx : mid.dx) - c.width / 2) / p, y = (c.height / 2 - (kind == 'text' || tiny ? a.dy : mid.dy)) / p;
    final cut = widget.mask ? int.tryParse(ws.selected ?? '') : null;
    // (one step, asked without waiting: the new thing is the one the add made (thing -1), picked once it is there)
    final before = m.things.length;
    m.later('together', number: 1);
    var made = 0;
    ws.after(m.later('add', name: kind == 'text' ? '' : 'shape', word: kind == 'text' ? ws.bodyFace : kind), null, (t) => made = t);   // (a text in the body's font, as a text box is in PowerPoint's Body font)
    m
      ..later('set', thing: -1, name: 'x', number: x)
      ..later('set', thing: -1, name: 'y', number: y);
    wsInked(ws, kind == 'text' ? 'text' : 'shape', kind); // (the drawing inks)
    if (cut != null) { // (Mask: it lies just behind the picked thing, and that one is clipped by it; not seen itself: it only cuts, as After Effects' matte)
      final all = m.things;
      if (all.contains(cut)) {
        m
          ..later('raise', thing: -1, number: (all.indexOf(cut) - before).toDouble())
          ..later('set', thing: cut, name: 'clip', number: 1)
          ..later('set', thing: -1, name: 'opacity', number: 0);
      }
    }
    if (!tiny && kind != 'text') {
      if (kind == 'line') {   // (a line is a twenty-fifth of a unit thick: as long as drawn, turned as drawn)
        m
          ..later('set', thing: -1, name: 'wide', number: d.distance / p / .04)
          ..later('set', thing: -1, name: 'rotate', number: math.atan2(d.dy, d.dx) * 180 / math.pi);
      } else {
        m
          ..later('set', thing: -1, name: 'size', number: d.dy.abs() / p)
          ..later('set', thing: -1, name: 'wide', number: d.dx.abs() / math.max(d.dy.abs(), 1e-3));
      }
    }
    ws.after(m.later('together'), (done, _) {
      if (!done || made <= 0) return;
      if (kind == 'text') textToType = true;   // (typed at once: the Text card takes the keys, inspector_material.dart)
      ws.select('$made');
    });
    ws.setView('stage.tool', 0);   // (one drawn, the hand is free: Alight Motion's Add is one act and Figma's tool goes back to select; After Effects' stays, so they part and Alight Motion decides: golf_noise.md head, ㊼)
  }

  // Joined: the pen: a point where the hand came down, its handle drawn out as far as the hand was dragged (a tap: a
  // corner). Its first goes to the picked thing, the rest to the one the pen made (thing -1); the inside reckons all of
  // it (a new path begun, closed on its first point, a point put on or taken off a closed one) from where the hand is.
  bool _penning = false;

  void _pen(_View v, Offset down, Offset up) {
    final ws = WsScope.read(context), m = ws.inside;
    _drawFrom = null;
    setState(() {});
    if (m == null) return;
    final a = _doc(v, down), h = _doc(v, up) - a;
    final first = !_penning;
    _penning = true;
    if (first) m.later('together', number: 1);
    var made = 0;
    ws.after(m.later('pen', thing: first ? int.tryParse(widget.selected ?? '') ?? 0 : -1, word: '${a.dx} ${a.dy} ${h.dx} ${h.dy}'), null, (t) => made = t); // (first on the picked thing: a closed path gains or loses a point, anything else begins a new one)
    if (first) {
      if (ws.ink.isNotEmpty) m.later('say', thing: -1, name: 'fill', word: ws.ink);
      if (ws.ink.contains(' ') && ws.inkBlend.isNotEmpty) m.later('say', thing: -1, name: 'gradient_blend', word: ws.inkBlend);
      if (ws.edge.isNotEmpty) {
        m
          ..later('say', thing: -1, name: 'stroke', word: ws.edge)
          ..later('set', thing: -1, name: 'stroke_width', number: 6);
      }
      ws.after(m.later('together'), (done, _) => done && made > 0 ? ws.select('$made') : null);
    }
  }

  // (the Brush's way so far, in points of the window; when let go, its points as perfect-freehand (MIT, Steve Ruiz; Dart
  //  port, Adil Hanney) evens them and the pressure it simulates from their speed, given to the inside, which keeps the
  //  stroke and has AbsentMotion make its outline as Alight Motion's brush does (doc_brush): each point's pressure, as
  //  AbsentMotion's brush reads it, its radius here over half the brush)
  List<Offset>? _brush;
  void _brushLetGo(_View v) {
    final way = _brush, ws = WsScope.read(context), m = ws.inside;
    setState(() => _brush = null);
    if (way == null || way.length < 2 || m == null) return;
    final options = StrokeOptions(size: widget.brushSize, simulatePressure: true, isComplete: true);
    final points = getStrokePoints([for (final p in way) PointVector.fromOffset(offset: _doc(v, p))], options: options);
    if (points.length < 2) return;
    getStrokeOutlinePoints(points, options: options, rememberSimulatedPressure: true);
    final said = [for (final p in points) '${p.point.x} ${p.point.y} ${2 * getStrokeRadius(options.size, options.thinning, p.pressure, options.easing) / options.size}'];
    if (widget.brushTool == 3) {   // (an eraser: taken from the picked drawing, as Alight Motion's eraser takes from the drawing layer it is in)
      final into = int.tryParse(ws.selected ?? '');
      if (into == null) {
        ws.notice = 'Pick a drawing to erase from';
        return;
      }
      ws.after(m.later('brush', thing: into, word: '3 ${options.size} ${said.join(' ')}'), (done, why) {
        if (!done) ws.notice = why;
      });
      return;
    }
    m.later('together', number: 1);
    var made = 0;
    ws.after(m.later('brush', word: '${widget.brushTool} ${options.size} ${said.join(' ')}'), null, (t) => made = t);
    m.later('say', thing: -1, name: 'fill', word: ws.ink.isEmpty ? 'ffffff' : ws.ink);   // (an outline with no fill shows nothing: white, as a drawing app's brush begins, when no ink is chosen)
    ws.after(m.later('together'), (done, _) => done && made > 0 ? ws.select('$made') : null);
  }

  /// The camera the Camera tool turns: the picked one, else the document's first.
  int? _cameraOf(Ws ws) {
    final m = ws.inside, picked = int.tryParse(widget.selected ?? '');
    if (m == null) return null;
    if (picked != null && m.word(picked, 'kind') == 'camera') return picked;
    for (final l in ws.layers) {
      if (int.tryParse(l.id) case final t? when m.word(t, 'kind') == 'camera') return t;
    }
    return null;
  }

  ({double x, double y})? _orbitFrom;   // (the camera's turn when the hand took hold)
  double? _dollyAt;   // (its distance while the wheel turns)
  Timer? _dollyRest;

  /// After Effects' Unified Camera tool's wheel: the camera nearer or further from where it looks, the wheel's turning
  /// one step once it rests (golf_noise.md ㊲: no hand on the stage turned or brought the camera).
  void _dolly(double by) {
    final ws = WsScope.read(context), m = ws.inside, cam = _cameraOf(ws);
    if (m == null || cam == null) return;
    if (ws.freeEye) return ws.setView('stage.eye', [ws.eye[0], ws.eye[1], (ws.eye[2] * math.exp(by * .002)).clamp(.01, 100.0)]);   // (After Effects' working views: the tool moves the view, not the camera)
    if (_dollyAt == null) {
      m.take(cam);
      _dollyAt = m.value(cam, 'distance', ws.frame / Ws.fps) ?? 1;
    }
    _dollyAt = (_dollyAt! * math.exp(by * .002)).clamp(.01, 100.0);
    m.carry('distance', _dollyAt!);
    _dollyRest?.cancel();
    _dollyRest = Timer(const Duration(milliseconds: 300), () {
      _dollyAt = null;
      ws.after(m.later('let_go', number: 1));
    });
  }

  double _orbitRate = .25;   // (degrees a pixel)
  ({double zoom, double reach, Offset middle})? _lensFrom;   // (the frame's size when the hand took hold)

  /// A hand on a camera in the working eye: where it is goes round the point it looks at (After Effects' camera and point
  /// of interest), a picked one's corner sizes the frame, which is the lens (AbsentMotion's withCameraFrameScaled), and
  /// the frame or that point moves it (cameraGesture). [inside]: anywhere in the frame, or only on its marks.
  bool _gizmoHold(_View v, Offset local, {required bool inside}) {
    final ws = WsScope.read(context), m = ws.inside!, scene = widget.scene, at = ws.frame / Ws.fps;
    for (final id in scene.gizmos.keys) {
      final g = [for (final p in scene.gizmo(id)) p == null ? null : v.toScreen(WsArt.toView(scene, p))], cam = int.parse(id);
      if (g.length < 6) continue;
      final frame = [g[0], g[1], g[2], g[3]].nonNulls.toList();
      bool on(Offset? p, double reach) => p != null && (p - local).distance <= reach;
      final String by;
      if (on(g[5], 10)) {
        by = 'gizmo orbit';
        _orbitRate = g[4] == null ? .25 : 180 / math.pi / math.max((g[5]! - g[4]!).distance, 60);   // (a turn as far as the hand has gone on the circle it goes round)
        _orbitFrom = (x: m.value(cam, 'rotate_x', at) ?? 0, y: m.value(cam, 'rotate_y', at) ?? 0);
      } else if (widget.selected == id && frame.any((p) => on(p, 8)) && g[4] != null) {
        by = 'gizmo lens';
        _lensFrom = (zoom: m.value(cam, 'zoom', at) ?? 1, reach: (local - g[4]!).distance, middle: g[4]!);
      } else if (on(g[4], 10) || (inside && frame.length == 4 && (Path()..addPolygon(frame, true)).contains(local))) {
        by = 'place';
      } else {
        continue;
      }
      m.take(cam);
      ws.select(id);
      if (ws.playing) ws.playing = false;
      _held = (from: local, moved: false, by: by == 'place' ? null : by);
      return true;
    }
    return false;
  }

  void _hold(_View v, Offset local) {
    if (widget.make == 'camera') {   // (After Effects' Unified Camera tool: a drag turns the camera round where it looks)
      final ws = WsScope.read(context), m = ws.inside, cam = _cameraOf(ws);
      if (m == null || cam == null) {
        ws.notice = 'No camera: add one from Create › Layers';
        return;
      }
      if (ws.freeEye) {
        _orbitFrom = (x: ws.eye[1], y: ws.eye[0]);
        _held = (from: local, moved: false, by: 'orbit');
        return;
      }
      final at = ws.frame / Ws.fps;
      m.take(cam);
      if (ws.playing) ws.playing = false;
      _orbitFrom = (x: m.value(cam, 'rotate_x', at) ?? 0, y: m.value(cam, 'rotate_y', at) ?? 0);
      _held = (from: local, moved: false, by: 'orbit');
      return;
    }
    if (widget.make == 'brush') {
      setState(() => _brush = [local]);
      return;
    }
    if (widget.make != null) {
      setState(() => _drawFrom = local);
      return;
    }
    final ws = WsScope.read(context), m = ws.inside;
    // (joined: a hand on one of the picked thing's handles carries it the handle's way: the inside reckons it from where it took hold)
    final picked = int.tryParse(widget.selected ?? '');
    if (m != null && picked != null) {
      String? by, grip;
      var near = 8.0;
      for (final MapEntry(:key, :value) in handles.entries) {
        final d = (value - local).distance;
        if (d > near) continue;
        near = d;
        by = switch (key.split(' ').first) { 'corner' => 'scale', 'turn' => 'turn', 'anchor' => 'pivot', 'round' => 'round', 'points' => 'points', 'crop' => key, _ => null };
        grip = key;
      }
      for (final p in [...m.points(picked), for (final (h, _) in m.handles(picked)) h]) { // (a path's own point, moved with its handles; or a handle alone: the inside takes the nearest)
        final d = (v.toScreen(WsArt.toView(widget.scene, p)) - local).distance;
        if (d > near) continue;
        near = d;
        by = 'point';
        grip = null;
      }
      if (by != null) {
        m.take(picked);
        final at = _doc(v, local); // (points of the picture, as the inside counts them)
        m.carry('grip', at.dx, at.dy);
        if (ws.playing) ws.playing = false;
        _held = (from: local, moved: false, by: by);
        if (grip != _grip) setState(() => _grip = grip);
        return;
      }
    }
    if (m != null && _gizmoHold(v, local, inside: false)) return;
    if (_pull(v, local)) return; // (a guide out of a ruler, or one taken hold of: after the handles, which lie over everything)
    final id = m == null ? null : _hit(v, local), t = int.tryParse(id ?? '');
    if (m != null && t == null && _gizmoHold(v, local, inside: true)) return;   // (AbsentMotion's TopLayerAt: the things first, then the frame they lie in)
    if (m != null && t == null) { // (where no thing is: drawn round)
      ws.select(null);
      setState(() => _lasso = [local]);
      return;
    }
    if (m == null || t == null) return;
    m.take(t);
    ws.select(id);
    if (ws.playing) ws.playing = false; // (a hand on a thing stops the play)
    _held = (from: local, moved: false, by: null);
    _snapFrom(id!);
  }

  void _carry(_View v, Offset local) {
    final h = _held, ws = WsScope.read(context), m = ws.inside;
    if (h == null || m == null) return;
    if (h.by == 'gizmo orbit' && _orbitFrom != null) {   // (it goes the way the hand does)
      final d = (local - h.from) * _orbitRate, o = _orbitFrom!;
      m.carry('rotate_y', o.y + d.dx);
      m.carry('rotate_x', (o.x + d.dy).clamp(-89.0, 89.0));
      _held = (from: h.from, moved: true, by: h.by);
      return;
    }
    if (h.by == 'gizmo lens' && _lensFrom != null) {   // (as far from the middle as the hand now is, against where it took hold: a larger frame, a wider lens)
      final l = _lensFrom!;
      m.carry('zoom', (l.zoom * l.reach / math.max((local - l.middle).distance, 1)).clamp(.01, 100.0));
      _held = (from: h.from, moved: true, by: h.by);
      return;
    }
    if (h.by == 'orbit' && _orbitFrom != null) {   // (sideways round it, up and down over it: a quarter of a degree a pixel)
      final d = local - h.from, o = _orbitFrom!;
      if (ws.freeEye) return ws.setView('stage.eye', [o.y - d.dx * .25, (o.x - d.dy * .25).clamp(-89.0, 89.0), ws.eye[2]]);
      m.carry('rotate_y', o.y - d.dx * .25);
      m.carry('rotate_x', (o.x - d.dy * .25).clamp(-89.0, 89.0));
      _held = (from: h.from, moved: true, by: 'orbit');
      setState(() {});
      return;
    }
    final all = (local - h.from) / v.s; // (screen px -> points of the picture)
    if (h.by case final way? when way.startsWith('crop ')) {   // (a photo's side cut in as far as the hand goes in from where it is cut, no further than leaves some of it)
      final side = int.parse(way.substring(5)), c = [for (var i = 0; i < 4; i++) handles['corner $i'] ?? Offset.zero], crop = _cropOf();
      final across = side.isEven ? c[1] - c[0] : c[3] - c[0], len = math.max(1.0, across.distance), dir = across / len * (side < 2 ? 1.0 : -1.0);
      final at = handles[way] ?? h.from, inBy = ((local - at).dx * dir.dx + (local - at).dy * dir.dy) / len;
      if (crop.length == 4) m.carry(_crops[side], (crop[side] + inBy).clamp(0.0, .95 - crop[(side + 2) % 4]));
      _held = (from: h.from, moved: true, by: way);
      setState(() {});
      return;
    }
    if (h.by case final way? when way == 'round' || way == 'points') {   // (a shape's own: its corners' roundness, or its points, as far as the hand goes)
      final c = [for (var i = 0; i < 4; i++) handles['corner $i'] ?? Offset.zero], g = _gripOf(c), t = int.tryParse(widget.selected ?? '');
      if (way == 'round') m.carry('round', g.roundOf(local));
      if (way == 'points' && t != null) m.carry('points', ((m.value(t, 'points', 0) ?? 5) + (((local - h.from).dx * g.u.dx + (local - h.from).dy * g.u.dy) / 24).round()).clamp(3, 64).toDouble());
      _held = (from: h.from, moved: true, by: way);
      setState(() {});
      return;
    }
    if (h.by case final way?) {
      final gone = _doc(v, local) - _doc(v, h.from);
      // (a corner: the side across stays, as Figma's and Keynote's; with ⌥ held, about its pivot, from the middle)
      m.carry(way == 'scale' && !HardwareKeyboard.instance.isAltPressed ? 'side' : way, gone.dx, gone.dy);
      _held = (from: h.from, moved: true, by: way);
      setState(() {});
      return;
    }
    final by = _snapped(switch (widget.axis) { 1 => Offset(all.dx, 0), 2 => Offset(0, all.dy), _ => all }, v.s); // (the Move tool's way)
    m.carry('place', by.dx, by.dy);
    _held = (from: h.from, moved: true, by: null);
    setState(() {});
  }

  void _letGo({bool keep = true}) {
    final h = _held, ws = WsScope.read(context), m = ws.inside, free = h?.by == 'orbit' && ws.freeEye;
    _held = null;
    _orbitFrom = null;
    _lensFrom = null;
    _snapTo = null;
    if (_grip != null) setState(() => _grip = null);
    if (_met != (null, null)) setState(() => _met = (null, null));
    if (h == null || m == null || free) return;
    ws.after(m.later('let_go', number: keep && h.moved ? 1 : 0));   // (let go without waiting: the document read again once the inside has it)
  }

  void _scroll(PointerSignalEvent e, _View v) {
    if (e is PointerScrollEvent && widget.make == 'camera') return _dolly(e.scrollDelta.dy);
    if (e is PointerScrollEvent) _panBy(-e.scrollDelta, v);
  }

  /// Panned by (a trackpad's two fingers as they go, a wheel the other way), when zoomed in; at Fit there is nowhere to go.
  void _panBy(Offset by, _View v) {
    if (widget.zoom == 0) return;
    final lim = Offset(WsArt.comp.width * v.s / 2 + v.area.width / 2 - 60, WsArt.comp.height * v.s / 2 + v.area.height / 2 - 60);
    final p = _pan + by;
    setState(() => _pan = Offset(p.dx.clamp(-lim.dx, lim.dx), p.dy.clamp(-lim.dy, lim.dy)));
  }

  /// The picked thing's parent, by its id (null: none).
  String? _parentOf() {
    final m = WsScope.read(context).inside, t = int.tryParse(widget.selected ?? '');
    final p = m == null || t == null ? 0 : (m.value(t, 'parent', 0) ?? 0).round();
    return p > 0 ? '$p' : null;
  }

  /// What the hand is changing, as the stage shows it now (its frame, carried values and all): where it is, how wide and
  /// high, how far turned, or where it turns about (points of the picture, degrees clockwise as seen).
  String? _heldSays(String? by) {
    final m = WsScope.read(context).inside, t = int.tryParse(widget.selected ?? '');
    final f = m == null || t == null ? null : m.frame(t);
    if (f == null) return null;
    return switch (by) {
      null => '${f[0].round()}, ${f[1].round()}',
      'scale' => '${f[2].round()} × ${f[3].round()}',
      'turn' => '${(f[4] % 360).round()}°',
      'pivot' => '${f[5].round()}, ${f[6].round()}',
      _ => null,
    };
  }

  // The picked thing's way over its time: where the inside places it, moment by moment (motolii_path, its wave and all),
  // in points of the picture, drawn thin with arrows the way it goes (Alight Motion's motion path). Asked again only
  // when the document or the pick changes; none when it stays put.
  (Object?, List<Offset>?) _trail = (null, null);
  List<Offset>? _trailOf() {
    final ws = WsScope.read(context), m = ws.inside, t = int.tryParse(widget.selected ?? '');
    if (m == null || t == null || const {'behaviour', 'camera', 'light'}.contains(m.word(t, 'kind'))) return null;
    final key = (ws.rev, t);
    if (_trail.$1 != key) {
      _trail = (key, null);
      final start = m.value(t, 'start', 0) ?? 0, lasts = m.lasts(t), end = lasts > 0 ? start + lasts : m.seconds;
      if (end > start) {
        m.path(t, from: start, to: end, n: 90).then((p) {
          if (!mounted || _trail.$1 != key || p == null) return;
          final c = WsArt.centre, pts = [for (var i = 0; i + 1 < p.length; i += 4) c + Offset(p[i], -p[i + 1]) * m.point];
          setState(() => _trail = (key, pts.any((q) => (q - pts.first).distance > 1) ? pts : null));
        });
      }
    }
    return _trail.$2;
  }

  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, box) {
      final v = _shown = _view(box.biggest), scene = widget.scene, trail = _trailOf();
      _deskTo(v);
      _pickTo();
      return Listener(
        onPointerSignal: (e) => _scroll(e, v),
        child: WsTwoFingers(
        onPan: (by, _) => _panBy(by, v),
        child: MouseRegion(
          onHover: (e) {
            widget.cursor.value = v.toComp(e.localPosition);
            final h = _hit(v, e.localPosition);
            if (h != _hover) setState(() => _hover = h);
          },
          onExit: (_) {
            widget.cursor.value = null;
            if (_hover != null) setState(() => _hover = null);
          },
          child: GestureDetector(
            behavior: HitTestBehavior.opaque,
            supportedDevices: wsPressing,
            onTapUp: (d) {
              widget.onPoke?.call();
              if (widget.make == 'pen') return _pen(v, d.localPosition, d.localPosition); // (a tap: a corner)
              if (widget.make == 'brush') return; // (a stroke is drawn, not tapped)
              if (widget.make != null) return _draw(v, _drawFrom = d.localPosition); // (only come down: one as it comes, there)
              final hit = _hit(v, d.localPosition), ws = WsScope.read(context);
              if (HardwareKeyboard.instance.isShiftPressed || HardwareKeyboard.instance.isMetaPressed) return hit == null ? null : wsPickAlso(ws, hit); // (Shift or Cmd: one more picked together, or one less; on nothing, nothing lost)
              ws.select(hit);
            },
            dragStartBehavior: DragStartBehavior.down, // (a thing is held where the hand came down, not where it had got to when the drag was known for one)
            onPanStart: (d) => _hold(v, _last = d.localPosition),
            onPanUpdate: (d) => _brush != null
                ? setState(() => _brush!.add(d.localPosition))
                : _lasso != null
                ? setState(() => _lasso!.add(d.localPosition))
                : _drawFrom != null
                ? setState(() => _last = d.localPosition)
                : _pulling != null
                ? (widget.cursor.value = v.toComp(_last = d.localPosition))
                : _carry(v, _last = d.localPosition),
            onPanEnd: (_) => _brush != null
                ? _brushLetGo(v)
                : _lasso != null
                ? _lassoLetGo(v)
                : _drawFrom != null
                ? (widget.make == 'pen' ? _pen(v, _drawFrom!, _last ?? _drawFrom!) : _draw(v, _last ?? _drawFrom!))
                : _pulling != null
                ? _pullLetGo(v, _last!)
                : _letGo(),
            onPanCancel: () => _brush != null ? setState(() => _brush = null) : _lasso != null ? setState(() => _lasso = null) : _drawFrom != null ? setState(() => _drawFrom = null) : _pulling != null ? _pulling = null : _letGo(keep: false),
            child: Stack(
              fit: StackFit.expand,
              children: [
                RepaintBoundary(
                  child: CustomPaint(
                    painter: _ViewPainter(scene, v, widget.selected, _hover == widget.selected ? null : _hover, grid: widget.grid, safe: widget.safe, part: widget.texture == null ? _Part.all : _Part.under, pivots: widget.pivots, met: _met, trail: trail, parent: _parentOf(), grip: _grip, form: _formOf(), crop: _cropOf()),
                  ),
                ),
                if (_desk case final desk? when widget.texture != null) ...[ // (the whole pane as the inside draws it: off the stage covered half way, the picked thing marked)
                  Positioned.fromRect(rect: v.area, child: IgnorePointer(child: Texture(textureId: desk, filterQuality: FilterQuality.medium))),
                  Positioned.fromRect(rect: v.comp, child: const IgnorePointer(child: SizedBox(key: ValueKey('ws-stage-comp')))), // (where the picture lies in it)
                ] else
                Positioned.fromRect(
                  rect: v.comp,
                  child: IgnorePointer(
                    child: widget.texture == null
                        ? const SizedBox(key: ValueKey('ws-stage-comp'))
                        : ClipRect(key: const ValueKey('ws-stage-comp'), child: Texture(textureId: widget.texture!, filterQuality: FilterQuality.medium)),
                  ),
                ),
                if (widget.snapshot case final shot?)   // (Show Snapshot held: the earlier picture where the picture lies)
                  Positioned.fromRect(rect: v.comp, child: IgnorePointer(child: Image.file(File(shot), key: const ValueKey('ws-stage-snapshot'), fit: BoxFit.fill, gaplessPlayback: true))),
                if (int.tryParse(widget.selected ?? '') case final picked? when WsScope.read(context).inside != null) // (the picked path's own points, to be taken hold of)
                  IgnorePointer(child: CustomPaint(size: Size.infinite, painter: _PointsPainter([for (final p in WsScope.read(context).inside!.points(picked)) v.toScreen(WsArt.toView(widget.scene, p))],
                      [for (final (h, o) in WsScope.read(context).inside!.handles(picked)) (v.toScreen(WsArt.toView(widget.scene, h)), v.toScreen(WsArt.toView(widget.scene, o)))]))),
                if (_drawFrom case final from? when _last != null && widget.make == 'pen') // (the pen's handle being drawn out: through its point, both ways)
                  IgnorePointer(child: CustomPaint(size: Size.infinite, painter: _HandlePainter(from, _last!)))
                else if (_drawFrom case final from? when _last != null) // (what is being drawn: its box)
                  Positioned.fromRect(
                    rect: Rect.fromPoints(from, _last!),
                    child: IgnorePointer(child: DecoratedBox(decoration: BoxDecoration(border: Border.all(color: WsT.accent)))),
                  ),
                if (_held case final h? when h.moved && _last != null)   // (the numbers while a hand moves it, by the hand, as Figma shows them; Alight Motion shows none)
                  if (_heldSays(h.by) case final says?)
                    Positioned(
                      left: _last!.dx + 16,
                      top: _last!.dy + 16,
                      child: IgnorePointer(
                        child: Container(
                          key: const ValueKey('ws-held-says'),
                          padding: EdgeInsets.symmetric(horizontal: Shape.s(6), vertical: Shape.s(3)),
                          decoration: BoxDecoration(color: WsT.raised, borderRadius: BorderRadius.all(Shape.r(3))),
                          child: Text(says, style: T.micro(Grey.g95)),
                        ),
                      ),
                    ),
                if (widget.texture != null)
                  IgnorePointer(
                    child: CustomPaint(
                      painter: _ViewPainter(scene, v, widget.selected, _hover == widget.selected ? null : _hover, grid: widget.grid, safe: widget.safe, part: _Part.over, pivots: widget.pivots, met: _met, marked: _desk != null, trail: trail, parent: _parentOf(), grip: _grip, form: _formOf(), crop: _cropOf()),
                    ),
                  ),
                IgnorePointer(child: CustomPaint(painter: _RulerPainter(v, widget.cursor, scene))),
                _RelationMarks(scene: scene, v: v, lasso: _lasso),
                if (_brush case final way?) IgnorePointer(child: CustomPaint(painter: _BrushWay(way, WsT.accent))), // (the way drawn so far, thin: the stroke itself is the inside's once let go) // (drawn round, the things picked together, the relation picked: its shape, its handles, what it moves)
                Positioned(
                  left: _V.ruler + _V.pad,
                  bottom: _V.pad,
                  child: _Hud(scale: v.s, cursor: widget.cursor),
                ),
                if (widget.zoom != 0 && box.maxWidth > 360 && box.maxHeight > 220)
                  Positioned(
                    right: _V.pad,
                    bottom: _V.pad,
                    child: _Navigator(scene: scene, view: v, texture: widget.texture),
                  ),
              ],
            ),
          ),
        ),
        ),
      );
    },
  );
}

class _BrushWay extends CustomPainter {
  const _BrushWay(this.way, this.ink);
  final List<Offset> way;
  final Color ink;
  @override
  void paint(Canvas c, Size s) => c.drawPoints(PointMode.polygon, way, Paint()
    ..color = ink
    ..strokeWidth = 2
    ..strokeCap = StrokeCap.round);
  @override
  bool shouldRepaint(_BrushWay o) => true;
}
