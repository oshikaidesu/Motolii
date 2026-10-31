import 'dart:async' show Timer;
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_rows_prop_row.dart';
import 'package:lab_book/sets/inspector/inspector_rows_sect.dart';

import '../../parts/controls.dart';
import '../../tokens.dart';
import 'inspector_parts.dart';
import '../foundations/text_hand.dart';

// ---- 9. material -------------------------------------------------------------------------------------------------------
const _blends = ['Normal', 'Add', 'Multiply', 'Screen', 'Overlay', 'Soft light', 'Hard light', 'Difference'];

class BlendRows extends StatelessWidget {
  const BlendRows({super.key, this.titled = true, this.first = false});
  final bool titled, first;
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    // contract: Blend opens the blend chooser; Ghost and Clip to below are On/Off words; hidden for a camera (G1).
    return Sect(
      title: titled ? 'Blend' : null,
      first: first,
      tone: Pop.toneBlend,
      mark: CardMark.blend,
      brief: [doc.s2['v_blend'] ?? _blends.first, if (doc.b['ghost'] ?? false) 'Ghost', if ((doc.get('v_clip') ?? 0) > .5) 'Clip'],
      children: [
        PropRow(
          id: 'v_blend',
          label: 'Blend',
          lead: const _MixGlyph(clip: false),
          keyable: false,
          cells: [Chooser(id: 'v_blend', options: doc.choices['v_blend'] ?? _blends)],
        ),
        if (doc.seat('ghost')) // (a seat the document has not, joined, is not shown: a switch that moves nothing is not a switch)
          PropRow(
            id: 'ghost',
            label: 'Ghost',
            keyable: false,
            cells: [
              Align(
                alignment: Alignment.centerLeft,
                child: OnOff(on: doc.b['ghost'] ?? false, onChanged: (v) => doc.flag('ghost', v)),
              ),
            ],
          ),
        PropRow(
          id: 'v_clip',
          label: 'Clip to below',
          lead: const _MixGlyph(clip: true),
          keyable: false,
          cells: [
            Align(
              alignment: Alignment.centerLeft,
              child: ListenableBuilder( // (a number shown as a switch: drawn again when it is told anew, as the shadow's drawing is)
                listenable: doc.watch('v_clip'),
                builder: (context, _) => OnOff(on: (doc.get('v_clip') ?? 0) > .5, onChanged: (v) => doc.set('v_clip', v ? 1 : 0)),
              ),
            ),
          ],
        ),
      ],
    );
  }
}

class FillStroke extends StatelessWidget {
  const FillStroke({super.key, this.first = true});
  final bool first;
  @override
  // contract: the field shows the value and hands over to the Colours tool aimed at THIS slot (I1). Stroke width counts only while a stroke exists.
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc, none = (doc.s2['v_stroke'] ?? '—') == '—', edge = doc.seat('v_stroke'); // (a solid or a light has no edge)
    final media = (doc.s2['v_fill_media'] ?? '—').split('/').last, fill = media == '—' || media.isEmpty ? doc.s2['v_fill'] ?? '—' : media;   // (filled with a picture, it says so: a colour picked takes its place)
    return Sect(
      title: edge ? 'Fill and stroke' : 'Fill',
      first: false,
      tone: Pop.toneFill,
      keyIds: const ['v_stroke_width'],
      mark: CardMark.fill,
      brief: [fill, if (edge) none ? 'No stroke' : '${doc.s2['v_stroke']} ${brief(doc.get('v_stroke_width'))} px'],
      children: [
        PropRow(
          id: 'v_fill',
          label: 'Fill',
          keyable: false,
          cells: [PanelLink(to: LinkTo.colors, id: 'v_fill', value: fill, lead: LinkSwatch(fill))],
        ),
        if (edge) ...[
        PropRow(
          id: 'v_stroke',
          label: 'Stroke',
          lead: const _LineGlyph(weight: false),
          keyable: false,
          cells: [PanelLink(to: LinkTo.colors, id: 'v_stroke', value: doc.s2['v_stroke'] ?? '—', lead: LinkSwatch(doc.s2['v_stroke'] ?? '—'))],
        ),
        PropRow(
          id: 'v_stroke_width',
          label: 'Stroke width',
          lead: const _LineGlyph(weight: true),
          enabled: !none,
          cells: [NumField(id: 'v_stroke_width', unit: 'px', decimals: 1, min: 0, max: 64, perPx: .1)],
        ),
        ],
      ],
    );
  }
}

