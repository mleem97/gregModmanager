#!/usr/bin/env python3
"""Align Facepunch's Linux Steam interface names with the bundled SDK."""

from pathlib import Path
import sys


def main() -> int:
    if len(sys.argv) != 2:
        print(f"usage: {sys.argv[0]} <published-binary>", file=sys.stderr)
        return 2

    binary = Path(sys.argv[1])
    data = binary.read_bytes()
    replacements = {
        b"SteamAPI_SteamApps_v008": b"SteamAPI_SteamApps_v009",
        b"SteamAPI_SteamUGC_v020": b"SteamAPI_SteamUGC_v021",
        b"SteamAPI_SteamFriends_v017": b"SteamAPI_SteamFriends_v018",
        b"SteamAPI_SteamGameServerUGC_v020": b"SteamAPI_SteamGameServerUGC_v021",
        b"SteamAPI_SteamGameServerUtils_v010": b"SteamAPI_SteamGameServerUtils_v011",
        b"SteamAPI_SteamInput_v006": b"SteamAPI_SteamInput_v007",
        b"SteamAPI_SteamRemotePlay_v002": b"SteamAPI_SteamRemotePlay_v004",
        b"SteamAPI_SteamUtils_v010": b"SteamAPI_SteamUtils_v011",
        b"SteamAPI_SteamNetworkingSockets_SteamAPI_v012": b"SteamAPI_SteamNetworkingSockets_SteamAPI_v013",
    }

    for old, new in replacements.items():
        count = data.count(old)
        if count:
            data = data.replace(old, new)

    binary.write_bytes(data)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
