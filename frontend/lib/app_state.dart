import 'package:flutter/foundation.dart';

import 'models/config.dart';
import 'services/json_parser.dart';
import 'services/user_preferences.dart';

/// Shared app state: analysis settings and last scan outputs.
class AppState extends ChangeNotifier {
  AppState({UserPreferencesStore? preferencesStore})
      : _prefs = preferencesStore ?? UserPreferencesStore();

  final UserPreferencesStore _prefs;

  AnalysisConfig config = AnalysisConfig.defaults();

  /// Optional path to `texture_tool` chosen in Settings (persisted).
  String? textureToolExecutableOverride;

  /// Last scan input folder (persisted); used to pre-fill Scan and folder picker.
  String? lastScanInputFolder;

  /// Path to last written `config.json` for texture_tool (temp file).
  String? lastConfigPath;

  /// Path to last `result.json` from a successful scan.
  String? lastResultPath;

  ScanResult? lastScanResult;

  Future<void> loadUserPreferences() async {
    textureToolExecutableOverride =
        await _prefs.loadTextureToolExecutableOverride();
    lastScanInputFolder = await _prefs.loadLastScanInputFolder();
    notifyListeners();
  }

  Future<void> setTextureToolExecutableOverride(String? path) async {
    textureToolExecutableOverride =
        (path == null || path.isEmpty) ? null : path;
    await _prefs.saveTextureToolExecutableOverride(textureToolExecutableOverride);
    notifyListeners();
  }

  Future<void> setLastScanInputFolder(String? path) async {
    lastScanInputFolder = (path == null || path.isEmpty) ? null : path;
    await _prefs.saveLastScanInputFolder(lastScanInputFolder);
    notifyListeners();
  }

  void setConfig(AnalysisConfig next) {
    config = next;
    notifyListeners();
  }

  void applyConfigFrom(AnalysisConfig Function(AnalysisConfig) fn) {
    config = fn(config.copy());
    notifyListeners();
  }

  void setLastOutputs({
    String? configPath,
    String? resultPath,
    ScanResult? result,
  }) {
    lastConfigPath = configPath ?? lastConfigPath;
    lastResultPath = resultPath ?? lastResultPath;
    lastScanResult = result ?? lastScanResult;
    notifyListeners();
  }
}
