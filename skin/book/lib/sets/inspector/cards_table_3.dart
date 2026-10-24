// Part of cards_table.dart (cut there by size, as it was).
part of 'cards_table.dart';

CardM waveCard() => CardM('Wave', Pop.tonePath, [
  Val('waveH', 'Height', 30, 0, 90, unit: 'px'),
  Val('waveS', 'Speed', 1, 0, 4, dec: 2, perCopy: false),
], back: [_px('waveX', 'Sideways', 0, 0, 90), Val('waveRot', 'Rotate by wave', 0, 0, 90, unit: '°'), _pc('waveSize', 'Scale by wave', 0), Val('phase', 'Phase', 0, 0, 1, dec: 2)]);

CardM scatterCard() => CardM('Scatter', Pop.toneParticles, [
  Val('scatter', 'Amount', 50, 0, 120, unit: 'px'),
  Val('scatterTurn', 'Rotation', 0, 0, 180, unit: '°'),
], back: [_pc('scatterSize', 'Scale', 0), Val('seed', 'Seed', 0, 0, 99, perCopy: false)]);

/// The ring's cards as the window shows a shape with copies today.
ThingM ringThing({bool wave = true, Offset base = Offset.zero}) => ThingM('Ring', base: base, shelf: [scatterCard(), if (!wave) waveCard()], [
  CardM('Layer', Pop.toneLayer, [
    Val('size', 'Size', 44, 10, 120, unit: 'px'),
    _deg('turn', 'Rotation'),
    Val('opacity', 'Opacity', 100, 0, 100, unit: '%'),
  ], back: [_px('posX', 'Position X'), _px('posY', 'Position Y'), _pc('scaleX', 'Scale X'), _pc('scaleY', 'Scale Y'), _deg('rotX', 'Rotation X'), _deg('rotY', 'Rotation Y')]),
  CardM('Copies', Pop.tonePlace, [
    Val('count', 'Count', 12, 3, 48, perCopy: false),
    Val('radius', 'Radius', 180, 20, 300, unit: 'px'),
    Val('spin', 'Turn each', 30, 0, 180, unit: '°'),
  ], back: [_pc('copySize', 'Copy scale'), _px('copyZ', 'Depth', 0, 0, 400), Val('delay', 'Delay', 0, 0, 2, unit: 's', dec: 1, perCopy: false)]),
  if (wave) waveCard(),
  CardM('Fill', Pop.toneFill, [
    _deg('hue', 'Hue', 72),
    Val('light', 'Lightness', 62, 20, 90, unit: '%'),
  ], back: [Val('strokeW', 'Stroke width', 0, 0, 12, unit: 'px')]),
]);

/// One square: a thing to link the ring to.
ThingM squareThing({Offset base = Offset.zero, String name = 'Square', String? short, double size = 120, double hue = 200}) => ThingM(name, short: short, ring: false, base: base, [
  CardM('Layer', Pop.toneLayer, [
    Val('size', 'Size', size, 20, 300, unit: 'px'),
    _deg('turn', 'Rotation'),
    Val('opacity', 'Opacity', 100, 0, 100, unit: '%'),
  ], back: [_px('posX', 'Position X'), _px('posY', 'Position Y')]),
  CardM('Fill', Pop.toneFill, [
    _deg('hue', 'Hue', hue),
    Val('light', 'Lightness', 60, 20, 90, unit: '%'),
  ]),
]);

CardTable starRing({bool wave = true}) => CardTable([ringThing(wave: wave)]);

/// Many things: the ring and six squares beside it, to carry one thing's value into many others.
CardTable manyThings() => CardTable([
  ringThing(base: const Offset(-200, 0)),
  for (var k = 0; k < 6; k++)
    squareThing(name: 'Square ${k + 1}', short: 'S${k + 1}', base: Offset(190 + (k ~/ 3) * 150, -220 + (k % 3) * 220), size: 90, hue: 160 + k * 30),
]);

String sayVal(Val v, double x) => '${x.toStringAsFixed(v.dec)}${v.unit}';

// ---------------------------------------------------------------- the table

