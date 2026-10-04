// The workspace: one window that gathers the lab's panels (browser dock, stage, inspector, timeline, desk).
// This file is what every seat shares: the mock document they all read (Ws), and the window's own tokens (WsT).
// Tone and manner come from the Inspector's Pop rules (sets/inspector_parts.dart): containers square and tiled edge to edge,
// what you touch or read round, colour flat and opaque, one lime accent, each kind of thing wearing its own hue.
import 'package:flutter/widgets.dart';

import '../tokens.dart';

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
  static const fps = 30, duration = 240;

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
    notifyListeners();
  }

  bool _playing = false;
  bool get playing => _playing;
  set playing(bool p) {
    if (p == _playing) return;
    _playing = p;
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

  /// Effects kept on each layer, in stack order (layer id -> effect names).
  final _fx = <String, List<String>>{};
  List<String> fxOf(String? id) => List.unmodifiable(_fx[id] ?? const <String>[]);

  /// The effect being tried on the selected layer: the renderer draws it as if added; Add keeps it, Esc drops it.
  String? _trial;
  String? get trial => canTry ? _trial : null;
  bool get canTry => switch (layer?.kind) {
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
