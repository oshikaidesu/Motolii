// A particle layer (not an effect): one card, its thirty handles grouped by what they shape; the rarely used groups fold away.
import 'inspector_studies.dart';
import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_param_controls_tile.dart';
import 'package:lab_book/sets/inspector/inspector_rows_sect.dart';

/// A particle layer (not an effect): one card, its thirty handles grouped by what they shape; the rarely used groups fold away.
/// contract: a group's dependent handles (Floor, Scale, the line pair) show only while the handle that turns them on is non-zero.
class ParticlesCard extends StatelessWidget {
  const ParticlesCard({super.key});
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc;
    doc.seed({for (final p in ptAll) ...paramDefaults('pt', p)}, strs: const {'pt_color': '#FFFFFF', 'pt_color_end': '#FFFFFF'});
    // contract: re-provides Ctx on every doc change, so the card's const blocks and pictures follow even under a host that does not rebuild
    return ListenableBuilder(
      listenable: doc,
      builder: (context, _) => Ctx(cfg: x.cfg, doc: doc, child: _body(doc)),
    );
  }

  Widget _body(Doc doc) {
    double g(String id) => doc.get('pt_$id') ?? 0;
    final turb = doc.b['pt_fold.turb'] ?? false, links = doc.b['pt_fold.links'] ?? false;
    return Sect(
      title: 'Particles',
      tone: Pop.toneParticles,
      mark: CardMark.particles,
      keyIds: [for (final p in ptAll) 'pt_${p.id}'],
      brief: ['${brief(g('rate'), 0)}/s', '${g('life').toStringAsFixed(1)} s', '${brief(g('direction'), 0)}°'],
      children: [
        const PtPreview(),
        const PtHead('Emit'),
        const ParamGrid('pt', ptEmit),
        const PtHead('Motion'),
        Padding(
          padding: EdgeInsets.only(top: 4),
          child: Row(
            children: [
              Expanded(child: PtAngle(PtAngle.dir)),
              SizedBox(width: Pop.tileGap),
              Expanded(child: PtAngle(PtAngle.spread)),
              SizedBox(width: Pop.tileGap),
              Expanded(child: ParamTile('pt', ptSpeed)),
            ],
          ),
        ),
        const PtHead('Over life'),
        const PtRamp(),
        const ParamGrid('pt', ptLife),
        SizedBox(height: Pop.tileGap),
        Row(
          children: [
            Expanded(child: PtColour('pt_color', 'Color')),
            SizedBox(width: Pop.tileGap),
            Expanded(child: PtColour('pt_color_end', 'End Color')),
          ],
        ),
        const PtHead('Forces'),
        ParamGrid('pt', [ptGravity, ptWindX, ptWindY, ptBounce, if (g('bounce') > 0) ptFloor]),
        const PtHead('Random'),
        const ParamGrid('pt', ptRandom),
        const SizedBox(height: 4),
        PtHead(
          'Turbulence',
          open: turb,
          summary: g('turbulence') > 0 ? '${brief(g('turbulence'), 0)} px' : 'Off',
          onTap: () => doc.flag('pt_fold.turb', !turb),
        ),
        if (turb) ParamGrid('pt', [ptTurb, if (g('turbulence') > 0) ptTurbSize]),
        PtHead('Links', open: links, summary: g('connect') > 0 ? '${brief(g('connect'), 0)} px' : 'Off', onTap: () => doc.flag('pt_fold.links', !links)),
        if (links) ParamGrid('pt', [ptConnect, if (g('connect') > 0) ...ptLines]),
        AdvancedFold('pt.adv', summary: 'Seed ${brief(g('seed'), 0)}', const [
          ParamGrid('pt', [ptSeed]),
        ]),
      ],
    );
  }
}