/// Said by the stage as it places a text: the Text card next built for it holds the keys, its words all picked, so
/// what is typed at once is the text's (AbsentMotion's addText opens its EditText panel; Figma's text tool types where
/// it was placed).
bool textToType = false;

class TextGroup extends StatefulWidget {
  const TextGroup({super.key, this.first = true});
  final bool first;
  @override
  State<TextGroup> createState() => _TextGroupState();
}

class _TextGroupState extends State<TextGroup> {
  final _c = TextEditingController();
  late final FocusNode _f = FocusNode(
    debugLabel: 'text-content',
    onKeyEvent: (n, e) {
      if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape) {
        // contract: Esc restores the text as it was when the field was entered (A3).
        _c.text = _entered;
        _f.unfocus();
        return KeyEventResult.handled;
      }
      return KeyEventResult.ignored;
    },
  );
  String _entered = '';
  Timer? _typed;

  @override
  void initState() {
    super.initState();
    _f.addListener(() {
      final x = Ctx.read(context);
      if (_f.hasFocus) {
        _entered = _c.text;
      } else {
        x.doc.str('v_text', _c.text);
      }
      setState(() {});
    });
    _c.addListener(() {   // (its letters picked by the hand, kept for the Colors shelf to colour those alone: wordsPicked)
      if (!_f.hasFocus) return;
      _typed?.cancel();
      if (_c.text != Ctx.read(context).doc.s2['v_text']) _typed = Timer(const Duration(milliseconds: 250), () => mounted && _f.hasFocus ? Ctx.read(context).doc.str('v_text', _c.text) : null);   // (on the stage as it is typed, After Effects' and Figma's text tools; a pause apart, so a word is one step)
      final s = _c.selection;
      wordsPicked.value = s.isValid && !s.isCollapsed ? (text: _c.text, start: s.start, end: s.end) : null;
    });
  }

  @override
  void dispose() {
    _typed?.cancel();
    _f.dispose();
    _c.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc;
    if (!_f.hasFocus && _c.text != (doc.s2['v_text'] ?? ''))
      _c.text = doc.s2['v_text'] ?? ''; // (out of the box it is the text the host has now: another thing picked, an undo)
    if (textToType) {   // (once built with the placed text's words: the keys to it, its words picked)
      textToType = false;
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (!mounted) return;
        if (doc.b['fold.Text'] ?? false) {   // (a folded card has no field for the keys to go to: the next build takes them)
          textToType = true;
          return doc.flag('fold.Text', false);
        }
        _f.requestFocus();
        _c.selection = TextSelection(baseOffset: 0, extentOffset: _c.text.length);
      });
    }
    const aligns = ['Left', 'Centre', 'Right']; // (ThorVG lines up between 0 and 1: these are its three steps, in hundredths 0, 50, 100)
    return Sect(
      title: 'Text',
      first: widget.first,
      tone: Pop.toneText,
      mark: CardMark.text,
      hint: (doc.s2['v_text'] ?? '').replaceAll('\n', ' '),
      children: [
        Padding(
          padding: EdgeInsets.symmetric(vertical: Shape.s(2)),
          child: Opacity(
            opacity: x.cfg.locked ? .5 : 1,
            child: IgnorePointer(
              ignoring: x.cfg.locked,
              child: Container(
                constraints: BoxConstraints(minHeight: Shape.z(48)),
                padding: EdgeInsets.all(Shape.s(8)),
                decoration: BoxDecoration(
                  color: x.cfg.look == Look.quiet ? Grey.g13 : Grey.g07,
                  borderRadius: BorderRadius.all(Shape.r(4)),
                  border: Border.all(color: _f.hasFocus ? Grey.g63 : Grey.g26), // (a box you type in, seen as one even with nothing written)
                ),
                child: TextHand(key: const ValueKey('ws-text-words'), field: (k) => EditableText(key: k, rendererIgnoresPointer: true,
                  controller: _c,
                  focusNode: _f,
                  style: T.name(Grey.g95).copyWith(fontWeight: FontWeight.w400, height: 1.4),
                  cursorColor: Grey.g95,
                  backgroundCursorColor: Grey.g20,
                  minLines: 2,
                  maxLines: 4,
                  selectionColor: Role.selected.withValues(alpha: .4),
                )),
              ),
            ),
          ),
        ),
        PropRow(
          id: 'v_file',
          label: 'Font',
          keyable: false,
          cells: [PanelLink(to: LinkTo.fonts, value: doc.s2['v_file'] ?? '—')],
        ),
        const PropRow(
          id: 'v_size',
          label: 'Size',
          cells: [NumField(id: 'v_size', unit: 'px', decimals: 0, min: 1, max: 2000)],
        ),
        const PropRow(   // (beside the size, in After Effects' Character, Premiere's Essential Graphics, Figma's Text, CapCut's Spacing; Alight Motion alone keeps it apart as an effect: golf_noise.md head)
          id: 'v_letter_spacing',
          label: 'Tracking',
          cells: [NumField(id: 'v_letter_spacing', unit: '×', decimals: 2, min: 0, max: 10)],
        ),
        const PropRow(
          id: 'v_line_spacing',
          label: 'Line gap',
          cells: [NumField(id: 'v_line_spacing', unit: '×', decimals: 2, min: 0, max: 10)],
        ),
        PropRow(
          id: 'v_align',
          label: 'Alignment',
          keyable: false,
          enabled: !x.cfg.locked,
          cells: [
            Row(
              children: [
                for (final (i, w) in aligns.indexed) ...[
                  if (i > 0) SizedBox(width: Shape.s(4)),
                  Expanded(
                    child: GlyphPlate(w, alignGlyph(i / 2), on: ((doc.get('v_align') ?? 0) / 50).round() == i, onTap: () {
                      doc.beginGesture();
                      doc.setMany({'v_align': i * 50.0, 'v_anchor_x': i * 50.0});
                      doc.endGesture();
                    }), // (lines set left, centred, right, round where it stands: After Effects' point text; the glyph says it)
                  ),
                ],
              ],
            ),
          ],
        ),
      ],
    );
  }
}

