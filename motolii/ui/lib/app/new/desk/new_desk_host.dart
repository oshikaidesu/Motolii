import 'package:flutter/widgets.dart';

import '../../../hf/bp/common.dart';
import '../../../hf/desk/common.dart' show kAccent;
import '../../../hf/glyphs.dart';
import '../../../session/editor_session.dart';
import 'desk_faces.dart';
import 'desk_host_controller.dart';

/// The Desk's own host, New-owned: which drawer is shown (Classic's own follow-the-selection rule, in
/// [DeskHostController]), and the empty "Tools" catalog when none is. Every drawer it can show — Depth, Ease,
/// Blend, History — already has a New face ([newDeskFace]); this host never falls back to Classic's own.
class NewDeskHost extends StatefulWidget {
  const NewDeskHost({super.key, required this.controller});
  final EditorSession controller;
  @override
  State<NewDeskHost> createState() => _NewDeskHostState();
}

class _NewDeskHostState extends State<NewDeskHost> {
  EditorSession get c => widget.controller;
  late final host = DeskHostController(c);

  @override
  void dispose() {
    host.dispose();
    super.dispose();
  }

  Widget _catalog() => ListenableBuilder(
        listenable: host,
        builder: (context, _) => ListView(
          key: const ValueKey('desk-catalog'),
          padding: const EdgeInsets.all(6),
          children: [
            for (final spec in host.catalog)
              GestureDetector(
                key: ValueKey('desk-catalog-${spec.name}'),
                behavior: HitTestBehavior.opaque,
                onTap: () => host.open(spec.name),
                child: Container(
                  height: 26,
                  margin: const EdgeInsets.only(bottom: 2),
                  padding: const EdgeInsets.symmetric(horizontal: 6),
                  child: Row(children: [
                    Icon(spec.icon, size: 14, color: kMuted),
                    const SizedBox(width: 8),
                    Expanded(child: Text(spec.name, style: sans(11, c: const Color(0xFFD0D1D5)))),
                    GestureDetector(
                      key: ValueKey('desk-star-${spec.name}'),
                      behavior: HitTestBehavior.opaque,
                      onTap: () => host.star(spec.name),
                      child: Padding(
                        padding: const EdgeInsets.all(4),
                        child: SizedBox(width: 12, height: 12, child: CustomPaint(painter: HgPainter(HG.star, host.starred == spec.name ? kAccent : kMuted, kGround))),
                      ),
                    ),
                  ]),
                ),
              ),
          ],
        ),
      );

  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: host,
        builder: (context, _) {
          final shown = host.shown;
          final live = shown != 'Tools';
          return Container(
            color: kGround,
            child: Column(children: [
              SizedBox(
                height: 26,
                child: Row(children: [
                  GestureDetector(
                    key: const ValueKey('desk-tools'),
                    behavior: HitTestBehavior.opaque,
                    onTap: host.toolsCatalog,
                    child: Padding(padding: const EdgeInsets.all(6), child: SizedBox(width: 14, height: 14, child: CustomPaint(painter: HgPainter(HG.grid4, kMuted, kGround)))),
                  ),
                  Text(live ? shown : 'Tools', style: sans(11, c: const Color(0xFFD0D1D5), w: FontWeight.w600)),
                ]),
              ),
              Expanded(child: live ? (newDeskFace(shown, c, ValueKey('desk-drawer-$shown')) ?? const SizedBox.shrink()) : _catalog()),
            ]),
          );
        },
      );
}
