// Visual prototype only. Throwaway; not wired to anything.
import 'package:flutter/widgets.dart';

// One chassis. Regions are separated by rules, not by card fills.
abstract final class P {
  static const bg = Color(0xFF151515);
  static const well = Color(0xFF0F0F0F);
  static const key = Color(0xFF202020);
  static const keyHi = Color(0xFF2A2A2A);
  static const rule = Color(0xFF262626);
  static const rule2 = Color(0xFF363636);
  static const text = Color(0xFFEDEDED);
  static const text2 = Color(0xFFC0C0C0);
  static const muted = Color(0xFF858585);
  static const dim = Color(0xFF555555);
  static const ink = Color(0xFF121212);

  static const blue = Color(0xFF6D8DF2);
  static const pink = Color(0xFFF680BF);
  static const mint = Color(0xFF6ED39A);
  static const lemon = Color(0xFFF2D24C);
  static const peach = Color(0xFFF7965C);
  static const lavender = Color(0xFFB39CF2);
  static const audio = Color(0xFF3C6B5A);

  static const sans = 'Inter';
  static const mono = 'Menlo';

  static const wordmark = TextStyle(fontFamily: sans, fontSize: 26, color: text, fontWeight: FontWeight.w600, letterSpacing: -0.6, height: 1);
  static const tagline = TextStyle(fontFamily: mono, fontSize: 10.5, color: muted, height: 1.4);
  static const lcd = TextStyle(fontFamily: mono, fontSize: 13, color: text, height: 1, letterSpacing: 0.3);
  static const lcdSmall = TextStyle(fontFamily: mono, fontSize: 10, color: muted, height: 1, letterSpacing: 0.4);
  static const segment = TextStyle(fontFamily: mono, fontSize: 11, color: text2, letterSpacing: 0.8, height: 1);
  static const tab = TextStyle(fontFamily: mono, fontSize: 10.5, color: muted, letterSpacing: 0.5, height: 1);
  static const tabOn = TextStyle(fontFamily: mono, fontSize: 10.5, color: text, letterSpacing: 0.5, height: 1);
  static const section = TextStyle(fontFamily: mono, fontSize: 10.5, color: muted, letterSpacing: 0.8, height: 1);
  static const tileLabel = TextStyle(fontFamily: sans, fontSize: 11.5, color: text2, height: 1);
  static const tileLabelInk = TextStyle(fontFamily: sans, fontSize: 11.5, color: ink, fontWeight: FontWeight.w500, height: 1);
  static const list = TextStyle(fontFamily: sans, fontSize: 12.5, color: text2, height: 1);
  static const listOn = TextStyle(fontFamily: sans, fontSize: 12.5, color: text, height: 1);
  static const body = TextStyle(fontFamily: sans, fontSize: 12.5, color: text, height: 1);
  static const bodyMed = TextStyle(fontFamily: sans, fontSize: 12.5, color: text, fontWeight: FontWeight.w500, height: 1);
  static const small = TextStyle(fontFamily: sans, fontSize: 11, color: muted, height: 1);
  static const num = TextStyle(fontFamily: mono, fontSize: 11.5, color: text, height: 1);
  static const numMuted = TextStyle(fontFamily: mono, fontSize: 10, color: muted, height: 1);
  static const title = TextStyle(fontFamily: sans, fontSize: 18, color: text, fontWeight: FontWeight.w600, letterSpacing: -0.3, height: 1);
  static const instrument = TextStyle(fontFamily: sans, fontSize: 17, color: text, fontWeight: FontWeight.w600, letterSpacing: -0.2, height: 1);
  static const row = TextStyle(fontFamily: sans, fontSize: 14, color: text, fontWeight: FontWeight.w500, height: 1);
  static const ruler = TextStyle(fontFamily: mono, fontSize: 9.5, color: muted, height: 1);
  static const track = TextStyle(fontFamily: sans, fontSize: 12, color: text2, height: 1);
  static const trackGroup = TextStyle(fontFamily: sans, fontSize: 12.5, color: text, fontWeight: FontWeight.w500, height: 1);
}

class Rule extends StatelessWidget {
  const Rule({super.key, this.vertical = false, this.color = P.rule});
  final bool vertical;
  final Color color;
  @override
  Widget build(BuildContext context) => vertical ? Container(width: 1, color: color) : Container(height: 1, color: color);
}

// A key: something you press. Flat, sharp, one thin rule.
class Cap extends StatelessWidget {
  const Cap({super.key, this.child, this.color = P.key, this.border = P.rule2, this.width, this.height, this.padding, this.alignment = Alignment.center});
  final Widget? child;
  final Color color, border;
  final double? width, height;
  final EdgeInsets? padding;
  final Alignment alignment;
  @override
  Widget build(BuildContext context) => Container(
        width: width,
        height: height,
        padding: padding,
        alignment: alignment,
        decoration: BoxDecoration(color: color, border: Border.all(color: border), borderRadius: BorderRadius.circular(2)),
        child: child,
      );
}

// Switch: a short bar with a square knob. Sharp, not a pill.
class Switch1 extends StatelessWidget {
  const Switch1({super.key, required this.on, this.color = P.blue, this.w = 30, this.h = 16});
  final bool on;
  final Color color;
  final double w, h;
  @override
  Widget build(BuildContext context) => Container(
        width: w,
        height: h,
        padding: const EdgeInsets.all(2),
        decoration: BoxDecoration(color: on ? color : const Color(0xFF2E2E2E), borderRadius: BorderRadius.circular(2), border: Border.all(color: on ? color : P.rule2)),
        alignment: on ? Alignment.centerRight : Alignment.centerLeft,
        child: Container(width: h - 6, height: h - 6, decoration: BoxDecoration(color: on ? P.ink : P.muted, borderRadius: BorderRadius.circular(1))),
      );
}

class Kebab extends StatelessWidget {
  const Kebab({super.key, this.color = P.muted});
  final Color color;
  @override
  Widget build(BuildContext context) => Column(
        mainAxisSize: MainAxisSize.min,
        children: List.generate(3, (_) => Container(width: 3, height: 3, margin: const EdgeInsets.symmetric(vertical: 1.5), decoration: BoxDecoration(color: color, shape: BoxShape.circle))),
      );
}

// A dot that is the same colour as the thing it names, wherever it appears.
class Ident extends StatelessWidget {
  const Ident(this.color, {super.key, this.size = 12});
  final Color color;
  final double size;
  @override
  Widget build(BuildContext context) => Container(width: size, height: size, decoration: BoxDecoration(color: color, shape: BoxShape.circle));
}
