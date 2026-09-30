import 'package:flutter/widgets.dart';

import '../../../../browser/parts.dart';

import '../../../../desks/parts.dart' show kAccentDim;
import '../../../panels/timeline.dart' show TimelineToolbarApi;
import '../../../../theme/surface.dart' show Surface;

/// The Timeline's own play/zoom buttons, hf-styled, over the same [TimelineToolbarApi] operations Classic's
/// bar calls. The overview strip beside them, the ruler, the tracks and the keyframes below are unchanged.
Widget newTimelineBarButtons(BuildContext context, TimelineToolbarApi api) => ListenableBuilder(
      listenable: api.playing,
      builder: (context, _) => Row(mainAxisSize: MainAxisSize.min, children: [
        const SizedBox(width: 6),
        _btn(api.playing.value ? 'Ⅱ' : '▶', api.togglePlayback, selected: api.playing.value),
        _btn('−', api.zoomOut),
        SizedBox(width: 46, child: Text('${api.zoomPercent.round()}%', textAlign: TextAlign.center, style: sans(11, c: Surface.muted))),
        _btn('+', api.zoomIn),
        _btn('Fit', api.fit),
      ]),
    );

Widget _btn(String label, VoidCallback? onTap, {bool selected = false}) => GestureDetector(
      behavior: HitTestBehavior.opaque,
      onTap: onTap,
      child: Container(
        height: 20,
        margin: const EdgeInsets.symmetric(horizontal: 2),
        padding: const EdgeInsets.symmetric(horizontal: 8),
        alignment: Alignment.center,
        decoration: BoxDecoration(color: selected ? kAccentDim : Surface.raised, borderRadius: BorderRadius.circular(4)),
        child: Text(label, style: sans(10.5, c: onTap == null ? const Color(0xFF55565C) : const Color(0xFFD0D1D5))),
      ),
    );
