// Part of panel_inspector_a_parts.dart (cut there by size, as it was).
part of 'panel_inspector_a.dart';

Widget _pI5Copy() {
  final iv = _Iv('Position X', '00:00', '00:12', _pe('Back Out'));
  final ivLong = _Iv('Wave Height Offset', '01:00', '02:00', _pe('Quart In Out'));
  return _sheet('I5 copy and paste', 'the two actions (24 px high, 4 px corners like every field), the clipboard chip that shows what it holds, and the interval list rows.', [
    ('Action button 122 x 24', [
      _Spec('on, rest (g20, edge g44)', 122, _Act('Copy ease', on: true, onTap: _noop)),
      _Spec('hover (g26, edge g63)', 122, _Act('Copy ease', on: true, hot: true, onTap: _noop)),
      _Spec('focus (edge g95)', 122, _Act('Copy ease', on: true, focus: true, onTap: _noop)),
      _Spec('pressed (sinks to g13, accent line)', 122, _Act('Copy ease', on: true, pressed: true, onTap: _noop)),
      _Spec('off (needs one interval)', 122, _Act('Copy ease', on: false, onTap: _noop)),
      _Spec('with a count', 122, _Act('Paste to 12', on: true, onTap: _noop)),
      _Spec('dock label', 122, _Act('Paste 12', on: true, onTap: _noop)),
    ]),
    ('Clipboard chip, 280 px', [
      _Spec('empty: a dashed slot', _panelIn, const _Clip(null)),
      _Spec('holds an ease: its thumb, name, numbers', _panelIn, _Clip(iv.e)),
    ]),
    ('Interval row, 280 px', [
      _Spec('rest', _panelIn, _gut(_IvRow(iv))),
      _Spec('hover (g15)', _panelIn, _gut(_IvRow(iv, hot: true))),
      _Spec('focus (1 px g76 edge)', _panelIn, _gut(_IvRow(iv, focus: true))),
      _Spec('pressed (g26)', _panelIn, _gut(_IvRow(iv, press: true))),
      _Spec('selected (g20 + tick)', _panelIn, _gut(_IvRow(iv, on: true))),
      _Spec('long property name', _panelIn, _gut(_IvRow(ivLong))),
      _Spec('dock 218: no time column', _dockIn, _gut(_IvRow(iv, dock: true))),
    ]),
  ], none: 'The clipboard chip is a read-out: no hover or focus. A button has no selected state (it acts once).');
}

// ================= I6 =================

Widget _pI6Tone() {
  _Spec row(String cap, String id, {Map<String, double> start = const {}, bool dock = false}) => _Spec(cap, dock ? _dockIn : _panelIn, _one((m) => _gut(_i6Label(m.rows[id]!, m, dock: dock)), start: start));
  return _sheet('I6 tone and filter', 'a label is g76 at default; changed = g95 with a dotted underline (and a corner mark on its field). Plus the count words, the filter bar and the empty answer.', [
    ('Row label, 280 px', [
      row('default (g76)', 'anchor.x'),
      row('changed (g95, dotted underline)', 'pos.x', start: const {'pos.x': 1180.5}),
      row('changed, one of a pair (the row lights)', 'turb.x', start: const {'turb.y': 700}),
      row('long name', 'turb.r', start: const {'turb.r': 3}),
      row('dock 218, changed', 'pos.x', start: const {'pos.x': 1180.5}, dock: true),
    ]),
    ('Count words', [
      _Spec('head: changed (g95)', 64, _count('7 changed', tone: N.g95)),
      _Spec('head: none (g76)', 64, _count('0 changed', tone: N.g76)),
      _Spec('group: "3 changed"', 64, _count('3 changed')),
      _Spec('group header with the count', _panelIn, _gut(_Sec('Transform  トランスフォーム', const [], first: true, trailing: _count('3 changed')))),
      _Spec('panel head with the count', _panelIn, _Head(name: 'Jewel Field', trailing: _count('7 changed', tone: N.g95))),
    ]),
    ('Filter bar', [
      _Spec('on: changed only, 280', _panelIn, const _FilterBar(on: true, n: 7, total: 31)),
      _Spec('off: all rows, 280', _panelIn, const _FilterBar(on: false, n: 7, total: 31)),
      _Spec('dock 218', _dockIn, const _FilterBar(on: true, n: 7, total: 31)),
    ]),
    ('Empty answer', [
      _Spec('nothing changed: one word, one button (rest)', _panelIn, const _EmptyCard()),
      _Spec('the button, hover', _panelIn, const _EmptyCard(hover: true)),
      _Spec('the button, focused', _panelIn, const _EmptyCard(focus: true)),
      _Spec('the button, pressed', _panelIn, const _EmptyCard(pressed: true)),
    ]),
  ], none: 'A label has no hover look except in the label grammar (I2-a): in I6 the label is read, and a double-click resets it. The count words are text: no states. The filter switch is the lab\'s PillSwitch.');
}

