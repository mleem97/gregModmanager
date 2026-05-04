# Design System: Terminal Core

## Principles
- Structured precision on an 8px grid.
- Sharp geometry (`<= 8px` radius on containers).
- Hairline separators and technical contrast.
- Monospace for metadata and status values.

## Tokens (Reference)
- Surface base: `#051424`
- Surface container low: `#0D1C2D`
- Surface container: `#122131`
- Primary: `#8AEBFF`
- Secondary: `#4DE082`
- On-surface: `#D4E4FA`
- On-surface-variant: `#BBC9CD`

## Component Patterns
- `titlebar`: custom window frame row.
- `panel`: technical glass panel with edge stroke.
- `input-terminal`: compact dark input fields.
- `status-chip`: LED + monospace text state.
- `btn-primary`: cyan action button.

## Content Rules
- Micro labels are uppercase monospace.
- Data fields use monospace where possible.
- Avoid soft shadows and rounded bubble UI.
