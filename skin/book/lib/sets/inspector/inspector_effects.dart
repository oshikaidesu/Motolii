import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_effects_effect_card.dart';
import 'package:lab_book/sets/inspector/inspector_rows_sect.dart';

import '../../tokens.dart';
import 'inspector_parts.dart';

// ---- 10. effects ------------------------------------------------------------------------------------------------------
/// An effect as a plugin ships it: a name and a WGSL uniform struct. Its rows are whatever [readParams] makes of the struct.
class Fx {
  Fx(this.id, this.name, this.wgsl, {required this.tone, required this.mark, this.vendor});
  final String id, name, wgsl;
  final String? vendor;
  final Color tone;
  final CardMark mark;
  late final List<Decl> decls = parseWgsl(wgsl);
  late final List<Param> params = [for (final d in decls) infer(d)];
}


class EffectsStack extends StatelessWidget {
  const EffectsStack({super.key});
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc;
    final cards = [
      for (var i = 0; i < doc.fxOrder.length; i++) EffectCard(doc.fxDefs.firstWhere((f) => f.id == doc.fxOrder[i]), key: ValueKey(doc.fxOrder[i]), first: i == 0),
    ];
    final none =
        doc.query.isNotEmpty &&
        !doc.fxOrder.any((id) => doc.fxDefs.firstWhere((f) => f.id == id).params.any((p) => p.label.toLowerCase().contains(doc.query.toLowerCase())));
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        if (x.cfg.maybe) const ParamSearch(),
        if (none)
          Padding(
            padding: const EdgeInsets.symmetric(vertical: 8),
            child: Text('No parameter matches', style: T.label(Grey.g56)),
          )
        else
          ...cards,
      ],
    );
  }
}

/// H3 (maybe): search flattens the groups, unfolds Advanced, and says so when nothing matches.
class ParamSearch extends StatefulWidget {
  const ParamSearch({super.key});
  @override
  State<ParamSearch> createState() => _ParamSearchState();
}

class _ParamSearchState extends State<ParamSearch> {
  final _c = TextEditingController();
  final _f = FocusNode(debugLabel: 'param-search');
  @override
  void dispose() {
    _c.dispose();
    _f.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), ed = Ed(x.cfg.ed);
    return Container(
      height: cellH,
      margin: const EdgeInsets.only(bottom: 4),
      padding: const EdgeInsets.symmetric(horizontal: 8),
      alignment: Alignment.centerLeft,
      decoration: BoxDecoration(
        color: Grey.g13,
        borderRadius: BorderRadius.circular(4),
        border: Border.all(color: ed.rule),
      ),
      child: Stack(
        alignment: Alignment.centerLeft,
        children: [
          if (_c.text.isEmpty) Text('Search parameters', style: T.label(Grey.g56)),
          EditableText(
            controller: _c,
            focusNode: _f,
            style: T.name(Grey.g95),
            cursorColor: Grey.g95,
            backgroundCursorColor: Grey.g20,
            maxLines: 1,
            onChanged: (s) {
              x.doc.query = s;
              x.doc.poke();
              setState(() {});
            },
          ),
        ],
      ),
    );
  }
}

/// A card of the host's own values, in the effects' manner: its Params, each field a value (keyed `v_` and its name).
class ValuesCard extends StatelessWidget {
  const ValuesCard(this.card, {super.key});
  final Fx card;
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, ed = Ed(x.cfg.ed), k = 'v.${card.name}', adv = doc.b['$k.adv'] ?? false;
    final heroes = [for (final p in card.params) if (!p.advanced) p], rest = [for (final p in card.params) if (p.advanced) p];
    // (the first row, then the rest folded under the name of what they are; the fold says nothing more than its glyph)
    return Sect(
      title: card.name,
      tone: card.tone,
      mark: card.mark,
      children: [
        ParamGrid('v', heroes),
        if (rest.isNotEmpty) ...[
          Padding(
            padding: const EdgeInsets.only(top: 4),
            child: Hov(
              onTap: () => doc.flag('$k.adv', !adv),
              builder: (_, h) => Container(
                height: 24,
                decoration: BoxDecoration(border: Border(top: BorderSide(color: ed.rule))),
                child: Row(
                  children: [
                    Text(rest.first.fold ?? 'Advanced', style: ed.labStyle(h || adv ? Grey.g95 : Grey.g63)),
                    const Spacer(),
                    FoldMark(open: adv, hot: h || adv),
                  ],
                ),
              ),
            ),
          ),
          if (adv) ParamGrid('v', rest),
        ],
      ],
    );
  }
}
