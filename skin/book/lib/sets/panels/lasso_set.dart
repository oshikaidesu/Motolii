// A try (not joined to the document): relations made with a lasso. Draw round some things on the stage, pick what they
// do together (Around, Row, Follow); the relation's handles show on the stage; the Relations card lists each, with its
// strength (0: as if it were not there, so taking it off never jumps), its members (x takes one out where it stands) and
// Release (all keep where they are now). A thing in a relation is still dragged by hand: what the hand adds stays as its
// own offset (Bitwig's way: a value driven is still turned, and turning moves its centre). research_notes/link_ui2/SUMMARY.md
import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import '../../tokens.dart';
import '../../workspace/ws.dart' show WsT;

enum _Kind { around, row, follow }

class _Thing {
  _Thing(this.id, this.at, this.colour, this.shape);
  final int id;
  Offset at; // (where it stands of its own, before any relation)
  Offset offset = Offset.zero; // (what a hand added while it was in a relation)
  final Color colour;
  final int shape; // (0 circle, 1 square, 2 triangle)
}

class _Relation {
  _Relation(this.kind, this.members, this.centre, this.size);
  final _Kind kind;
  final List<int> members;
  Offset centre; // (Around: the middle; Row: where it begins; Follow: unused)
  double size; // (Around: the radius; Row: the step)
  double strength = 1;
  final Map<int, Offset> from = {}; // (Follow: each member's place against the first one, when made)
  final Map<int, double> slot = {}; // (Around: its share of the turn; Row: its step; given when made, so one taken out moves no other)
}

class _Model extends ChangeNotifier {
  _Model() {
    const colours = [Color(0xFFE5432D), Color(0xFF2A62C9), Color(0xFFF7D046), Color(0xFF6BBF8C), Color(0xFFF2A7C6), Color(0xFF151515), Color(0xFF2A62C9), Color(0xFFE5432D)];
    final r = math.Random(3);
    for (var i = 0; i < 8; i++) {
      things.add(_Thing(i, Offset(90 + r.nextDouble() * 520, 70 + r.nextDouble() * 320), colours[i], i % 3));
    }
  }
  final things = <_Thing>[];
  final relations = <_Relation>[];
  Set<int> picked = {};
  _Relation? looked; // (the relation whose card is touched: its members bright, the rest dim)

  /// Where a thing shows: its own place, each relation it is in pulling it as strong as that relation is, then the hand's offset.
  Offset shown(_Thing t) {
    var p = t.at;
    for (final r in relations) {
      final k = r.members.indexOf(t.id);
      if (k < 0) continue;
      final Offset there = switch (r.kind) {
        _Kind.around => r.centre + Offset.fromDirection(-math.pi / 2 + 2 * math.pi * r.slot[t.id]!, r.size),
        _Kind.row => r.centre + Offset(r.size * r.slot[t.id]!, 0),
        _Kind.follow => k == 0 ? p : shownBase(r.members.first) + r.from[t.id]!,
      };
      p = Offset.lerp(p, there, r.strength)!;
    }
    return p + t.offset;
  }

  /// The first of a Follow, where it shows (the others follow it).
  Offset shownBase(int id) => shown(things[id]);

  void make(_Kind kind) {
    if (picked.length < 2) return;
    final ms = picked.toList()..sort((a, b) => shown(things[a]).dx.compareTo(shown(things[b]).dx));
    final ps = [for (final m in ms) shown(things[m])];
    final mid = ps.reduce((a, b) => a + b) / ps.length.toDouble();
    final r = switch (kind) {
      _Kind.around => _Relation(kind, ms, mid, ps.map((p) => (p - mid).distance).reduce((a, b) => a + b) / ps.length),
      _Kind.row => _Relation(kind, ms, ps.first, (ps.last.dx - ps.first.dx) / math.max(1, ps.length - 1)),
      _Kind.follow => _Relation(kind, ms, ps.first, 0),
    };
    for (var i = 0; i < ms.length; i++) {
      r.from[ms[i]] = ps[i] - ps.first;
      r.slot[ms[i]] = kind == _Kind.around ? i / ms.length : i.toDouble();
    }
    for (final m in ms) {
      things[m].at = shown(things[m]) - things[m].offset; // (made where they stand: the relation then pulls from there)
    }
    relations.add(r);
    looked = r;
    picked = {};
    notifyListeners();
  }

  /// One thing out of a relation, kept where it shows now.
  void takeOut(_Relation r, int id) {
    final t = things[id], now = shown(t);
    r.members.remove(id);
    t.at = now;
    t.offset = Offset.zero;
    if (r.members.length < 2) relations.remove(r);
    notifyListeners();
  }

  /// The relation gone, every member kept where it shows now.
  void release(_Relation r) {
    final now = {for (final m in r.members) m: shown(things[m])};
    relations.remove(r);
    for (final e in now.entries) {
      things[e.key]
        ..at = e.value
        ..offset = Offset.zero;
    }
    if (looked == r) looked = null;
    notifyListeners();
  }