class CardsTable extends StatefulWidget {
  const CardsTable(this.t, {super.key});
  final CardTable t;
  @override
  State<CardsTable> createState() => _CardsTableState();
}

class _CardsTableState extends State<CardsTable> with SingleTickerProviderStateMixin {
  late final Ticker tk;
  final time = ValueNotifier<double>(0); // (only a beat for what is painted moment by moment: the clocks are the table's)
  Duration last = Duration.zero;

  @override
  void initState() {
    super.initState();
    tk = createTicker((d) {
      final dt = (d - last).inMicroseconds / 1e6;
      last = d;
      widget.t.advance(dt);
      time.value += dt;
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

  Widget _table(BuildContext context) => CallbackShortcuts(
    bindings: {
      const SingleActivator(LogicalKeyboardKey.keyZ, meta: true): widget.t.undo,
      const SingleActivator(LogicalKeyboardKey.keyZ, meta: true, shift: true): widget.t.redo,
      const SingleActivator(LogicalKeyboardKey.keyZ, control: true): widget.t.undo,
    },
    child: Focus(autofocus: true, child: ListenableBuilder(
    listenable: widget.t,
    builder: (context, _) => ColoredBox(
      color: N.g07,
      child: Row(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        Expanded(child: _Stage(widget.t)),
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
  )),
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

/// The stage takes a shelf card (worn) and a behaviour card: where a behaviour comes in, a ring of the values most often moved
/// opens; the way the hand goes from there picks the value (shown before it is let go), the middle lets go of nothing.
class _Stage extends StatefulWidget {
  const _Stage(this.t);
  final CardTable t;
  @override
  State<_Stage> createState() => _StageState();
}

/// The four values a thing's ring offers: up, right, down, left.
List<String> _ringValsOf(ThingM th) => th.ring ? const ['size', 'radius', 'spin', 'hue'] : const ['size', 'turn', 'hue', 'posX'];

class _StageState extends State<_Stage> {
  Offset? anchor, at, restAt, restFrom; // the ring's place (it opens where the hand rests, pushed inside the stage), the hand's, where it rested
  int? aim;
  Timer? rest;
  ThingM? target; // the thing the ring opened over: a behaviour goes on the picked thing, a value on another thing
  ThingM? under; // a value carried over another thing: let go now, that thing's value of the same name wears its tag
  ValRef? pending; // a value let go on a thing without a value of its name: the ring stays open there, to be tapped

  static ValRef? _ref(Object? o, CardTable t) => switch (o) {
    Val v => ValRef(t.cur, v.id),
    ValRef r => r,
    _ => null,
  };
  CardTable get t => widget.t;

  @override
  void dispose() {
    rest?.cancel();
    super.dispose();
  }

  double get _k {
    final box = context.findRenderObject() as RenderBox?;
    return box == null ? 1 : math.min(box.size.width, box.size.height) / 720;
  }

  /// Where a thing stands on the stage and how far round it a hand still counts as over it.
  (Offset, double) _place(ThingM th) {
    final box = context.findRenderObject()! as RenderBox, k = _k;
    final c = box.size.center(Offset.zero) + (th.base + Offset(t.at('posX', 0, 1, of: th), t.at('posY', 0, 1, of: th))) * k;
    return (c, (th.ring ? t.at('radius', 0, 1, of: th) + t.at('size', 0, 1, of: th) : t.at('size', 0, 1, of: th) * .8) * k);
  }

  /// The box round a thing on the stage (to light the one a value would go to).
  Rect _box(ThingM th) {
    final (c, r) = _place(th);
    return Rect.fromCircle(center: c, radius: r);
  }

  ThingM? _thingAt(Offset p, {ThingM? not}) {
    ThingM? best;
    var near = double.infinity;
    for (final th in t.things) {
      if (th == not) continue;
      final (c, r) = _place(th);
      final d = (p - c).distance;
      if (d < r && d < near) (best, near) = (th, d);
    }
    return best;
  }

  /// Which thing a carried card is for: a behaviour, the picked one; a value, the thing under the hand other than its own.
  ThingM? _for(Object o, Offset p) => switch (o) {
    Beh() => t.cur,
    Val() => _thingAt(p, not: t.cur),
    ValRef r => _thingAt(p, not: r.thing),
    _ => null,
  };

  /// The hand moved: with no ring open, the ring waits for it to rest (0.25 s); far from an open ring, the ring closes and waits again.
  void _moved(Offset p, Object o) {
    at = p;
    if (o is Val || o is ValRef) {
      final th = _for(o, p);
      if (th != under) setState(() => under = th);
    }
    if (anchor == null && restFrom != null && (p - restFrom!).distance < 6) return; // (a hand trembling is a hand resting)
    restFrom = p;
    if (anchor != null && (p - (restAt ?? anchor!)).distance > 160) {
      setState(() => anchor = aim = target = null);
      t.hover(null);
    }
    if (anchor == null) {
      rest?.cancel();
      rest = Timer(const Duration(milliseconds: 250), () {
        final box = context.findRenderObject() as RenderBox?;
        if (!mounted || at == null || box == null) return;
        final th = _for(o, at!);
        if (th == null) return; // (a value over no other thing: no ring)
        restAt = at;
        setState(() {
          target = th;
          anchor = Offset(at!.dx.clamp(118, math.max(118, box.size.width - 118)), at!.dy.clamp(88, math.max(88, box.size.height - 88))); // (the whole ring inside the stage)
        });
      });
      return;
    }
    final a = _aimAt(p);
    if (a != aim) setState(() => aim = a);
    if (o is Beh) t.hover(a == null ? null : (_ringValsOf(target!)[a], o));
  }

  Offset _local(Offset global) => (context.findRenderObject()! as RenderBox).globalToLocal(global);

  /// Where each of the four holes sits: just outside the thing, then slid along the hole's side until its words clear the neighbours.
  /// A hole that lands on the next square cannot be read (Cultist: the mark of what a hole accepts has to stay a mark).
  List<Offset> _seats(ThingM th) {
    final (c, r) = _place(th);
    final stage = context.findRenderObject()! as RenderBox;
    Offset seat(int k) {
      final dir = const [Offset(0, -1), Offset(1, 0), Offset(0, 1), Offset(-1, 0)][k];
      final perp = Offset(-dir.dy, dir.dx);
      final wide = k.isEven;
      final hw = wide ? 58.0 : 52.0, hh = 15.0;
      final origin = c + dir * (r + (wide ? hh : hw) + 12);
      Rect box(Offset p) => Rect.fromCenter(center: p, width: hw * 2, height: hh * 2);
      bool clear(Offset p) {
        final b = box(p);
        if (b.overlaps(Rect.fromCircle(center: c, radius: r * .62))) return false;
        for (final o in t.things) {
          if (o != th && b.overlaps(_box(o).inflate(8))) return false;
        }
        return true;
      }

      Offset pick(Offset p) => Offset(p.dx.clamp(64, math.max(64, stage.size.width - 64)), p.dy.clamp(22, math.max(22, stage.size.height - 22)));
      // A clear seat can sit on the next square's column. Walk it back toward this thing and keep the last step that still clears.
      Offset hug(Offset p) {
        final goal = c + dir * (r * .55 + (wide ? hh : hw) + 8);
        var best = p;
        for (var i = 1; i <= 8; i++) {
          final q = Offset.lerp(p, goal, i / 8)!;
          if (!clear(q)) break;
          best = q;
        }
        return best;
      }

      if (clear(origin)) return pick(origin);
      for (var n = 1; n <= 12; n++) {
        final d = n * 14.0, a = origin + perp * d, b = origin - perp * d;
        final aOk = clear(a), bOk = clear(b);
        if (!aOk && !bOk) continue;
        if (aOk && bOk) {
          double room(Offset q) {
            var m = double.infinity;
            for (final o in t.things) {
              if (o == th) continue;
              m = math.min(m, (q - _place(o).$1).distance);
            }
            return m;
          }

          final ra = room(a), rb = room(b);
          if ((ra - rb).abs() > 8) return pick(hug(ra > rb ? a : b));
          var away = Offset.zero; // (the neighbours' side: the hole takes the other one)
          for (final o in t.things) {
            if (o != th) away += _place(o).$1 - c;
          }
          return pick(hug(away.dx * perp.dx + away.dy * perp.dy > 0 ? b : a));
        }
        return pick(hug(aOk ? a : b));
      }
      return pick(origin);
    }

    return [for (var k = 0; k < 4; k++) seat(k)];
  }

  int? _aimAt(Offset p) {
    final d = p - (restAt ?? anchor!); // (the way the hand went from where it rested: a ring pushed in from the edge picks nothing by itself)
    if (d.distance < 28 || target == null) return null;
    final k = d.dx.abs() > d.dy.abs() ? (d.dx > 0 ? 1 : 3) : (d.dy < 0 ? 0 : 2);
    return t.valOr(_ringValsOf(target!)[k], of: target) == null ? null : k;
  }

  void _leave() {
    rest?.cancel();
    restFrom = null;
    setState(() => anchor = aim = at = target = under = null);
    t.hover(null);
    t.holdCard(null);
  }

  @override
  void setState(VoidCallback fn) {
    super.setState(fn);
    t.ringOpen.value = anchor != null;
  }

  void _let(Object o) {
    final id = aim == null || target == null ? null : _ringValsOf(target!)[aim!];
    switch (o) {
      case Beh b:
        id != null ? t.lay(id, b) : (anchor == null ? t.tapLay(b) : null); // (thrown before the ring opens: as a tap; let go in its middle: nothing)
      case Val v when id != null:
        t.join(id, ValRef(t.cur, v.id), on: target);
      case ValRef r when id != null:
        t.join(id, r, on: target);
      case Val() || ValRef() when (under ?? target) != null:
        final r = _ref(o, t)!, th = under ?? target!;
        if (t.valOr(r.id, of: th) != null) {
          t.join(r.id, r, on: th); // (let go on a thing, at once or after its ring opened with nothing aimed: its value of the same name)
        } else if (anchor != null) {
          pending = r; // (its ring is open already: it stays, waiting for a tap)
          target = th;
        } else {
          final box = context.findRenderObject()! as RenderBox, p = at ?? box.size.center(Offset.zero);
          pending = r; // (none of that name there: its ring opens and waits for a tap)
          target = under;
          restAt = p;
          anchor = Offset(p.dx.clamp(118, math.max(118, box.size.width - 118)), p.dy.clamp(88, math.max(88, box.size.height - 88)));
        }
    }
  }

  @override
  Widget build(BuildContext context) => GestureDetector(
    onTapUp: (e) { // (a thing tapped on the stage is picked; a waiting ring is closed)
      if (pending != null) {
        pending = null;
        return _leave();
      }
      final th = _thingAt(e.localPosition);
      if (th != null) t.select(t.things.indexOf(th));
    },
    child: DragTarget<Object>(
      onWillAcceptWithDetails: (e) {
        if (e.data is CardM && !t.cards.contains(e.data)) {
          t.holdCard(e.data as CardM);
          return true;
        }
        if (e.data is Beh || e.data is Val || e.data is ValRef) {
          _moved(_local(e.offset), e.data);
          return true;
        }
        return false;
      },
      onMove: (e) {
        if (e.data is Beh || e.data is Val || e.data is ValRef) _moved(_local(e.offset), e.data);
      },
      onLeave: (_) => _leave(),
      onAcceptWithDetails: (e) {
        if (e.data is CardM) t.wear(e.data as CardM);
        _let(e.data);
        if (pending == null) return _leave();
        rest?.cancel();
        setState(() => aim = under = null); // (the ring stays, waiting for a tap)
        t.hover(null);
      },
      builder: (context, cand, _) {
        final held = t.carrying;
        final waiting = (held is CardM && !t.cards.contains(held)) || held is Beh || ((held is Val || held is ValRef) && t.things.length > 1); // (the stage takes these)
        final o = cand.where((x) => x is Beh || x is Val || x is ValRef).firstOrNull;
        final colour = o is Beh ? o.c : Pop.accent;
        return ClipRect(
          child: CustomPaint(
            painter: _Ring(t, _Time.of(context)),
            foregroundPainter: waiting
                ? _Frame(cand.isNotEmpty, faint: [
                    if (held is Val || held is ValRef)
                      for (final th in t.things)
                        if (!(held is Val && th == t.cur) && !(held is ValRef && held.thing == th)) _box(th),
                  ], things: [
                    if (under != null && anchor == null) _box(under!),
                  ])
                : null,
            child: Stack(children: [
              const SizedBox.expand(),
              if ((o != null || pending != null) && anchor != null && target != null)
                Positioned.fill(child: _ValueRing(t, target!, colour, aim, _seats(target!), _place(target!).$1, onPick: pending == null ? null : (k) {
                  final id = _ringValsOf(target!)[k];
                  if (t.valOr(id, of: target) != null) t.join(id, pending!, on: target);
                  pending = null;
                  _leave();
                })),
            ]),
          ),
        );
      },
    ),
  );
}

/// The ring a carried card opens on the stage: four values of the thing, each just off it and off its neighbours, the one aimed at lit.
class _ValueRing extends StatelessWidget {
  const _ValueRing(this.t, this.th, this.c, this.aim, this.seats, this.heart, {this.onPick});
  final CardTable t;
  final ThingM th;
  final Color c;
  final int? aim;
  final List<Offset> seats;
  final Offset heart;
  final void Function(int)? onPick; // a ring left open after a drop: its values are tapped
  @override
  Widget build(BuildContext context) {
    Widget pill(int k) {
      final v = t.valOr(_ringValsOf(th)[k], of: th), on = aim == k;
      return Container(
        padding: const EdgeInsets.symmetric(horizontal: 9, vertical: 5),
        decoration: BoxDecoration(color: on ? c : Grey.g13.withValues(alpha: .92), borderRadius: BorderRadius.circular(12), border: Border.all(color: on ? c : Grey.g38)),
        child: Text(v?.label ?? '—', style: T.name(on ? N.g07 : (v == null ? Grey.g44 : Grey.g91))),
      );
    }

    Widget pick(int k) => onPick == null ? pill(k) : GestureDetector(onTap: () => onPick!(k), child: MouseRegion(cursor: SystemMouseCursors.click, child: pill(k)));

    return Stack(children: [
      Positioned(left: heart.dx - 11, top: heart.dy - 11, child: Container(width: 22, height: 22, decoration: BoxDecoration(shape: BoxShape.circle, border: Border.all(color: Grey.g44)))),
      for (var k = 0; k < 4; k++)
        Positioned(left: seats[k].dx, top: seats[k].dy, child: FractionalTranslation(translation: const Offset(-.5, -.5), child: pick(k))),
    ]);
  }
}

/// The stage's edge while a shelf card is carried: the stage takes it.
class _Frame extends CustomPainter {
  _Frame(this.over, {this.things = const [], this.faint = const []});
  final bool over;
  final List<Rect> things, faint; // the things a carried value would go to, lit; the others that could take it, faintly
  @override
  void paint(Canvas c, Size s) {
    c.drawRRect(
      RRect.fromRectAndRadius((Offset.zero & s).deflate(10), const Radius.circular(10)),
      Paint()..color = Pop.accent.withValues(alpha: over ? .9 : .35)..style = PaintingStyle.stroke..strokeWidth = over ? 2 : 1,
    );
    for (final r in faint) {
      final rr = RRect.fromRectAndRadius(r.inflate(6), const Radius.circular(10));
      for (final m in (Path()..addRRect(rr)).computeMetrics()) { // (dashed: could take it)
        for (var d = 0.0; d < m.length; d += 10) {
          c.drawPath(m.extractPath(d, d + 5), Paint()..color = Pop.accent.withValues(alpha: .45)..style = PaintingStyle.stroke..strokeWidth = 1);
        }
      }
    }
    for (final r in things) {
      c.drawRRect(RRect.fromRectAndRadius(r.inflate(6), const Radius.circular(10)), Paint()..color = Pop.accent.withValues(alpha: .18));
      c.drawRRect(RRect.fromRectAndRadius(r.inflate(6), const Radius.circular(10)), Paint()..color = Pop.accent..style = PaintingStyle.stroke..strokeWidth = 2);
    }
  }

  @override
  bool shouldRepaint(_Frame o) => true;
}
