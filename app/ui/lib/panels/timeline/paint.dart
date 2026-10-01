import 'dart:math' as math;

import 'package:flutter/foundation.dart' show listEquals;
import 'package:flutter/widgets.dart';

import '../../foundation/metrics.dart';
import '../../foundation/theme.dart';
import 'ink.dart';
import 'layout.dart';

/// Timeline のレーンと定規の絵 — 帯・鍵・ease の区間・名前の欄・落とし先の案内。
/// 絵を描くだけ: 書類も操作も知らず、渡された値だけを読む。
class TimelinePainter extends CustomPainter {
  final EditorTheme colors;

  TimelinePainter({
    this.colors = EditorTheme.chromatic,
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
    this.dragLayers = const {},
    this.ink = EditorInk.dark,
  });
  final EditorInk ink;
  final Rect? rowDropGuide;
  final bool rowDropInside;
  final double labelWidth;
  final List<LaneContainer> containers;
  final String? activeLane;
  final List<TrackRow> rows;
  // Every selected layer keeps its mark while a lane is being worked on:
  // the active lane adds a highlight, it never hides the others.
  bool layerSelected(TrackRow row) =>
      row.property == null && selected.contains(row.id);
  bool laneSelected(TrackRow row) =>
      layerSelected(row) || (activeLane != null && row.laneId == activeLane);
  final List<int> selected;
  final List<Map<String, dynamic>> keys, markers, dragKeys;
  final int frame, duration, delta;
  final double scale, offset, fps;
  final bool ruler;
  final Rect? marquee;
  final Map<int, Map<String, dynamic>> dragLayers;

  /// 文字は painter の既定(この板の墨と、定規かどうか)を結んで版へ渡す。
  void text(
    Canvas canvas,
    String text,
    Offset at, {
    Color? color,
    double width = 200,
    double size = 11,
    bool centered = false,
    FontWeight weight = FontWeight.w500,
  }) => paintText(
    canvas,
    text,
    at,
    color: color ?? colors.ink,
    width: width,
    size: size,
    centered: centered,
    weight: weight,
    rowAligned: !ruler,
  );

