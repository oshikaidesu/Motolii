// workspace_cards.dart: the inspector's cards, made from what the inside says a thing has (its cards, their values' ranges and looks, the kind's word).
part of 'workspace.dart';

/// Joined: the document's word for what a thing is, where the inspector's kinds have none (film, picture, solid).
String? _kindWord(Ws ws, String? id) {
  final t = int.tryParse(id ?? ''), w = t == null ? '' : ws.inside?.word(t, 'kind') ?? '';
  if (w == 'behaviour') return wsRelations.contains(ws.inside?.word(t!, 'file')) ? 'Relation' : 'Behaviour';
  return const {'film', 'picture', 'solid', 'light'}.contains(w) ? w.replaceRange(0, 1, w[0].toUpperCase()) : null;
}

Kind _kindOf(WsLayer? l) => switch (l?.kind) {
  null || WsKind.audio => Kind.none,
  WsKind.text => Kind.text,
  WsKind.camera => Kind.camera,
  WsKind.group => Kind.group,
  _ => Kind.shape,
};

/// The seats a thing has: the document's own names, and the inspector's numbers whose value it holds (a light has no
/// scale and no opacity, a flat thing no scale z: those are not shown).
Set<String> _seatsOf(Set<String> names) => {...names, for (final n in names) 'v_$n', 'depth'}; // (a hand-drawn card asks by the value's id, 'v_x')

/// Joined: the colours on the inside's own cards, as the inspector writes a colour (none said: a dash).
Map<String, String> _cardColours(Motolii m, int t, [double at = 0]) => {
  for (final (_, names) in m.cards(t))   // (an effect's colour, three numbers shown as one: _makeCards; at the moment drawn, as the card's numbers are)
    for (final n in names)
      if (n.endsWith('_x') && names.contains('${n.substring(0, n.length - 2)}_y') && names.contains('${n.substring(0, n.length - 2)}_z') && (m.shown(n)?.label ?? '').endsWith(' r'))
        'v_${n.substring(0, n.length - 2)}': '#${[for (final c in 'xyz'.split('')) ((m.value(t, '${n.substring(0, n.length - 2)}_$c', at) ?? 0).clamp(0, 1) * 255).round().toRadixString(16).padLeft(2, '0')].join().toUpperCase()}',
  for (final (_, names) in m.cards(t))
    for (final n in names)
      if (m.words(t, n).isEmpty && m.range(t, n) == null)
        'v_$n': m.shown(n)?.look == 'words'
            ? m.word(t, n) // (words as they are, to be typed over)
            : m.word(t, n).isEmpty || m.word(t, n) == 'none'
            ? '—'
            : m.shown(n)?.look == 'file'
            ? m.word(t, n).split('/').last // (a file by its name)
            : m.shown(n)?.look == 'numbers'
            ? m.word(t, n) // (numbers, one for each copy, as they are)
            : m.word(t, n).split(' ').where((c) => c.isNotEmpty).map((w) => w.split('@').first).map((c) => int.tryParse(c, radix: 16) != null && (c.length == 6 || c.length == 8) ? '#${c.toUpperCase()}' : c.replaceRange(0, 1, c[0].toUpperCase())).join(' '),   // (a gradient's stop said by its colour, where it stands left out) // (a colour, or colours in their order; a word that is not one, a material, as it is: Glass)
};

/// Joined: the cards the inside shows a thing's values on beyond the sample's (motolii_cards), each as Params in the
/// effects' manner: a number its range and what it is now, a word the words it may be. Nothing of them is typed here.
List<Fx>? _valueCards(Ws ws, String? id) {
  final m = ws.inside, t = id == null ? 0 : int.tryParse(id); // (nothing picked: the stage's own, thing 0)
  if (m == null || t == null) return null;
  // (made again only when the cards it has, their values or the things it may name change: a number let go of changes
  //  none of them, and the numbers the cards show are read from wsTransform; what a card says a number is now, its
  //  @default, the inspector takes only when it makes its cards anew)
  final shape = '${m.cards(t)}|${ws.layers.map((l) => l.name).join(',')}';
  final made = _cardsMade[t];
  if (made != null && made.$1 == shape && identical(made.$3, m)) return made.$2;
  final cards = _makeCards(ws, m, t);
  _cardsMade[t] = (shape, cards, m);
  return cards;
}

final _cardsMade = <int, (String, List<Fx>, Motolii)>{};

