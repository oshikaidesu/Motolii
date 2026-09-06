import 'dart:convert';
import 'dart:io';
import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../session/editor_session.dart';
import '../foundation/theme.dart';
import '../foundation/metrics.dart';

class BrowserPanel extends StatefulWidget {
  const BrowserPanel({
    super.key,
    required this.controller,
    this.fixedTab,
    this.showTabs = true,
  });
  final String? fixedTab;
  final bool showTabs;
  final EditorSession controller;
  @override
  State<BrowserPanel> createState() => _BrowserPanelState();
}

class _BrowserPanelState extends State<BrowserPanel> {
  static const tabs = ['Create', 'Media', 'Effects', 'Colors'];
  String tab = 'Create';
  final classifications = <String, String>{};
  final search = TextEditingController();
  final searchFocus = FocusNode();
  final panelFocus = FocusNode();
  final queries = <String, String>{};
  final active = <String, String>{};
  final selected = <String, Set<String>>{};
  final scroll = ScrollController();
  List<Map<String, dynamic>> visible = [];
  int columns = 1;

  bool has(String op) =>
      (widget.controller.state['capabilities'] as List? ?? []).contains(op);
  List<Map<String, dynamic>> rows(dynamic value) => (value as List? ?? [])
      .whereType<Map>()
      .map((e) => Map<String, dynamic>.from(e))
      .toList();
  String id(Map<String, dynamic> item) => '${item['id'] ?? item['hex']}';

  @override
  void initState() {
    super.initState();
    tab = widget.fixedTab ?? 'Create';
  }

  @override
  void dispose() {
    search.dispose();
    searchFocus.dispose();
    panelFocus.dispose();
    scroll.dispose();
    super.dispose();
  }

  void changeTab(String value) {
    setState(() {
      queries[tab] = search.text;
      tab = value;
      search.text = queries[value] ?? '';
    });
  }

  String family(Map<String, dynamic> item) {
    final mime = '${item['mime'] ?? ''}'.toLowerCase();
    final path = '${item['path'] ?? ''}'.toLowerCase();
    if (mime.contains('video') ||
        RegExp(r'\.(mp4|mov|mkv|webm)$').hasMatch(path))
      return 'Video';
    if (mime.contains('audio') ||
        RegExp(r'\.(wav|mp3|flac|aac)$').hasMatch(path))
      return 'Audio';
    if (RegExp(r'\.(obj|glb|gltf|ply)$').hasMatch(path)) return '3D';
    return '2D';
  }

  String classification(Map<String, dynamic> item) {
    switch (tab) {
      case 'Create':
        return switch (id(item)) {
          'text' => 'Text',
          'rectangle' => 'Shapes',
          'bezier' => 'Paths',
          'cube' => '3D',
          'camera' => '3D',
          _ => 'Other',
        };
      case 'Media':
        final kind = family(item);
        return kind == '2D' ? 'Images' : kind;
      case 'Effects':
        return const <String, String>{
              'motolii.blur': 'Blur',
              'motolii.isf_bloom': 'Light',
              'motolii.glow': 'Light',
              'motolii.gain': 'Color',
              'motolii.gradient': 'Color',
              'motolii.tri_led': 'Stylize',
            }[id(item)] ??
            'Other';
      default:
        return item['used'] == true ? 'Used here' : 'Starter';
    }
  }

  List<Map<String, dynamic>> items(Map<String, dynamic> state) {
    switch (tab) {
      case 'Create':
        return [
          {
            'id': 'text',
            'name': 'Text',
            'detail': 'Adds a text layer',
            'glyph': 'T',
          },
          {
            'id': 'rectangle',
            'name': 'Rectangle',
            'detail': 'Adds a shape layer',
            'glyph': '■',
          },
          {'id': 'camera', 'name': 'Camera', 'detail': 'Adds a camera layer'},
          {
            'id': 'cube',
            'name': 'Cube',
            'detail': 'Adds a 3D cube',
            'glyph': '⬡',
          },
          {
            'id': 'bezier',
            'name': 'Bezier',
            'detail': 'Adds a path layer',
            'glyph': '〜',
          },
        ];
      case 'Media':
        return rows(state['assets']);
      case 'Effects':
        return rows(state['catalog']);
      default:
        return rows(state['palette']);
    }
  }