  bool selectedKey(
    TrackRow row,
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
    const h = timelineRowHeight;
    canvas.drawRect(Offset.zero & size, fillPaint(ink.laneGround));
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
          fillPaint(
            laneSelected(rows[i])
                ? colors.hover
                : rows[i].property != null
                ? ink.lane
                : (i.isEven ? ink.laneAlt : ink.lane),
          ),
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
          fillPaint(EditorTheme.white.withValues(alpha: .025)),
        );
      canvas.drawLine(
        Offset(x, 0),
        Offset(x, size.height),
        linePaint(ink.grid, 1.5),
      );
      for (var sub = 1; sub < 4; sub++) {
        if (gridStep * scale / 4 < 12) break;
        final sx = x + gridStep * scale * sub / 4;
        canvas.drawLine(
          Offset(sx, ruler ? size.height - 12 : 0),
          Offset(sx, size.height),
          linePaint(ink.gridMinor, 1),
        );
      }
      if (ruler) {
        canvas.drawLine(
          Offset(x, 9),
          Offset(x, size.height),
          linePaint(ink.tick),
        );
        for (var sub = 1; sub < 4; sub++) {
          final sx = x + gridStep * scale * sub / 4;
          canvas.drawLine(
            Offset(sx, size.height - 12),
            Offset(sx, size.height - 10),
            linePaint(ink.tickMinor),
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
          color: colors.muted,
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
              fillPaint(
                row.groupOpen
                    ? colors.border
                    : colors.timelineColor(child['id']),
              ),
            );
          }
        } else if (row.property == null) {
          final timing = dragLayers[row.id] ?? row.layer;
          final x =
              label +
              (timing['start'] as num? ?? 0).toDouble() * scale -
              offset;
          final width = (timing['duration'] as num? ?? 1).toDouble() * scale;
          final rect = Rect.fromLTWH(x, y + 2, math.max(1, width), h - 4);
          final own = colors.timelineColor(row.id);
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
                fillPaint(colors.muted.withValues(alpha: .28)),
              );
              canvas.drawRect(
                only.deflate(.5),
                strokePaint(own.withValues(alpha: .55), 1),
              );
            }
            if (ghost.left < zero) {
              final notch = scratchPath
                ..reset()
                ..moveTo(zero, ghost.top)
                ..lineTo(zero + 5, ghost.center.dy)
                ..lineTo(zero, ghost.bottom)
                ..close();
              canvas.drawPath(notch, fillPaint(own.withValues(alpha: .9)));
            }
          }
          final chosen = layerSelected(row);
          canvas.drawRect(
            rect,
            fillPaint(
              row.layer['hidden'] == true
                  ? colors.raised
                  : chosen
                  ? Color.lerp(own, EditorTheme.white, EditorTheme.lift)!
                  : own,
            ),
          );
          if (!row.lanesOpen)
            for (final f in row.summaryFrames) {
              bool at(List<Map<String, dynamic>> sel) =>
                  sel.any((s) => s['layer'] == row.id && s['frame'] == f);
              final cy = y + h / 2;
              void diamond(double kx, Color fill) {
                final path = scratchPath
                  ..reset()
                  ..moveTo(kx, cy - 3.5)
                  ..lineTo(kx + 3.5, cy)
                  ..lineTo(kx, cy + 3.5)
                  ..lineTo(kx - 3.5, cy)
                  ..close();
                canvas.drawPath(path, fillPaint(fill));
                canvas.drawPath(path, strokePaint(EditorTheme.black, 1));
              }

              // One diamond stands for every key at this frame. When only
              // some of them are in the grip, the ones staying keep their
              // diamond where it is and the grip carries the rest.
              final gripped = dragKeys
                  .where((s) => s['layer'] == row.id && s['frame'] == f)
                  .length;
              if (gripped > 0 &&
                  gripped < row.allKeys.where((k) => k['frame'] == f).length)
                diamond(label + f * scale - offset, colors.ink);
              diamond(
                label + (f + (gripped > 0 ? delta : 0)) * scale - offset,
                at(keys) ? colors.keyAccent : colors.ink,
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
            final paint = linePaint(
              chosen ? colors.keyAccent : colors.muted.withValues(alpha: .6),
              chosen ? 2 : 1,
            );
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
            final path = scratchPath
              ..reset()
              ..moveTo(x, y + h / 2 - 5)
              ..lineTo(x + 5, y + h / 2)
              ..lineTo(x, y + h / 2 + 5)
              ..lineTo(x - 5, y + h / 2)
              ..close();
            canvas.drawPath(
              path,
              fillPaint(
                selectedKey(row, key, keys) ? colors.keyAccent : colors.ink,
              ),
            );
          }
        }
      }
    if (!ruler) {
      for (double y = h; y <= rows.length * h && y <= size.height; y += h) {
        canvas.drawLine(
          Offset(label, y),
          Offset(size.width, y),
          linePaint(colors.line, 2),
        );
      }
    }
    canvas.drawLine(
      Offset(label, 0),
      Offset(size.width, 0),
      linePaint(colors.line, 2),
    );
    if (ruler)
      canvas.drawLine(
        Offset(label, size.height - 1),
        Offset(size.width, size.height - 1),
        linePaint(colors.line, 2),
      );
    for (final marker in markers) {
      final x =
          label + (marker['frame'] as num? ?? 0).toDouble() * scale - offset;
      canvas.drawLine(
        Offset(x, 0),
        Offset(x, size.height),
        linePaint(colors.muted.withValues(alpha: .4)),
      );
      if (ruler) canvas.drawCircle(Offset(x, 3), 3, fillPaint(colors.accent));
    }
    final playX = label + frame * scale - offset;
    if (ruler)
      canvas.drawPath(
        scratchPath
          ..reset()
          ..moveTo(playX - 5, 0)
          ..lineTo(playX + 5, 0)
          ..lineTo(playX, 6)
          ..close(),
        fillPaint(colors.keyAccent),
      );
    canvas.drawLine(
      Offset(playX, 0),
      Offset(playX, size.height),
      linePaint(colors.keyAccent, 1.5),
    );
    if (marquee != null) {
      canvas.drawRect(
        marquee!,
        fillPaint(colors.accent.withValues(alpha: .15)),
      );
      canvas.drawRect(marquee!, strokePaint(colors.accent));
    }
    canvas.restore();
    canvas.drawRect(
      Rect.fromLTWH(0, 0, math.min(label, size.width), size.height),
      fillPaint(colors.panel),
    );
    if (!ruler) {
      canvas.save();
      canvas.clipRect(Rect.fromLTWH(0, 0, label - 82, size.height));
      void surface(LaneContainer node) {
        final row = node.row;
        final background = row.property == null
            ? colors.timelineColor(row.id)
            : (laneSelected(row) ? colors.raised : colors.panel);
        canvas.drawRect(node.bounds, fillPaint(background));
        for (final child in node.children) surface(child);
        canvas.drawRect(node.bounds.deflate(.5), strokePaint(colors.line, 1));
      }

      for (final root in containers) surface(root);
      canvas.restore();
    }
    if (ruler) {
      text(
        canvas,
        'Layers',
        const Offset(6, 16),
        color: colors.muted,
        size: EditorMetrics.dense,
        weight: FontWeight.w600,
      );
      // 今のコマ / 尺。定規の左の欄に固定で描くので、桁が変わっても何も押し退けない。
      text(
        canvas,
        '$frame / $duration',
        Offset(label - 82, 16),
        color: colors.muted,
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
            fillPaint(colors.raised),
          );
        // Chosen rows read as one lit surface from the name cell to the
        // time field; nothing is outlined, the selection is a region.
        if (layerSelected(row))
          canvas.drawRect(
            Rect.fromLTWH(row.bounds.left, y, label - row.bounds.left, h),
            fillPaint(EditorTheme.white.withValues(alpha: EditorTheme.lift)),
          );
        if (row.property == null) {
          text(
            canvas,
            row.isGroup ? (row.groupOpen ? '⊟' : '⊞') : '',
            Offset(row.bounds.left + 5, y + 3),
            width: EditorMetrics.s12,
            centered: true,
            color: ink.headerInk,
          );
          text(
            canvas,
            '${row.layer['name']}',
            color: ink.headerInk,
            Offset(row.bounds.left + 23, y + 3),
            width: math.max(0.0, row.bounds.width - 29),
            weight: FontWeight.w500,
          );
          final keysOpen = row.lanesOpen;
          canvas.drawRect(
            Rect.fromLTWH(label - 81, y + 2, 14, h - 4),
            fillPaint(keysOpen ? colors.accent : colors.line),
          );
          text(
            canvas,
            keysOpen ? '◆' : '◇',
            Offset(label - 81, y + 3),
            width: EditorMetrics.s14,
            size: EditorMetrics.dense,
            centered: true,
            color: keysOpen ? ink.headerInk : colors.ink,
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
              fillPaint(on ? colors.accent : colors.line),
            );
            text(
              canvas,
              symbols[column],
              Offset(x, y + 3),
              width: EditorMetrics.s14,
              size: EditorMetrics.micro,
              centered: true,
              weight: FontWeight.w600,
              color: on ? ink.headerInk : colors.muted,
            );
          }
        } else {
          text(
            canvas,
            '${row.property!['label'] ?? row.property!['id'] ?? 'Content'}',
            Offset(row.bounds.left + 6, y + 3),
            width: EditorMetrics.s160,
            color: colors.muted,
          );
          text(
            canvas,
            row.property!['keyedNow'] == true ? '◆' : '◇',
            Offset(label - 21, y + 3),
            width: EditorMetrics.s17,
            color: colors.keyAccent,
          );
        }
        canvas.drawLine(
          Offset(row.bounds.left, y + h),
          Offset(size.width, y + h),
          linePaint(colors.line, 2),
        );
      }
    if (rowDropGuide case final Rect guide) {
      final paint = strokePaint(colors.select, 2);
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
          fillPaint(colors.select),
        );
      }
    }
    if (ruler)
      canvas.drawLine(
        Offset(0, size.height - 1),
        Offset(size.width, size.height - 1),
        linePaint(colors.line, 2),
      );
    canvas.drawLine(
      Offset(label, 0),
      Offset(label, size.height),
      linePaint(colors.line),
    );
  }

  @override
  bool shouldRepaint(covariant TimelinePainter old) =>
      colors != old.colors ||
      ink != old.ink ||
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
      !identical(dragLayers, old.dragLayers);
}
