import 'package:flutter/material.dart';
import 'screens/game_screen.dart';
import 'syn_game.dart';

void main() {
  final synGame = SynGame();
  runApp(
    MaterialApp(
      debugShowCheckedModeBanner: false,
      home: GameScreen(synGame: synGame),
    ),
  );
}
