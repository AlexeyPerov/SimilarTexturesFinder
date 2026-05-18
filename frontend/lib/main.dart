import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';

import 'app/theme/app_theme.dart';
import 'app_state.dart';
import 'screens/results_screen.dart';
import 'screens/scan_screen.dart';
import 'screens/settings_screen.dart';
import 'services/backend_runner.dart';
import 'services/diagnostic_log.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();

  FlutterError.onError = (FlutterErrorDetails details) {
    diagnosticLog(
      'FlutterError',
      data: {'library': details.library ?? ''},
      error: details.exception,
      stackTrace: details.stack,
    );
    FlutterError.presentError(details);
  };

  PlatformDispatcher.instance.onError = (error, stack) {
    diagnosticLog('uncaught_async', error: error, stackTrace: stack);
    return true;
  };

  final appState = AppState();
  await appState.loadUserPreferences();
  runApp(TextureFinderApp(appState: appState));
}

class TextureFinderApp extends StatefulWidget {
  const TextureFinderApp({super.key, required this.appState});

  final AppState appState;

  @override
  State<TextureFinderApp> createState() => _TextureFinderAppState();
}

class _TextureFinderAppState extends State<TextureFinderApp> {
  late final AppLifecycleListener _lifecycle = AppLifecycleListener(
    onDetach: killActiveTextureTool,
  );

  @override
  void dispose() {
    _lifecycle.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AnimatedBuilder(
      animation: widget.appState,
      builder: (context, _) {
        return MaterialApp(
          title: 'Similar Textures',
          theme: AppThemeData.lightThemeData,
          darkTheme: AppThemeData.darkThemeData,
          themeMode: widget.appState.themeMode,
          home: HomeShell(appState: widget.appState),
        );
      },
    );
  }
}

class HomeShell extends StatefulWidget {
  const HomeShell({super.key, required this.appState});

  final AppState appState;

  @override
  State<HomeShell> createState() => _HomeShellState();
}

class _HomeShellState extends State<HomeShell> {
  int _index = 0;

  @override
  Widget build(BuildContext context) {
    const titles = ['Settings', 'Scan', 'Results'];
    return Scaffold(
      appBar: AppBar(title: Text(titles[_index])),
      drawer: Drawer(
        child: ListView(
          padding: EdgeInsets.zero,
          children: [
            DrawerHeader(
              decoration: BoxDecoration(
                color: Theme.of(context).colorScheme.primaryContainer,
              ),
              child: Align(
                alignment: Alignment.bottomLeft,
                child: Text(
                  'Similar Textures',
                  style: Theme.of(context).textTheme.titleLarge,
                ),
              ),
            ),
            ListTile(
              leading: const Icon(Icons.settings),
              title: const Text('Settings'),
              selected: _index == 0,
              onTap: () {
                setState(() => _index = 0);
                Navigator.pop(context);
              },
            ),
            ListTile(
              leading: const Icon(Icons.play_circle_outline),
              title: const Text('Scan'),
              selected: _index == 1,
              onTap: () {
                setState(() => _index = 1);
                Navigator.pop(context);
              },
            ),
            ListTile(
              leading: const Icon(Icons.grid_on),
              title: const Text('Results'),
              selected: _index == 2,
              onTap: () {
                setState(() => _index = 2);
                Navigator.pop(context);
              },
            ),
          ],
        ),
      ),
      body: IndexedStack(
        index: _index,
        children: [
          SettingsScreen(appState: widget.appState),
          ScanScreen(appState: widget.appState),
          ResultsScreen(appState: widget.appState),
        ],
      ),
    );
  }
}
