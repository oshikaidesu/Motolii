// The Stage seat: header with view controls, the tool column, the viewport (stage_view.dart) and the tool options strip.
// The picture is the work (stage_art.dart); the chrome around it stays grey and round, with the accent only on what is on.
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../parts/glyphs.dart';
import '../tokens.dart';
import 'stage_art.dart';
import 'stage_trial.dart';
import 'stage_view.dart';
import 'ws.dart';

class WsStage extends StatelessWidget {
  const WsStage({super.key});
  @override
  Widget build(BuildContext context) => const WsSeat(child: _Stage());
}

const _tools = [
  (G.select, 'Select', 'V'),
  (G.move, 'Move', 'M'),
  (G.rect, 'Rectangle', 'R'),
  (G.ellipse, 'Ellipse', 'E'),
  (G.pen, 'Pen', 'P'),
  (G.text, 'Text', 'T'),
];

class _Stage extends StatefulWidget {
  const _Stage();
  @override
  State<_Stage> createState() => _StageState();
}

class _StageState extends State<_Stage> {
  int _zoom = 0, _tool = 0;
  bool _safe = true, _grid = false, _camera = true;
  final _opts = <String, int>{};
  final _cursor = ValueNotifier<Offset?>(null);
  final _focus = FocusNode(debugLabel: 'stage');

  @override
  void dispose() {
    _cursor.dispose();
    _focus.dispose();
    super.dispose();
  }

  KeyEventResult _key(FocusNode n, KeyEvent e) {
    if (e is! KeyDownEvent || HardwareKeyboard.instance.isMetaPressed || HardwareKeyboard.instance.isControlPressed) return KeyEventResult.ignored;
    final ch = e.character?.toUpperCase();
    final i = _tools.indexWhere((t) => t.$3 == ch);
    if (i >= 0) {
      setState(() => _tool = i);
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  @override
  Widget build(BuildContext context) {
    final ws = WsScope.of(context), scene = ArtScene.of(ws, camera: _camera);
    return Focus(
      focusNode: _focus,
      onKeyEvent: _key,
      child: Column(
        children: [
          WsHeader(
            '',
            trailing: [
              _Seg(items: stageZooms, index: _zoom, onChanged: (i) => setState(() => _zoom = i)),
              const SizedBox(width: 6),
              _Toggle(label: 'Safe', icon: _Icon.safe, on: _safe, onTap: () => setState(() => _safe = !_safe)),
              const SizedBox(width: WsT.gap),
              _Toggle(label: 'Grid', icon: _Icon.grid, on: _grid, onTap: () => setState(() => _grid = !_grid)),
              const SizedBox(width: 6),
              _Toggle(label: _camera ? 'Camera' : 'Free', glyph: G.camera, on: _camera, onTap: () => setState(() => _camera = !_camera)),
            ],
          ),
          Expanded(
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                _ToolColumn(active: _tool, onChanged: (i) => setState(() => _tool = i)),
                Expanded(
                  child: Stack(
                    fit: StackFit.expand,
                    children: [
                      StageView(scene: scene, selected: ws.selected, zoom: _zoom, grid: _grid, safe: _safe, cursor: _cursor, onPoke: _focus.requestFocus),
                      if (ws.trial != null) const Positioned(bottom: 44, left: 0, right: 0, child: Center(child: StageTrialBar())),
                    ],
                  ),
                ),
              ],
            ),
          ),
          _OptionsStrip(tool: _tool, opts: _opts, onOpt: (k, v) => setState(() => _opts[k] = v), scene: scene, layer: ws.layer),
        ],
      ),
    );
  }
}

// ---- small round things you touch --------------------------------------------------------------------------------------

class _Hov extends StatefulWidget {
  const _Hov({required this.builder, this.onTap});
  final Widget Function(BuildContext, bool) builder;
  final VoidCallback? onTap;
  @override
  State<_Hov> createState() => _HovState();
}

