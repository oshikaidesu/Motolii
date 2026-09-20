part of '../browser.dart';

/// How big a tile is and what one carries: the size knob the desk stores,
/// the card with its collection dot, and the bar that turns the size and
/// says what the count counts.

/// One knob for how big Browser tiles are; Settings turns it, Browser reads it.
abstract final class BrowserSize {
  static const double min = 72, max = 240, base = 120;

  /// A stored size below the floor came from an older, smaller scale; the
  /// default stands in until the shelf is sized again.
  static double tile(EditorSession c) {
    final stored = (c.deskWork.value['browserTile'] as num?)?.toDouble();
    if (stored == null || stored < min) return _BrowserPanelState.tileDefault;
    return stored.clamp(min, max);
  }
}

/// 寸法棒が畳めない押し所 2 つ分。棒が出ない幅では、これに %の欄が足される。
const _zoomFloor = EditorMetrics.row * 2;

extension BrowserFrameGrid on _BrowserPanelState {
  /// Tile size as a percentage of its default.
  Widget _zoomBar() => Container(
    height: EditorMetrics.tall,
    padding: const EdgeInsets.symmetric(
      horizontal: _BrowserPanelState._inset,
      vertical: EditorMetrics.s4,
    ),
    decoration: BoxDecoration(
      border: Border(top: BorderSide(color: EditorTheme.of(context).line)),
    ),
    child: LayoutBuilder(
      builder: (context, box) {
        // 譲る順は 件数 → 寸法棒。寸法棒は、棒が出る幅ならその分、
        // 出ない幅でも押し所 2 つ分を必ず残す。
        final slider =
            box.maxWidth >= EditorZoomBar.sliderRoom + EditorMetrics.s48;
        final countRoom =
            box.maxWidth -
            (slider
                ? EditorZoomBar.sliderRoom
                : _zoomFloor + EditorMetrics.s64) -
            EditorMetrics.s8;
        final compact = countRoom < EditorMetrics.s48;
        return Row(
          children: [
            Expanded(child: _sizeSlider()),
            ConstrainedBox(
              constraints: BoxConstraints(
                maxWidth: countRoom.clamp(0.0, EditorMetrics.s96),
              ),
              child: Padding(
                padding: const EdgeInsets.only(left: EditorMetrics.s8),
                child: ValueListenableBuilder<Set<String>>(
                  valueListenable: picked,
                  builder: (context, _, _) => EditorTooltip(
                    message: _countTip(),
                    child: Text(
                      compact ? '${visible.length}' : _countLabel(),
                      key: const ValueKey('browser:count'),
                      maxLines: 1,
                      softWrap: false,
                      overflow: TextOverflow.ellipsis,
                      style: TextStyle(
                        fontSize: EditorMetrics.dense,
                        color: EditorTheme.of(context).muted,
                      ),
                    ),
                  ),
                ),
              ),
            ),
          ],
        );
      },
    ),
  );

  Widget _sizeSlider() => EditorZoomBar(
    base: _BrowserPanelState.tileDefault,
    value: tile,
    min: BrowserSize.min,
    max: BrowserSize.max,
    keyPrefix: 'browser:tile',
    // A press moves a tenth of the default: one visible step of tile size.
    step: 10,
    onChanged: (v) => widget.controller.storeDesk('browserTile', v),
  );

  /// The number says what it counts in the same breath: the selection, the
  /// narrowed result against the whole tab, else the whole tab.
  String _countLabel() {
    final chosen = selected[tab]?.length ?? 0;
    if (chosen > 0) return '$chosen of $total selected';
    if (visible.length != total) return '${visible.length} of $total shown';
    return '$total items';
  }

  String _countTip() => [
    '$total items in $tab',
    '${visible.length} shown by the search and category',
    '${selected[tab]?.length ?? 0} selected',
  ].join('\n');

  /// A tile in a collection wears the collection's colour as a small dot in
  /// its lower right corner.
  Widget card(Map<String, dynamic> item) {
    final which = library.collectionOf(tab, id(item));
    final tile = _tile(item);
    if (which == null) return tile;
    return Stack(
      children: [
        tile,
        Positioned(
          right: EditorMetrics.s4,
          bottom: EditorMetrics.s4,
          child: IgnorePointer(
            child: Container(
              key: ValueKey('browser:collected:${id(item)}'),
              width: EditorMetrics.s6,
              height: EditorMetrics.s6,
              decoration: BoxDecoration(
                color: BrowserLibrary.collectionColors[which - 1],
                shape: BoxShape.circle,
                border: Border.all(
                  color: EditorTheme.of(context).app,
                  width: 1,
                ),
              ),
            ),
          ),
        ),
      ],
    );
  }

  Widget _tile(Map<String, dynamic> item) => Hover(
    builder: (hovered) =>
        ShelfTile(host: this, item: item, hovered: hovered, selection: picked),
  );
}
