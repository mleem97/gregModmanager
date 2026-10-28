=== gregModmanager - Steam Play / Proton Guide ===

This application is a native Windows desktop client (Rust + Slint), shipping the
native Steamworks library for Windows (steam_api64.dll).
On Linux and Steam Deck it runs through Steam's Proton compatibility layer.


--- Enabling Proton (Linux / Steam Deck) ---

1. Open Steam and go to your Library.
2. Right-click "gregModmanager" > Properties.
3. Open the "Compatibility" tab.
4. Enable "Force the use of a specific Steam Play compatibility tool".
5. Select "Proton Experimental" or "Proton 9.0+" from the dropdown.
   Alternatively, install Proton GE via ProtonUp-Qt for broader
   native library compatibility.
6. Close Properties and launch the app normally.


--- Recommended Proton versions ---

  Proton Experimental   - Official Valve build, updated frequently.
  Proton GE             - Community build by GloriousEggroll with
                          extra native library fixes.


--- Troubleshooting ---

* If the app fails to start, try a different Proton version.
* Delete the Steam prefix to reset app state:
    ~/.steam/steam/steamapps/compatdata/<AppID>/
* Steam Deck: works in both Desktop Mode and Gaming Mode.
  Use the touchscreen or configure controller input via Steam Input.


--- Links ---

  Steam Play docs      https://store.steampowered.com/steamplay
  Proton GE releases   https://github.com/GloriousEggroll/proton-ge-custom
  ProtonDB reports     https://www.protondb.com/
