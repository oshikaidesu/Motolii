part of 'editor_session.dart';

/// The state every responsibility reads, and the seams between them. A mixin
/// never calls another mixin's member directly: it calls one of the members
/// declared abstract here, so the traffic across a boundary is countable.
abstract class SessionCore {
  final _bridge = NativeBridge();
  int? _attachmentId;
  final _runtimeEpoch = ValueNotifier<int>(0);
  ValueListenable<int> get runtimeEpoch => _runtimeEpoch;
  final document = ValueNotifier<Map<String, dynamic>>({});
  Map<String, dynamic> get state => document.value;
  bool get animating => state['animate'] == true;

  /// On by default: the frame Animate was turned on at becomes the first key
  /// of anything touched later at another frame. Settings can turn it off.
  bool get animateFrom => deskWork.value['animateFrom'] != false;

  /// Settings: the projection new material is born with. 2.5D faces the
  /// camera wherever it sits; 3D stands in the world and turns with it.
  String get flatProjection =>
      deskWork.value['flatProjection'] == '3D' ? '3D' : '2.5D';
  String? _sentFlatProjection = '2.5D';

  Map<String, dynamic> get newKeyShape {
    final chosen = EditorSession.map(deskWork.value['newKeyShape']);
    return chosen.isEmpty ? Map.of(EditorSession.easyEase) : chosen;
  }

  /// The key selection before the current one, with the layers it sits on,
  /// so a shortcut can bring a motion back without a trip to the Timeline.
  Map<String, dynamic>? previousKeys;

  /// The output picture (Camera view). Playback gates on it.
  final textureId = ValueNotifier<int?>(null);

  /// One Flutter texture per live view: `Camera` (the output) and `User` (the Stage).
  final textureIds = ValueNotifier<Map<String, int>>({});
  final frame = ValueNotifier<int>(0);
  final rendered = ValueNotifier<Map<String, dynamic>>({});
  Future<bool> Function()? confirmClose;
  void Function(Map<String, dynamic>)? windowClosed;
  void Function(List<String>)? filesDropped;
  bool Function(Map<String, dynamic>)? fileDropTarget;
  final pendingEditors = <Future<void> Function()>{};

  final editingFocus = ValueNotifier<Map<String, dynamic>>({});
  final textStyleTarget = ValueNotifier<Map<String, dynamic>?>(null);
  final focusProperty = ValueNotifier<String?>(null);

  /// An anchor the pointer hovers in the Inspector, as a fraction of the
  /// layer's bounds; the Stage marks where the pivot would land.
  final anchorPreview = ValueNotifier<List<double>?>(null);
  final keyedOnly = ValueNotifier<bool>(false);
  final viewCommand = ValueNotifier<String?>(null);
  Map<String, dynamic> windowInfo = {};
  final playing = ValueNotifier<bool>(false);
  final busy = ValueNotifier<bool>(false);

  /// Files are being carried over this window; the shelves can say "drop here".
  final dragging = ValueNotifier<bool>(false);

  final error = ValueNotifier<String?>(null);

  /// 直前の取り込みで棚に入った asset の id。Browser が Media を開いて選ぶ。
  final importedAssets = ValueNotifier<List<String>>([]);

  /// While on, the next click on the Stage reads a colour instead of editing.
  final eyedropper = ValueNotifier<bool>(false);

  /// Timeline に見えているコマ数。尺の無い物を置く時の既定の長さの元(Rust が割合を決める)。
  final visibleFrames = ValueNotifier<int?>(null);
  final deskWork = ValueNotifier<Map<String, dynamic>>({});
  final panePlaces = ValueNotifier<Map<String, dynamic>>({});
  Future<void> Function(String, String)? panelPlacementRequested;
  final deskDefault = ValueNotifier<String>('Tools');
  final deskDrawer = ValueNotifier<String?>(null);
  final browserTab = ValueNotifier<String>('Create');

  Future<void> _tail = Future.value();
  bool _disposed = false, _playRequested = false, _pauseQueued = false;
  int _pendingWork = 0;
  int _generation = 0;

  /// Replies that arrive while a frame is being built wait until it is done:
  /// a build may not mark widgets outside its own subtree.
  List<void Function()>? _deferred;

  /// The same-frame path, once the host has handed over its runtime. Null in
  /// tests and while no document is open: everything then goes by channel.
  FfiFrames? _frames;
  bool get sameFrame => _frames != null;

  /// Panel windows the host shows besides this one; they read the document
  /// through the host's broadcast, which this window feeds after each reply.
  int _panelWindows = 0;

  List<Map<String, dynamic>> get layers => EditorSession.maps(state['layers']);
  List<int> get selectedIds =>
      (state['selectedIds'] as List? ?? [state['selectedId']])
          .whereType<num>()
          .map((n) => n.toInt())
          .toList();

  // 継ぎ目(口)。左が呼ぶ側の責任、右が答える責任。
  // native ↔ snapshot / render / commands / desk
  Future<dynamic> native(String method, [Map<String, dynamic> args = const {}]);
  Future<void> _serial(
    Future<void> Function() action, {
    bool displayBusy = true,
  });
  Future<dynamic> _request(
    DocumentOperation operation, [
    Map<String, dynamic> args = const {},
  ]);
  Map<String, dynamic> _requestNow(
    FfiFrames frames,
    DocumentOperation operation,
    Map<String, dynamic> args,
  );
  void _accept(dynamic reply, {bool notify = true});
  void _broadcast(
    String status, {
    bool frameReady = false,
    bool frameOnly = false,
  });
  void _flushDeferred();
  bool supports(String op);
  void absorb(Map<String, dynamic> next);
  // 口をまたぐのは既定のまま描く時だけ。notify/playback は render の中の話。
  Future<void> _render();
  bool _renderNow(FfiFrames frames);
  Future<void> refreshPreview();
  void _clearSurfaces();
  bool get _cadenceRunning;
  void _beginCadence(int generation);
  void _cancelCadence();
  void _schedulePause({required bool renderFinal});
  void _startPlayback();
  void stopPlayback();
  Future<void> command(String op, [Map<String, dynamic> args = const {}]);
  Future<void> flushEditors();
  Future<void> placePanel(String name, String placement);
  void _syncPreferences();
}
