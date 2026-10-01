// Fixtures for Phase 1: an effect the Inspector knows nothing about, declared only by types.
List<Map<String, dynamic>> unknownEffect() => [
      {'id': 'fx.param.foo', 'label': 'Foo', 'kind': 'f32', 'value': .35, 'default': .5, 'hero': true, 'animated': true, 'keyedNow': true},
      {'id': 'fx.param.mix', 'label': 'Mix', 'kind': 'f32', 'value': .62, 'default': .5, 'min': 0.0, 'max': 1.0, 'hero': true},
      {'id': 'fx.param.iterations', 'label': 'Iterations', 'kind': 'i32', 'value': 6, 'default': 4, 'min': 1, 'max': 64, 'hero': true},
      {'id': 'fx.param.invert', 'label': 'Invert', 'kind': 'bool', 'value': true, 'default': false, 'hero': true},
      {'id': 'fx.param.bar', 'label': 'Bar', 'kind': 'f32', 'value': -12840.5, 'default': 0.0, 'min': -1e9, 'max': 1e9, 'section': 'Numbers', 'animated': true},
      {'id': 'fx.param.gain', 'label': 'Gain', 'kind': 'f32', 'value': 1.0, 'default': 1.0, 'section': 'Numbers', 'unit': 'dB'},
      {'id': 'fx.param.bins', 'label': 'Bins', 'kind': 'u32', 'value': 256, 'default': 256, 'min': 0, 'section': 'Numbers'},
      {'id': 'fx.param.seed', 'label': 'Seed', 'kind': 'u32', 'value': 1337, 'default': 0, 'section': 'Numbers', 'actions': ['Reroll']},
      {'id': 'fx.param.mode', 'label': 'Mode', 'kind': 'enum', 'choices': ['Add', 'Mix', 'Screen'], 'value': 1, 'default': 0, 'section': 'Choices'},
      {'id': 'fx.param.pattern', 'label': 'Pattern', 'kind': 'enum', 'choices': ['Dots', 'Lines', 'Grid', 'Checks', 'Waves', 'Noise', 'Rings', 'Hex'], 'value': 4, 'default': 0, 'section': 'Choices'},
      {'id': 'fx.param.center', 'label': 'Center', 'kind': 'vec2', 'value': [.5, .5], 'default': [.5, .5], 'section': 'Vectors'},
      {'id': 'fx.param.tint', 'label': 'Tint', 'kind': 'vec3', 'value': [1.0, .4, .2], 'default': [1.0, 1.0, 1.0], 'section': 'Vectors'},
      {'id': 'fx.param.label', 'label': 'Label', 'kind': 'text', 'value': 'hello', 'default': '', 'section': 'Things'},
      {'id': 'fx.param.matte', 'label': 'Matte', 'layer': true, 'refs': ['Layer 1', 'Layer 2', 'Layer 3'], 'value': 'Layer 2', 'section': 'Things'},
      {'id': 'fx.param.glow', 'label': 'Glow', 'kind': 'color', 'value': '#F5C94A', 'section': 'Things'},
      {'id': 'fx.param.gamma', 'label': 'Gamma', 'kind': 'f32', 'value': 2.2, 'default': 2.2, 'advanced': true},
      {'id': 'fx.param.epsilon', 'label': 'Epsilon', 'kind': 'f32', 'value': 1e-6, 'default': 1e-6, 'advanced': true},
      {'id': 'fx.param.weird', 'label': 'Weird', 'kind': 'mystery', 'value': {'a': 1}, 'advanced': true},
    ];

