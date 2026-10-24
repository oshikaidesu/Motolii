// Part of panel_inspector_b_sheets.dart (cut there by size, as it was).
part of 'panel_inspector_b.dart';

/// A specimen of one card header of the stack (I8): number, grip, chevron, name, status word, switch.
Widget _cardHead(String name, {bool sel = false, bool on = true, String? status, Color statusColor = N.g76, bool muted = false, bool failed = false, bool open = false}) => Padding(
      padding: const EdgeInsets.fromLTRB(12, 0, 12, 0),
      child: Container(
        decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(4)),
        child: _Gut(
          tick: failed ? Role.error : (sel ? N.g95 : null),
          child: Opacity(
            opacity: muted ? .6 : 1,
            child: SizedBox(
              height: status == null ? 32 : 40,
              child: Row(children: [
                const SizedBox(width: 6),
                SizedBox(width: 20, child: Center(child: Text('02', style: _cnt(N.g76)))),
                SizedBox(width: 12, height: 12, child: CustomPaint(painter: _GripPaint())),
                const SizedBox(width: 4),
                _Chev(open),
                const SizedBox(width: 4),
                Expanded(
                  child: Column(mainAxisAlignment: MainAxisAlignment.center, crossAxisAlignment: CrossAxisAlignment.start, children: [
                    Text(name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(on ? N.g95 : N.g63).copyWith(decoration: on ? null : TextDecoration.lineThrough)),
                    if (status != null) ...[const SizedBox(height: 4), statusColor == Role.error ? Row(mainAxisSize: MainAxisSize.min, children: [const ErrMark(size: 10, gap: 4), Text(status, style: T.label(statusColor))]) : Text(status, style: T.label(statusColor))],
                  ]),
                ),
                PillSwitch(on: on),
                const SizedBox(width: 8),
              ]),
            ),
          ),
        ),
      ),
    );

WidgetbookUseCase _sheetStack() => _parts('Parts @200% Card header', 'Inspector I8 Stack: card header in every state, effect head, words', [
      _Grp('Card header', [
        _Cell('rest, folded', _cardHead('Gaussian Blur'), w: 282, h: 40),
        _Cell('open', _cardHead('Gaussian Blur', open: true), w: 282, h: 40),
        _Cell('selected: gutter tick g95', _cardHead('Gaussian Blur', sel: true), w: 282, h: 40),
        _Cell('bypassed: struck + word', _cardHead('Gaussian Blur', on: false, status: 'Bypassed'), w: 282, h: 40),
        _Cell('failed: danger tick + word', _cardHead('Glow', failed: true, status: 'Failed', statusColor: Role.error), w: 282, h: 40),
        _Cell('muted by solo', _cardHead('Fractal Noise', muted: true, status: 'Muted by solo'), w: 282, h: 40),
        _Cell('long name, ellipsis', _cardHead('Transform Anchor Point Offset (Scaled by Layer Distance)'), w: 282, h: 40),
        _Cell('JP', _cardHead('ガウスブラー'), w: 282, h: 40),
        _Cell('dock 224', _cardHead('Transform Anchor Point Offset'), w: 224, h: 40),
      ], note: 'Hover and focus of the chevron and name, the grip (grab cursor) and the switch are the live panel\'s: they cannot be pinned here. Not designed: disabled (a bypassed card is still editable).'),
      _Grp('Effect head (inside the Inspector)', [
        _Cell('on', _FxHead('Glow', '11 properties', true, (_) {}), w: 282, h: 40),
        _Cell('off', _FxHead('Glow', '11 properties', false, (_) {}), w: 282, h: 40),
        _Cell('long, ellipsis', _FxHead('Transform Anchor Point Offset (Scaled by Layer Distance)', '11 properties', true, (_) {}), w: 282, h: 40),
        _Cell('JP, long, ellipsis', _FxHead('タービュレントディスプレイス (長い名前のテスト)', '6 properties', true, (_) {}), w: 282, h: 40),
      ], note: 'Not designed: hover, pressed, focus on the head itself (only its switch acts).'),
      _Grp('Word buttons (Solo, Swap, Retry)', _wordCells().take(7).toList()),
    ]);

