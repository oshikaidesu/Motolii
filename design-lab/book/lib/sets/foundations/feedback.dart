import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import '../../kit.dart';
import '../../parts/controls.dart';
import '../../tokens.dart';
import 'feedback_parts.dart';

Kind _kind(BuildContext c, {Kind initial = Kind.info}) => c.knobs.object.dropdown<Kind>(label: 'Kind', options: Kind.values, initialOption: initial, labelBuilder: (k) => k.name);

// set: feedback, status & export
WidgetbookComponent feedbackSet() => WidgetbookComponent(name: 'feedback', useCases: [
      uc('Progress bar', (c) {
        final look = lookKnob(c);
        final v = c.knobs.double.slider(label: 'Value', initialValue: .42, min: 0, max: 1);
        final h = c.knobs.int.slider(label: 'Height', initialValue: 4, min: 2, max: 8).toDouble();
        final indet = c.knobs.boolean(label: 'Indeterminate shown', initialValue: true);
        return panel(
          Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            Flex(direction: Axis.horizontal, children: [Text('Rendering preview', style: T.name()), const Spacer(), Text('${(v * 100).round()}%', style: T.value())]),
            const SizedBox(height: 8),
            ProgressBar(value: v, height: h, look: look),
            if (indet) ...[
              const SizedBox(height: 20),
              Text('Analysing audio', style: T.name()),
              const SizedBox(height: 8),
              ProgressBar(height: h, look: look),
            ],
          ]),
          width: 320,
        );
      }, width: 320),
      uc('Spinner', (c) {
        final look = lookKnob(c);
        final size = c.knobs.int.slider(label: 'Size', initialValue: 20, min: 12, max: 48).toDouble();
        final ms = c.knobs.int.slider(label: 'Period ms', initialValue: 900, min: 400, max: 2000);
        final label = c.knobs.boolean(label: 'With label', initialValue: true);
        return panel(
          Flex(direction: Axis.horizontal, mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.center, children: [
            Spinner(size: size, look: look, speedMs: ms),
            if (label) ...[const SizedBox(width: 12), Text('Loading project', style: T.name(N.g76))],
          ]),
        );
      }),
      uc('Status bar', (c) {
        final look = lookKnob(c);
        return StatusBar(
          fps: c.knobs.double.slider(label: 'Render fps', initialValue: 59.4, min: 0, max: 120),
          target: c.knobs.double.slider(label: 'Target fps', initialValue: 60, min: 24, max: 120),
          memGb: c.knobs.double.slider(label: 'Memory GB', initialValue: 5.2, min: 0, max: 16),
          memMax: 16,
          gpu: c.knobs.double.slider(label: 'GPU %', initialValue: 64, min: 0, max: 100),
          selected: c.knobs.int.slider(label: 'Selected', initialValue: 3, min: 0, max: 40),
          look: look,
        );
      }, width: 640),
      uc('Toast stack', (c) {
        final look = lookKnob(c);
        final n = c.knobs.int.slider(label: 'Count', initialValue: 4, min: 1, max: 4);
        final s = c.knobs.double.slider(label: 'Cycle seconds', initialValue: 8, min: 3, max: 20);
        return ToastStack(count: n, seconds: s, look: look);
      }, width: 340, height: 400),
      uc('Inline banner', (c) {
        final look = lookKnob(c);
        return Banner2(
          key: ValueKey(c.knobs.boolean(label: 'Dismissible', initialValue: true)),
          kind: _kind(c, initial: Kind.warn),
          title: c.knobs.string(label: 'Title', initialValue: 'Missing font'),
          message: c.knobs.string(label: 'Message', initialValue: 'Inter Tight is not installed; text uses a fallback.'),
          action: c.knobs.string(label: 'Action', initialValue: 'Replace'),
          dismissible: c.knobs.boolean(label: 'Dismissible', initialValue: true),
          look: look,
        );
      }, width: 560),
      uc('Empty state', (c) {
        final look = lookKnob(c);
        return panel(
          EmptyState(
            title: c.knobs.string(label: 'Title', initialValue: 'No layers yet'),
            body: c.knobs.string(label: 'Body', initialValue: 'Drop a clip, image or sound here, or start with an empty shape layer.'),
            action: c.knobs.string(label: 'Action', initialValue: 'Add layer'),
            illustration: c.knobs.boolean(label: 'Illustration', initialValue: true),
            look: look,
          ),
          width: 360,
          padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 40),
        );
      }, width: 360),
      uc('Skeleton loader', (c) {
        final look = lookKnob(c);
        return panel(
          Skeleton(rows: c.knobs.int.slider(label: 'Rows', initialValue: 4, min: 1, max: 8), animate: c.knobs.boolean(label: 'Animate', initialValue: true), look: look),
          width: 320,
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
        );
      }, width: 320),
      uc('Tooltip with shortcut', (c) {
        final look = lookKnob(c);
        final keys = c.knobs.string(label: 'Keys (space separated)', initialValue: 'Cmd K').split(' ').where((s) => s.isNotEmpty).toList();
        final detail = c.knobs.string(label: 'Detail', initialValue: 'Open the command palette');
        return Column(mainAxisSize: MainAxisSize.min, children: [
          const Tool(size: 28, child: SizedBox(width: 10, height: 10, child: DecoratedBox(decoration: BoxDecoration(color: N.g76, shape: BoxShape.circle)))),
          const SizedBox(height: 6),
          TipBubble(label: c.knobs.string(label: 'Label', initialValue: 'Command palette'), keys: keys, detail: detail, look: look),
        ]);
      }),
      uc('Keyboard shortcut chip', (c) {
        final look = lookKnob(c);
        final pressed = c.knobs.boolean(label: 'Pressed', initialValue: false);
        final custom = c.knobs.string(label: 'Custom (space separated)', initialValue: 'Shift Cmd P').split(' ').where((s) => s.isNotEmpty).toList();
        Widget row(String name, List<String> keys) => Padding(
              padding: const EdgeInsets.symmetric(vertical: 6),
              child: Flex(direction: Axis.horizontal, children: [Expanded(child: Text(name, style: T.name(N.g76))), Combo(keys, pressed: pressed, look: look)]),
            );
        return panel(
          Column(mainAxisSize: MainAxisSize.min, children: [
            row('Play / pause', const ['Space']),
            row('Split layer', const ['Cmd', 'K']),
            row('Zoom to fit', const ['Shift', 'Z']),
            row('Cancel', const ['Esc']),
            row('Custom', custom),
          ]),
          width: 280,
        );
      }, width: 280),
      uc('Badge and change dot', (c) {
        final count = c.knobs.int.slider(label: 'Count', initialValue: 7, min: 0, max: 150);
        final max = c.knobs.int.slider(label: 'Overflow at', initialValue: 99, min: 9, max: 99);
        final change = c.knobs.object.dropdown<Change>(label: 'Row change', options: Change.values, initialOption: Change.modified, labelBuilder: (k) => k.name);
        final size = c.knobs.int.slider(label: 'Dot size', initialValue: 6, min: 4, max: 10).toDouble();
        Widget prop(String name, String val, Change ch) => SizedBox(
              height: 24,
              child: Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.center, children: [
                SizedBox(width: 14, child: ChangeDot(ch, size: size)),
                Expanded(child: Text(name, style: T.name(N.g76))),
                Text(val, style: T.value(ch == Change.none ? N.g91 : N.g95)),
              ]),
            );
        return panel(
          Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.center, children: [
              Text('Changes', style: T.name(N.g95)),
              const SizedBox(width: 8),
              CountPill(count, max: max),
              const Spacer(),
              Text('Errors', style: T.label(N.g56)),
              const SizedBox(width: 6),
              CountPill(count ~/ 3, kind: Kind.error, max: max),
            ]),
            const SizedBox(height: 12),
            prop('Position', '960.0, 540.0', change),
            prop('Opacity', '100%', Change.none),
            prop('Scale', '120%', Change.added),
            prop('Rotation', '0.0', Change.removed),
          ]),
          width: 260,
        );
      }, width: 260),
      uc('Error row with Why?', (c) {
        final look = lookKnob(c);
        final open = c.knobs.boolean(label: 'Expanded', initialValue: false);
        return ErrorRow(
          key: ValueKey(open),
          message: c.knobs.string(label: 'Message', initialValue: 'Could not decode Hero_BG.mov'),
          detail: c.knobs.string(label: 'Detail', initialValue: 'codec: prores_ks\nreason: profile 4444 XQ is not supported by the decoder.\nTry: re-encode as ProRes 422 or H.264.'),
          expanded: open,
          look: look,
        );
      }, width: 480),
      uc('Render queue row', (c) {
        final look = lookKnob(c);
        final st = c.knobs.object.dropdown<QState>(label: 'State', options: QState.values, initialOption: QState.rendering, labelBuilder: (k) => k.name);
        final p = c.knobs.double.slider(label: 'Progress', initialValue: .64, min: 0, max: 1);
        final eta = c.knobs.string(label: 'ETA', initialValue: '00:42');
        final name = c.knobs.string(label: 'Name', initialValue: 'Title_v3.mp4');
        return Column(mainAxisSize: MainAxisSize.min, children: [
          QueueRow(name: name, sub: '1920 x 1080  H.264', state: st, progress: p, eta: eta, look: look),
          const SizedBox(height: 2),
          QueueRow(name: 'Teaser_4k.mov', sub: '3840 x 2160  ProRes', state: QState.queued, progress: 0, eta: '--', seed: 2, look: look),
          const SizedBox(height: 2),
          QueueRow(name: 'Loop.gif', sub: '800 x 450  GIF', state: QState.done, progress: 1, eta: '', seed: 4, look: look),
        ]);
      }, width: 560),
      uc('Export sheet', (c) {
        final look = lookKnob(c);
        final st = c.knobs.object.dropdown<ExportState>(label: 'State', options: ExportState.values, initialOption: ExportState.idle, labelBuilder: (k) => k.name);
        final p = c.knobs.double.slider(label: 'Progress', initialValue: .42, min: 0, max: 1);
        return ExportSheet(state: st, progress: p, fileName: c.knobs.string(label: 'File name', initialValue: 'Title_v3'), look: look);
      }, width: 420),
      uc('Undo history', (c) {
        final look = lookKnob(c);
        final n = c.knobs.int.slider(label: 'Items', initialValue: 9, min: 2, max: 10);
        final cur = c.knobs.int.slider(label: 'Current', initialValue: 6, min: 0, max: 9);
        return panel(HistoryList(key: ValueKey('$n-$cur'), count: n, current: cur, look: look), width: 280, padding: const EdgeInsets.all(8));
      }, width: 280),
    ]);