  void select(Map<String, dynamic> item) {
    panelFocus.requestFocus();
    final key = id(item);
    final multi = tab == 'Media' || tab == 'Effects';
    final modifiers = HardwareKeyboard.instance;
    setState(() {
      final chosen = selected.putIfAbsent(tab, () => <String>{});
      if (multi && modifiers.isShiftPressed && active[tab] != null) {
        final from = visible.indexWhere((e) => id(e) == active[tab]);
        final to = visible.indexWhere((e) => id(e) == key);
        if (from >= 0 && to >= 0) {
          if (!modifiers.isMetaPressed && !modifiers.isControlPressed)
            chosen.clear();
          chosen.addAll(
            visible.sublist(math.min(from, to), math.max(from, to) + 1).map(id),
          );
        }
      } else if (multi &&
          (modifiers.isMetaPressed || modifiers.isControlPressed)) {
        chosen.contains(key) ? chosen.remove(key) : chosen.add(key);
      } else {
        chosen
          ..clear()
          ..add(key);
      }
      active[tab] = key;
    });
  }

  Future<void> apply(Map<String, dynamic> item) async {
    final c = widget.controller;
    switch (tab) {
      case 'Create':
        if (has('create')) await c.command('create', {'kind': id(item)});
        break;
      case 'Media':
        if (has('placeAsset') && item['missing'] != true)
          await c.command('placeAsset', {'id': item['id']});
        break;
      case 'Effects':
        if (has('applyEffect')) {
          final chosen = visible
              .where((row) => selected[tab]?.contains(id(row)) ?? false)
              .map((row) => row['id'])
              .toList();
          await c.command('applyEffect', {
            'pluginIds': chosen.isEmpty ? [item['id']] : chosen,
          });
        }
        break;
      case 'Colors':
        if (has('applyPalette')) {
          await c.command('applyPalette', {'rgba': _rgba(item['rgba'])});
        }
        break;
    }
  }

