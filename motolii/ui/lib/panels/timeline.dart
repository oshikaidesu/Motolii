import 'dart:math' as math;

import 'package:flutter/foundation.dart' show listEquals;
import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter/scheduler.dart';
import 'package:flutter/services.dart';

import '../session/editor_session.dart';
import '../foundation/theme.dart';
import '../input/viewport_motion.dart';
import '../foundation/metrics.dart';
import '../foundation/panel_controls.dart';

part 'timeline_layout.dart';

class TimelinePanel extends StatefulWidget {
  const TimelinePanel({super.key, required this.controller});
  final EditorSession controller;
  @override
  State<TimelinePanel> createState() => _TimelinePanelState();
}

class _TimelinePanelState extends State<TimelinePanel> {
  static const rowHeight = 20.0, rulerHeight = 32.0;
  double get labelWidth => layout.nameWidth + 82;
  final expanded = <int>{};
  final allProperties = <int>{};
  final collapsedGroups = <int>{};
  final vertical = ScrollController(), horizontal = ScrollController();
  final focus = FocusNode();
  double pixelsPerFrame = 4;
  double viewportWidth = 1;
  Offset navigationOrigin = Offset.zero;
  int navigationRevision = 0;
  bool navigating = false;
  ViewportMotion? _motion;
  ViewportMotion get motion => _motion ??= ViewportMotion(navigateView);
  bool overTimeRuler(Offset p) =>
      p.dx >= labelWidth && p.dy >= 22 && p.dy < 22 + rulerHeight;

  double baseNameWidth = 138;
  double resizeStart = 138;
  double resizePointerStart = 0;
  int? anchor;
  String? activeLane;
  List<_TrackRow> tracks = [];
  _LaneLayout layout = _LaneLayout([], {}, {}, {});
  String? gesture;
  List<int> rowDragIds = [];
  Map<String, dynamic>? rowDrop;
  Offset? assetDrop;
  final rowsKey = GlobalKey();
  Rect? rowDropGuide;
  bool rowDropInside = false;
  Offset? start, current;
  _TrackRow? dragRow;
  List<_TrackRow> timingRows = [];
  Future<void>? previewFlight;
  Future<void>? seekFlight;
  int? pendingSeek;

  /// 掴んでいる間のヘッドの位置。絵(native の seek と render)が返るのを待たずにここへ描く。
  int? scrubFrame;
  void requestSeek(int value) {
    // 再生中でも止めない: 飛ぶだけで回り続ける(Ableton と同じ)。
    pendingSeek = value;
    setState(() => scrubFrame = value);
    seekFlight ??= pumpSeek();
  }

  Future<void> pumpSeek() async {
    try {
      while (pendingSeek != null) {
        final frame = pendingSeek!;
        pendingSeek = null;
        await widget.controller.command('seek', {'frame': frame});
      }
    } finally {
      seekFlight = null;
      if (mounted) setState(() => scrubFrame = null);
    }
  }

  List<Map<String, dynamic>>? queuedTimings;
  bool previewUsed = false;
  bool finishing = false;
  Future<void> pumpPreview() async {
    try {
      while (queuedTimings != null) {
        final changes = queuedTimings!;
        queuedTimings = null;
        await widget.controller.command('previewTimings', {'changes': changes});
      }
    } finally {
      previewFlight = null;
    }
  }

  Future<void> finishTiming(List<Map<String, dynamic>> changes) async {
    finishing = true;
    final used = previewUsed;
    previewUsed = false;
    await previewFlight;
    if (used)
      await widget.controller.command('commitPreview');
    else if (has('setTimings'))
      await widget.controller.command('setTimings', {'changes': changes});
    else if (changes.length == 1)
      await widget.controller.command('setTiming', changes.single);
    finishing = false;
  }

  int deltaFrames = 0;
  List<Map<String, dynamic>> initialKeys = [];
  List<int> initialIds = [];
  bool additive = false;
  bool get primary =>
      HardwareKeyboard.instance.isMetaPressed ||
      HardwareKeyboard.instance.isControlPressed;
  double get offset =>
      horizontal.hasClients ? horizontal.positions.last.pixels : 0;

  /// 横に流れた合図。絵を持つのは目盛り・レーン・全体図の三つだけなので、
  /// 流すたびに板ごと建て直さず、その三つを起こす。
  final scrolled = ValueNotifier<int>(0);
  int get duration =>
      (widget.controller.state['durationFrames'] as num? ?? 1).toInt();

  /// 縮図(レンズ)が写す幅: 尺・今のコマ・一番遠い層の端(ゴースト込み)のうち一番遠い所に、
  /// 終わりの遊び(1 秒か全体の 1/8 の大きい方)を足した所まで。頭は 0 で詰める。
  /// 尺は壁ではなく印なので、越えた所も写す。
  int get overviewExtent {
    final farthest = math.max<int>(
      overviewExtentWithoutFrame,
      widget.controller.frame.value + 1,
    );
    final fps = (widget.controller.state['fps'] as num? ?? 30).round();
    return farthest + math.max<int>(fps, farthest ~/ 8);
  }

  List<Map<String, dynamic>> get selectedKeys =>
      EditorSession.maps(widget.controller.state['selectedKeys']);
  bool has(String op) => widget.controller.supports(op);
  @override
  void initState() {
    super.initState();
    horizontal.addListener(changed);
    focus.addListener(laneFocusChanged);
    widget.controller.frame.addListener(frameMoved);
    // 一番に並びを組む。目盛りもレーンも横帯も、後から同じ書類を見る。
    _timeline.addListener(_relane);
    _relane();
  }

  /// 今のコマが帯の端を越えたら、見えている幅の 2 倍ぶんまとめて伸ばす(尺は壁ではない)。
  /// 越えていない間は組み直さない。
  void frameMoved() {
    if (!mounted) return;
    final frame = widget.controller.frame.value;
    final base = overviewExtentWithoutFrame;
    if (frame + 1 <= base + grownFrames) return;
    final visibleFrames =
        (math.max(1, viewportWidth - labelWidth) / pixelsPerFrame).ceil();
    setState(() => grownFrames = frame + 1 - base + visibleFrames * 2);
  }

  /// 縮図のうち、今のコマと遊びを除いた「物がある所」の端。

