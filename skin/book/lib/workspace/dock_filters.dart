// The filter band under a shelf's search, after Motolii's browser (filter_view.dart, the Live 12 way): one block per group,
// its name with a fold mark, its tags as chips flowing across; a results bar under them with the count of filters on and Clear.
// Groups combine with AND, tags within a group with OR. A click picks one tag in its group; Cmd/Ctrl-click adds or removes.
import 'package:flutter/gestures.dart' show DragStartBehavior;
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../tokens.dart';
import 'dock_glyphs.dart';
import 'dock_parts.dart';
import 'ws.dart';

class DockFilterBand extends StatelessWidget {
  const DockFilterBand({
    super.key,
    required this.groups,
    required this.chosen,
    required this.folded,
    required this.count,
    required this.results,
    required this.onFold,
    required this.onToggle,
    required this.onClear,
  });
  final List<(String, List<String>)> groups;
  final Map<String, Set<String>> chosen;
  final Set<String> folded;

  /// How many rows a tag would leave, with the other groups as they are.
  final int Function(String group, String tag) count;
  final int results;
  final ValueChanged<String> onFold;
  final void Function(String group, String tag, bool add) onToggle;
  final VoidCallback onClear;

  int get _active => chosen.values.where((s) => s.isNotEmpty).length;

  @override
  Widget build(BuildContext context) => Column(
    key: const ValueKey('browser:filters'),
    mainAxisSize: MainAxisSize.min,
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      Expanded(
        child: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              const SizedBox(height: 2),
              for (final (name, tags) in groups) _Group(name, tags, band: this),
              const SizedBox(height: 4),
            ],
          ),
        ),
      ),
      _Results(results: results, active: _active, onClear: _active == 0 ? null : onClear),
    ],
  );
}

const _label = 78.0;

class _Group extends StatelessWidget {
  const _Group(this.name, this.tags, {required this.band});
  final String name;
  final List<String> tags;
  final DockFilterBand band;
  @override
  Widget build(BuildContext context) {
    final fold = band.folded.contains(name), on = band.chosen[name] ?? const <String>{};
    final head = DockHover(
      key: ValueKey('browser:filter-group:$name'),
      onTap: () => band.onFold(name),
      builder: (context, h) => SizedBox(
        width: _label,
        height: 20,
        child: Row(
          children: [
            AnimatedRotation(
              turns: fold ? 0 : .25,
              duration: Mo.dur,
              curve: Mo.ease,
              child: DockGlyph(DockG.chevron, size: 7, color: Grey.g56),
            ),
            const SizedBox(width: 4),
            Flexible(
              child: Text(
                name,
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                style: T.micro(h ? Grey.g95 : (on.isEmpty ? Grey.g76 : WsT.accent)).copyWith(fontWeight: FontWeight.w700),
              ),
            ),
          ],
        ),
      ),
    );
    return Padding(
      padding: const EdgeInsets.fromLTRB(WsT.inset, 2, WsT.inset, 2),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          head,
          Expanded(
            child: fold
                ? SizedBox(
                    height: 20,
                    child: Align(
                      alignment: Alignment.centerLeft,
                      child: Text(
                        on.isEmpty ? '${tags.length} tags' : on.join(', '),
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: T.micro(on.isEmpty ? Grey.g56 : WsT.accent),
                      ),
                    ),
                  )
                : Wrap(
                    spacing: WsT.gap,
                    runSpacing: WsT.gap,
                    children: [
                      for (final t in tags)
                        _Chip(
                          key: ValueKey('browser:filter:$name:$t'),
                          text: t,
                          count: band.count(name, t),
                          on: on.contains(t),
                          onTap: (add) => band.onToggle(name, t, add),
                        ),
                    ],
                  ),
          ),
        ],
      ),
    );
  }
}

/// A tag: rest well, hover one step up, chosen the accent. A tag that would leave nothing is quiet and inert unless chosen.
class _Chip extends StatelessWidget {
  const _Chip({super.key, required this.text, required this.count, required this.on, required this.onTap});
  final String text;
  final int count;
  final bool on;
  final ValueChanged<bool> onTap;
  @override
  Widget build(BuildContext context) {
    final dead = count == 0 && !on;
    return DockHover(
      onTap: dead ? null : () => onTap(HardwareKeyboard.instance.isMetaPressed || HardwareKeyboard.instance.isControlPressed),
      builder: (context, h) => AnimatedContainer(
        duration: Mo.dur,
        curve: Mo.ease,
        height: 20,
        padding: const EdgeInsets.symmetric(horizontal: 7),
        decoration: BoxDecoration(
          color: on ? WsT.accent : (dead ? null : (h ? WsT.raised : WsT.well)),
          borderRadius: BorderRadius.circular(10),
          border: dead ? Border.all(color: WsT.line) : null,
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Text(text, style: T.micro(on ? WsT.onAccent : (dead ? Grey.g56 : Grey.g91)).copyWith(fontWeight: on ? FontWeight.w700 : FontWeight.w500)),
            const SizedBox(width: 4),
            Text('$count', style: T.value(on ? WsT.onAccent : Grey.g56).copyWith(fontSize: 10)),
          ],
        ),
      ),
    );
  }
}

