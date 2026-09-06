import 'dart:async';

import 'package:flutter/material.dart';

import '../session/editor_session.dart';
import '../foundation/theme.dart';
import '../foundation/panel_catalog.dart';
import '../workspace/layout.dart';
import '../workspace/panel_ids.dart';
import '../workspace/workspace_view.dart';
import '../input/editor_shortcuts.dart';
import '../panels/registry.dart';
import '../panels/panel_settings.dart';
import '../panels/composition_controls.dart';
import '../panels/export_controls.dart';
import '../foundation/metrics.dart';

class EditorWindow extends StatefulWidget {
  const EditorWindow({super.key});
  @override
  State<EditorWindow> createState() => _EditorWindowState();
}

class _EditorWindowState extends State<EditorWindow> {
  final c = EditorSession();
  late final shortcuts = EditorShortcuts(
    c,
    onMenu: menu,
    hasSheet: () => sheet != null,
    closeSheet: () => setState(() => sheet = null),
    showComposition: () => setState(() => sheet = 'Composition'),
    showInspector: () => c.placePanel('Inspector', 'show'),
  );

  final workspace = WorkspaceLayout();
  DockNode get dock => workspace.root;
  set dock(DockNode value) => workspace.root = value;
  final paneKeys = {for (final id in paneNames) id: GlobalKey()};
  Timer? saveTimer;
  double uiScale = 1, dim = .75;
  String? sheet;
  bool ready = false;
  final detached = <String, List<String>>{};
  final hiddenPanels = <String>{};
  @override
  void initState() {
    super.initState();
    c.deskDefault.addListener(persist);
    c.confirmClose = confirmReplacement;
    c.panelPlacementRequested = _placePanel;
    c.windowClosed = (info) {
      final panes = (info['panels'] as List? ?? []).whereType<String>();
      if (mounted)
        setState(() {
          detached.remove('${info['id']}');
          if (panes.contains('Desk')) showPane('Desk');
          _publishPlaces();
          persist();
        });
    };
    c.filesDropped = (paths) {
      if (paths.isNotEmpty) c.command('import', {'paths': paths});
    };
    _initialize();
  }

  Future<void> _initialize() async {
    await c.initialize();
    if (!mounted) return;
    Map<String, dynamic>? restored;
    if (c.windowInfo['main'] == false) {
      final settings = EditorSession.map(await c.native('readSettings'));
      c.deskWork.value = EditorSession.map(settings['deskWork']);
      dock = DockNode.leaf(
        'detached',
        (c.windowInfo['panels'] as List? ?? []).whereType<String>().toList(),
      );
    } else {
      try {
        final data = EditorSession.map(await c.native('readSettings'));
        if (data['dock'] is Map)
          dock = DockNode.read(EditorSession.map(data['dock']));
        restored = data['panelPlacements'] is Map
            ? EditorSession.map(data['panelPlacements'])
            : null;
        c.deskWork.value = EditorSession.map(data['deskWork']);
        c.deskDefault.value = data['deskDefault'] as String? ?? 'Tools';
        if (restored != null) {
          hiddenPanels.addAll(
            restored.entries
                .where((e) => e.value == 'hidden')
                .map((e) => e.key),
          );
          for (final entry in restored.entries) {
            if (entry.value == 'drawer' && panelSpec(entry.key)?.drawer != true)
              workspace.show(entry.key);
          }
        }
        uiScale = (data['scale'] as num? ?? 1).toDouble();
      } catch (_) {}
    }
    setState(() => ready = true);
    if (c.windowInfo['main'] != false) {
      await _publishPlaces();
      final windows = restored == null
          ? ['Notes']
          : restored.entries
                .where((e) => e.value == 'window')
                .map((e) => e.key)
                .toList();
      for (final name in windows) {
        await _placePanel(name, 'window');
      }
    }
  }

  @override
  void dispose() {
    saveTimer?.cancel();
    c.dispose();
    super.dispose();
  }

