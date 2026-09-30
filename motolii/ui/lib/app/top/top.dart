// The top seat of the reference, at its own coordinates. What it shows and what its keys do come from a [TopModel]:
// proto_hf hands it fixed values, production hands it the session.
import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';
import '../../theme/glyphs.dart';
import '../../theme/tokens.dart';
import '../../theme/neutral.dart';

/// The three modes of the top seat's track.
enum TopMode { edit, play, export }

class TopModel {
  const TopModel({
    required this.readouts,
    this.mode = TopMode.edit,
    this.onPlay,
    this.onStop,
    this.onAnimate,
    this.onMarker,
    this.onMode,
    this.onFit,
    this.onPin,
    this.onOpen,
  });

  /// The three readouts left to right; the only part that follows the playhead.
  final ValueListenable<List<String>> readouts;

  final TopMode mode;
  final VoidCallback? onPlay, onStop, onAnimate, onMarker;
  final ValueChanged<TopMode>? onMode;

  /// The keys on the right: Fit, Pin, Open. A key with no operation is shown but does nothing.
  final VoidCallback? onFit, onPin, onOpen;
}

RI _tap(double x, double y, double w, double h, VoidCallback? f) => Wd(
      x, y, w, h,
      MouseRegion(cursor: f == null ? SystemMouseCursors.basic : SystemMouseCursors.click, child: GestureDetector(behavior: HitTestBehavior.opaque, onTap: f, child: const SizedBox.expand())),
    );

List<RI> top(TopModel m) {
  const keyBorder = N.g20;
  Rc key(double x, double w, Color fill, Color border) => Rc(x, 14, w, 37, fill: fill, border: border, r: 3);
  const readout = N.g86;
  return [
    Ln(0, 61, 1536, 1, H.rule),
    Tx(20, 43, 'Motolii', H.s(28, w: FontWeight.w600, ls: -0.4, color: H.text2), w: 97),
    Tx(152, 30, 'Motion', H.s(12, color: N.g76)),
    Tx(152, 44, 'for More Relations.', H.s(12, color: N.g76), w: 102),
    key(363, 41, H.play, H.play),
    key(410, 40, N.g10, keyBorder),
    key(456, 41, N.g13, N.g26),
    Pt(_TopGlyphs()),
    Wd(0, 0, 1536, 62, ValueListenableBuilder<List<String>>(
      valueListenable: m.readouts,
      builder: (_, r, __) => RF([
        Tx(533, 37, r[0], H.m(12.5, color: readout), w: 44),
        Tx(616, 37, r[1], H.m(12.5, color: readout), w: 33),
        Tx(687, 37, r[2], H.m(12.5, color: readout), w: 60),
      ]),
    )),
    Hg(770, 31, 15, HG.plus, N.g82),
    Rc(830, 14, 264, 37, fill: H.raised, border: H.rule, r: 3,
        child: ClipRRect(borderRadius: BorderRadius.circular(2), child: Align(alignment: Alignment(m.mode.index - 1.0, 0), child: const SizedBox(width: 88, height: double.infinity, child: ColoredBox(color: H.mode))))),
    Tx(860, 37, 'EDIT', H.s(12, w: FontWeight.w600, ls: 0.8, color: m.mode == TopMode.edit ? N.g100 : H.text2), w: 29),
    Tx(947, 37, 'PLAY', H.s(12, w: FontWeight.w600, ls: 0.8, color: m.mode == TopMode.play ? N.g100 : H.text2), w: 29),
    Tx(1026, 37, 'EXPORT', H.s(12, w: FontWeight.w600, ls: 0.8, color: m.mode == TopMode.export ? N.g100 : H.text2), w: 46),
    for (final x in [1148.0, 1201.0, 1252.0]) Rc(x, 14, 40, 37, fill: H.raised, border: H.rule, r: 3),
    Hg(1168, 32, 24, HG.fit, N.g86, bg: N.g13),
    Hg(1221, 32, 24, HG.pin, N.g86),
    Hg(1272, 32, 24, HG.folder, N.g86),
    Tx(1506, 27, 'Less numbers.', H.m(12, color: H.text2), al: Al.right, w: 101),
    Tx(1506, 45, 'More motion.', H.m(12, color: H.text2), al: Al.right, w: 92),
    // the keys: invisible hit areas over what is drawn above; a key with no operation does nothing
    _tap(363, 14, 41, 37, m.onPlay),
    _tap(410, 14, 40, 37, m.onStop),
    _tap(456, 14, 41, 37, m.onAnimate),
    _tap(758, 19, 24, 24, m.onMarker),
    for (final (i, mode) in TopMode.values.indexed) _tap(830 + 88.0 * i, 14, 88, 37, m.onMode == null ? null : () => m.onMode!(mode)),
    for (final (x, f) in [(1148.0, m.onFit), (1201.0, m.onPin), (1252.0, m.onOpen)]) _tap(x, 14, 40, 37, f),
  ];
}

class _TopGlyphs extends CustomPainter {
  @override
  void paint(Canvas cv, Size s) {
    cv.drawPath(Path()..moveTo(377.4, 24.5)..lineTo(393.4, 32.5)..lineTo(377.4, 40.5)..close(), Paint()..color = N.g00);
    cv.drawRect(const Rect.fromLTWH(424.5, 27, 11, 11), Paint()..color = N.g56);
    cv.drawCircle(const Offset(475.5, 32), 6.5, Paint()..color = H.record);
    cv.drawRect(const Rect.fromLTWH(596, 24, 1.2, 14), Paint()..color = N.g44);
  }

  @override
  bool shouldRepaint(_TopGlyphs o) => false;
}
