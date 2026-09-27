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
