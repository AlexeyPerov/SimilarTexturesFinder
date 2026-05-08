import 'package:flutter_test/flutter_test.dart';

import 'package:frontend/app_state.dart';
import 'package:frontend/main.dart';

void main() {
  testWidgets('App shell loads', (WidgetTester tester) async {
    final state = AppState();
    await tester.pumpWidget(TextureFinderApp(appState: state));
    await tester.pumpAndSettle();
    expect(find.text('Settings'), findsOneWidget);
  });
}
