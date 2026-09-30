/// An ease kind's one-line meaning, as every Ease desk says it under the curve's name.
String curveMeaning(String kind) => switch (kind) {
  'Hold' => 'Wait, then change in one jump.',
  'Linear' => 'Move at a constant pace.',
  'Bezier' => 'Shape the start and finish.',
  'Bounce' => 'Reach the end, then rebound.',
  'Elastic' => 'Pass the end and spring back.',
  'Cyclic' => 'Repeat a wave as time advances.',
  'Random' => 'Vary the pace irregularly.',
  'Steps' => 'Move through distinct levels.',
  'ElasticSteps' => 'Spring into each new level.',
  _ => 'Preview the change from start to finish.',
};
