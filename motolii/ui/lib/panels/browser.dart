import 'native_visual_sample.dart';

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

  /// One tile's width at the shelf's current size. The grid already knows it,
  /// so a card's name is fitted against this instead of measuring itself.
  double tileWidth = BrowserSize.base;
  /// Media opens on pictures alone: its items are told apart by their
  /// picture, not their name. Create and Effects keep the grid.
  int get viewMode =>
      (widget.controller.deskWork.value['browserView'] as num? ??
              (tab == 'Media' ? 2 : 0))
          .toInt();

  /// Category rail width while dragging; null means "as stored".
  double? railDrag;

  static const double tileDefault = BrowserSize.base, railMin = 48;

  /// Grid: the gutter between tiles, and the single caption line under every
  /// preview. Nothing else shares that line, so the name keeps the tile's
  /// whole width; format and status ride on the picture instead.
  static const double _gutter = EditorMetrics.s2,
      _inset = EditorMetrics.s8,
      _captionHeight = EditorMetrics.control;
  int total = 0;

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

  /// The tile is one drawing that scales: caption, badge and airs keep
  /// their proportion to the picture at every size. Only the type has a
  /// floor, below which a name stops being legible.
  double get tileScale => tile / BrowserSize.base;
  double get captionSize =>
      math.max(EditorMetrics.micro, EditorMetrics.font * tileScale);
  double get captionHeight => _captionHeight * tileScale;

  /// A mark grows slower than the picture it marks: by the square root, so
  /// at twice the tile it is 1.4× and stays a mark beside the name.
  double get markScale => math.sqrt(tileScale);
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
          final target = EditorSession.map(c.state['colorTarget']);
          final layer = c.activeLayer;
          final fill = EditorSession.map(layer?['fill']);
          final slot = target['slot'] ?? fill['slot'];
          if (has('setGradient') && slot != null) {
            await c.command('setGradient', {'slot': slot, 'stops': colors});
          }
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
    valueListenable: widget.controller.slice(
      'browser',
      const [
        'assets',
        'backgrounds',
        'catalog',
        'palette',
        'colorTarget',
        'importExtensions',
        'capabilities',
      ],
      // The shelf reads the selection twice only: the colour it would edit,
      // and whether there is anything to apply to. Naming those instead of
      // the selection keeps the shelves still while layers are picked.
      derived: () => [
        _colorTarget(widget.controller),
        widget.controller.selectedIds.isEmpty,
      ],
    ),
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
      total = all.length;
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
                height: EditorMetrics.tall,
                padding: const EdgeInsets.symmetric(
                  horizontal: _inset,
                  vertical: EditorMetrics.s4,
                ),
                decoration: const BoxDecoration(
                  border: Border(bottom: BorderSide(color: EditorTheme.line)),
                ),
                child: Row(
                  children: [
                    Expanded(
                      child: Container(
                        decoration: BoxDecoration(
                          color: EditorTheme.app,
                          border: Border.all(
                            color: searchFocus.hasFocus
                                ? EditorTheme.border
                                : EditorTheme.line,
                          ),
                        ),
                        child: TextField(
                          controller: search,
                          focusNode: searchFocus,
                          style: const TextStyle(
                            fontSize: EditorMetrics.font,
                            color: EditorTheme.ink,
                          ),
                          decoration: InputDecoration(
                            isDense: true,
                            prefixIcon: const Icon(
                              Icons.search,
                              size: EditorMetrics.s14,
                              color: EditorTheme.muted,
                            ),
                            prefixIconConstraints: const BoxConstraints(
                              minWidth: EditorMetrics.control,
                              minHeight: EditorMetrics.row,
                            ),
                            contentPadding: const EdgeInsets.symmetric(
                              horizontal: EditorMetrics.s4,
                              vertical: EditorMetrics.s4,
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
                    ),
                    if (tab == 'Media') ...[
                      const SizedBox(width: EditorMetrics.s6),
                      _action(
                        'Import',
                        has('import') ? widget.controller.importFiles : null,
                      ),
                    ],
                    if (tab != 'Colors') ...[
                      const SizedBox(width: EditorMetrics.s6),
                      _views(),
                    ],
                    if (tab == 'Colors') ...[
                      const SizedBox(width: EditorMetrics.s6),
                      _action('From image', _paletteFromFile),
                    ],
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
                                _inset,
                                EditorMetrics.s8,
                                EditorMetrics.s4,
                                EditorMetrics.s4,
                              ),
                              child: Text(
                                tab.toUpperCase(),
                                maxLines: 1,
                                overflow: TextOverflow.clip,
                                style: const TextStyle(
                                  fontSize: EditorMetrics.micro,
                                  letterSpacing: 1,
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
                      EditorTooltip(
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
                          if (viewMode == 1 && tab != 'Colors') columns = 1;
                          tileWidth =
                              (constraints.maxWidth -
                                  _gutter * (columns - 1)) /
                              columns;
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
                                    : ColoredBox(
                                        color: tab == 'Colors'
                                            ? EditorTheme.panel
                                            : EditorTheme.line,
                                        child: GridView.builder(
                                        controller: scroll,
                                        padding: EdgeInsets.all(
                                          tab == 'Colors'
                                              ? EditorMetrics.s6
                                              : 0,
                                        ),
                                        gridDelegate:
                                            SliverGridDelegateWithFixedCrossAxisCount(
                                              crossAxisCount: columns,
                                              mainAxisExtent: tab == 'Colors'
                                                  ? tile * .55
                                                  : viewMode == 1
                                                  ? EditorMetrics.s48
                                                  : (constraints.maxWidth -
                                                                _gutter *
                                                                    (columns -
                                                                        1)) /
                                                            columns *
                                                            9 /
                                                            16 +
                                                        (viewMode == 2
                                                            ? 0
                                                            : captionHeight),
                                              crossAxisSpacing: tab == 'Colors'
                                                  ? EditorMetrics.s4
                                                  : _gutter,
                                              mainAxisSpacing: tab == 'Colors'
                                                  ? EditorMetrics.s4
                                                  : _gutter,
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
        color: EditorTheme.line,
        alignment: Alignment.center,
        child: Container(
          width: vertical ? EditorMetrics.s16 : EditorMetrics.s2,
          height: vertical ? EditorMetrics.s2 : EditorMetrics.s16,
          color: EditorTheme.raised,
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

  /// Tile size as a percentage of its default.
  Widget _zoomBar() => Container(
    height: EditorMetrics.tall,
    padding: const EdgeInsets.symmetric(
      horizontal: _inset,
      vertical: EditorMetrics.s4,
    ),
    decoration: const BoxDecoration(
      border: Border(top: BorderSide(color: EditorTheme.line)),
    ),
    child: LayoutBuilder(
      builder: (context, box) {
        // 譲る順は 件数 → 寸法棒。寸法棒は、棒が出る幅ならその分、
        // 出ない幅でも押し所 2 つ分を必ず残す。
        final slider = box.maxWidth >= EditorMetrics.cell + EditorMetrics.s48;
        final countRoom =
            box.maxWidth -
            (slider ? EditorMetrics.cell : _zoomFloor + EditorMetrics.s64) -
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
            child: EditorTooltip(
              message: _countTip(),
              child: Text(
                compact ? '${visible.length}' : _countLabel(),
                key: const ValueKey('browser:count'),
                maxLines: 1,
                softWrap: false,
                overflow: TextOverflow.ellipsis,
                style: const TextStyle(
                  fontSize: EditorMetrics.dense,
                  color: EditorTheme.muted,
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

  /// 寸法棒が畳めない押し所 2 つ分。棒が出ない幅では、これに %の欄が足される。
  static const _zoomFloor = EditorMetrics.row * 2;

  /// Grid / List / Thumbnails, beside the search field.
  Widget _views() => DecoratedBox(
    decoration: BoxDecoration(
      color: EditorTheme.app,
      border: Border.all(color: EditorTheme.line),
    ),
    child: Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        for (final (mode, icon, label) in [
          (0, Icons.grid_view, 'Grid'),
          (1, Icons.view_list, 'List'),
          (2, Icons.crop_landscape, 'Thumbnails'),
        ])
          Container(
            width: EditorMetrics.control,
            height: EditorMetrics.row,
            color: viewMode == mode ? EditorTheme.raised : Colors.transparent,
            child: EditorTooltip(
              message: label,
              child: IconButton(
                key: ValueKey('browser:view:$mode'),
                iconSize: EditorMetrics.s14,
                color: viewMode == mode ? EditorTheme.ink : EditorTheme.muted,
                onPressed: () =>
                    widget.controller.storeDesk('browserView', mode),
                icon: Icon(icon),
              ),
            ),
          ),
      ],
    ),
  );

  Widget _sizeSlider() => EditorZoomBar(
    base: tileDefault,
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

  /// The badge already carries the extension, so the name drops it and keeps
  /// the width for the part that tells one file from another.
  String _displayName(Map<String, dynamic> item) {
    final name = '${item['name'] ?? item['id']}';
    final dot = name.lastIndexOf('.');
    return dot > 0 && name.substring(dot + 1).toUpperCase() == _format(item)
        ? name.substring(0, dot)
        : name;
  }

  /// The badge's laid-out width, so the name knows how much of the caption
  /// it keeps. Formats repeat across every tile; lay each out once.
  static final _badgeWidths = <String, double>{};
  static double _badgeWidth(String format) =>
      _badgeWidths[format] ??= () {
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

  String _format(Map<String, dynamic> item) {
    if (tab != 'Media') return '';
    if (item['builtin'] == true) return 'HDR';
    final filename = '${item['path'] ?? item['name'] ?? ''}'.split('/').last;
    if (filename.contains('.')) return filename.split('.').last.toUpperCase();
    final mime = '${item['mime'] ?? ''}';
    return mime.contains('/')
        ? mime.split('/').last.toUpperCase()
        : family(item);
  }

  void _menu(Map<String, dynamic> item, Offset point) {
    select(item);
    final missing = item['missing'] == true;
    final own = tab == 'Media' && item['builtin'] != true;
    showMenu<String>(
      context: context,
      position: RelativeRect.fromLTRB(point.dx, point.dy, point.dx, point.dy),
      items: [
        PopupMenuItem(
          value: 'apply',
          child: Text(tab == 'Media' ? 'Place' : 'Apply'),
        ),
        if (own) ...[
          PopupMenuItem(
            value: 'replace',
            enabled:
                has('replaceAsset') &&
                !missing &&
                widget.controller.selectedIds.isNotEmpty,
            child: const Text('Replace selected layer'),
          ),
          if (item['path'] != null && !missing) ...[
            const PopupMenuItem(
              value: 'reveal',
              child: Text('Reveal in Finder'),
            ),
            if ('${item['mime']}'.startsWith('image/'))
              const PopupMenuItem(
                value: 'palette',
                child: Text('Extract palette'),
              ),
          ],
          PopupMenuItem(
            value: 'remove',
            enabled: has('removeAsset') && item['used'] != true,
            child: const Text('Remove from library'),
          ),
        ],
        if (item['saved'] == true)
          const PopupMenuItem(value: 'forget', child: Text('Forget swatch')),
      ],
    ).then((action) {
      if (!mounted) return;
      switch (action) {
        case 'apply':
          apply(item);
        case 'replace':
          widget.controller.command('replaceAsset', {'id': item['id']});
        case 'reveal':
          widget.controller.native('reveal', {'path': item['path']});
        case 'palette':
          _savePalette(File('${item['path']}').readAsBytesSync());
        case 'remove':
          widget.controller.command('removeAsset', {'id': item['id']});
        case 'forget':
          final kept = _saved(widget.controller)
            ..removeAt(int.parse('${item['id']}'.split(':').last));
          widget.controller.storeDesk('swatches', kept);
      }
    });
  }

  Widget card(Map<String, dynamic> item) {
    final supported = switch (tab) {
      'Create' => has('create'),
      'Media' =>
        item['builtin'] == true
            ? has('create')
            : has('placeAsset') && item['missing'] != true,
      'Effects' =>
        has('applyEffect') && widget.controller.selectedIds.isNotEmpty,
      _ =>
        (_stops(item).length > 1 ? has('setGradient') : has('applyPalette')) &&
            widget.controller.selectedIds.isNotEmpty,
    };
    final isSelected = selected[tab]?.contains(id(item)) ?? false;
    final isColor = tab == 'Colors';
    final identityColor = EditorTheme.kindColor(
      tab == 'Media' ? family(item) : id(item),
    );
    final missing = item['missing'] == true;
    final name = '${item['name'] ?? item['id']}';
    final format = _format(item);
    Widget preview = isColor
        ? _gradientBox(widget.controller, _stops(item))
        : tab == 'Media' || item['thumbnail'] != null
        ? _thumbnail(item)
        : Center(
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
          );
    // Every preview sits in the same ground so light and dark pictures read
    // as separate tiles; colours are their own ground.
    if (!isColor) preview = ColoredBox(color: EditorTheme.app, child: preview);
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
                    Icons.error_outline,
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
            key: ValueKey('browser:format:${id(item)}'),
            height: EditorMetrics.s14 * markScale,
            padding: EdgeInsets.symmetric(
              horizontal: EditorMetrics.s4 * markScale,
            ),
            alignment: Alignment.center,
            decoration: BoxDecoration(
              color: identityColor,
              borderRadius: BorderRadius.circular(
                EditorMetrics.s3 * markScale,
              ),
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
      key: ValueKey('browser:name:${id(item)}'),
      height: captionHeight,
      child: Padding(
        padding: EdgeInsets.symmetric(horizontal: air),
        child: Row(
          children: [
            Expanded(
              child: Align(
                alignment: Alignment.centerLeft,
                child: _FittedName(
                  _displayName(item),
                  tileWidth - air * 2 - badgeRoom,
                  size: captionSize,
                ),
              ),
            ),
            if (badge != null) ...[
              SizedBox(width: air),
              badge,
            ],
          ],
        ),
      ),
    );
    final body = EditorTooltip(
      key: ValueKey('browser:$tab:${id(item)}'),
      message: [
        name,
        if (missing) 'Missing file',
        if (item['used'] == true) 'In use by a layer',
        if (item['detail'] != null) '${item['detail']}',
        supported
            ? (isColor
                  ? 'Click to apply · Right-click for actions'
                  : 'Double-click or Enter to apply · Right-click for actions')
            : 'Apply unavailable',
      ].join('\n'),
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        // Selection happens on the press itself: with a double-tap handler
        // beside it, onTap would wait out the double-tap window first.
        onTapDown: (_) => select(item),
        onTap: isColor && supported ? () => apply(item) : null,
        onSecondaryTapDown: (event) => _menu(item, event.globalPosition),
        onDoubleTap: isColor ? null : () => apply(item),
        child: Container(
          decoration: BoxDecoration(
            color: EditorTheme.panel,
            border: Border.all(
              color: isSelected ? EditorTheme.spatial : Colors.transparent,
            ),
          ),
          child: Stack(
            children: [
              Positioned.fill(
                child: isColor
                    ? preview
                    : viewMode == 2
                    ? _Hover(
                        builder: (hovered) => Stack(
                        fit: StackFit.expand,
                        children: [
                          preview,
                          if (badge != null)
                            Positioned(
                              right: air,
                              bottom: air,
                              child: badge,
                            ),
                          // The chosen card says its name, and so does the
                          // one under the pointer: a band over the picture's
                          // foot, so the picture stays the point.
                          if (isSelected || hovered)
                            Positioned(
                              left: 0,
                              right: 0,
                              bottom: 0,
                              child: Container(
                                key: ValueKey('browser:band:${id(item)}'),
                                height: EditorMetrics.row * tileScale,
                                padding: EdgeInsets.only(
                                  left: air,
                                  right: badgeRoom + air,
                                ),
                                alignment: Alignment.centerLeft,
                                color: EditorTheme.app.withValues(alpha: .75),
                                child: _FittedName(
                                  _displayName(item),
                                  tileWidth - air * 2 - badgeRoom,
                                  size: captionSize,
                                  sliding: hovered,
                                ),
                              ),
                            ),
                        ],
                        ),
                      )
                    : viewMode == 1
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
    // A mesh has no picture to show, so its name draws the body it names;
    // otherwise every 3D file would wear one icon.
    final shape = item['missing'] == true || family(item) != '3D'
        ? null
        : _shapeOf('${item['name'] ?? item['id']}');
    final fallback = Center(
      child: shape != null
          ? SizedBox(
              width: EditorMetrics.s32 * tileScale,
              height: EditorMetrics.s32 * tileScale,
              child: CustomPaint(
                key: ValueKey('browser:shape:${shape.name}'),
                painter: _ShapeMark(shape, EditorTheme.kindColor('3d')),
              ),
            )
          : Icon(
              item['missing'] == true
                  ? Icons.broken_image_outlined
                  : family(item) == 'Audio'
                  ? Icons.audiotrack
                  : family(item) == '3D'
                  ? Icons.view_in_ar
                  : Icons.image_outlined,
              size: EditorMetrics.s19 * tileScale,
              color: EditorTheme.muted,
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

/// A bordered action beside the search field.
Widget _action(String label, VoidCallback? press) => EditorTooltip(
  message: press == null ? '$label · unavailable' : label,
  child: InkWell(
    onTap: press,
    child: Container(
      height: EditorMetrics.row,
      alignment: Alignment.center,
      padding: const EdgeInsets.symmetric(horizontal: EditorMetrics.s8),
      decoration: BoxDecoration(
        color: EditorTheme.app,
        border: Border.all(
          color: press == null ? EditorTheme.line : EditorTheme.border,
        ),
      ),
      child: Text(
        label,
        maxLines: 1,
        style: TextStyle(
          fontSize: EditorMetrics.font,
          color: press == null ? EditorTheme.disabledInk : EditorTheme.ink,
        ),
      ),
    ),
  ),
);

Widget _smallButton(
  String label,
  VoidCallback? press, {
  bool selected = false,
}) => EditorTooltip(
  message: press == null ? '$label · unavailable' : label,
  child: InkWell(
    onTap: press,
    child: Container(
      height: EditorMetrics.control,
      alignment: Alignment.centerLeft,
      padding: const EdgeInsets.symmetric(horizontal: EditorMetrics.s8),
      color: selected ? EditorTheme.raised : Colors.transparent,
      child: Text(
        label,
        maxLines: 1,
        overflow: TextOverflow.ellipsis,
        style: TextStyle(
          fontSize: EditorMetrics.font,
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
        EditorTooltip(
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
                          builder: (context, on, _) => EditorTooltip(
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
                        EditorTooltip(
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
Widget _gradientBox(EditorSession controller, List<List<double>> stops) =>
    stops.length > 1
    ? NativeVisualSample(
        controller: controller,
        request: {'kind': 'gradient', 'type': 'linear', 'stops': stops},
        fit: BoxFit.fill,
      )
    : ColoredBox(color: _color(stops.single));

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

/// The bodies a mesh file is usually named after. A file whose name says none
/// of them keeps the generic 3D icon.
enum _Shape { sphere, torus, cube, cylinder, cone, pyramid, plane }

const _shapeWords = <String, _Shape>{
  'sphere': _Shape.sphere,
  'ball': _Shape.sphere,
  'icosphere': _Shape.sphere,
  'torus': _Shape.torus,
  'donut': _Shape.torus,
  'cube': _Shape.cube,
  'box': _Shape.cube,
  'cylinder': _Shape.cylinder,
  'tube': _Shape.cylinder,
  'cone': _Shape.cone,
  'pyramid': _Shape.pyramid,
  'tetra': _Shape.pyramid,
  'plane': _Shape.plane,
  'quad': _Shape.plane,
};

_Shape? _shapeOf(String name) {
  final text = name.toLowerCase();
  for (final entry in _shapeWords.entries)
    if (text.contains(entry.key)) return entry.value;
  return null;
}

/// The outline of one body, drawn from the box it is given so the same mark
/// works at any tile size.
class _ShapeMark extends CustomPainter {
  const _ShapeMark(this.shape, this.color);
  final _Shape shape;
  final Color color;

  @override
  void paint(Canvas canvas, Size size) {
    final stroke = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1
      ..color = color;
    final s = size.shortestSide;
    final c = Offset(size.width / 2, size.height / 2);
    Rect around(double rx, double ry, [double dy = 0]) => Rect.fromCenter(
      center: c.translate(0, dy * s),
      width: rx * s,
      height: ry * s,
    );
    switch (shape) {
      case _Shape.sphere:
        canvas.drawCircle(c, s * .38, stroke);
        canvas.drawOval(around(.76, .3), stroke);
      case _Shape.torus:
        canvas.drawOval(around(.86, .5), stroke);
        canvas.drawOval(around(.34, .2), stroke);
      case _Shape.cube:
        final path = Path();
        for (var i = 0; i < 6; i++) {
          final a = math.pi / 6 + i * math.pi / 3;
          final p = c + Offset(math.cos(a), math.sin(a)) * s * .4;
          i == 0 ? path.moveTo(p.dx, p.dy) : path.lineTo(p.dx, p.dy);
        }
        canvas.drawPath(path..close(), stroke);
        for (final a in [math.pi / 2, math.pi * 7 / 6, math.pi * 11 / 6])
          canvas.drawLine(
            c,
            c + Offset(math.cos(a), math.sin(a)) * s * .4,
            stroke,
          );
      case _Shape.cylinder:
        canvas.drawOval(around(.6, .22, -.28), stroke);
        canvas.drawOval(around(.6, .22, .28), stroke);
        for (final x in [-.3, .3])
          canvas.drawLine(
            c + Offset(x * s, -s * .28),
            c + Offset(x * s, s * .28),
            stroke,
          );
      case _Shape.cone:
        canvas.drawOval(around(.7, .24, .3), stroke);
        for (final x in [-.35, .35])
          canvas.drawLine(
            c + Offset(0, -s * .38),
            c + Offset(x * s, s * .3),
            stroke,
          );
      case _Shape.pyramid:
        final apex = c + Offset(0, -s * .38);
        final left = c + Offset(-s * .38, s * .3);
        final right = c + Offset(s * .38, s * .3);
        final back = c + Offset(s * .08, s * .12);
        canvas.drawPath(
          Path()
            ..moveTo(apex.dx, apex.dy)
            ..lineTo(left.dx, left.dy)
            ..lineTo(right.dx, right.dy)
            ..close(),
          stroke,
        );
        canvas.drawLine(apex, back, stroke);
        canvas.drawLine(left, back, stroke);
        canvas.drawLine(right, back, stroke);
      case _Shape.plane:
        canvas.drawPath(
          Path()
            ..moveTo(c.dx - s * .2, c.dy - s * .22)
            ..lineTo(c.dx + s * .44, c.dy - s * .22)
            ..lineTo(c.dx + s * .2, c.dy + s * .22)
            ..lineTo(c.dx - s * .44, c.dy + s * .22)
            ..close(),
          stroke,
        );
    }
  }

  @override
  bool shouldRepaint(_ShapeMark old) =>
      old.shape != shape || old.color != color;
}

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

/// One line that keeps the whole name: the type shrinks to the tile, down to
/// the micro size, and only past that does the tail get cut.
/// A name at full size. One that does not fit is cut at the edge, and while
/// the pointer rests on it the name slides left until its end shows, then
/// slides back — so nothing is ever shrunk to fit.
class _FittedName extends StatefulWidget {
  const _FittedName(
    this.name,
    this.width, {
    this.size = EditorMetrics.font,
    this.sliding,
  });
  final String name;
  final double width;
  final double size;

  /// When the parent already knows the pointer is on the card, it drives the
  /// slide; otherwise the name watches for the pointer itself.
  final bool? sliding;

  /// The laid-out width of a name at a size. Names repeat across tiles and
  /// survive rebuilds, so lay each one out once.
  static final _natural = <String, double>{};
  static double widthOf(String name, double size) {
    if (_natural.length > 4096) _natural.clear();
    return _natural['$size:$name'] ??= () {
      final painter = TextPainter(
        text: TextSpan(text: name, style: TextStyle(fontSize: size)),
        maxLines: 1,
        textDirection: TextDirection.ltr,
      )..layout();
      final width = painter.width;
      painter.dispose();
      return width;
    }();
  }

  @override
  State<_FittedName> createState() => _FittedNameState();
}

class _FittedNameState extends State<_FittedName>
    with SingleTickerProviderStateMixin {
  late final _slide = AnimationController(vsync: this);

  double get _overflow =>
      math.max(0, _FittedName.widthOf(widget.name, widget.size) - widget.width);

  @override
  void initState() {
    super.initState();
    if (widget.sliding == true) _enter();
  }

  @override
  void didUpdateWidget(_FittedName old) {
    super.didUpdateWidget(old);
    if (widget.sliding != old.sliding) {
      if (widget.sliding == true) _enter();
      if (widget.sliding == false) _leave();
    }
  }

  @override
  void dispose() {
    _slide.dispose();
    super.dispose();
  }

  void _enter() {
    if (_overflow <= 0) return;
    // A steady reading pace: forty pixels a second, at least half a second.
    _slide.duration = Duration(
      milliseconds: math.max(500, (_overflow / 40 * 1000).round()),
    );
    _slide.forward();
  }

  void _leave() => _slide.reverse();

  @override
  Widget build(BuildContext context) {
    final overflow = _overflow;
    final text = Text(
      widget.name,
      maxLines: 1,
      softWrap: false,
      overflow: TextOverflow.visible,
      style: TextStyle(color: EditorTheme.ink, fontSize: widget.size),
    );
    if (overflow <= 0) return text;
    return MouseRegion(
      onEnter: widget.sliding == null ? (_) => _enter() : null,
      onExit: widget.sliding == null ? (_) => _leave() : null,
      child: ClipRect(
        child: AnimatedBuilder(
          animation: _slide,
          builder: (context, child) => Transform.translate(
            offset: Offset(
              -overflow * Curves.easeInOut.transform(_slide.value),
              0,
            ),
            child: child,
          ),
          child: OverflowBox(
            alignment: Alignment.centerLeft,
            maxWidth: double.infinity,
            child: text,
          ),
        ),
      ),
    );
  }
}

/// Tells its child whether the pointer rests on it.
class _Hover extends StatefulWidget {
  const _Hover({required this.builder});
  final Widget Function(bool hovered) builder;
  @override
  State<_Hover> createState() => _HoverState();
}

class _HoverState extends State<_Hover> {
  bool hovered = false;
  @override
  Widget build(BuildContext context) => MouseRegion(
    onEnter: (_) => setState(() => hovered = true),
    onExit: (_) => setState(() => hovered = false),
    child: widget.builder(hovered),
  );
}
