// Part of ws.dart (cut there by size, as it was).
part of 'ws.dart';

class Ws extends ChangeNotifier {
  /// Each aspect told on its own (a Flutter ValueNotifier: what listens to one hears only it).
  late final on = {for (final a in WsAspect.values) a: ValueNotifier<int>(0)};
  void _tell(WsAspect a) {
    if (a == WsAspect.picked || a == WsAspect.document) _watch();   // (what the inside puts in the moment's picture follows what is picked)
    on[a]!.value++;
    notifyListeners();   // (WsScope's own: it says which aspects to the parts below)
  }


  final layers = <WsLayer>[];

  /// The composition's rate and length in frames (the sample's; a joined document's own length).
  static int fps = 30, duration = 240;

  /// The inside this window is joined to (motolii.dart), if it is: the layers are then the document's things, a
  /// change is a call to it, and the playhead is its moment. null: the sample's mock document.
  Motolii? _inside;
  Motolii? get inside => _inside;
  void join(Motolii inside) {
    _inside = inside;
    inside.shaping;   // (read once, at the opening: what a card is made by)
    inside.told = ({required document, required moment, required kept}) {   // (the inside's pictures are new: skin/BRIDGE.md)
      if (_gone) return;
      if (kept) this.kept.value = inside.kept;
      if (document) reread();
      if (moment) seen.value++;
    };
    fps = _fpsOf(inside); // (a frame is the document's: the timeline counts what a written film has)
    WsArt.comp = Size(inside.size.$1.toDouble(), inside.size.$2.toDouble());
    moment.value = (inside.now * fps).round(); // (the playhead is the inside's moment from the first)
    reread();
    final kept = inside.settings;   // (the choices kept with the document: its inks and fonts, golf (90))
    _ink = kept['ink'] ?? '';
    _edge = kept['edge'] ?? '';
    _inkBlend = kept['ink_blend'] ?? '';
    _titleFont = kept['title_font'] ?? '';
    _bodyFont = kept['body_font'] ?? '';
  }

  /// Joined: the document's name, and how the window is told to open it again at another size (the host does it).
  String? document;
  String? documentFolder; // (where the document's file is: the dock offers it, so a new document is one step from its footage)
  void Function(int wide, int high)? onSize;
  VoidCallback? refit;

  /// Joined: how the document is written as a film (the host does it), and what the window has to say about the
  /// document just now (it is being written, where it was written, why something was not done); null: nothing.
  VoidCallback? onRender;

  /// Where the next film is written (set by the settings window; the window's own default when null).
  String? renderTo;
  /// Joined: the inks the drawing tools give a new thing (a fill word, a stroke colour; '': the document's own), and how
  /// the ink's colours are mixed when it is a gradient (gradient_blend; '': sRGB). Kept with the document, not steps
  /// of it, as a PowerPoint file keeps its theme's colours (the inside's setting: golf_noise.md (90), engine_needs.md E4).
  String _ink = '', _edge = '', _inkBlend = '';
  String get ink => _ink;
  set ink(String v) => _ink = _kept('ink', _ink, v);
  String get edge => _edge;
  set edge(String v) => _edge = _kept('edge', _edge, v);
  String get inkBlend => _inkBlend;
  set inkBlend(String v) => _inkBlend = _kept('ink_blend', _inkBlend, v);
  String _kept(String name, String was, String now) {
    if (now != was) _inside?.later('setting', name: name, word: now);
    return now;
  }

  /// The fonts a title and a body text are set in as they are added, and which of the two the Fonts shelf sets now:
  /// PowerPoint's theme fonts (Headings and Body, a text box in Body), Canva's brand fonts (a heading's, a body's).
  /// '': the usual one (golf_noise.md ㉚). Kept with the document, as a PowerPoint file keeps its theme's fonts (golf (90)).
  String _titleFont = '', _bodyFont = '', fontRole = 'title';
  String get titleFont => _titleFont;
  set titleFont(String v) => _titleFont = _kept('title_font', _titleFont, v);
  String get bodyFont => _bodyFont;
  set bodyFont(String v) => _bodyFont = _kept('body_font', _bodyFont, v);
  String get bodyFace => bodyFont.isEmpty ? wsTextFont : bodyFont;
  String get titleFace => titleFont.isEmpty ? bodyFace : titleFont;

