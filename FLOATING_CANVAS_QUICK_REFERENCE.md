# SYN Floating Canvas UI - Quick Reference

## 🎨 Component Layout

### Screen Coordinates (Vector2 positioning)
```
(0,0) ┌─────────────────────────────────────────────────────────┐
      │                      TopBar                             │
      │                    (height: 60px)                       │
      └─────────────────────────────────────────────────────────┘

      ┌──────────────┐                    ┌──────────────┐
      │ StatPanel    │                    │ Relationship │
      │              │                    │ Panel        │
      │  55 pixels   │       Center       │  x-335       │
      │  from left   │       EventCard    │  from right  │
      │              │                    │              │
      │  (280×280)   │     (60% width)    │  (280×280)   │
      │              │      (65% height)  │              │
      │              │                    │              │
      └──────────────┘                    └──────────────┘

      ┌─────────────────────────────────────────────────────────┐
      │                   QuickMenuBar                          │
      │                  (height: 100px)                        │
      │            (positioned at y - 120)                      │
      └─────────────────────────────────────────────────────────┘
(max_x, max_y)
```

## 📊 StatPanel (280×280px)

**Grid Layout: 3 columns × 2 rows**

```
┌─────────────────────────────────┐
│  ⚪   ⚪   ⚪                   │
│ HP   $   CHR                     │
│ ⚪   ⚪   ⚪                   │
│ INT  WIS  STR                    │
└─────────────────────────────────┘
```

**Ring Colors:**
- HP (Red):        #FF4444
- $ (Green):       #44FF44
- CHR (Cyan):      #00D9FF
- INT (Orange):    #FFAA00
- WIS (Purple):    #DD44FF
- STR (Orange-Red):#FF8844

**Ring Specifications:**
- Each ring: 70×70px
- Ring radius: 28px
- Center spacing: 110px
- Edge padding: 22px

## 🤝 RelationshipPanel (280×280px)

**Display: Up to 3 relationships**

```
┌─────────────────────────────────┐
│ NPC Name    [BAD]  ❤️ 🔗        │
│            Aff: ████  Tru: ███  │
│            F:5  R:2              │
│                                 │
│ NPC Name 2  [GUD]  ❤️ 🔗        │
│            Aff: ████  Tru: ████ │
│            F:8  R:0              │
│                                 │
│ NPC Name 3  [FRI]  ❤️ 🔗        │
│            Aff: ████  Tru: ████ │
│            F:7  R:1              │
└─────────────────────────────────┘
```

**State Badges (3 letters):**
- STR: Stranger (Gray)
- ACQ: Acquaintance (Light Blue)
- FRI: Friend (Green)
- CF+: CloseFriend (Bright Green)
- BF+: BestFriend (Cyan)
- ROM: RomanticInterest (Pink)
- PRT: Partner (Magenta)
- SPO: Spouse (White)
- RIV: Rival (Orange-Red)
- EST: Estranged (Dark Red)
- BH: BrokenHeart (Purple)

## 🎬 EventCard (Center Focal Point)

**Size:** 60% of screen width (clamped 400px min), 65% of screen height (clamped 200px min)

**Visual Elements:**
```
┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃                                 ┃ ← Cyan angled border (#00D9FF)
┃  ╱╱ TITLE BANNER ╲╲            ┃
┃ ╱╱ With Jagged    ╲╲           ┃ ← Gradient bg (cyan→violet)
┃╱╱  Polygon & Glow ╲╲           ┃
┃                                 ┃
┃  Event Description Text         ┃
┃  Multiple lines for full        ┃
┃  context and narrative flavor.  ┃
┃                                 ┃
┃  ┌─────────────────────────────┐┃ ← Choice buttons
┃  │ Choice 1                    │┃   (staggered entrance)
┃  └─────────────────────────────┘┃
┃  ┌─────────────────────────────┐┃
┃  │ Choice 2                    │┃
┃  └─────────────────────────────┘┃
┃                                 ┃
┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛
```

