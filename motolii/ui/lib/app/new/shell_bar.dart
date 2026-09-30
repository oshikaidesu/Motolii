import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';

import '../../theme/material_icons.dart';
import '../../theme/editor_metrics.dart';
import '../../controls/panel.dart';
import '../../foundation/shell_tokens.dart';
import '../../theme/editor_theme.dart';
import '../../session/editor_session.dart';
import '../../panels/browser.dart' show BrowserSize;
import '../editor_actions.dart';
import '../../session/status_notice.dart' show freezeNotice;

/// The strip across the top, read left to right as an instrument: the name,
/// the document menus, the transport and its readouts, the three sheets,
/// and the document's own name.
class NewTopBar extends StatelessWidget {
  const NewTopBar({
    super.key,
    required this.c,
    required this.viewItems,
    required this.sheet,
    required this.onMenu,
    required this.onSheet,
  });
  final EditorSession c;

  /// What the View menu lists: every panel by name, and the last entry that puts the workspace back.
  final List<String> viewItems;
  final String? sheet;
  final ValueChanged<String> onMenu;
  final ValueChanged<String> onSheet;

  Widget _menu(BuildContext context, String label, List<String> items) =>
      Builder(
        builder: (context) => _Label(
          label,
          onTap: () async {
            final box = context.findRenderObject() as RenderBox;
            final chosen = await showEditorMenu<String>(
              context,
              box.localToGlobal(box.size.bottomLeft(Offset.zero)),
              [
                for (final item in items)
                  EditorMenuItem(value: item, child: Text(item)),
              ],
            );
            if (chosen != null) onMenu(chosen);
          },
        ),
      );

  @override
  Widget build(BuildContext context) => Container(
    height: ShellTokens.topBar,
    color: ShellTokens.ground,
    padding: const EdgeInsets.symmetric(horizontal: ShellTokens.gutter),
    child: Row(
      children: [
        const Text(
          'Motolii',
          style: TextStyle(
            fontSize: ShellTokens.wordmark,
            fontWeight: FontWeight.w700,
            letterSpacing: ShellTokens.wordmarkTracking,
            color: ShellTokens.ink,
          ),
        ),
        const SizedBox(width: ShellTokens.tabGap),
        _menu(context, 'File', fileActions),
        _menu(context, 'Edit', editActions.keys.toList()),
        _menu(context, 'View', viewItems),
        const SizedBox(width: ShellTokens.tabGap),
        _Transport(c: c),
        const Spacer(),
        _Segment(
          names: const ['Composition', 'Export', 'Settings'],
          active: sheet,
          onPick: onSheet,
        ),
        const SizedBox(width: ShellTokens.tabGap),
        ValueListenableBuilder<Map<String, dynamic>>(
          valueListenable: c.slice('title', const ['dirty', 'path']),
          builder: (_, state, __) => Text(
            '${state['dirty'] == true ? '● ' : ''}'
            '${(state['path'] as String? ?? 'Untitled').split('/').last}',
            style: ShellTokens.kickerStyle(ShellTokens.inkMuted),
          ),
        ),
      ],
    ),
  );
}

/// Play, back to the start, and Animate, then where the playhead is. Only
/// the frame readout follows playback, and it is one line of text.
class _Transport extends StatelessWidget {
  const _Transport({required this.c});
  final EditorSession c;

  static String _clock(int frame, num fps) {
    final rate = fps <= 0 ? 1 : fps;
    final seconds = frame / rate;
    final m = seconds ~/ 60;
    final sec = (seconds % 60).floor();
    final ff = (frame % rate.round().clamp(1, 1000)).toString().padLeft(2, '0');
    return '${m.toString().padLeft(2, '0')}:${sec.toString().padLeft(2, '0')}:$ff';
  }

