// The workspace: one window that gathers the lab's panels (browser dock, stage, inspector, timeline, desk).
// This file is what every seat shares: the mock document they all read (Ws), and the window's own tokens (WsT).
// Tone and manner come from the Inspector's Pop rules (sets/inspector_parts.dart): containers square and tiled edge to edge,
// what you touch or read round, colour flat and opaque, one lime accent, each kind of thing wearing its own hue.
import 'dart:developer' show Timeline, TimelineTask;
import 'dart:io' show Platform;
import 'package:flutter/scheduler.dart';
import 'package:flutter/widgets.dart';

import '../motolii.dart';
import '../tokens.dart';
import 'desk_kit.dart' show DeskTool, deskTools;
import 'stage_art.dart' show WsArt;

/// The window's own sizes and colours. A seat names one of these; it never types a number that is the window's business.
abstract final class WsT {
  /// The gutter between seats: the dark you see between tiles.
  static const gutter = 2.0;

  /// A seat's header strip (title, tabs, small actions).
  static const header = 28.0;

  /// The stage's tool column.
  static const rail = 40.0;

  /// Seat widths and the lower band's height, at the reference window (1600 x 960).
  static const dock = 300.0, inspector = 372.0, lower = 300.0, topBar = 40.0;
  static const minWindow = Size(1280, 800), refWindow = Size(1600, 960);

  /// Grounds: the gutter, a seat's body, a card or row inside it, a well (field) inside that.
  static Color get ground => Grey.g00;
  static Color get body => Grey.g07;
  static Color get card => Grey.g13;
  static Color get well => Grey.g15;
  static Color get raised => Grey.g20;
  static Color get line => Grey.g20;

  /// The one accent (same as the Inspector's) and the ink drawn on it.
  static const accent = Color(0xFFB3C66B), accentHot = Color(0xFFC5D67E), onAccent = Color(0xFF15170D), keyDot = Color(0xFFF2C94C);

  /// The accent drawn as a line or text on a ground. Lime reads on dark but vanishes on white, so the light shade inks it darker; fills stay [accent].
  static Color get accentInk => Grey.light ? const Color(0xFF6B7A1E) : accent;

  /// Round things you touch: fields, chips, tabs, buttons.
  static const radius = 6.0, chipRadius = 4.0;

  /// Paddings inside a seat.
  static const inset = 8.0, gap = 4.0;

  /// Hues per kind of layer / asset, shared by timeline bars, browser tiles and the stage's artwork, so a thing keeps its colour everywhere.
  static const toneShape = Color(0xFFF08A5D), toneText = Color(0xFFD9D2C3), toneImage = Color(0xFF7FB8B0);
  static const toneCamera = Color(0xFFA7B4C8), toneGroup = Color(0xFFB0A6F0), toneParticles = Color(0xFFF2A65A), toneAudio = Color(0xFF5CC8E8);

  /// The playhead line: the accent, so "now" is the loudest line in the window. Its head is an [accent] fill.
  static Color get playhead => accentInk;
}

enum WsKind { shape, text, image, camera, group, particles, audio }

Color wsTone(WsKind k) => switch (k) {
  WsKind.shape => WsT.toneShape,
  WsKind.text => WsT.toneText,
  WsKind.image => WsT.toneImage,
  WsKind.camera => WsT.toneCamera,
  WsKind.group => WsT.toneGroup,
  WsKind.particles => WsT.toneParticles,
  WsKind.audio => WsT.toneAudio,
};

/// One layer of the mock composition: where it lives in time, and where its keys are (frames).
class WsLayer {
  WsLayer(this.id, this.name, this.kind, this.inF, this.outF, this.keys, {this.parent, this.hidden = false, this.solo = false, this.locked = false});
  final String id, name;
  final WsKind kind;
  int inF, outF;
  final List<int> keys;
  final String? parent;
  final bool hidden, solo, locked; // (as the inside said when the layers were last read: a row asks nothing of it while the stage plays)
}