Widget _pI6Reset() {
  Widget ghostRow(String id, Map<String, String> ghost, {Map<String, double> start = const {}, bool dock = false}) => _one((m) => _gut(_i6Label(m.rows[id]!, m, ghost: ghost, dock: dock)), start: start);
  const names = ['Default', 'Before this session', 'Snapshot “v2 soft glow”'], subs = ['the layer\'s own defaults', 'the state when you opened the layer', 'saved 00:41 ago'];
  Widget head(Widget t) => _Head(name: 'Jewel Field', trailing: t);
  Widget gh(int k, {bool hover = false, bool focus = false, bool pressed = false}) => _gut(_Sec('Transform  トランスフォーム', const [], first: true, trailing: _GroupReset(k, hover: hover, focus: focus, pressed: pressed)));
  return _sheet('I6 reset', 'the group reset button, the three targets, the header trigger, and the ghost values a hovered or focused target previews on the rows.', [
    ('Group reset button, 280 px', [
      _Spec('3 changed: rest', _panelIn, gh(3)),
      _Spec('hover', _panelIn, gh(3, hover: true)),
      _Spec('focus', _panelIn, gh(3, focus: true)),
      _Spec('pressed', _panelIn, gh(3, pressed: true)),
      _Spec('nothing to reset: no count, a dead button', _panelIn, gh(0)),
    ]),
    ('Header trigger, 280 px', [
      _Spec('rest', _panelIn, head(const _Btn('Reset to…'))),
      _Spec('hover', _panelIn, head(const _Btn('Reset to…', hover: true))),
      _Spec('focus', _panelIn, head(const _Btn('Reset to…', focus: true))),
      _Spec('pressed', _panelIn, head(const _Btn('Reset to…', pressed: true))),
      _Spec('open (the targets are out)', _panelIn, head(const _Btn('Reset to…', open: true))),
    ]),
    ('Target, 280 px', [
      _Spec('rest', _panelIn, _gut(_Target(names[0], subs[0], 5))),
      _Spec('hover (g15; ghosts show on the rows)', _panelIn, _gut(_Target(names[0], subs[0], 5, hover: true))),
      _Spec('focus (g20 + tick; ghosts show too)', _panelIn, _gut(_Target(names[0], subs[0], 5, focus: true))),
      _Spec('pressed (g26 + tick)', _panelIn, _gut(_Target(names[0], subs[0], 5, pressed: true))),
      _Spec('nothing would change', _panelIn, _gut(_Target(names[1], subs[1], 0))),
      _Spec('long name', _panelIn, _gut(_Target(names[2], subs[2], 12))),
      _Spec('dock 218: long name cut', _dockIn, _gut(_Target(names[2], subs[2], 12, focus: true))),
    ]),
    ('Ghost: the value a target would give, 280 px', [
      _Spec('a pair: → 1020 → 560', _panelIn, ghostRow('pos.x', const {'pos.x': '1020', 'pos.y': '560'}, start: const {'pos.x': 1180.5})),
      _Spec('a single number', _panelIn, ghostRow('op', const {'op': '80'}, start: const {'op': 100})),
      _Spec('a choice', _panelIn, ghostRow('blur.d', const {'blur.d': 'Vertical'})),
      _Spec('a toggle', _panelIn, ghostRow('blur.e', const {'blur.e': 'On'})),
      _Spec('dock 218, a pair (numbers only)', _dockIn, ghostRow('pos.x', const {'pos.x': '1020', 'pos.y': '560'}, start: const {'pos.x': 1180.5}, dock: true)),
    ]),
  ], none: 'A ghost value has no hover or focus look: it is a preview, never a control. A target has no selected state (a click acts at once). The count of values on a target is text.');
}
