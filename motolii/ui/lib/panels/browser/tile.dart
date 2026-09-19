import 'dart:math' as math;

import 'package:flutter/gestures.dart' show kPrimaryButton;
import 'package:flutter/widgets.dart';

import '../../foundation/metrics.dart';
import '../../foundation/theme.dart';
import 'parts.dart';
import 'shelf.dart';
import '../../foundation/glyphs.dart';

/// One tile of the grid: the shelf's preview in the frame's caption, badge,
/// marks and selection, laid out for the view in force.
class ShelfTile extends StatelessWidget {
  const ShelfTile({
    super.key,
    required this.host,
    required this.item,
    required this.hovered,
    required this.selected,
  });
  final BrowserHost host;
  final Map<String, dynamic> item;
  final bool hovered;
  final bool selected;

  @override
  Widget build(BuildContext context) {
    final host = this.host,
        item = this.item,
        hovered = this.hovered,
        isSelected = selected;
    final shelf = host.shelf;
    final tileScale = host.tileScale;
    final captionSize = math.max(
      EditorMetrics.micro,
      EditorMetrics.font * tileScale,
    );
    final captionHeight = EditorMetrics.control * tileScale;
    // A mark grows slower than the picture it marks: by the square root, so
    // at twice the tile it is 1.4× and stays a mark beside the name.
    final markScale = math.sqrt(tileScale);
    final supported = shelf.supported(host, item);
    final bare = shelf.bare;
    final twice = shelf.doubleClick(host, item);
    final identityColor = EditorTheme.kindColor(shelf.identity(host, item));
    final missing = item['missing'] == true;
    final name = '${item['name'] ?? item['id']}';
    final format = shelf.format(host, item);
    Widget preview = shelf.preview(host, item, identityColor);
    // Every preview sits in the same ground so light and dark pictures read
    // as separate tiles; colours are their own ground.
    if (!bare) preview = ColoredBox(color: EditorTheme.app, child: preview);
    // Marks ride on the picture, never on the frame: the frame is only ever
    // the selection. A pale dot means the item already sits in a layer; the
    // warning means its file is gone.
    if (missing || item['used'] == true)
      preview = Stack(
        fit: StackFit.expand,
        children: [
          preview,
          Positioned(
            left: EditorMetrics.s4,
            top: EditorMetrics.s4,
            child: missing
                ? const Icon(
                    Glyph.error_outline,
                    size: EditorMetrics.dense,
                    color: EditorTheme.accent,
                  )
                : Container(
                    key: const ValueKey('browser:used'),
                    width: EditorMetrics.s5,
                    height: EditorMetrics.s5,
                    decoration: const BoxDecoration(
                      color: EditorTheme.ink,
                      shape: BoxShape.circle,
                    ),
                  ),
          ),
        ],
      );
    // The caption is one line: the name takes the tile's width, and the
    // format sits at the right edge as a small badge in its family's colour,
    // the way AEViewer labels a card.
    final badge = format.isEmpty
        ? null
        : Container(
            key: ValueKey('browser:format:${host.id(item)}'),
            height: EditorMetrics.s14 * markScale,
            padding: EdgeInsets.symmetric(
              horizontal: EditorMetrics.s4 * markScale,
            ),
            alignment: Alignment.center,
            decoration: BoxDecoration(
              color: identityColor,
              borderRadius: BorderRadius.circular(EditorMetrics.s3 * markScale),
            ),
            child: Text(
              format,
              maxLines: 1,
              style: TextStyle(
                fontSize: EditorMetrics.micro * markScale,
                fontWeight: FontWeight.w600,
                letterSpacing: .5,
                color: EditorTheme.tabInk,
              ),
            ),
          );
    final badgeRoom = badge == null
        ? 0.0
        : (_badgeWidth(format) + EditorMetrics.s4) * markScale;
    final air = EditorMetrics.s4 * tileScale;
    final caption = SizedBox(
      key: ValueKey('browser:name:${host.id(item)}'),
      height: captionHeight,
      child: Padding(
        padding: EdgeInsets.symmetric(horizontal: air),
        child: Row(
          children: [
            Expanded(
              child: Align(
                alignment: Alignment.centerLeft,
                child: FittedName(
                  displayName(host, item),
                  host.tileWidth - air * 2 - badgeRoom,
                  size: captionSize,
                  sliding: hovered,
                ),
              ),
            ),
            if (badge != null) ...[SizedBox(width: air), badge],
          ],
        ),
      ),
    );
    final body = EditorTooltip(
      key: ValueKey('browser:${host.tab}:${host.id(item)}'),
      message: [
        name,
        if (missing) 'Missing file',
        if (item['used'] == true) 'In use by a layer',
        if (item['detail'] != null) '${item['detail']}',
        supported
            ? (bare && !twice
                  ? 'Click to apply · Right-click for actions'
                  : 'Double-click or Enter to apply · Right-click for actions')
            : 'Apply unavailable',
      ].join('\n'),
      // The pick is on the press itself, below the gesture arena: a tap
      // recognizer beside a double-tap one reports its down only after the
      // press deadline.
      child: Listener(
        onPointerDown: (e) {
          if (e.buttons == kPrimaryButton) host.select(item);
        },
        child: GestureDetector(
          behavior: HitTestBehavior.opaque,
          onTap: bare && supported && !twice ? () => host.apply(item) : null,
          onSecondaryTapDown: (event) => host.menu(item, event.globalPosition),
          onDoubleTap: !bare || (twice && supported)
              ? () => host.apply(item)
              : null,
          child: Container(
            decoration: BoxDecoration(
              color: EditorTheme.panel,
              border: Border.all(
                color: isSelected ? EditorTheme.spatial : EditorTheme.clear,
              ),
            ),
            child: Stack(
              children: [
                Positioned.fill(
                  child: bare
                      ? preview
                      : host.viewMode == 2
                      ? Stack(
                          fit: StackFit.expand,
                          children: [
                            preview,
                            // The chosen card says its name, and so does the
                            // one under the pointer: a band over the picture's
                            // foot, so the picture stays the point.
                            if (isSelected || hovered)
                              Positioned(
                                left: 0,
                                right: 0,
                                bottom: 0,
                                child: Container(
                                  key: ValueKey(
                                    'browser:band:${host.id(item)}',
                                  ),
                                  height: EditorMetrics.row * tileScale,
                                  padding: EdgeInsets.only(
                                    left: air,
                                    right: badgeRoom + air,
                                  ),
                                  alignment: Alignment.centerLeft,
                                  color: EditorTheme.app.withValues(alpha: .75),
                                  child: FittedName(
                                    displayName(host, item),
                                    host.tileWidth - air * 2 - badgeRoom,
                                    size: captionSize,
                                    sliding: hovered,
                                  ),
                                ),
                              ),
                            if (badge != null)
                              Positioned(right: air, bottom: air, child: badge),
                          ],
                        )
                      : host.viewMode == 1
                      ? Row(
                          children: [
                            SizedBox(width: EditorMetrics.s76, child: preview),
                            Expanded(child: caption),
                          ],
                        )
                      : Column(
                          children: [
                            Expanded(
                              child: SizedBox(
                                width: double.infinity,
                                child: preview,
                              ),
                            ),
                            caption,
                          ],
                        ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
    final wrapped = shelf.draggable(host, item, body);
    // A shelf that does not drag its tiles out still lets the frame drag them
    // onto a collection in the rail (the picked rows come along).
    if (!identical(wrapped, body)) return wrapped;
    return Draggable<BrowserDrag>(
      data: BrowserDrag(host.tab, {...host.selectedIds, host.id(item)}),
      dragAnchorStrategy: pointerDragAnchorStrategy,
      feedback: dragFeedback(
        host.selectedIds.contains(host.id(item)) && host.selectedIds.length > 1
            ? '${host.selectedIds.length} rows'
            : name,
      ),
      child: body,
    );
  }
}

/// The badge already carries the extension, so the name drops it and keeps
/// the width for the part that tells one file from another.
String displayName(BrowserHost host, Map<String, dynamic> item) {
  final name = '${item['name'] ?? item['id']}';
  final dot = name.lastIndexOf('.');
  return dot > 0 &&
          name.substring(dot + 1).toUpperCase() == host.shelf.format(host, item)
      ? name.substring(0, dot)
      : name;
}

/// The badge's laid-out width, so the name knows how much of the caption
/// it keeps. Formats repeat across every tile; lay each out once.
final _badgeWidths = <String, double>{};
double _badgeWidth(String format) => _badgeWidths[format] ??= () {
  final painter = TextPainter(
    text: TextSpan(
      text: format,
      style: const TextStyle(
        fontSize: EditorMetrics.micro,
        fontWeight: FontWeight.w600,
        letterSpacing: .5,
      ),
    ),
    maxLines: 1,
    textDirection: TextDirection.ltr,
  )..layout();
  final width = painter.width + EditorMetrics.s4 * 2;
  painter.dispose();
  return width;
}();

/// What a tile dragged inside the Browser carries: the shelf and the ids
/// (a collection row in the rail takes them).
class BrowserDrag {
  const BrowserDrag(this.shelf, this.ids);
  final String shelf;
  final Set<String> ids;
}

Widget dragFeedback(String label) => DefaultTextStyle(
  style: EditorTheme.text,
  child: Container(
    height: EditorMetrics.row,
    padding: const EdgeInsets.symmetric(horizontal: EditorMetrics.s6),
    decoration: BoxDecoration(
      color: EditorTheme.panel,
      border: Border.all(color: EditorTheme.border),
    ),
    child: Text(
      label,
      style: const TextStyle(
        fontSize: EditorMetrics.font,
        color: EditorTheme.ink,
      ),
    ),
  ),
);