/// A key the user picked: which layer, which frame.
typedef WsKey = ({String layer, int frame});

/// The window's state: the document's things as rows (read again from the inside), what is picked, the moment, the
/// view. Every seat listens to this and nothing else; a seat never keeps a copy of what is here.
/// contract: changes notify once per user action; nothing here ticks while idle (play advances only while [playing]).
class Ws extends ChangeNotifier {

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
    var asking = false;
    inside.again = () {   // (a read put off while the hand was busy: the seats look again a picture later, once)
      if (asking) return;
      asking = true;
      SchedulerBinding.instance.addPostFrameCallback((_) {
        asking = false;
        shown();
      });
      SchedulerBinding.instance.ensureVisualUpdate();
    };
    fps = inside.rate; // (a frame is the inside's: the timeline counts what a written film has)
    WsArt.comp = Size(inside.size.$1.toDouble(), inside.size.$2.toDouble());
    _frame = (inside.now * fps).round(); // (the playhead is the inside's moment from the first)
    reread();
  }

  /// Joined: the document's name, and how the window is told to open it again at another size (the host does it).
  String? document;
  String? documentFolder; // (where the document's file is: the dock offers it, so a new document is one step from its footage)
  void Function(int wide, int high)? onSize;
  VoidCallback? refit;

  /// Joined: how the document is written as a film (the host does it), and what the window has to say about the
  /// document just now (it is being written, where it was written, why something was not done); null: nothing.
  VoidCallback? onRender;
  /// Joined: the inks the drawing tools give a new thing (a fill word, a stroke colour; '': the document's own).
  String ink = '', edge = '';

  String? _notice;
  String? get notice => _notice;
  set notice(String? said) {
    if (said == _notice) return;
    _notice = said;
    notifyListeners();
  }

  /// How many times the document has been read again.
  int rev = 0;

  /// A value carried by a hand (the inspector's): only what shows the picked thing's numbers looks again, not the
  /// window (a step of a drag tells no other listener: [shown] at every step was 27 ms of the window's time).
  final carried = ValueNotifier<int>(0);

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

  /// What the inside shows changed without the document changing (a thing held and carried): the seats look again.
  void shown() {
    lastChange = null; // (not a step of the document: what it changed is not known)
    rev++;
    notifyListeners();
  }

  /// The document read again, after any change to it: what the seats list is what the document holds.
  void reread() => Timeline.timeSync('ws reread', _reread); // (named in Flutter's timeline: frames_test TRACE)
  void _reread() {
    final m = _inside;
    if (m == null) return;
    if (!m.fits && refit != null) return refit!(); // (the document says another size, by a hand or by undo: the host opens it again, and that reads it)
    m.forget();
    lastChange = m.changes();
    rev++;
    reads++;
    duration = Timeline.timeSync('MARK seconds', () => (m.seconds * fps).round().clamp(1, 1 << 20));
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
                hidden: (m.value(t, 'visible', 0) ?? 1) < .5, solo: (m.value(t, 'solo', 0) ?? 0) > .5, locked: (m.value(t, 'lock', 0) ?? 0) > .5);
          }(),
      ]);
    if (_sel != null && layers.every((l) => l.id != _sel)) _sel = layers.firstOrNull?.id; // (a picked thing gone: the first; nothing picked stays so, the stage's)
    _keys.removeWhere((k) => !layers.any((l) => l.id == k.layer && l.keys.contains(k.frame))); // (a picked key stays picked while it is still there)
    tl.finish();
    Timeline.timeSync('MARK notify', notifyListeners);
  }

  /// The inside's own picture of the stage, as a Flutter texture, when the window is joined to it (null: the stand-in art).
  int? _stageTexture;
  int? get stageTexture => _stageTexture;
  set stageTexture(int? id) {
    if (id == _stageTexture) return;
    _stageTexture = id;
    notifyListeners();
  }

  String? _sel; // (nothing picked, as a document opens)
  String? get selected => _sel;
  WsLayer? get layer => layers.where((l) => l.id == _sel).firstOrNull;
  void select(String? id) {
    if (id == _sel) return;
    _sel = id;
    aim = null;
    _keys.clear();
    deskManual = null;
    notifyListeners();
  }

  final _keys = <WsKey>{};
  Set<WsKey> get keys => _keys;
  void pickKey(WsKey k, {bool add = false}) {
    if (!add) _keys.clear();
    _keys.add(k);
    _sel = k.layer;
    deskManual = null;
    notifyListeners();
  }

  void clearKeys() {
    if (_keys.isEmpty) return;
    _keys.clear();
    notifyListeners();
  }

  int _frame = 112;
  int get frame => _frame;
  set frame(int f) {
    final v = f.clamp(0, duration);
    if (v == _frame) return;
    _frame = v;
    if (!_playing) _inside?.at(v / fps);   // (standing: the stage goes to the moment the playhead is put at)
    notifyListeners();
  }

  bool _playing = false;
  bool get playing => _playing;
  set playing(bool p) {
    if (p == _playing) return;
    _playing = p;
    if (p) _inside?.at(_frame / fps);
    _inside?.play(p);   // (the inside keeps the clock; the playhead follows its moment: timeline.dart)
    notifyListeners();
  }

  /// The browser shelf in front: Media, Effects, Fonts, Colors, Create (Motolii's Browser shelves, each a pane of its own).
  String _shelf = 'Media';
  String get shelf => _shelf;
  set shelf(String s) {
    if (s == _shelf) return;
    _shelf = s;
    notifyListeners();
  }

  /// How the window looks, by name ('stage.tool', 'fold.Transform'...): what a pane keeps of itself that is not the
  /// document's. Any pane may read it; the window opened again on the same document at the same view shows the same.
  final view = <String, Object?>{};
  T viewOf<T>(String name, T otherwise) => view[name] as T? ?? otherwise;
  void setView(String name, Object? value) {
    if (view[name] == value) return;
    view[name] = value;
    notifyListeners();
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
    notifyListeners();
  }
  (String, int)? fronted;

  /// The desk follows the selection (Motolii panels/desk.dart): keys picked -> Ease, a camera -> Depth, else Tools.
  String get desk => deskManual ?? deskAuto;

  /// What the selection alone would put on the desk: keys picked, Ease; a camera, Depth; else the Tools catalog.
  String get deskAuto => 'Tools';   // (Ease and Camera are panes of their own now)

  /// Joined: the value of the picked thing (or of the stage, with nothing picked) a colour picked on the Colors shelf is
  /// said to, when a card's colour link aimed it there; null: the fill.
  String? aim;

  /// A panel link elsewhere asked for a panel (fonts, colours, ease): the dock or desk shows it.
  void route(String to) {
    switch (to) {
      case final s when s.startsWith('Browser'):
        _shelf = s.split('›').last.trim();
    }
    notifyListeners();
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

  /// Joined: what changes only with the document (the work area, the marks, the guides), read once a reading of it:
  /// asked each picture they would wait for the inside's moment while it plays.
  (int, (double, double))? _work;
  (int, List<(double, String)>)? _marks;
  (int, List<(String, double)>)? _guides;
  (double, double) get work => (_work?.$1 == rev ? _work! : (_work = (rev, _inside!.work))).$2;
  List<(double, String)> get marks => (_marks?.$1 == rev ? _marks! : (_marks = (rev, _inside!.marks))).$2;
  List<(String, double)> get guides => (_guides?.$1 == rev ? _guides! : (_guides = (rev, _inside!.guides))).$2;
  set animating(bool on) {
    _inside?.keying(on);
    notifyListeners();
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
    int? answer;
    var waited = 0;
    _afters++;
    void look(Duration _) {
      final m = _inside;
      if (m == null) return;
      if (answer == null) {
        final a = m.answer(ticket);
        if (a >= 0) answer = a;
      }
      if (answer == null || (m.busy && waited++ < 3)) { // (done: read once the inside has told the stage the new moment too, so the reading does not wait on it; three pictures at most)
        SchedulerBinding.instance.scheduleFrameCallback(look);
        return;
      }
      final a = answer!;
      then?.call(a >= 1, a >= 1 ? '' : m.answerWhy);
      if (a >= 1) got?.call(a);   // (an add's or a make's answer: the new thing)
      if (--_afters == 0) reread();
    }

    SchedulerBinding.instance
      ..scheduleFrameCallback(look)
      ..ensureVisualUpdate();
  }

  /// Joined: the whole-screen effects on the stage (the files its script says, nearest the picture first).
  List<String> get screenFx => [
    for (final f in (_inside?.word(0, 'script') ?? '').split(' '))
      if (f.isNotEmpty && f != 'none') f,
  ];

  /// Joined: a whole-screen effect put over the others on the stage, or taken off (one step either way).
  bool screenToggle(String file) {
    final m = _inside;
    if (m == null) return false;
    final now = screenFx, on = now.remove(file);
    if (!on) now.add(file);
    after(m.later('say', name: 'script', word: now.isEmpty ? 'none' : now.join(' '))); // (an effect's shader made: the window does not wait for it)
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
    final v = f ?? _frame, s = v ~/ fps, fr = v % fps;
    return '${(s ~/ 60).toString().padLeft(2, '0')}:${(s % 60).toString().padLeft(2, '0')}:${fr.toString().padLeft(2, '0')}';
  }
}


