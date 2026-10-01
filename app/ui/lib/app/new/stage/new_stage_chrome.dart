import 'package:flutter/widgets.dart';

import '../../../hf/bp/common.dart';
import '../../../hf/bp/shell.dart' show kTile;
import '../../../hf/desk/common.dart' show kAccent;
import '../../../hf/glyphs.dart';
import '../../../panels/stage.dart' show StageToolbarApi;

/// The Stage's own bars, hf-styled, over exactly the [StageToolbarApi] operations Classic's bars call. The picture,
/// the gesture surface and the gizmos underneath are unchanged — this only redraws the strip above and below them.
Widget _scrollingBar({required double height, required List<Widget> children}) => Container(
      height: height,
      color: kRaised,
      child: LayoutBuilder(
        builder: (context, box) => SingleChildScrollView(
          scrollDirection: Axis.horizontal,
          child: ConstrainedBox(
            constraints: BoxConstraints(minWidth: box.maxWidth),
            child: IntrinsicWidth(
              child: Padding(padding: const EdgeInsets.symmetric(horizontal: 6), child: Row(children: children)),
            ),
          ),
        ),
      ),
    );

Widget newStageTopBar(BuildContext context, StageToolbarApi api) => _scrollingBar(height: 28, children: [
      if (api.userStage) _btn('Front', api.canGoHome ? api.goHome : null),
      const Spacer(),
      _btn('Fit', api.fit),
      _btn('100%', api.resetZoom),
      _btn('−', api.zoomOut),
      SizedBox(
        width: 46,
        child: Text('${api.zoomPercent.round()}%', textAlign: TextAlign.center, style: sans(11, c: kMuted)),
      ),
      _btn('+', api.zoomIn),
    ]);

Widget newStageBottomBar(BuildContext context, StageToolbarApi api) => _scrollingBar(height: 24, children: [
      Text('${api.width.round()} × ${api.height.round()}', style: sans(10, c: kMuted)),
      const SizedBox(width: 8),
      GestureDetector(
        key: const ValueKey('stage:transparentGround'),
        behavior: HitTestBehavior.opaque,
        onTap: api.canToggleGround ? api.toggleGround : null,
        child: SizedBox(width: 20, height: 12, child: CustomPaint(painter: HgPainter(HG.grid, api.transparentGround ? kAccent : kMuted, kRaised))),
      ),
      if (api.userStage) ...[
        const SizedBox(width: 8),
        _btn(api.extending ? '● Extend' : 'Extend', api.toggleExtend),
      ],
      const Spacer(),
      if (!api.gesturesAvailable) ...[
        Text('Transform gestures unavailable', style: sans(10, c: kMuted)),
        const SizedBox(width: 8),
      ],
      Text('Frame ${api.frame}', style: sans(10, c: kMuted)),
    ]);

Widget _btn(String label, VoidCallback? onTap) => GestureDetector(
      behavior: HitTestBehavior.opaque,
      onTap: onTap,
      child: Container(
        height: 20,
        margin: const EdgeInsets.symmetric(horizontal: 2),
        padding: const EdgeInsets.symmetric(horizontal: 8),
        alignment: Alignment.center,
        decoration: BoxDecoration(color: kTile, borderRadius: BorderRadius.circular(4)),
        child: Text(label, style: sans(10.5, c: onTap == null ? const Color(0xFF55565C) : const Color(0xFFD0D1D5))),
      ),
    );

