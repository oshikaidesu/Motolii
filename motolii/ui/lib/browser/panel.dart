import 'dart:typed_data';

import 'package:flutter/widgets.dart';

import 'face.dart';
import 'parts.dart' show sans;
import '../effects/shelf.dart';
import 'create/faces.dart' show QuietFace;
import '../hf/shell/menu.dart' show showHfMenu;
import 'seat.dart';
import 'things.dart';
import '../session/editor_session.dart';
import 'session.dart';
import 'media/seat.dart';
import '../colors/instrument.dart';
import '../hf/metrics.dart' show Dn, Surface;

/// The Browser as a skin over [BrowserSession]: it lays out the reference faces (browserFace), dresses tiles with what
/// they do, and hands every gesture to the session.
class LiveBrowser extends StatefulWidget {
  const LiveBrowser({super.key, required this.c, required this.scene, this.fixedTab});
  final EditorSession c;
  final EffectScene scene;
  /// When set, this Browser surface is one real dock panel instead of an internal tab stack.
  final int? fixedTab;
  @override
  State<LiveBrowser> createState() => _LiveBrowserState();
}

class _LiveBrowserState extends State<LiveBrowser> {
  EditorSession get c => widget.c;
  late final s = BrowserSession.of(c);
  late int tab = widget.fixedTab ?? 0;
  late final seat = _LiveSeat(s, widget.scene);

  @override
  void initState() {
    super.initState();
    s.addListener(_changed);
  }

  void _changed() {
    if (mounted) setState(() {});
  }

  @override
  void dispose() {
    s.removeListener(_changed);
    seat.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final name = const ['Create', 'Effects', 'Colors', 'Fonts', 'Media'][tab];
    seat.shelf = name;
    final fonts = s.fonts;
    return BrowserSeatScope(
      seat: seat,
      child: browserFace(
        BrowserModel(
          catalog: s.catalog,
          tab: tab,
          tabs: widget.fixedTab == null ? browserTabs : [browserTabs[tab]],
          user: s.user(name).views,
          scene: widget.scene,
          colors: s.colors,
          gradients: s.gradients,
          onColor: s.applyColor,
          onColorMenu: (swatch, at) async {
            final saved = swatch.$1.startsWith('saved:');
            // the swatch as it is now, before the menu waits (the list may change while it is open)
            final item = saved ? s.savedSwatches.elementAtOrNull(int.parse(swatch.$1.split(':').last)) : null;
            final chosen = await showHfMenu<String>(context, Rect.fromLTWH(at.dx, at.dy, 180, 0), [('apply', 'Apply'), if (saved) ('forget', 'Forget swatch')]);
            if (chosen == 'apply') await s.applyColor(swatch);
            if (chosen == 'forget' && item != null) await s.forget(item);
          },
          usedFonts: s.usedFonts,
          onGradient: s.applyGradient,
          onGradientMenu: (gradient, at) async {
            final chosen = await showHfMenu<String>(context, Rect.fromLTWH(at.dx, at.dy, 180, 0), [('apply', 'Apply'), ('forget', 'Forget swatch')]);
            if (chosen == 'apply') await s.applyGradient(gradient);
            if (chosen == 'forget') await s.forget(gradient);
          },
          colorEditor: (wheel) => LiveColorInstrument(c: c, wheel: wheel),
          currentColor: s.currentColor,
          fonts: fonts,
          fontGroups: s.fontGroups(fonts),
          selectedFont: s.dressingFont,
          onFont: (font) => s.applyFont(font, create: false),
          onFontCreate: (font) => s.applyFont(font, create: s.dressing == null),
          onFontFavorite: s.toggleFavoriteFont,
          fontController: c,
          media: tab == 4 ? MediaSeat(c: c) : null,
          onTab: widget.fixedTab == null ? (i) => setState(() => tab = i) : null,
        ),
      ),
    );
  }
}

class _LiveSeat extends ChangeNotifier implements BrowserSeat {
  _LiveSeat(this.s, this.scene) {
    s.addListener(notifyListeners);
  }
  final BrowserSession s;
  final EffectScene scene;
  EditorSession get c => s.c;

  /// The shelf this seat shows (Create, Effects, Colors, Fonts, Media).
  String shelf = 'Create';
  bool get effectsTab => shelf == 'Effects';
  bool get colorsTab => shelf == 'Colors';
  bool get fontsTab => shelf == 'Fonts';
  final _things = <Thing>[];