/// The font a new text is set in, a font every machine of its kind has (one place: the Create shelf and the stage's tool).
final wsTextFont = Platform.isWindows ? 'C:/Windows/Fonts/arial.ttf' : '/System/Library/Fonts/Supplemental/Arial Unicode.ttf';

class WsScope extends InheritedNotifier<Ws> {
  const WsScope({super.key, required Ws ws, required super.child}) : super(notifier: ws);
  static Ws of(BuildContext c) => c.dependOnInheritedWidgetOfExactType<WsScope>()!.notifier!;
  static Ws read(BuildContext c) => c.getInheritedWidgetOfExactType<WsScope>()!.notifier!;
}

/// A seat's header: title on the left, [trailing] on the right, a hairline under it.
class WsHeader extends StatelessWidget {
  const WsHeader(this.title, {super.key, this.trailing = const [], this.leading});
  final String title;
  final List<Widget> trailing;
  final Widget? leading;
  @override
  Widget build(BuildContext context) => Container(
    height: WsT.header,
    padding: const EdgeInsets.symmetric(horizontal: WsT.inset),
    decoration: BoxDecoration(
      color: WsT.body,
      border: Border(bottom: BorderSide(color: WsT.line)),
    ),
    child: Row(
      children: [
        if (leading case final l?) ...[l, const SizedBox(width: 6)],
        if (title.isNotEmpty) Text(title, style: T.name(Grey.g95).copyWith(fontWeight: FontWeight.w700)),
        const Spacer(),
        ...trailing,
      ],
    ),
  );
}

/// A seat: a square tile of [WsT.body] that clips what it holds.
class WsSeat extends StatelessWidget {
  const WsSeat({super.key, required this.child});
  final Widget child;
  @override
  Widget build(BuildContext context) => ClipRect(
    child: ColoredBox(color: WsT.body, child: child),
  );
}

/// How many cells of at least [cell] px fit across [width] with [gap] between them, kept within [min]..[max].
/// Every grid in the window reads this, so a pane made wider gains columns instead of fatter tiles.
int wsCols(double width, double cell, {double gap = WsT.gutter, int min = 1, int max = 8}) => ((width + gap) / (cell + gap)).floor().clamp(min, max);

/// The width of one of [cols] cells across [width].
double wsCell(double width, int cols, {double gap = WsT.gutter}) => (width - gap * (cols - 1)) / cols;