  @override
  Widget build(BuildContext context) {
    final timing = c.slice('transport', const ['fps', 'durationFrames']);
    final animate = c.slice('transportAnimate', const ['animate']);
    return Row(
      children: [
        ValueListenableBuilder<bool>(
          valueListenable: c.playing,
          builder: (_, playing, __) => _Key(
            tip: playing ? 'Pause (Space)' : 'Play (Space)',
            lit: playing ? ShellTokens.mint : null,
            onTap: c.togglePlayback,
            child: playing
                ? const _Bars()
                : const Icon(
                    Glyph.play_arrow_outlined,
                    size: ShellTokens.readout + EditorMetrics.s4,
                    color: ShellTokens.ink,
                  ),
          ),
        ),
        _Key(
          tip: 'Stop and go to the start (Home)',
          onTap: () {
            c.stopPlayback();
            c.seek(0);
          },
          child: Container(
            width: ShellTokens.keyGlyph - EditorMetrics.s2,
            height: ShellTokens.keyGlyph - EditorMetrics.s2,
            color: ShellTokens.ink,
          ),
        ),
        ValueListenableBuilder(
          valueListenable: animate,
          builder: (_, __, ___) => _Key(
            tip: 'Animate (A): values you touch become keys',
            lit: c.animating ? ShellTokens.pink : null,
            onTap: c.supports('animate')
                ? () => c.toggleAnimate()
                : null,
            child: Container(
              width: ShellTokens.keyGlyph,
              height: ShellTokens.keyGlyph,
              decoration: BoxDecoration(
                color: c.animating ? ShellTokens.inkOnAccent : ShellTokens.pink,
                shape: BoxShape.circle,
              ),
            ),
          ),
        ),
        const SizedBox(width: ShellTokens.readoutGap),
        ValueListenableBuilder(
          valueListenable: timing,
          builder: (_, __, ___) {
            final fps = (c.state['fps'] as num?) ?? 30;
            final duration = (c.state['durationFrames'] as num?)?.toInt() ?? 0;
            return Row(
              children: [
                Text(
                  fps.toStringAsFixed(2),
                  style: ShellTokens.readoutStyle(ShellTokens.ink),
                ),
                Text(
                  ' FPS',
                  style: ShellTokens.kickerStyle(ShellTokens.inkFaint),
                ),
                const _Tick(),
                ValueListenableBuilder<int>(
                  valueListenable: c.frame,
                  builder: (_, frame, __) => Text(
                    '${_clock(frame, fps)}  '
                    '${frame.toString().padLeft(4, '0')}'
                    '/${duration.toString().padLeft(4, '0')}',
                    style: ShellTokens.readoutStyle(ShellTokens.ink),
                  ),
                ),
              ],
            );
          },
        ),
      ],
    );
  }
}

class _Tick extends StatelessWidget {
  const _Tick();
  @override
  Widget build(BuildContext context) => Container(
    width: ShellTokens.ruleWidth,
    height: ShellTokens.readout,
    margin: const EdgeInsets.symmetric(horizontal: ShellTokens.gutter),
    color: ShellTokens.ruleStrong,
  );
}

class _Bars extends StatelessWidget {
  const _Bars();
  @override
  Widget build(BuildContext context) => Row(
    mainAxisSize: MainAxisSize.min,
    children: [
      for (var i = 0; i < 2; i++) ...[
        if (i == 1) const SizedBox(width: EditorMetrics.s3),
        Container(
          width: EditorMetrics.s3,
          height: ShellTokens.keyGlyph,
          color: ShellTokens.inkOnAccent,
        ),
      ],
    ],
  );
}

/// A square key of the transport: flat, ruled, lit in its colour when on.
class _Key extends StatelessWidget {
  const _Key({required this.tip, required this.child, this.onTap, this.lit});
  final String tip;
  final Widget child;
  final VoidCallback? onTap;
  final Color? lit;
  @override
  Widget build(BuildContext context) => EditorTooltip(
    message: tip,
    child: Semantics(
      button: true,
      label: tip,
      child: MouseRegion(
        cursor: onTap == null
            ? SystemMouseCursors.basic
            : SystemMouseCursors.click,
        child: GestureDetector(
          behavior: HitTestBehavior.opaque,
          onTap: onTap,
          child: Container(
            width: ShellTokens.key,
            height: ShellTokens.key,
            margin: const EdgeInsets.only(right: EditorMetrics.s2),
            alignment: Alignment.center,
            decoration: BoxDecoration(
              color: lit ?? ShellTokens.raised,
              border: Border.all(color: lit ?? ShellTokens.ruleStrong),
            ),
            child: child,
          ),
        ),
      ),
    ),
  );
}

/// Neighbouring choices in one ruled box, the chosen one lit.
class _Segment extends StatelessWidget {
  const _Segment({required this.names, required this.onPick, this.active});
  final List<String> names;
  final String? active;
  final ValueChanged<String> onPick;
  @override
  Widget build(BuildContext context) => Container(
    height: ShellTokens.key,
    decoration: BoxDecoration(
      border: Border.all(color: ShellTokens.ruleStrong),
    ),
    child: Row(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        for (final name in names)
          Semantics(
            button: true,
            selected: name == active,
            label: name,
            child: MouseRegion(
              cursor: SystemMouseCursors.click,
              child: GestureDetector(
                behavior: HitTestBehavior.opaque,
                onTap: () => onPick(name),
                child: Container(
                  padding: const EdgeInsets.symmetric(
                    horizontal: ShellTokens.gutter + EditorMetrics.s4,
                  ),
                  alignment: Alignment.center,
                  color: name == active ? ShellTokens.sky : EditorTheme.clear,
                  child: Text(
                    name.toUpperCase(),
                    style: ShellTokens.kickerStyle(
                      name == active
                          ? ShellTokens.inkOnAccent
                          : ShellTokens.ink,
                    ),
                  ),
                ),
              ),
            ),
          ),
      ],
    ),
  );
}

