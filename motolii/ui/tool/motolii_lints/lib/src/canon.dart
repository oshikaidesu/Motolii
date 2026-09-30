import 'dimension_use.dart';

/// The canonical token a literal stands for, by its value AND its role (the kind of measure it sits in): a literal of 3 in a gap is
/// `Surface.inlineGap`; the same 3 as a width is nothing canonical and stays a custom dimension. Null when no token has that value
/// and role. This is the one table of the codemod (`bin/check.dart --fix`); the tokens themselves live in lib/theme/metrics.dart.
///
/// [face] is a file of the face family (Browser, Effects, Colors, Fonts): its rows and tiles are face tokens, not the chrome's.
String? canonicalToken(DimensionUse use, double value, {bool face = false}) {
  switch (use.kind) {
    case DimensionKind.font:
      return switch (value) { 11 => 'Dn.nameSize', 10 => 'Dn.labelSize', 9.5 => 'Dn.microSize', _ => null };
    case DimensionKind.radius:
      if (use.name == 'blurRadius' || use.name == 'spreadRadius') return null;
      return switch (value) { 3 => 'Surface.controlRadius', 4.5 => 'Surface.faceRadius', _ => null };
    case DimensionKind.space:
      return switch (value) {
        3 => 'Surface.inlineGap',
        6 => 'Surface.sectionGap',
        9 => 'Surface.panelInset',
        2 when use.vertical == true => 'Surface.labelGap',
        _ => null,
      };
    case DimensionKind.thickness:
      return value == 1.5 ? 'Surface.focusStroke' : null;
    case DimensionKind.extent:
      final tall = const {'height', 'minHeight', 'maxHeight'}.contains(use.name);
      if (tall) {
        if (face && value == 32) return 'Surface.faceRow';
        if (face && value == 40) return 'Surface.faceTile';
        return switch (value) {
          32 => 'Surface.topBar',
          28 => 'Surface.namedHeader',
          24 => 'Surface.chromeRow',
          20 => 'Surface.workRow',
          18 => 'Surface.control',
          _ => null,
        };
      }
      if (use.name == 'size' || use.name == 'dimension') {
        return switch (value) { 10 => 'Surface.glyph', 5 => 'Surface.mark', _ => null };
      }
      return null;
    case DimensionKind.other:
      return null;
  }
}

/// Whether a file belongs to the face family (its own sizes, CLAMPed).
bool faceFile(String path) {
  final unix = path.replaceAll('\\', '/');
  return const ['/lib/browser/', '/lib/effects/', '/lib/colors/', '/lib/fonts/'].any(unix.contains);
}

/// The value the retired `Step.sN` rungs named.
double? stepValue(String name) {
  final m = RegExp(r'^s(\d+)$').firstMatch(name);
  return m == null ? null : double.parse(m.group(1)!);
}
