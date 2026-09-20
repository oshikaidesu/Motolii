import 'package:flutter/widgets.dart';

import '../glyphs.dart';
import '../leaves.dart';
import '../metrics.dart';
import '../theme.dart';

/// What the controls sit in: bars, cards, heads, folds and the anchor's grid.

Widget panelButton(
  String label,
  VoidCallback? action, {
  String? tooltip,
  bool selected = false,
}) =>
    EditorButton(label, action, tooltip: tooltip ?? label, selected: selected);
Widget panelTitle(String title) => Builder(
  builder: (context) => Container(
    height: EditorMetrics.s22,
    alignment: Alignment.centerLeft,
    padding: const EdgeInsets.symmetric(horizontal: EditorMetrics.s8),
    decoration: BoxDecoration(
      color: EditorTheme.of(context).raised,
      border: Border(bottom: BorderSide(color: EditorTheme.of(context).line)),
    ),
    child: Text(
      title,
      style: TextStyle(
        fontSize: EditorMetrics.font,
        color: EditorTheme.of(context).ink,
      ),
    ),
  ),
);

/// A panel bar. Its Row keeps Spacer alignment while the content fits and
/// slides sideways when it does not, instead of overflowing the pane.
class EditorBar extends StatelessWidget {
  const EditorBar({
    super.key,
    required this.children,
    this.height = EditorMetrics.s22,
    this.padding = EdgeInsets.zero,
    this.decoration,
  });
  final List<Widget> children;
  final double height;
  final EdgeInsetsGeometry padding;
  final BoxDecoration? decoration;
  @override
  Widget build(BuildContext context) => Container(
    height: height,
    decoration:
        decoration ?? BoxDecoration(color: EditorTheme.of(context).panel),
    child: LayoutBuilder(
      builder: (context, box) => SingleChildScrollView(
        scrollDirection: Axis.horizontal,
        child: ConstrainedBox(
          constraints: BoxConstraints(minWidth: box.maxWidth),
          child: IntrinsicWidth(
            child: Padding(
              padding: padding,
              child: Row(children: children),
            ),
          ),
        ),
      ),
    ),
  );
}

/// The line that hides the seldom-used rows of a card: a chevron, the word,
/// a rule. The same line on every card, so the eye learns it once.
class EditorFold extends StatelessWidget {
  const EditorFold({required this.open, required this.onTap});
  final bool open;
  final VoidCallback onTap;
  @override
  Widget build(BuildContext context) => EditorTooltip(
    message: open ? 'Hide the advanced controls' : 'Show the advanced controls',
    child: GestureDetector(
      behavior: HitTestBehavior.opaque,
      onTap: onTap,
      child: Row(
        children: [
          Icon(
            open ? Glyph.expand_more : Glyph.chevron_right,
            size: EditorMetrics.s14,
            color: EditorTheme.of(context).muted,
          ),
          const SizedBox(width: EditorMetrics.s2),
          Text(
            'Advanced',
            style: TextStyle(
              fontSize: EditorMetrics.dense,
              color: EditorTheme.of(context).muted,
            ),
          ),
          const SizedBox(width: EditorMetrics.s6),
          const Expanded(child: EditorRule(height: 1)),
        ],
      ),
    ),
  );
}

/// Where the layer turns and scales from: nine places, the current one lit.
class EditorAnchorGrid extends StatelessWidget {
  const EditorAnchorGrid({
    super.key,
    required this.fraction,
    required this.onPick,
    this.onHover,
    this.cell = EditorMetrics.s14,
  });
  final List<double>? fraction;
  final double cell;
  final void Function(double x, double y)? onPick;

