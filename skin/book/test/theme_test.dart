// The colour theme (theme.dart, Ableton's .ask manner): a file read back as it was written; a role it says is the
// window's colour, one it does not say is the window's own. And the window's own two themes written out as files
// (themes/Default Dark.xml, Default Light.xml), to be copied and changed.
import 'dart:io';
import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/theme.dart';
import 'package:lab_book/tokens.dart';
import 'package:lab_book/workspace/ws.dart' show WsT;

Map<String, Color> _own() => {
  'Grey00': Grey.g00, 'Grey07': Grey.g07, 'Grey10': Grey.g10, 'Grey13': Grey.g13, 'Grey15': Grey.g15, 'Grey20': Grey.g20,
  'Grey26': Grey.g26, 'Grey38': Grey.g38, 'Grey44': Grey.g44, 'Grey56': Grey.g56, 'Grey63': Grey.g63, 'Grey76': Grey.g76,
  'Grey91': Grey.g91, 'Grey95': Grey.g95, 'Grey100': Grey.g100, 'BandA': Grey.bandA, 'BandB': Grey.bandB, 'RowLine': Grey.rowLine,
  'Glaze9': Grey.glaze9, 'Glaze15': Grey.glaze15,
  'ChosenDefault': WsT.accent, 'ControlOnForeground': WsT.onAccent, 'Keyframe': WsT.keyDot,
  'Clip1': WsT.toneShape, 'Clip2': WsT.toneText, 'Clip3': WsT.toneImage, 'Clip4': WsT.toneCamera, 'Clip5': WsT.toneGroup, 'Clip6': WsT.toneParticles, 'Clip7': WsT.toneAudio,
};

void main() {
  test('a theme file read back as written; what it does not say is the window\'s own', () {
    ColorTheme.roles.value = const {};
    final usual = WsT.accent, file = ColorTheme.write({'ChosenDefault': const Color(0xFF3366FF), 'Grey07': const Color(0x80101010)});
    ColorTheme.roles.value = ColorTheme.parse(file);
    expect(WsT.accent, const Color(0xFF3366FF));
    expect(Grey.g07, const Color(0x80101010));
    expect(Grey.g13, N.g13);   // (not said: the window's own)
    ColorTheme.roles.value = const {};
    expect(WsT.accent, usual);
  });
  test('the window\'s own themes, written out', () {
    ColorTheme.roles.value = const {};
    final dir = Directory('themes')..createSync();
    for (final (name, shade) in [('Default Dark', Shade.dark), ('Default Light', Shade.light)]) {
      Grey.shade.value = shade;
      final own = _own(), file = File('${dir.path}/$name.xml')..writeAsStringSync(ColorTheme.write(own));
      expect(ColorTheme.parse(file.readAsStringSync()), own);   // (read back, every role the same)
    }
    Grey.shade.value = Shade.dark;
  });
}
