// Itten's three forms, borrowed as pressable pieces. Not a Motolii control.
import 'package:flutter/widgets.dart';

const _paper = Color(0xFFF4F0E6);
const _ink = Color(0xFF141414);
const _red = Color(0xFFE23B2F);
const _blue = Color(0xFF2F4FA0);
const _yellow = Color(0xFFF2C200);

class BauhausKit extends StatefulWidget {
  const BauhausKit({super.key});

  @override
  State<BauhausKit> createState() => _BauhausKitState();
}

class _BauhausKitState extends State<BauhausKit> {
  var shape = 0;
  var down = false;
  var on = true;

  static const _type = TextStyle(fontFamily: 'Futura', fontSize: 13, letterSpacing: 1.6, fontWeight: FontWeight.w500, color: _ink, decoration: TextDecoration.none);

  @override
  Widget build(BuildContext context) {
    return ColoredBox(
      color: _paper,
      child: Align(
        alignment: Alignment.topLeft,
        child: SingleChildScrollView(
          padding: const EdgeInsets.fromLTRB(72, 80, 72, 80),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              _form(0, 'kreis', const _Circle()),
              const SizedBox(height: 28),
              _form(1, 'quadrat', const _Square()),
              const SizedBox(height: 28),
              _form(2, 'dreieck', const CustomPaint(painter: _Triangle(), child: SizedBox.square(dimension: 148))),
              const SizedBox(height: 72),
              _press(),
              const SizedBox(height: 64),
              _switch(),
            ],
          ),
        ),
      ),
    );
  }

  Widget _form(int i, String name, Widget face) {
    final picked = shape == i;
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      child: GestureDetector(
        onTap: () => setState(() => shape = i),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.center,
          children: [
            SizedBox(
              width: 164,
              height: 164,
              child: Stack(
                children: [
                  if (picked) const Positioned(left: 14, top: 14, child: ColoredBox(color: _ink, child: SizedBox.square(dimension: 148))),
                  face,
                ],
              ),
            ),
            const SizedBox(width: 18),
            Text(name, style: _type),
          ],
        ),
      ),
    );
  }

  Widget _press() {
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      child: GestureDetector(
        onTapDown: (_) => setState(() => down = true),
        onTapUp: (_) => setState(() => down = false),
        onTapCancel: () => setState(() => down = false),
        child: SizedBox(
          width: 300,
          height: 88,
          child: Stack(
            children: [
              const Positioned(left: 10, top: 10, child: ColoredBox(color: _ink, child: SizedBox(width: 268, height: 68))),
              Positioned(
                left: down ? 10 : 0,
                top: down ? 10 : 0,
                child: const ColoredBox(
                  color: _yellow,
                  child: SizedBox(
                    width: 268,
                    height: 68,
                    child: Center(child: Text('DRUCKEN', style: TextStyle(fontFamily: 'Futura', fontSize: 22, letterSpacing: 3, fontWeight: FontWeight.w700, color: _ink, decoration: TextDecoration.none))),
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _switch() {
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      child: GestureDetector(
        onTap: () => setState(() => on = !on),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            SizedBox(
              width: 78,
              height: 78,
              child: Stack(
                children: [
                  const Positioned(left: 8, top: 8, child: ColoredBox(color: _ink, child: SizedBox.square(dimension: 64))),
                  Positioned(
                    left: 0,
                    top: 0,
                    child: DecoratedBox(
                      decoration: BoxDecoration(color: on ? _red : _paper, shape: BoxShape.circle, border: on ? null : Border.all(color: _ink, width: 2)),
                      child: const SizedBox.square(dimension: 64),
                    ),
                  ),
                ],
              ),
            ),
            const SizedBox(width: 16),
            Text(on ? 'an' : 'aus', style: _type.copyWith(fontSize: 22, letterSpacing: 2)),
          ],
        ),
      ),
    );
  }
}

class _Circle extends StatelessWidget {
  const _Circle();

  @override
  Widget build(BuildContext context) => const DecoratedBox(decoration: BoxDecoration(color: _red, shape: BoxShape.circle), child: SizedBox.square(dimension: 148));
}

class _Square extends StatelessWidget {
  const _Square();

  @override
  Widget build(BuildContext context) => const ColoredBox(color: _blue, child: SizedBox.square(dimension: 148));
}

class _Triangle extends CustomPainter {
  const _Triangle();

  @override
  void paint(Canvas c, Size s) {
    c.drawPath(Path()..moveTo(s.width / 2, 0)..lineTo(s.width, s.height)..lineTo(0, s.height)..close(), Paint()..color = _yellow);
  }

  @override
  bool shouldRepaint(_Triangle o) => false;
}
