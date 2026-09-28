import 'dart:math' as math;
import 'dart:ui' as ui show Vertices, VertexMode;

import 'package:flutter/widgets.dart';

import '../../foundation/hsv_triangle.dart';
import '../glyphs.dart';
import 'classify.dart';
import 'common.dart';
import 'search.dart';
import 'shell.dart';
import '../../session/editor_session.dart';
import 'native_visual_sample.dart';

typedef Sw = (String, int, String); // name, argb, class

String hexText(Color c) =>
    '#${(c.toARGB32() & 0xFFFFFF).toRadixString(16).padLeft(6, '0').toUpperCase()}';

const _pal = <String, List<(String, int)>>{
  'Used Here': [
    ('Pink', 0xFFF27AB6),
    ('Orange', 0xFFF69260),
    ('Amber', 0xFFFFC83D),
    ('Green', 0xFF4ADE80),
    ('Teal', 0xFF3B9DB8),
    ('Blue', 0xFF5B7CFF),
    ('Violet', 0xFFA855F7),
  ],
  'Saved': [
    ('Slate', 0xFF66707A),
    ('Silver', 0xFF9AA0A6),
    ('Red', 0xFFFF4B3E),
    ('Tangerine', 0xFFFF9B1F),
    ('Lemon', 0xFFFFE03D),
    ('Cyan', 0xFF38D6E8),
    ('Cobalt', 0xFF2E6BFF),
  ],
  'Starter': [
    ('White', 0xFFFFFFFF),
    ('Grey', 0xFFB4B7BC),
    ('Charcoal', 0xFF4A4F57),
    ('Steel', 0xFF5C7F92),
    ('Sky', 0xFF37C2E0),
    ('Indigo', 0xFF6366F1),
    ('Rose pink', 0xFFFF6DB0),
    ('Hot pink', 0xFFFF2E7E),
    ('Coral', 0xFFFF7A4D),
    ('Gold', 0xFFFFC42E),
    ('Lime', 0xFFB5E82F),
    ('Aqua', 0xFF19D2C4),
    ('Blue', 0xFF1C73FF),
    ('Purple', 0xFF8A3FE6),
  ],
  'Seasonal': [
    ('Snow', 0xFFEAF2FA),
    ('Frost', 0xFFB9D7EC),
    ('Spring', 0xFFA8E6A1),
    ('Bloom', 0xFFFFB3C7),
    ('Summer', 0xFFFFD84A),
    ('Sea', 0xFF29B6D6),
    ('Autumn', 0xFFD9752B),
    ('Ember', 0xFF8C2F1F),
  ],
  'Nature': [
    ('Moss', 0xFF5B7A3A),
    ('Fern', 0xFF3F8F4E),
    ('Clay', 0xFFB5764A),
    ('Sand', 0xFFE3CFA0),
    ('Stone', 0xFF8C8A84),
    ('Dusk', 0xFF5A4E7C),
    ('Lake', 0xFF2F6F8F),
    ('Bark', 0xFF4A3524),
  ],
  'Material': [
    ('Red 500', 0xFFF44336),
    ('Pink 500', 0xFFE91E63),
    ('Purple 500', 0xFF9C27B0),
    ('Indigo 500', 0xFF3F51B5),
    ('Cyan 500', 0xFF00BCD4),
    ('Green 500', 0xFF4CAF50),
    ('Amber 500', 0xFFFFC107),
    ('Brown 500', 0xFF795548),
  ],
  'Neon': [
    ('Neon pink', 0xFFFF2BD6),
    ('Neon lime', 0xFFB6FF00),
    ('Neon cyan', 0xFF00F0FF),
    ('Neon orange', 0xFFFF6A00),
    ('Neon violet', 0xFF9D4DFF),
    ('Neon yellow', 0xFFF9FF2B),
    ('Neon red', 0xFFFF1F4B),
    ('Neon blue', 0xFF2B6BFF),
  ],
  'Pastel': [
    ('Pastel pink', 0xFFFFC9DE),
    ('Pastel peach', 0xFFFFDAB8),
    ('Pastel butter', 0xFFFFF3B0),
    ('Pastel mint', 0xFFC7F2D4),
    ('Pastel sky', 0xFFC2E4FF),
    ('Pastel lilac', 0xFFDCCBFF),
    ('Pastel sage', 0xFFCFE3C3),
    ('Pastel cloud', 0xFFE8EDF5),
  ],
  'Monochrome': [
    ('Black', 0xFF000000),
    ('Ink', 0xFF141419),
    ('Graphite', 0xFF2B2E34),
    ('Iron', 0xFF4A4E56),
    ('Ash', 0xFF787C84),
    ('Fog', 0xFFA9ADB4),
    ('Pearl', 0xFFD7D9DD),
    ('Paper', 0xFFF7F7F5),
  ],
};

