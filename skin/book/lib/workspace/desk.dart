// The Desk: the drawer under the Inspector. It follows the selection and holds one tool at a time (keys picked -> Ease,
// a camera -> Depth, otherwise the Tools catalog); a tool opened by hand stays pinned until the selection moves or you unpin it.
import 'package:flutter/widgets.dart';

import '../sets/inspector/inspector_parts.dart' show Hov;
import '../tokens.dart';
import 'desk_depth.dart';
import 'desk_ease.dart';
import 'desk_kit.dart';
import 'ws.dart';

class WsDesk extends StatelessWidget {
  const WsDesk({super.key});
  @override
  Widget build(BuildContext context) {
    final ws = WsScope.of(context), name = ws.desk;
    return WsSeat(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          _DeskHeader(name: name),
          Expanded(
            child: switch (name) {
              'Ease' => const DeskEase(),
              'Depth' => const DeskDepth(),
              'Tools' => const _Catalog(),
              _ => _Placeholder(name: name),
            },
          ),
        ],
      ),
    );
  }
}

/// The tool's sticker and name, whether it follows the selection or is pinned, and the way back to the catalog.
class _DeskHeader extends StatelessWidget {
  const _DeskHeader({required this.name});
  final String name;
  @override
  Widget build(BuildContext context) {
    final ws = WsScope.of(context), pinned = ws.deskManual != null;
    return LayoutBuilder(
      builder: (context, box) {
        final wide = box.maxWidth >= 340;
        return WsHeader(
          name,
          leading: DeskSticker(name),
          trailing: [
            if (pinned)
              DeskChip(label: wide ? 'Pinned · Follow' : 'Pinned', dot: Grey.g76, onTap: () => ws.openDesk(null))
            else
              DeskChip(label: wide ? 'Follows selection' : 'Follows', dot: WsT.accent),
            const SizedBox(width: WsT.gap),
            DeskIconButton(glyph: (ink) => DeskGlyph('Tools', ink), on: name == 'Tools', onTap: () => ws.openDesk(name == 'Tools' && pinned ? null : 'Tools')),
          ],
        );
      },
    );
  }
}

/// Every desk tool as a tile: sticker, name, what it acts on. The one the selection would pick wears an Auto tag.
/// Columns come from the width; the rows share the height while each keeps [_minTile], else the grid scrolls; a tall desk never stretches a tile past [_maxTile].
class _Catalog extends StatelessWidget {
  const _Catalog();
  static const _cell = 104.0, _minTile = 62.0, _maxTile = 92.0;
  @override
  Widget build(BuildContext context) {
    final ws = WsScope.of(context), auto = deskAuto(ws);
    return LayoutBuilder(
      builder: (context, box) {
        final w = box.maxWidth - WsT.gap * 2, h = box.maxHeight - WsT.gap * 2;
        final cols = wsCols(w, _cell, gap: WsT.gap, min: 2), rows = (deskTools.length / cols).ceil();
        final tile = ((h - WsT.gap * (rows - 1)) / rows).clamp(_minTile, _maxTile), cw = wsCell(w, cols, gap: WsT.gap);
        return ListView(
          padding: const EdgeInsets.all(WsT.gap),
          children: [
            Wrap(
              spacing: WsT.gap,
              runSpacing: WsT.gap,
              children: [
                for (final t in deskTools)
                  SizedBox(
                    width: cw,
                    height: tile,
                    child: _ToolTile(t, auto: t.name == auto, onTap: () => ws.openDesk(t.name)),
                  ),
              ],
            ),
          ],
        );
      },
    );
  }
}

class _ToolTile extends StatelessWidget {
  const _ToolTile(this.tool, {required this.auto, required this.onTap});
  final DeskTool tool;
  final bool auto;
  final VoidCallback onTap;
  @override
  Widget build(BuildContext context) => Hov(
    onTap: onTap,
    builder: (_, h) => Container(
      padding: const EdgeInsets.fromLTRB(7, 7, 6, 7),
      decoration: BoxDecoration(color: h ? WsT.raised : WsT.card, borderRadius: BorderRadius.circular(WsT.radius)),
      child: LayoutBuilder(
        builder: (context, box) => Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                DeskSticker(tool.name, size: 26),
                const Spacer(),
                if (auto)
                  Flexible(
                    child: FittedBox(
                      fit: BoxFit.scaleDown,
                      alignment: Alignment.topRight,
                      child: Container(
                        height: 14,
                        padding: const EdgeInsets.symmetric(horizontal: 4),
                        alignment: Alignment.center,
                        decoration: BoxDecoration(color: WsT.accent, borderRadius: BorderRadius.circular(WsT.chipRadius)),
                        child: Text('AUTO', style: T.micro(WsT.onAccent).copyWith(fontSize: 9, fontWeight: FontWeight.w700, letterSpacing: .6)),
                      ),
                    ),
                  ),
              ],
            ),
            const Spacer(),
            Text(
              tool.name,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: T.name(h ? Grey.g100 : Grey.g95).copyWith(fontSize: 12, fontWeight: FontWeight.w700),
            ),
            if (box.maxHeight >= 54) ...[const SizedBox(height: 3), Text(tool.acts, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(Grey.g56))],
          ],
        ),
      ),
    ),
  );
}

/// A tool the lab has not drawn yet: a poster of it (its hue, its glyph, its name) and the way back.
class _Placeholder extends StatelessWidget {
  const _Placeholder({required this.name});
  final String name;
  @override
  Widget build(BuildContext context) {
    final ws = WsScope.read(context), tool = deskTool(name);
    return Padding(
      padding: const EdgeInsets.only(top: WsT.gutter),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Expanded(
            child: Container(
              color: WsT.card,
              padding: const EdgeInsets.fromLTRB(WsT.inset * 1.5, WsT.inset * 1.5, WsT.inset, WsT.inset),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text('DESK TOOL', style: T.micro(tool.tone).copyWith(fontWeight: FontWeight.w700, letterSpacing: 1.4)),
                  const SizedBox(height: 6),
                  FittedBox(
                    fit: BoxFit.scaleDown,
                    alignment: Alignment.centerLeft,
                    child: Text(name, style: T.title(Grey.g95).copyWith(fontSize: 34, fontWeight: FontWeight.w800, letterSpacing: -1, height: 1)),
                  ),
                  const SizedBox(height: 10),
                  Text('Works on ${tool.acts.toLowerCase()}.', style: T.label(Grey.g76).copyWith(fontSize: 11)),
                  const SizedBox(height: 5),
                  Text('Not designed in the lab yet.', style: T.label(Grey.g56)),
                  const Spacer(),
                  Wrap(
                    spacing: WsT.gap,
                    runSpacing: WsT.gap,
                    children: [
                      DeskChip(label: 'Back to Tools', dot: Grey.g76, onTap: () => ws.openDesk('Tools')),
                      DeskChip(label: 'Follow selection', dot: WsT.accent, onTap: () => ws.openDesk(null)),
                    ],
                  ),
                ],
              ),
            ),
          ),
          const SizedBox(width: WsT.gutter),
          AspectRatio(
            aspectRatio: .5,
            child: ColoredBox(
              color: tool.tone,
              child: Center(
                child: FractionallySizedBox(
                  widthFactor: .8,
                  child: AspectRatio(aspectRatio: 1, child: CustomPaint(painter: DeskGlyph(name, WsT.onAccent))),
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
