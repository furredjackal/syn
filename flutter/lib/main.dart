import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'models/game_state.dart';
import 'theme/theme.dart';
import 'screens/splash_screen.dart';
import 'screens/main_menu_screen.dart';
import 'screens/character_creation_screen.dart';
import 'screens/game_screen.dart';
import 'screens/memory_journal_screen.dart';
import 'screens/settings_screen.dart';
import 'screens/detailed_stat_screen.dart';
import 'screens/relationship_network_screen.dart';
import 'screens/inventory_screen.dart';
import 'screens/world_map_screen.dart';
import 'screens/save_load_screen.dart';
import 'screens/end_of_life_screen.dart';
import 'screens/debug_console_screen.dart';

void main() {
  runApp(
    MultiProvider(
      providers: [
        ChangeNotifierProvider(create: (_) => GameState()),
      ],
      child: const SynApp(),
    ),
  );
}

class SynApp extends StatelessWidget {
  const SynApp({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'SYN: Simulate Your Narrative',
      theme: SynTheme.dark,
      home: const SplashScreen(),
      routes: {
        '/splash': (context) => const SplashScreen(),
        '/menu': (context) => const MainMenuScreen(),
        '/character_creation': (context) => const CharacterCreationScreen(),
        '/game': (context) => const GameScreen(),
        '/journal': (context) => const MemoryJournalScreen(),
        '/settings': (context) => const SettingsScreen(),
        '/detailed_stats': (context) => const DetailedStatScreen(),
        '/relationships': (context) => const RelationshipNetworkScreen(),
        '/inventory': (context) => const InventoryScreen(),
        '/map': (context) => const WorldMapScreen(),
        '/save_load': (context) => const SaveLoadScreen(),
        '/end_of_life': (context) => const EndOfLifeSummaryScreen(),
        '/debug': (context) => const DebugConsoleScreen(),
      },
      debugShowCheckedModeBanner: false,
    );
  }
}
