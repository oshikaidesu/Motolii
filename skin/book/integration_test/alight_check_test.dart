// reports/alight_checklist.md, a section at a time: each row's bundled effect put on the left of two photos in the
// window, a knob set where it is plainly seen (else as it comes: effects/first.tsv; a transition half way), the stage
// shot; the right photo is the same picture without it (the effect is the layer's, not the stage's). Pictures in a
// temporary folder, its path printed: ALIGHT=<folder>.
//   flutter drive --profile -d macos --driver=test_driver/integration_test.dart --target=integration_test/alight_check_test.dart --dart-define=SECTION=D
import 'dart:io';
import 'dart:ui' show ImageByteFormat;

import 'package:flutter/rendering.dart' show OffsetLayer;
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:lab_book/motolii.dart' show motoliiDir;
import 'package:lab_book/theme.dart';
import 'package:lab_book/workspace/timeline.dart';
import 'package:lab_book/workspace/workspace.dart' show Workspace;
import 'package:lab_book/workspace/ws.dart';
import 'package:lab_book/workspace/zoom.dart';

// (the row, the bundled effect the checklist names, knobs set as the document names them: a thing's script knob,
//  doc_values.h doc_screen_knob; a colour's parts _x _y _z)
const _section = String.fromEnvironment('SECTION', defaultValue: 'A');
const _rowsA = <(String, String, Map<String, Object>)>[
  ('Brightness / Contrast', 'Color Controls', {'bright': .35}),
  ('Color Temperature', 'White Point Adjust', {'newWhite_x': 1, 'newWhite_y': .85, 'newWhite_z': .6}),
  ('Color Tune', 'LGG', {'gain_z': .45}),
  ('Exposure / Gamma', 'Exposure Adjust', {'inputEV': 1.5}),
  ('Highlights and Shadows', 'Color Levels', {'minLevel': .35, 'maxLevel': .65}),
  ('Lens Flare', 'Lens Flare', {}),
  ('Light Glow', 'Bloom', {'intensity': .7}),
  ('Saturation / Vibrance', 'Vibrance', {'vibrance': .6}),
];
// (section D: the ISF-Files effect the checklist names, as it comes; a transition (progress) half way)
const _rowsD = <(String, String, Map<String, Object>)>[
  ('Block Dissolve', 'Random Squares Mask', {}),
  ('Block Noise', 'JPEG Block Corruption', {}),
  ('Box Blur', 'Fast Blur', {}),
  ('Bump Map', 'Emboss', {}),
  ('Channel Remap (HSV)', 'RGBtoHSV', {}),
  ('Channel Remap (HSV)', 'HSVtoRGB', {}),
  ('Channel Remap (RGB)', 'RGBA Swap', {}),
  ('Checker', 'Checkerboard', {}),
  ('Chroma Key', 'Chroma Mask', {}),
  ('Circular Ripple', 'Ripples', {}),
  ('Clouds', 'Simplex Noise', {}),
  ('Curl', 'Inverted Page Curl', {'progress': .5}),
  ('Directional Blur', 'Linear Blur', {'progress': .5}),
  ('Displacement Map', 'Displace', {}),
  ('Dissolve', 'Noise', {}),
  ('Dots', 'Polka Dots Curtain', {'progress': .5}),
  ('Edge Glow', 'Edges', {}),
  ('Fade In/Out', 'Fade', {'progress': .5}),
  ('Find Edges', 'Edges', {}),
  ('Flicker', 'RGB Strobe', {}),
  ('Four-color Gradient', 'Corner Colors', {}),
  ('Fractal Ridges', 'Ridgelines', {}),
  ('Glass', 'Frosted Glass', {}),
  ('Gradient Map', 'Duotone', {}),
  ('Grid', 'Graph Paper', {}),
  ('Halftone Lines', 'Line Screen', {}),
  ('Hexagon Tiling', 'Hexagonalize', {'progress': .5}),
  ('Hot Color', 'Thermal Camera', {}),
  ('Hue Shift', 'Multi Hue Shift', {}),
  ('Invert', 'Color Invert', {}),
  ('Invert', 'RGB Invert', {}),
  ('Matte Choker', 'Erode', {}),
  ('Matte Choker', 'Dilate', {}),
  ('Mosaic', 'Mosaic', {'progress': .5}),
  ('Motion Blur', 'VVMotionBlur 3.0', {}),
  ('Offset', 'Side Scroller And Flip', {}),
  ('Palette Map', 'Color Schemes', {}),
  ('Pixelate', 'Pixellate', {}),
  ('Radial Wipe', 'Radial', {'progress': .5}),
  ('Replace Color', 'Color Replacement', {}),
  ('Sharpen', 'Sharpen RGB', {}),
  ('Simple Starfield', 'Star', {}),
  ('Soft Glow', 'Glow', {}),
  ('Spectral Map', 'Color Scales', {}),
  ('Spin', 'Rotate', {}),
  ('Spot Color', 'Chroma Desaturation Mask', {}),
  ('Star', 'Star', {}),
  ('Stripes', 'Random Stripes', {}),
  ('Text Randomizer', 'Random Characters', {}),
  ('Threshold', 'Color Monochrome', {}),
  ('Tile Rotate', 'Grid Flip', {'progress': .5}),
  ('Tiles', 'Quad Tile', {}),
  ('Timecode', 'Digital Clock', {}),
  ('Turbulence', 'Simplex Noise', {}),
  ('Unsharp Mask', 'Unsharp Mask', {}),
  ('Voronoi Cells', 'Worley Cells', {}),
  ('Wipe', 'Wipe Left', {'progress': .5}),
  ('Zoom Blur', 'Zoom', {}),
];
// (section D again, the rows that show nothing as they come: a knob set as the sentence says; the sample's sky is
//  about 0.55 0.75 0.95, its balloon's red about 0.85 0.1 0.2)
const _rowsD2 = <(String, String, Map<String, Object>)>[
  ('Channel Remap (RGB)', 'RGBA Swap', {'redInput': 2, 'blueInput': 0}),
  ('Chroma Key', 'Chroma Mask', {'mask_color_x': .55, 'mask_color_y': .75, 'mask_color_z': .95, 'hueTol': .15, 'satTol': .4, 'valTol': .4}),
  ('Replace Color', 'Color Replacement', {'mask_color_x': .55, 'mask_color_y': .75, 'mask_color_z': .95, 'target_color_x': .9, 'target_color_y': .2, 'target_color_z': .2, 'hueTol': .15, 'satTol': .4, 'valTol': .4}),
  ('Spot Color', 'Chroma Desaturation Mask', {'mask_color_x': .85, 'mask_color_y': .1, 'mask_color_z': .2, 'hueTol': .1, 'satTol': .5, 'valTol': .5}),
  ('Sharpen', 'Sharpen RGB', {'intensityR': 5, 'intensityG': 5, 'intensityB': 5}),
  ('Unsharp Mask', 'Unsharp Mask', {'intensity': 5}),
];
// (section B: what only shapes bent, on a photo)
const _rowsB = <(String, String, Map<String, Object>)>[
  ('Fractal Warp', 'Noise Displace', {'displaceX': .05, 'displaceY': .05}),
  ('Pinch/Bulge', 'Bump Distortion', {'level': .8, 'radius': .4}),
  ('Turbulent Displace', 'Noise Displace', {'displaceX': .05}),
  ('Wave Warp', 'Sine Warp Tile', {}),
];
// (section E: the rows reports/alight_sources_E.md finds a bundled ISF-Files file for, put on as it says)
const _rowsE = <(String, String, Map<String, Object>)>[
  ('Bend', 'Circle Wrap Distortion', {}),
  ('Dark Glow', 'Gloom', {'intensity': .6}),
  ('Hexagon Array', 'Hexagonalize', {'progress': .5}),
  ('Iridescence', 'Sine Warp Gradient', {}),
];
// (section C: the thing's picture on a solid's faces, the solid named in its fx (filament/solids/*.js, as Alight
//  Motion's faux-3D); the thing turned a little (its own values, '@') so its sides are seen)
const _turned = <String, Object>{'@rotate_x': 25, '@rotate_y': 35};
const _rowsC = <(String, String, Map<String, Object>)>[
  ('Box', 'solids/Box.js', _turned), ('Cube', 'solids/Cube.js', _turned), ('Cylinder', 'solids/Cylinder.js', _turned),
  ('Ellipsoid', 'solids/Ellipsoid.js', _turned), ('Heart', 'solids/Heart.js', _turned), ('Hexagonal Prism', 'solids/Hexagonal_Prism.js', _turned),
  ('Hollow Box', 'solids/Hollow_Box.js', _turned), ('Octahedron', 'solids/Octahedron.js', _turned), ('Pyramid', 'solids/Pyramid.js', _turned),
  ('Raster Extrude', 'solids/Raster_Extrude.js', _turned), ('Spherize', 'solids/Spherize.js', _turned), ('Star Prism', 'solids/Star_Prism.js', _turned),
  ('Three-axis Cross', 'solids/Three-axis_Cross.js', _turned), ('Torus', 'solids/Torus.js', _turned), ('Tunnel', 'solids/Tunnel.js', _turned),
];
// (section E built of what a thing has already (reports/alight_sources_E.md: its copies, AE's repeater), lightened
//  over each other; no effect file: '-')
const _rowsE2 = <(String, String, Map<String, Object>)>[
  ('Linear Streaks', '-', {'@copies': 12, '@copy_x': .03, '@copy_y': -.015, '@blend': 'lighten'}),
  ('Spin Streaks', '-', {'@copies': 12, '@copy_rotate': 3, '@blend': 'lighten'}),
  ('Zoom Streaks', '-', {'@copies': 12, '@copy_size': 1.03, '@blend': 'lighten'}),
];
// (section F, 有る: the thing's own values the checklist names, on a photo; no effect file: '-')
const _rowsF = <(String, String, Map<String, Object>)>[
  ('Colorize', '-', {'@tint': .8, '@tint_black': '000000', '@tint_white': 'ff8800'}),
  ('Flip Layer', '-', {'@rotate_y': 180}),
  ('Gaussian Blur', '-', {'@blur': 20}),
  ('Glow', '-', {'@glow': 'ffffff', '@glow_blur': 30, '@glow_opacity': 1}),
  ('Grid Repeat', '-', {'@size': .4, '@copies': 4, '@copy_columns': 2, '@copy_x': .45, '@copy_y': .45}),   // (a photo is 1 wide: made small, the copies a little further apart than it)
  ('Linear Repeat', '-', {'@copies': 4, '@copy_x': .15, '@copy_y': .05}),
  ('Repeat', '-', {'@copies': 3, '@copy_x': .2}),
  ('Scatter Repeat', '-', {'@copies': 6, '@scatter_x': .3, '@scatter_y': .3}),
  ('Solid Color', '-', {'@tint': 1, '@tint_black': '3366ff', '@tint_white': '3366ff'}),
];
// (section AM: AbsentMotion's effects that read the picture blurred, made by effects/from_absent.py's two blur passes)
final _rowsAM = <(String, String, Map<String, Object>)>[
  for (final n in const ['contourgradient', 'contourlines', 'darkglow', 'edgeglow', 'electricedges', 'feather', 'fillbehind', 'glowscan', 'innerblur', 'innerglow', 'lightglow', 'mattechoker', 'roughenedges', 'smoothbevel', 'smoothedges', 'softglow'])
    (n, 'effects/am_$n.wgsl', const {}),
];
// (section S: AbsentMotion effects already on the shelf, drawn at the defaults in each file's head)
final _rowsS = <(String, String, Map<String, Object>)>[
  for (final n in const ['threshold', 'mosaic', 'grid', 'dissolve', 'dots', 'palettemap'])
    (n, 'effects/am_$n.wgsl', const {}),
];
// (section S2: the next shelf effects whose head defaults are not the unchanged picture)
final _rowsS2 = <(String, String, Map<String, Object>)>[
  for (final n in const ['wavewarp', 'directionalblur', 'gradientmap', 'radialblur', 'hexagontiling', 'flicker'])
    (n, 'effects/am_$n.wgsl', const {}),
];
// (section S3: more shelf effects whose head defaults are not the unchanged picture)
final _rowsS3 = <(String, String, Map<String, Object>)>[
  for (final n in const ['gradientoverlay', 'kaleidoscope', 'posterize', 'rgbsplit', 'noise', 'vignette', 'cmykhalftone', 'iridescence'])
    (n, 'effects/am_$n.wgsl', const {}),
];
// (section S4: more shelf effects whose head defaults are not the unchanged picture)
final _rowsS4 = <(String, String, Map<String, Object>)>[
  for (final n in const ['innerbulge', 'lightning', 'lumakey', 'radialrays', 'twirl', 'tilerotate', 'mirror', 'zoomstreaks'])
    (n, 'effects/am_$n.wgsl', const {}),
];
// (section S5: shelf effects still open, head defaults that are not the unchanged picture)
final _rowsS5 = <(String, String, Map<String, Object>)>[
  for (final n in const ['spotcolor', 'simplestarfield', 'contourstrips', 'ominodiffusion', 'ribbon', 'stretch', 'lensblur', 'chromakey'])
    (n, 'effects/am_$n.wgsl', const {}),
];
// (section W: the open rows, shelf or not. A head default of "unchanged" is set to a value that file already declares. The last four are the effects those rows already name, not another am_ file.)
final _rowsW = <(String, String, Map<String, Object>)>[
  for (final n in const ['spotcolor', 'simplestarfield', 'contourstrips', 'ominodiffusion', 'ribbon', 'stretch', 'lensblur', 'chromakey', 'polar'])
    (n, 'effects/am_$n.wgsl', const {}),
  ('squeeze', 'effects/am_squeeze.wgsl', const {'amount': 1.2}),
  ('offset', 'effects/am_offset.wgsl', const {'offset_x': 220}),
  ('wipe', 'effects/am_wipe.wgsl', const {'end': .4}),
  ('stretchaxis', 'effects/am_stretchaxis.wgsl', const {'scale': 2.2}),
  ('halftone', 'effects/halftone.wgsl', const {}),
  ('fine_grain', 'effects/fine_grain.wgsl', const {'amount': .8}),
  ('vignette3', 'effects/vignette.wgsl', const {}),
  ('rays', 'God Rays', const {}),
];
// (section X: open rows whose shelf file has not been shot. A head default of "unchanged" is a value that file already declares.)
final _rowsX = <(String, String, Map<String, Object>)>[
  ('highlightshadow', 'effects/am_highlightshadow.wgsl', const {'highlights': .15, 'shadows': 1.8}),
  ('unsharpmask', 'effects/am_unsharpmask.wgsl', const {'radius': 1.5, 'amount': 2}),
  ('longshadow', 'effects/am_longshadow.wgsl', const {}),
  ('radialshadow', 'effects/am_radialshadow.wgsl', const {'reach': 2, 'alpha': .9}),
  ('starfield', 'effects/am_starfield.wgsl', const {'brightness': 1.5}),
  ('starpolyhedron', 'effects/am_starpolyhedron.wgsl', const {}),
  ('rasterextrude', 'effects/am_rasterextrude.wgsl', const {}),
  ('tileshift', 'effects/am_tileshift.wgsl', const {'strength': .8}),
];
// (section Y: open rows not yet shot. Hexagon tiles use a value the file already declares, because the head default does not turn or slide them.)
final _rowsY = <(String, String, Map<String, Object>)>[
  ('hexrot', 'effects/am_hexagontilerotate.wgsl', const {'random': 1}),
  ('hexshift', 'effects/am_hexagontileshift.wgsl', const {'strength': .8}),
  ('viewer360', 'effects/am_viewer360.wgsl', const {}),
  ('reorient360', 'effects/am_reorient360.wgsl', const {}),
  ('thermal', 'Thermal Camera', const {}),
  ('falsecolor', 'False Color', const {}),
  ('scales', 'Color Scales', const {'iAuthor': 1}),
  ('motionblur', 'VVMotionBlur 3.0', const {}),
];
// (section Z: open rows shot again at a value the file already declares, or with the ISF the row names.)
final _rowsZ = <(String, String, Map<String, Object>)>[
  ('wipe', 'effects/am_wipe.wgsl', const {'start': .2, 'end': .75}),
  ('stretch', 'effects/am_stretch.wgsl', const {'stretch': 400}),
  ('lensblur', 'effects/am_lensblur.wgsl', const {'radius': 1.5}),
  ('star', 'effects/am_simplestarfield.wgsl', const {'starSize': 3}),
  ('omino', 'effects/am_ominodiffusion.wgsl', const {'weight': 8}),
  ('graph', 'Graph Paper', const {}),
  ('dotscreen', 'effects/dot_screen.wgsl', const {}),
  ('sine', 'Sine Warp Tile', const {'angle': .35}),
];
// (section P: the ISF the open row names, or a value that file already declares when the head default is the unchanged picture.)
final _rowsP = <(String, String, Map<String, Object>)>[
  ('frost', 'Frosted Glass', const {'magnitude': .1}),
  ('displace', 'Displace', const {'uDisplaceAmt': 1}),
  ('chromamask', 'Chroma Mask', const {}),
  ('desat', 'Chroma Desaturation Mask', const {}),
  ('gridflip', 'Grid Flip', const {'progress': .5}),
  ('echo', 'Echo Trace', const {}),
  ('rotate', 'Rotate', const {'angle': .25}),
  ('chromakey', 'effects/am_chromakey.wgsl', const {'tolerance': .7}),
];
// (section Q: open rows whose value is the thing's own, not an effect file. text and shape: replace the left photo for that row.)
final _rowsQ = <(String, String, Map<String, Object>)>[
  ('spacing', 'text', const {'@letter_spacing': 1.5, '@size': .7}),
  ('stroke', 'shape:rect', const {'@stroke': 'ff2266', '@stroke_width': 36, '@size': .5, '@fill': '2244aa'}),
  ('trim', 'shape:circle', const {'@fill': '', '@stroke': 'ffffff', '@stroke_width': 20, '@trim_begin': .1, '@trim': .4, '@size': .55}),
  ('radial', '-', const {'@copies': 8, '@copy_turn': 360, '@size': .28}),
];
// (section R: a spin the ISF already names, the ISF an open row names, and copies laid on a shape's outline.)
final _rowsR = <(String, String, Map<String, Object>)>[
  ('spinblur', 'Radial Blur', const {'mode': 1, 'amount': .4, 'samples': 2}),
  ('unsharp', 'Unsharp Mask', const {'intensity': 6}),
  ('chars', 'Random Characters', const {}),
  ('clock', 'Digital Clock', const {'clockMode': 2}),
  ('along', 'along', const {'@copies': 8, '@size': .1}),
];
// (section T: a matte the shader already fills, letters shown part way through an animator, letters moved each their own, one copy on an outline.)
final _rowsT = <(String, String, Map<String, Object>)>[
  ('matte', 'text+effects/am_solidmatte.wgsl', const {'@size': .32}),
  ('progress', 'text+animators/print_on.js', const {'@size': .32}),
  ('xform', 'text', const {'@split': 'letters', '@copy_y': .045, '@size': .32}),
  ('move', 'along', const {'@copies': 1, '@size': .12, '@copy_on_offset': .25}),
];
final _rows = switch (_section) { 'T' => _rowsT, 'R' => _rowsR, 'Q' => _rowsQ, 'P' => _rowsP, 'Z' => _rowsZ, 'Y' => _rowsY, 'X' => _rowsX, 'W' => _rowsW, 'S5' => _rowsS5, 'S4' => _rowsS4, 'S3' => _rowsS3, 'S2' => _rowsS2, 'S' => _rowsS, 'AM' => _rowsAM, 'D' => _rowsD, 'D2' => _rowsD2, 'B' => _rowsB, 'E' => _rowsE, 'E2' => _rowsE2, 'C' => _rowsC, 'F' => _rowsF, _ => _rowsA };

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  testWidgets('section $_section: each row on one layer', (t) async {
    t.view.physicalSize = const Size(3200, 1920);
    addTearDown(t.view.resetPhysicalSize);
    final folder = Directory('${Directory.systemTemp.createTempSync('alight_$_section').path}/Motolii')..createSync();
    WsZoom.set(1, keep: false);
    ColorTheme.at = '${folder.path}/none.xml';
    await t.pumpWidget(WidgetsApp(color: const Color(0xFF000000), builder: (c, _) => Workspace(document: '${folder.path}/Untitled.motolii')));
    for (var i = 0; i < 100 && find.byType(WsTimeline).evaluate().isEmpty; i++) {
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 20)));
      await t.pump();
    }
    await t.pumpAndSettle();
    final ws = WsScope.read(t.element(find.byType(WsTimeline))), m = ws.inside!;
    var left = m.add('picture', '$motoliiDir/fx_sample.jpg');
    final right = m.add('picture', '$motoliiDir/fx_sample.jpg');
    m
      ..set(left, 'x', -.5)
      ..set(right, 'x', .5);
    Future<void> shot(String png) async {
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 1500)));   // (the inside draws the stage in real time)
      for (var k = 0; k < 20; k++) {
        await t.pump(const Duration(milliseconds: 50));
      }
      final root = t.binding.renderViews.first.layer! as OffsetLayer;
      final bytes = await (await root.toImage(Offset.zero & t.view.physicalSize)).toByteData(format: ImageByteFormat.png);
      File('${folder.path}/$png').writeAsBytesSync(bytes!.buffer.asUint8List());
    }
    ws.reread();
    await shot('0_none.png');
    for (final (i, (row, effect, knobs)) in _rows.indexed) {
      var path = 0;
      if (effect == 'text' || effect.startsWith('shape:')) {
        expect(m.remove(left), isTrue, reason: m.trouble);
        left = effect == 'text' ? m.add('text', '/System/Library/Fonts/Supplemental/Arial Unicode.ttf') : m.add('shape', effect.substring('shape:'.length));
        expect(m.set(left, 'x', -.5), isTrue, reason: m.trouble);
        if (effect == 'text') expect(m.say(left, 'text', 'MOTOLII'), isTrue, reason: m.trouble);
      } else if (effect.startsWith('text+')) {
        expect(m.remove(left), isTrue, reason: m.trouble);
        left = m.add('text', '/System/Library/Fonts/Supplemental/Arial Unicode.ttf');
        expect(m.set(left, 'x', -.5), isTrue, reason: m.trouble);
        expect(m.say(left, 'text', 'MOTOLII'), isTrue, reason: m.trouble);
        final file = '$motoliiDir/${effect.substring('text+'.length)}';
        expect(m.say(left, file.endsWith('.js') ? 'script' : 'fx', file), isTrue, reason: '$row: ${m.trouble}');
      } else if (effect == 'along') {
        expect(m.remove(left), isTrue, reason: m.trouble);
        path = m.add('shape', 'circle');
        expect(m.set(path, 'x', -.5), isTrue, reason: m.trouble);
        expect(m.set(path, 'size', .55), isTrue, reason: m.trouble);
        expect(m.say(path, 'fill', ''), isTrue, reason: m.trouble);
        expect(m.say(path, 'stroke', '88ddff'), isTrue, reason: m.trouble);
        expect(m.set(path, 'stroke_width', 8), isTrue, reason: m.trouble);
        left = m.add('shape', 'rect');
        expect(m.set(left, 'x', -.5), isTrue, reason: m.trouble);
        expect(m.say(left, 'fill', 'ffcc33'), isTrue, reason: m.trouble);
        expect(m.set(left, 'copy_on', path.toDouble()), isTrue, reason: m.trouble);
      } else if (effect != '-') {
        final file = effect.contains('/') ? '$motoliiDir/$effect' : '$motoliiDir/effects/$effect.fs';
        expect(m.say(left, 'fx', file), isTrue, reason: '$row: ${m.trouble}');
      }
      final stem = effect.split('/').last.replaceFirst(RegExp(r'\.(js|wgsl|fs)$'), '').replaceAll(RegExp(r'[^A-Za-z0-9_]'), '_');
      for (final MapEntry(:key, :value) in knobs.entries) {
        final name = key.startsWith('@') ? key.substring(1) : 'js_${stem}_$key';   // (@: the thing's own value)
        expect(value is String ? m.say(left, name, value) : m.set(left, name, (value as num).toDouble()), isTrue, reason: '$row $key: ${m.trouble}');
      }
      ws.reread();
      if (effect.contains('print_on.js')) expect(m.at(.35), isTrue, reason: m.trouble);   // (inside the animator's own length, 0.67: some letters on)
      await shot('${i + 1}_${row.replaceAll(RegExp(r'[^A-Za-z0-9]+'), '_')}.png');
      if (effect.contains('print_on.js')) expect(m.at(0), isTrue, reason: m.trouble);
      if (path != 0) expect(m.remove(path), isTrue, reason: m.trouble);
      expect(m.remove(left), isTrue, reason: m.trouble);   // (the next row on a photo as it comes)
      left = m.add('picture', '$motoliiDir/fx_sample.jpg');
      m.set(left, 'x', -.5);
    }
    // ignore: avoid_print
    print('ALIGHT=${folder.path}');
  });
}
