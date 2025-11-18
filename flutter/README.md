# SYN – Flutter / Flame Frontend

This package is the **Flutter + Flame** frontend for  
**SYN: Simulate Your Narrative** — a synthwave life-sim / narrative RPG.

The Flutter side is responsible *only* for:

- Rendering the UI (Flame `Component` tree, Canvas-driven)
- Handling input (keyboard / mouse / gamepad)
- Animations, transitions, overlays, and audio hooks
- Talking to the Rust simulation core via `flutter_rust_bridge` (FRB)

All simulation logic, stats, AI, and world state live in Rust. Flutter/Flame is the *view and controller*, not the model.

---

## 1. Tech Stack

- **Flutter** 3.x (Impeller renderer)
- **Flame** 1.x (pure component tree – no Flutter widgets in gameplay)
- **flutter_rust_bridge (FRB)** – async bridge to Rust ECS core
- **Canvas / Sprite rendering** – HUD & event cards
- **SQLite** (accessed indirectly via Rust)

Key rules for this frontend:

- Gameplay screens = **Flame components only**
- Flutter widgets are allowed only in **overlays** (pause menu, settings, debug)
- All HUD elements are **PositionComponents** with custom `render()` logic
- All movement/opacity/scale are **Flame Effects**, not Flutter animations

---

## 2. High-Level Architecture

```text
Flutter app
 ├─ GameWidget<SynGame>        # root Flame game
 │   └─ SynGame                # Flame Game subclass
 │       ├─ GameScreenComponent
 │       │   ├─ TopBarComponent
 │       │   ├─ StatPanelComponent
 │       │   ├─ EventCardComponent
 │       │   ├─ RelationshipPanelComponent
 │       │   ├─ QuickMenuBarComponent
 │       └─ Overlays (Flutter) # pause, settings, debug HUD
 └─ FRB bindings               # generated bridge to Rust core