  String? _notice;
  String? get notice => _notice;
  set notice(String? said) {
    if (said == _notice) return;
    _notice = said;
    _tell(WsAspect.notice);
  }

  /// How many times the document has been read again.
  int rev = 0;

  /// How many times the document has been read again (not counting what a hand only carries): what is made from its
  /// shape (the cards) is made again when this moves.
  int reads = 0;

  /// Where the window's building spends its time (for the rampage's report): a name and the time it took, summed.
  static final spent = <String, Duration>{};
  /// The longest each named part took at once.
  static final slowest = <String, Duration>{};
  static T timed<T>(String name, T Function() does) {
    final w = Stopwatch()..start();
    final made = Timeline.timeSync(name, does); // (named in Flutter's own timeline too: frames_test TRACE)
    spent[name] = (spent[name] ?? Duration.zero) + w.elapsed;
    if (w.elapsed > (slowest[name] ?? Duration.zero)) slowest[name] = w.elapsed;
    return made;
  }

  /// Where a bar ends, in frames: a thing on for good runs to the document's end.
  static int _outF(int from, double lasts) => lasts < 0 ? duration : (from + (lasts * fps).round()).clamp(from + 1, duration);

  /// One thing's bar read again from the inside (while a hand carries it: what the inside says is what is drawn).
  void respan(String id) {
    final m = _inside, l = layers.where((l) => l.id == id).firstOrNull, t = int.tryParse(id);
    if (m == null || l == null || t == null) return;
    l.inF = ((m.value(t, 'start', 0) ?? 0) * fps).round();
    l.outF = _outF(l.inF, m.lasts(t));
    shown();
  }

  bool _gone = false;
  final seen = ValueNotifier<int>(0);

  /// The stage's pictures kept on disk for playing, as stretches of frames (the timeline marks them, as After Effects'
  /// green line): told again by the inside when they change.
  final kept = ValueNotifier<List<(int, int)>>(const []);

  @override
  void dispose() {
    _gone = true;
    seen.dispose();
    kept.dispose();
    moment.dispose();
    for (final n in on.values) {
      n.dispose();
    }
    super.dispose();
  }

  /// What the inside shows changed without the document changing (a thing held and carried): the seats look again.
  void shown() {
    lastChange = null; // (not a step of the document: what it changed is not known)
    rev++;
    _tell(WsAspect.document);
  }

  /// The document read again, after any change to it: what the seats list is what the document holds.
  /// The document's tempo, beats a minute (0: none), and its beats.
  double get bpm => _inside?.value(0, 'bpm', 0) ?? 0;
  WsBeats? get beats => WsBeats.of(bpm, _inside?.value(0, 'bpm_from', 0) ?? 0, Ws.fps);