List<Sw> colorsBase() => [
  for (final e in _pal.entries)
    for (final s in e.value) (s.$1, s.$2, e.key),
];

/// Growth fixture: a thousand named swatches across many palettes. Not real content.
List<Sw> colorsStress() {
  final out = <Sw>[...colorsBase()];
  const names = [
    'Studio',
    'Retro',
    'Ocean',
    'Forest',
    'Desert',
    'City',
    'Candy',
    'Metal',
    'Ink',
    'Aurora',
    'Ember',
    'Glacier',
    'Orchard',
    'Circuit',
    'Velvet',
    'Paper',
  ];
  for (var p = 0; p < names.length; p++) {
    for (var k = 0; k < 40; k++) {
      final h = (p * 41 + k * 9) % 360.0;
      out.add((
        '${names[p]} ${k + 1}',
        HSLColor.fromAHSL(
          1,
          h,
          .35 + (k % 5) * .13,
          .3 + (k % 7) * .08,
        ).toColor().toARGB32(),
        names[p],
      ));
    }
  }
  return out;
}

String _hex(int v) =>
    '#${(v & 0xFFFFFF).toRadixString(16).padLeft(6, '0').toUpperCase()}';

const _grads = <(String, List<int>)>[
  ('Candy', [0xFFFF5FA8, 0xFFB65CFF]),
  ('Sky', [0xFF5BC0FF, 0xFF3D5BFF]),
  ('Sunset', [0xFFFFB13D, 0xFFFF3D6E]),
  ('Ocean', [0xFF3B6BFF, 0xFF7A4DFF]),
  ('Meadow', [0xFF7BE0A0, 0xFFD6C46A]),
  ('Dusk', [0xFF6C4DFF, 0xFFFF6A9E]),
];

List<List<String>> colorGroups(List<Sw> items) {
  final seen = <String>[];
  for (final it in items) {
    if (!seen.contains(it.$3)) seen.add(it.$3);
  }
  return [
    ['All', ...seen, 'Gradients'],
    ['Wheel'],
  ];
}

class ColorsPanel extends StatefulWidget {
  const ColorsPanel({
    super.key,
    this.search,
    this.classify,
    this.items,
    this.gradients,
    this.onSwatch,
    this.onSwatchMenu,
    this.onGradient,
    this.editor,
    this.current,
    this.controller,
  });
  final SearchCapability? search;
  final ClassifyCapability? classify;
  final List<Sw>? items;
  final List<Map<String, dynamic>>? gradients;
  final ValueChanged<Sw>? onSwatch;

  /// A swatch's right-click, at the pointer.
  final void Function(Sw swatch, Offset at)? onSwatchMenu;
  final ValueChanged<Map<String, dynamic>>? onGradient;
  /// The live colour editor at the wheel's side; without one the reference instrument is drawn.
  final Widget Function(double wheel)? editor;

  /// The colour being edited, for the compact readouts (filtering, narrow).
  final Color? current;
  final EditorSession? controller;
  @override
  State<ColorsPanel> createState() => _ColorsPanelState();
}

