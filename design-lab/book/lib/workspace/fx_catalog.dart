// The effects the engine knows, as the shelf receives them: what static analysis of the shader reads (seat, time, inputs, field count)
// and the two things only the author can say (`@tags` for the look, `@preview` for how to show it). Dock and Stage both read this.

/// The family an effect is listed under (Motolii's Effects rails).
enum FxFamily { blur, light, color, stylize, distort, space, path, place, time }

extension FxFamilyName on FxFamily {
  String get label => switch (this) {
    FxFamily.space => '3D',
    FxFamily.place => 'Place',
    _ => name[0].toUpperCase() + name.substring(1),
  };
}

class FxDef {
  const FxDef(
    this.name,
    this.family,
    this.desc, {
    this.seat = 'Pass',
    this.time = 'Static',
    this.below = false,
    this.layerIn = false,
    this.params = 2,
    this.builtIn = true,
    this.looks = const [],
    this.preview,
  });
  final String name, desc;
  final FxFamily family;

  /// Read from the shader: where it sits, whether it reads the clock or keeps a frame, what it takes in, how many fields it shows.
  final String seat, time;
  final bool below, layerIn;
  final int params;
  final bool builtIn;

  /// Written by the author: `@tags(...)`.
  final List<String> looks;

  /// Written by the author: `@preview(...)`, how a try-on should be shown when the plain frame says nothing.
  final String? preview;

  String get applies => switch (seat) {
    'Path' => 'Shape',
    'Text' => 'Text',
    'Field' || 'Surface' || 'Solid' => '3D',
    'Pass' || 'Warp' => 'Image',
    _ => 'Any',
  };
  String get inputs => below ? 'Reads below' : (layerIn ? 'Takes a layer' : 'Single');
  String get origin => builtIn ? 'Built-in' : 'Imported';

  /// Every value this effect carries, per filter group.
  String? valueIn(String group) => switch (group) {
    'Seat' => seat,
    'Applies to' => applies,
    'Time' => time,
    'Inputs' => inputs,
    'Parameters' => '$params',
    'Origin' => origin,
    _ => null,
  };

  /// What the try-on says about itself before you look: derived from the analysis unless the author wrote a preview line.
  String? get trialNote =>
      preview ??
      (below
          ? 'Seen on what is below the layer'
          : layerIn
          ? 'Takes a layer: pick one in the Inspector'
          : time == 'Feedback'
          ? 'Builds up over frames: play to see it'
          : null);
}

/// The filter groups, in band order. `Look` holds the author's tags; `Parameters` lists the counts that occur.
const fxGroups = <String>['Seat', 'Applies to', 'Time', 'Inputs', 'Look', 'Parameters', 'Origin'];

const fxDeclared = <String, List<String>>{
  'Seat': ['Pass', 'Warp', 'Surface', 'Field', 'Clip', 'Path', 'Placement', 'Text', 'Output'],
  'Applies to': ['Image', 'Shape', 'Text', '3D', 'Any'],
  'Time': ['Static', 'Uses time', 'Feedback'],
  'Inputs': ['Single', 'Reads below', 'Takes a layer'],
  'Origin': ['Built-in', 'Imported'],
};

/// The values a group offers: declared, or gathered from what the effects carry.
List<String> fxTagsOf(String group) => switch (group) {
  'Look' => ({for (final f in fxCatalog) ...f.looks}.toList()..sort()),
  'Parameters' => ({for (final f in fxCatalog) f.params}.toList()..sort()).map((n) => '$n').toList(),
  _ => fxDeclared[group] ?? const [],
};

bool fxHas(FxDef f, String group, String tag) => group == 'Look' ? f.looks.contains(tag) : f.valueIn(group) == tag;

/// Groups combine with AND, tags within a group with OR.
bool fxPasses(FxDef f, Map<String, Set<String>> filter) => filter.entries.every((e) => e.value.isEmpty || e.value.any((t) => fxHas(f, e.key, t)));

FxDef? fxNamed(String? name) => fxCatalog.where((f) => f.name == name).firstOrNull;

