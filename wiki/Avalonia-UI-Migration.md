# Avalonia UI Migration Plan

## Goals
- Build a production-ready Avalonia desktop client.
- Preserve existing workflow behavior.
- Improve UX density and scaling.

## Implemented Foundation
- New project: `GregModmanager.Avalonia`
- Custom chrome window (`SystemDecorations=None`, extended client area)
- Terminal Core themed shell and compact upload-focused layout
- Status row with Steam/GregAPI/Login text conventions

## Next Steps
1. Extract shared service interfaces from MAUI-specific code.
2. Wire real session/Steam state into Avalonia view models.
3. Port primary workflows:
   - Project browser
   - Mod upload editor
   - Workshop browser
4. Add accessibility and keyboard navigation QA pass.

## UX Scaling Rules
- Use compact type sizes for metadata (`10-12px`).
- Avoid oversized cards and empty whitespace.
- Keep controls aligned to a strict grid.
- Use high contrast in dark mode only.