  /// The document's frame rate (its `frame_rate`, the stage's own), or the inside's when it says none.
  static int _fpsOf(Motolii m) => switch ((m.value(0, 'frame_rate', 0) ?? 0).round()) { > 0 && final r => r, _ => m.rate };
  void reread() => Timeline.timeSync('ws reread', _reread); // (named in Flutter's timeline: frames_test TRACE)
  void _reread() {
    final m = _inside;
    if (m == null) return;
    if (!m.fits && refit != null) return refit!(); // (the document says another size, by a hand or by undo: the host opens it again, and that reads it)
    lastChange = m.changes();
    if (_fpsOf(m) != fps) {   // (the document's frame rate said again: the playhead kept where it is in time)
      final at = moment.value / fps;
      fps = _fpsOf(m);
      moment.value = (at * fps).round();
    }
    rev++;
    reads++;
    duration = Timeline.timeSync('MARK seconds', () => (m.seconds * fps).round().clamp(1, 1 << 20));
    wsChosenLabels
      ..clear()
      ..addAll({for (final t in m.things) if (m.words(t, 'label').indexOf(m.word(t, 'label')) case final i when i > 0) '$t': i - 1});
    final tl = TimelineTask()..start('MARK layers');
    layers
      ..clear()
      ..addAll([
        for (final t in m.things.reversed) // (the sample's manner: the nearest thing is the top row)
          () {
            final kind = m.word(t, 'kind'), from = ((m.value(t, 'start', 0) ?? 0) * fps).round(), lasts = m.lasts(t);
            final name = m.word(t, 'name'); // (what it is called is the inside's to say)
            // (a key on any value of it that takes keys is a key on its bar: the inside says which those are)
            final keys = <int>{for (final s in m.thingKeyMoments(t)) (s * fps).round()}.toList()..sort();
            return WsLayer('$t', name, switch (kind) {
              'text' => WsKind.text,
              'camera' => WsKind.camera,
              'film' || 'picture' => WsKind.image,
              _ => WsKind.shape,
            }, from, _outF(from, lasts), keys,
                hidden: (m.value(t, 'visible', 0) ?? 1) < .5, solo: (m.value(t, 'solo', 0) ?? 0) > .5, locked: (m.value(t, 'lock', 0) ?? 0) > .5, clipped: (m.value(t, 'clip', 0) ?? 0) > .5,
                stretches: [
                  for (final (a, b, way) in m.thingStretches(t))
                    (from: (a * fps).round(), to: (b * fps).round(), way: way, shape: easeShape(way, m, 16)),
                ]);
          }(),
      ]);
    if (_sel != null && layers.every((l) => l.id != _sel)) _sel = layers.firstOrNull?.id; // (a picked thing gone: the first; nothing picked stays so, the stage's)
    _keys.removeWhere((k) => !layers.any((l) => l.id == k.layer && l.keys.contains(k.frame))); // (a picked key stays picked while it is still there)
    if (_stretch case final s?) _stretch = _stretchOf(s.layer, s.name, s.from);   // (and a picked stretch: to where that value's next key now is)
    tl.finish();
    _tell(WsAspect.document);
  }

  /// The inside's own picture of the stage, as a Flutter texture, when the window is joined to it (null: the stand-in art).
  int? _stageTexture;
  int? get stageTexture => _stageTexture;
  set stageTexture(int? id) {
    if (id == _stageTexture) return;
    _stageTexture = id;
    _tell(WsAspect.stage);
  }

  /// The things whose moving values the window shows (told the inside: Motolii.subscribe): the picked one and those picked
  /// with it, the cameras (the Depth desk's lens), the relations (the Relations desk's map), and a picked relation's things.
  void _watch() {
    final m = _inside;
    if (m == null) return;
    final sel = int.tryParse(_sel ?? '');
    m.subscribe({
      ?sel,
      for (final id in many) ?int.tryParse(id),
      for (final l in layers) if (l.kind == WsKind.camera) ?int.tryParse(l.id),
      for (final l in layers) if (int.tryParse(l.id) case final t? when m.word(t, 'kind') == 'behaviour') t,   // (the relations: the Relations desk draws each by its knobs, at the moment)
      if (sel != null && m.word(sel, 'kind') == 'behaviour') for (final w in m.word(sel, 'members').split(' ')) ?int.tryParse(w),
    });
  }


  String? _sel; // (nothing picked, as a document opens)
  String? get selected => _sel;
  WsLayer? get layer => layers.where((l) => l.id == _sel).firstOrNull;
  void select(String? id) {
    if (id == _sel && many.isEmpty) return;
    _sel = id;
    many.clear();
    _aim = null;
    _keys.clear();
    _stretch = null;
    deskManual = null;
    _tell(WsAspect.picked);
  }

