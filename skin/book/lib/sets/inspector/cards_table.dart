// Cards handled by hand: the window's inspector (cards of value tiles, as today) with the cards free to pick up.
// One table: the stage (a star ring) on the left, the inspector's cards on the right, the hand (Desk's place) under them.
// What the hand does (each a use case in cards_set.dart): a behaviour card laid on a value moves it (shown while it only hovers),
// pulled off again; a card flipped (front: its main tiles, back: every number); cards reordered (the order is what is done first);
// a card turned face down (off); same behaviour on it again = stronger, another = swapped (the old one back to the hand).
// A mock: not joined to the inside. Sizes and colours are the inspector's own (Pop).
import 'dart:math' as math;

import 'package:flutter/scheduler.dart';
import 'package:flutter/widgets.dart';

import '../../tokens.dart';
import 'inspector_parts.dart' show Pop;

// ---------------------------------------------------------------- the model

class Val {
  Val(this.id, this.label, this.v, this.lo, this.hi, {this.unit = '', this.dec = 0, this.perCopy = true});
  final String id, label, unit;
  double v;
  final double lo, hi;
  final int dec;
  final bool perCopy; // false: one value for the whole ring (a count)
}

class CardM {
  CardM(this.name, this.tone, this.front, {this.back = const []});
  final String name;
  final Color tone;
  final List<Val> front;
  final List<String> back; // the rest of today's values, shown on the back as numbers
  bool flipped = false, down = false;
}

/// A behaviour card (Cavalry's behaviours): what it gives a value, copy by copy and moment by moment, in -1..1.
enum Beh {
  sway('揺れ', Color(0xFF7DD5B1)),
  stagger('ずれ', Color(0xFF4781E5)),
  noise('ノイズ', Color(0xFFE974AB)),
  random('ランダム', Color(0xFFEFCB4E));

  const Beh(this.word, this.c);
  final String word;
  final Color c;

  double at(double t, int i, int n) => switch (this) {
    Beh.sway => math.sin(2 * math.pi * (t * .55 + i / math.max(n, 1))),
    Beh.stagger => n <= 1 ? 0 : i / (n - 1) * 2 - 1,
    Beh.noise => _noise(t * .7 + i * 3.17),
    Beh.random => _hash(i * 7.0 + 1) * 2 - 1,
  };
}

double _hash(double x) {
  final s = math.sin(x * 12.9898) * 43758.5453;
  return s - s.floorToDouble();
}

double _noise(double x) {
  final i = x.floorToDouble(), f = x - i, u = f * f * (3 - 2 * f);
  return (_hash(i) * (1 - u) + _hash(i + 1) * u) * 2 - 1;
}

class Drive {
  Drive(this.beh, [this.power = 1]);
  final Beh beh;
  int power; // the same card laid again: stronger
}

class CardTable extends ChangeNotifier {
  CardTable(this.cards, {this.hand = const [Beh.sway, Beh.stagger, Beh.noise, Beh.random]});
  final List<CardM> cards;
  final List<Beh> hand;
  final drives = <String, Drive>{};
  (String, Beh)? preview; // a card hovering over a value: shown before it is let go
  String? flash; // the value a card was just laid on (its tile lights once)

  Val val(String id) => [for (final c in cards) ...c.front].firstWhere((v) => v.id == id);
  CardM? cardOf(String id) {
    for (final c in cards) {
      if (c.front.any((v) => v.id == id)) return c;
    }
    return null;
  }

  bool has(String id) => cards.any((c) => c.front.any((v) => v.id == id));

  /// The value at a moment for one copy: its number, moved by what drives it (and by a hovering card), nothing if its card is down.
  double at(String id, double t, int i, int n, {double fallback = 0}) {
    if (!has(id)) return fallback;
    final v = val(id), card = cardOf(id)!;
    if (card.down) return fallback;
    var x = v.v;
    final d = drives[id], p = preview;
    final ii = v.perCopy ? i : 0;
    if (d != null) x += d.beh.at(t, ii, n) * (v.hi - v.lo) * .22 * d.power;
    if (p != null && p.$1 == id && (d == null || d.beh != p.$2)) x += p.$2.at(t, ii, n) * (v.hi - v.lo) * .22;
    return x.clamp(v.lo, v.hi);
  }

