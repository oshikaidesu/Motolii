// The workspace: one window that gathers the lab's panels (browser dock, stage, inspector, timeline, desk).
// This file is what every seat shares: the mock document they all read (Ws), and the window's own tokens (WsT).
// Tone and manner come from the Inspector's Pop rules (sets/inspector_parts.dart): containers square and tiled edge to edge,
// what you touch or read round, colour flat and opaque, one lime accent, each kind of thing wearing its own hue.
import 'package:flutter/widgets.dart';

import '../motolii.dart';
import '../tokens.dart';
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
  WsLayer(this.id, this.name, this.kind, this.inF, this.outF, this.keys, {this.parent});
  final String id, name;
  final WsKind kind;
  int inF, outF;
  final List<int> keys;
  final String? parent;
}

/// A key the user picked: which layer, which frame.
typedef WsKey = ({String layer, int frame});

/// The shared mock document. Every seat listens to this and nothing else; a seat never keeps a copy of what is here.
/// contract: changes notify once per user action; nothing here ticks while idle (play advances only while [playing]).
class Ws extends ChangeNotifier {
  Ws() {
    layers.addAll([
      WsLayer('cam', 'Camera', WsKind.camera, 0, 240, [0, 120, 240]),
      WsLayer('title', 'CODA', WsKind.text, 12, 228, [12, 36, 180, 204]),
      WsLayer('blob', 'Blob', WsKind.shape, 0, 240, [0, 48, 96, 144]),
      WsLayer('ring', 'Ring', WsKind.shape, 24, 200, [24, 60, 120]),
      WsLayer('confetti', 'Confetti', WsKind.particles, 40, 240, [40, 90]),
      WsLayer('chips', 'Chips', WsKind.group, 60, 220, [60, 100, 160]),
      WsLayer('bg', 'Background', WsKind.image, 0, 240, const []),
      WsLayer('music', 'Beat.wav', WsKind.audio, 0, 240, const []),
    ]);
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
    rev++;
    notifyListeners();
  }

  /// The document read again, after any change to it: what the seats list is what the document holds.
  void reread() {
    final m = _inside;
    if (m == null) return;
    if (!m.fits && refit != null) return refit!(); // (the document says another size, by a hand or by undo: the host opens it again, and that reads it)
    rev++;
    duration = (m.seconds * fps).round().clamp(1, 1 << 20);
    layers
      ..clear()
      ..addAll([
        for (final t in m.things.reversed) // (the sample's manner: the nearest thing is the top row)
          () {
            final kind = m.word(t, 'kind'), from = ((m.value(t, 'start', 0) ?? 0) * fps).round(), lasts = m.lasts(t);
            final name = m.word(t, 'name'); // (what it is called is the inside's to say)
            // (a key on any value of it that takes keys is a key on its bar: the inside says which those are)
            final keys = <int>{for (final v in m.moving(t)) ...m.keyMoments(t, v).map((s) => (s * fps).round())}.toList()..sort();
            return WsLayer('$t', name, switch (kind) {
              'text' => WsKind.text,
              'camera' => WsKind.camera,
              'film' || 'picture' => WsKind.image,
              _ => WsKind.shape,
            }, from, _outF(from, lasts), keys);
          }(),
      ]);
    if (layers.every((l) => l.id != _sel)) _sel = layers.firstOrNull?.id;
    _keys.removeWhere((k) => !layers.any((l) => l.id == k.layer && l.keys.contains(k.frame))); // (a picked key stays picked while it is still there)
    notifyListeners();
  }

  /// The inside's own picture of the stage, as a Flutter texture, when the window is joined to it (null: the stand-in art).
  int? _stageTexture;
  int? get stageTexture => _stageTexture;
  set stageTexture(int? id) {
    if (id == _stageTexture) return;
    _stageTexture = id;
    notifyListeners();
  }