class _ColorsPanelState extends State<ColorsPanel>
    with WithDiscovery<ColorsPanel> {
  @override
  SearchCapability? get injectedSearch => widget.search;
  @override
  ClassifyCapability? get injectedClassify => widget.classify;
  List<Sw> get items => widget.items ?? colorsBase();
  List<List<String>> get groups => colorGroups(items);

  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: discovery,
    builder: (_, __) {
      final sel = classify.selected;
      final wheelOnly = sel == 'Wheel', gradients = sel == 'Gradients';
      final pool = (wheelOnly || gradients)
          ? <Sw>[]
          : classify.apply(items, (e) => e.$3);
      final shown = search.apply(pool, (e) => [e.$1, _hex(e.$2), e.$3]);
      final filtering = search.active || (classify.filtering && !wheelOnly);
      final sections = <String, List<Sw>>{};
      for (final it in shown) {
        (sections[it.$3] ??= []).add(it);
      }
      final n = shown.length;
      final hasGradients = widget.gradients == null
          ? true
          : widget.gradients!.isNotEmpty;
      return PanelShell(
        title: 'Colors',
        icon: const GlyphBox(HG.color, size: 22, color: Color(0xFFF2F2F4)),
        search: search,
        classify: classify,
        groups: groups,
        hint: 'Search colors',
        count: '$n',
        wide: (c, s) => _wide(
          s,
          sections,
          n,
          wheelOnly,
          gradients,
          filtering,
          hasGradients,
        ),
        narrow: (c, s) => _narrow(s, shown, wheelOnly, filtering),
        strip: (c, s) => _strip(
          shown.isEmpty
              ? (widget.items == null
                    ? [for (final e in _pal['Used Here']!) ('', e.$2, '')]
                    : const <Sw>[])
              : shown,
          s.height,
        ),
      );
    },
  );

  Widget _wide(
    Size s,
    Map<String, List<Sw>> sections,
    int n,
    bool wheelOnly,
    bool gradients,
    bool filtering,
    bool hasGradients,
  ) {
    const pad = 12.0;
    final wheel = math.min((s.width - pad * 2) * .68, 176.0);
    final full = !filtering || wheelOnly;
    return SingleChildScrollView(
      physics: const ClampingScrollPhysics(),
      padding: const EdgeInsets.fromLTRB(pad, 14, pad, 14),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          if (full)
            (widget.editor?.call(wheel) ?? _Instrument(wheel: wheel))
          else
            _MiniInstrument(color: widget.current),
          if (!wheelOnly && (n > 0 || gradients || !filtering))
            const SizedBox(height: 4),
          if (n == 0 && !gradients && !wheelOnly)
            emptyBody('No colour matches "${search.query}".'),
          for (final e in sections.entries) ...[
            SectionLabel(e.key),
            _Swatches(e.value, 22, onTap: widget.onSwatch, onMenu: widget.onSwatchMenu),
          ],
          if (gradients && !hasGradients) emptyBody('No saved gradients.'),
          if (gradients || (!filtering && !wheelOnly && hasGradients)) ...[
            const SectionLabel('Gradients'),
            _Gradients(
              items: widget.gradients,
              onTap: widget.onGradient,
              controller: widget.controller,
            ),
          ],
        ],
      ),
    );
  }

  Widget _narrow(Size s, List<Sw> shown, bool wheelOnly, bool filtering) {
    final wheel = (s.width - 24).clamp(70.0, 150.0);
    return SingleChildScrollView(
      physics: const ClampingScrollPhysics(),
      padding: const EdgeInsets.all(12),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          if (!filtering || wheelOnly)
            Center(
              child:
                  widget.editor?.call(wheel) ??
                  SizedBox(
                    width: wheel,
                    height: wheel,
                    child: CustomPaint(painter: WheelPainter()),
                  ),
            ),
          if ((!filtering || wheelOnly) && widget.editor == null)
            Padding(
              padding: const EdgeInsets.only(top: 6, bottom: 4),
              child: Center(
                child: Text(
                  '#E8508F',
                  style: mono(10.5, c: const Color(0xFFC0C1C3)),
                ),
              ),
            ),
          if (shown.isEmpty && filtering && !wheelOnly)
            emptyBody('No colour matches.'),
          if (!wheelOnly) ...[
            const SizedBox(height: 4),
            _Swatches(shown, 20, onTap: widget.onSwatch, onMenu: widget.onSwatchMenu),
          ],
        ],
      ),
    );
  }

  Widget _strip(List<Sw> shown, double h) {
    final addControl = widget.onSwatch == null;
    return ListView.builder(
      scrollDirection: Axis.horizontal,
      physics: const ClampingScrollPhysics(),
      padding: const EdgeInsets.all(10),
      itemCount: shown.length + (addControl ? 1 : 0),
      itemBuilder: (_, i) => Padding(
        padding: const EdgeInsets.only(right: 6),
        child: addControl && i == shown.length
            ? SizedBox(
                width: 30,
                child: Center(
                  child: Text('+', style: sans(16, c: kMuted)),
                ),
              )
            : Container(
                width: math.min(h - 20, 38),
                decoration: BoxDecoration(
                  color: Color(shown[i].$2),
                  borderRadius: BorderRadius.circular(3),
                ),
              ),
      ),
    );
  }
}