  void set(Val v, double x) {
    v.v = x.clamp(v.lo, v.hi);
    notifyListeners();
  }

  void hover((String, Beh)? p) {
    if (p == preview) return;
    preview = p;
    notifyListeners();
  }

  /// A behaviour card let go on a value: the same one again is stronger, another takes its place.
  void lay(String id, Beh b) {
    final d = drives[id];
    if (d != null && d.beh == b) {
      d.power = math.min(d.power + 1, 3);
    } else {
      drives[id] = Drive(b);
    }
    preview = null;
    flash = id;
    notifyListeners();
  }

  void lift(String id) {
    drives.remove(id);
    notifyListeners();
  }

  void flip(CardM c) {
    c.flipped = !c.flipped;
    notifyListeners();
  }

  void turnDown(CardM c) {
    c.down = !c.down;
    notifyListeners();
  }

  void move(int from, int to) {
    final c = cards.removeAt(from);
    cards.insert(to, c);
    notifyListeners();
  }

  /// Which comes first, the copies or the wave: wave first = the ring waves as one; copies first = each copy waves on its own.
  bool get copiesFirst {
    final a = cards.indexWhere((c) => c.name == '複製'), b = cards.indexWhere((c) => c.name == '波');
    return a < 0 || b < 0 || a < b;
  }
}

/// The ring's cards as the window shows a shape with copies today (labels in plain Japanese).
CardTable starRing({bool wave = true}) => CardTable([
  CardM('見た目', Pop.toneLayer, [
    Val('size', '大きさ', 44, 10, 120, unit: 'px'),
    Val('turn', '回転', 0, 0, 360, unit: '°'),
    Val('opacity', '不透明', 100, 0, 100, unit: '%'),
  ], back: const ['位置 X', '位置 Y', '位置 Z', '基準点 X', '基準点 Y', '拡大 X', '拡大 Y', '回転 X', '回転 Y', '空間', '親']),
  CardM('複製', Pop.tonePlace, [
    Val('count', '数', 12, 3, 48, perCopy: false),
    Val('radius', '半径', 180, 20, 300, unit: 'px'),
    Val('spin', '1つずつ回す', 30, 0, 180, unit: '°'),
  ], back: const ['列', '間隔 X', '間隔 Y', '回転 X', '回転 Y', '大きさ', '幅', '奥行き', '高さ', '遅れ', '遅れの起点', '色の並び', '色の流れ']),
  if (wave)
    CardM('波', Pop.tonePath, [
      Val('waveH', '高さ', 0, 0, 90, unit: 'px'),
      Val('waveS', '速さ', 1, 0, 4, dec: 2),
    ], back: const ['波 X', '波 Z', '波で回す', '波で大きく', '位相', 'ゆらぎ', 'ゆらぎの細かさ']),
  CardM('色', Pop.toneFill, [
    Val('hue', '色相', 72, 0, 360, unit: '°'),
    Val('light', '明るさ', 62, 20, 90, unit: '%'),
  ], back: const ['線', '線の太さ', 'グラデーション', '角度']),
]);

String sayVal(Val v, double x) => '${x.toStringAsFixed(v.dec)}${v.unit}';

// ---------------------------------------------------------------- the table

class CardsTable extends StatefulWidget {
  const CardsTable(this.t, {super.key, this.tag = ''});
  final CardTable t;
  final String tag;
  @override
  State<CardsTable> createState() => _CardsTableState();
}

class _CardsTableState extends State<CardsTable> with SingleTickerProviderStateMixin {
  late final Ticker tk;
  final time = ValueNotifier<double>(0);
  Duration last = Duration.zero;

  @override
  void initState() {
    super.initState();
    tk = createTicker((d) {
      time.value += (d - last).inMicroseconds / 1e6;
      last = d;
    })..start();
  }

  @override
  void dispose() {
    tk.dispose();
    time.dispose();
    super.dispose();
  }

