// The desk's Color tool: the dock's own Colors shelf (used here, palettes, saved gradients, the swatch field, glass;
// a colour picked, then Fill: to the picked thing, or to where a card's colour link aims, or as the ink of the next
// thing drawn). One shelf, two doors: the desk keeps its own picks.
import 'package:flutter/widgets.dart';

import 'dock_make.dart' show ColorsShelf;
import 'dock_parts.dart' show DockCtx, DockMem;
import 'ws.dart';

class DeskColor extends StatefulWidget {
  const DeskColor({super.key});
  @override
  State<DeskColor> createState() => _DeskColorState();
}

class _DeskColorState extends State<DeskColor> {
  @override
  Widget build(BuildContext context) {
    final ws = WsScope.of(context), mem = ws.view.putIfAbsent('desk.Colors', DockMem.new) as DockMem;
    return ColorsShelf(DockCtx('Colors', mem, '', ws, setState));
  }
}
