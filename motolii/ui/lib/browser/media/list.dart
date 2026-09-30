import 'package:flutter/widgets.dart';

import '../../theme/metrics.dart';
import '../../theme/neutral.dart';

/// The columns' titles: a click asks the owner for that order (an arrow shows the one in force).
class MediaListHeader extends StatelessWidget {
  const MediaListHeader({super.key, required this.width, required this.sort, required this.descending, required this.onSort});
  final double width;
  final String sort;
  final bool descending;
  final ValueChanged<String> onSort;

  static bool wide(double w) => w >= 300;
  static bool roomy(double w) => w >= 420;

  @override
  Widget build(BuildContext context) {
    Widget header(String label, String? key, {double? width, TextAlign align = TextAlign.left}) {
      final on = key != null && key == sort;
      final text = Text(on ? '$label ${descending ? '↓' : '↑'}' : label, softWrap: false, overflow: TextOverflow.clip, textAlign: align, style: Dn.micro(on ? N.g95 : N.g56));
      Widget cell = width == null ? Expanded(child: text) : SizedBox(width: width, child: text);
      if (key != null) cell = width == null ? Expanded(child: GestureDetector(behavior: HitTestBehavior.opaque, onTap: () => onSort(key), child: text)) : GestureDetector(behavior: HitTestBehavior.opaque, onTap: () => onSort(key), child: cell);
      return cell;
    }

    return Container(
      height: 20,
      padding: const EdgeInsets.symmetric(horizontal: 8),
      decoration: const BoxDecoration(border: Border(bottom: BorderSide(color: N.g15))),
      child: Row(children: [
        const SizedBox(width: 54),
        header('NAME', 'name'),
        header('TYPE', 'kind', width: 42),
        header('LENGTH', null, width: 40),
        if (wide(width)) header('SIZE', 'size', width: 54, align: TextAlign.right),
        if (roomy(width)) header('DATE', 'modified', width: 74, align: TextAlign.right),
      ]),
    );
  }
}
