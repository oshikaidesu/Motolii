import 'package:flutter/widgets.dart';

import '../../foundation/leaves.dart';
import '../../foundation/metrics.dart';
import '../../foundation/shell_tokens.dart';
import '../../foundation/theme.dart';

/// The New face's own parts. Each is drawn whole here; the behaviour it
/// drives comes from the caller.


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