  String? _sel = 'blob';
  String? get selected => _sel;
  WsLayer? get layer => layers.where((l) => l.id == _sel).firstOrNull;
  void select(String? id) {
    if (id == _sel) return;
    _sel = id;
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

  /// What the user opened in the desk by hand; null lets the desk follow the selection.
  String? deskManual;
  void openDesk(String? name) {
    deskManual = name;
    notifyListeners();
  }

  /// The desk follows the selection (Motolii panels/desk.dart): keys picked -> Ease, a camera -> Depth, else Tools.
  String get desk => deskManual ?? (_keys.isNotEmpty ? 'Ease' : (layer?.kind == WsKind.camera ? 'Depth' : 'Tools'));

  /// A panel link elsewhere asked for a panel (fonts, colours, ease): the dock or desk shows it.
  void route(String to) {
    switch (to) {
      case 'Ease':
        deskManual = 'Ease';
      case final s when s.startsWith('Browser'):
        _shelf = s.split('›').last.trim();
    }
    notifyListeners();
  }

  /// Effects kept on each layer, in stack order (layer id -> effect names). Joined: the shelf's effects the document
  /// has on the thing (a value said, and not nought).
  final _fx = <String, List<String>>{};
  List<String> fxOf(String? id) {
    if (_inside case final m?) {
      final t = int.tryParse(id ?? '');
      if (t == null) return const [];
      return [
        for (final e in fxValues.entries)
          if (_on(m, t, e.value.$1)) e.key,
      ];
    }
    return List.unmodifiable(_fx[id] ?? const <String>[]);
  }

  /// Joined: whether every change a hand makes is keyed (the top bar's Animate, the inspector's Animate all: one truth, the inside's).
  bool get animating => _inside?.keyingNow ?? false;
  set animating(bool on) {
    _inside?.keying(on);
    notifyListeners();
  }

  /// Joined: whether a value that puts an effect on a thing is on (keyed, or said and not nought).
  static bool _on(Motolii m, int t, String name) {
    final w = m.word(t, name);
    return m.keyMoments(t, name).isNotEmpty || (w.isNotEmpty && (m.range(t, name) == null || (double.tryParse(w) ?? 1) != 0)); // (a colour said is on, black too; a number, when not nought)
  }

  /// Joined: an effect of the shelf put on the picked thing as it comes, or taken off (one step either way).
  bool fxToggle(String name) {
    final m = _inside, t = int.tryParse(selected ?? ''), v = fxValues[name];
    if (m == null || t == null || v == null) return false;
    final on = fxOf(selected).contains(name);
    final at = frame / fps;
    final done = switch (v.$2) {
      _ when on && v.$2 is! String => m.set(t, v.$1, 0), // (taken off: its keys too)
      final num n => m.set(t, v.$1, n.toDouble()),
      final List<(double, double)> keys => m.together(() => keys.every((k) => m.key(t, v.$1, at + k.$1, k.$2))),
      final w => m.say(t, v.$1, on ? '' : '$w'),
    };
    reread();
    return done;
  }

  /// The effect being tried on the selected layer: the renderer draws it as if added; Add keeps it, Esc drops it.
  String? _trial;
  String? get trial => canTry ? _trial : null;
  bool get canTry => _inside != null
      ? false // (joined: the inside has no effects yet, so there is nothing to try on)
      : switch (layer?.kind) {
    null || WsKind.audio || WsKind.camera => false,
    _ => true,
  };
  void tryFx(String? name) {
    if (name == _trial) return;
    _trial = name;
    notifyListeners();
  }

  void keepFx() {
    final l = layer, t = trial;
    if (l == null || t == null) return;
    (_fx[l.id] ??= []).add(t);
    _trial = null;
    notifyListeners();
  }

  void dropFx() => tryFx(null);

  String timecode([int? f]) {
    final v = f ?? _frame, s = v ~/ fps, fr = v % fps;
    return '${(s ~/ 60).toString().padLeft(2, '0')}:${(s % 60).toString().padLeft(2, '0')}:${fr.toString().padLeft(2, '0')}';
  }
}

/// Joined: the shelf's effects the inside has, by the document's value that puts each on a thing (ThorVG's own), and
/// what it is told as it is put on (the sample's Blur radius; After Effects' black shadow; all of the two or three colours).
const fxValues = <String, (String, Object)>{
  'Gaussian Blur': ('blur', 12),
  'Drop Shadow': ('shadow', '000000'),
  'Glow': ('glow', 'ffd166'), // (a warm light all round)
  'Trim Paths': ('trim', [(0.0, 1.0), (1.0, 0.0)]), // (drawn on from here, in a second: keys as After Effects' trim paths is mostly used)
  'Duotone': ('tint', 1),
  'Gradient Map': ('tritone', 1),
};

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
