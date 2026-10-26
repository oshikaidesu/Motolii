// The ways under the sample's eight (the document's own, the maker's presets, the kinds the sample has no seat for):
// they scroll sideways, a table picked as the font shelf picks one (All or ★). A table's order never moves (where a
// tile is stays where the hand learnt it); the last used come first, set apart, as Photoshop's and Figma's font menus
// keep the recent above the list; the starred are a table of their own, as Ableton keeps Collections. Both are kept in
// ~/.motolii/ease_stars.json, as the maker's colours are in colors.json.
import 'dart:convert';
import 'dart:io';

import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart' show Hov;
import 'package:lab_book/tokens.dart';
import 'package:lab_book/workspace/desk_ease.dart';
import 'package:lab_book/workspace/desk_ease_presets.dart';
import 'package:lab_book/workspace/dock_parts.dart' show DockChips, DockStar;
import 'package:lab_book/workspace/ws.dart';

/// Where the starred and the recent are kept (a test points it elsewhere, and forgets what was read: [easeStarsAt]).
File _starsFile = File('${Platform.environment['HOME']}/.motolii/ease_stars.json');
List<String>? _stars, _recent;
void easeStarsAt(File f) => (_starsFile = f, _stars = null, _recent = null);
List<String> _read(String key) {
  try {
    return [for (final s in ((jsonDecode(_starsFile.readAsStringSync()) as Map)[key] as List? ?? const [])) '$s'];
  } catch (_) {
    return <String>[];
  }
}

void _keep() {
  try {
    _starsFile.parent.createSync(recursive: true);
    _starsFile.writeAsStringSync(jsonEncode({'stars': easeStars(), 'recent': easeRecent()}));
  } catch (_) {}   // (not kept: known while the window is open)
}

List<String> easeStars() => _stars ??= _read('stars');
List<String> easeRecent() => _recent ??= _read('recent');
void easeStar(String name) {
  final s = easeStars();
  s.contains(name) ? s.remove(name) : s.add(name);
  _keep();
}

/// A way put on a stretch: the first of the recent (four kept, as a font menu's recent are a few).
void easeUsed(String name) {
  easeRecent()
    ..remove(name)
    ..insert(0, name);
  if (_recent!.length > 4) _recent!.removeRange(4, _recent!.length);
  _keep();
}

const easeTables = ['All', '★'];

class EaseLibrary extends StatefulWidget {
  const EaseLibrary({super.key, required this.ease, required this.more, required this.onPick});
  final Ease ease;
  final List<Ease> more;
  final ValueChanged<Ease> onPick;
  @override
  State<EaseLibrary> createState() => _EaseLibraryState();
}

String _table = 'All';   // (the table last picked: the desk opened again shows it)

class _EaseLibraryState extends State<EaseLibrary> {
  /// The tiles of a table, in its own order (the starred: in the order starred).
  List<Ease> tiles(String table) => table == '★' ? [for (final s in easeStars()) ...widget.more.where((e) => e.name == s)] : widget.more;

  @override
  Widget build(BuildContext context) {
    final shown = tiles(_table), lately = [for (final r in easeRecent()) ...widget.more.where((e) => e.name == r)];
    final row = [...lately, ...shown], now = widget.ease.name;
    return Container(
      color: WsT.card,
      padding: EdgeInsets.fromLTRB(Shape.s(5), Shape.s(3), Shape.s(5), Shape.s(5)),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          DockChips(key: const ValueKey('ease.tables'), [for (final t in easeTables) (t, tiles(t).length)], active: _table, onPick: (t) => setState(() => _table = t)),
          SizedBox(height: Shape.s(4)),
          SizedBox(
            height: Shape.z(50),
            child: row.isEmpty
                ? Center(child: Text(_table == '★' ? 'Star a way to keep it here' : 'Nothing here', style: T.micro(Grey.g56)))
                : ListView.separated(
                    key: ValueKey('ease.library.$_table'),
                    scrollDirection: Axis.horizontal,
                    itemCount: row.length,
                    separatorBuilder: (_, i) => i == lately.length - 1
                        ? Container(key: const ValueKey('ease.recent.end'), width: 1, margin: EdgeInsets.symmetric(horizontal: Shape.s(4)), color: Grey.g38)   // (the recent apart from the table)
                        : SizedBox(width: Shape.s(3)),
                    itemBuilder: (_, i) {
                      final e = row[i], on = now == e.name || (now?.startsWith('${e.name} ') ?? false), starred = easeStars().contains(e.name);
                      return SizedBox(
                        width: Shape.z(58),
                        child: Hov(
                          cursor: MouseCursor.defer,
                          builder: (_, h) => Stack(
                            children: [
                              Positioned.fill(
                                child: PresetTile(e, key: ValueKey(i < lately.length ? 'ease.recent.${e.name}' : 'ease.${e.name}'), on: on, onTap: () {
                                  if (on) return;
                                  easeUsed(e.name!);
                                  widget.onPick(e);
                                }),
                              ),
                              if (starred || h)   // (the star shown only where it is on, or under the hand: as the shelves show theirs)
                                Positioned(right: 0, top: 0, child: DockStar(key: ValueKey('ease.star.${e.name}'), on: starred, onTap: () => setState(() => easeStar(e.name!)))),
                            ],
                          ),
                        ),
                      );
                    },
                  ),
          ),
        ],
      ),
    );
  }
}