WidgetbookUseCase _sheetMixed() => _parts('Parts @200% Mixed cells', 'Inspector I12 Mixed: number cell states, mixed word, locked word', [
      _Grp('Number cell', _numCells()),
      _Grp('Label and control rows', [
        _Cell('rest', _VRow('Position X', _Num(value: 960, unit: 'px', onDelta: (_) {})), w: 282),
        _Cell('mixed', _VRow('Position X', _Num(value: 0, mixed: true, onDelta: (_) {})), w: 282),
        _Cell('mixed + grey tick', _VRow('Position X', _Num(value: 0, mixed: true, onDelta: (_) {}), tick: N.g44), w: 282),
        _Cell('locked layer', _VRow('Position X', _Num(value: 960, word: 'Locked', enabled: false, onDelta: (_) {})), w: 282),
        _Cell('long JP label', _VRow('変形アンカーポイントのオフセット', _Num(value: 0, mixed: true, onDelta: (_) {})), w: 282, h: 44),
        _Cell('dock 200 stacked', _VRow('Position X', _Num(value: 0, mixed: true, onDelta: (_) {})), w: 200, h: 52),
      ], note: 'A row is not a control: hover, pressed and focus belong to its cell.'),
      _Grp('Status', [
        _hint('range under Mixed', '120 to 1,840'),
        _hint('error (danger)', 'Could not write 2 layers.', color: Role.error),
      ]),
    ]);

WidgetbookUseCase _sheetStagger() => _parts('Parts @200% Step and basis', 'Inspector I12 Stagger, Falloff, Grab: number, word choice, status', [
      _Grp('Step number cell', _numCells().take(6).toList()),
      _Grp('Order and shape (the one segmented bar of the Inspector)', [
        _Cell('Stage selected', _pad(Segmented(items: const ['Stage', 'Timeline', 'Picked'], index: 0, expand: true)), w: 280, h: 36),
        _Cell('Timeline selected', _pad(Segmented(items: const ['Stage', 'Timeline', 'Picked'], index: 1, expand: true)), w: 280, h: 36),
        _Cell('Picked selected', _pad(Segmented(items: const ['Stage', 'Timeline', 'Picked'], index: 2, expand: true)), w: 280, h: 36),
        _Cell('Falloff shape, Linear', _pad(Segmented(items: const ['Linear', 'Ease', 'Step'], index: 0, expand: true)), w: 280, h: 36),
        _Cell('Grab property, long names', _pad(Segmented(items: const ['Position X', 'Position Y', 'Scale'], index: 2, expand: true)), w: 280, h: 36),
      ], note: 'Selected = g20 pill + accent underline; hover, pressed and focus are the shared Segmented\'s (Controls sheet). Not designed: disabled.'),
      _Grp('Grab row, layer rows', [
        _row('all layers: row stands for 6', 'fn.contrast'),
        _row('mixed after grab', 'fn.contrast', set: {'fn.contrast': 140}),
        _Cell('locked layer skipped', _VRow('title_card', _Num(value: 100, word: 'Locked', enabled: false, onDelta: (_) {})), w: 282),
      ]),
      _Grp('Status', [
        _hint('rest', 'title_card  f14'),
        _hint('off the Stage', '3 off Stage', color: N.g91),
      ]),
    ]);

WidgetbookUseCase _sheetControls() => _parts('Parts @200% Number choice colour', 'Inspector X controls: number grammar, choice, colour, row', [
      _Grp('Number cell', _numCells()),
      _Grp('Choice cell', _pickCells(), note: 'Left-aligned word and chevron on purpose: it opens a menu. Number cells are right-aligned.'),
      _Grp('Colour cell', _colorCells()),
      _Grp('Property row', _rowCells().take(7).toList()),
    ]);

WidgetbookUseCase _sheetStates() => _parts('Parts @200% State block', 'Inspector X states: dead-end block, heading, switch, hint', [
      _Grp('State block', [
        _Cell('neutral', const _StateBlock('Nothing is selected', ['Select the top layer', 'Open Browser']), w: 282, h: 76),
        _Cell('error (danger tick) + one reason', const _StateBlock('Glow could not run', ['Retry', 'Open source', 'Bypass'], why: 'Shader error, line 14', bad: true), w: 282, h: 92),
        _Cell('long JP', const _StateBlock('タービュレントディスプレイスを実行できません', ['再試行', 'ソースを開く'], why: 'シェーダーのエラー (14 行目)', bad: true), w: 282, h: 104),
        _Cell('dock 200', const _StateBlock('city_pass.mov is missing', ['Relink', 'Replace with solid'], bad: true), w: 200, h: 88),
      ], note: 'A block is a title and the words that act (and at most one short reason), no sentences; it is not a control: no hover or pressed (its words have their own, see Words). Not designed: disabled.'),
      _Grp('Heading', _sectCells().take(6).toList()),
      _Grp('Switch and word', [
        _Cell('on (switch first, word to its right)', _pad(_Tog(true, (_) {})), w: 140),
        _Cell('off', _pad(_Tog(false, (_) {})), w: 140),
        _Cell('Locked number', _VRow('Opacity', _Num(value: 100, unit: '%', enabled: false, word: 'Locked', onDelta: (_) {})), w: 282),
      ], note: 'The switch\'s hover, pressed and focus are the shared PillSwitch\'s (Controls sheet).'),
      _Grp('Hint', [
        _hint('rest', 'Saved'),
        _hint('error', 'Glow failed on 1 layer.', color: Role.error),
      ]),
    ]);
