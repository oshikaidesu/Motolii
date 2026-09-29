import 'dart:math' as math;

import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import 'frame.dart';
import 'layout.dart';
import 'semantics.dart';

/// Timeline の掴み手 — 押す・選ぶ・動かす・詰める・滑らす・囲む、
/// そして行と素材の落とし先。絵に出す途中の姿もここが持つ。
mixin TimelineGrip<T extends StatefulWidget> on State<T>, TimelineFrame<T> {
  int? anchor;
  String? gesture;
  List<int> rowDragIds = [];
  Map<String, dynamic>? rowDrop;
  Offset? assetDrop;
  Rect? rowDropGuide;
  bool rowDropInside = false;
  Offset? start, current;
  TrackRow? dragRow;
  List<TrackRow> timingRows = [];
  Future<void>? previewFlight;

  List<Map<String, dynamic>>? queuedTimings;
  bool previewUsed = false;
  bool finishing = false;
  Future<void> pumpPreview() async {
    try {
      while (queuedTimings != null) {
        final changes = queuedTimings!;
        queuedTimings = null;
        await timelineSession.command('previewTimings', {'changes': changes});
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
      await timelineSession.command('commitPreview');
    else if (has('setTimings'))
      await timelineSession.command('setTimings', {'changes': changes});
    finishing = false;
  }

  int deltaFrames = 0;
  List<Map<String, dynamic>> initialKeys = [];
  List<int> initialIds = [];
  List<Map<String, dynamic>> settlingKeys = const [];
  int settlingDelta = 0;
  bool additive = false;

  /// Both ends of the span under x on a property row, or null off any span.
  List<Map<String, dynamic>>? spanAt(TrackRow row, double x) {
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

  Map<String, dynamic> keyOf(TrackRow row, Map<String, dynamic> key) => {
    'layer': row.id,
    'property': row.property?['id'],
    'frame': key['frame'],
  };
  bool sameKey(Map<String, dynamic> a, Map<String, dynamic> b) =>
      a['layer'] == b['layer'] &&
      a['property'] == b['property'] &&
      a['frame'] == b['frame'];
  void chooseLayer(int id) {
    var ids = timelineSession.selectedIds.toList();
    if (HardwareKeyboard.instance.isShiftPressed && anchor != null) {
      final ordered = timelineSession.layers
          .map((l) => (l['id'] as num).toInt())
          .toList();
      final a = ordered.indexOf(anchor!), b = ordered.indexOf(id);
      if (a >= 0 && b >= 0) {
        final range = ordered.sublist(math.min(a, b), math.max(a, b) + 1);
        ids = primary ? {...ids, ...range}.toList() : range;
      }
    } else if (primary) {
      anchor = id;
      timelineSession.command('select', {'toggle': id});
      return;
    } else {
      ids = [id];
    }
    anchor = id;
    timelineSession.command('select', {'ids': ids, 'keys': []});
  }

  void begin(PointerDownEvent event) {
    beginAt(event, event.localPosition);
  }

  /// Pointer entry point for presentations with a transformed drawing surface.
  void beginAt(PointerDownEvent event, Offset p) {
    focus.requestFocus();
    if (finishing) return;
    if (event.buttons != kPrimaryMouseButton) return;
    final index = layout.rowAt(p.dy);
    start = p;
    current = p;
    deltaFrames = 0;
    additive = primary || HardwareKeyboard.instance.isShiftPressed;
    initialKeys = selectedKeys;
    initialIds = timelineSession.selectedIds.toList();
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
          relane();
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
          relane();
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
          if (has('clip')) timelineSession.command('clip', {'layer': row.id});
        } else if (has('toggle'))
          timelineSession.command('toggle', {'layer': row.id, 'flag': field});
        gesture = null;
        return;
      }
      if (row.property != null && p.dx >= labelWidth - 25 && has('toggleKey')) {
        timelineSession.command('toggleKey', {
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
        final chosen = tlPick(selectedKeys, found, additive: additive);
        timelineSession.command('select', {
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
        final chosen = tlPick(selectedKeys, picked, additive: additive);
        timelineSession.command('select', {
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
            ? timelineSession.layers
                  .where(
                    (l) => initialIds.contains(l['id']) && l['locked'] != true,
                  )
                  .map((l) => TrackRow(l))
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
    moveAt(event, event.localPosition);
  }

  void moveAt(PointerMoveEvent event, Offset p) {
    if (gesture == null || start == null) return;
    if (gesture == 'rowPending' || gesture == 'rowDrag') {
      if ((p - start!).distance < 4 && gesture == 'rowPending') return;
      setState(() {
        gesture = 'rowDrag';
        current = p;
        aimRowDrop(current!, rowDragIds);
      });
      return;
    }
    setState(() {
      current = p;
      deltaFrames = ((current!.dx - start!.dx) / pixelsPerFrame).round();
      if (timingRows.isNotEmpty &&
          ['move', 'trimIn', 'trimOut', 'slip'].contains(gesture)) {
        final starts = timingRows.map(
          (r) => (r.layer['start'] as num? ?? 0).toInt(),
        );
        if (gesture == 'move')
          deltaFrames = math.max(deltaFrames, -starts.reduce(math.min));
      }
      // Keys stop at frame 0 the way the bars do, and native clamps the same
      // way, so what is drawn is what lands.
      if (gesture == 'keys' && initialKeys.isNotEmpty)
        deltaFrames = math.max(
          deltaFrames,
          -initialKeys.map((k) => (k['frame'] as num).toInt()).reduce(math.min),
        );
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
    timelineSession.command('placeAsset', {
      'id': details.data['asset'],
      'target': drop['target'],
      'placement': drop['placement'],
      if (at != null && at.dx >= labelWidth) 'start': frameAt(at.dx),
    });
  }

  void aimAsset(DragTargetDetails<Map<String, dynamic>> details) {
    final box = rowsKey.currentContext?.findRenderObject() as RenderBox?;
    if (box == null) return;
    aimAssetAt(box.globalToLocal(details.offset));
  }

  /// [at] in the tracks' own coordinates, for presentations that place the rows themselves.
  void aimAssetAt(Offset at) => setState(() {
    assetDrop = at;
    aimRowDrop(at, const []);
  });

  void leaveAsset() => setState(() {
    assetDrop = null;
    rowDrop = null;
    rowDropGuide = null;
  });

  /// Where the bar is drawn while gripped (a picture; see [timing] for what is sent).
  Map<String, dynamic> drawnTiming(TrackRow row) => tlDrawnTiming(row.id, row.layer, const {
        'trimIn': TlGesture.trimIn,
        'trimOut': TlGesture.trimOut,
        'slip': TlGesture.slip,
      }[gesture] ?? TlGesture.move, deltaFrames);

  /// The kind of timing change and how far: the host works out the new start, length and source (timing_delta),
  /// against the document as it is when the change arrives.
  Map<String, dynamic> timing(TrackRow row) => {
        'layer': row.id,
        'mode': const {'trimIn': 'trimStart', 'trimOut': 'trimEnd', 'slip': 'slip'}[gesture] ?? 'move',
        'delta': deltaFrames,
      };

  void end(PointerUpEvent event) {
    endAt(event, event.localPosition);
  }

  void endAt(PointerUpEvent event, Offset p) {
    if (gesture == null) return;
    current = p;
    if (gesture == 'rowPending' || gesture == 'rowDrag') {
      if (gesture == 'rowDrag' && rowDrop != null) {
        if (has('moveLayers'))
          timelineSession.command('moveLayers', Map.of(rowDrop!));
        else
          timelineSession.error.value =
              'Layer move is waiting for the native update';
      } else if (gesture == 'rowPending' && dragRow != null)
        chooseLayer(dragRow!.id);
      resetGesture();
      return;
    }
    if (gesture == 'keys' && deltaFrames != 0) {
      // The shift stays drawn until the document carries it; otherwise the
      // keys jump back for the frames between the release and the reply.
      settlingKeys = initialKeys;
      settlingDelta = deltaFrames;
      timelineSession
          .command('moveKeys', {'deltaFrames': deltaFrames})
          .whenComplete(() {
            if (!mounted) return;
            setState(() {
              settlingKeys = const [];
              settlingDelta = 0;
            });
          });
    } else if (['move', 'trimIn', 'trimOut', 'slip'].contains(gesture) &&
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
                timelineGeometry.rowHeight - 4,
              ),
            ))
              ids.add(row.id);
          }
        }
      }
      timelineSession.command('select', {'ids': ids.toList(), 'keys': keys});
    }
    if (previewUsed) {
      queuedTimings = null;
      previewUsed = false;
      timelineSession.cancelPreview();
    }
    resetGesture();
  }

  void cancel() {
    queuedTimings = null;
    if (previewUsed) {
      previewUsed = false;
      timelineSession.cancelPreview();
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

  @override
  void dispose() {
    queuedTimings = null;
    if (previewUsed) timelineSession.cancelPreview();
    super.dispose();
  }
}
