# Flutter frontend

This directory contains a minimal Flutter UI that replaces the old interactive `eframe/egui` editor flow with a conversion-only launcher.

The app intentionally contains only two user actions:

- `Конвертировать сейвы` starts the existing Rust headless save conversion.
- `Отмена` closes the app when no conversion has started or cancels a queued launch before the process is spawned.

All conversion input comes from startup arguments. Pass the same arguments supported by the Rust GUI binary, except that `--save` is added automatically when the Flutter button starts conversion.

By default the frontend tries to execute `ree-save-editor` from `PATH`. Use `--backend /path/to/ree-save-editor` when the binary is not on `PATH`.
