import 'package:flutter/material.dart';

import 'models/config.dart';
import 'services/json_parser.dart';
import 'services/user_preferences.dart';

ThemeMode themeModeFromStorage(String? raw) {
  switch (raw) {
    case 'light':
      return ThemeMode.light;
    case 'system':
      return ThemeMode.system;
    case 'dark':
    default:
      return ThemeMode.dark;
  }
}

String themeModeToStorage(ThemeMode mode) {
  switch (mode) {
    case ThemeMode.light:
      return 'light';
    case ThemeMode.dark:
      return 'dark';
    case ThemeMode.system:
      return 'system';
  }
}

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

  ThemeMode themeMode = ThemeMode.dark;

  /// When true, Results hides groups with only one image (persisted, default true).
  bool hideSingletonResultGroups = true;

  /// Path to last written `config.json` for texture_tool (temp file).
  String? lastConfigPath;

  /// Path to last `result.json` from a successful scan.
  String? lastResultPath;

  ScanResult? lastScanResult;

  Future<void> loadUserPreferences() async {
    textureToolExecutableOverride =
        await _prefs.loadTextureToolExecutableOverride();
    lastScanInputFolder = await _prefs.loadLastScanInputFolder();
    final appearance = await _prefs.loadAppearancePreferences();
    themeMode = themeModeFromStorage(appearance.themeModeRaw);
    hideSingletonResultGroups = appearance.hideSingletonGroups;
    notifyListeners();
  }

  Future<void> setThemeMode(ThemeMode mode) async {
    themeMode = mode;
    await _prefs.saveThemeModeRaw(themeModeToStorage(mode));
    notifyListeners();
  }

  Future<void> setHideSingletonResultGroups(bool value) async {
    hideSingletonResultGroups = value;
    await _prefs.saveHideSingletonResultGroups(value);
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