/// How it lies over what is below, drawn as its value: blend, two squares whose overlap is as the mode makes it (lighter
/// for screen and add, darker for multiply, the top one alone for normal); clip, a disc seen only inside the square
/// beneath it (or whole, when not clipped).
class _MixGlyph extends StatelessWidget {
  const _MixGlyph({required this.clip});
  final bool clip;
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    return CustomPaint(size: const Size(24, 20), painter: _MixPaint(clip, clip ? ((doc.get('v_clip') ?? 0) > .5 ? 'on' : 'off') : (doc.s2['v_blend'] ?? 'normal').toLowerCase()));
  }
}

class _MixPaint extends CustomPainter {
  const _MixPaint(this.clip, this.how);
  final bool clip;
  final String how;
  @override
  void paint(Canvas c, Size s) {
    final below = const Rect.fromLTWH(2, 5, 12, 12), top = const Rect.fromLTWH(9, 2, 12, 12);
    final under = Paint()..color = const Color(0xFF6E8BC4), over = Paint()..color = const Color(0xFFE0A35A);
    c.drawRect(below, under);
    if (clip) {
      c.save();
      if (how == 'on') c.clipRect(below); // (seen only through the one beneath)
      c.drawCircle(top.center, 6, over);
      c.restore();
      return;
    }
    c.drawRect(top, over);
    final both = below.intersect(top);
    switch (how) {
      case 'screen' || 'add':
        c.drawRect(both, Paint()..color = const Color(0xFFFFF1D6)); // (lighter where they meet)
      case 'multiply':
        c.drawRect(both, Paint()..color = const Color(0xFF3B3524)); // (darker where they meet)
      default: // (normal: the top one covers)
    }
  }

  @override
  bool shouldRepaint(_MixPaint o) => o.how != how || o.clip != clip;
}

/// The edge, drawn: a square's outline (its colour), or three lines heavier each (how wide: the line-weight mark every
/// drawing program shows).
class _LineGlyph extends StatelessWidget {
  const _LineGlyph({required this.weight});
  final bool weight;
  @override
  Widget build(BuildContext context) => CustomPaint(size: const Size(24, 20), painter: _LinePaint(weight));
}

class _LinePaint extends CustomPainter {
  const _LinePaint(this.weight);
  final bool weight;
  @override
  void paint(Canvas c, Size s) {
    final ink = Paint()
      ..color = Grey.g76
      ..style = PaintingStyle.stroke
      ..strokeCap = StrokeCap.round;
    if (!weight) {
      c.drawRect(const Rect.fromLTWH(5, 4, 13, 12), ink..strokeWidth = 2);
      return;
    }
    for (final (y, w) in const [(5.0, 1.0), (10.0, 2.2), (15.5, 3.6)]) {
      c.drawLine(Offset(4, y), Offset(20, y), ink..strokeWidth = w);
    }
  }

  @override
  bool shouldRepaint(_LinePaint o) => o.weight != weight;
}