  /// The things picked together (drawn round on the stage): what a relation is made of. Picking one alone clears them.
  final many = <String>{};
  bool manyByHand = false;   // (picked one by one, in that order: the first leads; drawn round, they have no order)
  void pickMany(Iterable<String> ids, {bool byHand = false}) {
    many
      ..clear()
      ..addAll(ids);
    manyByHand = byHand;
    _tell(WsAspect.picked);
  }

  final _keys = <WsKey>{};
  Set<WsKey> get keys => _keys;
  void pickKey(WsKey k, {bool add = false}) {
    if (!add) _keys.clear();
    _keys.add(k);
    _stretch = null;
    _sel = k.layer;
    deskManual = null;
    _tell(WsAspect.picked);
  }

  /// Keys picked together (a box drawn round them): these and no others.
  void pickKeys(Iterable<WsKey> ks) {
    _keys
      ..clear()
      ..addAll(ks);
    _stretch = null;
    if (_keys.isNotEmpty) _sel = _keys.first.layer;
    deskManual = null;
    _tell(WsAspect.picked);
  }

  void clearKeys() {
    if (_keys.isEmpty && _stretch == null) return;
    _keys.clear();
    _stretch = null;
    _tell(WsAspect.picked);
  }

  /// The stretch picked by its line, as Theatre.js's connector between two keyframes is pressed to edit its curve
  /// (BasicKeyframeConnector.tsx, Apache-2.0): one value's, where a key is every value's there.
  WsStretchPicked? _stretch;
  WsStretchPicked? get stretch => _stretch;
  WsStretchPicked? _stretchOf(String layer, String name, int from) {
    final t = int.tryParse(layer), at = t == null ? const <double>[] : _inside?.keyMoments(t, name) ?? const <double>[];
    final i = at.indexWhere((s) => (s * Ws.fps).round() == from);
    return i < 0 || i + 1 >= at.length ? null : (layer: layer, name: name, at: at[i], from: from, to: (at[i + 1] * Ws.fps).round());
  }

  void pickStretch(String layer, String name, int from) {
    _keys.clear();
    _stretch = _stretchOf(layer, name, from);
    _sel = layer;
    deskManual = null;
    _tell(WsAspect.picked);
    if (_stretch != null) front('Ease');
  }

  /// The playhead, told on its own (Flutter's ValueNotifier): only what shows the moment listens ([WsScope.frameOf]),
  /// the rest of the window is not built again as it moves.
  final moment = WsMomentNow(112);
  int get frame => moment.value;
  set frame(int f) {
    final v = f.clamp(0, duration);
    if (v == moment.value) return;
    moment.value = v;
    if (!_playing) _inside?.at(v / fps);   // (standing: the stage goes to the moment the playhead is put at)
  }

  bool _playing = false;
  bool get playing => _playing;
  set playing(bool p) {
    if (p == _playing) return;
    _playing = p;
    if (p) _inside?.at(frame / fps);
    _inside?.play(p);   // (the inside keeps the clock; the playhead follows its moment: timeline.dart)
    _tell(WsAspect.playing);
  }

  /// The browser shelf in front: Media, Effects, Fonts, Colors, Create (Motolii's Browser shelves, each a pane of its own).
  String _shelf = 'Media';
  String get shelf => _shelf;
  set shelf(String s) {
    if (s == _shelf) return;
    _shelf = s;
    _tell(WsAspect.shelf);
  }

  /// How the window looks, by name ('stage.tool', 'fold.Transform'...): what a pane keeps of itself that is not the
  /// document's. Any pane may read it; the window opened again on the same document at the same view shows the same.
  final view = <String, Object?>{};
  T viewOf<T>(String name, T otherwise) => view[name] as T? ?? otherwise;
  void setView(String name, Object? value) {
    if (view[name] == value) return;
    view[name] = value;
    _tell(WsAspect.view);
  }

