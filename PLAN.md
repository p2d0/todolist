# Implementation Plan: PomoTasker Refinement

## 1. Visual Identity (Dark Theme)
- [ ] **Force Dark Preference**: Update `src/main.rs` to set `gtk-application-prefer-dark-theme` to true via `gtk4::Settings`.
- [ ] **Modern CSS Overhaul**: 
    - Rewrite `data/styles.css` with a deep charcoal palette (Background: `#181825`, Card: `#313244`).
    - Implement smooth button transitions and rounded corners (12px radius).
    - Style the "active" state of habit circles with a vibrant glow.

## 2. Core Logic Fixes
- [ ] **Start Logic**: Implement `timer_banner::start` in `src/ui/timer_banner.rs`.
- [ ] **State Handling**: Ensure the banner button correctly toggles between "Start" and "Stop" and persists the last habit ID in settings.

## 3. Interaction Improvements (Date Popover)
- [ ] **Popover Widget**: Create a `gtk4::Popover` in `src/ui/mod.rs` attached to date circles.
- [ ] **Action Menu**: Add buttons for:
    - `Start Pomodoro` (Timed mode).
    - `Start Stopwatch` (Continuous mode).
    - `Toggle Manual` (Quick mark as done).
- [ ] **UI Feedback**: Ensure circles update their visual state immediately after popover actions.

## 4. Visual Feedback (Circular Clock)
- [ ] **Custom Widget**: Create `src/ui/clock.rs` using `gtk4::DrawingArea`.
- [ ] **Cairo Drawing**: 
    - Draw a background track (dimmed circle).
    - Draw a progress arc with rounded caps.
    - Animate smooth progress updates.
- [ ] **Integration**: Replace the text-only label in the banner with the Circular Clock + digital time display.
