This directory previously contained legacy Flutter `Scaffold` screens for gameplay.

SYN now renders all gameplay and menus via Flame components under `lib/components/`
and presents the entire experience inside `GameWidget(game: SynGame(...))`.

Do **not** add new gameplay UI here—use Flame components/overlays instead.***