  int get overviewExtentWithoutFrame {
    var extent = duration;
    for (final l in widget.controller.layers) {
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

  @override
  void dispose() {
    _motion?.dispose();
    pendingSeek = null;
    queuedTimings = null;
    if (previewUsed) widget.controller.cancelPreview();
    horizontal.removeListener(changed);
    _timeline.removeListener(_relane);
    widget.controller.frame.removeListener(frameMoved);
    horizontal.dispose();
    vertical.dispose();
    scrolled.dispose();
    focus.removeListener(laneFocusChanged);
    focus.dispose();
    super.dispose();
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

  /// Both ends of the span under x on a property row, or null off any span.
  List<Map<String, dynamic>>? spanAt(_TrackRow row, double x) {
    for (var i = 0; i + 1 < row.keys.length; i++) {
      final a = row.keys[i], b = row.keys[i + 1];
      final x1 =
          labelWidth + (a['frame'] as num).toDouble() * pixelsPerFrame - offset;
      final x2 =
          labelWidth + (b['frame'] as num).toDouble() * pixelsPerFrame - offset;
      if (x > x1 + 7 && x < x2 - 7) return [keyOf(row, a), keyOf(row, b)];
    }
    return null;
  }

  Map<String, dynamic> keyOf(_TrackRow row, Map<String, dynamic> key) => {
    'layer': row.id,
    'property': row.property?['id'],
    'frame': key['frame'],
  };
  bool sameKey(Map<String, dynamic> a, Map<String, dynamic> b) =>
      a['layer'] == b['layer'] &&
      a['property'] == b['property'] &&
      a['frame'] == b['frame'];
  void chooseLayer(int id) {
    var ids = widget.controller.selectedIds.toList();
    if (HardwareKeyboard.instance.isShiftPressed && anchor != null) {
      final ordered = widget.controller.layers
          .map((l) => (l['id'] as num).toInt())
          .toList();
      final a = ordered.indexOf(anchor!), b = ordered.indexOf(id);
      if (a >= 0 && b >= 0) {
        final range = ordered.sublist(math.min(a, b), math.max(a, b) + 1);
        ids = primary ? {...ids, ...range}.toList() : range;
      }
    } else if (primary) {
      ids.contains(id) ? ids.remove(id) : ids.add(id);
    } else {
      ids = [id];
    }
    anchor = id;
    widget.controller.command('select', {'ids': ids, 'keys': []});
  }

  void begin(PointerDownEvent event) {
    focus.requestFocus();
    if (finishing) return;
    if (event.buttons != kPrimaryMouseButton) return;
    final p = event.localPosition, index = layout.rowAt(p.dy);
    start = p;
    current = p;
    deltaFrames = 0;
    additive = primary || HardwareKeyboard.instance.isShiftPressed;
    initialKeys = selectedKeys;
    initialIds = widget.controller.selectedIds.toList();
    if (index < 0 || index >= tracks.length) {
      activeLane = null;
      gesture = 'marquee';
      setState(() {});
      return;
    }
    final row = tracks[index];
    setState(() => activeLane = row.laneId);
    dragRow = row;
    if (p.dx < labelWidth) {
      if (row.property == null &&
          p.dx >= labelWidth - 81 &&
          p.dx < labelWidth - 65) {
        setState(() {
          allProperties.remove(row.id);
          expanded.contains(row.id)
              ? expanded.remove(row.id)
              : expanded.add(row.id);
          _relane();
        });
        gesture = null;
        return;
      }
      if (row.property == null &&
          row.isGroup &&
          row.disclosureBounds.contains(p)) {
        setState(() {
          collapsedGroups.contains(row.id)
              ? collapsedGroups.remove(row.id)
              : collapsedGroups.add(row.id);
          _relane();
        });
        gesture = null;
        return;
      }
      if (row.property == null && p.dx >= labelWidth - 65) {
        final column = ((p.dx - (labelWidth - 65)) / 16).floor();
        final field = [
          'hidden',
          'solo',
          'locked',
          'clipToBelow',
        ][column.clamp(0, 3)];
        if (field == 'clipToBelow') {
          if (has('clip')) widget.controller.command('clip', {'layer': row.id});
        } else if (has('setAttrs'))
          widget.controller.command('setAttrs', {
            'layers': [row.id],
            'patch': {field: row.layer[field] != true},
          });
        gesture = null;
        return;
      }
      if (row.property != null && p.dx >= labelWidth - 25 && has('toggleKey')) {
        widget.controller.command('toggleKey', {
          'layer': row.id,
          'property': row.property!['id'],
        });
        gesture = null;
        return;
      }
      if (row.property == null) {
        rowDragIds = initialIds.contains(row.id)
            ? initialIds.toList()
            : [row.id];
        if (!initialIds.contains(row.id)) chooseLayer(row.id);
        gesture = 'rowPending';
      } else {
        chooseLayer(row.id);
        gesture = null;
      }
      return;
    }
    if (row.property == null && !row.lanesOpen) {
      int? frame;
      for (final f in row.summaryFrames) {
        if ((labelWidth + f * pixelsPerFrame - offset - p.dx).abs() <= 7) {
          frame = f;
          break;
        }
      }
      if (frame != null) {
        final found = row.allKeys.where((k) => k['frame'] == frame).toList();
        var chosen = selectedKeys.toList();
        final already = found.every((f) => chosen.any((k) => sameKey(k, f)));
        if (additive) {
          already
              ? chosen.removeWhere((k) => found.any((f) => sameKey(k, f)))
              : chosen.addAll(
                  found.where((f) => !chosen.any((k) => sameKey(k, f))),
                );
        } else if (!already)
          chosen = found;
        widget.controller.command('select', {
          'ids': chosen.map((k) => k['layer']).toSet().toList(),
          'keys': chosen,
        });
        initialKeys = chosen;
        gesture = has('moveKeys') ? 'keys' : null;
        setState(() {});
        return;
      }
    }
    if (row.property != null) {
      Map<String, dynamic>? found;
      for (final key in row.keys) {
        if ((labelWidth +
                    (key['frame'] as num).toDouble() * pixelsPerFrame -
                    offset -
                    p.dx)
                .abs() <=
            7) {
          found = keyOf(row, key);
          break;
        }
      }
      final span = found == null ? spanAt(row, p.dx) : null;
      if (found != null || span != null) {
        final picked = span ?? [found!];
        var chosen = selectedKeys.toList();
        final already = picked.every((f) => chosen.any((k) => sameKey(k, f)));
        if (additive) {
          already
              ? chosen.removeWhere((k) => picked.any((f) => sameKey(k, f)))
              : chosen.addAll(
                  picked.where((f) => !chosen.any((k) => sameKey(k, f))),
                );
        } else if (!already)
          chosen = picked;
        widget.controller.command('select', {
          'ids': chosen.map((k) => k['layer']).toSet().toList(),
          'keys': chosen,
        });
        initialKeys = chosen;
        gesture = has('moveKeys') && span == null ? 'keys' : null;
        setState(() {});
        return;
      }
    } else {
      final left =
          labelWidth +
          (row.layer['start'] as num? ?? 0).toDouble() * pixelsPerFrame -
          offset;
      final right =
          left +
          (row.layer['duration'] as num? ?? 0).toDouble() * pixelsPerFrame;
      if (p.dx >= left - 4 && p.dx <= right + 4) {
        if (!initialIds.contains(row.id)) chooseLayer(row.id);
        timingRows = initialIds.contains(row.id)
            ? widget.controller.layers
                  .where(
                    (l) => initialIds.contains(l['id']) && l['locked'] != true,
                  )
                  .map((l) => _TrackRow(l))
                  .toList()
            : [row];
        if (row.layer['locked'] == true ||
            (!has('setTiming') && !has('setTimings')) ||
            (timingRows.length > 1 && !has('setTimings'))) {
          gesture = null;
          return;
        }
        gesture = HardwareKeyboard.instance.isAltPressed
            ? 'slip'
            : (p.dx - left).abs() < 6
            ? 'trimIn'
            : (p.dx - right).abs() < 6
            ? 'trimOut'
            : 'move';
        setState(() {});
        return;
      }
    }
    gesture = 'marquee';
    setState(() {});
  }

  void move(PointerMoveEvent event) {
    if (gesture == null || start == null) return;
    if (gesture == 'rowPending' || gesture == 'rowDrag') {
      if ((event.localPosition - start!).distance < 4 &&
          gesture == 'rowPending')
        return;
      setState(() {
        gesture = 'rowDrag';
        current = event.localPosition;
        aimRowDrop(current!, rowDragIds);
      });
      return;
    }
    setState(() {
      current = event.localPosition;
      deltaFrames = ((current!.dx - start!.dx) / pixelsPerFrame).round();
      if (timingRows.isNotEmpty &&
          ['move', 'trimIn', 'trimOut', 'slip'].contains(gesture)) {
        final starts = timingRows.map(
          (r) => (r.layer['start'] as num? ?? 0).toInt(),
        );
        if (gesture == 'move')
          deltaFrames = math.max(deltaFrames, -starts.reduce(math.min));
      }
    });
    if (timingRows.isNotEmpty &&
        has('previewTimings') &&
        ['move', 'trimIn', 'trimOut', 'slip'].contains(gesture)) {
      queuedTimings = timingRows.map(timing).toList();
      previewUsed = true;
      previewFlight ??= pumpPreview();
    }
  }

  /// 行の上の点から落とし先を決める。行の drag も Media からの drag もこれを読む。
  void aimRowDrop(Offset at, List<int> moving) {
    rowDrop = null;
    rowDropGuide = null;
    final index = layout.rowAt(at.dy);
    if (index < 0) {
      if (at.dy >= layout.height) {
        rowDrop = {'layers': moving, 'target': null, 'placement': 'rootEnd'};
        rowDropInside = false;
        rowDropGuide = Rect.fromLTWH(0, layout.height, labelWidth, 0);
      }
      return;
    }
    final target = tracks[index];
    if (moving.contains(target.id) || target.ancestors.any(moving.contains))
      return;
    final inside =
        target.isGroup &&
        target.property == null &&
        (at.dy - target.bounds.top) > 5 &&
        (target.bounds.bottom - at.dy) > 5;
    final placement = inside
        ? 'inside'
        : at.dy < target.bounds.center.dy
        ? 'before'
        : 'after';
    rowDrop = {'layers': moving, 'target': target.id, 'placement': placement};
    rowDropInside = inside;
    final container = layout.container(target.id)!;
    rowDropGuide = inside
        ? target.bounds
        : Rect.fromLTWH(
            container.bounds.left,
            placement == 'before'
                ? container.bounds.top
                : container.bounds.bottom,
            labelWidth - container.bounds.left,
            0,
          );
  }

  /// Media から落ちてきた素材。落とした行の前後/中に置き、x が時間軸なら開始コマも取る。
  void acceptAsset(DragTargetDetails<Map<String, dynamic>> details) {
    final drop = rowDrop;
    final at = assetDrop;
    setState(() {
      assetDrop = null;
      rowDrop = null;
      rowDropGuide = null;
    });
    if (drop == null) return;
    widget.controller.command('placeAsset', {
      'id': details.data['asset'],
      'target': drop['target'],
      'placement': drop['placement'],
      if (at != null && at.dx >= labelWidth) 'start': frameAt(at.dx),
    });
  }

  void aimAsset(DragTargetDetails<Map<String, dynamic>> details) {
    final box = rowsKey.currentContext?.findRenderObject() as RenderBox?;
    if (box == null) return;
    setState(() {
      assetDrop = box.globalToLocal(details.offset);
      aimRowDrop(assetDrop!, const []);
    });
  }

  Map<String, dynamic> timing(_TrackRow row) {
    final s = (row.layer['start'] as num? ?? 0).toInt(),
        d = (row.layer['duration'] as num? ?? 1).toInt(),
        i = (row.layer['sourceIn'] as num? ?? 0).toInt();
    switch (gesture) {
      case 'trimIn':
        final delta = deltaFrames.clamp(-math.min(s, i), d - 1);
        return {
          'layer': row.id,
          'start': s + delta,
          'duration': d - delta,
          'sourceIn': i + delta,
        };
      case 'trimOut':
        return {
          'layer': row.id,
          'start': s,
          'duration': math.max(1, d + deltaFrames),
          'sourceIn': i,
        };
      case 'slip':
        return {
          'layer': row.id,
          'start': s,
          'duration': d,
          'sourceIn': math.max(0, i - deltaFrames),
        };
      default:
        return {
          'layer': row.id,
          'start': math.max(0, s + deltaFrames),
          'duration': d,
          'sourceIn': i,
        };
    }
  }

  void end(PointerUpEvent event) {
    if (gesture == null) return;
    if (gesture == 'rowPending' || gesture == 'rowDrag') {
      if (gesture == 'rowDrag' && rowDrop != null) {
        if (has('moveLayers'))
          widget.controller.command('moveLayers', Map.of(rowDrop!));
        else
          widget.controller.error.value =
              'Layer move is waiting for the native update';
      } else if (gesture == 'rowPending' && dragRow != null)
        chooseLayer(dragRow!.id);
      resetGesture();
      return;
    }
    if (gesture == 'keys' && deltaFrames != 0)
      widget.controller.command('moveKeys', {'deltaFrames': deltaFrames});
    else if (['move', 'trimIn', 'trimOut', 'slip'].contains(gesture) &&
        deltaFrames != 0 &&
        dragRow != null)
      finishTiming(timingRows.map(timing).toList());
    else if (gesture == 'marquee' && start != null && current != null) {
      final rect = Rect.fromPoints(start!, current!);
      final keys = <Map<String, dynamic>>[if (additive) ...initialKeys];
      final ids = <int>{if (additive) ...initialIds};
      if (rect.width > 3 || rect.height > 3) {
        for (var n = 0; n < tracks.length; n++) {
          final row = tracks[n], cy = tracks[n].bounds.center.dy;
          if (row.property != null) {
            for (final key in row.keys) {
              if (rect.contains(
                Offset(
                  labelWidth +
                      (key['frame'] as num).toDouble() * pixelsPerFrame -
                      offset,
                  cy,
                ),
              )) {
                final candidate = keyOf(row, key);
                if (!keys.any((k) => sameKey(k, candidate)))
                  keys.add(candidate);
                ids.add(row.id);
              }
            }
          } else {
            if (!row.lanesOpen)
              for (final k in row.allKeys) {
                final x =
                    labelWidth +
                    (k['frame'] as num).toDouble() * pixelsPerFrame -
                    offset;
                if (rect.contains(Offset(x, cy)) &&
                    !keys.any((s) => sameKey(s, k))) {
                  keys.add(k);
                  ids.add(row.id);
                }
              }
            final left =
                labelWidth +
                (row.layer['start'] as num? ?? 0).toDouble() * pixelsPerFrame -
                offset;
            final width =
                (row.layer['duration'] as num? ?? 0).toDouble() *
                pixelsPerFrame;
            if (rect.overlaps(
              Rect.fromLTWH(
                left,
                tracks[n].bounds.top + 2,
                width,
                rowHeight - 4,
              ),
            ))
              ids.add(row.id);
          }
        }
      }
      widget.controller.command('select', {'ids': ids.toList(), 'keys': keys});
    }
    if (previewUsed) {
      queuedTimings = null;
      previewUsed = false;
      widget.controller.cancelPreview();
    }
    resetGesture();
  }

  void cancel() {
    queuedTimings = null;
    if (previewUsed) {
      previewUsed = false;
      widget.controller.cancelPreview();
    }
    resetGesture();
  }

  void resetGesture() {
    setState(() {
      gesture = null;
      rowDrop = null;
      rowDropGuide = null;
      rowDragIds = [];
      start = null;
      current = null;
      dragRow = null;
      timingRows = [];
      deltaFrames = 0;
    });
  }

  KeyEventResult key(FocusNode node, KeyEvent event) {
    if (event is! KeyDownEvent) return KeyEventResult.ignored;
    final k = event.logicalKey;
    if (k == LogicalKeyboardKey.escape) {
      cancel();
      widget.controller.cancelPreview();
      return KeyEventResult.handled;
    }
    if (k == LogicalKeyboardKey.arrowLeft ||
        k == LogicalKeyboardKey.arrowRight) {
      final delta =
          (k == LogicalKeyboardKey.arrowLeft ? -1 : 1) *
          (HardwareKeyboard.instance.isShiftPressed ? 10 : 1);
      if (selectedKeys.isNotEmpty && has('moveKeys'))
        widget.controller.command('moveKeys', {'deltaFrames': delta});
      else
        requestSeek(math.max(0, widget.controller.frame.value + delta));
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  void navigateView(double scale, double x, double y) {
    final nextScale = scale.clamp(.1, 40.0);
    final visible = math.max(1.0, viewportWidth - labelWidth);
    final targetX = x.clamp(
      0.0,
      math.max(0.0, overviewExtent * nextScale - visible),
    );
    final revision = ++navigationRevision;
    if (nextScale != pixelsPerFrame) setState(() => pixelsPerFrame = nextScale);
    void apply() {
      if (!mounted || revision != navigationRevision) return;
      if (horizontal.hasClients) {
        final position = horizontal.positions.last;
        if (position.hasContentDimensions)
          position.jumpTo(
            targetX
                .clamp(position.minScrollExtent, position.maxScrollExtent)
                .toDouble(),
          );
      }
      if (vertical.hasClients) {
        final position = vertical.positions.last;
        if (position.hasContentDimensions)
          position.jumpTo(
            y.clamp(position.minScrollExtent, position.maxScrollExtent),
          );
      }
    }

    apply();
    WidgetsBinding.instance.addPostFrameCallback((_) => apply());
  }

  double get verticalOffset =>
      vertical.hasClients ? vertical.positions.last.pixels : 0;
  void zoomAt(double factor, double x) {
    final anchorX = (x - labelWidth).clamp(
      0.0,
      math.max(1.0, viewportWidth - labelWidth),
    );
    final at = (offset + anchorX) / pixelsPerFrame;
    final scale = (pixelsPerFrame * factor).clamp(.1, 40.0);
    navigateView(scale, at * scale - anchorX, verticalOffset);
  }

  void zoom(double factor) => zoomAt(
    factor,
    labelWidth + math.max(1.0, viewportWidth - labelWidth) / 2,
  );
  void stopMomentum() {
    _motion?.stop();
    navigationRevision++;
    for (final controller in [horizontal, vertical]) {
      if (controller.hasClients) {
        final position = controller.positions.last;
        if (position is ScrollPositionWithSingleContext) position.goIdle();
      }
    }
  }

  Widget navigation(Widget child) => GestureDetector(
    supportedDevices: const {PointerDeviceKind.trackpad},
    onScaleStart: (e) {
      stopMomentum();
      navigating = true;
      navigationOrigin = e.localFocalPoint;
      final anchor = math.max(0.0, e.localFocalPoint.dx - labelWidth);
      motion.begin(
        mode: overTimeRuler(e.localFocalPoint)
            ? ViewportGestureMode.scrubZoom
            : ViewportGestureMode.pan,
        scale: pixelsPerFrame,
        frame: (offset + anchor) / pixelsPerFrame,
        anchor: anchor,
        y: verticalOffset,
      );
    },
    onScaleUpdate: (e) => motion.update(
      e.localFocalPoint - navigationOrigin,
      e.scale,
      e.sourceTimeStamp,
    ),
    onScaleEnd: (_) {
      navigating = false;
      navigationRevision++;
      motion.end();
    },
    child: Listener(
      onPointerDown: (_) => stopMomentum(),
      onPointerSignal: (event) {
        if (event is! PointerScrollEvent || navigating) return;
        GestureBinding.instance.pointerSignalResolver.register(event, (_) {
          stopMomentum();
          if (primary || overTimeRuler(event.localPosition))
            zoomAt(
              math.exp(-event.scrollDelta.dy * .002),
              event.localPosition.dx,
            );
          else if (HardwareKeyboard.instance.isShiftPressed)
            navigateView(
              pixelsPerFrame,
              offset + event.scrollDelta.dy + event.scrollDelta.dx,
              verticalOffset,
            );
          else
            navigateView(
              pixelsPerFrame,
              offset + event.scrollDelta.dx,
              verticalOffset + event.scrollDelta.dy,
            );
        });
      },
      child: child,
    ),
  );

  Future<void> menu(TapDownDetails details) async {
    final row = layout.rowAt(details.localPosition.dy);
    final target = row >= 0 && row < tracks.length ? tracks[row].id : null;
    if (row >= 0 &&
        row < tracks.length &&
        !widget.controller.selectedIds.contains(tracks[row].id)) {
      await widget.controller.command('select', {
        'ids': [tracks[row].id],
        'keys': [],
      });
    }
    if (!mounted) return;
    final actions = <String, String>{
      'copy': 'Copy',
      'cut': 'Cut',
      'paste': 'Paste',
      'duplicate': 'Duplicate',
      'delete': 'Delete',
      'group': 'Group',
      'ungroup': 'Ungroup',
      'split': 'Split',
    };
    final chosen = await showEditorMenu<String>(
      context,
      details.globalPosition,
      [
        if (target != null) ...[
          const EditorMenuItem<String>(
            value: 'lanes:keyed',
            child: Text('Show animated properties'),
          ),
          const EditorMenuItem<String>(
            value: 'lanes:all',
            child: Text('Show all properties'),
          ),
          const EditorMenuItem<String>(
            value: 'lanes:hide',
            child: Text('Hide properties'),
          ),
          const EditorMenuDivider(),
        ],
        for (final entry in actions.entries)
          EditorMenuItem<String>(
            value: entry.key,
            enabled: has(entry.key),
            child: Row(
              children: [
                Expanded(child: Text(entry.value)),
                const SizedBox(width: EditorMetrics.s16),
                Text(
                  const {
                        'copy': '⌘C',
                        'cut': '⌘X',
                        'paste': '⌘V',
                        'duplicate': '⌘D',
                        'delete': '⌫',
                        'group': '⌘G',
                        'ungroup': '⇧⌘G',
                        'split': '⌘K',
                      }[entry.key] ??
                      '',
                ),
              ],
            ),
          ),
      ],
    );
    if (chosen != null && chosen.startsWith('lanes:') && target != null) {
      setState(() {
        allProperties.remove(target);
        if (chosen == 'lanes:hide')
          expanded.remove(target);
        else {
          expanded.add(target);
          if (chosen == 'lanes:all') allProperties.add(target);
        }
        _relane();
      });
    } else if (chosen != null)
      widget.controller.command(chosen);
  }

  DocumentSlice get _timeline => widget.controller.slice('timeline', const [
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
  void _relane() {
    final liveIds = widget.controller.layers
        .map((layer) => (layer['id'] as num).toInt())
        .toSet();
    expanded.retainAll(liveIds);
    allProperties.retainAll(liveIds);
    collapsedGroups.retainAll(liveIds);
    layout = _LaneLayout(
      widget.controller.layers,
      expanded,
      allProperties,
      collapsedGroups,
      baseNameWidth: baseNameWidth,
    );
    tracks = layout.rows;
    if (activeLane != null && !tracks.any((row) => row.laneId == activeLane))
      activeLane = null;
    _reportVisible();
  }

  /// How many frames the bar shows, for the session's zoom-to-fit.
  void _reportVisible() {
    final visible = ((viewportWidth - labelWidth) / pixelsPerFrame).round();
    if (widget.controller.visibleFrames.value != visible)
      SchedulerBinding.instance.addPostFrameCallback((_) {
        if (mounted) widget.controller.visibleFrames.value = visible;
      });
  }

  @override
  Widget build(BuildContext context) => Focus(
    key: const ValueKey('timeline-bounded-layout'),
    focusNode: focus,
    onKeyEvent: key,
    child: ColoredBox(
      color: EditorTheme.panel,
      child: LayoutBuilder(
        builder: (context, bounds) {
          viewportWidth = bounds.maxWidth;
          _reportVisible();
          return navigation(
            ListenableBuilder(
              listenable: _timeline,
              child: Column(
                children: [
                  _bar(bounds),
                  ListenableBuilder(
                    listenable: Listenable.merge([
                      widget.controller.frame,
                      _timeline,
                      scrolled,
                    ]),
                    builder: (context, _) => GestureDetector(
                      supportedDevices: const {
                        PointerDeviceKind.mouse,
                        PointerDeviceKind.touch,
                        PointerDeviceKind.stylus,
                      },
                      onTapDown: (e) {
                        if (e.localPosition.dx >= labelWidth)
                          requestSeek(frameAt(e.localPosition.dx));
                      },
                      onHorizontalDragUpdate: (e) {
                        if (e.localPosition.dx >= labelWidth)
                          requestSeek(frameAt(e.localPosition.dx));
                      },
                      child: SizedBox(
                        height: rulerHeight,
                        width: double.infinity,
                        child: CustomPaint(
                          painter: _TimelinePainter(
                            labelWidth: labelWidth,
                            rows: const [],
                            selected: const [],
                            keys: const [],
                            frame: scrubFrame ?? widget.controller.frame.value,
                            scale: pixelsPerFrame,
                            offset: offset,
                            duration: duration,
                            fps: (widget.controller.state['fps'] as num? ?? 30)
                                .toDouble(),
                            markers: EditorSession.maps(
                              widget.controller.state['markers'],
                            ),
                            ruler: true,
                          ),
                        ),
                      ),
                    ),
                  ),
                  Expanded(
                    child: Scrollbar(
                      controller: vertical,
                      child: SingleChildScrollView(
                        controller: vertical,
                        physics: const ClampingScrollPhysics(
                          parent: NeverScrollableScrollPhysics(),
                        ),
                        child: DragTarget<Map<String, dynamic>>(
                          onWillAcceptWithDetails: (d) =>
                              d.data['asset'] != null && has('placeAsset'),
                          onMove: aimAsset,
                          onLeave: (_) => setState(() {
                            assetDrop = null;
                            rowDrop = null;
                            rowDropGuide = null;
                          }),
                          onAcceptWithDetails: acceptAsset,
                          builder: (_, __, ___) => GestureDetector(
                            onSecondaryTapDown: menu,
                            child: Listener(
                              onPointerDown: begin,
                              onPointerMove: move,
                              onPointerUp: end,
                              onPointerCancel: (_) => cancel(),
                              child: ListenableBuilder(
                                listenable: Listenable.merge([
                                  widget.controller.frame,
                                  _timeline,
                                  scrolled,
                                ]),
                                builder: (context, _) => SizedBox(
                                  key: rowsKey,
                                  width: bounds.maxWidth,
                                  height: math.max(
                                    layout.height,
                                    bounds.maxHeight - 66,
                                  ),
                                  child: CustomPaint(
                                    key: const ValueKey('timeline-lanes'),
                                    painter: _TimelinePainter(
                                      labelWidth: labelWidth,
                                      rows: tracks,
                                      rowDropGuide: rowDropGuide,
                                      rowDropInside: rowDropInside,
                                      containers: layout.roots,
                                      activeLane: activeLane,
                                      selected: widget.controller.selectedIds,
                                      keys: selectedKeys,
                                      frame:
                                          scrubFrame ??
                                          widget.controller.frame.value,
                                      scale: pixelsPerFrame,
                                      offset: offset,
                                      duration: duration,
                                      fps:
                                          (widget.controller.state['fps']
                                                      as num? ??
                                                  30)
                                              .toDouble(),
                                      markers: EditorSession.maps(
                                        widget.controller.state['markers'],
                                      ),
                                      marquee:
                                          gesture == 'marquee' &&
                                              start != null &&
                                              current != null
                                          ? Rect.fromPoints(start!, current!)
                                          : null,
                                      dragKeys: gesture == 'keys'
                                          ? initialKeys
                                          : const [],
                                      delta: deltaFrames,
                                      dragLayer:
                                          dragRow != null &&
                                              [
                                                'move',
                                                'trimIn',
                                                'trimOut',
                                                'slip',
                                              ].contains(gesture)
                                          ? timing(dragRow!)
                                          : null,
                                    ),
                                  ),
                                ),
                              ),
                            ),
                          ),
                        ),
                      ),
                    ),
                  ),
                  ListenableBuilder(
                    listenable: _timeline,
                    builder: (context, _) => Padding(
                      padding: EdgeInsets.only(left: labelWidth),
                      child: SizedBox(
                        height: EditorMetrics.s12,
                        child: Scrollbar(
                          controller: horizontal,
                          thumbVisibility: true,
                          child: SingleChildScrollView(
                            controller: horizontal,
                            physics: const ClampingScrollPhysics(
                              parent: NeverScrollableScrollPhysics(),
                            ),
                            scrollDirection: Axis.horizontal,
                            child: SizedBox(
                              width: math.max(
                                bounds.maxWidth - labelWidth,
                                contentWidth,
                              ),
                              height: EditorMetrics.s12,
                            ),
                          ),
                        ),
                      ),
                    ),
                  ),
                ],
              ),
              builder: (context, column) => Stack(
                children: [
                  column!,
                  Positioned(
                    left: layout.nameWidth - EditorMetrics.s3,
                    top: EditorMetrics.s22,
                    bottom: EditorMetrics.s12,
                    width: EditorMetrics.s6,
                    child: MouseRegion(
                      cursor: SystemMouseCursors.resizeColumn,
                      child: GestureDetector(
                        supportedDevices: const {
                          PointerDeviceKind.mouse,
                          PointerDeviceKind.touch,
                          PointerDeviceKind.stylus,
                        },
                        behavior: HitTestBehavior.opaque,
                        onHorizontalDragStart: (e) {
                          resizeStart = layout.nameWidth - layout.indentation;
                          resizePointerStart = e.globalPosition.dx;
                        },
                        onHorizontalDragUpdate: (e) => setState(() {
                          baseNameWidth =
                              (resizeStart +
                                      e.globalPosition.dx -
                                      resizePointerStart)
                                  .clamp(114.0 - layout.indentation, 162.0);
                          _relane();
                        }),
                        onHorizontalDragCancel: () => setState(() {
                          baseNameWidth = resizeStart;
                          _relane();
                        }),
                        onDoubleTap: () => setState(() {
                          baseNameWidth = 138;
                          _relane();
                        }),
                        child: const SizedBox.expand(),
                      ),
                    ),
                  ),
                ],
              ),
            ),
          );
        },
      ),
    ),
  );

  /// 帯は寸法だけで建つ。書類を見る二つ — 全体図と Marker — は自分で
  /// 書類を購読するので、帯そのものは建て直さなくてよい。
  Widget _bar(BoxConstraints bounds) => SizedBox(
    height: EditorMetrics.s22,
    child: Row(
      children: [
        const SizedBox(width: EditorMetrics.s6),
        ValueListenableBuilder<bool>(
          valueListenable: widget.controller.playing,
          builder: (_, playing, __) => EditorButton(
            playing ? 'Ⅱ' : '▶',
            widget.controller.togglePlayback,
            selected: playing,
            tooltip: 'Play / Pause · Space',
          ),
        ),
        EditorButton(
          '−',
          () => zoom(
            ((pixelsPerFrame / 4 * 100).round() - 1).clamp(3, 1000) *
                .04 /
                pixelsPerFrame,
          ),
        ),
        EditorPercentField(
          value: pixelsPerFrame / 4 * 100,
          min: 3,
          max: 1000,
          label: 'Timeline zoom',
          onChanged: (v) => zoom(v * .04 / pixelsPerFrame),
        ),
        EditorButton(
          '+',
          () => zoom(
            ((pixelsPerFrame / 4 * 100).round() + 1) * .04 / pixelsPerFrame,
          ),
        ),
        EditorButton('Fit', () {
          setState(
            () => pixelsPerFrame = math.max(
              .1,
              (bounds.maxWidth - labelWidth) /
                  math.max(1, overviewExtentWithoutFrame),
            ),
          );
        }),
        Expanded(
          child: SizedBox(
            height: EditorMetrics.s18,
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: EditorMetrics.s8),
              child: LayoutBuilder(
                builder: (context, overview) {
                  void navigate(double x) {
                    if (!horizontal.hasClients) return;
                    final target =
                        x /
                            math.max(1, overview.maxWidth) *
                            overviewExtent *
                            pixelsPerFrame -
                        (bounds.maxWidth - labelWidth) / 2;
                    horizontal.jumpTo(
                      target.clamp(
                        0,
                        horizontal.positions.last.maxScrollExtent,
                      ),
                    );
                  }

                  return GestureDetector(
                    supportedDevices: const {
                      PointerDeviceKind.mouse,
                      PointerDeviceKind.touch,
                      PointerDeviceKind.stylus,
                    },
                    onTapDown: (e) => navigate(e.localPosition.dx),
                    onHorizontalDragUpdate: (e) => navigate(e.localPosition.dx),
                    onDoubleTap: () {
                      setState(
                        () => pixelsPerFrame = math.max(
                          .1,
                          (bounds.maxWidth - labelWidth) /
                              math.max(1, overviewExtentWithoutFrame),
                        ),
                      );
                    },
                    child: ListenableBuilder(
                      listenable: Listenable.merge([
                        widget.controller.frame,
                        _timeline,
                        scrolled,
                      ]),
                      builder: (context, _) => CustomPaint(
                        size: Size(overview.maxWidth, EditorMetrics.s18),
                        painter: _ArrangementOverview(
                          layers: widget.controller.layers,
                          duration: duration,
                          extent: overviewExtent,
                          frame: scrubFrame ?? widget.controller.frame.value,
                          offset: offset,
                          scale: pixelsPerFrame,
                          viewportWidth: bounds.maxWidth - labelWidth,
                        ),
                      ),
                    ),
                  );
                },
              ),
            ),
          ),
        ),
        ListenableBuilder(
          listenable: _timeline,
          builder: (context, _) => EditorButton(
            'Marker',
            has('addMarker')
                ? () => widget.controller.command('addMarker')
                : null,
          ),
        ),
      ],
    ),
  );
}

/// Laid-out labels kept across paints: a row's name is the same text at the
/// same width on every frame of playback.
final _labels = <(String, Color, double, double, FontWeight), TextPainter>{};

class _TimelinePainter extends CustomPainter {
  _TimelinePainter({
    required this.rows,
    required this.selected,
    required this.keys,
    required this.frame,
    required this.scale,
    required this.offset,
    required this.duration,
    required this.markers,
    this.rowDropGuide,
    this.rowDropInside = false,
    this.labelWidth = 220,
    this.containers = const [],
    this.activeLane,
    this.fps = 30,
    this.ruler = false,
    this.marquee,
    this.dragKeys = const [],
    this.delta = 0,
    this.dragLayer,
  });
  final Rect? rowDropGuide;
  final bool rowDropInside;
  final double labelWidth;
  final List<_LaneContainer> containers;
  final String? activeLane;
  final List<_TrackRow> rows;
  // Every selected layer keeps its mark while a lane is being worked on:
  // the active lane adds a highlight, it never hides the others.
  bool layerSelected(_TrackRow row) =>
      row.property == null && selected.contains(row.id);
  bool laneSelected(_TrackRow row) =>
      layerSelected(row) || (activeLane != null && row.laneId == activeLane);
  final List<int> selected;
  final List<Map<String, dynamic>> keys, markers, dragKeys;
  final int frame, duration, delta;
  final double scale, offset, fps;
  final bool ruler;
  final Rect? marquee;
  final Map<String, dynamic>? dragLayer;
  void text(
    Canvas canvas,
    String text,
    Offset at, {
    Color color = EditorTheme.ink,
    double width = 200,
    double size = 11,
    bool centered = false,
    FontWeight weight = FontWeight.w400,
  }) {
    if (_labels.length >= 512) _labels.clear();
    final p = _labels[(text, color, width, size, weight)] ??= TextPainter(
      text: TextSpan(
        text: text,
        style: TextStyle(
          color: color,
          fontSize: size,
          fontFamily: 'Arial',
          fontWeight: weight,
        ),
      ),
      textDirection: TextDirection.ltr,
      maxLines: 1,
      ellipsis: '…',
    )..layout(maxWidth: math.max(0, width));
    final aligned = ruler
        ? at
        : Offset(
            at.dx,
            (at.dy / _TimelinePanelState.rowHeight).floor() *
                    _TimelinePanelState.rowHeight +
                (_TimelinePanelState.rowHeight - p.height) / 2,
          );
    p.paint(
      canvas,
      centered ? Offset(at.dx + (width - p.width) / 2, aligned.dy) : aligned,
    );
  }

  bool selectedKey(
    _TrackRow row,
    Map<String, dynamic> key,
    List<Map<String, dynamic>> selection,
  ) => selection.any(
    (s) =>
        s['layer'] == row.id &&
        s['property'] == row.property?['id'] &&
        s['frame'] == key['frame'],
  );
  @override
  void paint(Canvas canvas, Size size) {
    final label = labelWidth;
    const h = _TimelinePanelState.rowHeight;
    canvas.drawRect(
      Offset.zero & size,
      Paint()..color = const Color(0xff3c3c3c),
    );
    final unit = 85 / scale >= fps ? fps : 1.0;
    final desired = 85 / scale / unit;
    final magnitude = math
        .pow(10, (math.log(math.max(1, desired)) / math.ln10).floor())
        .toDouble();
    final gridStep =
        ([1, 2, 5, 10].firstWhere(
                  (n) => n * magnitude >= desired,
                  orElse: () => 10,
                ) *
                magnitude *
                unit)
            .round()
            .clamp(1, 100000000);
    canvas.save();
    canvas.clipRect(
      Rect.fromLTWH(label, 0, math.max(0, size.width - label), size.height),
    );
    if (!ruler) {
      for (var i = 0; i < rows.length; i++) {
        canvas.drawRect(
          Rect.fromLTWH(label, rows[i].bounds.top, size.width - label, h),
          Paint()
            ..color = laneSelected(rows[i])
                ? EditorTheme.hover
                : rows[i].property != null
                ? const Color(0xff383838)
                : (i.isEven
                      ? const Color(0xff3d3d3d)
                      : const Color(0xff383838)),
        );
      }
    }
    for (
      var t = (offset / scale / gridStep).floor() * gridStep;
      label + t * scale - offset < size.width;
      t += gridStep
    ) {
      final x = label + t * scale - offset;
      if ((t ~/ gridStep).isEven)
        canvas.drawRect(
          Rect.fromLTWH(x, 0, gridStep * scale, size.height),
          Paint()..color = Colors.white.withValues(alpha: .025),
        );
      canvas.drawLine(
        Offset(x, 0),
        Offset(x, size.height),
        Paint()
          ..color = const Color(0xff262626)
          ..strokeWidth = 1.5,
      );
      for (var sub = 1; sub < 4; sub++) {
        if (gridStep * scale / 4 < 12) break;
        final sx = x + gridStep * scale * sub / 4;
        canvas.drawLine(
          Offset(sx, ruler ? size.height - 12 : 0),
          Offset(sx, size.height),
          Paint()
            ..color = const Color(0xff303030)
            ..strokeWidth = 1,
        );
      }
      if (ruler) {
        canvas.drawLine(
          Offset(x, 9),
          Offset(x, size.height),
          Paint()..color = const Color(0xff9b9b9b),
        );
        for (var sub = 1; sub < 4; sub++) {
          final sx = x + gridStep * scale * sub / 4;
          canvas.drawLine(
            Offset(sx, size.height - 12),
            Offset(sx, size.height - 10),
            Paint()..color = const Color(0xff929292),
          );
        }
        final seconds = t / fps;
        text(
          canvas,
          '${seconds.toStringAsFixed(seconds == seconds.roundToDouble() ? 0 : 2)}s',
          Offset(x + 4, 2),
          width: EditorMetrics.s78,
          size: EditorMetrics.dense,
        );
        text(
          canvas,
          '${t}f',
          Offset(x + 4, 17),
          width: EditorMetrics.s78,
          size: EditorMetrics.micro,
          color: EditorTheme.muted,
        );
      }
    }
    if (!ruler)
      for (var i = 0; i < rows.length; i++) {
        final row = rows[i], y = rows[i].bounds.top;
        if (row.property == null && row.isGroup) {
          final children = row.descendants
              .where((l) => l['kind'] != 'Group')
              .toList();
          final stripe = math.min(4.0, (h - 4) / math.max(1, children.length));
          for (var n = 0; n < children.length; n++) {
            final child = children[n];
            final x = label + (child['start'] as num? ?? 0) * scale - offset;
            final width = (child['duration'] as num? ?? 1) * scale;
            canvas.drawRect(
              Rect.fromLTWH(
                x,
                y + 2 + n * stripe,
                math.max(1, width),
                math.max(1, stripe - 1),
              ),
              Paint()
                ..color = row.groupOpen
                    ? EditorTheme.border
                    : EditorTheme.layerColor(child['id']),
            );
          }
        } else if (row.property == null) {
          final timing = dragLayer?['layer'] == row.id ? dragLayer! : row.layer;
          final x =
              label +
              (timing['start'] as num? ?? 0).toDouble() * scale -
              offset;
          final width = (timing['duration'] as num? ?? 1).toDouble() * scale;
          final rect = Rect.fromLTWH(x, y + 2, math.max(1, width), h - 4);
          final own = EditorTheme.layerColor(row.id);
          // ゴースト: 同じ行に、遅れの分だけずれた帯が 1 つ。掴めない。
          // 実物と重なる区間は実物の帯がそのまま語る(両方が鳴る)。描くのは
          // はみ出し = 実物の時間の外でゴーストだけが評価される区間で、それをグレーにする。
          // 前へずれて 0 より手前に出た分は描かず、frame 0 に切り欠きだけ置く。
          if (row.layer['ghost'] case final num d) {
            final ghost = rect.shift(Offset(d.toDouble() * scale, 0));
            final zero = label - offset;
            final only = d > 0
                ? Rect.fromLTRB(
                    rect.right,
                    ghost.top,
                    ghost.right,
                    ghost.bottom,
                  )
                : Rect.fromLTRB(
                    math.max(zero, ghost.left),
                    ghost.top,
                    rect.left,
                    ghost.bottom,
                  );
            if (only.width > 0) {
              canvas.drawRect(
                only,
                Paint()..color = EditorTheme.muted.withValues(alpha: .28),
              );
              canvas.drawRect(
                only.deflate(.5),
                Paint()
                  ..color = own.withValues(alpha: .55)
                  ..strokeWidth = 1
                  ..style = PaintingStyle.stroke,
              );
            }
            if (ghost.left < zero) {
              final notch = Path()
                ..moveTo(zero, ghost.top)
                ..lineTo(zero + 5, ghost.center.dy)
                ..lineTo(zero, ghost.bottom)
                ..close();
              canvas.drawPath(
                notch,
                Paint()..color = own.withValues(alpha: .9),
              );
            }
          }
          final chosen = layerSelected(row);
          canvas.drawRect(
            rect,
            Paint()
              ..color = row.layer['hidden'] == true
                  ? EditorTheme.raised
                  : chosen
                  ? Color.lerp(own, Colors.white, EditorTheme.lift)!
                  : own,
          );
          if (!row.lanesOpen)
            for (final f in row.summaryFrames) {
              bool at(List<Map<String, dynamic>> sel) =>
                  sel.any((s) => s['layer'] == row.id && s['frame'] == f);
              final shift = at(dragKeys) ? delta : 0;
              final kx = label + (f + shift) * scale - offset;
              final cy = y + h / 2;
              final diamond = Path()
                ..moveTo(kx, cy - 3.5)
                ..lineTo(kx + 3.5, cy)
                ..lineTo(kx, cy + 3.5)
                ..lineTo(kx - 3.5, cy)
                ..close();
              canvas.drawPath(
                diamond,
                Paint()
                  ..color = at(keys) ? EditorTheme.keyAccent : EditorTheme.ink,
              );
              canvas.drawPath(
                diamond,
                Paint()
                  ..color = Colors.black
                  ..strokeWidth = 1
                  ..style = PaintingStyle.stroke,
              );
            }
        } else {
          double keyX(Map<String, dynamic> key) {
            final shift = selectedKey(row, key, dragKeys) ? delta : 0;
            return label +
                ((key['frame'] as num).toDouble() + shift) * scale -
                offset;
          }

          // The span between two keys is where an ease lives: solid once it
          // has a shape, dashed while still linear; lit when both ends are
          // chosen. Pressing it chooses both ends.
          for (var i = 0; i + 1 < row.keys.length; i++) {
            final a = row.keys[i], b = row.keys[i + 1];
            final x1 = keyX(a) + 5, x2 = keyX(b) - 5;
            if (x2 <= x1) continue;
            final cy = y + h / 2;
            final chosen =
                selectedKey(row, a, keys) && selectedKey(row, b, keys);
            final paint = Paint()
              ..color = chosen
                  ? EditorTheme.keyAccent
                  : EditorTheme.muted.withValues(alpha: .6)
              ..strokeWidth = chosen ? 2 : 1;
            if (a['interp']?['kind'] == 'Linear') {
              for (var x = x1; x < x2; x += 6) {
                canvas.drawLine(
                  Offset(x, cy),
                  Offset(math.min(x + 3, x2), cy),
                  paint,
                );
              }
            } else {
              canvas.drawLine(Offset(x1, cy), Offset(x2, cy), paint);
            }
          }
          for (final key in row.keys) {
            final x = keyX(key);
            final path = Path()
              ..moveTo(x, y + h / 2 - 5)
              ..lineTo(x + 5, y + h / 2)
              ..lineTo(x, y + h / 2 + 5)
              ..lineTo(x - 5, y + h / 2)
              ..close();
            canvas.drawPath(
              path,
              Paint()
                ..color = selectedKey(row, key, keys)
                    ? EditorTheme.keyAccent
                    : EditorTheme.ink,
            );
          }
        }
      }
    if (!ruler) {
      for (double y = h; y <= rows.length * h && y <= size.height; y += h) {
        canvas.drawLine(
          Offset(label, y),
          Offset(size.width, y),
          Paint()
            ..color = const Color(0xff242424)
            ..strokeWidth = 2,
        );
      }
    }
    canvas.drawLine(
      Offset(label, 0),
      Offset(size.width, 0),
      Paint()
        ..color = const Color(0xff242424)
        ..strokeWidth = 2,
    );
    if (ruler)
      canvas.drawLine(
        Offset(label, size.height - 1),
        Offset(size.width, size.height - 1),
        Paint()
          ..color = const Color(0xff242424)
          ..strokeWidth = 2,
      );
    for (final marker in markers) {
      final x =
          label + (marker['frame'] as num? ?? 0).toDouble() * scale - offset;
      canvas.drawLine(
        Offset(x, 0),
        Offset(x, size.height),
        Paint()..color = EditorTheme.muted.withValues(alpha: .4),
      );
      if (ruler)
        canvas.drawCircle(Offset(x, 3), 3, Paint()..color = EditorTheme.accent);
    }
    final playX = label + frame * scale - offset;
    if (ruler)
      canvas.drawPath(
        Path()
          ..moveTo(playX - 5, 0)
          ..lineTo(playX + 5, 0)
          ..lineTo(playX, 6)
          ..close(),
        Paint()..color = EditorTheme.keyAccent,
      );
    canvas.drawLine(
      Offset(playX, 0),
      Offset(playX, size.height),
      Paint()
        ..color = EditorTheme.keyAccent
        ..strokeWidth = 1.5,
    );
    if (marquee != null) {
      canvas.drawRect(
        marquee!,
        Paint()..color = EditorTheme.accent.withValues(alpha: .15),
      );
      canvas.drawRect(
        marquee!,
        Paint()
          ..color = EditorTheme.accent
          ..style = PaintingStyle.stroke,
      );
    }
    canvas.restore();
    canvas.drawRect(
      Rect.fromLTWH(0, 0, math.min(label, size.width), size.height),
      Paint()..color = EditorTheme.panel,
    );
    if (!ruler) {
      canvas.save();
      canvas.clipRect(Rect.fromLTWH(0, 0, label - 82, size.height));
      void surface(_LaneContainer node) {
        final row = node.row;
        final background = row.property == null
            ? EditorTheme.layerColor(row.id)
            : (laneSelected(row) ? EditorTheme.raised : EditorTheme.panel);
        canvas.drawRect(node.bounds, Paint()..color = background);
        for (final child in node.children) surface(child);
        canvas.drawRect(
          node.bounds.deflate(.5),
          Paint()
            ..color = EditorTheme.line
            ..style = PaintingStyle.stroke
            ..strokeWidth = 1,
        );
      }

      for (final root in containers) surface(root);
      canvas.restore();
    }
    if (ruler) {
      text(
        canvas,
        'Layers',
        const Offset(6, 16),
        color: EditorTheme.muted,
        size: EditorMetrics.dense,
        weight: FontWeight.w600,
      );
      // 今のコマ / 尺。定規の左の欄に固定で描くので、桁が変わっても何も押し退けない。
      text(
        canvas,
        '$frame / $duration',
        Offset(label - 82, 16),
        color: EditorTheme.muted,
        size: EditorMetrics.dense,
      );
    } else
      for (var i = 0; i < rows.length; i++) {
        final row = rows[i], y = rows[i].bounds.top;
        if (laneSelected(row))
          canvas.drawRect(
            Rect.fromLTWH(
              row.property == null ? label - 82 : row.bounds.left,
              y,
              label - (row.property == null ? label - 82 : row.bounds.left),
              h,
            ),
            Paint()..color = EditorTheme.raised,
          );
        // Chosen rows read as one lit surface from the name cell to the
        // time field; nothing is outlined, the selection is a region.
        if (layerSelected(row))
          canvas.drawRect(
            Rect.fromLTWH(row.bounds.left, y, label - row.bounds.left, h),
            Paint()..color = Colors.white.withValues(alpha: EditorTheme.lift),
          );
        if (row.property == null) {
          text(
            canvas,
            row.isGroup ? (row.groupOpen ? '⊟' : '⊞') : '',
            Offset(row.bounds.left + 5, y + 3),
            width: EditorMetrics.s12,
            centered: true,
            color: const Color(0xff202020),
          );
          text(
            canvas,
            '${row.layer['name']}',
            color: const Color(0xff202020),
            Offset(row.bounds.left + 23, y + 3),
            width: math.max(0.0, row.bounds.width - 29),
            weight: FontWeight.w500,
          );
          final keysOpen = row.lanesOpen;
          canvas.drawRect(
            Rect.fromLTWH(label - 81, y + 2, 14, h - 4),
            Paint()
              ..color = keysOpen ? EditorTheme.accent : const Color(0xff242424),
          );
          text(
            canvas,
            keysOpen ? '◆' : '◇',
            Offset(label - 81, y + 3),
            width: EditorMetrics.s14,
            size: EditorMetrics.dense,
            centered: true,
            color: keysOpen ? const Color(0xff202020) : EditorTheme.ink,
          );
          final states = [
            row.layer['hidden'] == true,
            row.layer['solo'] == true,
            row.layer['locked'] == true,
            row.layer['clipToBelow'] == true,
          ];
          const symbols = ['M', 'S', 'L', '↳'];
          for (var column = 0; column < 4; column++) {
            final x = label - 65 + 16 * column;
            final on = states[column];
            canvas.drawRect(
              Rect.fromLTWH(x, y + 2, 14, h - 4),
              Paint()
                ..color = on ? EditorTheme.accent : const Color(0xff242424),
            );
            text(
              canvas,
              symbols[column],
              Offset(x, y + 3),
              width: EditorMetrics.s14,
              size: EditorMetrics.micro,
              centered: true,
              weight: FontWeight.w600,
              color: on ? const Color(0xff202020) : EditorTheme.muted,
            );
          }
        } else {
          text(
            canvas,
            '${row.property!['label'] ?? row.property!['id'] ?? 'Content'}',
            Offset(row.bounds.left + 6, y + 3),
            width: EditorMetrics.s160,
            color: EditorTheme.muted,
          );
          text(
            canvas,
            row.property!['keyedNow'] == true ? '◆' : '◇',
            Offset(label - 21, y + 3),
            width: EditorMetrics.s17,
            color: EditorTheme.keyAccent,
          );
        }
        canvas.drawLine(
          Offset(row.bounds.left, y + h),
          Offset(size.width, y + h),
          Paint()
            ..color = const Color(0xff242424)
            ..strokeWidth = 2,
        );
      }
    if (rowDropGuide case final Rect guide) {
      final paint = Paint()
        ..color = const Color(0xffaedce8)
        ..strokeWidth = 2
        ..style = PaintingStyle.stroke;
      if (rowDropInside)
        canvas.drawRect(
          Rect.fromLTRB(
            guide.left,
            guide.top,
            size.width,
            guide.bottom,
          ).deflate(1),
          paint,
        );
      else {
        canvas.drawLine(
          Offset(guide.left, guide.top),
          Offset(size.width, guide.top),
          paint,
        );
        canvas.drawCircle(
          Offset(guide.left + 3, guide.top),
          3,
          Paint()..color = const Color(0xffaedce8),
        );
      }
    }
    if (ruler)
      canvas.drawLine(
        Offset(0, size.height - 1),
        Offset(size.width, size.height - 1),
        Paint()
          ..color = EditorTheme.line
          ..strokeWidth = 2,
      );
    canvas.drawLine(
      Offset(label, 0),
      Offset(label, size.height),
      Paint()..color = EditorTheme.line,
    );
  }

  @override
  bool shouldRepaint(covariant _TimelinePainter old) =>
      !identical(rows, old.rows) ||
      !listEquals(selected, old.selected) ||
      !listEquals(keys, old.keys) ||
      frame != old.frame ||
      scale != old.scale ||
      offset != old.offset ||
      duration != old.duration ||
      !listEquals(markers, old.markers) ||
      rowDropGuide != old.rowDropGuide ||
      rowDropInside != old.rowDropInside ||
      labelWidth != old.labelWidth ||
      !identical(containers, old.containers) ||
      activeLane != old.activeLane ||
      fps != old.fps ||
      ruler != old.ruler ||
      marquee != old.marquee ||
      !listEquals(dragKeys, old.dragKeys) ||
      delta != old.delta ||
      !identical(dragLayer, old.dragLayer);
}

class _ArrangementOverview extends CustomPainter {
  const _ArrangementOverview({
    required this.layers,
    required this.duration,
    required this.extent,
    required this.frame,
    required this.offset,
    required this.scale,
    required this.viewportWidth,
  });
  final List<Map<String, dynamic>> layers;
  final int duration, extent, frame;
  final double offset, scale, viewportWidth;
  @override
  void paint(Canvas canvas, Size size) {
    canvas.drawRect(Offset.zero & size, Paint()..color = EditorTheme.app);
    final unit = size.width / math.max(1, extent);
    final lane = 12 / math.max(1, layers.length);
    canvas.save();
    canvas.clipRect(Offset.zero & size);
    for (var i = 0; i < layers.length; i++) {
      final l = layers[i];
      canvas.drawRect(
        Rect.fromLTWH(
          (l['start'] as num? ?? 0) * unit,
          3 + i * lane,
          (l['duration'] as num? ?? 1) * unit,
          math.max(1, lane - 1),
        ),
        Paint()
          ..color = l['hidden'] == true
              ? EditorTheme.border
              : EditorTheme.layerColor(l['id']),
      );
    }
    final left = (offset / scale * unit).clamp(0.0, size.width);
    final right = ((offset + viewportWidth) / scale * unit).clamp(
      left,
      size.width,
    );
    canvas.drawRect(
      Rect.fromLTRB(left, 1, right, 17),
      Paint()..color = Colors.white.withValues(alpha: .10),
    );
    canvas.drawRRect(
      RRect.fromRectAndRadius(
        Rect.fromLTRB(left, 1, right, 17),
        const Radius.circular(EditorMetrics.s3),
      ),
      Paint()
        ..color = EditorTheme.tab
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.5,
    );
    // 尺の印(壁ではない)と、今のコマ。
    if (extent > duration) {
      final x = duration * unit;
      canvas.drawLine(
        Offset(x, 1),
        Offset(x, 17),
        Paint()
          ..color = EditorTheme.muted
          ..strokeWidth = 1,
      );
    }
    final now = frame * unit;
    canvas.drawLine(
      Offset(now, 0),
      Offset(now, size.height),
      Paint()
        ..color = EditorTheme.keyAccent
        ..strokeWidth = 1.5,
    );
    canvas.restore();
  }

  @override
  bool shouldRepaint(covariant _ArrangementOverview old) =>
      !identical(layers, old.layers) ||
      duration != old.duration ||
      extent != old.extent ||
      frame != old.frame ||
      offset != old.offset ||
      scale != old.scale ||
      viewportWidth != old.viewportWidth;
}
