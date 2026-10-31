// A way's numbers, said exactly: under the graph its name wears the Inspector's fold mark, and folds them open over the
// graph's foot as a card of it does (the maker, 10/31: "数値の指定", and of a name that gave no sign of it: "わかるかー！").
// Their names are those of AbsentMotion's Keyframes.kt; Alight Motion has them only as handles. Waves and steps are said by
// how many, not by how long each is, which is how they are thought of. Overshoot is here, where it decides how far a number
// goes, not in a menu Alight Motion's way (the maker, 10/31: a number written turned it on unseen, and the curve went out of
// reach): off, Y stops at 0 and 1.
import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart' show Hov;
import 'package:lab_book/sets/inspector/inspector_rows.dart' show FoldMark;
import 'package:lab_book/tokens.dart';
import 'package:lab_book/workspace/desk_ease.dart';
import 'package:lab_book/workspace/desk_ease_cards.dart';
import 'package:lab_book/workspace/desk_kit.dart';
import 'package:lab_book/workspace/ws.dart';

typedef _Number = ({String name, int at, double lo, double hi, bool many});
_Number _n(String name, int at, double lo, double hi, {bool many = false}) => (name: name, at: at, lo: lo, hi: hi, many: many);

// (each kind's numbers as filament/interp.h has them, how far a hand takes each, and what they are when the way says none)
final _kinds = <String, (List<double>, List<_Number>)>{
  'Bezier': (const [0, 0, 1, 1], [_n('X1', 0, 0, 1), _n('Y1', 1, -2, 3), _n('X2', 2, 0, 1), _n('Y2', 3, -2, 3)]),
  'Bounce': (const [.3, .8], [_n('Length', 0, .02, 1), _n('Bounce', 1, 0, .95)]),
  'Elastic': (const [.25, 1, .5, 1], [_n('Half', 0, .02, .98), _n('Decay', 2, 0, 1), _n('Strength', 3, 0, 1)]),
  'Cyclic': (const [2 / 7, 0, .5, 0, 0], [_n('Waves', 0, 1, 50, many: true), _n('Sharp', 1, 0, 1), _n('Crest', 2, .02, .98), _n('Floor', 3, 0, 1)]),
  'Random': (const [.5, .5, .5, .5, .5, 0], [_n('Rate', 0, 0, 1), _n('Size', 1, 0, 1), _n('Seed', 2, 0, 1), _n('Fade', 3, 0, 1), _n('Warp', 4, 0, 1), _n('Tilt', 5, 0, 1)]),
  'Steps': (const [.25, 0], [_n('Steps', 0, 1, 50, many: true), _n('Smooth', 1, 0, 1)]),
  'ElasticSteps': (const [.2, .5], [_n('Steps', 0, 1, 20, many: true), _n('Over', 1, 0, 1)]),
};

/// Whether they are open: left so, whatever is pressed, as the Inspector's cards are.
bool easeNumbersOpen = false;

/// Whether a way goes by its kind's usual numbers: a tile then draws what the kind does, and otherwise its own curve.
bool easeIsUsual(String said) {
  final w = said.split(' '), usual = _kinds[w.first]?.$1;
  if (usual == null || w.length == 1 || w.first == 'Bezier') return true;   // (a bezier's tile draws its own numbers already)
  return [for (final (i, x) in w.skip(1).indexed) (double.tryParse(x) ?? 0) - (i < usual.length ? usual[i] : 0)].every((d) => d.abs() < 1e-6);
}

class EaseNumbers extends StatefulWidget {
  const EaseNumbers({super.key, required this.card, required this.onChanged, this.overshoot = false, this.onOvershoot});
  final Ease card;   // (the card in hand, or the whole: a whole has a name, and no numbers)
  final ValueChanged<Ease> onChanged;
  final bool overshoot;   // (a bezier's: Alight Motion's Enable Overshoot)
  final VoidCallback? onOvershoot;
  @override
  State<EaseNumbers> createState() => _EaseNumbersState();
}

class _EaseNumbersState extends State<EaseNumbers> {
  @override
  Widget build(BuildContext context) {
    final word = said(widget.card), laid = easeIsLaid(word), turned = easeIsTurned(word), plain = turned ? easeTurn(word) : word;
    final w = plain.split(' '), bezier = w.first == 'Linear' || w.first == 'Bezier', kind = laid ? null : _kinds[bezier ? 'Bezier' : w.first];
    Widget name(bool hot) => SizedBox(
      height: Shape.z(16),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text((laid ? 'Result' : easeWord(widget.card)).toUpperCase(), maxLines: 1, style: T.micro(hot ? Grey.g95 : Grey.g63).copyWith(letterSpacing: .6)),
          if (kind != null) ...[SizedBox(width: Shape.s(3)), FoldMark(open: easeNumbersOpen, hot: hot)],
        ],
      ),
    );
    if (kind == null) return Align(alignment: Alignment.bottomCenter, child: name(false));
    final said_ = [for (final x in w.skip(1)) double.tryParse(x) ?? 0];
    final now = w.first == 'Linear' ? kind.$1 : [for (final (i, u) in kind.$1.indexed) i < said_.length ? said_[i] : u];
    void change(int at, double to) {
      final next = [...now]..[at] = to, made = bezier ? 'Bezier ${next.join(' ')}' : '${w.first} ${next.join(' ')}';
      widget.onChanged(heard(turned ? easeTurn(made) : made));
    }

    return Align(
      alignment: Alignment.bottomCenter,
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          if (easeNumbersOpen)
            Container(
              color: WsT.card,
              padding: EdgeInsets.all(Shape.s(3)),
              child: LayoutBuilder(
                builder: (_, box) {
                  final each = (box.maxWidth - Shape.s(3)) / (box.maxWidth / Shape.z(82)).floor().clamp(1, kind.$2.length) - Shape.s(3);
                  return Wrap(
                    spacing: Shape.s(3),
                    runSpacing: Shape.s(3),
                    children: [
                      for (final n in kind.$2)
                        if ((n.name.startsWith('Y') && !widget.overshoot ? 0.0 : n.lo, n.name.startsWith('Y') && !widget.overshoot ? 1.0 : n.hi) case (final lo, final hi))
                        SizedBox(
                          width: each,
                          child: DeskScrub(
                            key: ValueKey('ease-number-${n.name}'),
                            label: n.name,
                            value: n.many ? 1 / now[n.at] : now[n.at],
                            min: lo,
                            max: hi,
                            perPx: n.many ? .05 : (hi - lo) / 200,
                            format: (v) => n.many ? v.toStringAsFixed(v == v.roundToDouble() ? 0 : 2) : v.toStringAsFixed(2),
                            onChanged: (v) => change(n.at, n.many ? 1 / v : v),
                          ),
                        ),
                      if (bezier) SizedBox(width: each, child: DeskChip(key: const ValueKey('ease-overshoot'), label: 'Overshoot', on: widget.overshoot, onTap: widget.onOvershoot)),
                    ],
                  );
                },
              ),
            ),
          Hov(key: const ValueKey('ease-numbers'), onTap: () => setState(() => easeNumbersOpen = !easeNumbersOpen), builder: (_, h) => name(h)),
        ],
      ),
    );
  }
}
