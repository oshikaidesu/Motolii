import 'dart:math' as math;

import 'package:flutter/scheduler.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../session/editor_session.dart';
import '../timeline/geometry.dart';
import '../timeline/rows.dart';

/// Timeline の骨格 — 書類の切片、行の組み立て、板の寸法、x とコマの読み替え、
/// そして再生ヘッドの要求。他の三つ(掴み手・見る位置・献立)はここの上に乗る。
mixin TimelineFrame<T extends StatefulWidget> on State<T> {
  EditorSession get timelineSession;
  TimelineGeometry get timelineGeometry => TimelineGeometry.classic;
  double get labelWidth => layout.nameWidth + 82;
  final expanded = <int>{};
  final allProperties = <int>{};
  final collapsedGroups = <int>{};
  final vertical = ScrollController(), horizontal = ScrollController();
  final focus = FocusNode();
  double pixelsPerFrame = 4;
  double viewportWidth = 1;
  bool overTimeRuler(Offset p) =>
      p.dx >= labelWidth &&
      p.dy >= timelineGeometry.rulerTop &&
      p.dy < timelineGeometry.rulerTop + timelineGeometry.rulerHeight;
  double baseNameWidth = 138;
  double resizeStart = 138;
  double resizePointerStart = 0;
  String? activeLane;
  List<TrackRow> tracks = [];
  LaneLayout layout = LaneLayout([], {}, {}, {});
  final rowsKey = GlobalKey();
  Future<void>? seekFlight;
  int? pendingSeek;

  /// 掴んでいる間のヘッドの位置。絵(native の seek と render)が返るのを待たずにここへ描く。
  /// ヘッドを読むのは三枚の painter だけなので、State ではなくここに置く。
  /// setState にすると、線を動かすたびに行も lane も組み直すことになる。
  final scrubFrame = ValueNotifier<int?>(null);
  void requestSeek(int value) {
    // 再生中でも止めない: 飛ぶだけで回り続ける(Ableton と同じ)。
    pendingSeek = value;
    scrubFrame.value = value;
    seekFlight ??= pumpSeek();
  }

  Future<void> pumpSeek() async {
    try {
      while (pendingSeek != null) {
        final frame = pendingSeek!;
        pendingSeek = null;
        await timelineSession.command('seek', {'frame': frame});
      }
    } finally {
      seekFlight = null;
      if (mounted) scrubFrame.value = null;
    }
  }

  bool get primary =>
      HardwareKeyboard.instance.isMetaPressed ||
      HardwareKeyboard.instance.isControlPressed;
  double get offset =>
      horizontal.hasClients ? horizontal.positions.last.pixels : 0;

  /// 横に流れた合図。絵を持つのは目盛り・レーン・全体図の三つだけなので、
  /// 流すたびに板ごと建て直さず、その三つを起こす。
  final scrolled = ValueNotifier<int>(0);
  int get duration =>
      (timelineSession.state['durationFrames'] as num? ?? 1).toInt();

  /// 縮図(レンズ)が写す幅: 尺・今のコマ・一番遠い層の端(ゴースト込み)のうち一番遠い所に、
  /// 終わりの遊び(1 秒か全体の 1/8 の大きい方)を足した所まで。頭は 0 で詰める。
  /// 尺は壁ではなく印なので、越えた所も写す。
  int get overviewExtent {
    final farthest = math.max<int>(
      overviewExtentWithoutFrame,
      timelineSession.frame.value + 1,
    );
    final fps = (timelineSession.state['fps'] as num? ?? 30).round();
    return farthest + math.max<int>(fps, farthest ~/ 8);
  }

  List<Map<String, dynamic>> get selectedKeys =>
      EditorSession.maps(timelineSession.state['selectedKeys']);
  bool has(String op) => timelineSession.supports(op);
  @override
  void initState() {
    super.initState();
    horizontal.addListener(changed);
    focus.addListener(laneFocusChanged);
    timelineSession.frame.addListener(frameMoved);
    // 一番に並びを組む。目盛りもレーンも横帯も、後から同じ書類を見る。
    timelineSlice.addListener(relane);
    relane();
  }

  /// 今のコマが帯の端を越えたら、見えている幅の 2 倍ぶんまとめて伸ばす(尺は壁ではない)。
  /// 越えていない間は組み直さない。
  void frameMoved() {
    if (!mounted) return;
    final frame = timelineSession.frame.value;
    final base = overviewExtentWithoutFrame;
    if (frame + 1 <= base + grownFrames) return;
    final visibleFrames =
        (math.max(1, viewportWidth - labelWidth) / pixelsPerFrame).ceil();
    setState(() => grownFrames = frame + 1 - base + visibleFrames * 2);
  }

  /// 縮図のうち、今のコマと遊びを除いた「物がある所」の端。

  int? _extentCache;
  int get overviewExtentWithoutFrame => _extentCache ??= _measureExtent();
  int _measureExtent() {
    var extent = duration;
    for (final l in timelineSession.layers) {
      final int end =
          (l['start'] as num? ?? 0).toInt() +
          (l['duration'] as num? ?? 0).toInt() +
          math.max<int>(0, (l['ghost'] as num? ?? 0).toInt());
      extent = math.max<int>(extent, end);
    }
    return extent;
  }

  void laneFocusChanged() {
    if (!focus.hasFocus && mounted) setState(() => activeLane = null);
  }

  void changed() {
    if (mounted) scrolled.value++;
  }

  /// 尺は壁ではない: 頭は 0 で止め、先は自由(再生と同じ)。
  int frameAt(double x) =>
      math.max(0, ((x - labelWidth + offset) / pixelsPerFrame).round());

  /// 帯の幅: 物のある所 + まとめて伸ばした分 + 見えている幅ぶんの余白(先へ掴んで行ける)。
  /// 今のコマに毎コマ追随させると Timeline 全体を毎コマ組み直してカタつくので、伸ばすのは塊で。
  int grownFrames = 0;
  double get contentWidth =>
      (overviewExtentWithoutFrame +
          grownFrames +
          math.max(1, viewportWidth - labelWidth) / pixelsPerFrame) *
      pixelsPerFrame;

  DocumentSlice get timelineSlice => timelineSession.slice('timeline', const [
    'layers',
    'selectedId',
    'selectedIds',
    'selectedKeys',
    'durationFrames',
    'fps',
    'fpsNum',
    'fpsDen',
    'markers',
    'capabilities',
  ]);

  /// 板の骨格は寸法だけで建つ。目盛りより上の帯は [ValueListenableBuilder] の
  /// `child` として一度だけ建て、書類が動いても建て直さない。
  /// 帯の並びは書類が動いた時に一度だけ組む。板の骨格はこれを読むだけで、
  /// 書類を購読するのは目盛り・レーン・横帯・掴み手の四つに限る。
  void relane() {
    _extentCache = null;
    final liveIds = timelineSession.layers
        .map((layer) => (layer['id'] as num).toInt())
        .toSet();
    expanded.retainAll(liveIds);
    allProperties.retainAll(liveIds);
    collapsedGroups.retainAll(liveIds);
    layout = LaneLayout(
      timelineSession.layers,
      expanded,
      allProperties,
      collapsedGroups,
      baseNameWidth: baseNameWidth,
      geometry: timelineGeometry,
    );
    tracks = layout.rows;
    if (activeLane != null && !tracks.any((row) => row.laneId == activeLane))
      activeLane = null;
    reportVisible();
  }

  /// How many frames the bar shows, for the session's zoom-to-fit.
  void reportVisible() {
    if (viewportWidth <= labelWidth) return;
    final visible = ((viewportWidth - labelWidth) / pixelsPerFrame).round();
    if (timelineSession.visibleFrames.value != visible)
      SchedulerBinding.instance.addPostFrameCallback((_) {
        if (mounted) timelineSession.visibleFrames.value = visible;
      });
  }

  @override
  void dispose() {
    pendingSeek = null;
    scrubFrame.dispose();
    horizontal.removeListener(changed);
    timelineSlice.removeListener(relane);
    timelineSession.frame.removeListener(frameMoved);
    horizontal.dispose();
    vertical.dispose();
    scrolled.dispose();
    focus.removeListener(laneFocusChanged);
    focus.dispose();
    super.dispose();
  }
}
