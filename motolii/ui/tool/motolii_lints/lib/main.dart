import 'package:analysis_server_plugin/plugin.dart';
import 'package:analysis_server_plugin/registry.dart';

import 'src/raw_dimension.dart';
import 'src/use_metric.dart';

final plugin = MotoliiLints();

class MotoliiLints extends Plugin {
  @override
  String get name => 'motolii_lints';

  @override
  void register(PluginRegistry registry) {
    registry.registerWarningRule(RawDimension());
    registry.registerFixForRule(RawDimension.code, UseMetric.new);
  }
}
