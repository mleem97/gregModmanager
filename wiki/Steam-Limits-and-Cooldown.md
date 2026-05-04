# Steam Limits and Cooldown

## Why Cooldown Exists
Steam workshop operations can fail or throttle when upload/update requests are too frequent.

## Implemented Policy
`SteamPublishRateLimiter` enforces:
- Minimum interval between publish attempts: **30s**
- Rolling window: **10 minutes**
- Maximum attempts per window: **5**

If blocked, publish returns a user-facing message with retry time.

## Integration Point
- `Services/SteamWorkshopService.cs`
- cooldown check happens before `SubmitAsync(...)`

## UX Behavior
- UI should show cooldown banner while blocked.
- Upload buttons should disable when cooldown is active.
- Retry timer must be explicit in seconds.
