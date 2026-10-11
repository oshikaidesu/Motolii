// World D: six small worlds of the per-Effect level (proposals 19-24 of research/op1-translation.md): Glass, Opacity / Blend, Fill, Trim paths,
// Mask feather. (20 Transform is skipped: see the note at the bottom.) Each use case shows the world at 320x200 and the same world in a 282 px
// Inspector row between two plain number rows, with the Driven knob (Off | Wiggle | Pulse) that moves the parameters from outside.
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import 'world_d_blend.dart';
import 'world_d_fill.dart';
import 'world_d_glass.dart';
import 'world_d_kit.dart';
import 'world_d_mask.dart';
import 'world_d_trim.dart';

WdDrive _drive(BuildContext c) => c.knobs.object.dropdown<WdDrive>(label: 'Driven', options: WdDrive.values, initialOption: WdDrive.off, labelBuilder: (d) => d.label);

WidgetbookUseCase _uc(String use, String name, String sil, List<WdSpec> specs, WdMini Function(WdDoc) make, double rowH, (String, String) above, (String, String) below) =>
    WidgetbookUseCase(
        name: use,
        builder: (c) => WdStory(name: name, silhouette: sil, specs: specs, make: make, rowH: rowH, above: above, below: below, drive: _drive(c)));

WidgetbookComponent worldDSet() => WidgetbookComponent(name: 'World D', useCases: [
      _uc('19 Glass', 'Glass', 'a wedge of glass bending a beam', glassSpecs, GlassMini.new, 120, ('Position', '640, 360'), ('Opacity', '100 %')),
      _uc('21 Opacity / Blend', 'Opacity / Blend', 'two frosted plates', blendSpecs, BlendMini.new, 110, ('Anchor', '0, 0'), ('Scale', '100 %')),
      _uc('22 Fill', 'Fill', 'strata of pigment on dotted paper', fillSpecs, FillMini.new, 120, ('Stroke', 'none'), ('Corner', '4 px')),
      _uc('23 Trim paths', 'Trim paths', 'a knotted line being drawn', trimSpecs, TrimMini.new, 120, ('Fill', 'on'), ('Dashes', '0')),
      _uc('24 Mask feather', 'Mask feather', 'a soft window in a plane', maskSpecs, MaskMini.new, 110, ('Mask path', '4 pts'), ('Mode', 'Add')),
    ]);

// 20 Transform: not built. Any four grabs on a layer's position / scale / rotation / anchor (even as traces, ghosts or a pendulum) read as the
// familiar gizmo: a body to drag, a ring to turn, handles to pull. It failed the silhouette test on paper before it was drawn.