  bool inAny(int id) => relations.any((r) => r.members.contains(id));
  void changed() => notifyListeners();
}

const _radius = 22.0;

class _Lasso extends StatefulWidget {
  const _Lasso();
  @override
  State<_Lasso> createState() => _LassoState();
}

class _LassoState extends State<_Lasso> {
  final m = _Model();
  final lasso = <Offset>[];
  int? dragging; // (a thing)
  (_Relation, int)? handle; // (a relation's handle: 0 centre, 1 size)

  @override
  void dispose() {
    m.dispose();
    super.dispose();
  }

  List<(Offset, _Relation, int)> get handles => [
    for (final r in m.relations)
      if (r.kind != _Kind.follow) ...[
        (r.centre, r, 0),
        (r.kind == _Kind.around ? r.centre + Offset.fromDirection(-math.pi / 2 + math.pi / r.slot.length, r.size) : r.centre + Offset(r.size, -34), r, 1),   // (the size between the first two, not on a thing)
      ],
  ];

  void down(Offset p) {
    for (final h in handles) {
      if ((h.$1 - p).distance < 12) return setState(() => handle = (h.$2, h.$3));
    }
    for (final t in m.things.reversed) {
      if ((m.shown(t) - p).distance < _radius) {
        return setState(() {
          dragging = t.id;
          if (!m.picked.contains(t.id)) m.picked = {t.id};
        });
      }
    }
    setState(() {
      lasso
        ..clear()
        ..add(p);
      m.picked = {};
      m.looked = null;
    });
  }

  void move(Offset p, Offset by) {
    if (handle case (final r, final k)) {
      k == 0 ? r.centre += by : r.size = r.kind == _Kind.around ? (p - r.centre).distance : math.max(8, p.dx - r.centre.dx);
      return m.changed();
    }
    if (dragging case final id?) {
      final t = m.things[id];
      m.inAny(id) ? t.offset += by : t.at += by; // (in a relation: the hand's part kept apart from it)
      return m.changed();
    }
    setState(() => lasso.add(p));
  }

  void up() {
    if (lasso.length > 2) {
      final path = Path()..addPolygon(lasso, true);
      m.picked = {for (final t in m.things) if (path.contains(m.shown(t))) t.id};
    }
    setState(() {
      lasso.clear();
      dragging = null;
      handle = null;
    });
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: m,
    builder: (context, _) {
      final ps = [for (final id in m.picked) m.shown(m.things[id])];
      final corner = ps.isEmpty ? null : ps.reduce((a, b) => Offset(math.max(a.dx, b.dx), math.max(a.dy, b.dy)));
      return Row(
        children: [
          Expanded(
            child: ClipRect(
              child: Stack(
                children: [
                  Positioned.fill(
                    child: GestureDetector(
                      onPanDown: (d) => down(d.localPosition),
                      onPanUpdate: (d) => move(d.localPosition, d.delta),
                      onPanEnd: (_) => up(),
                      onPanCancel: up,
                      child: CustomPaint(painter: _StagePaint(m, lasso, handles)),
                    ),
                  ),
                  if (corner != null && m.picked.length > 1 && lasso.isEmpty && dragging == null)   // (what they may do together, beside what was drawn round: above the stage's hand, so a press on it is its own)
                    Positioned(
                      left: corner.dx + 14,
                      top: corner.dy + 10,
                      child: Row(children: [for (final k in _Kind.values) _Chip(k.name[0].toUpperCase() + k.name.substring(1), () => m.make(k))]),
                    ),
                ],
              ),
            ),
          ),
          SizedBox(width: 260, child: _Card(m)),
        ],
      );
    },
  );
}