/// n declared rows of every type, in sections of eight, no heroes declared.
List<Map<String, dynamic>> stress(int n) {
  final out = <Map<String, dynamic>>[];
  for (var i = 0; i < n; i++) {
    final id = 'p.param.p$i';
    final sec = n > 8 ? 'Group ${i ~/ 8 + 1}' : null;
    final base = {'id': id, 'label': 'Param $i', if (sec != null) 'section': sec};
    out.add(switch (i % 8) {
      0 => {...base, 'kind': 'f32', 'value': i * .1, 'default': 0.0},
      1 => {...base, 'kind': 'f32', 'value': .5, 'default': .5, 'min': 0.0, 'max': 1.0},
      2 => {...base, 'kind': 'i32', 'value': i, 'default': 0, 'min': 0, 'max': 1000},
      3 => {...base, 'kind': 'bool', 'value': i.isEven, 'default': false},
      4 => {...base, 'kind': 'enum', 'choices': ['A', 'B', 'C'], 'value': 0, 'default': 0},
      5 => {...base, 'kind': 'vec2', 'value': [0.0, 1.0], 'default': [0.0, 1.0]},
      6 => {...base, 'kind': 'text', 'value': 'text $i'},
      _ => {...base, 'kind': 'f32', 'value': -i * 100.0, 'default': 0.0, 'min': -1e9, 'max': 1e9},
    });
  }
  return out;
}

/// Phase 2: rows whose meaning is declared. Same body, a little more behaviour. Nothing here is guessed from a name.
List<Map<String, dynamic>> nativeLike() => [
      {'id': 'l.param.rotation', 'label': 'Rotation', 'kind': 'f32', 'value': 32.0, 'default': 0.0, 'subtype': 'ANGLE', 'section': 'Semantic values', 'animated': true, 'keyedNow': true},
      {'id': 'l.param.count', 'label': 'Count', 'kind': 'f32', 'value': 12.0, 'default': 1.0, 'subtype': 'COUNT', 'section': 'Semantic values'},
      {'id': 'l.param.seed', 'label': 'Seed', 'kind': 'u32', 'value': 1842, 'default': 0, 'subtype': 'SEED', 'section': 'Semantic values'},
      {'id': 'l.param.opacity', 'label': 'Opacity', 'kind': 'f32', 'value': .72, 'default': 1.0, 'min': 0.0, 'max': 1.0, 'subtype': 'OPACITY', 'section': 'Semantic values'},
      {'id': 'l.param.position_x', 'label': 'Position X', 'kind': 'f32', 'value': 320.0, 'default': 0.0, 'unit': 'px', 'character': 'position', 'group': 'position', 'route': 'Depth', 'section': 'Semantic values'},
      {'id': 'l.param.position_y', 'label': 'Position Y', 'kind': 'f32', 'value': 180.0, 'default': 0.0, 'unit': 'px', 'character': 'position', 'group': 'position', 'section': 'Semantic values'},
      {'id': 'l.param.scale_x', 'label': 'Scale X', 'kind': 'f32', 'value': 100.0, 'default': 100.0, 'unit': '%', 'character': 'scale', 'group': 'scale', 'section': 'Semantic values'},
      {'id': 'l.param.scale_y', 'label': 'Scale Y', 'kind': 'f32', 'value': 100.0, 'default': 100.0, 'unit': '%', 'character': 'scale', 'group': 'scale', 'section': 'Semantic values'},
      {'id': 'l.param.fill', 'label': 'Color', 'kind': 'color', 'value': '#F5C94A', 'section': 'Routes'},
      {'id': 'l.param.font', 'label': 'Font', 'kind': 'font', 'value': 'Inter', 'section': 'Routes'},
      {'id': 'l.param.blend', 'label': 'Blend', 'kind': 'blend', 'value': 'Multiply', 'section': 'Routes'},
      {'id': 'l.param.ease', 'label': 'Ease', 'kind': 'ease', 'value': 'Ease In Out', 'section': 'Routes'},
      // names alone change nothing: these say nothing, so they are plain Values
      {'id': 'l.param.angle', 'label': 'Angle', 'kind': 'f32', 'value': 45.5, 'default': 0.0, 'section': 'Named, not declared'},
      {'id': 'l.param.opacity_boost', 'label': 'Opacity boost', 'kind': 'f32', 'value': 250.0, 'default': 10.0, 'min': -50.0, 'max': 500.0, 'subtype': 'OPACITY', 'section': 'Named, not declared'},
    ];
