# Architecture

## Current State
- `GregModmanager` is the existing MAUI-based desktop application.
- `GregModmanager.Avalonia` is the new cross-platform UI foundation.

## Target Architecture
- **UI Layer:** Avalonia (`GregModmanager.Avalonia`) with custom chrome and design-system styles.
- **Domain Layer:** Shared services for Steam integration, auth/session, project metadata.
- **Infrastructure Layer:** Packaging scripts and CI pipelines for Windows/Linux distribution.

## Migration Strategy
1. Keep MAUI app functional while introducing Avalonia in parallel.
2. Move reusable business logic from MAUI-only concerns into shared services.
3. Replace page-by-page UI workflows in Avalonia.
4. Transition packaging and release channels to multi-platform outputs.

## Design Constraints
- Compact information density for usability.
- Keyboard-first interaction patterns.
- Strict Steam publish cooldown enforcement.
- Fully custom window chrome to avoid duplicate title bars.
