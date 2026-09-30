part of '../ease_desk.dart';

/// One preset in the choices grid. Immutable inputs only; the desk hands the
/// same preset object across builds so a tile rebuilds only when its own
/// selection, hover or focus changes.
class _PresetTile extends StatelessWidget {
  const _PresetTile({
    required this.index,
    required this.preset,
    required this.width,
    required this.height,
    required this.selected,
    required this.hovered,
    required this.focused,
    required this.free,
    required this.onEnter,
    required this.onExit,
    required this.onTap,
  });
  final int index;
  final Map<String, dynamic> preset;
  final double width, height;
  final bool selected, hovered, focused, free;
  final VoidCallback onEnter, onExit, onTap;

  @override
  Widget build(BuildContext context) {
    final kind = '${preset['kind']}';
    return MouseRegion(
      onEnter: (_) => onEnter(),
      onExit: (_) => onExit(),
      child: EditorTooltip(
        key: ValueKey('ease-preset:$index'),
        message: kind,
        child: Semantics(
          label: '$kind preset',
          button: true,
          selected: selected,
          child: EditorPress(
            canRequestFocus: false,
            onTap: onTap,
            borderRadius: BorderRadius.circular(EditorMetrics.s4),
            child: Container(
              width: width,
              height: height,
              decoration: BoxDecoration(
                color: selected
                    ? EditorInk.dark.easePaper
                    : hovered
                    ? EditorTheme.of(context).hover
                    : EditorTheme.of(context).panel,
                borderRadius: BorderRadius.circular(EditorMetrics.s4),
                border: Border.all(
                  color: focused ? EditorInk.dark.easePaper : EditorTheme.clear,
                ),
              ),
              child: Padding(
                padding: const EdgeInsets.all(EditorMetrics.s4),
                child: Column(
                  children: [
                    Expanded(
                      child: Center(
                        child: SizedBox.square(
                          dimension: EditorMetrics.s44,
                          child: CustomPaint(
                            painter: EaseCurvePainter(
                              colors: EditorTheme.of(context),
                              shape: preset,
                              selected: selected,
                              free: free,
                            ),
                            child: const SizedBox.expand(),
                          ),
                        ),
                      ),
                    ),
                    Text(
                      _curveName(kind),
                      maxLines: 2,
                      textAlign: TextAlign.center,
                      overflow: TextOverflow.ellipsis,
                      style: TextStyle(
                        fontSize: EditorMetrics.font,
                        color: selected
                            ? EditorInk.dark.easeInk
                            : EditorTheme.of(context).ink,
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}

/// One numeric parameter of the shape in force.
class _EaseParamField extends StatelessWidget {
  const _EaseParamField({
    required this.kind,
    required this.width,
    required this.name,
    required this.value,
    required this.onPreview,
    required this.onCommit,
    required this.onFinish,
    required this.onCancel,
  });
  final String kind, name;
  final double width, value;
  final Future<void> Function(double) onPreview, onCommit;
  final Future<void> Function() onFinish, onCancel;

  @override
  Widget build(BuildContext context) {
    final label = name.replaceAll('_', ' ');
    return SizedBox(
      width: width,
      height: EditorMetrics.control,
      child: Row(
        children: [
          Expanded(
            child: EditorTooltip(
              message: label,
              child: Text(
                label,
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                style: TextStyle(
                  fontSize: EditorMetrics.dense,
                  color: EditorTheme.of(context).muted,
                ),
              ),
            ),
          ),
          SizedBox(
            width: EditorMetrics.s44,
            child: EditorNumericField(
              key: ValueKey('$kind:$name'),
              value: value,
              label: name,
              enabled: true,
              speed: .005,
              onPreview: onPreview,
              onCommit: onCommit,
              onFinish: onFinish,
              onCancel: onCancel,
            ),
          ),
        ],
      ),
    );
  }
}

/// Name, meaning, the motion strip and (when not compact) the parameters.
/// Only the strip listens to the motion; the rest is plain values.
class _EaseInfo extends StatelessWidget {
  const _EaseInfo({
    required this.width,
    required this.leading,
    required this.kind,
    required this.previewKind,
    required this.shown,
    required this.free,
    required this.motion,
    required this.onPlay,
    required this.fields,
  });
  final double width;
  final Widget? leading;
  final String kind;
  final String? previewKind;
  final Map<String, dynamic> shown;
  final bool free;
  final Animation<double> motion;
  final VoidCallback onPlay;
  final List<Widget> fields;

  @override
  Widget build(BuildContext context) => SizedBox(
    width: width,
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        Row(
          children: [
            if (leading != null) ...[
              leading!,
              const SizedBox(width: EditorMetrics.s4),
            ],
            Expanded(
              child: Text(
                _curveName(kind),
                key: const ValueKey('ease-name'),
                maxLines: 2,
                style: const TextStyle(
                  fontSize: EditorMetrics.title,
                  fontWeight: FontWeight.w600,
                ),
              ),
            ),
          ],
        ),
        const SizedBox(height: EditorMetrics.s2),
        Text(
          previewKind == null
              ? _curveMeaning(kind)
              : '${_curveName(previewKind!)} · ${_curveMeaning(previewKind!)}',
          key: const ValueKey('ease-meaning'),
          maxLines: 2,
          overflow: TextOverflow.ellipsis,
          style: TextStyle(
            fontSize: EditorMetrics.font,
            color: EditorTheme.of(context).muted,
          ),
        ),
        const SizedBox(height: EditorMetrics.s6),
        SizedBox(
          height: EditorMetrics.bar,
          child: Row(
            children: [
              _EaseIcon(
                tooltip: 'Preview motion',
                icon: Glyph.play_arrow_outlined,
                color: EditorTheme.of(context).ink,
                onPressed: onPlay,
              ),
              const SizedBox(width: EditorMetrics.s4),
              Expanded(
                child: AnimatedBuilder(
                  animation: motion,
                  builder: (_, __) => Semantics(
                    label: 'Motion from start to finish',
                    child: CustomPaint(
                      key: const ValueKey('ease-motion'),
                      painter: EaseMotionPainter(
                        colors: EditorTheme.of(context),
                        shape: shown,
                        time: motion.value,
                        free: free,
                      ),
                      child: const SizedBox.expand(),
                    ),
                  ),
                ),
              ),
            ],
          ),
        ),
        if (fields.isNotEmpty) ...[
          const SizedBox(height: EditorMetrics.s4),
          Wrap(spacing: EditorMetrics.s8, children: fields),
        ],
      ],
    ),
  );
}

/// The interval line; only the frame readout listens to the head.
class _EaseTargetRow extends StatelessWidget {
  const _EaseTargetRow({
    required this.target,
    required this.title,
    required this.frame,
    required this.playhead,
  });
  final String target, title;
  final ValueListenable<int> frame;
  final double? Function() playhead;

  @override
  Widget build(BuildContext context) => SizedBox(
    height: EditorMetrics.row,
    child: EditorTooltip(
      message: target,
      child: Row(
        children: [
          Expanded(
            child: Text(
              title,
              key: const ValueKey('ease-interval-target'),
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: TextStyle(
                fontSize: EditorMetrics.dense,
                color: EditorTheme.of(context).ink,
              ),
            ),
          ),
          const SizedBox(width: EditorMetrics.s4),
          ListenableBuilder(
            listenable: frame,
            builder: (context, _) {
              final u = playhead();
              return Text(
                '${frame.value} f${u == null
                    ? ''
                    : u < 0
                    ? ' · before'
                    : u > 1
                    ? ' · after'
                    : ''}',
                key: const ValueKey('ease-current-frame'),
                style: TextStyle(
                  fontSize: EditorMetrics.dense,
                  color: u != null && (u < 0 || u > 1)
                      ? EditorTheme.of(context).muted
                      : EditorTheme.of(context).accent,
                ),
              );
            },
          ),
        ],
      ),
    ),
  );
}

/// The strip of selected intervals under the head; repaints with the frame.
class _EaseRail extends StatelessWidget {
  const _EaseRail({
    required this.target,
    required this.label,
    required this.hasInterval,
    required this.frame,
    required this.segments,
    required this.active,
  });
  final String target, label;
  final bool hasInterval;
  final ValueListenable<int> frame;
  final List<Map<String, dynamic>> segments;
  final int active;

  @override
  Widget build(BuildContext context) => SizedBox(
    height: EditorMetrics.s12,
    child: EditorTooltip(
      message: target,
      child: ListenableBuilder(
        listenable: frame,
        builder: (context, _) => Semantics(
          label: hasInterval ? '$label, playhead at ${frame.value} f' : label,
          child: CustomPaint(
            key: const ValueKey('ease-interval-rail'),
            painter: EaseIntervalPainter(
              colors: EditorTheme.of(context),
              segments: segments,
              active: active,
              frame: frame.value,
            ),
            child: const SizedBox.expand(),
          ),
        ),
      ),
    ),
  );
}

class _OvershootToggle extends StatelessWidget {
  const _OvershootToggle({
    required this.free,
    required this.narrow,
    required this.onPressed,
  });
  final bool free, narrow;
  final VoidCallback onPressed;

  @override
  Widget build(BuildContext context) => EditorTooltip(
    message: 'Overshoot',
    child: EditorTextButton(
      foreground: free
          ? EditorInk.dark.easePaper
          : EditorTheme.of(context).muted,
      minimumSize: Size.zero,
      onPressed: onPressed,
      child: Semantics(
        toggled: free,
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(
              free ? Glyph.check_box_outlined : Glyph.check_box_outline_blank,
              size: EditorMetrics.s14,
            ),
            if (!narrow) ...[
              const SizedBox(width: EditorMetrics.s3),
              const Text(
                'Overshoot',
                style: TextStyle(fontSize: EditorMetrics.dense),
              ),
            ],
          ],
        ),
      ),
    ),
  );
}

class _ApplyButton extends StatelessWidget {
  const _ApplyButton({required this.message, required this.onPressed});
  final String message;
  final VoidCallback? onPressed;

  @override
  Widget build(BuildContext context) => EditorTooltip(
    message: message,
    child: EditorTextButton(
      background: EditorInk.dark.easePaper,
      foreground: EditorInk.dark.easeInk,
      disabledBackground: EditorTheme.of(context).washDisabled,
      border: BorderSide.none,
      padding: const EdgeInsets.symmetric(horizontal: EditorMetrics.s8),
      // The compact density Material took 8 off the declared 52 × 24.
      minimumSize: const Size(
        EditorMetrics.field - EditorMetrics.s8,
        EditorMetrics.control - EditorMetrics.s8,
      ),
      radius: const BorderRadius.all(Radius.circular(EditorMetrics.s4)),
      textStyle: const TextStyle(
        fontSize: EditorMetrics.font,
        fontWeight: FontWeight.w600,
      ),
      onPressed: onPressed,
      child: const Text('Apply'),
    ),
  );
}