  void persist() {
    if (c.windowInfo['main'] == false) return;
    saveTimer?.cancel();
    saveTimer = Timer(const Duration(milliseconds: 300), () {
      c.native('writeSettings', {
        'dock': dock.json(),
        'scale': uiScale,
        'dim': dim,
        'deskDefault': c.deskDefault.value,
        'deskWork': c.deskWork.value,
        'panelPlacements': c.panePlaces.value,
      });
    });
  }

  void showPane(String name) => workspace.show(name);
  void closePane(String name) {
    if (name != 'Desk') {
      c.placePanel(name, panelSpec(name)?.drawer == true ? 'drawer' : 'hidden');
      return;
    }
    if (c.windowInfo['main'] == false) {
      c.native('closeWindow', {'id': c.windowInfo['id']});
      return;
    }
    setState(() => workspace.close(name));
    _publishPlaces();
    persist();
  }

  void movePane(String name, DockNode target, String edge) {
    setState(() => workspace.move(name, target, edge));
    _publishPlaces();
    persist();
  }

  Future<void> _publishPlaces() async {
    if (c.windowInfo['main'] == false) return;
    final places = {
      for (final spec in panelCatalog)
        spec.name: spec.drawer && !hiddenPanels.contains(spec.name)
            ? 'drawer'
            : 'hidden',
    };
    for (final name in dock.leaves.expand((n) => n.tabs)) {
      if (name != 'Desk') places[name] = 'tab';
    }
    for (final name in detached.values.expand((v) => v)) {
      if (name != 'Desk') places[name] = 'window';
    }
    c.panePlaces.value = places;
    await c.native('setPaneState', {
      'places': places,
      'drawer': c.deskDrawer.value,
    });
  }

  Future<void> _revealDesk() async {
    final host = detached.entries
        .where((e) => e.value.contains('Desk'))
        .firstOrNull;
    if (host != null) {
      await c.native('focusWindow', {'id': host.key});
    } else {
      setState(() => workspace.show('Desk'));
    }
  }

  Future<void> _placePanel(String name, String placement) async {
    if (!paneNames.contains(name)) return;
    if (placement == 'drawer' && panelSpec(name)?.drawer != true) return;
    if (placement == 'hidden') {
      hiddenPanels.add(name);
    } else {
      hiddenPanels.remove(name);
    }
    final existing = detached.entries
        .where((e) => e.value.contains(name))
        .firstOrNull;
    if (placement == 'window' && existing != null) {
      await c.native('focusWindow', {'id': existing.key});
      return;
    }
    if (placement == 'tab' &&
        existing == null &&
        dock.leaves.any((n) => n.tabs.contains(name))) {
      setState(() => workspace.show(name));
      return;
    }
    if (placement == 'show') {
      if (existing != null) {
        await c.native('focusWindow', {'id': existing.key});
        return;
      }
      if (dock.leaves.any((n) => n.tabs.contains(name))) {
        setState(() => workspace.show(name));
      } else if (panelSpec(name)?.drawer == true) {
        c.deskDrawer.value = name;
        await _revealDesk();
      } else {
        setState(() => workspace.show(name));
      }
      await _publishPlaces();
      persist();
      return;
    }
    await c.native('flushEditors');
    if (existing != null) {
      detached.remove(existing.key);
      await c.native('closeWindow', {'id': existing.key});
    }
    setState(() => workspace.close(name));
    if (placement == 'drawer') {
      await _revealDesk();
    }
    if (placement == 'tab') {
      setState(() => workspace.show(name));
    }
    if (placement == 'window') {
      await _detachPanel(name);
    }
    await _publishPlaces();
    persist();
  }

  Future<void> detachPane(String name) => c.placePanel(name, 'window');

  Future<void> _detachPanel(String name) async {
    try {
      final info = EditorSession.map(
        await c.native('openPanelWindow', {
          'panels': [name],
        }),
      );
      if (!mounted) return;
      detached['${info['id']}'] = [name];
      setState(() => workspace.close(name));
    } catch (e) {
      c.error.value = '$e';
    }
  }

