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
import 'package:perfect_freehand/perfect_freehand.dart' show PointVector, StrokeOptions, getStroke;
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
// (section U: Cavalry's oscillator, already a behaviour, at two moments of its own wave.)
final _rowsU = <(String, String, Map<String, Object>)>[
  ('oscHi', 'wave:x', const {'@minimum': -.7, '@maximum': -.2, '@when': .25}),
  ('oscLo', 'wave:x', const {'@minimum': -.7, '@maximum': -.2, '@when': .75}),
  ('opHi', 'wave:opacity', const {'@minimum': .15, '@maximum': 1, '@when': .25}),
  ('opLo', 'wave:opacity', const {'@minimum': .15, '@maximum': 1, '@when': .75}),
  ('szHi', 'wave:size', const {'@minimum': .35, '@maximum': .8, '@when': .25}),
  ('szLo', 'wave:size', const {'@minimum': .35, '@maximum': .8, '@when': .75}),
  ('swHi', 'wave:rotate', const {'@minimum': -18, '@maximum': 18, '@when': .25}),
  ('swLo', 'wave:rotate', const {'@minimum': -18, '@maximum': 18, '@when': .75}),
];
// (section V: the same behaviours, on opacity with a step, on noise, on random, and a smaller swing.)
final _rowsV = <(String, String, Map<String, Object>)>[
  ('blinkOn', 'wave:opacity', const {'@minimum': 0, '@maximum': 1, '@step': 1, '@when': .25}),
  ('blinkOff', 'wave:opacity', const {'@minimum': 0, '@maximum': 1, '@step': 1, '@when': .75}),
  ('shakeA', 'noise:xy', const {'@minimum': -.75, '@maximum': -.35, '@ymin': -.12, '@ymax': .12, '@when': .2}),
  ('shakeB', 'noise:xy', const {'@minimum': -.75, '@maximum': -.35, '@ymin': -.12, '@ymax': .12, '@when': .85}),
  ('jitterA', 'random:x', const {'@minimum': -.75, '@maximum': -.35, '@when': 0}),
  ('jitterB', 'random:x', const {'@minimum': -.75, '@maximum': -.35, '@when': 1}),
  ('swHi', 'wave:rotate', const {'@minimum': -8, '@maximum': 8, '@size': .5, '@when': .25}),
  ('swLo', 'wave:rotate', const {'@minimum': -8, '@maximum': 8, '@size': .5, '@when': .75}),
];
// (section AA: named ISF files at a value each file already declares, and noise at four moments.)
final _rowsAA = <(String, String, Map<String, Object>)>[
  ('scroll', 'effects/Side Scroller And Flip.fs', const {'slide': .35, 'shift': .15, 'mirrorHorizontal_on': 0, 'mirrorVertical_on': 0}),
  ('pixels', 'effects/Pixel Shifter.fs', const {'hFrequency': 4, 'vFrequency': 3}),
  ('hyper', 'effects/Hyperspace.fs', const {'scrollAmount': .8}),
  ('fade', 'effects/Fade.fs', const {'progress': .5}),
  ('rd1', 'noise:x', const {'@minimum': -.8, '@maximum': -.35, '@when': .05}),
  ('rd2', 'noise:x', const {'@minimum': -.8, '@maximum': -.35, '@when': .3}),
  ('rd3', 'noise:x', const {'@minimum': -.8, '@maximum': -.35, '@when': .55}),
  ('rd4', 'noise:x', const {'@minimum': -.8, '@maximum': -.35, '@when': .8}),
];
// (section BP: the thing's picture on a flat-faced ring, the solid turned a little so its sides are seen.)
final _rowsBP = <(String, String, Map<String, Object>)>[
  ('side', 'solids/Ring.js', const {'@rotate_x': 25, '@rotate_y': 35, '@size': 0.28, '@x': -.88}),
  ('ring', 'solids/Ring.js', const {'@rotate_x': 65, '@rotate_y': 20, '@size': 0.28, '@x': -.88}),
];
// (section BO: pixelate stretched into bands, the centre of each band filling the band.)
final _rowsBO = <(String, String, Map<String, Object>)>[
  ('plain', 'effects/am_pixelate.wgsl', const {'blockSize': 24, 'stretch_x': 1, 'stretch_y': 1, '@size': 0.28, '@x': -.88}),
  ('strip', 'effects/am_pixelate.wgsl', const {'blockSize': 24, 'stretch_x': 8, 'stretch_y': 1, '@size': 0.28, '@x': -.88}),
  ('tall', 'effects/am_pixelate.wgsl', const {'blockSize': 24, 'stretch_x': 1, 'stretch_y': 8, '@size': 0.28, '@x': -.88}),
];
// (section BN: a small photo shifted by more than the crop's margin, so the wrap can show.)
final _rowsBN = <(String, String, Map<String, Object>)>[
  ('plain', 'effects/am_offset.wgsl', const {'offset_x': 0, '@size': 0.28, '@x': -.88}),
  ('wrap', 'effects/am_offset.wgsl', const {'offset_x': 180, '@size': 0.28, '@x': -.88}),
  ('key', 'effects/am_chromakey.wgsl', const {'key_x': 0.50, 'key_y': 0.69, 'key_z': 0.90, 'tolerance': 0.12, '@size': 0.28, '@x': -.88}),
  ('spill', 'spill', const <String, Object>{}),
];
// (section BM: red and blue zoomed apart from the centre, then the same with a radial blur.)
final _rowsBM = <(String, String, Map<String, Object>)>[
  ('plain', 'effects/Chroma Zoom.fs', const {'red_zoom': 1, 'blue_zoom': 1, 'green_zoom': 1, 'master_zoom': 1, '@size': 0.28, '@x': -.88}),
  ('chroma', 'effects/Chroma Zoom.fs', const {'red_zoom': 1.2, 'blue_zoom': 1.4, '@size': 0.28, '@x': -.88}),
  ('zoom', 'chroma2', const <String, Object>{}),
];
// (section BL: unsharp mask at the file's own threshold, which leaves small differences alone.)
final _rowsBL = <(String, String, Map<String, Object>)>[
  ('flat', 'effects/am_unsharpmask.wgsl', const {'radius': 0.6, 'amount': 0, 'threshold': 1, '@size': 0.28, '@x': -.88}),
  ('edge', 'effects/am_unsharpmask.wgsl', const {'radius': 0.6, 'amount': 1.5, 'threshold': 1, '@size': 0.28, '@x': -.88}),
];
// (section BK: a keyed rect, then the same move mixed with the frame it just left. An arrow head on a line.)
final _rowsBK = <(String, String, Map<String, Object>)>[
  ('still', 'echo', const {'@when': 0.4}),
  ('ghost', 'echo', const {'@when': 0.4}),
  ('arrow', 'shape:line', const {'@fill': 'none', '@stroke': 'ffffff', '@stroke_width': 8, '@heads': 'both', '@head_shape': 'arrow', '@head_size': 6, '@size': 0.22, '@x': -.84}),
];
// (section BJ: a separable box of two equal-weight blurs, and a matte behind small text.)
final _rowsBJ = <(String, String, Map<String, Object>)>[
  ('plain', 'shape:rect', const {'@fill': 'ffcc33', '@size': 0.28, '@x': -.88}),
  ('box', 'box2', const <String, Object>{}),
  ('matte', 'text+effects/am_solidmatte.wgsl', const {'@x': -.92, '@size': 0.16}),
];
// (section BI: a freehand stroke, the brush's own outline.)
final _rowsBI = <(String, String, Map<String, Object>)>[
  ('ink', 'ink', const <String, Object>{}),
];
// (section BH: color balance, a raster scale, and a curl on a wide thin rect. Each file's own numbers.)
final _rowsBH = <(String, String, Map<String, Object>)>[
  ('balance', 'effects/am_colorbalance.wgsl', const {'amount': 1, '@x': -.88, '@size': 0.28}),
  ('raster', 'effects/am_rastertransform.wgsl', const {'scale': 1.35, '@x': -.88, '@size': 0.28}),
  ('curl', 'rect+effects/am_curl.wgsl', const {'offset': .35, 'radius': 8, '@fill': 'ffcc33', '@size': 0.12, '@scale_x': 5, '@x': -.82}),
];
// (section BG: brightness mapped to two declared colours, and a line with its own glow.)
final _rowsBG = <(String, String, Map<String, Object>)>[
  ('false', 'effects/False Color.fs', const <String, Object>{}),
  ('line', 'shape:line', const {'@fill': 'ffffff', '@stroke': 'ffffff', '@stroke_width': 10, '@size': 0.28, '@x': -.78}),
  ('beam', 'shape:line', const {'@fill': 'ffffff', '@stroke': 'ffffff', '@stroke_width': 10, '@size': 0.28, '@x': -.78, '@glow': 'ffffff', '@glow_opacity': 1, '@glow_blur': 80}),
];
// (section BF: a text layer's clock at two document times.)
final _rowsBF = <(String, String, Map<String, Object>)>[
  ('t0', 'text+effects/Digital Clock.fs', const {'clockMode': 2, '@when': 0, '@x': -.78, '@size': 0.22}),
  ('t1', 'text+effects/Digital Clock.fs', const {'clockMode': 2, '@when': 5, '@x': -.78, '@size': 0.22}),
];
// (section BE: a circle's own vignette, and a 3x3 unsharp mask on the photo.)
final _rowsBE = <(String, String, Map<String, Object>)>[
  ('rim0', 'circle+effects/am_vignette.wgsl', const {'extent': 0.7, 'strength': 0, 'feather': 0.35, '@fill': 'ffcc33', '@size': 0.4, '@x': -.72}),
  ('rim', 'circle+effects/am_vignette.wgsl', const {'extent': 0.55, 'strength': 1, 'feather': 0.35, '@fill': 'ffcc33', '@size': 0.4, '@x': -.72}),
  ('sharp', 'effects/am_sharpen.wgsl', const {'amount': 6, 'radius': 1.5}),
];
// (section BD: a white shape's own glow, a child's share of its parent's turn and size, and a flat colour cut by the photo's brightness.)
final _rowsBD = <(String, String, Map<String, Object>)>[
  ('plain', 'shape:circle', const {'@fill': 'ffffff', '@size': 0.22, '@x': -.72}),
  ('glow', 'shape:circle', const {'@fill': 'ffffff', '@size': 0.22, '@x': -.72, '@glow': 'ffffff', '@glow_opacity': 1, '@glow_blur': 80}),
  ('follow', 'help', const {'@turn': 40, '@parent_rotate': 1}),
  ('lock', 'help', const {'@turn': 40, '@parent_rotate': 0}),
  ('grown', 'help', const {'@wide': 2.2, '@parent_size': 1}),
  ('kept', 'help', const {'@wide': 2.2, '@parent_size': 0}),
  ('stamp', 'stamp', const <String, Object>{}),
];
// (section BC: a shape squashed so its area stays, and copies sat on a circle.)
final _rowsBC = <(String, String, Map<String, Object>)>[
  ('box', 'shape:rect', const {'@fill': 'ffcc33', '@size': 0.36, '@x': -.72}),
  ('squash', 'shape:rect', const {'@fill': 'ffcc33', '@size': 0.36, '@x': -.72, '@squash': 0.6}),
  ('ring', 'along', const {'@copies': 8, '@size': 0.08}),
];
// (section BB: more, shorter contour strips, and a vignette small enough to reach the photo.)
final _rowsBB = <(String, String, Map<String, Object>)>[
  ('paper', 'effects/am_contourstrips.wgsl', const {'count': 36, 'height': 0.35}),
  ('paper2', 'effects/am_contourstrips.wgsl', const {'count': 60, 'height': 0.5}),
  ('vig', 'effects/am_vignette.wgsl', const {'extent': 0.04, 'strength': 1, 'feather': 0.2}),
];
// (section BA: stars shifted by the file's offset, and letters each at their own height, kept on the left.)
final _rowsBA = <(String, String, Map<String, Object>)>[
  ('s0', 'effects/am_starfield.wgsl', const {'brightness': 1.5, 'scale': 0.3}),
  ('s1', 'effects/am_starfield.wgsl', const {'brightness': 1.5, 'scale': 0.3, 'offset_x': 40}),
  ('s2', 'effects/am_starfield.wgsl', const {'brightness': 1.5, 'scale': 0.3, 'offset_y': 40}),
  ('letters', 'text', const {'@split': 'letters', '@copy_y': 0.05, '@size': 0.18, '@x': -.78}),
];
// (section AZ: one copy on a circle, once stagger makes it a member, and a layer whose keys step.)
final _rowsAZ = <(String, String, Map<String, Object>)>[
  ('path25', 'along', const {'@stagger': 0.01, '@copy_on_offset': 0.25, '@size': 0.1}),
  ('path50', 'along', const {'@stagger': 0.01, '@copy_on_offset': 0.5, '@size': 0.1}),
  ('q1', 'quant', const {'@when': 0.1}),
  ('q2', 'quant', const {'@when': 0.2}),
  ('q3', 'quant', const {'@when': 0.3}),
];
// (section AY: a vignette whose size sits inside the photo, a vertical stretch, and noise that steps.)
final _rowsAY = <(String, String, Map<String, Object>)>[
  ('vig', 'effects/am_vignette.wgsl', const {'extent': 0.18, 'strength': 1, 'feather': 0.3}),
  ('vig2', 'effects/am_vignette.wgsl', const {'extent': 0.12, 'strength': 1, 'feather': 0.2}),
  ('tall', 'effects/am_stretchaxis.wgsl', const {'scale': 1.8, 'angle': 90}),
  ('tall2', 'effects/am_stretchaxis.wgsl', const {'scale': 1.4, 'angle': 90}),
  ('j0', 'noise:x', const {'@minimum': -.9, '@maximum': -.4, '@frequency': 6, '@step': .2, '@when': 0}),
  ('j1', 'noise:x', const {'@minimum': -.9, '@maximum': -.4, '@frequency': 6, '@step': .2, '@when': .25}),
  ('j2', 'noise:x', const {'@minimum': -.9, '@maximum': -.4, '@frequency': 6, '@step': .2, '@when': .55}),
  ('j3', 'noise:x', const {'@minimum': -.9, '@maximum': -.4, '@frequency': 6, '@step': .2, '@when': .9}),
];
// (section AX: noise at a frequency that crosses more than one cell, so the photo is not carried one way.)
final _rowsAX = <(String, String, Map<String, Object>)>[
  ('d0', 'noise:xy', const {'@minimum': -.85, '@maximum': -.45, '@ymin': -.12, '@ymax': .12, '@frequency': 4, '@when': 0}),
  ('d1', 'noise:xy', const {'@minimum': -.85, '@maximum': -.45, '@ymin': -.12, '@ymax': .12, '@frequency': 4, '@when': .35}),
  ('d2', 'noise:xy', const {'@minimum': -.85, '@maximum': -.45, '@ymin': -.12, '@ymax': .12, '@frequency': 4, '@when': .7}),
  ('d3', 'noise:xy', const {'@minimum': -.85, '@maximum': -.45, '@ymin': -.12, '@ymax': .12, '@frequency': 4, '@when': 1.15}),
];
// (section AW: a wipe whose start and end sit inside the file's range, and a stretch short of the one that reached the other photo.)
final _rowsAW = <(String, String, Map<String, Object>)>[
  ('wipe', 'effects/am_wipe.wgsl', const {'start': 0.35, 'end': 0.65}),
  ('wipe2', 'effects/am_wipe.wgsl', const {'start': 0.45, 'end': 0.55}),
  ('stretch', 'effects/am_stretchaxis.wgsl', const {'scale': 1.2}),
];
// (section AV: rays the file declares. The short length, then a longer one still inside the file's range.)
final _rowsAV = <(String, String, Map<String, Object>)>[
  ('rays0', 'effects/am_rays.wgsl', const <String, Object>{}),
  ('rays1', 'effects/am_rays.wgsl', const {'reach': 1.5, 'intensity': 2}),
];
// (section AU: a modest squeeze the file allows, a page curl at a progress the file allows, and a starfield moved a short way on each axis.)
final _rowsAU = <(String, String, Map<String, Object>)>[
  ('squeeze', 'effects/am_squeeze.wgsl', const {'amount': 0.5}),
  ('curl', 'effects/Inverted Page Curl.fs', const {'progress': 0.4}),
  ('stars0', 'effects/am_starfield.wgsl', const {'brightness': 1.5, 'offset_x': 0, 'offset_y': 0}),
  ('starsX', 'effects/am_starfield.wgsl', const {'brightness': 1.5, 'offset_x': 20, 'offset_y': 0}),
  ('starsY', 'effects/am_starfield.wgsl', const {'brightness': 1.5, 'offset_x': 0, 'offset_y': 20}),
];
// (section AT: angles the files declare. Extrude at its own rotation. A panorama turned by yaw. Flicker at two times.)
final _rowsAT = <(String, String, Map<String, Object>)>[
  ('extrude', 'effects/am_rasterextrude.wgsl', const <String, Object>{}),
  ('reorient', 'effects/am_reorient360.wgsl', const {'yaw': 90}),
  ('flick0', 'effects/am_flicker.wgsl', const {'@when': 0}),
  ('flick1', 'effects/am_flicker.wgsl', const {'@when': 0.2}),
];
// (section AS: the layer's length, and an angle the file declares. Fade at the start, the middle, and the end. Lens blur at a sweep the file allows. Contour strips at the file's own height.)
final _rowsAS = <(String, String, Map<String, Object>)>[
  ('fade0', 'effects/am_fade.wgsl', const {'@when': 0}),
  ('fade1', 'effects/am_fade.wgsl', const {'@when': 2}),
  ('fade2', 'effects/am_fade.wgsl', const {'@when': 7.9}),
  ('lens', 'effects/am_lensblur.wgsl', const {'sweep': 45}),
  ('strips', 'effects/am_contourstrips.wgsl', const <String, Object>{}),
];
// (section AR: the scene size AbsentMotion passes as screen, so a wave and an unsharp mask use the reach the file states.)
final _rowsAR = <(String, String, Map<String, Object>)>[
  ('wave', 'effects/am_wavewarp.wgsl', const {'magnitude': 4}),
  ('wave2', 'effects/am_wavewarp.wgsl', const {'magnitude': 12, 'spacing': 8}),
  ('sharp', 'effects/am_unsharpmask.wgsl', const {'radius': 1, 'amount': 1}),
];
// (section AQ: the file's own width, now that a shader which already has resolution keeps that knob. Each one as the file declares it.)
final _rowsAQ = <(String, String, Map<String, Object>)>[
  ('grid', 'effects/am_grid.wgsl', const <String, Object>{}),
  ('rays', 'effects/am_radialrays.wgsl', const <String, Object>{}),
  ('bolt', 'effects/am_lightning.wgsl', const <String, Object>{}),
  ('ribbon', 'effects/am_ribbon.wgsl', const <String, Object>{}),
];
// (section AP: a small text matte, and a glow on a circle, both kept clear of the right photo.)
final _rowsAP = <(String, String, Map<String, Object>)>[
  ('matte', 'text+effects/am_solidmatte.wgsl', const {'@x': -.72, '@size': .2, 'color_x': .1765, 'color_y': .1176, 'color_z': .9647, 'color_w': 1, 'alpha': 1}),
  ('glow', 'shape:circle', const {'@x': -.72, '@size': .36, '@fill': 'ffcc33', '@glow': 'ffffff', '@glow_opacity': 1, '@glow_blur': 40}),
];
// (section AO: shadows from a circle, short enough to stay off the right photo. The shadow colour is one the file allows, so it shows on the black stage.)
final _rowsAO = <(String, String, Map<String, Object>)>[
  ('long', 'circle+effects/am_longshadow.wgsl', const {'@fill': 'ffcc33', '@size': .42, 'reach': 40, 'alpha': .9, 'fade': .3, 'color_x': 1, 'color_y': 1, 'color_z': 1, 'color_w': 1}),
  ('radial', 'circle+effects/am_radialshadow.wgsl', const {'@fill': 'ffcc33', '@size': .42, 'light_x': .15, 'light_y': .5, 'reach': .4, 'alpha': .85, 'color_x': 1, 'color_y': 1, 'color_z': 1, 'color_w': 1}),
];
// (section AN: a cut the file's stretch can open, and a small wrap the file's offset allows.)
final _rowsAN = <(String, String, Map<String, Object>)>[
  ('cut', 'effects/am_stretch.wgsl', const {'stretch': 48, 'spread': 16}),
  ('slide', 'effects/am_offset.wgsl', const {'offset_x': 36}),
];
// (section AL: tile shifts at a small strength the file allows, so the tiles stay on the left photo.)
final _rowsAL = <(String, String, Map<String, Object>)>[
  ('hex', 'effects/am_hexagontileshift.wgsl', const {'count': 6, 'strength': .08, 'seed': 1}),
  ('grid', 'effects/am_tileshift.wgsl', const {'count': 6, 'strength': .08, 'seed': 1}),
];
// (section AK: tile rotate, each tile turned by the file's Randomize, which is 0 at rest.)
final _rowsAK = <(String, String, Map<String, Object>)>[
  ('tiles', 'effects/am_tilerotate.wgsl', const {'count': 4, 'random': 1, 'seed': 1}),
];
// (section AJ: spot colour set to a red the balloon actually has, choke and threshold inside the file's range.)
final _rowsAJ = <(String, String, Map<String, Object>)>[
  ('spotWide', 'effects/am_spotcolor.wgsl', const {'color_x': .71, 'color_y': .02, 'color_z': .13, 'color_w': 1, 'threshold': .5, 'feather': .25, 'saturation': 0, 'choke': 1}),
  ('spot', 'effects/am_spotcolor.wgsl', const {'color_x': .71, 'color_y': .02, 'color_z': .13, 'color_w': 1, 'threshold': .35, 'feather': .25, 'saturation': 0, 'choke': 5}),
];
// (section AI: grid lines and sunburst rays, thinner than the file's default, in a colour the file allows.)
final _rowsAI = <(String, String, Map<String, Object>)>[
  ('grid', 'effects/am_grid.wgsl', const {'count': 12, 'width': .04, 'color_x': 1, 'color_y': .1, 'color_z': .1, 'color_w': 1}),
  ('rays', 'effects/am_radialrays.wgsl', const {'count': 8, 'width': .08, 'alpha': 1, 'color_x': 1, 'color_y': .15, 'color_z': .1, 'color_w': 1}),
];
// (section AH: the starfield the file already draws, at two offsets it declares, so near and far can move apart.)
final _rowsAH = <(String, String, Map<String, Object>)>[
  ('stars0', 'effects/am_starfield.wgsl', const {'brightness': 1.5, 'offset_x': 0, 'scale': .3}),
  ('stars1', 'effects/am_starfield.wgsl', const {'brightness': 1.5, 'offset_x': 80, 'scale': .3}),
];
// (section AG: vignette inside the file's size, so the edge is dark and the middle stays.)
final _rowsAG = <(String, String, Map<String, Object>)>[
  ('vignette', 'effects/am_vignette.wgsl', const {'extent': .55, 'strength': 1, 'feather': .4}),
];
// (section AF: a sine the file already names, once screen is the picture's size, and a stretch inside the file's scale.)
final _rowsAF = <(String, String, Map<String, Object>)>[
  ('wave', 'effects/am_wavewarp.wgsl', const {'magnitude': 12, 'spacing': 8}),
  ('stretch', 'effects/am_stretchaxis.wgsl', const {'scale': 1.45}),
];
// (section AE: chroma key at the sky color the photo already has, inside the file's 0..1.)
final _rowsAE = <(String, String, Map<String, Object>)>[
  ('keyA', 'effects/am_chromakey.wgsl', const {'key_x': .50, 'key_y': .69, 'key_z': .90, 'tolerance': .12}),
  ('keyB', 'effects/am_chromakey.wgsl', const {'key_x': .50, 'key_y': .69, 'key_z': .90, 'tolerance': .25}),
];
// (section AD: AbsentMotion effects the checklist names, at a value each file already declares.)
final _rowsAD = <(String, String, Map<String, Object>)>[
  ('balance', 'effects/am_colorbalance.wgsl', const {'amount': 1}),
  ('curl', 'effects/am_curl.wgsl', const {'offset': .2, 'radius': 40}),
  ('raster', 'effects/am_rastertransform.wgsl', const {'scale': 1.6, 'offset_x': 80}),
];
// (section AC: fill "behind" on a shape over the left photo, letter fades at two moments, noise on opacity at two moments.)
final _rowsAC = <(String, String, Map<String, Object>)>[
  ('behind', 'behind', const {}),
  ('in0', 'text+animators/fade_in.js', const {'@size': .32, '@when': 0}),
  ('in1', 'text+animators/fade_in.js', const {'@size': .32, '@when': .9}),
  ('out0', 'text+animators/fade_out.js', const {'@size': .32, '@when': 0}),
  ('out1', 'text+animators/fade_out.js', const {'@size': .32, '@when': 3.6}),
  ('flickA', 'noise:opacity', const {'@minimum': .15, '@maximum': 1, '@when': .2}),
  ('flickB', 'noise:opacity', const {'@minimum': .15, '@maximum': 1, '@when': .85}),
];
// (section AB: a behaviour on the camera. Both photos move together, because the camera moved.)
final _rowsAB = <(String, String, Map<String, Object>)>[
  ('xHi', 'cam:x', const {'@minimum': -.35, '@maximum': .35, '@when': .25}),
  ('xLo', 'cam:x', const {'@minimum': -.35, '@maximum': .35, '@when': .75}),
  ('rotHi', 'cam:rotate', const {'@minimum': -12, '@maximum': 12, '@when': .25}),
  ('rotLo', 'cam:rotate', const {'@minimum': -12, '@maximum': 12, '@when': .75}),
  ('zoomHi', 'cam:zoom', const {'@minimum': .7, '@maximum': 1.6, '@when': .25}),
  ('zoomLo', 'cam:zoom', const {'@minimum': .7, '@maximum': 1.6, '@when': .75}),
];
final _rows = switch (_section) { 'BP' => _rowsBP, 'BO' => _rowsBO, 'BN' => _rowsBN, 'BM' => _rowsBM, 'BL' => _rowsBL, 'BK' => _rowsBK, 'BJ' => _rowsBJ, 'BI' => _rowsBI, 'BH' => _rowsBH, 'BG' => _rowsBG, 'BF' => _rowsBF, 'BE' => _rowsBE, 'BD' => _rowsBD, 'BC' => _rowsBC, 'BB' => _rowsBB, 'BA' => _rowsBA, 'AZ' => _rowsAZ, 'AY' => _rowsAY, 'AX' => _rowsAX, 'AW' => _rowsAW, 'AV' => _rowsAV, 'AU' => _rowsAU, 'AT' => _rowsAT, 'AS' => _rowsAS, 'AR' => _rowsAR, 'AQ' => _rowsAQ, 'AP' => _rowsAP, 'AO' => _rowsAO, 'AN' => _rowsAN, 'AL' => _rowsAL, 'AK' => _rowsAK, 'AJ' => _rowsAJ, 'AI' => _rowsAI, 'AH' => _rowsAH, 'AG' => _rowsAG, 'AF' => _rowsAF, 'AE' => _rowsAE, 'AD' => _rowsAD, 'AC' => _rowsAC, 'AB' => _rowsAB, 'AA' => _rowsAA, 'V' => _rowsV, 'U' => _rowsU, 'T' => _rowsT, 'R' => _rowsR, 'Q' => _rowsQ, 'P' => _rowsP, 'Z' => _rowsZ, 'Y' => _rowsY, 'X' => _rowsX, 'W' => _rowsW, 'S5' => _rowsS5, 'S4' => _rowsS4, 'S3' => _rowsS3, 'S2' => _rowsS2, 'S' => _rowsS, 'AM' => _rowsAM, 'D' => _rowsD, 'D2' => _rowsD2, 'B' => _rowsB, 'E' => _rowsE, 'E2' => _rowsE2, 'C' => _rowsC, 'F' => _rowsF, _ => _rowsA };

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
      var path = 0, path2 = 0, path3 = 0;
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
      } else if (effect == 'quant') {
        expect(m.key(left, 'x', 0, -.85) && m.key(right, 'x', 0, .35), isTrue, reason: m.trouble);
        expect(m.key(left, 'x', 1, -.45) && m.key(right, 'x', 1, .75), isTrue, reason: m.trouble);
        expect(m.set(left, 'frame_rate', 4), isTrue, reason: m.trouble);
      } else if (effect == 'help') {
        expect(m.remove(left), isTrue, reason: m.trouble);
        path = m.add('shape', 'rect');
        expect(m.set(path, 'x', -.72) && m.set(path, 'size', .2), isTrue, reason: m.trouble);
        expect(m.say(path, 'fill', '88ddff'), isTrue, reason: m.trouble);
        if (knobs['@turn'] is num) expect(m.set(path, 'rotate', (knobs['@turn'] as num).toDouble()), isTrue, reason: m.trouble);
        if (knobs['@wide'] is num) expect(m.set(path, 'scale_x', (knobs['@wide'] as num).toDouble()), isTrue, reason: m.trouble);
        left = m.add('shape', 'rect');
        expect(m.set(left, 'size', .14), isTrue, reason: m.trouble);
        expect(m.say(left, 'fill', 'ffcc33'), isTrue, reason: m.trouble);
        expect(m.set(left, 'parent', path.toDouble()), isTrue, reason: m.trouble);
      } else if (effect == 'stamp') {
        expect(m.remove(right), isTrue, reason: m.trouble);
        path = m.add('shape', 'rect');
        expect(m.set(path, 'x', -.5) && m.set(path, 'size', .55) && m.set(path, 'clip', 1), isTrue, reason: m.trouble);
        expect(m.say(path, 'fill', 'ff2200') && m.say(path, 'clip_matte', 'luma'), isTrue, reason: m.trouble);
        final again = m.add('picture', '$motoliiDir/fx_sample.jpg');
        expect(m.set(again, 'x', .5), isTrue, reason: m.trouble);
      } else if (effect == 'behind') {
        path = m.add('shape', 'rect');
        expect(path, isNot(0), reason: m.trouble);
        expect(m.set(path, 'x', -.5) && m.set(path, 'y', .08) && m.set(path, 'size', .42), isTrue, reason: m.trouble);
        expect(m.say(path, 'fill', 'behind'), isTrue, reason: m.trouble);
      } else if (effect.startsWith('wave:') || effect.startsWith('noise:') || effect.startsWith('random:')) {
        final kind = effect.startsWith('wave:') ? 'oscillator' : effect.startsWith('noise:') ? 'noise' : 'random';
        final name = effect.substring(effect.indexOf(':') + 1);
        path = m.add('behaviour', kind);
        expect(path, isNot(0), reason: m.trouble);
        expect(m.set(path, 'minimum', (knobs['@minimum'] as num).toDouble()), isTrue, reason: m.trouble);
        expect(m.set(path, 'maximum', (knobs['@maximum'] as num).toDouble()), isTrue, reason: m.trouble);
        expect(m.set(path, 'frequency', knobs['@frequency'] is num ? (knobs['@frequency'] as num).toDouble() : 1), isTrue, reason: m.trouble);
        if (knobs['@step'] is num) expect(m.set(path, 'step', (knobs['@step'] as num).toDouble()), isTrue, reason: m.trouble);
        if (name == 'xy') {
          expect(m.set(left, 'x_by', path.toDouble()), isTrue, reason: '$row: ${m.trouble}');
          path2 = m.add('behaviour', kind);
          expect(m.set(path2, 'minimum', (knobs['@ymin'] as num).toDouble()), isTrue, reason: m.trouble);
          expect(m.set(path2, 'maximum', (knobs['@ymax'] as num).toDouble()), isTrue, reason: m.trouble);
          expect(m.set(path2, 'frequency', knobs['@frequency'] is num ? (knobs['@frequency'] as num).toDouble() : 1), isTrue, reason: m.trouble);
          expect(m.set(path2, 'seed', 7), isTrue, reason: m.trouble);
          expect(m.set(left, 'y_by', path2.toDouble()), isTrue, reason: '$row: ${m.trouble}');
        } else {
          expect(m.set(left, '${name}_by', path.toDouble()), isTrue, reason: '$row: ${m.trouble}');
        }
      } else if (effect.startsWith('cam:')) {
        path = m.add('behaviour', 'oscillator');
        expect(path, isNot(0), reason: m.trouble);
        expect(m.set(path, 'minimum', (knobs['@minimum'] as num).toDouble()), isTrue, reason: m.trouble);
        expect(m.set(path, 'maximum', (knobs['@maximum'] as num).toDouble()), isTrue, reason: m.trouble);
        expect(m.set(path, 'frequency', 1), isTrue, reason: m.trouble);
        path2 = m.add('camera', 'camera');
        expect(path2, isNot(0), reason: m.trouble);
        expect(m.remove(left), isTrue, reason: m.trouble);
        left = m.add('solid', 'cube');   // (a solid is in the world, so the camera moves it)
        expect(m.set(left, 'size', .45), isTrue, reason: m.trouble);
        expect(m.say(left, 'fill', 'ffcc33'), isTrue, reason: m.trouble);
        path3 = m.add('text', '/System/Library/Fonts/Supplemental/Arial Unicode.ttf');
        expect(m.say(path3, 'text', 'SUB') && m.say(path3, 'space', '2.5d'), isTrue, reason: m.trouble);
        expect(m.set(path3, 'y', .35) && m.set(path3, 'size', .25) && m.set(path3, 'parent', path2.toDouble()), isTrue, reason: m.trouble);
        expect(m.set(path2, '${effect.substring('cam:'.length)}_by', path.toDouble()), isTrue, reason: '$row: ${m.trouble}');
      } else if (effect == 'echo') {
        expect(m.remove(left), isTrue, reason: m.trouble);
        left = m.add('shape', 'rect');
        expect(m.set(left, 'size', .12) && m.set(left, 'y', .42), isTrue, reason: m.trouble);
        expect(m.say(left, 'fill', 'ffcc33'), isTrue, reason: m.trouble);
        expect(m.key(left, 'x', 0, -.9) && m.key(left, 'x', .4, -.62), isTrue, reason: m.trouble);
        if (row == 'ghost') {
          expect(m.say(left, 'fx', '$motoliiDir/effects/VVMotionBlur 3.0.fs'), isTrue, reason: m.trouble);
          expect(m.set(left, 'js_VVMotionBlur_3_0_blurAmount', .5), isTrue, reason: m.trouble);
        }
        expect(m.at(0), isTrue, reason: m.trouble);
        await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 800)));
        for (var k = 0; k < 8; k++) {
          await t.pump(const Duration(milliseconds: 50));
        }
      } else if (effect == 'spill') {
        expect(m.set(left, 'x', -.88) && m.set(left, 'size', .28), isTrue, reason: m.trouble);
        final key = '$motoliiDir/effects/am_chromakey.wgsl', clean = '$motoliiDir/effects/Chroma Desaturation Mask.fs';
        expect(m.say(left, 'fx', '$key $clean'), isTrue, reason: m.trouble);
        expect(m.set(left, 'js_am_chromakey_key_x', .50) && m.set(left, 'js_am_chromakey_key_y', .69) && m.set(left, 'js_am_chromakey_key_z', .90) && m.set(left, 'js_am_chromakey_tolerance', .12), isTrue, reason: m.trouble);
        expect(m.set(left, 'js_Chroma_Desaturation_Mask_mask_color_x', .50) && m.set(left, 'js_Chroma_Desaturation_Mask_mask_color_y', .69) && m.set(left, 'js_Chroma_Desaturation_Mask_mask_color_z', .90), isTrue, reason: m.trouble);
      } else if (effect == 'chroma2') {
        expect(m.set(left, 'x', -.88) && m.set(left, 'size', .28), isTrue, reason: m.trouble);
        final zoom = '$motoliiDir/effects/Chroma Zoom.fs', blur = '$motoliiDir/effects/am_radialblur.wgsl';
        expect(m.say(left, 'fx', '$zoom $blur'), isTrue, reason: m.trouble);
        expect(m.set(left, 'js_Chroma_Zoom_red_zoom', 1.2) && m.set(left, 'js_Chroma_Zoom_blue_zoom', 1.4), isTrue, reason: m.trouble);
        expect(m.set(left, 'js_am_radialblur_amount', .35), isTrue, reason: m.trouble);
      } else if (effect == 'box2') {
        expect(m.remove(left), isTrue, reason: m.trouble);
        left = m.add('shape', 'rect');
        expect(m.set(left, 'x', -.88) && m.set(left, 'size', .28), isTrue, reason: m.trouble);
        expect(m.say(left, 'fill', 'ffcc33'), isTrue, reason: m.trouble);
        final blur = '$motoliiDir/effects/Diagonal Blur.fs', dir = '$motoliiDir/effects/am_directionalblur.wgsl';
        expect(m.say(left, 'fx', '$blur $dir'), isTrue, reason: m.trouble);
        expect(m.set(left, 'js_Diagonal_Blur_width', .02) && m.set(left, 'js_Diagonal_Blur_angle', 0) && m.set(left, 'js_Diagonal_Blur_quality', 2), isTrue, reason: m.trouble);
        expect(m.set(left, 'js_am_directionalblur_blurLength', 20) && m.set(left, 'js_am_directionalblur_angle', 90), isTrue, reason: m.trouble);
      } else if (effect == 'ink') {
        final way = [for (var i = 0; i <= 48; i++) PointVector.fromOffset(offset: Offset(1180 + i * 7, 760))];
        final outline = getStroke(way, options: StrokeOptions(size: 32, simulatePressure: true, isComplete: true));
        expect(outline.length, greaterThan(3), reason: 'ink');
        m.later('together', number: 1);
        m.later('freehand', word: [for (final p in outline) '${p.dx} ${p.dy}'].join(' '));
        m.later('say', thing: -1, name: 'fill', word: 'ffcc33');
        m.later('together');
      } else if (effect.startsWith('circle+') || effect.startsWith('rect+')) {
        final wide = effect.startsWith('rect+');
        expect(m.remove(left), isTrue, reason: m.trouble);
        left = m.add('shape', wide ? 'rect' : 'circle');
        expect(m.set(left, 'x', -.5), isTrue, reason: m.trouble);
        final file = '$motoliiDir/${effect.substring(wide ? 'rect+'.length : 'circle+'.length)}';
        expect(m.say(left, 'fx', file), isTrue, reason: '$row: ${m.trouble}');
      } else if (effect != '-') {
      final file = effect.contains('/') ? '$motoliiDir/$effect' : '$motoliiDir/effects/$effect.fs';
        expect(m.say(left, 'fx', file), isTrue, reason: '$row: ${m.trouble}');
      }
      final stem = effect.split('/').last.replaceFirst(RegExp(r'\.(js|wgsl|fs)$'), '').replaceAll(RegExp(r'[^A-Za-z0-9_]'), '_');
      for (final MapEntry(:key, :value) in knobs.entries) {
        if (key == '@when' || key == '@minimum' || key == '@maximum' || key == '@step' || key == '@ymin' || key == '@ymax' || key == '@frequency' || key == '@turn' || key == '@wide') continue;   // (the behaviour's own, or when the playhead sits)
        final name = key.startsWith('@') ? key.substring(1) : 'js_${stem}_$key';   // (@: the thing's own value)
        expect(value is String ? m.say(left, name, value) : m.set(left, name, (value as num).toDouble()), isTrue, reason: '$row $key: ${m.trouble}');
      }
      ws.reread();
      final when = knobs['@when'];
      if (when is num) expect(m.at(when.toDouble()), isTrue, reason: m.trouble);
      else if (effect.contains('print_on.js')) expect(m.at(.35), isTrue, reason: m.trouble);   // (inside the animator's own length, 0.67: some letters on)
      await shot('${i + 1}_${row.replaceAll(RegExp(r'[^A-Za-z0-9]+'), '_')}.png');
      if (when is num || effect.contains('print_on.js')) expect(m.at(0), isTrue, reason: m.trouble);
      if (path3 != 0) expect(m.remove(path3), isTrue, reason: m.trouble);
      if (path2 != 0) expect(m.remove(path2), isTrue, reason: m.trouble);
      if (path != 0) expect(m.remove(path), isTrue, reason: m.trouble);
      expect(m.remove(left), isTrue, reason: m.trouble);   // (the next row on a photo as it comes)
      left = m.add('picture', '$motoliiDir/fx_sample.jpg');
      m.set(left, 'x', -.5);
    }
    // ignore: avoid_print
    print('ALIGHT=${folder.path}');
  });
}
