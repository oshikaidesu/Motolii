// Part of dock_make.dart (cut there by size, as it was).
part of 'dock_make.dart';

class _CopiesPanel extends StatelessWidget {
  const _CopiesPanel(this.c);
  final DockCtx c;
  @override
  Widget build(BuildContext context) {
    final picked = int.tryParse(c.ws.selected ?? '') != null, m = c.ws.inside;
    Widget slide(String label, String shown, double v, ValueChanged<double> to) => Padding(
      padding: EdgeInsets.only(bottom: Shape.s(4)),
      child: Row(
        children: [
          SizedBox(width: Shape.z(54), child: Text(label, style: T.label(Grey.g76))),
          Expanded(
            child: LayoutBuilder(
              builder: (context, box) {
                void at(Offset p) => c.set(() {
                  to((p.dx / box.maxWidth).clamp(0.0, 1.0));
                  if (picked) _lay(c);
                });
                return GestureDetector(
                  supportedDevices: wsPressing,
                  behavior: HitTestBehavior.opaque,
                  onPanDown: (d) {
                    if (picked) m?.later('together', number: 1);
                    at(d.localPosition);
                  },
                  onPanUpdate: (d) => at(d.localPosition),
                  onPanEnd: (_) => picked ? c.ws.after(m!.later('together')) : null,
                  onPanCancel: () => picked ? c.ws.after(m!.later('together')) : null,
                  child: SizedBox(
                    height: Shape.z(16),
                    child: Stack(
                      alignment: Alignment.centerLeft,
                      children: [
                        Container(height: 4, decoration: BoxDecoration(color: WsT.well, borderRadius: BorderRadius.all(Shape.r(2)))),
                        FractionallySizedBox(widthFactor: v, child: Container(height: 4, decoration: BoxDecoration(color: WsT.accent, borderRadius: BorderRadius.all(Shape.r(2))))),
                      ],
                    ),
                  ),
                );
              },
            ),
          ),
          SizedBox(width: Shape.z(40), child: Text(shown, textAlign: TextAlign.right, style: T.value(Grey.g91))),
        ],
      ),
    );
    return Container(
      color: WsT.card,
      padding: EdgeInsets.fromLTRB(WsT.inset, Shape.s(8), WsT.inset, Shape.s(8)),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Row(
            children: [
              Expanded(child: Text('Repeater', style: T.name(Grey.g95).copyWith(fontWeight: FontWeight.w700))),
              Text(picked ? 'on the picked thing, live' : 'pick a thing to lay out', style: T.label(Grey.g56)),
            ],
          ),
          SizedBox(height: Shape.s(6)),
          slide('Count', '${_copyCount.round()}', (_copyCount - 1) / 39, (f) => _copyCount = 1 + f * 39),
          slide('Rows', '${(_copyCount / _copyColumns.round().clamp(1, _copyCount.round())).ceil()}', (_copyColumns - 1) / 19, (f) => _copyColumns = 1 + f * 19),
          slide('Bend', '${(_copyBend * 360).round()}°', _copyBend, (f) => _copyBend = f),
        ],
      ),
    );
  }
}

// ---- a recipe's motion: asked of the inside once (create_preview.dart), on a document of its own.
bool _isRecipe(DockCtx c, String name) {
  final m = c.ws.inside;
  if (m == null || !m.recipes.any((r) => r.$1 == name)) return false;
  RecipeMotions.ask(File(m.libPath).parent.parent.path, [for (final r in m.recipes) r.$1]);
  return true;
}

/// The colour's number, typed: After Effects' Color Picker and Figma's both take one, #d4a72c, d4a72c or #da2. Told on Return or on leaving it.
class _HexField extends StatefulWidget {
  const _HexField(this.hex, this.onTold);
  final String hex;
  final ValueChanged<Color> onTold;
  @override
  State<_HexField> createState() => _HexFieldState();
}

class _HexFieldState extends State<_HexField> {
  final _c = TextEditingController();
  final _f = FocusNode(debugLabel: 'colour-hex');

  void _tell() {
    final said = _c.text.trim().replaceFirst('#', ''), whole = said.length == 3 ? [for (final k in said.split('')) '$k$k'].join() : said;
    if (whole.length != 6 || _colourOf(whole) == null || '#${whole.toLowerCase()}' == widget.hex.toLowerCase()) return;
    widget.onTold(_colourOf(whole)!);
  }

  @override
  void initState() {
    super.initState();
    _f.addListener(() {
      if (!_f.hasFocus) return _tell();
      WidgetsBinding.instance.addPostFrameCallback((_) => mounted ? _c.selection = TextSelection(baseOffset: 0, extentOffset: _c.text.length) : null);
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
    if (!_f.hasFocus) _c.text = widget.hex;
    return TextHand(field: (k) => EditableText(key: k, rendererIgnoresPointer: true,
      controller: _c,
      focusNode: _f,
      style: T.value(Grey.g91).copyWith(fontWeight: FontWeight.w600),
      cursorColor: WsT.accent,
      backgroundCursorColor: Grey.g44,
      selectionColor: Grey.g38,
      cursorWidth: 1.5,
      maxLines: 1,
      onSubmitted: (_) => _f.unfocus(),
    ));
  }
}
