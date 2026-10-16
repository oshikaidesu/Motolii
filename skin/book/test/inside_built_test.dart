// The checks of a real document skip where the inside is not built: this one says so in red, so a green run is not a run
// that checked nothing (build it: filament/edit.sh).
import 'package:flutter_test/flutter_test.dart';

import 'door.dart';

void main() {
  test('the inside is built, so the checks of a real document run', () {
    expect(insideBuilt, isTrue, reason: 'no $motoliiDir/out/libmotolii.dylib: build it with filament/edit.sh; the joined checks were all skipped');
  });
}