  Future<bool> confirmReplacement() async {
    c.stopPlayback();
    if (c.state['dirty'] != true) return true;
    if (!mounted) return false;
    final result = await showDialog<String>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Save changes?'),
        content: const Text('Save the current document before closing it.'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, 'cancel'),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () => Navigator.pop(context, 'discard'),
            child: const Text("Don't Save"),
          ),
          TextButton(
            onPressed: () => Navigator.pop(context, 'save'),
            child: const Text('Save'),
          ),
        ],
      ),
    );
    if (result == 'save') {
      await c.save();
      return c.state['dirty'] != true;
    }
    return result == 'discard';
  }

  Future<void> menu(String action) async {
    setState(() => sheet = null);
    switch (action) {
      case 'New':
        if (await confirmReplacement()) await c.command('new');
        break;
      case 'Open':
        if (await confirmReplacement()) await c.chooseOpen();
        break;
      case 'Save':
        await c.save();
        break;
      case 'Save as':
        await c.save(as: true);
        break;
      case 'Import':
        await c.importFiles();
        break;
      case 'Reset layout':
        setState(() {
          dock = initialDock();
          for (final name in detached.values.expand((v) => v)) {
            workspace.close(name);
          }
        });
        _publishPlaces();
        persist();
        break;
      default:
        if (paneNames.contains(action)) {
          await c.placePanel(action, 'show');
        } else {
          final commands = {
            'Undo': 'undo',
            'Redo': 'redo',
            'Cut': 'cut',
            'Copy': 'copy',
            'Paste': 'paste',
            'Duplicate': 'duplicate',
            'Delete': 'delete',
            'Group': 'group',
            'Ungroup': 'ungroup',
            'Split': 'split',
          };
          if (commands.containsKey(action)) await c.command(commands[action]!);
        }
    }
  }

  Widget topMenu(String label, List<String> items) => PopupMenuButton<String>(
    tooltip: label,
    onSelected: menu,
    itemBuilder: (_) => [
      for (final item in items)
        PopupMenuItem(
          value: item,
          height: EditorMetrics.section,
          child: Text(
            item,
            style: const TextStyle(fontSize: EditorMetrics.font),
          ),
        ),
    ],
    child: Padding(
      padding: const EdgeInsets.symmetric(
        horizontal: EditorMetrics.s7,
        vertical: EditorMetrics.s4,
      ),
      child: Text(label),
    ),
  );
  Widget pane(String name) =>
      buildPanel(name, c, paneKeys.putIfAbsent(name, () => GlobalKey()));
  @override
  Widget build(BuildContext context) => Focus(
    autofocus: true,
    onKeyEvent: shortcuts.handle,
    child: Scaffold(
      body: DefaultTextStyle(
        style: const TextStyle(
          fontSize: EditorMetrics.font,
          color: EditorTheme.ink,
        ),
        child: LayoutBuilder(
          builder: (context, box) => Transform.scale(
            scale: uiScale,
            alignment: Alignment.topLeft,
            child: SizedBox(
              width: box.maxWidth / uiScale,
              height: box.maxHeight / uiScale,
              child: Stack(
                children: [
                  Column(
                    children: [
                      if (c.windowInfo['main'] != false)
                        Container(
                          height: EditorMetrics.control,
                          color: EditorTheme.app,
                          child: Row(
                            children: [
                              topMenu('File', [
                                'New',
                                'Open',
                                'Save',
                                'Save as',
                                'Import',
                              ]),
                              topMenu('Edit', [
                                'Undo',
                                'Redo',
                                'Cut',
                                'Copy',
                                'Paste',
                                'Duplicate',
                                'Split',
                                'Delete',
                                'Group',
                                'Ungroup',
                              ]),
                              topMenu('View', [...paneNames, 'Reset layout']),
                              for (final name in [
                                'Composition',
                                'Export',
                                'Settings',
                              ])
                                EditorButton(
                                  name,
                                  () => setState(
                                    () => sheet = sheet == name ? null : name,
                                  ),
                                ),
                              const Spacer(),
                              ValueListenableBuilder<Map<String, dynamic>>(
                                valueListenable: c.document,
                                builder: (_, state, __) => Text(
                                  '${state['dirty'] == true ? '• ' : ''}${(state['path'] as String? ?? 'Untitled').split('/').last}  ',
                                ),
                              ),
                            ],
                          ),
                        ),
                      Expanded(
                        child: ready
                            ? WorkspaceView(
                                layout: dock,
                                panelBuilder: pane,
                                onMove: movePane,
                                onClose: closePane,
                                onDetach: detachPane,
                                onLayoutChanged: persist,
                              )
                            : const Center(child: CircularProgressIndicator()),
                      ),
                      ValueListenableBuilder<String?>(
                        valueListenable: c.error,
                        builder: (_, message, __) => Container(
                          height: EditorMetrics.row,
                          alignment: Alignment.centerLeft,
                          padding: const EdgeInsets.symmetric(
                            horizontal: EditorMetrics.s6,
                          ),
                          color: EditorTheme.app,
                          child: Text(
                            message ?? '',
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                          ),
                        ),
                      ),
                    ],
                  ),
                  if (sheet != null) ...[
                    Positioned.fill(
                      child: GestureDetector(
                        behavior: HitTestBehavior.opaque,
                        onTap: () => setState(() => sheet = null),
                      ),
                    ),
                    Positioned(
                      left: sheet == 'Composition'
                          ? EditorMetrics.s85
                          : sheet == 'Export'
                          ? EditorMetrics.s155
                          : EditorMetrics.s200,
                      top: EditorMetrics.control,
                      child: Material(
                        color: EditorTheme.panel,
                        elevation: 4,
                        child: Container(
                          width: sheet == 'Settings'
                              ? EditorMetrics.sheetWide
                              : EditorMetrics.sheet,
                          decoration: BoxDecoration(
                            border: Border.all(color: EditorTheme.border),
                          ),
                          child: sheet == 'Composition'
                              ? CompositionControls(controller: c)
                              : sheet == 'Export'
                              ? ExportControls(controller: c)
                              : Column(
                                  mainAxisSize: MainAxisSize.min,
                                  children: [
                                    SizedBox(
                                      height: EditorMetrics.sheet,
                                      child: PanelSettings(controller: c),
                                    ),
                                    const EditorSection(
                                      'View',
                                      SizedBox.shrink(),
                                    ),
                                    Row(
                                      children: [
                                        const SizedBox(width: EditorMetrics.s8),
                                        const Expanded(
                                          child: Text('Outside dim'),
                                        ),
                                        EditorButton(
                                          '−',
                                          () => setState(
                                            () => dim = (dim - .05).clamp(0, 1),
                                          ),
                                        ),
                                        Text('${(dim * 100).round()}%'),
                                        EditorButton(
                                          '+',
                                          () => setState(
                                            () => dim = (dim + .05).clamp(0, 1),
                                          ),
                                        ),
                                      ],
                                    ),
                                    Row(
                                      children: [
                                        const SizedBox(width: EditorMetrics.s8),
                                        const Expanded(child: Text('Scale')),
                                        EditorButton('−', () {
                                          setState(
                                            () => uiScale = (uiScale - .05)
                                                .clamp(.5, 2),
                                          );
                                          persist();
                                        }),
                                        Text('${(uiScale * 100).round()}%'),
                                        EditorButton('+', () {
                                          setState(
                                            () => uiScale = (uiScale + .05)
                                                .clamp(.5, 2),
                                          );
                                          persist();
                                        }),
                                      ],
                                    ),
                                  ],
                                ),
                        ),
                      ),
                    ),
                  ],
                ],
              ),
            ),
          ),
        ),
      ),
    ),
  );
}