class _HovState extends State<_Hov> {
  bool _h = false;
  @override
  Widget build(BuildContext context) => MouseRegion(
    cursor: widget.onTap == null ? MouseCursor.defer : SystemMouseCursors.click,
    onEnter: (_) => setState(() => _h = true),
    onExit: (_) => setState(() => _h = false),
    child: GestureDetector(behavior: HitTestBehavior.opaque, onTap: widget.onTap, child: widget.builder(context, _h)),
  );
}

/// A segmented chip: a well with round items, the chosen one filled with the accent.
class _Seg extends StatelessWidget {
  const _Seg({required this.items, required this.index, required this.onChanged});
  final List<String> items;
  final int index;
  final ValueChanged<int> onChanged;
  @override
  Widget build(BuildContext context) => Container(
    height: 20,
    padding: const EdgeInsets.all(2),
    decoration: BoxDecoration(color: WsT.well, borderRadius: BorderRadius.circular(WsT.radius)),
    child: Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        for (var i = 0; i < items.length; i++)
          _Hov(
            onTap: () => onChanged(i),
            builder: (_, h) => Container(
              height: 16,
              padding: const EdgeInsets.symmetric(horizontal: 7),
              alignment: Alignment.center,
              decoration: BoxDecoration(color: i == index ? WsT.accent : (h ? WsT.raised : WsT.well), borderRadius: BorderRadius.circular(WsT.chipRadius)),
              child: Text(items[i], style: T.label(i == index ? WsT.onAccent : (h ? Grey.g95 : Grey.g76)).copyWith(fontWeight: FontWeight.w600)),
            ),
          ),
      ],
    ),
  );
}

enum _Icon { safe, grid }

/// An on/off chip: raised with an accent mark when on, a quiet well when off.
class _Toggle extends StatelessWidget {
  const _Toggle({required this.label, required this.on, required this.onTap, this.icon, this.glyph});
  final String label;
  final bool on;
  final VoidCallback onTap;
  final _Icon? icon;
  final G? glyph;
  @override
  Widget build(BuildContext context) => _Hov(
    onTap: onTap,
    builder: (_, h) {
      final mark = on ? WsT.accent : (h ? Grey.g91 : Grey.g56);
      return Container(
        height: 20,
        padding: const EdgeInsets.only(left: 6, right: 8),
        decoration: BoxDecoration(color: on ? Grey.g26 : (h ? WsT.raised : WsT.well), borderRadius: BorderRadius.circular(WsT.radius)),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            if (glyph case final g?) Glyph(g, size: 14, color: mark) else CustomPaint(size: const Size.square(12), painter: _IconPainter(icon!, mark)),
            const SizedBox(width: 5),
            Text(label, style: T.label(on || h ? Grey.g95 : Grey.g63).copyWith(fontWeight: FontWeight.w600)),
          ],
        ),
      );
    },
  );
}

class _IconPainter extends CustomPainter {
  const _IconPainter(this.icon, this.c);
  final _Icon icon;
  final Color c;
  @override
  void paint(Canvas cv, Size s) {
    final p = Paint()
      ..color = c
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.2;
    final r = (Offset.zero & s).deflate(1);
    switch (icon) {
      case _Icon.safe:
        cv.drawRect(r, p);
        cv.drawRect(r.deflate(2.5), p);
      case _Icon.grid:
        cv.drawRect(r, p);
        for (final t in const [1 / 3, 2 / 3]) {
          cv.drawLine(Offset(r.left + r.width * t, r.top), Offset(r.left + r.width * t, r.bottom), p);
          cv.drawLine(Offset(r.left, r.top + r.height * t), Offset(r.right, r.top + r.height * t), p);
        }
    }
  }

  @override
  bool shouldRepaint(_IconPainter o) => o.icon != icon || o.c != c;
}

// ---- tool column -------------------------------------------------------------------------------------------------------

