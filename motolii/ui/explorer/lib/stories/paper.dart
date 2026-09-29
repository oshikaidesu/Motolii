// Paper sheets: freehand compositions to find a form, beside the product's own panels. Not wired to anything.
import 'package:widgetbook/widgetbook.dart';

import '../paper/broken.dart';
import '../paper/density.dart';
import 'package:flutter/widgets.dart';

import '../paper/library.dart';
import '../paper/media.dart';
import '../paper/models3d.dart';
import '../story.dart';

Widget _faces(double size, {bool named = true}) => ColoredBox(
      color: const Color(0xFF191919),
      child: Wrap(spacing: 6, runSpacing: 8, children: [
        for (final m in models)
          SizedBox(width: size, child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
            SizedBox(width: size, height: size, child: ColoredBox(color: const Color(0xFF202020), child: ModelFace(m))),
            if (named) Padding(padding: const EdgeInsets.only(top: 3), child: Text(m, style: const TextStyle(fontSize: 9.5, color: Color(0xFFD0D0D0), decoration: TextDecoration.none))),
          ])),
      ]),
    );

final paperStories = <Story>[
  Story('Density specimen roles and rhythm', const Scene(''), (_) => densitySpecimen(), width: 780, height: 560),
  for (final name in paperLibrary.keys) Story('Browser library $name', const Scene(''), (_) => PaperLibrary(name), width: 320, height: 900),
  Story('Browser 3D faces 140', const Scene(''), (_) => _faces(140), width: 320, height: 400),
  Story('Browser 3D faces 68', const Scene(''), (_) => _faces(68), width: 320, height: 300),
  for (final name in paperBroken.keys) Story('Browser broken Media $name', const Scene(''), (_) => PaperBroken(name), width: 320, height: 900),
  for (final name in paperMedia.keys) Story('Browser paper Media $name', const Scene(''), (_) => PaperMedia(name), width: 320, height: 900),
];

final paperComponent = WidgetbookComponent(name: 'Paper', useCases: [for (final s in paperStories) useCase(s)]);
