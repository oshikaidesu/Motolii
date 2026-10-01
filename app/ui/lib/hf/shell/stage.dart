// The Stage seat of the reference, at its own coordinates: the seat's edges and the composition tab row. What sits
// under the tab row (the work area and its controls) is the Stage itself, handed in as [StageModel.body].
import 'package:flutter/widgets.dart';
import '../glyphs.dart';
import 'place.dart';

class StageModel {
  const StageModel({required this.body, this.title = 'Composition 1'});

  /// The Stage under the tab row: 777 x 591 at (345, 101).
  final List<RI> body;
  final String title;
}

List<RI> stage(StageModel m) => <RI>[
      Ln(344, 62, 1, 631, H.rule), Ln(1122, 62, 1, 631, H.rule), Ln(344, 62, 779, 1, H.rule), Ln(344, 692, 779, 1, H.rule),
      Rc(345, 71, 140, 30, fill: H.sel, borders: const Border(top: BorderSide(color: H.rule), left: BorderSide(color: H.rule), right: BorderSide(color: H.rule))),
      Tx(358, 90, m.title, H.s(13, w: FontWeight.w500, color: H.text), w: 91),
      Tx(463, 90, '×', H.s(16, color: H.text2)),
      Rc(486, 71, 40, 30, fill: H.raised, borders: const Border(top: BorderSide(color: H.rule), right: BorderSide(color: H.rule))),
      Hg(506, 87, 17, HG.plus, const Color(0xFFB4B3B5)),
      Ln(345, 100, 777, 1, H.rule2),
      ...m.body,
    ];

class StageBodyModel {
  const StageBodyModel({required this.picture, this.zoom = '100%', this.onZoom, this.onFit, this.onView});

  /// What fills the work area (777 x 537 at 345, 101).
  final Widget picture;
  final String zoom;

  /// The zoom box and the view box were tapped; [at] is where they are on screen, for a menu.
  final void Function(Rect at)? onZoom, onView;
  final VoidCallback? onFit;
}

RI _tap(double x, double y, double w, double h, void Function(Rect at)? f) => Wd(
      x, y, w, h,
      Builder(
        builder: (context) => MouseRegion(
          cursor: f == null ? SystemMouseCursors.basic : SystemMouseCursors.click,
          child: GestureDetector(
            behavior: HitTestBehavior.opaque,
            onTap: f == null
                ? null
                : () {
                    final box = context.findRenderObject() as RenderBox;
                    f(box.localToGlobal(Offset.zero) & box.size);
                  },
            child: const SizedBox.expand(),
          ),
        ),
      ),
    );

/// The reference's Stage under the tab row: the work area, the tool column over it, and the bottom bar.
List<RI> stageBody(StageBodyModel m) {
  const tools = [HG.arrow, HG.move, HG.rect, HG.ellipse, HG.pen, HG.type, HG.crop];
  const dd = Color(0xFF1D1D1D);
  const ddB = Color(0xFF3E3E3D);
  final fit = m.onFit;
  return [
    Wd(345, 101, 777, 537, m.picture),
    Ln(345, 637, 777, 1, H.rule2),
    // tool column, over the work
    Rc(354, 111, 42, 304, fill: H.raised, border: H.rule, r: 2),
    Rc(355, 112, 40, 43, fill: H.selHi),
    for (var i = 1; i < 7; i++) Ln(355, 111 + 43.4 * i, 40, 1, H.rule2),
    for (var i = 0; i < 7; i++) Hg(375, 111 + 43.4 * i + 21.7, 23, tools[i], i == 0 ? const Color(0xFFEEEEF0) : const Color(0xFFD8D8DA), bg: const Color(0xFF222223)),
    // bottom bar
    Rc(354, 649, 117, 33, fill: dd, border: ddB, r: 3),
    Tx(371, 670, m.zoom, H.s(13, color: H.text), w: 31),
    Hg(453, 665, 15, HG.chevronDown, const Color(0xFFDDDDDD)),
    Rc(489, 649, 38, 33, fill: dd, border: ddB, r: 3), Hg(508, 665, 20, HG.fit, const Color(0xFFD0D0D0)),
    Rc(534, 649, 37, 33, fill: dd, border: ddB, r: 3), Hg(552, 665, 20, HG.fit, const Color(0xFFD0D0D0)),
    Rc(894, 649, 144, 33, fill: dd, border: ddB, r: 3),
    Tx(905, 670, 'Camera View', H.s(13, color: H.text), w: 80),
    Hg(1020, 665, 15, HG.chevronDown, const Color(0xFFDDDDDD)),
    Rc(1057, 649, 37, 33, fill: dd, border: ddB, r: 3), Hg(1075.5, 665, 20, HG.corners, const Color(0xFFD0D0D0)),
    // hit areas over what is drawn above; a control with no operation does nothing
    _tap(354, 649, 117, 33, m.onZoom),
    _tap(489, 649, 38, 33, fit == null ? null : (_) => fit()),
    _tap(894, 649, 144, 33, m.onView),
  ];
}
