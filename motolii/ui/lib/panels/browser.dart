import 'dart:convert';
import 'dart:io';
import 'dart:math' as math;
import 'dart:ui'
    as ui
    show Vertices, VertexMode, instantiateImageCodec, ImageByteFormat;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../session/editor_session.dart';
import '../foundation/panel_controls.dart';
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

  /// Colours stacked under the wheel; one is a solid, more make a gradient.
  List<List<double>> stops = [];
  final scroll = ScrollController();
  List<Map<String, dynamic>> visible = [];
  int columns = 1;

  /// Category rail width while dragging; null means "as stored".
  double? railDrag;

  static const double tileDefault = 88, railMin = 48;

  /// Colours: the wheel's size is the picker's height; the grip under the
  /// picker drags it.
  static const double wheelDefault = 160;
  double? wheelDrag;
  double get wheelSize =>
      (wheelDrag ??
              (widget.controller.deskWork.value['browserWheel'] as num? ??
                      wheelDefault)
                  .toDouble())
          .clamp(EditorMetrics.thumb, EditorMetrics.s200);

  double get tile => BrowserSize.tile(widget.controller);
  double get rail =>
      railDrag ??
      (widget.controller.deskWork.value['browserRail'] as num? ??
              EditorMetrics.s96)
          .toDouble();

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
    widget.controller.importedAssets.addListener(_revealImported);
    widget.controller.deskWork.addListener(_redraw);
  }

  void _redraw() {
    if (mounted) setState(() {});
  }

  /// 取り込んだ物は Media に居る。開いて、絞り込みを外して、選んでおく。
  void _revealImported() {
    final ids = widget.controller.importedAssets.value;
    if (ids.isEmpty || !mounted) return;
    if (widget.fixedTab != null && widget.fixedTab != 'Media') return;
    setState(() {
      queries[tab] = search.text;
      tab = 'Media';
      search.text = '';
      queries['Media'] = '';
      classifications['Media'] = 'All';
      selected['Media'] = ids.toSet();
      active['Media'] = ids.first;
    });
  }

  @override
  void dispose() {
    widget.controller.importedAssets.removeListener(_revealImported);
    widget.controller.deskWork.removeListener(_redraw);
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
    // 空として置く画(1.0 超を持つ形式)。native の ENVIRONMENT_EXTENSIONS と同じ 2 つ。
    if (mime.contains('/hdr') ||
        mime.contains('/exr') ||
        RegExp(r'\.(hdr|exr)$').hasMatch(path))
      return 'HDR';
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
          'stage' => '3D',
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
              'motolii.radiance': 'Light',
              'motolii.gain': 'Color',
              'motolii.gradient': 'Color',
              'motolii.tri_led': 'Stylize',
              'motolii.repeat': 'Place',
              'motolii.clip': '3D',
            }[id(item)] ??
            'Other';
      default:
        return item['saved'] == true
            ? 'Saved'
            : item['used'] == true
            ? 'Used here'
            : 'Starter';
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
            'id': 'stage',
            'name': 'Stage',
            'detail': 'Widens the working area around the frame',
            'glyph': '⬚',
          },
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
        // 同梱の HDRI は素材の棚に、取り込んだ物と同じ札で並ぶ(Finder・置換・削除は無い)。
        return [
          ...rows(state['assets']),
          for (final b in rows(state['backgrounds']))
            {
              ...b,
              'id': 'background:${b['id']}',
              'mime': 'image/hdr',
              'builtin': true,
              'detail': 'HDR · bundled (Poly Haven, CC0)',
            },
        ];
      case 'Effects':
        return rows(state['catalog']);
      default:
        return [
          ...rows(state['palette']),
          for (final (i, s) in _saved(widget.controller).indexed)
            {...s, 'id': 'saved:$i', 'saved': true},
        ];
    }
  }

  static List<Map<String, dynamic>> _saved(EditorSession c) =>
      EditorSession.maps(c.deskWork.value['swatches']);
  static List<List<double>> _stops(Map<String, dynamic> item) => [
    for (final s in item['stops'] as List? ?? [item['rgba']]) _rgba(s),
  ];

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
        if (item['builtin'] == true) {
          if (has('create')) await c.command('create', {'kind': id(item)});
        } else if (has('placeAsset') && item['missing'] != true) {
          await c.command('placeAsset', {'id': item['id']});
        }
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
        final colors = _stops(item);
        if (colors.length > 1) {
          setState(() => stops = colors);
        } else if (has('applyPalette')) {
          await c.command('applyPalette', {'rgba': colors.single});
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
        'Media' => ['All', 'Video', 'Images', 'HDR', 'Audio', '3D'],
        'Effects' => [
          'All',
          'Blur',
          'Light',
          'Color',
          'Stylize',
          'Place',
          'Other',
        ],
        _ => ['All', 'Saved', 'Used here', 'Starter'],
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
                        has('import') ? widget.controller.importFiles : null,
                      ),
                    if (tab == 'Colors')
                      _smallButton('From image', _paletteFromFile),
                  ],
                ),
              ),
              Expanded(
                child: Row(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    if (rail >= railMin)
                      Container(
                        width: rail,
                        clipBehavior: Clip.hardEdge,
                        decoration: const BoxDecoration(),
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
                                maxLines: 1,
                                overflow: TextOverflow.clip,
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
                    if (rail < railMin)
                      Tooltip(
                        message: 'Show ${tab.toLowerCase()} categories',
                        child: InkWell(
                          key: const ValueKey('browser:rail-tab'),
                          onTap: () => widget.controller.storeDesk(
                            'browserRail',
                            EditorMetrics.s96,
                          ),
                          child: SizedBox(
                            width: EditorMetrics.row,
                            child: Column(
                              children: [
                                const Padding(
                                  padding: EdgeInsets.only(
                                    top: EditorMetrics.s4,
                                  ),
                                  child: Icon(
                                    Icons.chevron_right,
                                    size: EditorMetrics.s14,
                                    color: EditorTheme.muted,
                                  ),
                                ),
                                RotatedBox(
                                  quarterTurns: 1,
                                  child: Text(
                                    chosen == 'All' ? tab : chosen,
                                    maxLines: 1,
                                    style: const TextStyle(
                                      fontSize: EditorMetrics.micro,
                                      color: EditorTheme.accent,
                                    ),
                                  ),
                                ),
                              ],
                            ),
                          ),
                        ),
                      ),
                    _grip(
                      key: const ValueKey('browser:rail-grip'),
                      onDrag: (dx) => setState(
                        () => railDrag = (rail + dx).clamp(
                          0.0,
                          EditorMetrics.s200,
                        ),
                      ),
                      onEnd: () {
                        final width = rail < railMin ? 0.0 : rail;
                        railDrag = null;
                        widget.controller.storeDesk('browserRail', width);
                      },
                      onDoubleTap: () => widget.controller.storeDesk(
                        'browserRail',
                        rail >= railMin ? 0.0 : EditorMetrics.s96,
                      ),
                    ),
                    Expanded(
                      child: LayoutBuilder(
                        builder: (context, constraints) {
                          columns = math.max(
                            1,
                            (constraints.maxWidth /
                                    (tab == 'Colors' ? tile * .6 : tile))
                                .floor(),
                          );
                          return Column(
                            crossAxisAlignment: CrossAxisAlignment.stretch,
                            children: [
                              if (tab == 'Colors') ...[
                                if (target != null)
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
                                _ColorPicker(
                                  controller: widget.controller,
                                  target: target,
                                  enabled: has('setColor'),
                                  size: wheelSize,
                                  stops: stops,
                                  onStops: (next) =>
                                      setState(() => stops = next),
                                ),
                                _grip(
                                  key: const ValueKey('browser:picker-grip'),
                                  vertical: true,
                                  onDrag: (d) =>
                                      setState(() => wheelDrag = wheelSize + d),
                                  onEnd: () {
                                    final size = wheelSize;
                                    wheelDrag = null;
                                    widget.controller.storeDesk(
                                      'browserWheel',
                                      size,
                                    );
                                  },
                                ),
                              ],
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
                                                  ? tile * .55
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
                                                  height: tile * .55,
                                                  child: card(visible[index]),
                                                ),
                                              )
                                            : card(visible[index]),
                                      ),
                              ),
                              _zoomBar(),
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

  /// A thin line you drag; it reports the movement along its axis.
  Widget _grip({
    required Key key,
    required void Function(double delta) onDrag,
    required VoidCallback onEnd,
    VoidCallback? onDoubleTap,
    bool vertical = false,
  }) => MouseRegion(
    cursor: vertical
        ? SystemMouseCursors.resizeUpDown
        : SystemMouseCursors.resizeLeftRight,
    child: GestureDetector(
      key: key,
      behavior: HitTestBehavior.opaque,
      onHorizontalDragUpdate: vertical ? null : (d) => onDrag(d.delta.dx),
      onHorizontalDragEnd: vertical ? null : (_) => onEnd(),
      onVerticalDragUpdate: vertical ? (d) => onDrag(d.delta.dy) : null,
      onVerticalDragEnd: vertical ? (_) => onEnd() : null,
      onDoubleTap: onDoubleTap,
      child: Container(
        width: vertical ? null : EditorMetrics.s4,
        height: vertical ? EditorMetrics.s4 : null,
        decoration: BoxDecoration(
          border: vertical
              ? const Border(bottom: BorderSide(color: EditorTheme.line))
              : const Border(right: BorderSide(color: EditorTheme.line)),
        ),
      ),
    ),
  );

  static const _imageExtensions = [
    'png',
    'jpg',
    'jpeg',
    'webp',
    'bmp',
    'gif',
    'tif',
    'tiff',
  ];
  Future<void> _paletteFromFile() async {
    final picked = await widget.controller.native('pickImport', {
      'extensions': _imageExtensions,
    });
    if (picked is! List) return;
    for (final path in picked.whereType<String>()) {
      await _savePalette(await File(path).readAsBytes());
    }
  }

  /// The picture's main colours become saved solids, shown at once.
  Future<void> _savePalette(Uint8List bytes) async {
    final colors = await paletteOf(bytes);
    if (colors.isEmpty) {
      widget.controller.error.value = 'No colours found in that image';
      return;
    }
    await widget.controller.storeDesk('swatches', [
      ..._saved(widget.controller),
      for (final c in colors)
        {
          'stops': [c],
        },
    ]);
    if (mounted) setState(() => classifications['Colors'] = 'Saved');
  }

  /// Tile size, relative: each step is a fixed ratio, the slider spans the
  /// same range Settings shows.
  Widget _zoomBar() => EditorZoomBar(
    value: tile,
    min: BrowserSize.min,
    max: BrowserSize.max,
    keyPrefix: 'browser:tile',
    onChanged: (v) => widget.controller.storeDesk('browserTile', v),
  );

  Widget card(Map<String, dynamic> item) {
    final supported = switch (tab) {
      'Create' => has('create'),
      'Media' =>
        item['builtin'] == true
            ? has('create')
            : has('placeAsset') && item['missing'] != true,
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
    final body = Tooltip(
      message: isColor && _stops(item).length > 1
          ? 'Double-click to edit its stops · right-click to forget'
          : supported
          ? 'Select · double-click or Enter to drop at the top · drag to place'
          : 'Apply unavailable',
      child: GestureDetector(
        onTap: () => select(item),
        onSecondaryTap: item['saved'] == true
            ? () {
                final kept = _saved(widget.controller)
                  ..removeAt(int.parse('${item['id']}'.split(':').last));
                widget.controller.storeDesk('swatches', kept);
              }
            : null,
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
              ? _gradientBox(_stops(item))
              : Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    Container(height: EditorMetrics.s3, color: identityColor),
                    AspectRatio(
                      aspectRatio: 16 / 9,
                      child: tab == 'Media' || item['thumbnail'] != null
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
                    if (tab == 'Media' && item['builtin'] != true)
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
                            if (item['path'] != null &&
                                !missing &&
                                '${item['mime']}'.startsWith('image/'))
                              Expanded(
                                child: _smallButton(
                                  'Palette',
                                  () => _savePalette(
                                    File('${item['path']}').readAsBytesSync(),
                                  ),
                                ),
                              ),
                            if (item['path'] != null && !missing)
                              Expanded(
                                child: _smallButton(
                                  'Finder',
                                  () => widget.controller.native('reveal', {
                                    'path': item['path'],
                                  }),
                                ),
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
    if (tab != 'Media' || !supported) return body;
    // Timeline の行へ落とすと、その位置に置く。掴んだ札は名前だけ持ち出す。
    return Draggable<Map<String, dynamic>>(
      data: {'asset': item['id'], 'name': item['name']},
      dragAnchorStrategy: pointerDragAnchorStrategy,
      feedback: Material(
        color: Colors.transparent,
        child: Container(
          height: EditorMetrics.row,
          padding: const EdgeInsets.symmetric(horizontal: EditorMetrics.s6),
          decoration: BoxDecoration(
            color: EditorTheme.panel,
            border: Border.all(color: EditorTheme.accent),
          ),
          child: Text(
            '${item['name']}',
            style: const TextStyle(
              fontSize: EditorMetrics.dense,
              color: EditorTheme.ink,
            ),
          ),
        ),
      ),
      child: body,
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
    required this.size,
    required this.stops,
    required this.onStops,
  });
  final EditorSession controller;
  final Map<String, dynamic>? target;
  final bool enabled;

  /// Wheel side the panel asks for; the width may still shrink it.
  final double size;
  final List<List<double>> stops;
  final ValueChanged<List<List<double>>> onStops;
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

  List<double> get value =>
      draft ??
      (widget.target == null ? unbound : _rgba(widget.target!['rgba']));

  /// Set by the last layout; sampling reads the same geometry the paint used.
  _Wheel wheel = _Wheel(EditorMetrics.thumb, 'square');
  String get shape =>
      widget.controller.deskWork.value['colorShape'] as String? ?? 'square';

  void sample(Offset p) {
    pickerFocus.requestFocus();
    final v = value;
    final hsv = HSVColor.fromColor(_color(v));
    dragPart ??= wheel.hitsInner(p, hsv) ? 'sv' : 'hue';
    final updated = dragPart == 'sv'
        ? wheel.pickInner(p, hsv)
        : hsv.withHue(wheel.hueAt(p));
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

  static const double _stopsWidth = EditorMetrics.row;

  /// Stacked colours beside the wheel: tap one to pick it, right-click to
  /// drop it, `+` adds the current colour, Save keeps the strip as a swatch.
  Widget _stopsBar(List<double> current) {
    final stops = widget.stops;
    Widget small(IconData icon, String tip, VoidCallback? press, {Key? key}) =>
        Tooltip(
          message: tip,
          child: InkWell(
            key: key,
            onTap: press,
            child: SizedBox(
              width: _stopsWidth,
              height: EditorMetrics.row,
              child: Icon(
                icon,
                size: EditorMetrics.s14,
                color: press == null
                    ? EditorTheme.muted.withValues(alpha: .45)
                    : EditorTheme.muted,
              ),
            ),
          ),
        );
    return SizedBox(
      width: _stopsWidth,
      height: wheel.side,
      child: Column(
        children: [
          small(
            Icons.add,
            'Add this colour as a stop',
            () => widget.onStops([...stops, current]),
            key: const ValueKey('browser:stop-add'),
          ),
          Expanded(
            child: stops.isEmpty
                ? const SizedBox()
                : ClipRRect(
                    borderRadius: BorderRadius.circular(EditorMetrics.s5),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.stretch,
                      children: [
                        for (final (i, stop) in stops.indexed)
                          Expanded(
                            child: GestureDetector(
                              key: ValueKey('browser:stop:$i'),
                              onTap: () {
                                draft = List.of(stop);
                                commit();
                              },
                              onSecondaryTap: () =>
                                  widget.onStops([...stops]..removeAt(i)),
                              child: ColoredBox(color: _color(stop)),
                            ),
                          ),
                      ],
                    ),
                  ),
          ),
          small(
            Icons.bookmark_add_outlined,
            stops.length > 1 ? 'Save gradient' : 'Save colour',
            stops.isEmpty
                ? null
                : () {
                    widget.controller.storeDesk('swatches', [
                      ..._BrowserPanelState._saved(widget.controller),
                      {'stops': stops},
                    ]);
                    widget.onStops([]);
                  },
            key: const ValueKey('browser:stop-save'),
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final v = value;
    final color = _color(v);
    final display =
        '#${(color.toARGB32() & 0xffffff).toRadixString(16).padLeft(6, '0')}';
    if (!hexFocus.hasFocus) hex.text = display;
    return CallbackShortcuts(
      bindings: {
        const SingleActivator(LogicalKeyboardKey.escape): () {
          widget.controller.eyedropper.value = false;
          cancel();
        },
      },
      child: Focus(
        focusNode: pickerFocus,
        child: Padding(
          padding: const EdgeInsets.all(EditorMetrics.s6),
          child: LayoutBuilder(
            builder: (context, constraints) {
              wheel = _Wheel(
                math
                    .min(
                      widget.size,
                      constraints.maxWidth - _stopsWidth - EditorMetrics.s6,
                    )
                    .clamp(EditorMetrics.s96, EditorMetrics.s200),
                shape,
              );
              return Column(
                children: [
                  Row(
                    mainAxisSize: MainAxisSize.min,
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
                          width: wheel.side,
                          height: wheel.side,
                          child: CustomPaint(
                            painter: _WheelPainter(color, wheel),
                          ),
                        ),
                      ),
                      const SizedBox(width: EditorMetrics.s6),
                      _stopsBar(v),
                    ],
                  ),
                  Container(
                    width: wheel.side + EditorMetrics.s6 + _stopsWidth,
                    height: EditorMetrics.s23,
                    margin: const EdgeInsets.only(top: EditorMetrics.s6),
                    padding: const EdgeInsets.symmetric(
                      horizontal: EditorMetrics.s5,
                    ),
                    decoration: BoxDecoration(
                      color: EditorTheme.line,
                      borderRadius: BorderRadius.circular(EditorMetrics.s5),
                    ),
                    child: Row(
                      children: [
                        _Swatch(color: color, size: EditorMetrics.s14),
                        const SizedBox(width: EditorMetrics.s6),
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
                        ValueListenableBuilder<bool>(
                          valueListenable: widget.controller.eyedropper,
                          builder: (context, on, _) => Tooltip(
                            message: on
                                ? 'Click the Stage to pick a colour · Esc cancels'
                                : 'Pick a colour from the Stage',
                            child: InkWell(
                              key: const ValueKey('browser:eyedropper'),
                              onTap: () =>
                                  widget.controller.eyedropper.value = !on,
                              child: Padding(
                                padding: const EdgeInsets.only(
                                  right: EditorMetrics.s6,
                                ),
                                child: Icon(
                                  Icons.colorize,
                                  size: EditorMetrics.s14,
                                  color: on
                                      ? EditorTheme.accent
                                      : EditorTheme.muted,
                                ),
                              ),
                            ),
                          ),
                        ),
                        Tooltip(
                          message: shape == 'square'
                              ? 'Switch to triangle'
                              : 'Switch to square',
                          child: InkWell(
                            key: const ValueKey('browser:color-shape'),
                            onTap: () => widget.controller.storeDesk(
                              'colorShape',
                              shape == 'square' ? 'triangle' : 'square',
                            ),
                            child: Icon(
                              shape == 'square'
                                  ? Icons.change_history
                                  : Icons.crop_square,
                              size: EditorMetrics.s14,
                              color: EditorTheme.muted,
                            ),
                          ),
                        ),
                      ],
                    ),
                  ),
                ],
              );
            },
          ),
        ),
      ),
    );
  }
}

/// Where the ring and the inner area are, for one wheel size and shape.
/// Sampling and painting both read this, so they cannot disagree.
class _Wheel {
  _Wheel(this.side, this.shape);
  final double side;
  final String shape;
  static const double ring = 14;
  Offset get center => Offset(side / 2, side / 2);
  double get hueRadius => side / 2 - ring / 2;
  double get inner => side / 2 - ring - EditorMetrics.s3;
  Rect get square => Rect.fromCenter(
    center: center,
    width: inner * math.sqrt2,
    height: inner * math.sqrt2,
  );
  Offset _rim(double degrees) {
    final a = (degrees - 90) * math.pi / 180;
    return center + Offset(math.cos(a), math.sin(a)) * inner;
  }

  /// Hue, white, black corners; the hue corner points at the hue handle.
  List<Offset> triangle(double hue) => [
    _rim(hue),
    _rim(hue + 120),
    _rim(hue + 240),
  ];
  double hueAt(Offset p) =>
      (math.atan2(p.dy - center.dy, p.dx - center.dx) * 180 / math.pi + 90) %
      360;

  /// Barycentric weights of p against the triangle (hue, white, black).
  List<double> _weights(Offset p, List<Offset> t) {
    final d =
        (t[1].dy - t[2].dy) * (t[0].dx - t[2].dx) +
        (t[2].dx - t[1].dx) * (t[0].dy - t[2].dy);
    final a =
        ((t[1].dy - t[2].dy) * (p.dx - t[2].dx) +
            (t[2].dx - t[1].dx) * (p.dy - t[2].dy)) /
        d;
    final b =
        ((t[2].dy - t[0].dy) * (p.dx - t[2].dx) +
            (t[0].dx - t[2].dx) * (p.dy - t[2].dy)) /
        d;
    return [a, b, 1 - a - b];
  }

  bool hitsInner(Offset p, HSVColor hsv) => shape == 'triangle'
      ? (p - center).distance <= inner + EditorMetrics.s4
      : square.inflate(EditorMetrics.s4).contains(p);
  HSVColor pickInner(Offset p, HSVColor hsv) {
    if (shape == 'triangle') {
      final w = _weights(
        p,
        triangle(hsv.hue),
      ).map((x) => x.clamp(0.0, 1.0)).toList();
      final sum = w[0] + w[1] + w[2];
      final a = w[0] / sum, b = w[1] / sum;
      final value = a + b;
      return hsv
          .withValue(value.clamp(0.0, 1.0))
          .withSaturation(value == 0 ? 0 : (a / value).clamp(0.0, 1.0));
    }
    final r = square;
    return hsv
        .withSaturation(((p.dx - r.left) / r.width).clamp(0.0, 1.0))
        .withValue((1 - (p.dy - r.top) / r.height).clamp(0.0, 1.0));
  }

  Offset innerHandle(HSVColor hsv) {
    if (shape == 'triangle') {
      final t = triangle(hsv.hue);
      final a = hsv.saturation * hsv.value, b = hsv.value - a;
      return t[0] * a + t[1] * b + t[2] * (1 - a - b);
    }
    final r = square;
    return Offset(
      r.left + hsv.saturation * r.width,
      r.top + (1 - hsv.value) * r.height,
    );
  }

  Offset hueHandle(HSVColor hsv) {
    final a = (hsv.hue - 90) * math.pi / 180;
    return center + Offset(math.cos(a), math.sin(a)) * hueRadius;
  }
}

class _WheelPainter extends CustomPainter {
  _WheelPainter(this.color, this.wheel);
  final Color color;
  final _Wheel wheel;

  @override
  void paint(Canvas canvas, Size size) {
    final hsv = HSVColor.fromColor(color);
    final pure = HSVColor.fromAHSV(1, hsv.hue, 1, 1).toColor();
    final bounds = Offset.zero & size;
    canvas.drawCircle(
      wheel.center,
      wheel.hueRadius,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = _Wheel.ring
        ..shader = SweepGradient(
          colors: [
            for (var i = 0; i <= 6; i++)
              HSVColor.fromAHSV(1, (i * 60) % 360, 1, 1).toColor(),
          ],
          transform: const GradientRotation(-math.pi / 2),
        ).createShader(bounds),
    );
    // A dark hairline on both rims lifts the ring off the panel.
    final rim = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1
      ..color = EditorTheme.line;
    canvas.drawCircle(wheel.center, wheel.side / 2 - _Wheel.ring, rim);
    canvas.drawCircle(wheel.center, wheel.side / 2 - .5, rim);

    final shadow = Paint()
      ..color = Colors.black45
      ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 2);
    if (wheel.shape == 'triangle') {
      final t = wheel.triangle(hsv.hue);
      final path = Path()..addPolygon(t, true);
      canvas.drawPath(path.shift(const Offset(0, 1)), shadow);
      canvas.drawVertices(
        ui.Vertices(
          ui.VertexMode.triangles,
          t,
          colors: [pure, Colors.white, Colors.black],
        ),
        BlendMode.srcOver,
        Paint(),
      );
      canvas.drawPath(path, rim);
    } else {
      final r = RRect.fromRectAndRadius(
        wheel.square,
        const Radius.circular(EditorMetrics.s4),
      );
      canvas.drawRRect(r.shift(const Offset(0, 1)), shadow);
      canvas.drawRRect(
        r,
        Paint()
          ..shader = LinearGradient(colors: [Colors.white, pure])
              .createShader(wheel.square),
      );
      canvas.drawRRect(
        r,
        Paint()
          ..shader = const LinearGradient(
            begin: Alignment.topCenter,
            end: Alignment.bottomCenter,
            colors: [Colors.transparent, Colors.black],
          ).createShader(wheel.square),
      );
      canvas.drawRRect(r, rim);
    }

    _handle(canvas, wheel.innerHandle(hsv), color.withValues(alpha: 1));
    _handle(canvas, wheel.hueHandle(hsv), pure);
  }

  void _handle(Canvas canvas, Offset at, Color fill) {
    canvas.drawCircle(
      at.translate(0, 1),
      5,
      Paint()
        ..color = Colors.black54
        ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 1.5),
    );
    canvas.drawCircle(at, 5, Paint()..color = Colors.white);
    canvas.drawCircle(at, 3.5, Paint()..color = fill);
  }

  @override
  bool shouldRepaint(covariant _WheelPainter oldDelegate) =>
      color != oldDelegate.color ||
      wheel.side != oldDelegate.wheel.side ||
      wheel.shape != oldDelegate.wheel.shape;
}

/// Transparency shows as the usual light/dark checker.
class _CheckerPainter extends CustomPainter {
  @override
  void paint(Canvas canvas, Size size) {
    const cell = EditorMetrics.s4;
    final light = Paint()..color = const Color(0xffbbbbbb);
    final dark = Paint()..color = const Color(0xff777777);
    for (var y = 0.0; y < size.height; y += cell)
      for (var x = 0.0; x < size.width; x += cell)
        canvas.drawRect(
          Rect.fromLTWH(x, y, cell, cell),
          ((x + y) / cell).round().isEven ? light : dark,
        );
  }

  @override
  bool shouldRepaint(covariant _CheckerPainter oldDelegate) => false;
}

/// A solid, or the same strip a saved gradient was made from.
Widget _gradientBox(List<List<double>> stops) => DecoratedBox(
  decoration: BoxDecoration(
    color: stops.length == 1 ? _color(stops.single) : null,
    gradient: stops.length > 1
        ? LinearGradient(colors: [for (final s in stops) _color(s)])
        : null,
  ),
);

class _Swatch extends StatelessWidget {
  const _Swatch({required this.color, required this.size});
  final Color color;
  final double size;
  @override
  Widget build(BuildContext context) => Container(
    width: size,
    height: size,
    clipBehavior: Clip.antiAlias,
    decoration: BoxDecoration(
      borderRadius: BorderRadius.circular(EditorMetrics.s4),
      border: Border.all(color: Colors.black26),
    ),
    child: CustomPaint(
      painter: _CheckerPainter(),
      child: ColoredBox(color: color),
    ),
  );
}

/// One knob for how big Browser tiles are; Settings turns it, Browser reads it.
abstract final class BrowserSize {
  static const double min = 48, max = 200;
  static double tile(EditorSession c) =>
      (c.deskWork.value['browserTile'] as num? ??
              _BrowserPanelState.tileDefault)
          .toDouble()
          .clamp(min, max);
}

/// The few colours a picture is mostly made of: decode small, drop
/// transparent pixels, k-means in RGB, brightest first.
Future<List<List<double>>> paletteOf(Uint8List bytes, {int count = 6}) async {
  final codec = await ui.instantiateImageCodec(bytes, targetWidth: 64);
  final image = (await codec.getNextFrame()).image;
  final data = await image.toByteData(format: ui.ImageByteFormat.rawRgba);
  image.dispose();
  if (data == null) return [];
  final px = <List<double>>[];
  for (var i = 0; i + 3 < data.lengthInBytes; i += 4) {
    if (data.getUint8(i + 3) < 128) continue;
    px.add([
      data.getUint8(i) / 255,
      data.getUint8(i + 1) / 255,
      data.getUint8(i + 2) / 255,
    ]);
  }
  if (px.isEmpty) return [];
  double luma(List<double> c) => .299 * c[0] + .587 * c[1] + .114 * c[2];
  px.sort((a, b) => luma(a).compareTo(luma(b)));
  final k = math.min(count, px.length);
  var centers = [
    for (var i = 0; i < k; i++) px[(px.length - 1) * i ~/ math.max(1, k - 1)],
  ];
  for (var round = 0; round < 8; round++) {
    final sums = List.generate(k, (_) => [0.0, 0.0, 0.0, 0.0]);
    for (final p in px) {
      var best = 0;
      var bestD = double.infinity;
      for (var i = 0; i < k; i++) {
        final c = centers[i];
        final d =
            (p[0] - c[0]) * (p[0] - c[0]) +
            (p[1] - c[1]) * (p[1] - c[1]) +
            (p[2] - c[2]) * (p[2] - c[2]);
        if (d < bestD) {
          bestD = d;
          best = i;
        }
      }
      final s = sums[best];
      s[0] += p[0];
      s[1] += p[1];
      s[2] += p[2];
      s[3] += 1;
    }
    centers = [
      for (var i = 0; i < k; i++)
        sums[i][3] == 0
            ? centers[i]
            : [
                sums[i][0] / sums[i][3],
                sums[i][1] / sums[i][3],
                sums[i][2] / sums[i][3],
              ],
    ];
  }
  // Edge blends and stray pixels do not count as a colour of the picture.
  final weights = List.filled(k, 0);
  for (final p in px) {
    var best = 0;
    var bestD = double.infinity;
    for (var i = 0; i < k; i++) {
      final c = centers[i];
      final d =
          (p[0] - c[0]) * (p[0] - c[0]) +
          (p[1] - c[1]) * (p[1] - c[1]) +
          (p[2] - c[2]) * (p[2] - c[2]);
      if (d < bestD) {
        bestD = d;
        best = i;
      }
    }
    weights[best]++;
  }
  final unique = <String, List<double>>{
    for (final (i, c) in centers.indexed)
      if (weights[i] * 20 >= px.length)
        c.map((v) => (v * 255).round()).join(','): [...c, 1.0],
  };
  return unique.values.toList()..sort((a, b) => luma(b).compareTo(luma(a)));
}
