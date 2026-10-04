part of 'inspector_parts.dart';

// ---- idea: text -------------------------------------------------------------------------------------------------------
/// contract: mirrors motolii-doc store/text.rs — runs over the content, each with its own style (size, wght axis, fill);
/// equal neighbours always merge (adjacent runs never share a style, 裁定89).
typedef _TxSt = ({double size, double w, String fill});
typedef _TxR = (String, _TxSt);

const _txAligns = [TextAlign.left, TextAlign.center, TextAlign.right];
const _txBase = (size: 48.0, w: 400.0, fill: '#F2F2F2');
const _txHi = Color(0x667A87E3);

/// (key, label, min, max, perPx, step, unit)
const _txProps = [('sz', 'Size', 4.0, 400.0, .5, 1.0, 'px'), ('w', 'Weight', 100.0, 900.0, 4.0, 100.0, '')];

final _txSeed = _txEnc([
  ('Hero ', (size: 96.0, w: 800.0, fill: '#F2F2F2')),
  ('title', (size: 96.0, w: 800.0, fill: '#B3C66B')),
  ('\nSmall move ', (size: 36.0, w: 400.0, fill: '#C1C1C1')),
  ('changes', (size: 36.0, w: 700.0, fill: '#F2F2F2')),
  (' everything.', (size: 36.0, w: 400.0, fill: '#C1C1C1')),
]);

String _txEnc(List<_TxR> rs) => rs.map((r) => [r.$2.size, r.$2.w, r.$2.fill, r.$1].join('\u001f')).join('\u001e');
List<_TxR> _txDec(String s) => [
  for (final p in s.isEmpty ? const <String>[] : s.split('\u001e'))
    if (p.split('\u001f') case [final a, final b, final c, ...final t]) (t.join('\u001f'), (size: double.parse(a), w: double.parse(b), fill: c)),
];
List<_TxR> _txRuns(Doc d) => _txDec(d.s2['tx_runs'] ?? '');
void _txPut(Doc d, List<_TxR> rs) => d.str('tx_runs', _txEnc(rs));
String _txStr(List<_TxR> rs) => rs.map((r) => r.$1).join();
List<_TxSt> _txPer(List<_TxR> rs) => [
  for (final r in rs)
    for (var i = 0; i < r.$1.length; i++) r.$2,
];

List<_TxR> _txJoin(String t, List<_TxSt> st) {
  final out = <_TxR>[];
  var from = 0;
  for (var i = 1; i <= t.length; i++) {
    if (i == t.length || st[i] != st[from]) {
      out.add((t.substring(from, i), st[from]));
      from = i;
    }
  }
  return out;
}

List<_TxR> _txMap(List<_TxR> rs, int a, int b, _TxSt Function(_TxSt) f) {
  final st = _txPer(rs);
  for (var i = math.max(0, a); i < math.min(b, st.length); i++) {
    st[i] = f(st[i]);
  }
  return _txJoin(_txStr(rs), st);
}

/// New text over old runs: the unchanged head and tail keep their styles; typed letters take the style of the letter before them.
List<_TxR> _txRetext(List<_TxR> rs, String nt) {
  final ot = _txStr(rs);
  if (ot == nt) return rs;
  final st = _txPer(rs);
  var p = 0, q = 0;
  while (p < ot.length && p < nt.length && ot[p] == nt[p]) {
    p++;
  }
  while (q < ot.length - p && q < nt.length - p && ot[ot.length - 1 - q] == nt[nt.length - 1 - q]) {
    q++;
  }
  final fill = p > 0 ? st[p - 1] : (st.isNotEmpty ? st.first : _txBase);
  return _txJoin(nt, [...st.take(p), for (var i = 0; i < nt.length - p - q; i++) fill, ...st.skip(ot.length - q)]);
}

_TxSt _txWith(_TxSt s, {double? size, double? w, String? fill}) => (size: size ?? s.size, w: w ?? s.w, fill: fill ?? s.fill);
double _txGet(_TxSt s, String k) => k == 'sz' ? s.size : s.w;

/// Size scales (a mixed selection keeps its proportions); weight moves by the same amount.
_TxSt _txBy(_TxSt s, String k, double old, double c) => k == 'sz'
    ? _txWith(s, size: ((old <= 0 ? c : s.size * c / old).clamp(4.0, 400.0) * 10).roundToDouble() / 10)
    : _txWith(s, w: (s.w + c - old).clamp(100.0, 900.0).roundToDouble());

/// The size most letters have: the text's base size.
double _txBaseSize(List<_TxR> rs) {
  final n = <double, int>{};
  for (final (t, s) in rs) {
    n[s.size] = (n[s.size] ?? 0) + t.replaceAll(RegExp(r'\s'), '').length;
  }
  return n.isEmpty ? _txBase.size : n.entries.reduce((a, b) => b.value > a.value ? b : a).key;
}

FontWeight _txW(double w) => FontWeight.values[((w / 100).round() - 1).clamp(0, 8)];

