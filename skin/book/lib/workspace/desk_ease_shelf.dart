// The shelf drawn out from the right of the Ease panel and put away (the maker, 10/31, drawn on a picture of it: "右から出せ
// る、しまえるプリセットメニュー", room for some fifty, what cards make among them). It comes in as a column of its own, pushing
// the rest left, so the flow stays in sight and its cards can be carried to it; it slides as the colours' boxes do
// (dock_make_2.dart); as many columns as the pane gives room for, Easey's presetsPage.js. A card carried to the shapes draws
// it out, as a Finder folder a thing is held over opens (macOS's spring-loaded folders).
import 'dart:convert';
import 'dart:io';

import 'package:flutter/widgets.dart';
import 'package:lab_book/motolii.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart' show Hov;
import 'package:lab_book/tokens.dart';
import 'package:lab_book/workspace/desk_ease.dart';
import 'package:lab_book/workspace/desk_ease_cards.dart';
import 'package:lab_book/workspace/desk_ease_presets.dart';
import 'package:lab_book/workspace/desk_ease_side.dart' show EaseChevron;
import 'package:lab_book/workspace/ws.dart';

/// What a hand kept, each as it is said (a curve, a card, all the cards), as a device carried to Live's browser by its title
/// bar is saved (Live's manual, 23.2.5 Saving Presets). In ~/.motolii, as the colours are in colors.json.
File _keptFile = File('${Platform.environment['HOME']}/.motolii/ease_kept.json');
List<String>? _kept;

/// (a test points it elsewhere, and forgets what was read: as coloursAt)
void easeKeptAt(File f) => (_keptFile = f, _kept = null);

List<String> easeKept() => _kept ??= () {
  try {
    return [for (final w in jsonDecode(_keptFile.readAsStringSync()) as List) '$w'];
  } catch (_) {
    return <String>[];
  }
}();

void easeKeep(String word, {bool off = false}) {
  final kept = easeKept()..remove(word);
  if (!off) kept.add(word);
  try {
    _keptFile.parent.createSync(recursive: true);
    _keptFile.writeAsStringSync(jsonEncode(kept));
  } catch (_) {}   // (not kept: known while the window is open)
}

final _tiles = <String, Ease>{};   // (each worked out once: fifty are not worked out again as a hand moves a handle)

double get easeGripWide => Shape.z(8);

/// The tab at the right edge that draws the shelf out, and at its side puts it away.
class EaseGrip extends StatelessWidget {
  const EaseGrip({super.key, required this.open, required this.onTap});
  final bool open;
  final VoidCallback onTap;
  @override
  Widget build(BuildContext context) => Hov(
    onTap: onTap,
    builder: (_, h) => Container(
      width: easeGripWide,
      height: Shape.z(34),
      decoration: BoxDecoration(color: h ? Grey.g56 : Grey.g38, borderRadius: BorderRadius.horizontal(left: Shape.r(4))),
      child: CustomPaint(painter: EaseChevron(h ? Grey.g95 : Grey.g76, !open)),
    ),
  );
}

/// The shelf: what a hand kept, with an empty frame for the next, and under a line the ways of the maker's folder and the
/// library's (a file put in filament/ways is one more). A press puts a tile on the stretch whole, in place of all that is
/// there, so one after another can be tried (a carried one is laid on what is there, as a shape is). A card let go on it is
/// kept, what all the cards make too (the maker, 10/31: "easeの組み合わせの結果も保存できるように"), and so is what is on
/// the stretch when the empty frame is pressed, as an empty part of Photoshop's Swatches panel takes the colour now. A
/// kept one comes off by its ×.
class EaseShelf extends StatelessWidget {
  const EaseShelf({super.key, required this.now, required this.ways, required this.columns, required this.pitch, required this.side, required this.onPick, required this.onKeep, required this.onForget, required this.inside});
  final String now;   // (the whole way on the stretch, as said)
  final List<Ease> ways;   // (those of the document and the maker's folder)
  final int columns;
  final double pitch, side;
  final Motolii? inside;
  final ValueChanged<Ease> onPick, onForget;
  final ValueChanged<int> onKeep;   // (a card of the flow: its place; all of them: below 0)

  @override
  Widget build(BuildContext context) {
    final kept = easeKept();
    final across = SliverGridDelegateWithFixedCrossAxisCount(crossAxisCount: columns, mainAxisExtent: pitch);
    Widget tile(Ease e, Key key) => PresetTile(e, key: key, on: said(e) == now, side: side, onTap: () => said(e) == now ? null : onPick(e));
    return DragTarget<int>(
      onAcceptWithDetails: (d) => onKeep(d.data),
      builder: (_, hot, _) => DecoratedBox(
        decoration: BoxDecoration(color: WsT.card, border: hot.isEmpty ? Border(left: BorderSide(color: Grey.g38)) : Border.all(color: WsT.accent, width: 2)),
        child: CustomScrollView(
          slivers: [
            SliverGrid.builder(
              gridDelegate: across,
              itemCount: kept.length + 1,
              itemBuilder: (_, i) => i == kept.length
                  ? Hov(key: const ValueKey('ease.shelf.slot'), onTap: () => onKeep(-1), builder: (_, h) => Center(child: EaseSlot(side: side, ink: h ? Grey.g76 : Grey.g38)))
                  : Hov(
                      cursor: MouseCursor.defer,
                      builder: (_, h) {
                        final e = _tiles[kept[i]] ??= easeCard(kept[i], inside);
                        return Stack(
                          children: [
                            Positioned.fill(child: tile(e, ValueKey('ease.kept.${kept[i]}'))),
                            if (h) Positioned(right: 0, top: 0, child: EaseOff(key: ValueKey('ease.forget.${kept[i]}'), onTap: () => onForget(e))),
                          ],
                        );
                      },
                    ),
            ),
            SliverToBoxAdapter(child: Container(height: 1, margin: EdgeInsets.symmetric(vertical: Shape.s(3), horizontal: Shape.s(4)), color: Grey.g38)),
            SliverGrid.builder(gridDelegate: across, itemCount: ways.length, itemBuilder: (_, i) => tile(ways[i], ValueKey('ease.${ways[i].name}'))),
          ],
        ),
      ),
    );
  }
}
