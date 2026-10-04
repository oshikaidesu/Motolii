// set: Phenomenon D (time and motion). A parameter turned into a miniature of the phenomenon it means, so an Inspector row feels like a toy.
// Owner insight: the earlier GUI drafts were "a slider drawn in another shape"; here the number becomes a place, a journey, a physical thing.
// Each use case: story caption, the large miniature (320x200), and the same miniature inside a 282 px Inspector row between two plain number rows.
// Values: each miniature owns a [PhDoc] (pheno_d_kit.dart) with real, named parameters (ranges, defaults, units) a later integration can read and write.
import 'package:widgetbook/widgetbook.dart';

import 'pheno_d_ease.dart';
import 'pheno_d_kit.dart';
import 'pheno_d_loop.dart';
import 'pheno_d_shake.dart';

WidgetbookUseCase _uc(String name, String caption, List<PhSpec> specs, PhMini Function(PhDoc) make, double rowH, (String, String) above, (String, String) below) =>
    WidgetbookUseCase(
      name: name,
      builder: (c) => PhStory(caption: caption, specs: specs, make: make, rowH: rowH, above: above, below: below),
    );

WidgetbookComponent phenoDSet() => WidgetbookComponent(name: 'Phenomenon D', useCases: [
      _uc('1 Easing', 'Easing: easing.x1 y1 x2 y2 (cubic-bezier) in one journey. silhouette: a curved path with a ball hopping along an arch', easeSpecs, EaseMini.new, 112,
          ('Duration', '1.20 s'), ('Delay', '0.00 s')),
      _uc('2 Spring', 'Spring: spring.stiffness + spring.damping in one vial. silhouette: a coiled ball hanging in a tube of oil, ringing out beside it', springSpecs, SpringMini.new, 124,
          ('Mass', '1.00'), ('Velocity', '0.0')),
      _uc('3 Wiggle', 'Wiggle: wiggle.freq + wiggle.amp in one thread. silhouette: a pinned thread with a bead riding its wobble', wiggleSpecs, WiggleMini.new, 84,
          ('Seed', '1'), ('Octaves', '1')),
      _uc('4 Loop', 'Loop: loop.mode + loop.length in one track. silhouette: a racetrack bent from film frames with a small runner lapping and a diamond seam', loopSpecs, LoopMini.new, 116,
          ('In point', '0 f'), ('Out point', '120 f')),
      _uc('5 Time remap', 'Time remap: remap.speed in one belt. silhouette: a draped strip of film frames with a hopping ball in each, pushed through a viewfinder', tapeSpecs, TapeMini.new, 104,
          ('Frame rate', '30 fps'), ('Blending', 'Off')),
      _uc('6 Camera shake', 'Camera shake: shake.amount + shake.roughness in one frame. silhouette: a viewfinder with corner brackets over a fixed landscape, with a trail', shakeSpecs, ShakeMini.new, 112,
          ('Zoom', '100 %'), ('Roll', '0.0 deg')),
    ]);
