import 'package:analysis_server_plugin/plugin.dart';
import 'package:analysis_server_plugin/registry.dart';

import 'src/material_import.dart';
import 'src/raw_color.dart';
import 'src/raw_dimension.dart';
import 'src/use_metric.dart';

final plugin = MotoliiLints();

class MotoliiLints extends Plugin {
  @override
  String get name => 'motolii_lints';

  @override
  void register(PluginRegistry registry) {
    registry.registerWarningRule(RawDimension());
    registry.registerWarningRule(RawColor());
    registry.registerWarningRule(MaterialImport());
    registry.registerFixForRule(RawDimension.code, UseMetric.new);
  }
}