  @override
  UserViews? get user => s.user(shelf).views;

  @override
  Widget tile(BuildContext context, Thing thing, Widget tile) {
    if (s.bindings[thing.id] == null) return Opacity(opacity: .48, child: tile);
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: () => s.use(shelf, thing.id),
        onSecondaryTapDown: (event) {
          s.picked[shelf] = thing.id;
          more(context, event.globalPosition);
        },
        child: tile,
      ),
    );
  }

  @override
  Widget? face(BuildContext context, Thing thing) {
    if (!s.snapshots.contains(thing.id)) return null;
    return FutureBuilder<Uint8List?>(
      future: s.picture(thing.id),
      builder: (_, pic) => pic.data == null
          ? s.fixtureFallback.contains(thing.id)
                ? CustomPaint(
                    painter: FxPainter('${thing.face['base']}', scene),
                  )
                // an effect that ships no snapshot: a quiet face (its name is the caption under the tile)
                : const QuietFace()
          // over the quiet face: a snapshot that is empty (an effect with nothing to show alone) still reads as a tile
          : ClipRRect(borderRadius: BorderRadius.circular(2), child: Stack(fit: StackFit.expand, children: [const QuietFace(), Image.memory(pic.data!, fit: BoxFit.cover, gaplessPlayback: true)])),
    );
  }

  @override
  ({double column, double extent, double gap, double padding})? tiling(
    BuildContext context,
    double width,
  ) => null;
  @override
  double get tileScale => 1;
  @override
  Widget? tools(BuildContext context) => null;
  @override
  Widget? header(BuildContext context) {
    if (!fontsTab) return null;
    final t = s.dressing;
    // Classic BR-117 / BR-116: what is being dressed, else how to start
    final line = t == null
        ? 'Double-click a face to add a text layer'
        : '${t['name'] ?? 'Text'} · ${EditorSession.map(t['text'])['content'] ?? ''}'.trim();
    return Padding(
      padding: const EdgeInsets.fromLTRB(9, 4.5, 9, 1.5),
      child: Text(line, key: const ValueKey('fonts-header'), maxLines: 1, overflow: TextOverflow.ellipsis, style: sans(Dn.nameSize, c: Surface.muted)),
    );
  }
  @override
  Widget? editor(BuildContext context) => null;
  @override
  KeyEventResult key(FocusNode node, KeyEvent event) => KeyEventResult.ignored;
  @override
  void shows(List<Thing> things, int columns) {
    _things
      ..clear()
      ..addAll(things);
  }

  @override
  void more(BuildContext context, Offset at) {
    final id = s.picked[shelf];
    final reload = effectsTab && s.canReloadEffects;
    if (id == null && !reload && !colorsTab) return;
    final name = id == null ? null : _things.where((t) => t.id == id).map((t) => t.name).firstOrNull;
    final binding = id == null ? null : s.bindings[id];
    showHfMenu<String>(context, Rect.fromLTWH(at.dx, at.dy, 220, 0), [
      // Classic's tile menu: its name, Apply, then the collections (Favorites is the first)
      if (name != null) ('title', name),
      if (binding != null) ('apply', 'Apply'),
      if (id != null) ...s.user(shelf).collectionLines(id),
      if (reload) ('reload', 'Reload effects'),
      if (colorsTab) ...[
        ('saveColor', 'Save current color'),
        ('palette', 'Palette from image…'),
        s.wheelShape == 'triangle' ? ('square', 'Square wheel') : ('triangle', 'Triangle wheel'),
      ],
    ], info: {'title'}, disabled: {
      if (colorsTab && !s.canSaveColor) 'saveColor',
    }, dividers: {
      if (name != null) 'title',
      if (binding != null) 'apply',
    }).then((action) {
      if (action == 'square' || action == 'triangle') {
        s.setWheelShape(action!);
      } else if (action == 'saveColor') {
        s.saveCurrentColor();
      } else if (action == 'palette') {
        s.paletteFromImage();
      } else if (action == 'reload') {
        s.reloadEffects();
      } else if (action == 'apply' && id != null) {
        s.use(shelf, id);
      } else if (action != null && action.startsWith('collect:') && id != null) {
        s.user(shelf).collect([id], int.parse(action.substring(8)));
      }
    });
  }

  @override
  Listenable get changes => this;

  @override
  void dispose() {
    s.removeListener(notifyListeners);
    super.dispose();
  }
}