const fxCatalog = <FxDef>[
  FxDef('Gaussian Blur', FxFamily.blur, 'Soft', params: 1, looks: ['Soft']),
  FxDef('Motion Blur', FxFamily.blur, 'Directional', looks: ['Soft', 'Moving']),
  FxDef('Radial Blur', FxFamily.blur, 'Zoom · Spin', params: 3, looks: ['Moving']),
  FxDef('Lens Blur', FxFamily.blur, 'Depth map', params: 3, layerIn: true, looks: ['Soft']),
  FxDef('Glow', FxFamily.light, 'Radius · Threshold', params: 3, looks: ['Bright', 'Soft']),
  FxDef('Bloom', FxFamily.light, 'Highlights spill', params: 2, looks: ['Bright']),
  FxDef('Light Rays', FxFamily.light, 'Volumetric', params: 4, time: 'Uses time', looks: ['Bright'], preview: 'Rays form after a second: scrub past 00:01:00'),
  FxDef('Cast Shadow', FxFamily.light, 'From the sun', seat: 'Output', params: 3, below: true),
  FxDef('Curves', FxFamily.color, 'RGB · Luma', params: 1, looks: ['Grade']),
  FxDef('Hue Shift', FxFamily.color, 'Rotate', params: 1, looks: ['Grade']),
  FxDef('Duotone', FxFamily.color, 'Two inks', looks: ['Grade', 'Retro']),
  FxDef('Gradient Map', FxFamily.color, 'Luma to ramp', params: 3, builtIn: false, looks: ['Grade']),
  FxDef('Halftone', FxFamily.stylize, 'Dots · Lines', params: 3, looks: ['Retro']),
  FxDef('Pixelate', FxFamily.stylize, 'Block', params: 1, looks: ['Retro']),
  FxDef('Drop Shadow', FxFamily.stylize, 'Offset', params: 4),
  FxDef('Glitch Split', FxFamily.stylize, 'RGB · Jitter', params: 3, time: 'Uses time', builtIn: false, looks: ['Glitch']),
  FxDef('Film Grain', FxFamily.stylize, 'Grain · Flicker', time: 'Uses time', looks: ['Retro']),
  FxDef('Frosted Glass', FxFamily.stylize, 'Blurs what is behind', seat: 'Output', params: 2, below: true, looks: ['Soft']),
  FxDef('Wave Warp', FxFamily.distort, 'Sine · Square', seat: 'Warp', params: 4, time: 'Uses time', looks: ['Moving']),
  FxDef('Twirl', FxFamily.distort, 'Angle', seat: 'Warp', params: 2),
  FxDef('Bulge', FxFamily.distort, 'Lens', seat: 'Warp', params: 2),
  FxDef('Displace', FxFamily.distort, 'Map driven', seat: 'Warp', params: 3, layerIn: true, looks: ['Moving']),
  FxDef('Extrude', FxFamily.space, 'Depth · Bevel', seat: 'Solid', params: 3),
  FxDef('Surface Ripple', FxFamily.space, 'Mesh wave', seat: 'Surface', params: 3, time: 'Uses time', looks: ['Moving']),
  FxDef('Slice', FxFamily.space, 'World plane', seat: 'Clip', params: 2),
  FxDef('Trim Paths', FxFamily.path, 'Start · End', seat: 'Path', params: 3),
  FxDef('Pucker & Bloat', FxFamily.path, 'Amount', seat: 'Path', params: 1),
  FxDef('Offset Path', FxFamily.path, 'Grow · Shrink', seat: 'Path', params: 2),
  FxDef('Wiggle Path', FxFamily.path, 'Noise', seat: 'Path', params: 4, time: 'Uses time', looks: ['Moving']),
  FxDef('Repeater', FxFamily.place, 'Linear copies', seat: 'Placement', params: 5),
  FxDef('Radial Array', FxFamily.place, 'Around a centre', seat: 'Placement', params: 4),
  FxDef('Echo', FxFamily.time, 'Trails', params: 3, time: 'Feedback', looks: ['Moving'], preview: 'Trails read the 12 frames before this one'),
  FxDef('Posterize Time', FxFamily.time, 'Fewer frames', params: 1, time: 'Uses time', looks: ['Retro']),
];