  /// The cell under the pointer, or null when it leaves — so the Stage can
  /// show where that pivot would land before it is chosen.
  final void Function(double x, double y, bool inside)? onHover;
  @override
  Widget build(BuildContext context) => EditorTooltip(
    message: 'Anchor',
    child: Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        for (final y in [0.0, .5, 1.0])
          Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              for (final x in [0.0, .5, 1.0])
                MouseRegion(
                  onEnter: (_) => onHover?.call(x, y, true),
                  onExit: (_) => onHover?.call(x, y, false),
                  child: GestureDetector(
                    onTap: onPick == null ? null : () => onPick!(x, y),
                    child: Container(
                      width: cell,
                      height: cell,
                      margin: const EdgeInsets.all(1),
                      decoration: BoxDecoration(
                        color:
                            fraction != null &&
                                (fraction![0] - x).abs() < .05 &&
                                (fraction![1] - y).abs() < .05
                            ? EditorTheme.of(context).accent
                            : onPick == null
                            ? EditorTheme.of(context).raised
                            : EditorTheme.of(context).border,
                        borderRadius: BorderRadius.circular(EditorMetrics.s2),
                      ),
                    ),
                  ),
                ),
            ],
          ),
      ],
    ),
  );
}

/// A group of controls on one raised sheet, named by a glyph and a kicker.
class EditorCard extends StatelessWidget {
  static const contentInset = EditorMetrics.s6;
  const EditorCard({
    super.key,
    required this.title,
    required this.children,
    this.leading,
    this.trailing,
    this.dim = false,
    this.expanded = true,
    this.onToggle,
  });
  final String title;
  final List<Widget> children;
  final Widget? leading, trailing;
  final bool expanded;
  final VoidCallback? onToggle;

  /// The card's contents faded: what it holds is not applied right now.
  final bool dim;
  @override
  Widget build(BuildContext context) => DefaultTextStyle(
    style: EditorTheme.of(context).text,
    child: Container(
      padding: const EdgeInsets.symmetric(horizontal: contentInset),
      decoration: BoxDecoration(
        color: EditorTheme.of(context).panel,
        border: Border(bottom: BorderSide(color: EditorTheme.of(context).line)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          _SectionHead(
            title: title,
            leading: leading,
            trailing: trailing,
            expanded: expanded,
            onToggle: onToggle,
          ),
          if (expanded) ...[
            // The head sits close to its rows; the rows are a step apart; the
            // card ends a step past its last row, whatever that row is.
            const SizedBox(height: EditorMetrics.s2),
            if (dim)
              Opacity(
                opacity: .4,
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  spacing: EditorMetrics.s4,
                  children: children,
                ),
              )
            else
              Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                spacing: EditorMetrics.s4,
                children: children,
              ),
            const SizedBox(height: EditorMetrics.s8),
          ],
        ],
      ),
    ),
  );
}

/// A section's head row: its name, the fold mark when it folds, and what the
/// caller puts at either end.
class _SectionHead extends StatelessWidget {
  const _SectionHead({
    required this.title,
    required this.leading,
    required this.trailing,
    required this.expanded,
    required this.onToggle,
  });
  final String title;
  final Widget? leading, trailing;
  final bool expanded;
  final VoidCallback? onToggle;
  @override
  Widget build(BuildContext context) => SizedBox(
    height: EditorMetrics.row,
    child: Row(
      children: [
        if (leading != null) leading!,
        Expanded(
          child: Semantics(
            expanded: expanded,
            child: EditorPress(
              onTap: onToggle,
              child: SizedBox(
                height: EditorMetrics.row,
                child: Row(
                  children: [
                    if (onToggle != null)
                      Transform.translate(
                        offset: const Offset(-1.5, 0),
                        child: Icon(
                          expanded ? Glyph.expand_more : Glyph.chevron_right,
                          size: EditorMetrics.s14,
                          color: EditorTheme.of(context).muted,
                        ),
                      ),
                    Expanded(
                      child: Text(
                        title.toUpperCase(),
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: TextStyle(
                          fontSize: EditorMetrics.micro,
                          letterSpacing: 1,
                          color: EditorTheme.of(context).muted,
                        ),
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ),
        ),
        if (trailing != null) trailing!,
      ],
    ),
  );
}
