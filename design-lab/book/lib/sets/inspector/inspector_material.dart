part of 'inspector_parts.dart';

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
      brief: [doc.s2['blend'] ?? _blends.first, if (doc.b['ghost'] ?? false) 'Ghost', if (doc.b['clip'] ?? false) 'Clip'],
      children: [
        const PropRow(
          id: 'blend',
          label: 'Blend',
          keyable: false,
          cells: [Chooser(id: 'blend', options: _blends, word: 'Modes')],
        ),
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
          id: 'clip',
          label: 'Clip to below',
          keyable: false,
          cells: [
            Align(
              alignment: Alignment.centerLeft,
              child: OnOff(on: doc.b['clip'] ?? false, onChanged: (v) => doc.flag('clip', v)),
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
    final doc = Ctx.of(context).doc, none = (doc.s2['stroke'] ?? 'None') == 'None';
    return Sect(
      title: 'Fill and stroke',
      first: false,
      tone: Pop.toneFill,
      keyIds: const ['stroke.w'],
      mark: CardMark.fill,
      brief: [doc.s2['fill'] ?? '#D9D2C3', none ? 'No stroke' : '${doc.s2['stroke']} ${brief(doc.get('stroke.w'))} px'],
      children: [
        PropRow(
          id: 'fill',
          label: 'Fill',
          keyable: false,
          cells: [PanelLink(to: LinkTo.colors, value: doc.s2['fill'] ?? '#D9D2C3', lead: LinkSwatch(doc.s2['fill'] ?? '#D9D2C3'))],
        ),
        const PropRow(
          id: 'stroke',
          label: 'Stroke',
          keyable: false,
          cells: [
            Chooser(id: 'stroke', options: ['None', '#F2F2F2', '#C1C1C1', '#8E8E8E'], word: 'Colours'),
          ],
        ),
        PropRow(
          id: 'stroke.w',
          label: 'Stroke width',
          enabled: !none,
          note: none ? 'Off while Stroke is None' : null,
          cells: [NumField(id: 'stroke.w', unit: 'px', decimals: 1, min: 0, max: 64, perPx: .1)],
        ),
      ],
    );
  }
}

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
  bool _init = false;

  @override
  void initState() {
    super.initState();
    _f.addListener(() {
      final x = Ctx.read(context);
      if (_f.hasFocus) {
        _entered = _c.text;
      } else {
        x.doc.str('text.content', _c.text);
      }
      setState(() {});
    });
  }

  @override
  void dispose() {
    _f.dispose();
    _c.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, ed = Ed(x.cfg.ed);
    if (!_init) {
      _c.text = doc.s2['text.content'] ?? '';
      _init = true;
    }
    const aligns = ['Left', 'Centre', 'Right', 'Justify'];
    return Sect(
      title: 'Text',
      first: widget.first,
      tone: Pop.toneText,
      mark: CardMark.text,
      hint: (doc.s2['text.content'] ?? '').replaceAll('\n', ' '),
      children: [
        Padding(
          padding: const EdgeInsets.symmetric(vertical: 2),
          child: Opacity(
            opacity: x.cfg.locked ? .5 : 1,
            child: IgnorePointer(
              ignoring: x.cfg.locked,
              child: Container(
                constraints: const BoxConstraints(minHeight: 48),
                padding: const EdgeInsets.all(8),
                decoration: BoxDecoration(
                  color: x.cfg.look == Look.quiet ? Grey.g13 : Grey.g07,
                  borderRadius: BorderRadius.circular(4),
                  border: Border.all(color: _f.hasFocus ? Grey.g63 : (x.cfg.look == Look.quiet && x.cfg.ed == Editorial.a ? const Color(0x00000000) : ed.rule)),
                ),
                child: EditableText(
                  controller: _c,
                  focusNode: _f,
                  style: T.name(Grey.g95).copyWith(fontWeight: FontWeight.w400, height: 1.4),
                  cursorColor: Grey.g95,
                  backgroundCursorColor: Grey.g20,
                  minLines: 2,
                  maxLines: 4,
                  selectionColor: Role.selected.withValues(alpha: .4),
                ),
              ),
            ),
          ),
        ),
        Cap('Applies when you leave the box · Esc restores', color: Grey.g56),
        PropRow(
          id: 'font',
          label: 'Font',
          keyable: false,
          cells: [PanelLink(to: LinkTo.fonts, value: doc.s2['font'] ?? '—')],
        ),
        const PropRow(
          id: 'text.size',
          label: 'Size',
          cells: [NumField(id: 'text.size', unit: 'px', decimals: 0, min: 1, max: 2000)],
        ),
        PropRow(
          id: 'align',
          label: 'Alignment',
          keyable: false,
          enabled: !x.cfg.locked,
          cells: [
            Dis(
              child: Segmented(items: aligns, index: aligns.indexOf(doc.s2['align'] ?? 'Left'), expand: true, onChanged: (i) => doc.str('align', aligns[i])),
            ),
          ],
        ),
      ],
    );
  }
}