  KeyEventResult key(FocusNode node, KeyEvent event) {
    if (event is! KeyDownEvent || searchFocus.hasFocus)
      return KeyEventResult.ignored;
    final k = event.logicalKey;
    final primary =
        HardwareKeyboard.instance.isMetaPressed ||
        HardwareKeyboard.instance.isControlPressed;
    if (primary && k == LogicalKeyboardKey.keyF) {
      searchFocus.requestFocus();
      search.selection = TextSelection(
        baseOffset: 0,
        extentOffset: search.text.length,
      );
      return KeyEventResult.handled;
    }
    if (k == LogicalKeyboardKey.escape) {
      setState(() {
        if (search.text.isNotEmpty) {
          search.clear();
        } else {
          selected[tab]?.clear();
          active.remove(tab);
        }
      });
      return KeyEventResult.handled;
    }
    if (primary &&
        k == LogicalKeyboardKey.keyA &&
        (tab == 'Media' || tab == 'Effects')) {
      setState(() => selected[tab] = visible.map(id).toSet());
      return KeyEventResult.handled;
    }
    if (visible.isEmpty) return KeyEventResult.ignored;
    final current = visible.indexWhere((e) => id(e) == active[tab]);
    if (k == LogicalKeyboardKey.enter || k == LogicalKeyboardKey.numpadEnter) {
      apply(visible[current < 0 ? 0 : current]);
      return KeyEventResult.handled;
    }
    int? next;
    if (k == LogicalKeyboardKey.arrowLeft) next = current - 1;
    if (k == LogicalKeyboardKey.arrowRight) next = current + 1;
    if (k == LogicalKeyboardKey.arrowUp) next = current - columns;
    if (k == LogicalKeyboardKey.arrowDown) next = current + columns;
    if (k == LogicalKeyboardKey.home) next = 0;
    if (k == LogicalKeyboardKey.end) next = visible.length - 1;
    if (next != null) {
      select(visible[next.clamp(0, visible.length - 1)]);
      return KeyEventResult.handled;
    }
    if ((k == LogicalKeyboardKey.delete || k == LogicalKeyboardKey.backspace) &&
        tab == 'Media') {
      final item = visible[current < 0 ? 0 : current];
      if (has('removeAsset') && item['used'] != true)
        widget.controller.command('removeAsset', {'id': item['id']});
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  @override
  Widget build(
    BuildContext context,
  ) => ValueListenableBuilder<Map<String, dynamic>>(
    valueListenable: widget.controller.document,
    builder: (context, state, _) {
      final all = items(state);
      final rails = switch (tab) {
        'Create' => ['All', 'Text', 'Shapes', '3D', 'Paths'],
        'Media' => ['All', 'Video', 'Images', 'Audio', '3D'],
        'Effects' => ['All', 'Blur', 'Light', 'Color', 'Stylize', 'Other'],
        _ => ['All', 'Used here', 'Starter'],
      };
      final chosen = classifications[tab] ?? 'All';
      visible = all.where((item) {
        final label = '${item['name'] ?? item['hex'] ?? item['id']}'
            .toLowerCase();
        return label.contains(search.text.trim().toLowerCase()) &&
            (chosen == 'All' || classification(item) == chosen);
      }).toList();
      final target = _colorTarget(widget.controller);
      return Focus(
        focusNode: panelFocus,
        onKeyEvent: key,
        child: ColoredBox(
          color: EditorTheme.panel,
          child: Column(
            children: [
              if (widget.showTabs)
                SizedBox(
                  height: EditorMetrics.section,
                  child: Row(
                    children: [
                      for (final value in tabs)
                        Expanded(
                          child: _smallButton(
                            value,
                            () => changeTab(value),
                            selected: value == tab,
                          ),
                        ),
                    ],
                  ),
                ),
              Container(
                height: EditorMetrics.bar,
                padding: const EdgeInsets.symmetric(
                  horizontal: EditorMetrics.s6,
                  vertical: EditorMetrics.s3,
                ),
                decoration: const BoxDecoration(
                  border: Border(bottom: BorderSide(color: EditorTheme.line)),
                ),
                child: Row(
                  children: [
                    Expanded(
                      child: TextField(
                        controller: search,
                        focusNode: searchFocus,
                        style: const TextStyle(
                          fontSize: EditorMetrics.font,
                          color: EditorTheme.ink,
                        ),
                        decoration: InputDecoration(
                          isDense: true,
                          contentPadding: const EdgeInsets.symmetric(
                            horizontal: EditorMetrics.s4,
                            vertical: EditorMetrics.s3,
                          ),
                          hintText: 'Search $tab',
                          hintStyle: const TextStyle(
                            fontSize: EditorMetrics.font,
                            color: EditorTheme.muted,
                          ),
                          border: InputBorder.none,
                        ),
                        onChanged: (_) => setState(() {}),
                      ),
                    ),
                    if (tab == 'Media')
                      _smallButton(
                        'Import',
                        has('import')
                            ? () async {
                                final paths = await widget.controller.native(
                                  'pickImport',
                                );
                                if (paths is List && paths.isNotEmpty)
                                  await widget.controller.command('import', {
                                    'paths': paths,
                                  });
                              }
                            : null,
                      ),
                  ],
                ),
              ),
              Expanded(
                child: Row(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    Container(
                      width: EditorMetrics.s96,
                      decoration: const BoxDecoration(
                        border: Border(
                          right: BorderSide(color: EditorTheme.line),
                        ),
                      ),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.stretch,
                        children: [
                          Padding(
                            padding: const EdgeInsets.fromLTRB(
                              EditorMetrics.s7,
                              EditorMetrics.s7,
                              EditorMetrics.s4,
                              EditorMetrics.s3,
                            ),
                            child: Text(
                              tab.toUpperCase(),
                              style: const TextStyle(
                                fontSize: EditorMetrics.micro,
                                color: EditorTheme.muted,
                              ),
                            ),
                          ),
                          for (final rail in rails)
                            _smallButton(rail, () {
                              setState(() => classifications[tab] = rail);
                            }, selected: chosen == rail),
                        ],
                      ),
                    ),
                    Expanded(
                      child: LayoutBuilder(
                        builder: (context, constraints) {
                          columns = math.max(
                            1,
                            (constraints.maxWidth / (tab == 'Colors' ? 52 : 88))
                                .floor(),
                          );
                          return Column(
                            crossAxisAlignment: CrossAxisAlignment.stretch,
                            children: [
                              if (tab == 'Colors' && target != null)
                                Padding(
                                  padding: const EdgeInsets.symmetric(
                                    horizontal: EditorMetrics.s6,
                                    vertical: EditorMetrics.s3,
                                  ),
                                  child: Text(
                                    '${target['label'] ?? 'Layer color'}',
                                    maxLines: 1,
                                    overflow: TextOverflow.ellipsis,
                                    style: const TextStyle(
                                      fontSize: EditorMetrics.dense,
                                      color: EditorTheme.muted,
                                    ),
                                  ),
                                ),
                              if (tab == 'Colors')
                                _ColorPicker(
                                  controller: widget.controller,
                                  target: target,
                                  enabled: has('setColor'),
                                ),
                              Expanded(
                                child: visible.isEmpty
                                    ? const Padding(
                                        padding: EdgeInsets.all(
                                          EditorMetrics.s8,
                                        ),
                                        child: Text(
                                          'No matches',
                                          style: TextStyle(
                                            color: EditorTheme.muted,
                                            fontSize: EditorMetrics.dense,
                                          ),
                                        ),
                                      )
                                    : GridView.builder(
                                        controller: scroll,
                                        padding: EdgeInsets.all(
                                          tab == 'Colors'
                                              ? EditorMetrics.s6
                                              : 1,
                                        ),
                                        gridDelegate:
                                            SliverGridDelegateWithFixedCrossAxisCount(
                                              crossAxisCount: columns,
                                              mainAxisExtent: tab == 'Colors'
                                                  ? 48
                                                  : (constraints.maxWidth /
                                                                    columns -
                                                                2) *
                                                            9 /
                                                            16 +
                                                        (tab == 'Media'
                                                            ? 46
                                                            : 28),
                                              crossAxisSpacing: tab == 'Colors'
                                                  ? EditorMetrics.s4
                                                  : 1,
                                              mainAxisSpacing: tab == 'Colors'
                                                  ? EditorMetrics.s4
                                                  : 1,
                                            ),
                                        itemCount: visible.length,
                                        itemBuilder: (context, index) =>
                                            tab == 'Colors'
                                            ? Align(
                                                alignment: Alignment.topLeft,
                                                child: SizedBox(
                                                  width: double.infinity,
                                                  height: EditorMetrics.s48,
                                                  child: card(visible[index]),
                                                ),
                                              )
                                            : card(visible[index]),
                                      ),
                              ),
                            ],
                          );
                        },
                      ),
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),
      );
    },
  );

  Widget card(Map<String, dynamic> item) {
    final supported = switch (tab) {
      'Create' => has('create'),
      'Media' => has('placeAsset') && item['missing'] != true,
      'Effects' =>
        has('applyEffect') && widget.controller.selectedIds.isNotEmpty,
      _ => has('applyPalette') && widget.controller.selectedIds.isNotEmpty,
    };
    final isSelected = selected[tab]?.contains(id(item)) ?? false;
    final isColor = tab == 'Colors';
    final identityColor = EditorTheme.kindColor(
      tab == 'Media' ? family(item) : id(item),
    );
    final missing = item['missing'] == true;
    return Tooltip(
      message: supported
          ? 'Select · double-click or Enter to apply'
          : 'Apply unavailable',
      child: GestureDetector(
        onTap: () => select(item),
        onDoubleTap: () {
          select(item);
          apply(item);
        },
        child: Container(
          decoration: BoxDecoration(
            color: EditorTheme.panel,
            border: Border.all(
              color: isSelected ? EditorTheme.accent : EditorTheme.line,
            ),
          ),
          child: isColor
              ? Stack(
                  fit: StackFit.expand,
                  children: [
                    ColoredBox(color: _color(_rgba(item['rgba']))),
                    Align(
                      alignment: Alignment.bottomCenter,
                      child: Container(
                        color: Colors.black.withValues(alpha: .62),
                        width: double.infinity,
                        padding: const EdgeInsets.all(EditorMetrics.s2),
                        child: Text(
                          '${item['hex'] ?? ''}',
                          textAlign: TextAlign.center,
                          style: const TextStyle(
                            color: EditorTheme.ink,
                            fontSize: EditorMetrics.micro,
                          ),
                        ),
                      ),
                    ),
                  ],
                )
              : Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    Container(height: EditorMetrics.s3, color: identityColor),
                    AspectRatio(
                      aspectRatio: 16 / 9,
                      child: tab == 'Media'
                          ? _thumbnail(item)
                          : ColoredBox(
                              color: const Color(0xff222222),
                              child: Center(
                                child: id(item) == 'camera'
                                    ? Icon(
                                        Icons.videocam_outlined,
                                        size: EditorMetrics.bar,
                                        color: identityColor,
                                      )
                                    : id(item) == 'cube'
                                    ? Icon(
                                        Icons.view_in_ar,
                                        size: EditorMetrics.bar,
                                        color: identityColor,
                                      )
                                    : Text(
                                        '${item['glyph'] ?? 'ƒ'}',
                                        style: TextStyle(
                                          fontSize: EditorMetrics.s23,
                                          color: identityColor,
                                        ),
                                      ),
                              ),
                            ),
                    ),
                    Container(
                      height: EditorMetrics.row,
                      alignment: Alignment.centerLeft,
                      padding: const EdgeInsets.symmetric(
                        horizontal: EditorMetrics.s5,
                      ),
                      child: Text(
                        '${item['name'] ?? item['id']}',
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: const TextStyle(
                          color: EditorTheme.ink,
                          fontSize: EditorMetrics.dense,
                          fontWeight: FontWeight.w600,
                        ),
                      ),
                    ),
                    if (tab == 'Media')
                      Padding(
                        padding: const EdgeInsets.symmetric(
                          horizontal: EditorMetrics.s5,
                        ),
                        child: Text(
                          '${item['detail'] ?? (tab == 'Media' ? '${family(item)}${missing
                                        ? ' · missing'
                                        : item['used'] == true
                                        ? ' · in use'
                                        : ''}' : 'Effect')}',
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                          style: const TextStyle(
                            color: EditorTheme.muted,
                            fontSize: EditorMetrics.micro,
                          ),
                        ),
                      ),
                    if (tab == 'Media')
                      Expanded(
                        child: Row(
                          children: [
                            if (widget.controller.selectedIds.isNotEmpty)
                              Expanded(
                                child: _smallButton(
                                  'Replace',
                                  has('replaceAsset') && !missing
                                      ? () => widget.controller.command(
                                          'replaceAsset',
                                          {'id': item['id']},
                                        )
                                      : null,
                                ),
                              ),
                            if (item['path'] != null && !missing)
                              _smallButton(
                                'Finder',
                                () => widget.controller.native('reveal', {
                                  'path': item['path'],
                                }),
                              ),
                            if (item['used'] != true)
                              _smallButton(
                                '×',
                                has('removeAsset')
                                    ? () => widget.controller.command(
                                        'removeAsset',
                                        {'id': item['id']},
                                      )
                                    : null,
                              ),
                          ],
                        ),
                      ),
                  ],
                ),
        ),
      ),
    );
  }

  Widget _thumbnail(Map<String, dynamic> item) {
    final path = item['thumbnail'] as String?;
    final fallback = ColoredBox(
      color: EditorTheme.raised,
      child: Center(
        child: Icon(
          item['missing'] == true
              ? Icons.broken_image_outlined
              : family(item) == 'Audio'
              ? Icons.audiotrack
              : Icons.image_outlined,
          size: EditorMetrics.s19,
          color: EditorTheme.muted,
        ),
      ),
    );
    if (path == null || path.isEmpty) return fallback;
    try {
      if (path.startsWith('data:'))
        return Image.memory(
          Uri.parse(path).data!.contentAsBytes(),
          fit: BoxFit.cover,
          gaplessPlayback: true,
          errorBuilder: (_, _, _) => fallback,
        );
      return Image.file(
        File(path.startsWith('file:') ? Uri.parse(path).toFilePath() : path),
        fit: BoxFit.cover,
        gaplessPlayback: true,
        errorBuilder: (_, _, _) => fallback,
      );
    } catch (_) {
      return fallback;
    }
  }
}

Widget _smallButton(
  String label,
  VoidCallback? press, {
  bool selected = false,
}) => Tooltip(
  message: press == null ? '$label · unavailable' : label,
  child: InkWell(
    onTap: press,
    child: Container(
      height: EditorMetrics.row,
      alignment: Alignment.centerLeft,
      padding: const EdgeInsets.symmetric(horizontal: EditorMetrics.s6),
      color: selected ? EditorTheme.raised : Colors.transparent,
      child: Text(
        label,
        maxLines: 1,
        overflow: TextOverflow.ellipsis,
        style: TextStyle(
          fontSize: EditorMetrics.dense,
          color: press == null
              ? EditorTheme.muted.withValues(alpha: .45)
              : selected
              ? EditorTheme.accent
              : EditorTheme.ink,
        ),
      ),
    ),
  ),
);

List<double> _rgba(dynamic raw) {
  final values = (raw as List? ?? [1, 0, 0, 1])
      .map((v) => (v as num).toDouble())
      .toList();
  if (values.length < 4) values.add(1);
  final scale = values.any((v) => v > 1) ? 255.0 : 1.0;
  return values.take(4).map((v) => (v / scale).clamp(0.0, 1.0)).toList();
}

Color _color(List<double> rgba) =>
    Color.from(alpha: rgba[3], red: rgba[0], green: rgba[1], blue: rgba[2]);
Map<String, dynamic>? _colorTarget(EditorSession controller) {
  if (controller.state['colorTarget'] is Map)
    return Map<String, dynamic>.from(controller.state['colorTarget']);
  final layer = controller.activeLayer;
  final colors = layer?['colors'] as List? ?? [];
  if (layer == null || colors.isEmpty) return null;
  return {
    ...Map<String, dynamic>.from(colors.first as Map),
    'layer': layer['id'],
  };
}

class _ColorPicker extends StatefulWidget {
  const _ColorPicker({
    required this.controller,
    required this.target,
    required this.enabled,
  });
  final EditorSession controller;
  final Map<String, dynamic>? target;
  final bool enabled;
  @override
  State<_ColorPicker> createState() => _ColorPickerState();
}

class _ColorPickerState extends State<_ColorPicker> {
  List<double> unbound = [1, 0, 0, 1];
  List<double>? draft;
  String? dragPart;
  final hex = TextEditingController();
  final hexFocus = FocusNode();
  final pickerFocus = FocusNode();
  Future<void>? previewFlight;
  Map<String, dynamic>? queuedPreview;
  bool previewUsed = false;
  bool get canPreview =>
      (widget.controller.state['capabilities'] as List? ?? []).contains(
        'previewColor',
      );

  void preview() {
    if (widget.target == null ||
        draft == null ||
        !canPreview ||
        !widget.enabled)
      return;
    previewUsed = true;
    queuedPreview = {
      'layer': widget.target!['layer'],
      'slot': widget.target!['slot'],
      'rgba': List<double>.of(draft!),
    };
    previewFlight ??= pumpPreview();
  }

  Future<void> pumpPreview() async {
    try {
      while (queuedPreview != null) {
        final request = queuedPreview!;
        queuedPreview = null;
        await widget.controller.command('previewColor', request);
      }
    } finally {
      previewFlight = null;
    }
  }

  @override
  void dispose() {
    queuedPreview = null;
    if (previewUsed) widget.controller.cancelPreview();
    hex.dispose();
    hexFocus.dispose();
    pickerFocus.dispose();
    super.dispose();
  }

  @override
  void didUpdateWidget(covariant _ColorPicker oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (jsonEncode(oldWidget.target?['slot']) !=
            jsonEncode(widget.target?['slot']) ||
        oldWidget.target?['layer'] != widget.target?['layer']) {
      draft = null;
      dragPart = null;
      queuedPreview = null;
      if (previewUsed) {
        previewUsed = false;
        widget.controller.cancelPreview();
      }
    }
  }

  bool get supportsAlpha {
    if (widget.target == null) return true;
    for (final layer in widget.controller.layers) {
      if (layer['id'] == widget.target!['layer'])
        return layer['kind'] != 'Shape';
    }
    return false;
  }

  List<double> get value =>
      draft ??
      (widget.target == null ? unbound : _rgba(widget.target!['rgba']));
  void sample(Offset p, {bool alpha = false}) {
    pickerFocus.requestFocus();
    final v = value;
    final hsv = HSVColor.fromColor(_color(v));
    if (alpha) {
      setState(() => draft = [v[0], v[1], v[2], (p.dx / 128).clamp(0.0, 1.0)]);
      preview();
      return;
    }
    dragPart ??=
        (p.dx >= 25.6 && p.dx <= 102.4 && p.dy >= 25.6 && p.dy <= 102.4)
        ? 'sv'
        : 'hue';
    final updated = dragPart == 'sv'
        ? hsv
              .withSaturation(((p.dx - 25.6) / 76.8).clamp(0.0, 1.0))
              .withValue((1 - (p.dy - 25.6) / 76.8).clamp(0.0, 1.0))
        : hsv.withHue(
            (math.atan2(p.dy - 64, p.dx - 64) * 180 / math.pi + 90) % 360,
          );
    final c = updated.toColor();
    setState(() => draft = [c.r, c.g, c.b, v[3]]);
    preview();
  }

  Future<void> commit() async {
    final target = widget.target;
    final next = draft;
    draft = null;
    dragPart = null;
    final used = previewUsed;
    previewUsed = false;
    if (next == null) return;
    if (target == null) {
      setState(() => unbound = next);
      return;
    }
    await previewFlight;
    if (widget.enabled) {
      if (used) {
        await widget.controller.command('commitPreview');
      } else {
        await widget.controller.command('setColor', {
          'layer': target['layer'],
          'slot': target['slot'],
          'rgba': next,
        });
      }
    }
    if (mounted) setState(() {});
  }

  void cancel() {
    queuedPreview = null;
    if (previewUsed) {
      previewUsed = false;
      widget.controller.cancelPreview();
    }
    setState(() {
      draft = null;
      dragPart = null;
    });
  }

  @override
  Widget build(BuildContext context) {
    final v = value;
    final color = _color(v);
    final display =
        '#${(color.toARGB32() & 0xffffff).toRadixString(16).padLeft(6, '0')}';
    if (!hexFocus.hasFocus) hex.text = display;
    return CallbackShortcuts(
      bindings: {const SingleActivator(LogicalKeyboardKey.escape): cancel},
      child: Focus(
        focusNode: pickerFocus,
        child: Padding(
          padding: const EdgeInsets.all(EditorMetrics.s6),
          child: Column(
            children: [
              GestureDetector(
                onPanStart: (e) => sample(e.localPosition),
                onPanUpdate: (e) => sample(e.localPosition),
                onPanEnd: (_) => commit(),
                onPanCancel: cancel,
                onTapUp: (e) {
                  sample(e.localPosition);
                  commit();
                },
                child: SizedBox(
                  width: EditorMetrics.thumb,
                  height: EditorMetrics.thumb,
                  child: CustomPaint(painter: _WheelPainter(color)),
                ),
              ),
              SizedBox(
                width: EditorMetrics.thumb,
                height: EditorMetrics.s23,
                child: Row(
                  children: [
                    Container(
                      width: EditorMetrics.s12,
                      height: EditorMetrics.s12,
                      color: color,
                    ),
                    const SizedBox(width: EditorMetrics.s5),
                    Expanded(
                      child: TextField(
                        controller: hex,
                        focusNode: hexFocus,
                        readOnly: widget.target == null || !widget.enabled,
                        style: const TextStyle(
                          fontSize: EditorMetrics.font,
                          color: EditorTheme.ink,
                        ),
                        decoration: const InputDecoration(
                          isDense: true,
                          border: InputBorder.none,
                          contentPadding: EdgeInsets.zero,
                        ),
                        onSubmitted: (text) {
                          var raw = text.trim().replaceFirst('#', '');
                          if (raw.length == 3)
                            raw = raw.split('').map((s) => '$s$s').join();
                          final parsed = raw.length == 6
                              ? int.tryParse(raw, radix: 16)
                              : null;
                          if (parsed == null) {
                            widget.controller.error.value =
                                'Enter a valid hex color';
                            return;
                          }
                          draft = [
                            ((parsed >> 16) & 255) / 255,
                            ((parsed >> 8) & 255) / 255,
                            (parsed & 255) / 255,
                            v[3],
                          ];
                          commit();
                        },
                      ),
                    ),
                  ],
                ),
              ),
              if (supportsAlpha)
                GestureDetector(
                  onPanStart: (e) => sample(e.localPosition, alpha: true),
                  onPanUpdate: (e) => sample(e.localPosition, alpha: true),
                  onPanEnd: (_) => commit(),
                  onPanCancel: cancel,
                  onTapUp: (e) {
                    sample(e.localPosition, alpha: true);
                    commit();
                  },
                  child: Container(
                    width: EditorMetrics.thumb,
                    height: EditorMetrics.s12,
                    decoration: BoxDecoration(
                      gradient: LinearGradient(
                        colors: [
                          color.withValues(alpha: 0),
                          color.withValues(alpha: 1),
                        ],
                      ),
                    ),
                    child: Align(
                      alignment: Alignment(v[3] * 2 - 1, 0),
                      child: Container(
                        width: EditorMetrics.s3,
                        height: EditorMetrics.s12,
                        decoration: BoxDecoration(
                          border: Border.all(color: Colors.white),
                        ),
                      ),
                    ),
                  ),
                ),
            ],
          ),
        ),
      ),
    );
  }
}

class _WheelPainter extends CustomPainter {
  _WheelPainter(this.color);
  final Color color;
  @override
  void paint(Canvas canvas, Size size) {
    final hsv = HSVColor.fromColor(color);
    const center = Offset(64, 64);
    final ring = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 25.6
      ..shader = SweepGradient(
        startAngle: 0,
        endAngle: math.pi * 2,
        colors: [
          for (var i = 0; i <= 6; i++)
            HSVColor.fromAHSV(1, (i * 60) % 360, 1, 1).toColor(),
        ],
        transform: const GradientRotation(-math.pi / 2),
      ).createShader(const Rect.fromLTWH(0, 0, 128, 128));
    canvas.drawCircle(center, 51.2, ring);
    const square = Rect.fromLTWH(25.6, 25.6, 76.8, 76.8);
    canvas.drawRect(
      square,
      Paint()
        ..shader = LinearGradient(
          colors: [Colors.white, HSVColor.fromAHSV(1, hsv.hue, 1, 1).toColor()],
        ).createShader(square),
    );
    canvas.drawRect(
      square,
      Paint()
        ..shader = const LinearGradient(
          begin: Alignment.topCenter,
          end: Alignment.bottomCenter,
          colors: [Colors.transparent, Colors.black],
        ).createShader(square),
    );
    for (final point in [
      Offset(25.6 + hsv.saturation * 76.8, 25.6 + (1 - hsv.value) * 76.8),
      center +
          Offset(
            math.cos((hsv.hue - 90) * math.pi / 180) * 51.2,
            math.sin((hsv.hue - 90) * math.pi / 180) * 51.2,
          ),
    ]) {
      canvas.drawCircle(
        point,
        4,
        Paint()
          ..color = Colors.black
          ..style = PaintingStyle.stroke
          ..strokeWidth = 2,
      );
      canvas.drawCircle(
        point,
        3,
        Paint()
          ..color = Colors.white
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1,
      );
    }
  }

  @override
  bool shouldRepaint(covariant _WheelPainter oldDelegate) =>
      color != oldDelegate.color;
}