/// The field stays a field: 13 px at the base size, bigger letters grow gently and stop at 22 px.
TextStyle _txStyle(_TxSt s, Doc d, double base) {
  final px = (13 * math.sqrt(s.size / base)).clamp(10.0, 22.0);
  return TextStyle(
    fontFamily: d.s2['tx_font'] ?? 'Inter',
    fontFamilyFallback: const ['.AppleSystemUIFont'],
    fontSize: px,
    fontWeight: _txW(s.w),
    color: T.ink(_hex(s.fill) ?? Grey.g95),
    height: 1.3,
    decoration: TextDecoration.none,
  );
}

TextSpan _txSpan(List<_TxR> rs, Doc d, {TextRange? hi}) {
  final base = _txBaseSize(rs), kids = <TextSpan>[];
  var at = 0;
  for (final (t, s) in rs) {
    final st = _txStyle(s, d, base), e = at + t.length;
    if (hi != null && hi.start < e && hi.end > at) {
      final a = math.max(hi.start, at) - at, b = math.min(hi.end, e) - at;
      if (a > 0) kids.add(TextSpan(text: t.substring(0, a), style: st));
      kids.add(
        TextSpan(
          text: t.substring(a, b),
          style: st.copyWith(backgroundColor: _txHi),
        ),
      );
      if (b < t.length) kids.add(TextSpan(text: t.substring(b), style: st));
    } else {
      kids.add(TextSpan(text: t, style: st));
    }
    at = e;
  }
  return TextSpan(style: _txStyle(rs.isEmpty ? _txBase : rs.first.$2, d, base), children: kids);
}

String _txQuote(String s) {
  final t = s.replaceAll('\n', ' ').trim();
  return '“${t.length > 16 ? '${t.substring(0, 15)}…' : t}”';
}

class TextIdeas extends StatelessWidget {
  const TextIdeas({super.key});
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc..seed({'tx_align': 0}, strs: {'tx_runs': _txSeed, 'tx_font': 'Inter'});
    final rs = _txRuns(doc), open = !(doc.b['tx_fold'] ?? false);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      mainAxisSize: MainAxisSize.min,
      children: [
        const IdeaLabel(
          'A',
          'Field and palette',
          'Type in a plain field; the palette under it styles the selected letters, or all of them when nothing is selected.',
        ),
        Sect(
          title: 'Text',
          tone: Pop.toneText,
          mark: CardMark.text,
          open: open,
          onToggle: () => doc.flag('tx_fold', open),
          brief: [doc.s2['tx_font'] ?? 'Inter', '${brief(_txBaseSize(rs), 0)} px', '${rs.length} runs'],
          children: const [_TxEdit()],
        ),
      ],
    );
  }
}

class _TxCtl extends TextEditingController {
  Doc? doc;
  List<_TxR> runs = const [];
  TextRange? hi;
  @override
  TextSpan buildTextSpan({required BuildContext context, TextStyle? style, required bool withComposing}) {
    final d = doc;
    if (d == null) return TextSpan(text: text, style: style);
    final h = hi;
    return _txSpan(_txRetext(runs, text), d, hi: h != null && h.end <= text.length ? h : null);
  }
}

/// A plain input, and under it a palette that is always there.
/// contract: the selection outlives focus — using the palette keeps it, tinted, until the field is clicked again or All is pressed.
class _TxEdit extends StatefulWidget {
  const _TxEdit();
  @override
  State<_TxEdit> createState() => _TxEditState();
}

class _TxEditState extends State<_TxEdit> {
  final _ctl = _TxCtl(), _focus = FocusNode(debugLabel: 'tx');
  TextRange? _sel;
  bool _sync = false;

  @override
  void initState() {
    super.initState();
    _ctl.addListener(_onCtl);
    _focus.addListener(_onFocus);
  }

  @override
  void dispose() {
    _ctl.dispose();
    _focus.dispose();
    super.dispose();
  }

  void _onFocus() => setState(() {});