  // (cards in the air and the reordering need an overlay of their own: the table works in any host)
  late final _entry = OverlayEntry(builder: _table);
  @override
  Widget build(BuildContext context) => _Time(time: time, child: Overlay(initialEntries: [_entry]));

  Widget _table(BuildContext context) => ListenableBuilder(
      listenable: widget.t,
      builder: (context, _) => ColoredBox(
        color: N.g07,
        child: Row(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          Expanded(child: _Stage(widget.t, widget.tag)),
          Container(
            width: 360,
            decoration: BoxDecoration(color: Grey.g10, border: Border(left: BorderSide(color: Grey.g20))),
            child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
              Expanded(child: _Inspector(widget.t)),
              _Hand(widget.t),
            ]),
          ),
        ]),
      ),
  );
}

class _Time extends InheritedWidget {
  const _Time({required this.time, required super.child});
  final ValueNotifier<double> time;
  static ValueNotifier<double> of(BuildContext c) => c.getInheritedWidgetOfExactType<_Time>()!.time;
  @override
  bool updateShouldNotify(_Time old) => false;
}

// ---------------------------------------------------------------- the stage

class _Stage extends StatelessWidget {
  const _Stage(this.t, this.tag);
  final CardTable t;
  final String tag;
  @override
  Widget build(BuildContext context) => ClipRect(
    child: CustomPaint(
      painter: _Ring(t, _Time.of(context)),
      child: Align(
        alignment: Alignment.topLeft,
        child: Padding(padding: const EdgeInsets.all(14), child: Text(tag, style: T.micro(N.g76))),
      ),
    ),
  );
}

class _Ring extends CustomPainter {
  _Ring(this.t, this.time) : super(repaint: Listenable.merge([t, time]));
  final CardTable t;
  final ValueNotifier<double> time;
  @override
  void paint(Canvas c, Size size) {
    final s = time.value, m = size.center(Offset.zero), k = math.min(size.width, size.height) / 720;
    final n = t.at('count', s, 0, 1, fallback: 1).round();
    for (var i = 0; i < n; i++) {
      double v(String id, double f) => t.at(id, s, i, n, fallback: f);
      final waveH = v('waveH', 0), waveS = v('waveS', 1);
      final phase = t.copiesFirst ? i / n * 2 * math.pi : 0.0; // (copies first: each copy waves on its own)
      final a = 2 * math.pi * i / n + v('turn', 0) * math.pi / 180 - math.pi / 2;
      final r = v('radius', 0);
      final p = m + (Offset.fromDirection(a, r) + Offset(0, -waveH * math.sin(s * waveS * 2.4 - phase))) * k;
      final col = HSLColor.fromAHSL(v('opacity', 100) / 100, v('hue', 72) % 360, .62, v('light', 62) / 100).toColor();
      c.drawPath(_star(p, v('size', 44) * k / 2, v('turn', 0) * math.pi / 180 + i * v('spin', 0) * math.pi / 180), Paint()..color = col);
    }
  }

  Path _star(Offset o, double r, double turn) {
    final p = Path();
    for (var j = 0; j < 10; j++) {
      final q = o + Offset.fromDirection(turn - math.pi / 2 + j * math.pi / 5, j.isEven ? r : r * .45);
      j == 0 ? p.moveTo(q.dx, q.dy) : p.lineTo(q.dx, q.dy);
    }
    return p..close();
  }

  @override
  bool shouldRepaint(_Ring o) => true;
}

// ---------------------------------------------------------------- the inspector

class _Inspector extends StatelessWidget {
  const _Inspector(this.t);
  final CardTable t;
  @override
  Widget build(BuildContext context) => ReorderableList(
    padding: const EdgeInsets.all(Pop.inset),
    itemCount: t.cards.length,
    onReorderItem: t.move,
    proxyDecorator: (child, i, anim) => AnimatedBuilder(
      animation: anim,
      builder: (context, c) => Transform.rotate(
        angle: -.02 * anim.value,
        child: DecoratedBox(
          decoration: BoxDecoration(boxShadow: [BoxShadow(color: const Color(0x99000000), blurRadius: 18 * anim.value, offset: Offset(0, 8 * anim.value))]),
          child: c,
        ),
      ),
      child: child,
    ),
    itemBuilder: (context, i) => Padding(
      key: ValueKey(t.cards[i].name),
      padding: const EdgeInsets.only(bottom: Pop.gap * 3),
      child: _Card(t, t.cards[i], i),
    ),
  );
}

