part of '../browser.dart';

/// The frame's own bars: the search row across the top, the category rail
/// down the side (and the turned label it folds into), and what the grid
/// shows when nothing matches.

/// Search field, the shelf's tools, the filter toggle and the view switch.
class _BrowserSearchBar extends StatelessWidget {
  const _BrowserSearchBar({
    required this.tab,
    required this.search,
    required this.searchFocus,
    required this.tools,
    required this.filterable,
    required this.filtering,
    required this.views,
    required this.onChanged,
    required this.onToggleFilters,
  });
  final String tab;
  final TextEditingController search;
  final FocusNode searchFocus;
  final List<Widget> tools;
  final bool filterable, filtering;
  final Widget? views;
  final VoidCallback onChanged, onToggleFilters;

  @override
  Widget build(BuildContext context) => Container(
    height: EditorMetrics.tall,
    padding: const EdgeInsets.symmetric(
      horizontal: _BrowserPanelState._inset,
      vertical: EditorMetrics.s4,
    ),
    decoration: BoxDecoration(
      border: Border(bottom: BorderSide(color: EditorTheme.of(context).line)),
    ),
    child: Row(
      children: [
        Expanded(
          child: EditorFieldFrame(
            focus: searchFocus,
            padding: EdgeInsets.zero,
            child: EditorTextField(
              controller: search,
              focusNode: searchFocus,
              style: TextStyle(
                fontSize: EditorMetrics.font,
                color: EditorTheme.of(context).ink,
              ),
              prefix: ConstrainedBox(
                constraints: const BoxConstraints(
                  minWidth: EditorMetrics.control,
                  minHeight: EditorMetrics.row,
                ),
                child: Icon(
                  Glyph.search,
                  size: EditorMetrics.s14,
                  color: EditorTheme.of(context).muted,
                ),
              ),
              // The frame is one row tall: its border and one line are all
              // it has room for, so the padding here is sideways only.
              padding: const EdgeInsets.symmetric(horizontal: EditorMetrics.s4),
              hint: 'Search $tab',
              onChanged: (_) => onChanged(),
            ),
          ),
        ),
        for (final tool in tools) ...[
          const SizedBox(width: EditorMetrics.s6),
          tool,
        ],
        if (filterable) ...[
          const SizedBox(width: EditorMetrics.s6),
          EditorTooltip(
            message: 'Show filters',
            child: EditorPress(
              key: const ValueKey('browser:filters-toggle'),
              onTap: onToggleFilters,
              child: SizedBox.square(
                dimension: EditorMetrics.row,
                child: Icon(
                  Glyph.filter_list,
                  size: EditorMetrics.s14,
                  color: filtering
                      ? EditorTheme.of(context).accent
                      : EditorTheme.of(context).muted,
                ),
              ),
            ),
          ),
        ],
        if (views != null) ...[const SizedBox(width: EditorMetrics.s6), views!],
      ],
    ),
  );
}

/// The category rail: the tab's name, its entries, then the collections.
class _BrowserRail extends StatelessWidget {
  const _BrowserRail({
    required this.width,
    required this.tab,
    required this.rails,
    required this.chosen,
    required this.onRail,
    required this.collections,
  });
  final double width;
  final String tab, chosen;
  final List<String> rails;
  final ValueChanged<String> onRail;
  final Widget collections;

  @override
  Widget build(BuildContext context) => Container(
    width: width,
    clipBehavior: Clip.hardEdge,
    decoration: const BoxDecoration(),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Padding(
          padding: const EdgeInsets.fromLTRB(
            _BrowserPanelState._inset,
            EditorMetrics.s8,
            EditorMetrics.s4,
            EditorMetrics.s4,
          ),
          child: Text(
            tab.toUpperCase(),
            maxLines: 1,
            overflow: TextOverflow.clip,
            style: TextStyle(
              fontSize: EditorMetrics.micro,
              letterSpacing: 1,
              color: EditorTheme.of(context).muted,
            ),
          ),
        ),
        for (final entry in rails)
          shelfButton(entry, () => onRail(entry), selected: chosen == entry),
        Expanded(child: SingleChildScrollView(child: collections)),
      ],
    ),
  );
}

/// The folded rail: one turned label that opens it again.
class _BrowserRailTab extends StatelessWidget {
  const _BrowserRailTab({
    required this.tab,
    required this.label,
    required this.onTap,
  });
  final String tab, label;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) => EditorTooltip(
    message: 'Show ${tab.toLowerCase()} categories',
    child: EditorPress(
      key: const ValueKey('browser:rail-tab'),
      onTap: onTap,
      child: SizedBox(
        width: EditorMetrics.row,
        child: Column(
          children: [
            Padding(
              padding: EdgeInsets.only(top: EditorMetrics.s4),
              child: Icon(
                Glyph.chevron_right,
                size: EditorMetrics.s14,
                color: EditorTheme.of(context).muted,
              ),
            ),
            RotatedBox(
              quarterTurns: 1,
              child: Text(
                label,
                maxLines: 1,
                style: TextStyle(
                  fontSize: EditorMetrics.micro,
                  color: EditorTheme.of(context).accent,
                ),
              ),
            ),
          ],
        ),
      ),
    ),
  );
}

/// The grid's empty state.
class _NoMatches extends StatelessWidget {
  const _NoMatches();

  @override
  Widget build(BuildContext context) => Padding(
    padding: EdgeInsets.all(EditorMetrics.s8),
    child: Text(
      'No matches',
      style: TextStyle(
        color: EditorTheme.of(context).muted,
        fontSize: EditorMetrics.dense,
      ),
    ),
  );
}