  void _onCtl() {
    if (_sync || !mounted) return;
    final doc = Ctx.read(context).doc, cur = _txRuns(doc);
    if (_ctl.text != _txStr(cur)) _txPut(doc, _txRetext(cur, _ctl.text));
    if (!_focus.hasFocus) return;
    final s = _ctl.selection, n = s.isValid && !s.isCollapsed ? TextRange(start: s.start, end: s.end) : null;
    if (n != _sel) setState(() => _sel = n);
  }

  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc, rs = _txRuns(doc), t = _txStr(rs);
    if (_ctl.text != t) {
      _sync = true;
      final s = _ctl.selection;
      _ctl.value = TextEditingValue(
        text: t,
        selection: s.isValid
            ? TextSelection(baseOffset: math.min(s.baseOffset, t.length), extentOffset: math.min(s.extentOffset, t.length))
            : const TextSelection.collapsed(offset: 0),
      );
      _sync = false;
    }
    final sel = _sel, s = sel == null || sel.end > t.length || sel.start >= sel.end ? null : sel;
    _ctl
      ..doc = doc
      ..runs = rs
      ..hi = _focus.hasFocus ? null : s;
    void edit(_TxSt Function(_TxSt) f) => _txPut(doc, s == null ? _txMap(rs, 0, t.length, f) : _txMap(rs, s.start, s.end, f));
    final first = rs.isEmpty ? _txBase : _txPer(rs)[s?.start ?? 0];
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Container(
          constraints: const BoxConstraints(minHeight: 64),
          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
          decoration: BoxDecoration(
            color: Pop.well,
            borderRadius: BorderRadius.circular(6),
            border: Border.all(color: _focus.hasFocus ? Grey.g38 : const Color(0x00000000)),
          ),
          child: EditableText(
            controller: _ctl,
            focusNode: _focus,
            style: _txStyle(_txBase, doc, _txBase.size),
            cursorColor: Grey.g95,
            backgroundCursorColor: Grey.g20,
            selectionColor: _txHi,
            textAlign: _txAligns[(doc.get('tx_align') ?? 0).round().clamp(0, 2)],
            maxLines: null,
          ),
        ),
        const SizedBox(height: 6),
        _TxTarget(s == null ? null : _txQuote(t.substring(s.start, s.end)), () => setState(() => _sel = null)),
        const SizedBox(height: 4),
        Row(
          children: [
            Expanded(
              child: PanelLink(to: LinkTo.fonts, value: doc.s2['tx_font'] ?? 'Inter'),
            ),
            const SizedBox(width: Pop.tileGap),
            const _TxAlign(),
          ],
        ),
        const SizedBox(height: Pop.tileGap),
        Row(
          children: [
            Expanded(child: _txField(doc, 'tx_sz', 'sz', first.size, edit)),
            const SizedBox(width: Pop.tileGap),
            Expanded(child: _txField(doc, 'tx_w', 'w', first.w, edit)),
          ],
        ),
        const SizedBox(height: Pop.tileGap),
        PanelLink(to: LinkTo.colors, label: 'Fill', value: first.fill, lead: LinkSwatch(first.fill)),
      ],
    );
  }
}

/// Which letters the palette is about to change.
class _TxTarget extends StatelessWidget {
  const _TxTarget(this.quote, this.onAll);
  final String? quote;
  final VoidCallback onAll;
  @override
  Widget build(BuildContext context) {
    final q = quote;
    return SizedBox(
      height: 18,
      child: Row(
        children: [
          Expanded(
            child: Text(
              q == null ? 'Styling all text. Select letters to style just those.' : 'Styling $q',
              style: T.label(q == null ? Grey.g56 : Grey.g95),
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
            ),
          ),
          if (q != null)
            Hov(
              key: const ValueKey('tx_all'),
              onTap: onAll,
              builder: (_, hot) => Text('All', style: T.label(hot ? Grey.g95 : Grey.g63)),
            ),
        ],
      ),
    );
  }
}

/// A number that edits whatever [apply] is given, reading its old value from its own id.
Widget _txField(Doc d, String id, String k, double cur, void Function(_TxSt Function(_TxSt)) apply) {
  final p = _txProps.firstWhere((p) => p.$1 == k);
  d.v[id] = cur;
  d.d.putIfAbsent(id, () => cur);
  return NumField(
    id: id,
    label: p.$2,
    unit: p.$7,
    min: p.$3,
    max: p.$4,
    perPx: p.$5,
    step: p.$6,
    onSet: (c) {
      final old = d.v[id] ?? c;
      d.v[id] = c;
      apply((s) => _txBy(s, k, old, c));
    },
    onReset: () => apply((s) => _txBy(s, k, cur, _txGet(_txBase, k))),
  );
}

class _TxAlign extends StatelessWidget {
  const _TxAlign();
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc, at = (doc.get('tx_align') ?? 0).round();
    return Container(
      width: 72,
      height: Pop.cell,
      padding: const EdgeInsets.all(2),
      decoration: BoxDecoration(color: Grey.g07, borderRadius: BorderRadius.circular(6)),
      child: Row(
        children: [
          for (var i = 0; i < 3; i++)
            Expanded(
              child: GestureDetector(
                key: ValueKey('tx_align_$i'),
                behavior: HitTestBehavior.opaque,
                onTap: () => doc.set('tx_align', i.toDouble()),
                child: Container(
                  alignment: Alignment.center,
                  decoration: BoxDecoration(
                    color: i == at ? Grey.g20 : null,
                    borderRadius: BorderRadius.circular(4),
                    border: Border(bottom: BorderSide(color: i == at ? Role.selected : const Color(0x00000000))),
                  ),
                  child: CustomPaint(size: const Size(12, 9), painter: _ParaJustPaint(i, i == at)),
                ),
              ),
            ),
        ],
      ),
    );
  }
}