class _Card extends StatelessWidget {
  const _Card(this.t, this.c, this.i);
  final CardTable t;
  final CardM c;
  final int i;
  @override
  Widget build(BuildContext context) {
    final head = Container(
      height: 30,
      padding: const EdgeInsets.symmetric(horizontal: Pop.inset),
      child: Row(children: [
        ReorderableDragStartListener(
          index: i,
          child: Row(children: [
            Container(width: 14, height: 14, decoration: BoxDecoration(color: c.down ? Grey.g26 : c.tone, borderRadius: BorderRadius.circular(3))),
            const SizedBox(width: 8),
            Text(c.name, style: T.title(c.down ? Grey.g44 : Grey.g95)),
          ]),
        ),
        const Spacer(),
        _HeadButton(c.flipped ? '表' : '裏', on: c.flipped, onTap: () => t.flip(c)),
        const SizedBox(width: 4),
        _HeadButton('伏せ', on: c.down, onTap: () => t.turnDown(c)),
      ]),
    );
    final body = c.flipped
        ? Column(children: [
            for (final v in c.front) _BackRow(v.label, sayVal(v, v.v), driven: t.drives[v.id]),
            for (final b in c.back) _BackRow(b, '0'),
          ])
        : LayoutBuilder(
            builder: (context, box) {
              final w = (box.maxWidth - Pop.tileGap * 2) / 3;
              return Wrap(spacing: Pop.tileGap, runSpacing: Pop.tileGap, children: [for (final v in c.front) SizedBox(width: w, child: _Tile(t, v))]);
            },
          );
    return AnimatedOpacity(
      duration: Mo.dur,
      opacity: c.down ? .45 : 1,
      child: Container(
        color: Pop.card,
        child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          head,
          Padding(
            padding: const EdgeInsets.fromLTRB(Pop.inset, 0, Pop.inset, Pop.insetY + 2),
            child: c.down ? const SizedBox(height: 4) : AnimatedSwitcher(duration: Mo.dur * 2, transitionBuilder: _flipIn, layoutBuilder: (cur, prev) => Stack(alignment: Alignment.topLeft, children: [...prev, ?cur]), child: KeyedSubtree(key: ValueKey(c.flipped), child: body)),
          ),
        ]),
      ),
    );
  }

  static Widget _flipIn(Widget child, Animation<double> a) => AnimatedBuilder(
    animation: a,
    builder: (context, ch) => Transform(alignment: Alignment.center, transform: Matrix4.identity()..setEntry(3, 2, .002)..rotateX((1 - a.value) * math.pi / 2), child: ch),
    child: child,
  );
}

class _HeadButton extends StatelessWidget {
  const _HeadButton(this.word, {this.on = false, this.onTap});
  final String word;
  final bool on;
  final VoidCallback? onTap;
  @override
  Widget build(BuildContext context) => GestureDetector(
    onTap: onTap,
    child: Container(
      padding: const EdgeInsets.symmetric(horizontal: 7, vertical: 3),
      decoration: BoxDecoration(color: on ? Grey.g26 : Pop.well, borderRadius: BorderRadius.circular(9)),
      child: Text(word, style: T.micro(on ? Grey.g95 : Grey.g63)),
    ),
  );
}

class _BackRow extends StatelessWidget {
  const _BackRow(this.label, this.value, {this.driven});
  final String label, value;
  final Drive? driven;
  @override
  Widget build(BuildContext context) => SizedBox(
    height: 19,
    child: Row(children: [
      Text(label, style: T.label(Grey.g63)),
      if (driven != null) ...[const SizedBox(width: 6), _Chip(driven!.beh, power: driven!.power, small: true)],
      const Spacer(),
      Text(value, style: T.value(Grey.g91).copyWith(fontSize: 10)),
    ]),
  );
}