  /// What the user opened in the desk by hand; null lets the desk follow the selection.
  String? deskManual;

  /// The desk's tools: the window's own, then any of your own (Workspace(tools:)).
  List<DeskTool> tools = deskTools;
  void openDesk(String? name) {
    deskManual = name;
    front('Desk');
  }

  /// A pane brought to the front of its tabs (the panes follow: panes.dart).
  void front(String pane) {
    fronted = (pane, (fronted?.$2 ?? 0) + 1);
    _tell(WsAspect.fronted);
  }
  (String, int)? fronted;

  /// The desk follows the selection (Motolii panels/desk.dart): keys picked -> Ease, a camera -> Depth, else Tools.
  String get desk => deskManual ?? deskAuto;

  /// What the selection alone would put on the desk: keys picked, Ease; a camera, Depth; else the Tools catalog.
  String get deskAuto => 'Tools';   // (Ease and Camera are panes of their own now)

  /// Joined: the value of the picked thing (or of the stage, with nothing picked) a colour picked on the Colors shelf is
  /// said to, when a card's colour link or the tool strip's ink or edge aimed it there; null: the fill. Told as a pick
  /// (the strip rings the one aimed, the shelf's foot says where its colour goes).
  String? _aim;
  String? get aim => _aim;
  set aim(String? to) {
    if (to == _aim) return;
    _aim = to;
    _tell(WsAspect.picked);
  }

  /// A panel link elsewhere asked for a panel (fonts, colours, ease): the dock or desk shows it.
  void route(String to) {
    switch (to) {
      case final s when s.startsWith('Browser'):
        shelf = s.split('›').last.trim();   // (told as any shelf named is: the panes bring it to the front)
    }
    _tell(WsAspect.route);
  }

  /// The shelf's effects on the thing: those whose card the inside shows (an effect's card shows while it is on).
  List<String> fxOf(String? id) {
    final m = _inside, t = int.tryParse(id ?? '');
    if (m == null || t == null) return const [];
    final shown = {for (final (title, _) in m.cards(t)) title};
    return [for (final (name, _) in m.thingEffects) if (shown.contains(name)) name];
  }

  /// Joined: whether every change a hand makes is keyed (the top bar's Animate, the inspector's Animate all: one truth, the inside's).
  bool get animating => _inside?.keyingNow ?? false;

  /// Joined: the work area, the marks, the guides: the document's picture's.
  (double, double) get work => _inside!.work;
  List<(double, String)> get marks => _inside!.marks;
  Map<double, int> get markColours => _inside!.markColours;
  List<(String, double)> get guides => _inside!.guides;
  set animating(bool on) {
    _inside?.keying(on);
    _tell(WsAspect.animating);
  }


  /// Joined: a change asked without waiting ([Motolii.later]): looked at once a picture till the inside has done it, then
  /// the document read again and [then] told (done or not, and why not). The window never stands still for the inside.
  /// Joined: what the steps changed that the last reading of the document took in (null: not known, as after a hand's
  /// carry): a seat that shows only some values builds itself again only when they are among these.
  ({bool things, Set<(int, String)> values})? lastChange;

  int _afters = 0; // (tickets still looked at: the document is read again once, when the last of them is done)

  /// Whether changes told to the inside are not all done yet (the window shows them before the document has them).
  bool get waiting => _afters > 0;
  void after(int ticket, [void Function(bool done, String why)? then, void Function(int answer)? got]) {
    final m = _inside;
    if (m == null) return;
    _afters++;
    m.answered(ticket).then((r) {   // (told by the inside when it is done: nothing looked at each picture)
      final (a, why) = r;
      then?.call(a >= 1, why);
      if (a >= 1) got?.call(a);   // (an add's or a make's answer: the new thing)
      _afters--;   // (what it changed comes with the inside's next picture of the document)
    });
  }


