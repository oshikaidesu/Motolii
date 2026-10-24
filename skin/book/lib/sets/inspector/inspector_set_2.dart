// Part of inspector_set.dart (cut there by size, as it was).
part of 'inspector_set.dart';

WidgetbookComponent inspectorSetSet() => WidgetbookComponent(
  name: 'inspector',
  useCases: [
    uc('Header — kind, name, Animate all', (c) {
      final k = _kindKnob(c);
      return _host(
        c,
        k,
        Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            if (k == Kind.none) const EmptyState() else const InspHeader(),
            const Cap(
              'One subject at a time, named in the panel (A1). Animate all: while on, an edit writes a key at the playhead (D2). The state is a word, not colour alone.',
            ),
          ],
        ),
        count: 3,
        lockedKnob: true,
      );
    }, width: 372),
    uc(
      'Number field — the core control',
      (c) => _host(
        c,
        Kind.shape,
        Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            const Sect(
              first: true,
              title: 'Try every gesture',
              children: [
                PropRow(
                  id: 'v_opacity',
                  label: 'Opacity',
                  cells: [
                    NumField(id: 'v_opacity', unit: '%', min: 0, max: 100, perPx: .5),
                  ],
                ),
                PropRow(
                  id: 'v_position',
                  label: 'Position',
                  ids: ['v_x', 'v_y'],
                  cells: [
                    NumField(
                      id: 'v_x',
                      label: 'X',
                      unit: 'px',
                      decimals: 1,
                      axis: Color(0xFFE974AB),
                    ),
                    NumField(
                      id: 'v_y',
                      label: 'Y',
                      unit: 'px',
                      decimals: 1,
                      axis: Color(0xFF7DD5B1),
                    ),
                  ],
                ),
                PropRow(
                  id: 'v_rotate',
                  label: 'Rotation',
                  cells: [
                    NumField(
                      id: 'v_rotate',
                      unit: '°',
                      decimals: 1,
                      perPx: .5,
                      turns: true,
                    ),
                  ],
                ),
                PropRow(
                  id: 'depth',
                  label: 'Depth',
                  cells: [NumField(id: 'depth', unit: 'px', zeroWord: 'Flat')],
                ),
              ],
            ),
            const _Rules(),
            const Cap(
              'Drag the field. Double-click types (with reset = double-click, a single click types after the double-click window, so no editor flashes). Enter types. Release = one undo step. Rotation 450 stays 450. A bounded value shows a fill; an open one does not. Hover and slide sideways: right = more, left = less, at the drag rate; an up/down slide scrolls the panel. Tab goes to the next field. Maybe: drag up/down changes precision.',
            ),
          ],
        ),
        lockedKnob: true,
      ),
      width: 372,
    ),
    uc('Number field — states and narrowing', (c) {
      final w = c.knobs.double.slider(
        label: 'Cell width',
        initialValue: 140,
        min: 48,
        max: 240,
      );
      return _host(
        c,
        Kind.several,
        Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            SizedBox(
              width: w,
              child: const Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  NumField(
                    id: 'v_opacity',
                    label: 'Opacity',
                    unit: '%',
                    min: 0,
                    max: 100,
                  ),
                  SizedBox(height: 6),
                  NumField(
                    id: 'v_x',
                    label: 'Mixed',
                    unit: 'px',
                    decimals: 1,
                  ),
                  SizedBox(height: 6),
                  NumField(
                    id: 'depth',
                    label: 'Depth',
                    unit: 'px',
                    zeroWord: 'Flat',
                  ),
                  SizedBox(height: 6),
                  NumField(
                    id: 'v_rotate',
                    label: 'Rot',
                    unit: '°',
                    decimals: 1,
                    turns: true,
                  ),
                  SizedBox(height: 6),
                  NumField(
                    id: 'v_scale_y',
                    label: 'Scale',
                    unit: '%',
                    decimals: 2,
                    enabled: false,
                  ),
                ],
              ),
            ),
            const Cap(
              'Narrow rule (one for the whole panel): below ~92 px the unit goes, below ~60 px the decimals go; a value that is not zero never reads 0 and no digit is cut (C9). Mixed shows —; a scrub on it moves every layer by the same amount, a typed number sets all (A7). The last field is disabled.',
            ),
            const _Rules(),
          ],
        ),
        count: 3,
      );
    }, width: 372),
    uc(
      'Key mark and reset mark',
      (c) => _host(
        c,
        Kind.shape,
        Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            const Row(
              children: [
                Expanded(child: Cap('Not animated')),
                Expanded(child: Cap('Animated, no key here')),
                Expanded(child: Cap('Keyed at the playhead')),
              ],
            ),
            const Row(
              children: [
                Expanded(
                  child: Align(
                    alignment: Alignment.centerLeft,
                    child: KeyMark(state: KeyS.off),
                  ),
                ),
                Expanded(
                  child: Align(
                    alignment: Alignment.centerLeft,
                    child: KeyMark(state: KeyS.anim),
                  ),
                ),
                Expanded(
                  child: Align(
                    alignment: Alignment.centerLeft,
                    child: KeyMark(state: KeyS.at),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 8),
            const Sect(
              first: true,
              title: 'Live rows',
              children: [
                PropRow(
                  id: 'v_position',
                  label: 'Position',
                  ids: ['v_x', 'v_y'],
                  cells: [
                    NumField(id: 'v_x', label: 'X', decimals: 1),
                    NumField(id: 'v_y', label: 'Y', decimals: 1),
                  ],
                ),
                PropRow(
                  id: 'v_rotate',
                  label: 'Rotation',
                  cells: [NumField(id: 'v_rotate', unit: '°', decimals: 1)],
                ),
                PropRow(
                  id: 'v_opacity',
                  label: 'Opacity',
                  cells: [NumField(id: 'v_opacity', unit: '%', min: 0, max: 100)],
                ),
              ],
            ),
            const Cap(
              'The three states differ by shape (hollow diamond / diamond with its link line / filled playhead diamond), not by tone alone (D1). Click keys at the playhead; click on a key removes it. Right-click a row for Key this frame / Remove key / Remove animation, with the reason when one is unavailable (D3). Changed values turn their label brighter; the reset arrow (knob) appears only on those (E4).',
            ),
          ],
        ),
        lockedKnob: true,
      ),
      width: 372,
    ),
    uc(
      'Effect — parameters, enable, Advanced',
      (c) => _host(
        c,
        Kind.shape,
        Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            EffectCard(kEffects.first, first: true),
            const Cap(
              'Heroes first, the rest behind Advanced with a count (F2). Two small controls pair up when there is room (F4). The switch bypasses the effect and dims its body; the name folds it; More: move, reset, randomise, remove (F1, F5).',
            ),
          ],
        ),
        lockedKnob: true,
      ),
      width: 372,
    ),
    uc(
      'Effects stack',
      (c) => _host(
        c,
        Kind.shape,
        Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            const EffectsStack(),
            const Cap(
              'Order matters and is visible; More moves an effect earlier or later (drag-to-reorder is not built in this lab). Maybe: search flattens the groups.',
            ),
          ],
        ),
        effects: 4,
        lockedKnob: true,
      ),
      width: 372,
      height: 720,
    ),
    uc(
      'Relations section',
      (c) => _host(
        c,
        Kind.group,
        Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            const RelationsBlock(first: true),
            const Cap(
              'One relation open as its card (the lab\'s RelationCard), the others one row each; clicking a row opens it. Hue only on the relation mark. Maybe: a driven row says who drives it (I4), see the next case.',
            ),
          ],
        ),
        lockedKnob: true,
      ),
      width: 372,
    ),
    uc(
      'Empty state — nothing selected',
      (c) => _host(
        c,
        Kind.none,
        const Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            EmptyState(),
            Cap(
              'One plain muted line, no controls, no stale values (A6). Maybe: the composition settings take its place.',
            ),
          ],
        ),
      ),
      width: 372,
    ),
    uc(
      'Inspector — direct manipulation prototype',
      (c) {
        final kind = _kindKnob(c);
        return _host(
          c,
          kind,
          const InspectorPanel(),
          count: 3,
          lockedKnob: true,
        );
      },
      width: 372,
      height: 720,
    ),
    uc(
      'Inspector — all in one',
      (c) => _host(
        c,
        _kindKnob(c),
        const _BothWays(
          Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [InspectorPanel(height: 1400), SizedBox(width: 16), HostGallery()],
          ),
        ),
        count: 3,
        lockedKnob: true,
      ),
    ),
    uc('Inspector — host ideas', (c) => _host(c, Kind.text, const _BothWays(IdeaGallery()), lockedKnob: true)),
    uc('Inspector — host features', (c) => _host(c, Kind.shape, const _BothWays(HostGallery()), lockedKnob: true)),
    uc(
      'Inspector — shape layer',
      (c) => _panel(c, Kind.shape),
      width: 372,
      height: 720,
    ),
    uc(
      'Inspector — text layer',
      (c) => _panel(c, Kind.text),
      width: 372,
      height: 720,
    ),
    uc(
      'Inspector — effects, long',
      (c) => _panel(c, Kind.shape, effects: 4, h: 760),
      width: 372,
      height: 760,
    ),
    uc(
      'Inspector — camera',
      (c) => _panel(c, Kind.camera),
      width: 372,
      height: 720,
    ),
    uc(
      'Inspector — narrow (280px)',
      (c) => _panel(c, Kind.shape, w: 280),
      width: 280,
      height: 720,
    ),
    uc(
      'Inspector — nothing chosen',
      (c) => _panel(c, Kind.none, h: 360),
      width: 372,
      height: 360,
    ),
  ],
);