class _Instrument extends StatelessWidget {
  const _Instrument({required this.wheel});
  final double wheel;
  @override
  Widget build(BuildContext context) => Row(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      Column(
        children: [
          SizedBox(
            width: wheel,
            height: wheel,
            child: CustomPaint(painter: WheelPainter()),
          ),
          const SizedBox(height: 8),
          SizedBox(
            width: wheel,
            child: Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Text('#E8508F', style: mono(11, c: const Color(0xFFD0D1D3))),
                const SizedBox(width: 8),
                const GlyphBox(HG.composite, size: 13, color: kMuted),
              ],
            ),
          ),
        ],
      ),
      const SizedBox(width: 12),
      SizedBox(
        height: wheel,
        child: Row(
          children: [
            SizedBox(
              width: 11,
              height: wheel,
              child: CustomPaint(painter: ColorBar(0)),
            ),
            const SizedBox(width: 9),
            SizedBox(
              width: 11,
              height: wheel,
              child: CustomPaint(painter: ColorBar(1)),
            ),
          ],
        ),
      ),
    ],
  );
}

class _MiniInstrument extends StatelessWidget {
  const _MiniInstrument({this.color});
  final Color? color;
  @override
  Widget build(BuildContext context) => Row(
    children: [
      SizedBox(
        width: 54,
        height: 54,
        child: CustomPaint(
          painter: WheelPainter(
            color == null ? null : HSVColor.fromColor(color!),
          ),
        ),
      ),
      const SizedBox(width: 12),
      Container(
        width: 30,
        height: 30,
        decoration: BoxDecoration(
          color: color ?? const Color(0xFFE8508F),
          borderRadius: BorderRadius.circular(3),
        ),
      ),
      const SizedBox(width: 10),
      Text(
        color == null ? '#E8508F' : hexText(color!),
        style: mono(11, c: const Color(0xFFD0D1D3)),
      ),
    ],
  );
}

/// A vertical bar: kind 0 runs from black up to the colour (its value), kind 1 from clear up to it over a checker
/// (its alpha); the handle sits at [at] from the top.
class ColorBar extends CustomPainter {
  ColorBar(this.kind, [this.color = const Color(0xFFE8508F), double? at])
    : at = at ?? (kind == 0 ? .1 : .22);
  final int kind;
  final Color color;
  final double at;
  @override
  void paint(Canvas c, Size s) {
    final r = RRect.fromRectAndRadius(
      Offset.zero & s,
      const Radius.circular(3),
    );
    if (kind == 1) {
      c.save();
      c.clipRRect(r);
      for (var y = 0.0; y < s.height; y += 4) {
        for (var x = 0.0; x < s.width; x += 4) {
          c.drawRect(
            Rect.fromLTWH(x, y, 4, 4),
            Paint()
              ..color = ((x + y) / 4).floor().isEven
                  ? const Color(0xFF3A3A3D)
                  : const Color(0xFF2A2A2D),
          );
        }
      }
      c.restore();
    }
    final opaque = color.withValues(alpha: 1);
    final colors = kind == 0
        ? [const Color(0xFF000000), opaque]
        : [opaque.withValues(alpha: 0), opaque];
    c.drawRRect(
      r,
      Paint()
        ..shader = LinearGradient(
          begin: Alignment.bottomCenter,
          end: Alignment.topCenter,
          colors: colors,
        ).createShader(Offset.zero & s),
    );
    final o = Offset(s.width / 2, s.height * at);
    c.drawCircle(o, 5.5, Paint()..color = const Color(0xFFF2F2F4));
    c.drawCircle(
      o,
      5.5,
      Paint()
        ..color = const Color(0xFF101012)
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.4,
    );
  }