/// An uppercase menu title that is pressed.
class _Label extends StatelessWidget {
  const _Label(this.text, {this.onTap});
  final String text;
  final VoidCallback? onTap;
  @override
  Widget build(BuildContext context) => Semantics(
    button: true,
    label: text,
    child: MouseRegion(
      cursor: SystemMouseCursors.click,
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: onTap,
        child: Container(
          padding: const EdgeInsets.symmetric(horizontal: ShellTokens.gutter),
          alignment: Alignment.center,
          child: Text(
            text.toUpperCase(),
            style: ShellTokens.kickerStyle(ShellTokens.ink),
          ),
        ),
      ),
    ),
  );
}

class NewStatusBar extends StatelessWidget {
  const NewStatusBar({required this.c, required this.notice});
  final EditorSession c;
  final ValueListenable<Map<String, dynamic>> notice;
  @override
  Widget build(BuildContext context) => Container(
    height: ShellTokens.statusBar,
    color: ShellTokens.ground,
    padding: const EdgeInsets.symmetric(horizontal: ShellTokens.gutter),
    alignment: Alignment.centerLeft,
    child: ValueListenableBuilder<String?>(
      valueListenable: c.error,
      builder: (_, message, __) => ValueListenableBuilder(
        valueListenable: notice,
        builder: (_, doc, __) => Text(
          message ?? freezeNotice(doc) ?? effectsNotice(doc),
          maxLines: 1,
          overflow: TextOverflow.ellipsis,
          style: ShellTokens.kickerStyle(ShellTokens.inkMuted),
        ),
      ),
    ),
  );
}

/// The preferences the New face can already reach; the panel placement table
/// and the Classic theme file belong to Classic's dock and are not routed.
class NewSettings extends StatelessWidget {
  const NewSettings({required this.c, required this.scale});
  final EditorSession c;

  /// The editor's one scale (50 to 200 percent), kept in the saved settings under `scale`, as Classic keeps it.
  final ValueNotifier<double> scale;
  Widget _row(String label, Widget control) => SizedBox(
    height: EditorMetrics.control,
    child: Row(
      children: [
        Expanded(child: Text(label)),
        control,
      ],
    ),
  );
  void _scale(double next) {
    scale.value = next.clamp(.5, 2.0).toDouble();
    c.storeSetting('scale', scale.value);
  }

  @override
  Widget build(BuildContext context) => ValueListenableBuilder(
    valueListenable: c.deskWork,
    builder: (context, _, __) => Padding(
      padding: const EdgeInsets.all(ShellTokens.gutter),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          _row(
            'Browser tile size',
            SizedBox(
              width: EditorMetrics.s200,
              child: EditorZoomBar(
                keyPrefix: 'settings:browserTile',
                base: BrowserSize.base,
                min: BrowserSize.min,
                max: BrowserSize.max,
                value: BrowserSize.tile(c),
                onChanged: (v) => c.storeDesk('browserTile', v),
              ),
            ),
          ),
          _row(
            'Animate: key the start too',
            EditorSwitch(
              key: const ValueKey('settings:animateFrom'),
              on: c.animateFrom,
              glyph: Glyph.diamond_outlined,
              label:
                  'When Animate is turned on, remember the frame; the first '
                  'touch at another frame keys both that frame and this one',
              onChanged: (on) => c.storeDesk('animateFrom', on),
            ),
          ),
          _row(
            'Scale',
            ValueListenableBuilder<double>(
              valueListenable: scale,
              builder: (context, value, _) => Row(
                children: [
                  EditorButton('−', () => _scale(((value * 100).round() - 1) / 100), tooltip: 'Smaller'),
                  EditorPercentField(value: value * 100, min: 50, max: 200, onChanged: (v) => _scale(v / 100)),
                  EditorButton('+', () => _scale(((value * 100).round() + 1) / 100), tooltip: 'Larger'),
                ],
              ),
            ),
          ),
          _row(
            'New layers',
            SizedBox(
              width: EditorMetrics.s76,
              child: EditorChoice<String>(
                key: const ValueKey('settings:flatProjection'),
                value: c.flatProjection,
                choices: const [MapEntry('2.5D', '2.5D'), MapEntry('3D', '3D')],
                onChanged: (v) => c.storeDesk('flatProjection', v),
              ),
            ),
          ),
        ],
      ),
    ),
  );
}