class _StagePaint extends CustomPainter {
  _StagePaint(this.m, this.lasso, this.handles);
  final _Model m;
  final List<Offset> lasso;
  final List<(Offset, _Relation, int)> handles;
  @override
  void paint(Canvas c, Size s) {
    c.drawRect(Offset.zero & s, Paint()..color = const Color(0xFFF1ECE2));
    final ink = Paint()
      ..color = const Color(0xFF151515)
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.5;
    for (final r in m.relations) {
      final faint = Paint()
        ..color = const Color(0x55151515)
        ..style = PaintingStyle.stroke;
      if (r.kind == _Kind.around) c.drawCircle(r.centre, r.size, faint);
      if (r.kind == _Kind.row) c.drawLine(r.centre, r.centre + Offset(r.size * r.slot.values.reduce(math.max), 0), faint);
    }
    for (final t in m.things) {
      final p = m.shown(t), dim = m.looked != null && !m.looked!.members.contains(t.id);
      final fill = Paint()..color = t.colour.withValues(alpha: dim ? .25 : 1);
      switch (t.shape) {
        case 0:
          c.drawCircle(p, _radius, fill);
        case 1:
          c.drawRect(Rect.fromCircle(center: p, radius: _radius * .85), fill);
        default:
          c.drawPath(Path()..addPolygon([p + const Offset(0, -_radius), p + const Offset(_radius, _radius * .8), p + const Offset(-_radius, _radius * .8)], true), fill);
      }
      if (m.picked.contains(t.id)) c.drawCircle(p, _radius + 5, ink);
      if (m.inAny(t.id)) c.drawCircle(p + const Offset(_radius * .8, -_radius * .8), 3.5, Paint()..color = const Color(0xFF151515)); // (in a relation: a dot, as a mark on the thing)
    }
    for (final h in handles) {
      c.drawCircle(h.$1, 7, Paint()..color = const Color(0xFFFFFFFF));
      c.drawCircle(h.$1, 7, ink);
    }
    if (lasso.length > 1) {
      c.drawPath(
        Path()..addPolygon(lasso, false),
        Paint()
          ..color = const Color(0xFF151515)
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1.2
          ..shader = ui.Gradient.linear(lasso.first, lasso.last, [const Color(0xFF151515), const Color(0xFF2A62C9)]),
      );
    }
  }

  @override
  bool shouldRepaint(_StagePaint o) => true;
}

class _Chip extends StatelessWidget {
  const _Chip(this.word, this.onTap);
  final String word;
  final VoidCallback onTap;
  @override
  Widget build(BuildContext context) => GestureDetector(
    onTap: onTap,
    child: Container(
      margin: const EdgeInsets.only(right: 4),
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
      decoration: BoxDecoration(color: const Color(0xFF151515), borderRadius: BorderRadius.circular(12)),
      child: Text(word, style: T.label(const Color(0xFFFFFFFF))),
    ),
  );
}

/// The Relations card: each relation, its strength, its members (x: that one out, kept where it stands), Release.
class _Card extends StatelessWidget {
  const _Card(this.m);
  final _Model m;
  @override
  Widget build(BuildContext context) => Container(
    color: WsT.body,
    padding: const EdgeInsets.all(10),
    child: ListView(
      children: [
        Text('Relations', style: T.title()),
        const SizedBox(height: 8),
        if (m.relations.isEmpty) Text('Draw round two or more things', style: T.label()),
        for (final r in m.relations)
          GestureDetector(
            onTap: () {
              m.looked = m.looked == r ? null : r;
              m.changed();
            },
            child: Container(
              margin: const EdgeInsets.only(bottom: 8),
              padding: const EdgeInsets.all(8),
              color: m.looked == r ? WsT.raised : WsT.card,
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  Row(
                    children: [
                      Expanded(child: Text('${r.kind.name[0].toUpperCase()}${r.kind.name.substring(1)} · ${r.members.length}', style: T.name())),
                      GestureDetector(onTap: () => m.release(r), child: Text('Release', style: T.label(WsT.accent))),
                    ],
                  ),
                  const SizedBox(height: 6),
                  _Strength(r, m),
                  const SizedBox(height: 6),
                  Wrap(
                    spacing: 4,
                    runSpacing: 4,
                    children: [
                      for (final id in r.members)
                        GestureDetector(
                          onTap: () => m.takeOut(r, id),
                          child: Container(
                            padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 3),
                            color: m.things[id].colour,
                            child: Text('${id + 1} ×', style: T.label(const Color(0xFFFFFFFF))),
                          ),
                        ),
                    ],
                  ),
                ],
              ),
            ),
          ),
      ],
    ),
  );
}

/// How strong the relation pulls, dragged sideways: at 0 every member stands where it would without it.
class _Strength extends StatelessWidget {
  const _Strength(this.r, this.m);
  final _Relation r;
  final _Model m;
  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, box) => GestureDetector(
      onHorizontalDragUpdate: (d) {
        r.strength = (d.localPosition.dx / box.maxWidth).clamp(0.0, 1.0);
        m.changed();
      },
      onTapDown: (d) {
        r.strength = (d.localPosition.dx / box.maxWidth).clamp(0.0, 1.0);
        m.changed();
      },
      child: Container(
        height: 18,
        color: WsT.well,
        alignment: Alignment.centerLeft,
        child: FractionallySizedBox(
          widthFactor: r.strength,
          child: Container(color: WsT.accent, alignment: Alignment.centerRight, padding: const EdgeInsets.only(right: 4), child: Text('${(r.strength * 100).round()}%', style: T.label(WsT.onAccent))),
        ),
      ),
    ),
  );
}

WidgetbookComponent lassoSet() => WidgetbookComponent(
  name: 'Lasso relations',
  useCases: [WidgetbookUseCase(name: 'Try', builder: (c) => const _Lasso())],
);
