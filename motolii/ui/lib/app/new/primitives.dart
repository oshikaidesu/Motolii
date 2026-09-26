import 'package:flutter/widgets.dart';

import '../../foundation/leaves.dart';
import '../../foundation/metrics.dart';
import '../../foundation/shell_tokens.dart';
import '../../foundation/theme.dart';

/// The New face's own parts. Each is drawn whole here; the behaviour it
/// drives comes from the caller.

/// A group's head: the name in the instrument face and a rule to the edge.
class NewGroupHead extends StatelessWidget {
  const NewGroupHead(this.name, {super.key, this.count});
  final String name;
  final int? count;
  @override
  Widget build(BuildContext context) => SizedBox(
    height: ShellTokens.groupHead,
    child: Row(
      children: [
        Text(
          name.toUpperCase(),
          style: ShellTokens.kickerStyle(ShellTokens.ink),
        ),
        if (count != null) ...[
          const SizedBox(width: EditorMetrics.s6),
          Text('$count', style: ShellTokens.kickerStyle(ShellTokens.inkFaint)),
        ],
        const SizedBox(width: EditorMetrics.s8),
        const Expanded(
          child: SizedBox(
            height: ShellTokens.ruleWidth,
            child: ColoredBox(color: ShellTokens.rule),
          ),
        ),
      ],
    ),
  );
}

/// One cell of a shelf: a flat ruled key with its mark above its name.
/// Picked, it is outlined in ink; under the pointer, it lifts one step.
class NewTile extends StatefulWidget {
  const NewTile({
    super.key,
    required this.name,
    required this.mark,
    required this.picked,
    this.picture = false,
    this.accent,
    this.badge,
  });
  final String name;
  final Widget mark;
  final bool picked, picture;
  final Color? accent;
  final String? badge;
  @override
  State<NewTile> createState() => _NewTileState();
}

class _NewTileState extends State<NewTile> {
  bool _hover = false;
  @override
  Widget build(BuildContext context) => MouseRegion(
    onEnter: (_) => setState(() => _hover = true),
    onExit: (_) => setState(() => _hover = false),
    child: Container(
      decoration: BoxDecoration(
        color: _hover ? ShellTokens.hover : ShellTokens.raised,
        border: Border.all(
          color: widget.picked ? ShellTokens.ink : ShellTokens.rule,
          width: widget.picked ? ShellTokens.activeRule : ShellTokens.ruleWidth,
        ),
      ),
      child: Stack(
        fit: StackFit.expand,
        children: [
          if (widget.accent != null)
            Positioned(
              left: 0,
              top: 0,
              bottom: 0,
              width: EditorMetrics.s3,
              child: ColoredBox(color: widget.accent!),
            ),
          Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Expanded(
                child: widget.picture
                    ? ClipRect(child: widget.mark)
                    : Center(child: widget.mark),
              ),
              Container(
                height: EditorMetrics.row,
                padding: const EdgeInsets.symmetric(
                  horizontal: EditorMetrics.s6,
                ),
                color: widget.picture ? ShellTokens.ground : null,
                alignment: Alignment.centerLeft,
                child: Row(
                  children: [
                    Expanded(
                      child: Text(
                        widget.name,
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: const TextStyle(
                          fontSize: EditorMetrics.font,
                          color: ShellTokens.ink,
                        ),
                      ),
                    ),
                    if (widget.badge != null && widget.badge!.isNotEmpty)
                      Text(
                        widget.badge!.toUpperCase(),
                        style: ShellTokens.kickerStyle(ShellTokens.inkMuted),
                      ),
                  ],
                ),
              ),
            ],
          ),
        ],
      ),
    ),
  );
}

/// A search line: a glyph, the text, and a rule under it, nothing boxed.
class NewSearch extends StatelessWidget {
  const NewSearch({
    super.key,
    required this.controller,
    required this.hint,
    required this.onChanged,
    this.focusNode,
  });
  final TextEditingController controller;
  final String hint;
  final ValueChanged<String> onChanged;
  final FocusNode? focusNode;
  @override
  Widget build(BuildContext context) => Container(
    height: ShellTokens.field,
    decoration: const BoxDecoration(
      border: Border(bottom: BorderSide(color: ShellTokens.ruleStrong)),
    ),
    child: Row(
      children: [
        Text('⌕', style: ShellTokens.readoutStyle(ShellTokens.inkMuted)),
        const SizedBox(width: EditorMetrics.s6),
        Expanded(
          child: EditorTextField(
            controller: controller,
            focusNode: focusNode,
            hint: hint,
            cursorWidth: 1,
            cursorColor: EditorTheme.of(context).caret,
            style: ShellTokens.readoutStyle(ShellTokens.ink),
            onChanged: onChanged,
          ),
        ),
      ],
    ),
  );
}