class _ToolColumn extends StatelessWidget {
  const _ToolColumn({required this.active, required this.onChanged});
  final int active;
  final ValueChanged<int> onChanged;
  @override
  Widget build(BuildContext context) => Container(
    width: WsT.rail,
    decoration: BoxDecoration(
      color: WsT.body,
      border: Border(right: BorderSide(color: WsT.line)),
    ),
    child: ClipRect(
      child: OverflowBox(
        alignment: Alignment.topCenter,
        maxHeight: double.infinity,
        child: Padding(
          padding: const EdgeInsets.symmetric(vertical: 6),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              for (var i = 0; i < _tools.length; i++) ...[
                if (i == 2 || i == 4) Container(width: 20, height: 1, margin: const EdgeInsets.symmetric(vertical: 5), color: WsT.line),
                if (i != 0 && i != 2 && i != 4) const SizedBox(height: 3),
                _ToolButton(glyph: _tools[i].$1, on: i == active, onTap: () => onChanged(i)),
              ],
              Container(width: 20, height: 1, margin: const EdgeInsets.symmetric(vertical: 8), color: WsT.line),
              const _Swatches(),
            ],
          ),
        ),
      ),
    ),
  );
}

class _ToolButton extends StatelessWidget {
  const _ToolButton({required this.glyph, required this.on, required this.onTap});
  final G glyph;
  final bool on;
  final VoidCallback onTap;
  @override
  Widget build(BuildContext context) => _Hov(
    onTap: onTap,
    builder: (_, h) => Container(
      width: 28,
      height: 28,
      alignment: Alignment.center,
      decoration: BoxDecoration(color: on ? WsT.accent : (h ? WsT.raised : WsT.body), borderRadius: BorderRadius.circular(WsT.radius)),
      child: Glyph(glyph, size: 16, color: on ? WsT.onAccent : (h ? Grey.g95 : Grey.g63)),
    ),
  );
}

/// Fill and stroke inks of the drawing tools, Photoshop style: fill in front, stroke behind.
class _Swatches extends StatelessWidget {
  const _Swatches();
  @override
  Widget build(BuildContext context) => SizedBox(
    width: 26,
    height: 26,
    child: Stack(
      children: [
        Positioned(
          right: 0,
          bottom: 0,
          child: Container(
            width: 16,
            height: 16,
            decoration: BoxDecoration(
              color: WsT.body,
              borderRadius: BorderRadius.circular(WsT.chipRadius),
              border: Border.all(color: ArtInk.cream, width: 3),
            ),
          ),
        ),
        Container(
          width: 16,
          height: 16,
          decoration: BoxDecoration(
            color: ArtInk.orange,
            borderRadius: BorderRadius.circular(WsT.chipRadius),
            border: Border.all(color: WsT.body),
          ),
        ),
      ],
    ),
  );
}

// ---- tool options strip ------------------------------------------------------------------------------------------------

class _OptionsStrip extends StatelessWidget {
  const _OptionsStrip({required this.tool, required this.opts, required this.onOpt, required this.scene, required this.layer});
  final int tool;
  final Map<String, int> opts;
  final void Function(String, int) onOpt;
  final ArtScene scene;
  final WsLayer? layer;

  Widget _seg(String key, List<String> items) => _Seg(items: items, index: opts[key] ?? 0, onChanged: (i) => onOpt(key, i));
  Widget _flag(String key, String label, {bool initial = false, G? glyph}) {
    final on = (opts[key] ?? (initial ? 1 : 0)) == 1;
    return _Toggle(label: label, on: on, glyph: glyph, icon: glyph == null ? _Icon.safe : null, onTap: () => onOpt(key, on ? 0 : 1));
  }

