import 'package:flutter/widget_previews.dart';
import 'package:flutter/widgets.dart';

import 'foundation/color_field.dart';
import 'foundation/metrics.dart';
import 'foundation/panel_controls.dart';
import 'foundation/theme.dart';
import 'workspace/layout.dart';
import 'workspace/workspace_view.dart';

void main() => runApp(const GalleryApp());

class GalleryApp extends StatefulWidget {
  const GalleryApp({super.key});

  @override
  State<GalleryApp> createState() => _GalleryAppState();
}

class _GalleryAppState extends State<GalleryApp> {
  late String story = _initialStory();

  static String _initialStory() {
    final requested = Uri.base.queryParameters['story'];
    return galleryStories.any((entry) => entry.id == requested)
        ? requested!
        : galleryStories.first.id;
  }

  @override
  Widget build(BuildContext context) => WidgetsApp(
    debugShowCheckedModeBanner: false,
    color: EditorTheme.app,
    textStyle: EditorTheme.text,
    pageRouteBuilder: <T>(settings, builder) => PageRouteBuilder<T>(
      settings: settings,
      pageBuilder: (context, _, __) => builder(context),
    ),
    builder: (context, child) => galleryPreviewWrapper(child!),
    home: _GalleryHome(
      story: story,
      onStory: (value) => setState(() => story = value),
    ),
  );
}

Widget galleryPreviewWrapper(Widget child) => EditorLook(
  tooltips: false,
  child: ColoredBox(
    color: EditorTheme.app,
    child: DefaultTextStyle(
      style: EditorTheme.text,
      child: IconTheme(data: EditorTheme.icon, child: child),
    ),
  ),
);

class GalleryStory {
  const GalleryStory(this.id, this.group, this.name, this.builder);

  final String id;
  final String group;
  final String name;
  final WidgetBuilder builder;
}

final galleryStories = <GalleryStory>[
  GalleryStory(
    'foundation-controls',
    'Foundation',
    'Controls',
    (_) => const FoundationControlsStory(),
  ),
  GalleryStory(
    'panel-chrome',
    'Panels',
    'Panel chrome',
    (_) => const PanelChromeStory(),
  ),
  GalleryStory(
    'workspace-dock',
    'Workspace',
    'Dock',
    (_) => const WorkspaceDockStory(),
  ),
];

class _GalleryHome extends StatelessWidget {
  const _GalleryHome({required this.story, required this.onStory});

  final String story;
  final ValueChanged<String> onStory;

