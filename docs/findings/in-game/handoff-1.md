# In-game hand-off 1

Date: 2026-10-19
Build: alaydriem-zoom 0.1.0, LeviLamina 26.51.6 client, Minecraft Bedrock 1.26.51.01 (LeviLauncher)
Defaults: keyCode 192 (tilde), zoomFactor 4.0, zoomInMs 250, zoomOutMs 250, easing easeInOutCubic, sensitivityMatchStrength 1.0

## Result

The user reported: "Works great, as far as I can tell."

- The zoom works when tilde is held, and eases back when it is released.
- The user did not report a problem with mouse speed, the hand or HUD, or the scroll wheel.
- The turn-delta hook works as far as the user can see, so the mouse-delta fallback (plan Task 8) is not needed.

## Not yet checked one by one

The user did not confirm each item of the checklist separately. These spec section 11 checks are still open for hand-off 2:

- the log lines `getFov hook active` and `turn delta hook active`
- low and high FPS feel the same
- releasing halfway has no jump, and the speed change on reversal feels acceptable
- Alt+Tab while zoomed does not stick; chat or inventory while zoomed resets
- sprint while zoomed does not wobble
- config edit + `/clill reactivate alaydriem-zoom` works without a restart

Task Completed