/// One value as today's block (name top-left, number large at the bottom); a behaviour card can be laid on it.
class _Tile extends StatefulWidget {
  const _Tile(this.t, this.v);
  final CardTable t;
  final Val v;
  @override
  State<_Tile> createState() => _TileState();
}

class _TileState extends State<_Tile> {
  @override
  Widget build(BuildContext context) {
    final t = widget.t, v = widget.v, d = t.drives[v.id], time = _Time.of(context);
    return DragTarget<Beh>(
      onWillAcceptWithDetails: (e) {
        t.hover((v.id, e.data));
        return true;
      },
      onLeave: (_) => t.hover(null),
      onAcceptWithDetails: (e) => t.lay(v.id, e.data),
      builder: (context, cand, _) {
        final over = cand.isNotEmpty ? cand.first : null;
        return GestureDetector(
          onHorizontalDragUpdate: (e) => t.set(v, v.v + e.delta.dx / 180 * (v.hi - v.lo)),
          child: TweenAnimationBuilder<double>(
            key: ValueKey(t.flash == v.id ? d?.power : 0),
            tween: Tween(begin: t.flash == v.id ? 1 : 0, end: 0),
            duration: const Duration(milliseconds: 420),
            builder: (context, glow, _) => AnimatedContainer(
              duration: Mo.dur,
              height: Pop.tile,
              decoration: BoxDecoration(
                color: Color.lerp(over != null ? Grey.g20 : Pop.well, (d?.beh.c ?? Pop.accent), glow * .35),
                borderRadius: BorderRadius.circular(6),
                border: Border.all(color: over != null ? over.c : const Color(0x00000000), width: 1.5),
              ),
              padding: const EdgeInsets.fromLTRB(Pop.tilePad, 6, 6, 4),
              child: Stack(children: [
                Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisAlignment: MainAxisAlignment.spaceBetween, children: [
                  Text(v.label, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.micro(Grey.g63).copyWith(height: 1.1)),
                  ValueListenableBuilder<double>(
                    valueListenable: time,
                    builder: (context, s, _) => Text(
                      d == null && over == null ? sayVal(v, v.v) : sayVal(v, t.at(v.id, s, 0, 12)),
                      style: T.value(d != null || over != null ? (d ?? Drive(over!)).beh.c : Grey.g95).copyWith(fontSize: 15, fontWeight: FontWeight.w600),
                    ),
                  ),
                ]),
                if (d != null) Positioned(right: 0, top: 0, child: _LiftChip(t, v.id, d)),
                if (over != null && d?.beh != over) Positioned(right: 0, top: 0, child: Opacity(opacity: .6, child: _Chip(over))),
              ]),
            ),
          ),
        );
      },
    );
  }
}

/// The behaviour on a value: dragged off the tile, it lets go of it.
class _LiftChip extends StatelessWidget {
  const _LiftChip(this.t, this.id, this.d);
  final CardTable t;
  final String id;
  final Drive d;
  @override
  Widget build(BuildContext context) => Draggable<String>(
    data: id,
    feedback: _Lifted(_Chip(d.beh, power: d.power)),
    childWhenDragging: const SizedBox(width: 18, height: 18),
    onDragEnd: (e) {
      if (!e.wasAccepted) t.lift(id);
    },
    child: _Chip(d.beh, power: d.power),
  );
}

class _Chip extends StatelessWidget {
  const _Chip(this.b, {this.power = 1, this.small = false});
  final Beh b;
  final int power;
  final bool small;
  @override
  Widget build(BuildContext context) => Container(
    height: small ? 14 : 18,
    padding: EdgeInsets.symmetric(horizontal: small ? 4 : 5),
    decoration: BoxDecoration(color: b.c, borderRadius: BorderRadius.circular(9)),
    child: Row(mainAxisSize: MainAxisSize.min, children: [
      SizedBox.square(dimension: small ? 8 : 10, child: CustomPaint(painter: BehGlyph(b, N.g07))),
      if (power > 1) Text(' ×$power', style: T.micro(N.g07).copyWith(fontSize: small ? 8 : 9)),
    ]),
  );
}