  @override
  Widget build(BuildContext context) {
    final current = galleryStories.firstWhere((entry) => entry.id == story);
    return ColoredBox(
      color: EditorTheme.app,
      child: Row(
        children: [
          SizedBox(
            width: EditorMetrics.s200,
            child: DecoratedBox(
              decoration: const BoxDecoration(
                color: EditorTheme.panel,
                border: Border(right: BorderSide(color: EditorTheme.line)),
              ),
              child: ListView(
                padding: const EdgeInsets.all(EditorMetrics.s8),
                children: [
                  const Padding(
                    padding: EdgeInsets.all(EditorMetrics.s8),
                    child: Text(
                      'MOTOLII UI',
                      style: TextStyle(
                        fontSize: EditorMetrics.title,
                        fontWeight: FontWeight.w600,
                      ),
                    ),
                  ),
                  for (final group in const [
                    'Foundation',
                    'Panels',
                    'Workspace',
                  ]) ...[
                    Padding(
                      padding: const EdgeInsets.fromLTRB(
                        EditorMetrics.s8,
                        EditorMetrics.s12,
                        EditorMetrics.s8,
                        EditorMetrics.s4,
                      ),
                      child: Text(
                        group,
                        style: const TextStyle(color: EditorTheme.muted),
                      ),
                    ),
                    for (final entry in galleryStories.where(
                      (item) => item.group == group,
                    ))
                      EditorButton(
                        entry.name,
                        () => onStory(entry.id),
                        selected: entry.id == story,
                        tooltip: '?story=${entry.id}',
                      ),
                  ],
                ],
              ),
            ),
          ),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                EditorBar(
                  padding: const EdgeInsets.symmetric(
                    horizontal: EditorMetrics.s8,
                  ),
                  children: [
                    Text('${current.group} / ${current.name}'),
                    const Spacer(),
                    Text(
                      '?story=${current.id}',
                      style: const TextStyle(color: EditorTheme.muted),
                    ),
                  ],
                ),
                Expanded(child: current.builder(context)),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

@Preview(name: 'Controls', group: 'Foundation', wrapper: galleryPreviewWrapper)
Widget foundationControlsPreview() => const FoundationControlsStory();

class FoundationControlsStory extends StatefulWidget {
  const FoundationControlsStory({super.key});

  @override
  State<FoundationControlsStory> createState() =>
      _FoundationControlsStoryState();
}

class _FoundationControlsStoryState extends State<FoundationControlsStory> {
  bool selected = true;
  String draft = '24';

  @override
  Widget build(BuildContext context) => SingleChildScrollView(
    padding: const EdgeInsets.all(EditorMetrics.s16),
    child: Align(
      alignment: Alignment.topLeft,
      child: SizedBox(
        width: EditorMetrics.sheetWide,
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            panelTitle('Controls'),
            EditorBar(
              padding: const EdgeInsets.symmetric(horizontal: EditorMetrics.s4),
              children: [
                EditorButton(
                  'Selected',
                  () => setState(() => selected = !selected),
                  selected: selected,
                ),
                EditorButton('Action', () {}),
                const EditorButton('Disabled', null),
              ],
            ),
            const SizedBox(height: EditorMetrics.s12),
            EditorSection(
              'Fields',
              Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  EditorDraftField(
                    value: draft,
                    label: 'Frame',
                    onCommit: (value) async => setState(() => draft = value),
                  ),
                  const SizedBox(height: EditorMetrics.s8),
                  Row(
                    children: [
                      EditorColorField(
                        value: EditorTheme.spatial,
                        label: 'spatial',
                        onFocus: () {},
                      ),
                      const SizedBox(width: EditorMetrics.s8),
                      EditorColorField(
                        value: EditorTheme.amount,
                        label: 'amount',
                        onFocus: () {},
                      ),
                      const SizedBox(width: EditorMetrics.s8),
                      const EditorColorField(
                        value: EditorTheme.time,
                        label: 'time',
                      ),
                    ],
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    ),
  );
}

@Preview(name: 'Panel chrome', group: 'Panels', wrapper: galleryPreviewWrapper)
Widget panelChromePreview() => const PanelChromeStory();

class PanelChromeStory extends StatefulWidget {
  const PanelChromeStory({super.key});

  @override
  State<PanelChromeStory> createState() => _PanelChromeStoryState();
}

class _PanelChromeStoryState extends State<PanelChromeStory> {
  String x = '640';
  String y = '360';

  @override
  Widget build(BuildContext context) => Center(
    child: SizedBox(
      width: EditorMetrics.s280,
      height: EditorMetrics.sheetWide,
      child: DecoratedBox(
        decoration: BoxDecoration(
          color: EditorTheme.panel,
          border: Border.all(color: EditorTheme.line),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            panelTitle('Inspector'),
            Expanded(
              child: ListView(
                padding: const EdgeInsets.all(EditorMetrics.s8),
                children: [
                  EditorSection(
                    'Transform',
                    Column(
                      children: [
                        _fieldRow('X', x, (value) => setState(() => x = value)),
                        const SizedBox(height: EditorMetrics.s4),
                        _fieldRow('Y', y, (value) => setState(() => y = value)),
                      ],
                    ),
                  ),
                  const SizedBox(height: EditorMetrics.s8),
                  EditorSection(
                    'Appearance',
                    Row(
                      children: [
                        EditorColorField(
                          value: EditorTheme.spatial,
                          label: 'fill',
                          onFocus: () {},
                        ),
                        const SizedBox(width: EditorMetrics.s8),
                        const Expanded(child: Text('Fill')),
                        EditorButton('Reset', () {}),
                      ],
                    ),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    ),
  );

  Widget _fieldRow(String label, String value, ValueChanged<String> onCommit) =>
      Row(
        children: [
          SizedBox(width: EditorMetrics.s32, child: Text(label)),
          Expanded(
            child: EditorDraftField(
              value: value,
              label: label,
              onCommit: (next) async => onCommit(next),
            ),
          ),
        ],
      );
}

@Preview(name: 'Dock', group: 'Workspace', wrapper: galleryPreviewWrapper)
Widget workspaceDockPreview() => const WorkspaceDockStory();

class WorkspaceDockStory extends StatefulWidget {
  const WorkspaceDockStory({super.key});

  @override
  State<WorkspaceDockStory> createState() => _WorkspaceDockStoryState();
}

class _WorkspaceDockStoryState extends State<WorkspaceDockStory> {
  final layout = WorkspaceLayout();

  @override
  void dispose() {
    layout.dispose();
    super.dispose();
  }

  void changed(VoidCallback change) => setState(change);

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.all(EditorMetrics.s8),
    child: DecoratedBox(
      decoration: BoxDecoration(
        color: EditorTheme.panel,
        border: Border.all(color: EditorTheme.line),
      ),
      child: WorkspaceView(
        layout: layout.root,
        panelBuilder: (name) => _WorkspacePlaceholder(name: name),
        onMove: (name, target, edge) =>
            changed(() => layout.move(name, target, edge)),
        onClose: (name) => changed(() => layout.close(name)),
        onDetach: (_) {},
        onLayoutChanged: () => setState(() {}),
      ),
    ),
  );
}

class _WorkspacePlaceholder extends StatelessWidget {
  const _WorkspacePlaceholder({required this.name});

  final String name;

  @override
  Widget build(BuildContext context) => ColoredBox(
    color: name == 'Stage' ? EditorTheme.app : EditorTheme.panel,
    child: Center(
      child: Text(name, style: const TextStyle(color: EditorTheme.muted)),
    ),
  );
}
