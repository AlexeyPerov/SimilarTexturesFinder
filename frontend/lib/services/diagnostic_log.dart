import 'dart:convert';
import 'dart:io';

import 'package:path/path.dart' as p;
import 'package:path_provider/path_provider.dart';

const _logFileName = 'similar_textures_diagnostic.log';
const _maxLogBytes = 768 * 1024;
const _trimToBytes = 384 * 1024;

Future<File> _logFile() async {
  final dir = await getApplicationSupportDirectory();
  await Directory(dir.path).create(recursive: true);
  return File(p.join(dir.path, _logFileName));
}

String _compactJson(Map<String, Object?>? data) {
  if (data == null || data.isEmpty) return '';
  try {
    return jsonEncode(data);
  } catch (_) {
    return '{encode_error}';
  }
}

Future<void> _rotateIfHuge(File file) async {
  if (!file.existsSync()) return;
  final len = await file.length();
  if (len <= _maxLogBytes) return;
  final text = await file.readAsString();
  final excess = text.length - _trimToBytes;
  final trimmed =
      '--- truncated ${excess > 0 ? excess : 0} chars ---\n${text.substring(text.length - _trimToBytes)}';
  await file.writeAsString(trimmed);
}

/// Append a timestamped line to Application Support log (sandbox-safe).
Future<void> diagnosticLog(
  String message, {
  Map<String, Object?>? data,
  Object? error,
  StackTrace? stackTrace,
}) async {
  try {
    final file = await _logFile();
    await _rotateIfHuge(file);
    final ts = DateTime.now().toUtc().toIso8601String();
    final buf = StringBuffer('$ts $message');
    final j = _compactJson(data);
    if (j.isNotEmpty) buf.write(' $j');
    if (error != null) buf.write(' err=$error');
    if (stackTrace != null) {
      buf.write('\n$stackTrace');
    }
    buf.write('\n');
    await file.writeAsString(buf.toString(), mode: FileMode.append);
  } catch (_) {}
}

/// Best-effort path string for documentation / UI (async).
Future<String> diagnosticLogFilePath() async {
  try {
    final f = await _logFile();
    return f.path;
  } catch (_) {
    return '(application support)/$_logFileName';
  }
}
