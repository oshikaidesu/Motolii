// The window's letters as the Mac draws them in a test, not Flutter's test blocks: each family the window names read
// from the Mac's own font file under that name, as golden_toolkit's loadAppFonts gives a test an app's fonts (Flutter's
// FontLoader); a family the Mac has not, the face it falls back to (San Francisco). A picture a test takes reads.
import 'dart:io';
import 'dart:typed_data';

import 'package:flutter/services.dart';

const _sf = '/System/Library/Fonts/SFNS.ttf', _sup = '/System/Library/Fonts/Supplemental';
const _files = {
  'Inter': _sf, '.AppleSystemUIFont': _sf, 'Roboto': _sf,   // (not on the Mac: the app shows its fallback, San Francisco)
  'Menlo': '/System/Library/Fonts/Menlo.ttc',
  'Helvetica Neue': '/System/Library/Fonts/HelveticaNeue.ttc',
  'Courier': '/System/Library/Fonts/Courier.ttc',
  'Avenir Next': '/System/Library/Fonts/Avenir Next.ttc',
  'Arial': '$_sup/Arial.ttf',
  'Arial Black': '$_sup/Arial Black.ttf',
  'Futura': '$_sup/Futura.ttc',
};

Future<void> loadMacFonts() async {
  for (final MapEntry(key: family, value: path) in _files.entries) {
    final f = File(path);
    if (!f.existsSync()) continue;   // (a Mac without it: that family stays the test's blocks)
    await (FontLoader(family)..addFont(Future.value(ByteData.sublistView(f.readAsBytesSync())))).load();
  }
}