  /// Joined: the effects on the picked thing (its fx: as the camera sees it, After Effects' effects on a layer), nearest
  /// the picture first. Effects are always a thing's: there is no stage-wide one (a whole picture's is a 2D plate showing
  /// what is behind it, as a lens, with the effect on the plate).
  static final _files = RegExp(r'/.*?\.(?:wgsl|fs|js|mjs|json|am)(?= |$)|[^ ]+'); // (as the inside reads the list: a path, spaces and all, to its extension: script.h script_files)
  List<String> get screenFx {
    final t = int.tryParse(selected ?? '');
    if (t == null) return const [];
    return [
      for (final f in _files.allMatches(_inside?.word(t, 'fx') ?? '').map((m) => m[0]!))
        if (f != 'none') f,
    ];
  }

  /// Joined: an effect put over the others on the picked thing, or taken off (one step either way). Nothing picked: nothing.
  bool screenToggle(String file) {
    final m = _inside, t = int.tryParse(selected ?? '');
    if (m == null || t == null) return false;
    // (at the end of its effects or out, by the inside's rule for its list; not taken, said why: a camera takes motions only)
    after(m.later('fx_file', thing: t, word: file), (done, why) => done || why.isEmpty ? null : notice = 'Not taken: ${why.split(':').first}');
    return true;
  }

  /// Joined: the motion the picked thing has (its script's file; '' none).
  String get motion {
    final t = int.tryParse(selected ?? '');
    final w = t == null ? '' : (_inside?.word(t, 'script') ?? '');
    return w == 'none' ? '' : w;
  }

  /// Joined: a motion given to the picked thing in place of the one it had, or taken off (one step either way).
  void motionToggle(String file) {
    final m = _inside, t = int.tryParse(selected ?? '');
    if (m == null || t == null) return;
    after(m.later('say', thing: t, name: 'script', word: motion == file ? 'none' : file));   // (without waiting)
  }

  /// Joined: an effect of the shelf put on the picked thing as it comes, or taken off (one step either way).
  void fxToggle(String name) {
    final m = _inside, t = int.tryParse(selected ?? '');
    if (m == null || t == null) return;
    after(m.later('effect', thing: t, name: name, number: frame / fps));   // (without waiting)
  }


  String timecode([int? f]) {
    final v = f ?? frame, s = v ~/ fps, fr = v % fps;
    return '${(s ~/ 60).toString().padLeft(2, '0')}:${(s % 60).toString().padLeft(2, '0')}:${fr.toString().padLeft(2, '0')}';
  }
}


/// The font a new text is set in, a font every machine of its kind has (one place: the Create shelf and the stage's tool).
final wsTextFont = Platform.isWindows ? 'C:/Windows/Fonts/arial.ttf' : '/System/Library/Fonts/Supplemental/Arial Unicode.ttf';

/// The chosen inks said to the thing just added (thing -1), wherever it was added from (P03: put after the key colours
/// are chosen, it comes in them; the stage's drawing gave them, the Fonts and Create shelves did not: golf_noise.md ㉛).
/// A text or a flat shape takes them, as the stage draws only those; a line its colour alone.
void wsInked(Ws ws, String kind, String shape) {
  final m = ws.inside;
  if (m == null || !(kind == 'text' || kind == 'shape' && shape != 'null')) return;
  final line = shape == 'line';
  if (ws.ink.isNotEmpty) m.later('say', thing: -1, name: 'fill', word: ws.ink);
  // (no ink chosen: a shape comes in a colour of its own, as Alight Motion's does: AbsentMotion's addShape, randomColor,
  //  any hue, saturation 0.3 to 0.8, value 0.55 to 1; said, so the Inspector and the Colors shelf show it: golf (85))
  if (ws.ink.isEmpty && kind == 'shape') {
    final r = math.Random(), c = HSVColor.fromAHSV(1, r.nextDouble() * 360, .3 + r.nextDouble() * .5, .55 + r.nextDouble() * .45).toColor();
    m.later('say', thing: -1, name: 'fill', word: [c.r, c.g, c.b].map((v) => (v * 255).round().toRadixString(16).padLeft(2, '0')).join());
  }
  if (ws.ink.contains(' ') && ws.inkBlend.isNotEmpty && !line) m.later('say', thing: -1, name: 'gradient_blend', word: ws.inkBlend);   // (a gradient ink, mixed as it was chosen)
  if (ws.edge.isNotEmpty && !line) {
    m
      ..later('say', thing: -1, name: 'stroke', word: ws.edge)
      ..later('set', thing: -1, name: 'stroke_width', number: 6);
  }
}

