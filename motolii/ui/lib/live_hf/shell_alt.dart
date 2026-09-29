// TEMPORARY — the Skin Swap Proof for the shell. A deliberately different container: no dock, no seats, one pane and
// a row of text tabs; each tab builds the same PanelDef the dock builds. If every panel works here with no change to
// the sessions or the host, the shell is only where panels sit. Delete after the proof.
import 'package:flutter/widgets.dart';

import '../hf/bp/common.dart' show mono;
import '../hf/neutral.dart';
import '../workspace/dock_workspace.dart';

final altShellSkin = ValueNotifier(false);

class AltShell extends StatefulWidget {
  const AltShell({super.key, required this.defs});
  final Map<String, PanelDef> defs;
  @override
  State<AltShell> createState() => _AltShellState();
}

class _AltShellState extends State<AltShell> {
  String _shown = 'Timeline';

  @override
  Widget build(BuildContext context) {
    final def = widget.defs[_shown] ?? widget.defs.values.first;
    return ColoredBox(
      color: N.g00,
      child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        SizedBox(
          height: 22,
          child: Row(children: [
            for (final id in widget.defs.keys)
              GestureDetector(
                key: ValueKey('alt-tab-$id'),
                onTap: () => setState(() => _shown = id),
                child: Text(' ${id == def.id ? '[$id]' : id} ', style: mono(10, c: id == def.id ? N.g100 : N.g56)),
              ),
          ]),
        ),
        Expanded(child: KeyedSubtree(key: ValueKey(def.id), child: def.build())),
      ]),
    );
  }
}
