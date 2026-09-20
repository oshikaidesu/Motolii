part of '../inspector.dart';

// The Inspector's small widgets: they take what they show and hold no
// state of the panel, so they are read where they are drawn.

/// One reading of the Inspector: it rebuilds when the rows it names move and
/// stays as it is while the rest of the panel is rebuilt around it.
class _Live extends StatelessWidget {
  const _Live({required this.pulse, required this.body});
  final ValueNotifier<Object?> pulse;
  final Widget Function() body;
  @override
  Widget build(BuildContext context) => ValueListenableBuilder<Object?>(
    valueListenable: pulse,
    builder: (_, __, ___) => body(),
  );
}

/// A section inside a card: the same kicker as the card's title, on a rule.
class _SectionLabel extends StatelessWidget {
  const _SectionLabel(this.text);
  final String text;
  @override
  Widget build(BuildContext context) => Container(
    alignment: Alignment.bottomLeft,
    decoration: BoxDecoration(
      border: Border(bottom: BorderSide(color: EditorTheme.of(context).line)),
    ),
    child: Text(
      text.toUpperCase(),
      style: TextStyle(
        fontSize: EditorMetrics.micro,
        letterSpacing: 1,
        color: EditorTheme.of(context).muted,
      ),
    ),
  );
}

/// One glyph on a card's head; quiet, and quieter still when it cannot act.
class _HeadGlyph extends StatelessWidget {
  const _HeadGlyph({
    required this.icon,
    required this.tip,
    this.onTap,
    this.ink,
  });
  final IconData icon;
  final String tip;
  final VoidCallback? onTap;

  /// The glyph's colour when the head it sits on is not the panel grey.
  final Color? ink;
  @override
  Widget build(BuildContext context) => EditorTooltip(
    message: tip,
    child: GestureDetector(
      behavior: HitTestBehavior.opaque,
      onTap: onTap,
      child: Container(
        height: EditorMetrics.row,
        constraints: const BoxConstraints(minWidth: EditorMetrics.row),
        padding: const EdgeInsets.only(left: EditorMetrics.s6),
        alignment: Alignment.center,
        child: Icon(
          icon,
          size: EditorMetrics.s12,
          color: onTap == null
              ? EditorTheme.of(context).disabledInk
              : ink ?? EditorTheme.of(context).muted,
        ),
      ),
    ),
  );
}

/// The word above a cell's control.
class _CellLabel extends StatelessWidget {
  const _CellLabel(this.label, {this.hero = false});
  final String label;
  final bool hero;
  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.only(bottom: EditorMetrics.s2),
    child: Text(
      label,
      maxLines: 1,
      overflow: TextOverflow.ellipsis,
      style: TextStyle(
        fontSize: EditorMetrics.micro,
        color: hero
            ? EditorTheme.of(context).ink
            : EditorTheme.of(context).muted,
      ),
    ),
  );
}

/// The handle that reorders an effect within its pipeline.
class _EffectGrip extends StatelessWidget {
  const _EffectGrip({required this.index});
  final int index;
  @override
  Widget build(BuildContext context) => ReorderableDragStartListener(
    index: index,
    child: EditorTooltip(
      message: 'Drag to reorder',
      child: MouseRegion(
        cursor: SystemMouseCursors.grab,
        child: Padding(
          padding: EdgeInsets.only(right: EditorMetrics.s4),
          child: Icon(
            Glyph.drag_indicator,
            size: EditorMetrics.s12,
            color: EditorTheme.of(context).muted,
          ),
        ),
      ),
    ),
  );
}

/// An effect's advanced rows behind one fold. Listens to the fold set alone,
/// so opening one effect rebuilds this and nothing above it.
class _AdvancedFold extends StatelessWidget {
  const _AdvancedFold({
    required this.opened,
    required this.id,
    required this.builder,
  });
  final ValueNotifier<Set<String>> opened;
  final String id;
  final Widget Function() builder;
  @override
  Widget build(BuildContext context) => Picked<Set<String>>(
    of: opened,
    test: (set) => set.contains(id),
    builder: (open) => Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        const SizedBox(height: EditorMetrics.s4),
        EditorFold(
          open: open,
          onTap: () => opened.value = open
              ? ({...opened.value}..remove(id))
              : {...opened.value, id},
        ),
        if (open) ...[const SizedBox(height: EditorMetrics.s4), builder()],
      ],
    ),
  );
}

/// The die beside a seed well.
class _SeedRoll extends StatelessWidget {
  const _SeedRoll({required this.onTap});
  final VoidCallback? onTap;
  @override
  Widget build(BuildContext context) => EditorTooltip(
    message: 'Roll a new seed',
    child: GestureDetector(
      behavior: HitTestBehavior.opaque,
      onTap: onTap,
      child: Icon(
        Glyph.casino_outlined,
        size: EditorMetrics.s16,
        color: EditorTheme.of(context).muted,
      ),
    ),
  );
}

/// One of the three spaces a layer can declare (2D, 2.5D, 3D).
class _SpaceChoice extends StatelessWidget {
  const _SpaceChoice({
    required this.projection,
    required this.selected,
    required this.onPick,
  });
  final String projection;
  final bool selected;
  final VoidCallback? onPick;
  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.only(right: EditorMetrics.s2),
    child: EditorTooltip(
      message: projection,
      child: EditorButton(projection, onPick, selected: selected),
    ),
  );
}
