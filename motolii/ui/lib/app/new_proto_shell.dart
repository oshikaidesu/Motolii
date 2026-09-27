import 'package:flutter/widgets.dart';

import '../hf/bp/common.dart' show kGround, kMuted, sans;
import '../hf/desk/common.dart' show kInk;
import '../hf/shell/shell_face.dart';
import '../panels/stage.dart';
import '../panels/timeline.dart';
import '../session/editor_session.dart';
import 'new/browser/shelf_panel.dart';
import 'new/inspector/new_transform.dart';

/// First production probe of the proto_hf visual authority: the same [ShellFace] proto_hf's own `Shell` builds,
/// with Motolii Live in every slot instead of fixtures. Root is ShellFace itself, not `NewShell` — the docking
/// package, its tab/header chrome and its persisted workspace are a later, separate step (ui-rebaseline brief
/// STOP conditions do not include "Dock capability isn't wired yet").
///
/// Deliberately small: Browser and the right seat already have session-backed hf faces (`ShelfPanel`,
/// `NewTransform`, following the pattern `SessionTransformStore` set), so they go in as-is. Stage and Timeline
/// are the real Session-backed production widgets already, unstyled (Classic's own bars) — hf chrome for them
/// is a later step, not a blocker for proving the shell itself. Top is the minimum real state (playback) rather
/// than a fixture, so nothing here is fixture-backed.
class ProductionProtoShell extends StatefulWidget {
  const ProductionProtoShell({super.key});
  @override
  State<ProductionProtoShell> createState() => _ProductionProtoShellState();
}

class _ProductionProtoShellState extends State<ProductionProtoShell> {
  final c = EditorSession();
  bool ready = false;

  @override
  void initState() {
    super.initState();
    _initialize();
  }

  Future<void> _initialize() async {
    await c.initialize();
    if (!mounted) return;
    setState(() => ready = true);
  }

  @override
  void dispose() {
    c.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => ColoredBox(
        color: kGround,
        child: !ready
            ? const SizedBox.expand()
            : Center(
                child: FittedBox(
                  fit: BoxFit.scaleDown,
                  child: ShellFace(
                    top: _ProtoTop(c: c),
                    browser: ShelfPanel(controller: c, name: 'Create'),
                    stage: StagePanel(controller: c),
                    right: NewTransform(controller: c),
                    timeline: TimelinePanel(controller: c),
                  ),
                ),
              ),
      );
}

/// The minimum real top: playback, nothing else yet. File/Edit/Composition/Export operations
/// (`NewTopBar`'s job in the current production shell) are a later step, not proto_hf's own visual grammar
/// to imitate — the top slot only needs to prove it is Motolii Live, not to be finished.
class _ProtoTop extends StatelessWidget {
  const _ProtoTop({required this.c});
  final EditorSession c;

  @override
  Widget build(BuildContext context) => DecoratedBox(
        decoration: const BoxDecoration(color: kGround),
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16),
          child: Row(children: [
            Text('Motolii', style: sans(15, c: kInk, w: FontWeight.w600)),
            const SizedBox(width: 16),
            ListenableBuilder(
              listenable: c.playing,
              builder: (context, _) => GestureDetector(
                onTap: c.togglePlayback,
                child: Text(c.playing.value ? 'Pause' : 'Play', style: sans(12, c: kMuted, w: FontWeight.w600)),
              ),
            ),
          ]),
        ),
      );
}