/// A card in the air: lifted, a little turned, with a shadow under it.
class _Lifted extends StatelessWidget {
  const _Lifted(this.child);
  final Widget child;
  @override
  Widget build(BuildContext context) => Transform.rotate(
    angle: -.06,
    child: DecoratedBox(
      decoration: const BoxDecoration(boxShadow: [BoxShadow(color: Color(0xAA000000), blurRadius: 16, offset: Offset(0, 10))]),
      child: child,
    ),
  );
}

// ---------------------------------------------------------------- the hand

class _Hand extends StatelessWidget {
  const _Hand(this.t);
  final CardTable t;
  @override
  Widget build(BuildContext context) => Container(
    padding: const EdgeInsets.fromLTRB(Pop.inset, 8, Pop.inset, 10),
    decoration: BoxDecoration(color: Grey.g07, border: Border(top: BorderSide(color: Grey.g20))),
    child: Row(children: [for (final b in t.hand) Expanded(child: Padding(padding: const EdgeInsets.symmetric(horizontal: 2), child: _HandCard(b)))]),
  );
}

class _HandCard extends StatelessWidget {
  const _HandCard(this.b);
  final Beh b;
  Widget face({bool lifted = false}) => Container(
    height: 64,
    width: lifted ? 80 : null,
    padding: const EdgeInsets.all(8),
    decoration: BoxDecoration(color: Grey.g13, borderRadius: BorderRadius.circular(8), border: Border.all(color: lifted ? b.c : Grey.g20)),
    child: Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisAlignment: MainAxisAlignment.spaceBetween, children: [
      Container(
        width: 24,
        height: 24,
        decoration: BoxDecoration(color: b.c, borderRadius: BorderRadius.circular(6)),
        child: Padding(padding: const EdgeInsets.all(5), child: CustomPaint(painter: BehGlyph(b, N.g07))),
      ),
      Text(b.word, style: T.name(Grey.g95)),
    ]),
  );
  @override
  Widget build(BuildContext context) => Draggable<Beh>(
    data: b,
    dragAnchorStrategy: pointerDragAnchorStrategy,
    feedback: Transform.translate(offset: const Offset(-40, -32), child: _Lifted(face(lifted: true))),
    childWhenDragging: Opacity(opacity: .35, child: face()),
    child: face(),
  );
}

/// What a behaviour does, drawn: a wave, a staircase, a wobble, scattered dots.
class BehGlyph extends CustomPainter {
  const BehGlyph(this.b, this.c);
  final Beh b;
  final Color c;
  @override
  void paint(Canvas cv, Size s) {
    final w = s.width, st = Paint()..color = c..style = PaintingStyle.stroke..strokeWidth = w * .14..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round;
    final p = Path();
    switch (b) {
      case Beh.sway:
        for (var x = 0.0; x <= 1.0001; x += .05) {
          final y = .5 - .32 * math.sin(x * 2 * math.pi);
          x == 0 ? p.moveTo(0, y * w) : p.lineTo(x * w, y * w);
        }
      case Beh.stagger:
        p.moveTo(0, w * .9);
        for (var k = 0; k < 4; k++) {
          p
            ..lineTo(w * k / 4, w * (.9 - k * .25))
            ..lineTo(w * (k + 1) / 4, w * (.9 - k * .25));
        }
      case Beh.noise:
        for (var x = 0.0; x <= 1.0001; x += .1) {
          final y = .5 + .38 * (_hash(x * 9 + 2) - .5) * 2;
          x == 0 ? p.moveTo(0, y * w) : p.lineTo(x * w, y * w);
        }
      case Beh.random:
        for (final (x, y) in const [(.2, .3), (.7, .2), (.45, .55), (.15, .8), (.8, .75)]) {
          cv.drawCircle(Offset(x * w, y * w), w * .1, Paint()..color = c);
        }
        return;
    }
    cv.drawPath(p, st);
  }

  @override
  bool shouldRepaint(BehGlyph o) => o.b != b || o.c != c;
}
