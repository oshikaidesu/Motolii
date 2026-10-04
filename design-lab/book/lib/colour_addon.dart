// The global 'Colour' control of the lab: one Widgetbook addon, available in every use case (settings panel, "Colour").
// It sets [Role.palette] / [Role.cvd] before the use case builds and remounts the use case when the choice changes (so const subtrees re-read Role).
// The colour-vision simulation is a lens around the use case (Machado 2009, severity 1.0, applied in linear RGB); it is lab chrome, never part of a panel.
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import 'tokens.dart';

enum ColourMode {
  grey('Grey (today)', Palette.grey, Cvd.none),
  a('Palette A', Palette.a, Cvd.none),
  b('Palette B', Palette.b, Cvd.none),
  aDeut('Palette A + deuteranopia sim', Palette.a, Cvd.deut),
  aProt('Palette A + protanopia sim', Palette.a, Cvd.prot),
  aTrit('Palette A + tritanopia sim', Palette.a, Cvd.trit);

  const ColourMode(this.label, this.palette, this.cvd);
  final String label;
  final Palette palette;
  final Cvd cvd;
}

const _machado = {
  Cvd.prot: [0.152286, 1.052583, -0.204868, 0.0, 0.0, 0.114503, 0.786281, 0.099216, 0.0, 0.0, -0.003882, -0.048116, 1.051998, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0],
  Cvd.deut: [0.367322, 0.860646, -0.227968, 0.0, 0.0, 0.280085, 0.672501, 0.047413, 0.0, 0.0, -0.011820, 0.042940, 0.968881, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0],
  Cvd.trit: [1.255528, -0.076749, -0.178779, 0.0, 0.0, -0.078411, 0.930809, 0.147602, 0.0, 0.0, 0.004733, 0.691367, 0.303900, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0],
};

class ColourAddon extends WidgetbookAddon<ColourMode> {
  ColourAddon() : super(name: 'Colour');

  @override
  List<Field> get fields => [
        ObjectDropdownField<ColourMode>(name: 'mode', values: ColourMode.values, initialValue: ColourMode.b, labelBuilder: (m) => m.label),
      ];

  @override
  ColourMode valueFromQueryGroup(Map<String, String> group) => valueOf<ColourMode>('mode', group)!;

  @override
  Widget buildUseCase(BuildContext context, Widget child, ColourMode setting) {
    // No widget listens to these notifiers while the build runs; the key below remounts the use case instead.
    Role.palette.value = setting.palette;
    Role.cvd.value = setting.cvd;
    Widget out = KeyedSubtree(key: ValueKey(setting), child: child);
    final m = _machado[setting.cvd];
    if (m != null) {
      out = ColorFiltered(
        colorFilter: const ColorFilter.linearToSrgbGamma(),
        child: ColorFiltered(colorFilter: ColorFilter.matrix(m), child: ColorFiltered(colorFilter: const ColorFilter.srgbToLinearGamma(), child: out)),
      );
    }
    return out;
  }
}
