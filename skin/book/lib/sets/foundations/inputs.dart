import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import '../../kit.dart';
import '../../tokens.dart';
import 'inputs_parts.dart';

// set: inputs. Every use case has a Look knob (concept / quiet / glow), a Disabled knob and the state of the control as knobs.
Widget _cap(String s, {bool wrap = false}) => Padding(padding: const EdgeInsets.only(bottom: 8), child: Text(s.toUpperCase(), maxLines: wrap ? 2 : 1, softWrap: wrap, overflow: TextOverflow.ellipsis, style: T.micro(N.g56).copyWith(letterSpacing: .6)));

WidgetbookComponent inputsSet() => WidgetbookComponent(name: 'inputs', useCases: [
      uc('Scrub number', (c) {
        final look = lookKnob(c);
        final off = c.knobs.boolean(label: 'Disabled');
        return Live<double>(
          value: c.knobs.double.slider(label: 'Opacity', initialValue: 62, min: 0, max: 100),
          builder: (_, v, set) => Live<double>(
            value: c.knobs.double.slider(label: 'Rotation', initialValue: 15, min: -180, max: 180),
            builder: (_, r, setR) => Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
              ScrubField(label: 'Opacity', unit: '%', value: v, min: 0, max: 100, look: look, enabled: !off, onChanged: set),
              const SizedBox(height: 8),
              ScrubField(label: 'Rotation', unit: '°', value: r, perPx: .5, look: look, enabled: !off, fill: false, onChanged: setR),
              const SizedBox(height: 8),
              _cap('Drag · Shift = fine'),
              if (c.knobs.boolean(label: 'Focus')) _cap('Double-click to type'),
            ]),
          ),
        );
      }, width: 240),
      uc('Vec3 row', (c) {
        final look = lookKnob(c);
        return Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          _cap('Position'),
          Vec3Row(
            x: c.knobs.double.slider(label: 'X', initialValue: 960, min: -2000, max: 2000),
            y: c.knobs.double.slider(label: 'Y', initialValue: 540, min: -2000, max: 2000),
            z: c.knobs.double.slider(label: 'Z', initialValue: 0, min: -2000, max: 2000),
            decimals: c.knobs.int.slider(label: 'Decimals', initialValue: 1, min: 0, max: 3),
            look: look,
            enabled: !c.knobs.boolean(label: 'Disabled'),
          ),
        ]);
      }, width: 320),
      uc('Link / lock ratio', (c) {
        final look = lookKnob(c);
        return Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          _cap('Size'),
          RatioPair(
            w: c.knobs.double.slider(label: 'Width', initialValue: 1920, min: 16, max: 4096),
            h: c.knobs.double.slider(label: 'Height', initialValue: 1080, min: 16, max: 4096),
            linked: c.knobs.boolean(label: 'Linked', initialValue: true),
            look: look,
            enabled: !c.knobs.boolean(label: 'Disabled'),
          ),
        ]);
      }, width: 320),
      uc('Text field', (c) {
        final look = lookKnob(c);
        return Live<String>(
          value: c.knobs.string(label: 'Text', initialValue: 'Hero title'),
          builder: (_, v, set) => Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            _cap('Name'),
            TextBox(text: v, hint: 'Layer name', look: look, ring: c.knobs.boolean(label: 'Focus ring'), enabled: !c.knobs.boolean(label: 'Disabled'), maxLength: c.knobs.int.slider(label: 'Max length', initialValue: 32, min: 4, max: 64), onChanged: set),
          ]),
        );
      }, width: 260),
      uc('Search with clear', (c) {
        final look = lookKnob(c);
        return Live<String>(
          value: c.knobs.string(label: 'Query', initialValue: 'blur'),
          builder: (_, v, set) => TextBox(
            text: v,
            hint: 'Search effects',
            look: look,
            enabled: !c.knobs.boolean(label: 'Disabled'),
            clearable: true,
            leading: const GlyphIcon(Glyph.search, size: 14, color: N.g56),
            onChanged: set,
          ),
        );
      }, width: 260),
      uc('Dropdown', (c) {
        final look = lookKnob(c);
        const items = ['Linear', 'Ease In', 'Ease Out', 'Ease In-Out', 'Hold'];
        return Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          _cap('Interpolation'),
          Live<int>(
            value: c.knobs.int.slider(label: 'Selected', initialValue: 2, min: 0, max: 4),
            builder: (_, i, set) => Dropdown(items: items, index: i, look: look, enabled: !c.knobs.boolean(label: 'Disabled'), initialOpen: c.knobs.boolean(label: 'Open'), ring: c.knobs.boolean(label: 'Focus ring'), onChanged: set),
          ),
        ]);
      }, width: 220, height: 240),
      uc('Checkbox / radio', (c) {
        final look = lookKnob(c);
        final off = c.knobs.boolean(label: 'Disabled');
        final st = c.knobs.object.dropdown<String>(label: 'Checkbox', options: const ['off', 'on', 'mixed'], initialOption: 'on', labelBuilder: (s) => s);
        return Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
          Expanded(
            child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
              _cap('Checkbox'),
              Live<String>(
                value: st,
                builder: (_, s, set) => CheckBox(label: 'Visible', value: s == 'mixed' ? null : s == 'on', look: look, enabled: !off, onChanged: (v) => set(v ? 'on' : 'off')),
              ),
              const SizedBox(height: 8),
              Live<bool>(value: true, builder: (_, v, set) => CheckBox(label: 'Motion blur', value: v, look: look, enabled: !off, onChanged: set)),
              const SizedBox(height: 8),
              Live<bool>(value: false, builder: (_, v, set) => CheckBox(label: 'Lock', value: v, look: look, enabled: !off, onChanged: set)),
            ]),
          ),
          Expanded(
            child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
              _cap('Radio'),
              Live<int>(
                value: c.knobs.int.slider(label: 'Radio', initialValue: 0, min: 0, max: 2),
                builder: (_, i, set) => Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
                  for (var k = 0; k < 3; k++) ...[
                    if (k > 0) const SizedBox(height: 8),
                    RadioDot(label: const ['Normal', 'Multiply', 'Screen'][k], on: i == k, look: look, enabled: !off, onTap: () => set(k)),
                  ],
                ]),
              ),
            ]),
          ),
        ]);
      }, width: 320),
      uc('Slider with ticks', (c) {
        final look = lookKnob(c);
        final ticks = c.knobs.int.slider(label: 'Ticks', initialValue: 10, min: 2, max: 20);
        return Live<double>(
          value: c.knobs.double.slider(label: 'Value', initialValue: 40, min: 0, max: 100),
          builder: (_, v, set) => Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            Row(children: [Text('SCALE', style: T.micro(N.g56).copyWith(letterSpacing: .6)), const Spacer(), Text('${v.toStringAsFixed(c.knobs.boolean(label: 'Snap to ticks') ? 0 : 1)} %', style: T.value(N.g95))]),
            const SizedBox(height: 6),
            TickSlider(lo: 0, hi: v, ticks: ticks, snap: c.knobs.boolean(label: 'Snap to ticks', initialValue: true), look: look, enabled: !c.knobs.boolean(label: 'Disabled'), onChanged: (_, h) => set(h)),
          ]),
        );
      }, width: 280),
      uc('Range slider', (c) {
        final look = lookKnob(c);
        return Live<(double, double)>(
          value: (c.knobs.double.slider(label: 'Low', initialValue: 20, min: 0, max: 100), c.knobs.double.slider(label: 'High', initialValue: 70, min: 0, max: 100)),
          builder: (_, r, set) => Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            Row(children: [
              Text('WORK AREA', style: T.micro(N.g56).copyWith(letterSpacing: .6)),
              const Spacer(),
              Text('${r.$1.toStringAsFixed(0)}  –  ${r.$2.toStringAsFixed(0)} f', style: T.value(N.g95)),
            ]),
            const SizedBox(height: 6),
            TickSlider(lo: r.$1, hi: r.$2, range: true, ticks: c.knobs.int.slider(label: 'Ticks', initialValue: 10, min: 2, max: 20), snap: c.knobs.boolean(label: 'Snap to ticks'), look: look, enabled: !c.knobs.boolean(label: 'Disabled'), onChanged: (l, h) => set((l, h))),
          ]),
        );
      }, width: 280),
      uc('Colour swatch + hex', (c) {
        final look = lookKnob(c);
        return Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          _cap('Fill'),
          HexColorField(hex: c.knobs.string(label: 'Hex', initialValue: 'E974AB'), look: look, enabled: !c.knobs.boolean(label: 'Disabled'), initialOpen: c.knobs.boolean(label: 'Picker open'), ring: c.knobs.boolean(label: 'Focus ring')),
        ]);
      }, width: 220, height: 280),
      uc('Tag input', (c) {
        final look = lookKnob(c);
        return Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          _cap('Tags · Enter adds'),
          TagInput(tags: c.knobs.string(label: 'Tags', initialValue: 'title, hero, loop').split(',').map((s) => s.trim()).where((s) => s.isNotEmpty).toList(), look: look, enabled: !c.knobs.boolean(label: 'Disabled')),
        ]);
      }, width: 260),
      uc('Stepper', (c) {
        final look = lookKnob(c);
        final off = c.knobs.boolean(label: 'Disabled');
        return Row(mainAxisSize: MainAxisSize.min, children: [
          Live<double>(value: c.knobs.double.slider(label: 'Value', initialValue: 4, min: 0, max: 12), builder: (_, v, set) => SizedBox(width: 104, child: StepperField(value: v, min: 0, max: 12, unit: 'px', look: look, enabled: !off, onChanged: set))),
          const SizedBox(width: 12),
          Live<double>(value: 1, builder: (_, v, set) => SizedBox(width: 104, child: StepperField(value: v, min: .5, max: 2, step: .1, unit: '×', look: look, enabled: !off, onChanged: set))),
        ]);
      }),
      uc('Toggle button group', (c) {
        final look = lookKnob(c);
        final multi = c.knobs.boolean(label: 'Multi select');
        final off = c.knobs.boolean(label: 'Disabled');
        final align = <Widget Function(Color)>[
          for (final g in [Glyph.alignL, Glyph.alignC, Glyph.alignR, Glyph.alignJ]) (k) => GlyphIcon(g, size: 14, color: k),
        ];
        final style = <Widget Function(Color)>[
          (k) => Text('B', style: T.name(k).copyWith(fontWeight: FontWeight.w800)),
          (k) => Text('I', style: T.name(k).copyWith(fontStyle: FontStyle.italic)),
          (k) => Text('U', style: T.name(k).copyWith(decoration: TextDecoration.underline, decorationColor: k)),
        ];
        return Row(mainAxisSize: MainAxisSize.min, children: [
          Live<Set<int>>(value: const {0}, builder: (_, s, set) => ToggleGroup(icons: align, selected: s, multi: multi, look: look, enabled: !off, onChanged: set)),
          const SizedBox(width: 16),
          Live<Set<int>>(value: const {0}, builder: (_, s, set) => ToggleGroup(icons: style, selected: s, multi: true, look: look, enabled: !off, onChanged: set)),
        ]);
      }),
      uc('Curve preset picker', (c) {
        final look = lookKnob(c);
        return Live<int>(
          value: c.knobs.int.slider(label: 'Selected', initialValue: 3, min: 0, max: 5),
          builder: (_, i, set) => CurvePicker(index: i, look: look, enabled: !c.knobs.boolean(label: 'Disabled'), onChanged: set),
        );
      }, width: 280),
    ]);