class _Results extends StatelessWidget {
  const _Results({required this.results, required this.active, this.onClear});
  final int results, active;
  final VoidCallback? onClear;
  @override
  Widget build(BuildContext context) => Container(
    height: 26,
    padding: const EdgeInsets.symmetric(horizontal: WsT.inset),
    color: active == 0 ? WsT.body : WsT.card,
    child: Row(
      children: [
        Text('Results', style: T.micro(Grey.g91).copyWith(fontWeight: FontWeight.w700)),
        const SizedBox(width: 8),
        Expanded(
          child: Text(
            active == 0 ? '$results' : '$results · $active ${active == 1 ? 'filter' : 'filters'}',
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
            style: T.value(active == 0 ? Grey.g56 : WsT.accent).copyWith(fontSize: 10),
          ),
        ),
        DockHover(
          key: const ValueKey('browser:filters-clear'),
          onTap: onClear,
          builder: (context, h) => Container(
            height: 18,
            padding: const EdgeInsets.symmetric(horizontal: 8),
            alignment: Alignment.center,
            decoration: BoxDecoration(color: h && onClear != null ? WsT.raised : null, borderRadius: BorderRadius.circular(9)),
            child: Text('Clear', style: T.micro(onClear == null ? Grey.g44 : Grey.g91)),
          ),
        ),
      ],
    ),
  );
}

/// The toggle beside the search: lit while the band is open, the accent while any filter is on.
class DockFilterToggle extends StatelessWidget {
  const DockFilterToggle({super.key, required this.open, required this.active, required this.onTap});
  final bool open;
  final int active;
  final VoidCallback onTap;
  @override
  Widget build(BuildContext context) => DockHover(
    key: const ValueKey('browser:filters-toggle'),
    onTap: onTap,
    builder: (context, h) => AnimatedContainer(
      duration: Mo.dur,
      curve: Mo.ease,
      height: 24,
      padding: const EdgeInsets.symmetric(horizontal: 6),
      decoration: BoxDecoration(color: active > 0 ? WsT.accent : (open || h ? WsT.raised : WsT.well), borderRadius: BorderRadius.circular(WsT.radius)),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          DockGlyph(DockG.filter, size: 12, color: active > 0 ? WsT.onAccent : (open || h ? Grey.g95 : Grey.g63)),
          if (active > 0) ...[const SizedBox(width: 3), Text('$active', style: T.value(WsT.onAccent).copyWith(fontSize: 10, fontWeight: FontWeight.w700))],
        ],
      ),
    ),
  );
}

/// A band whose height the user sets: the grip on its lower edge drags it up or down (clamped to [min]..[max]);
/// a double-click on the grip returns to the default.
class DockResizable extends StatefulWidget {
  const DockResizable({
    super.key,
    required this.height,
    required this.max,
    required this.onHeight,
    required this.child,
    this.min = 72,
    this.reset = dockFilterHeight,
    this.gripKey = const ValueKey('browser:filters-grip'),
  });

  /// [reset] is where a double-click on the grip puts the height back.
  final double height, min, max, reset;
  final Key gripKey;
  final ValueChanged<double> onHeight;
  final Widget child;
  @override
  State<DockResizable> createState() => _DockResizableState();
}

class _DockResizableState extends State<DockResizable> {
  final _box = GlobalKey();
  bool _hover = false, _drag = false;
  double _h = 0, _from = 0;

  double _clamp(double v) => v.clamp(widget.min, widget.max < widget.min ? widget.min : widget.max);

  @override
  Widget build(BuildContext context) {
    final lit = _hover || _drag;
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        SizedBox(key: _box, height: _clamp(widget.height), child: widget.child),
        MouseRegion(
          cursor: SystemMouseCursors.resizeRow,
          onEnter: (_) => setState(() => _hover = true),
          onExit: (_) => setState(() => _hover = false),
          child: GestureDetector(
            key: widget.gripKey,
            behavior: HitTestBehavior.opaque,
            dragStartBehavior: DragStartBehavior.down,
            onVerticalDragStart: (d) => setState(() {
              _drag = true;
              _from = d.globalPosition.dy;
              _h = _box.currentContext?.size?.height ?? widget.height;
            }),
            onVerticalDragUpdate: (d) => widget.onHeight(_clamp(_h + d.globalPosition.dy - _from)),
            onVerticalDragEnd: (_) => setState(() => _drag = false),
            onDoubleTap: () => widget.onHeight(widget.reset),
            child: Container(
              height: 7,
              color: lit ? WsT.raised : WsT.ground,
              alignment: Alignment.center,
              child: Container(
                width: 28,
                height: 3,
                decoration: BoxDecoration(color: lit ? WsT.accent : Grey.g38, borderRadius: BorderRadius.circular(1.5)),
              ),
            ),
          ),
        ),
      ],
    );
  }
}