  @override
  bool shouldRepaint(ColorBar o) =>
      o.kind != kind || o.color != color || o.at != at;
}

/// Hue on the ring, saturation (left to right) and value (bottom to top) in the square; handles where [hsv] sits.
/// Without a colour it draws the reference's #E8508F.
class WheelPainter extends CustomPainter {
  WheelPainter([this.hsv, this.triangle = false]);
  final HSVColor? hsv;

  /// The inner area is the colour triangle (hue, white and black corners) instead of the square.
  final bool triangle;

  /// The triangle's corners (hue, white, black) inside the ring, the hue corner toward the hue on the ring.
  static List<Offset> triangleAt(Size size, double hue) {
    final s = size.shortestSide;
    final ctr = Offset(size.width / 2, size.height / 2);
    final r = s / 2 - 1 - s * .105 - s * .02;
    Offset corner(double degrees) => ctr + Offset(math.cos(degrees * math.pi / 180), math.sin(degrees * math.pi / 180)) * r;
    return [corner(hue), corner(hue + 120), corner(hue + 240)];
  }
  static const reference = HSVColor.fromAHSV(1, 340, .65, .91);

  /// The ring's inner radius and the square, for a wheel of [size]: what the paint uses and a press reads.
  static (Offset centre, double rInner, double rOuter, Rect square) geometry(Size size) {
    final s = size.shortestSide;
    final ctr = Offset(size.width / 2, size.height / 2);
    final ringW = s * .105;
    final rOut = s / 2 - 1;
    final side = (rOut - ringW) * 1.32;
    return (ctr, rOut - ringW, rOut, Rect.fromCenter(center: ctr, width: side, height: side));
  }

  @override
  void paint(Canvas c, Size sz) {
    final h = hsv ?? reference;
    final s = sz.shortestSide;
    final ctr = Offset(sz.width / 2, sz.height / 2);
    final ringW = s * .105;
    final rOut = s / 2 - 1, rMid = rOut - ringW / 2;
    final rainbow = SweepGradient(
      colors: [
        for (var h = 0; h <= 360; h += 30)
          HSVColor.fromAHSV(1, h.toDouble() % 360, .85, 1).toColor(),
      ],
    );
    c.drawCircle(
      ctr,
      rMid,
      Paint()
        ..shader = rainbow.createShader(
          Rect.fromCircle(center: ctr, radius: rOut),
        )
        ..style = PaintingStyle.stroke
        ..strokeWidth = ringW,
    );
    final fill = hsv == null
        ? const Color(0xFFE8508F)
        : h.toColor().withValues(alpha: 1);
    final hp =
        ctr +
        Offset(math.cos(h.hue * math.pi / 180), math.sin(h.hue * math.pi / 180)) *
            rMid;
    if (triangle) {
      final t = triangleAt(sz, h.hue);
      c.drawVertices(
        ui.Vertices(ui.VertexMode.triangles, t, colors: [HSVColor.fromAHSV(1, h.hue, 1, 1).toColor(), const Color(0xFFFFFFFF), const Color(0xFF000000)]),
        BlendMode.srcOver,
        Paint(),
      );
      _handle(c, hp, s * .04 + 3, fill);
      _handle(c, triangleHandle(t, h), s * .035 + 2.5, fill);
      return;
    }
    final side = (rOut - ringW) * 1.32;
    final sq = Rect.fromCenter(center: ctr, width: side, height: side);
    final rr = RRect.fromRectAndRadius(sq, Radius.circular(s * .02));
    c.drawRRect(rr, Paint()..color = HSVColor.fromAHSV(1, h.hue, 1, 1).toColor());
    c.drawRRect(
      rr,
      Paint()
        ..shader = const LinearGradient(
          colors: [Color(0xFFFFFFFF), Color(0x00FFFFFF)],
        ).createShader(sq),
    );
    c.drawRRect(
      rr,
      Paint()
        ..shader = const LinearGradient(
          begin: Alignment.topCenter,
          end: Alignment.bottomCenter,
          colors: [Color(0x00000000), Color(0xFF000000)],
        ).createShader(sq),
    );
    _handle(c, hp, s * .04 + 3, fill);
    _handle(
      c,
      Offset(sq.left + sq.width * h.saturation, sq.top + sq.height * (1 - h.value)),
      s * .035 + 2.5,
      fill,
    );
  }