  List<Widget> _items() => switch (tool) {
    0 => [
      _seg('sel.level', const ['Layer', 'Group']),
      _flag('sel.snap', 'Snap', initial: true),
      _flag('sel.pivot', 'Pivots'),
    ],
    1 => [
      _seg('move.axis', const ['Free', 'X', 'Y']),
      _flag('move.snap', 'Snap', initial: true),
    ],
    2 => [
      const _Field('Fill', swatch: ArtInk.orange),
      const _Field('Stroke', value: '0 px'),
      const _Field('Radius', value: '24'),
      _seg('rect.mode', const ['Shape', 'Mask']),
    ],
    3 => [
      const _Field('Fill', swatch: ArtInk.pink),
      const _Field('Stroke', value: '0 px'),
      _seg('ell.from', const ['Corner', 'Centre']),
    ],
    4 => [
      _seg('pen.mode', const ['Path', 'Mask']),
      _flag('pen.close', 'Close path', initial: true),
    ],
    _ => [
      const _Field('Font', value: 'Inter Black'),
      const _Field('Size', value: '410'),
      _seg('text.align', const ['L', 'C', 'R']),
    ],
  };

  @override
  Widget build(BuildContext context) {
    final items = _items();
    return Container(
      height: WsT.header,
      padding: const EdgeInsets.symmetric(horizontal: WsT.inset),
      decoration: BoxDecoration(
        color: WsT.body,
        border: Border(top: BorderSide(color: WsT.line)),
      ),
      child: Row(
        children: [
          Expanded(
            child: ClipRect(
              child: OverflowBox(
                alignment: Alignment.centerLeft,
                maxWidth: double.infinity,
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    SizedBox(
                      width: 62,
                      child: Text(_tools[tool].$2.toUpperCase(), style: T.micro(Grey.g56).copyWith(fontWeight: FontWeight.w700, letterSpacing: 1)),
                    ),
                    for (var i = 0; i < items.length; i++) ...[if (i > 0) const SizedBox(width: 6), items[i]],
                  ],
                ),
              ),
            ),
          ),
          const SizedBox(width: WsT.inset),
          _Readout(scene: scene, layer: layer),
        ],
      ),
    );
  }
}

/// A labelled value in a well: the options strip's mock fields.
class _Field extends StatelessWidget {
  const _Field(this.label, {this.value, this.swatch});
  final String label;
  final String? value;
  final Color? swatch;
  @override
  Widget build(BuildContext context) => Container(
    height: 20,
    padding: const EdgeInsets.symmetric(horizontal: 7),
    decoration: BoxDecoration(color: WsT.well, borderRadius: BorderRadius.circular(WsT.radius)),
    child: Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Text(label, style: T.micro(Grey.g56)),
        const SizedBox(width: 6),
        if (swatch case final s?)
          Container(
            width: 22,
            height: 12,
            decoration: BoxDecoration(color: s, borderRadius: BorderRadius.circular(3)),
          )
        else
          Text(value ?? '', style: T.value(Grey.g91)),
      ],
    ),
  );
}

/// The selected layer at this frame: its hue and name, then position, scale and rotation as the stage sees them.
class _Readout extends StatelessWidget {
  const _Readout({required this.scene, required this.layer});
  final ArtScene scene;
  final WsLayer? layer;
  @override
  Widget build(BuildContext context) {
    final l = layer, box = WsArt.boxOf(scene, l?.id);
    if (l == null) return Text('Nothing selected', style: T.label(Grey.g56));
    Widget pair(String k, String v) => Padding(
      padding: const EdgeInsets.only(left: 10),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(k, style: T.micro(Grey.g56)),
          const SizedBox(width: 4),
          Text(v, style: T.value(Grey.g91)),
        ],
      ),
    );
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Container(
          width: 8,
          height: 8,
          decoration: BoxDecoration(color: wsTone(l.kind), borderRadius: BorderRadius.circular(2)),
        ),
        const SizedBox(width: 6),
        Text(l.name, style: T.name(Grey.g95).copyWith(fontWeight: FontWeight.w600)),
        if (box == null)
          Padding(
            padding: const EdgeInsets.only(left: 10),
            child: Text(l.kind == WsKind.audio ? 'No picture' : 'Not on this frame', style: T.label(Grey.g56)),
          )
        else ...[
          pair('P', '${box.anchor.dx.round()}, ${box.anchor.dy.round()}'),
          pair('S', '${(box.scale * 100).round()}%'),
          pair('R', '${(box.rot * 180 / 3.141592653589793).round()}°'),
        ],
      ],
    );
  }
}