**Animations:**
- **Entrance:** Scale 0.88→1.0 over 0.35s
- **Slash Transition:** 0.4s wipe from right to left (glow + blur edge)
- **Choice Buttons:** Fade/scale 0.2s each
  - Button 1: 0.25s delay
  - Button 2: 0.25s + 0.12s = 0.37s delay
  - Button 3: 0.25s + 0.24s = 0.49s delay
  - Button N: 0.25s + (N × 0.12s) delay

## 🎨 Color Scheme

| Element | Color | Hex | Usage |
|---------|-------|-----|-------|
| Primary Accent | Cyan | #00D9FF | Borders, highlights, focus |
| Dark BG | Black | #000000 | Background (65% opacity) |
| HP Ring | Red | #FF4444 | Health stat |
| Wealth Ring | Green | #44FF44 | Money/Resources stat |
| Charisma Ring | Cyan | #00D9FF | Social/Charm stat |
| INT Ring | Orange | #FFAA00 | Intelligence stat |
| WIS Ring | Purple | #DD44FF | Wisdom/Insight stat |
| STR Ring | Orange-Red | #FF8844 | Strength/Physical stat |
| Affection Gauge | Pink | #FF77DD | Relationship affection |
| Trust Gauge | Green | #77FF77 | Relationship trust |

## ⚙️ Responsive Behavior

**On Screen Resize:**
- EventCard recalculates center position
- StatPanel position: Y = screen.y × 0.30
- RelationshipPanel: X = screen.x - 335
- TopBar stretches: width = screen.x - 80
- QuickMenuBar stretches: width = screen.x - 80

**Fixed Sizes (not responsive):**
- StatPanel: 280×280px
- RelationshipPanel: 280×280px
- TopBar height: 60px
- QuickMenuBar height: 100px

## 📦 Component Hierarchy

```
GameScreenComponent
├── _componentLayer (PositionComponent)
│   ├── TopBarComponent
│   ├── EventCardComponent
│   │   ├── _EventCanvasBackground
│   │   ├── _PersonaEventBorder
│   │   ├── _SlashAccent
│   │   ├── _EventTitleBanner
│   │   ├── _SlashTransition
│   │   └── ChoiceButtonComponent[] (staggered)
│   ├── StatPanelComponent
│   │   ├── _PanelFrame
│   │   └── _StatRing[] (6 rings)
│   ├── RelationshipPanelComponent
│   │   ├── _PanelFrame
│   │   └── _RelationshipRow[] (up to 3)
│   └── QuickMenuBarComponent
└── PersonaBackground (full screen)
```

## 🧪 Test Status

**All Tests Passing:** ✅ 12/12 (00:26 duration)

Test Files:
- integration/ (4 tests)
- screens/ (4 tests)
- widgets/ (4 tests)

**No Type Errors:** ✅ math.max() conversions fixed

## 🚀 Build Status

**Flutter Analyze:**
- Errors: 0 ✅
- Type Errors: 0 ✅
- Warnings: 186 (pre-existing, non-critical)

**Compilation:** ✅ Ready for `flutter run`

## 📝 File Sizes

| File | Lines | Purpose |
|------|-------|---------|
| game_screen_component.dart | 276 | Layout orchestrator |
| event_card_component.dart | 702 | Event display + transitions |
| stat_panel_component.dart | 320 | Stat rings grid |
| relationship_panel_component.dart | 416 | Relationship display |
| **Total** | **1,714** | Main UI components |

## 🎯 Next Steps

1. **Visual Testing:** Run `flutter run -d linux` to see layout
2. **Rust Integration:** Connect to backend for event/stat data
3. **Polish:** Fine-tune colors, timing, particle effects
4. **Mobile:** Add responsive breakpoints for smaller screens
5. **Audio:** Add SFX for transitions and choices

## ✨ Key Features

✅ Floating canvas model (no rigid frames)
✅ Persona 5 aesthetic (angled borders, gradients, slashes)
✅ Responsive positioning (recalculates on resize)
✅ Smooth animations (transitions, staggers, easing)
✅ Type-safe Dart (no compilation errors)
✅ Efficient canvas rendering (direct paint operations)
✅ Clean component hierarchy (reusable sub-components)

---

**Status:** Implementation Complete, Ready for Visual Testing & Backend Integration
