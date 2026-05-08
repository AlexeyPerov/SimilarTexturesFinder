import 'dart:convert';
import 'dart:io';

import 'package:path/path.dart' as p;
import 'package:path_provider/path_provider.dart';

import '../models/config.dart';

Future<File> writeConfigToTempFile(AnalysisConfig config) async {
  final dir = await getTemporaryDirectory();
  await Directory(dir.path).create(recursive: true);
  final file = File(
    p.join(
      dir.path,
      'texture_tool_config_${DateTime.now().millisecondsSinceEpoch}.json',
    ),
  );
  await file.writeAsString(
    '${JsonEncoder.withIndent('  ').convert(config.toJson())}\n',
  );
  return file;
}
