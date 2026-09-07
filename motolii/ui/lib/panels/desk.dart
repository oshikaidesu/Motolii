import 'dart:convert';

import 'package:flutter/material.dart';

import '../session/editor_session.dart';
import '../foundation/panel_catalog.dart';
import '../foundation/theme.dart';
import '../foundation/metrics.dart';

class DeskPanel extends StatefulWidget {
  const DeskPanel({
    super.key,
    required this.controller,
    required this.panelBuilder,
  });
  final EditorSession controller;
  final Widget Function(String) panelBuilder;
  @override
  State<DeskPanel> createState() => _DeskPanelState();
}

class _DeskPanelState extends State<DeskPanel> {
  EditorSession get c => widget.controller;
  String? _automatic;
  String _selection = '';
  bool _inside = false;
  String _name(String value) =>
      value == 'Text' || value == 'Reference' ? 'Notes' : value;
  String get _identity => jsonEncode([
    c.selectedIds,
    c.state['selectedKeys'],
    c.activeLayer?['kind'],
  ]);
  String get _shown {
    final manual = c.deskDrawer.value;
    if (manual == 'Tools') return 'Tools';
    for (final candidate in [manual, _automatic, c.deskDefault.value]) {
      if (candidate == null) continue;
      final name = _name(candidate);
      if (panelSpec(name)?.drawer == true && _inDrawer(name)) return name;
    }
    return 'Tools';
  }

  bool _inDrawer(String name) =>
      c.panePlaces.value[name] == null || c.panePlaces.value[name] == 'drawer';
  String? _selectionPanel() {
    if (EditorSession.maps(c.state['selectedKeys']).isNotEmpty) return 'Ease';
    // 複数の層を選んだら Sequence(Stagger): 遅れを曲線で配る Ease のゴーストモード。
    if (c.selectedIds.length > 1) return 'Ease';
    return switch (c.activeLayer?['kind']) {
      'Camera' => 'Depth',
      _ => null,
    };
  }

  @override
  void initState() {
    super.initState();
    _selection = _identity;
    _automatic = _selectionPanel();
    c.document.addListener(_read);
    c.editingFocus.addListener(_focus);
  }

  @override
  void dispose() {
    c.document.removeListener(_read);
    c.editingFocus.removeListener(_focus);
    super.dispose();
  }

  void _read() {
    if (_identity == _selection) return;
    _selection = _identity;
    if (!_inside) _follow(_selectionPanel());
  }

  void _follow(String? name) {
    c.deskDrawer.value = null;
    setState(() => _automatic = name);
  }

  void _focus() {
    final focus = c.editingFocus.value;
    if (focus['selection'] == true) {
      if (!_inside) _follow(_selectionPanel());
      return;
    }
    final layer = c.layers.where((l) => l['id'] == focus['layer']).firstOrNull;
    if (layer == null || !c.selectedIds.contains(layer['id'])) return;
    final property = focus['property'];
    _follow(
      property == 'keyframes'
          ? 'Ease'
          : layer['kind'] == 'Camera'
          ? 'Depth'
          : property == 'blendMode'
          ? 'Blend'
          : null,
    );
  }

  void _open(String name) {
    if (_inDrawer(name)) {
      setState(() => c.deskDrawer.value = name);
    } else {
      c.placePanel(name, 'show');
    }
  }

  Widget _catalog() => ListView(
    children: [
      for (final spec in panelCatalog.where(
        (p) => p.drawer && _inDrawer(p.name),
      ))
        InkWell(
          onTap: () => _open(spec.name),
          child: SizedBox(
            height: EditorMetrics.control,
            child: Row(
              children: [
                const SizedBox(width: EditorMetrics.s8),
                Icon(spec.icon, size: EditorMetrics.s16),
                const SizedBox(width: EditorMetrics.s8),
                Expanded(
                  child: Text(
                    spec.name,
                    style: const TextStyle(fontSize: EditorMetrics.font),
                  ),
                ),
                IconButton(
                  tooltip: 'Use ${spec.name} when idle',
                  iconSize: EditorMetrics.s14,
                  padding: EdgeInsets.zero,
                  constraints: const BoxConstraints.tightFor(
                    width: EditorMetrics.control,
                    height: EditorMetrics.control,
                  ),
                  color: _name(c.deskDefault.value) == spec.name
                      ? EditorTheme.accent
                      : EditorTheme.muted,
                  icon: Icon(
                    _name(c.deskDefault.value) == spec.name
                        ? Icons.star
                        : Icons.star_border,
                  ),
                  onPressed: () => c.deskDefault.value =
                      _name(c.deskDefault.value) == spec.name
                      ? 'Tools'
                      : spec.name,
                ),
              ],
            ),
          ),
        ),
    ],
  );
  @override
  Widget build(BuildContext context) => AnimatedBuilder(
    animation: Listenable.merge([
      c.document,
      c.deskDrawer,
      c.deskDefault,
      c.panePlaces,
    ]),
    builder: (context, _) {
      final shown = _shown;
      final spec = panelSpec(shown);
      final live = spec?.drawer == true && _inDrawer(shown);
      return TapRegion(
        onTapInside: (_) => _inside = true,
        onTapOutside: (_) => _inside = false,
        child: Focus(
          canRequestFocus: false,
          onFocusChange: (focused) => _inside = focused,
          child: Material(
            color: EditorTheme.panel,
            child: Column(
              children: [
                SizedBox(
                  height: EditorMetrics.bar,
                  child: Row(
                    children: [
                      IconButton(
                        tooltip: 'Desk tools',
                        iconSize: EditorMetrics.s18,
                        onPressed: () => c.deskDrawer.value = 'Tools',
                        icon: const Icon(Icons.all_inbox_outlined),
                      ),
                      if (live) Icon(spec!.icon, size: EditorMetrics.s16),
                      const SizedBox(width: EditorMetrics.s6),
                      Text(
                        live ? shown : 'Tools',
                        style: const TextStyle(fontSize: EditorMetrics.font),
                      ),
                    ],
                  ),
                ),
                Expanded(child: live ? widget.panelBuilder(shown) : _catalog()),
              ],
            ),
          ),
        ),
      );
    },
  );
}