/// The window's state to the parts below (Flutter's InheritedModel): a part says which aspects it reads
/// ([WsScope.of]) and is built again only when one of those changed.
class WsScope extends StatefulWidget {
  const WsScope({super.key, required this.ws, required this.child});
  final Ws ws;
  final Widget child;
  /// The window's state, the part built again when one of [aspects] changes (none said: any, as a pane of your own
  /// may: the window's own parts each say theirs).
  static Ws of(BuildContext c, [Set<WsAspect>? aspects]) {
    for (final a in aspects ?? WsAspect.values) {
      InheritedModel.inheritFrom<_WsAll>(c, aspect: a);
    }
    return read(c);
  }
  static Ws read(BuildContext c) => c.getInheritedWidgetOfExactType<_WsAll>()!.ws;
  /// The playhead, and built again when it moves (nothing else of the window does that).
  static int frameOf(BuildContext c) => c.dependOnInheritedWidgetOfExactType<WsMoment>()!.notifier!.value;
  /// The inside's moment's picture: built again when a new one comes (what shows values at the moment shown).
  static int pictureOf(BuildContext c) => c.dependOnInheritedWidgetOfExactType<_WsPicture>()!.notifier!.value;
  @override
  State<WsScope> createState() => _WsScopeState();
}

class _WsScopeState extends State<WsScope> {
  late Map<WsAspect, int> _told = _now();
  Map<WsAspect, int> _now() => {for (final MapEntry(:key, :value) in widget.ws.on.entries) key: value.value};
  @override
  void initState() {
    super.initState();
    widget.ws.addListener(_changed);
  }
  @override
  void didUpdateWidget(WsScope old) {
    super.didUpdateWidget(old);
    if (old.ws != widget.ws) {
      old.ws.removeListener(_changed);
      widget.ws.addListener(_changed);
      _told = _now();
    }
  }
  @override
  void dispose() {
    widget.ws.removeListener(_changed);
    super.dispose();
  }
  void _changed() => setState(() => _told = _now());
  @override
  Widget build(BuildContext context) => _WsAll(ws: widget.ws, told: _told, child: WsMoment(notifier: widget.ws.moment, child: _WsPicture(notifier: widget.ws.seen, child: widget.child)));
}

class _WsPicture extends InheritedNotifier<ValueNotifier<int>> {
  const _WsPicture({required super.notifier, required super.child});
}

class _WsAll extends InheritedModel<WsAspect> {
  const _WsAll({required this.ws, required this.told, required super.child});
  final Ws ws;
  final Map<WsAspect, int> told;   // (how many times each aspect was told, as of this build)
  @override
  bool updateShouldNotify(_WsAll old) => !mapEquals(told, old.told) || ws != old.ws;
  @override
  bool updateShouldNotifyDependent(_WsAll old, Set<WsAspect> dependencies) => ws != old.ws || dependencies.any((a) => told[a] != old.told[a]);
}

/// The playhead: told when it moves, and again when what was read at it was put off (the inside busy).
class WsMomentNow extends ValueNotifier<int> {
  WsMomentNow(super.value);
  void again() => notifyListeners();
}

/// The playhead's own scope, under [WsScope]: [WsScope.frameOf].
class WsMoment extends InheritedNotifier<ValueNotifier<int>> {
  const WsMoment({super.key, required super.notifier, required super.child});
}