  void _handle(Canvas c, Offset o, double r, Color fill) {
    c.drawCircle(o, r, Paint()..color = fill);
    c.drawCircle(
      o,
      r,
      Paint()
        ..color = const Color(0xFFF6F6F8)
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2,
    );
  }

  @override
  bool shouldRepaint(WheelPainter o) => o.hsv != hsv || o.triangle != triangle;
}

class _Swatches extends StatelessWidget {
  const _Swatches(this.items, this.size, {this.onTap, this.onMenu});
  final List<Sw> items;
  final double size;
  final ValueChanged<Sw>? onTap;
  final void Function(Sw swatch, Offset at)? onMenu;
  @override
  Widget build(BuildContext context) => Wrap(
    spacing: 5,
    runSpacing: 5,
    children: [
      for (final v in items.take(600))
        GestureDetector(
          key: ValueKey('hf-color:${v.$1}'),
          behavior: HitTestBehavior.opaque,
          onTap: onTap == null ? null : () => onTap!(v),
          onSecondaryTapDown: onMenu == null ? null : (e) => onMenu!(v, e.globalPosition),
          child: Container(
            width: size,
            height: size,
            decoration: BoxDecoration(
              color: Color(v.$2),
              borderRadius: BorderRadius.circular(3),
            ),
          ),
        ),
      if (items.length > 600)
        SizedBox(
          width: size * 2,
          height: size,
          child: Center(child: Text('+${items.length - 600}', style: mono(10))),
        ),
    ],
  );
}

class _Gradients extends StatelessWidget {
  const _Gradients({this.items, this.onTap, this.controller});
  final List<Map<String, dynamic>>? items;
  final ValueChanged<Map<String, dynamic>>? onTap;
  final EditorSession? controller;

  List<Color> _colors(List<dynamic> stops) => [
    for (final stop in stops)
      if (stop is List && stop.length >= 3)
        Color.fromARGB(
          ((stop.length > 3 ? (stop[3] as num).toDouble() : 1.0) * 255)
              .round()
              .clamp(0, 255),
          ((stop[0] as num).toDouble() * 255).round().clamp(0, 255),
          ((stop[1] as num).toDouble() * 255).round().clamp(0, 255),
          ((stop[2] as num).toDouble() * 255).round().clamp(0, 255),
        ),
  ];

  Widget _tile(Map<String, dynamic> item) {
    final placeholder = Container(
      width: 78,
      height: 15,
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(3),
        gradient: LinearGradient(
          colors: _colors(item['stops'] as List? ?? const []),
        ),
      ),
    );
    final c = controller;
    if (c == null || c.state['visualSamples'] != true) return placeholder;
    return NativeVisualSample(
      controller: c,
      request: {
        'kind': 'gradient',
        'stops': item['stops'],
        if (item['blend'] != null) 'blend': item['blend'],
      },
      fit: BoxFit.cover,
      placeholder: placeholder,
    );
  }

  @override
  Widget build(BuildContext context) => Wrap(
    spacing: 8,
    runSpacing: 8,
    children: [
      if (items == null)
        for (final g in _grads)
          Container(
            width: 78,
            height: 15,
            decoration: BoxDecoration(
              borderRadius: BorderRadius.circular(3),
              gradient: LinearGradient(
                colors: [for (final c in g.$2) Color(c)],
              ),
            ),
          ),
      if (items != null)
        for (final item in items!)
          GestureDetector(
            behavior: HitTestBehavior.opaque,
            onTap: onTap == null ? null : () => onTap!(item),
            child: _tile(item),
          ),
    ],
  );
}
