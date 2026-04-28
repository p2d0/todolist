# TODO

## Goal
Build "PomoTasker" — a vertical GTK4 habit/pomodoro tracker with SQLite persistence, weekly progress circles, configurable timers, and stopwatch mode.

## Tasks

### 1. Project Setup
- [x] Create `Cargo.toml` (name: pomotasker, dependencies: gtk4, gtk4-layer-shell, rusqlite, chrono, serde)
- [x] Create `src/main.rs` with GTK4-layer-shell window (narrow vertical, ~300px wide, Hyprland overlay-ready)
- [x] Update `flake.nix` and `devshell.nix` to reflect project rename (appblocker → pomotasker)

### 2. Database Layer
- [x] Define SQLite schema: `habits` table (id, description, order_index, timer_duration_seconds, mode: timed|stopwatch) and `sessions` table (id, habit_id, date, duration_seconds, completed_at)
- [x] Implement database initialization & connection module
- [x] Implement CRUD operations for habits (add, edit, delete, reorder)
- [x] Implement session recording (start timer, stop timer, mark complete, insert into sessions)
- [x] Implement query helpers: sessions for a given date range, week aggregate per habit, streak calculation

### 3. Settings / Config
- [x] Implement default pomodoro duration (25min work / 5min break) as configurable fields
- [x] Build a settings dialog/section to adjust pomodoro duration, break duration, and session target time per habit

### 4. UI — Habit List
- [x] Build scrollable habit list view with each habit showing: description, drag handle for reorder, current week progress
- [x] Implement drag-and-drop reordering (reorder handle on each row)
- [x] Implement add/edit/delete habit actions (context menu or inline buttons)

### 5. UI — Weekly Progress View (Per Habit)
- [x] Build 7-circle row (Mon–Sun) per habit showing current week progress
- [x] Circle states: empty (gray), complete (filled/green), today (highlighted border)
- [x] Click on today's circle: if timed mode → start/stop timer; if stopwatch → start/stop stopwatch; if already complete → toggle off
- [x] Click on other day's circle: toggle complete/incomplete for past days

### 6. UI — Active Timer Display
- [x] Show active timer overlay/banner at top: countdown (timed) or elapsed (stopwatch), pause/resume, stop buttons
- [x] On timer completion (countdown reaches 0 or user stops): record session to DB, update progress circle
- [x] Show completed time converted to "X.X pomodoros" based on configured pomodoro length

### 7. UI — Week Overview / Streaks
- [x] Display current week summary: total sessions, total time, total pomodoros
- [x] Show streak indicator (consecutive days with at least one completed session)

## Notes
- Target platform: Linux, Hyprland Wayland compositor
- Use gtk4-layer-shell for overlay window capability
- Window should be narrow vertical (~300–350px wide)
- Use rusqlite (simpler, no async needed for a local desktop app)
- Habit reordering updates `order_index` in DB, queries order by `order_index ASC`