List<Fx> _makeCards(Ws ws, Motolii m, int t) {
  final at = _drawn(ws);   // (as the picture shown has it)
  String field(String n) {
    final sh = m.shown(n), label = '@label(${sh?.label ?? n.replaceAll('_', ' ').replaceRange(0, 1, n[0].toUpperCase())})';
    if (sh?.look == 'thing') {
      // (another thing, by the names the timeline shows: None, then the others; its number is the document's)
      final others = _others(ws, t), at = others.values.toList().indexOf((m.value(t, n, 0) ?? 0).round());
      return '  /// $label @enum(${['None', ...others.keys.map((o) => o.replaceAll(',', ' '))].join(', ')}) @default(${at + 1})\n  $n: u32,';
    }
    if (m.words(t, n) case final words when words.isNotEmpty) {
      return '  /// $label @enum(${words.join(', ')}) @default(${math.max(0, words.indexOf(m.word(t, n)))})\n  $n: u32,';
    }
    if (m.range(t, n) case (final lo0, final hi0)) {
      final a = _shown(m, t, n, sh != null && sh.high > sh.low ? sh.low : lo0), b = _shown(m, t, n, sh != null && sh.high > sh.low ? sh.high : hi0), lo = math.min(a, b), hi = math.max(a, b);
      final ui = switch (sh?.look) {
        'onoff' => '@toggle',
        'angle' => '@ui(angle)',
        'seed' => '@ui(seed)',
        'count' => hi - lo <= 12 ? '@ui(stepper)' : '@ui(bar)',
        'share' when lo >= 0 && hi <= 100 => '@ui(percent)',
        _ => lo < 0 && hi > 0 ? '@ui(bipolar)' : '@ui(bar)',
      };
      final whole = sh?.look == 'count' || sh?.look == 'onoff' || sh?.look == 'seed';
      return '  /// $label $ui @range($lo, $hi)${sh?.open == true ? ' @open' : ''} @unit(${sh?.unit ?? ''}) @default(${_shown(m, t, n, m.value(t, n, at) ?? lo0)})\n  $n: ${whole ? 'u32' : 'f32'},';
    }
    return '  /// $label${const ['numbers', 'colours', 'words', 'file'].contains(sh?.look) ? ' @unit(${sh!.look})' : ''}\n  $n: vec3<f32>,'; // (any word: colours, six digits each, given from the Colors shelf; or numbers, one for each copy)
  }

  // (an effect's colour, ISF's one input the inside holds as three numbers (_x _y _z) and an alpha: one colour on its
  //  card, as After Effects' and Alight Motion's effect colour is, the Colors shelf said to it (the inside's say puts the
  //  three); golf_noise.md (83): white took three number fields)
  String? colourOf(String n, List<String> names) => n.endsWith('_x') && names.contains('${n.substring(0, n.length - 2)}_y') && names.contains('${n.substring(0, n.length - 2)}_z') && (m.shown(n)?.label ?? '').endsWith(' r') ? n.substring(0, n.length - 2) : null;
  String colour(String base) => '  /// @label(${m.shown('${base}_x')!.label.substring(0, m.shown('${base}_x')!.label.length - 2)}) @unit(colours)\n  $base: vec3<f32>,';
  return [
    for (final (title, names) in m.cards(t))
      Fx('v', title, 'struct Params {\n${[
        for (final n in names)
          if (m.shown(n)?.fold == null)
            if (colourOf(n, names) case final base?) colour(base)
            else if (!{for (final x in names) if (colourOf(x, names) case final b?) ...['${b}_y', '${b}_z']}.contains(n)) field(n),
        for (final fold in {for (final n in names) ?m.shown(n)?.fold}) ...[
          '  // @advanced($fold)',
          for (final n in names) if (m.shown(n)?.fold == fold) field(n),
        ],
      ].join('\n')}\n}', toneOf: () => Pop.toneLayer, mark: CardMark.layer),
  ];
}

/// Joined: how many of a knob's units make one of the document's (the inside's shown look: a length of the stage in
/// points of the picture, a share in hundredths; else as it is).
/// A value of the inside's own cards as the inspector counts it: another thing by its place (None first), a number
/// in the card's units, a word by its place among those it may be (the stage's and a thing's alike).
double _onCard(Ws ws, Motolii m, int t, String n, double at) {
  final seen = const {'x', 'y', 'z', 'rotate', 'scale_x', 'scale_y', 'opacity'}.contains(n) ? m.shows(t, n, at) : null;   // (the values the moment's copy carries as the stage shows them: picture.h)
  if (seen != null) return _shown(m, t, n, seen.$1);   // (where it shows: a hand carrying it, a relation moving it, as After Effects shows a value an expression drives)
  return m.shown(n)?.look == 'thing' ? (_others(ws, t).values.toList().indexOf((m.value(t, n, 0) ?? 0).round()) + 1).toDouble() : m.words(t, n).isEmpty ? _shown(m, t, n, m.value(t, n, at) ?? 0) : math.max(0, m.words(t, n).indexOf(m.word(t, n))).toDouble(); // (the inside's own cards: a number as it is, a word by its place among those it may be)
}
