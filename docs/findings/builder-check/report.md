# Builder check: open questions before planning levi-zoom

The spec is complete enough to plan from. Only two questions are left where a wrong guess would matter. Both are about scope and process, not about how the mod works.

## 1. Which official LiteLDev projects count as "other client mods" under the clean-room rule? (most important)

**Question:** The allowed sources are "LeviLamina's own source, public headers, and docs" and "the official mod template". Can the builder also read these first-party LiteLDev projects?

- `ShrBox/ExampleMod`: the official tutorial links to it as the full example.
- `LiteLDev/LeviSchematic`: an official LiteLDev client mod that uses `KeyRegistry` / input.

Third-party non-zoom client mods, for example `engsr6982/FastMiner`, raise the same question.

**Why it changes the plan:** Section 6 bans "any other zoom or client mod", and section 4 makes clean-room code an anti-goal.

- A wrong "yes" breaks a hard rule.
- A wrong "no" removes the only working examples of client hook registration, removing hooks in `disable()`, and client event use on 26.51. The plan would then need header-only spike tasks to find these patterns by trial and error.

The answer decides whether the plan has a "read reference examples" step or a "header-only spike" step. It also decides what reviewers may compare against.

## 2. Who runs the in-game checks, and is a test install ready?

**Question:** Is a LeviLauncher install of Minecraft 26.51.1 with the LeviLamina 26.51.7 client already on this machine? Will the user run the section 11 in-game checklist and the section 10 curve and duration tuning, and report back? Or must the builder make these judgments?

**Why it changes the plan:** No end-to-end suite exists, and only a person at the game can judge "smooth" or "no wobble".

- **If the user does the checks**, the plan needs hand-off checkpoints: build and deploy to `mods/`, the user tests and reports, then the builder tunes the defaults.
- **The most important early checkpoint** is "do `getFov` and `_applyTurnDelta` resolve and fire on 26.51.7". If that fails, the `MouseInputEvent` fallback becomes the main path.
- **If no install exists**, the plan must start with an environment-setup task (LeviLauncher, lip, the LeviLamina client, VS 2022 with the Clang tools, xmake). No task can depend on in-game results until that setup is done.

## Not asked (the builder decides, or the spec already settles it)

- Snap or ease on reset, curve and duration tuning, and left and right Ctrl: section 10 leaves these to the builder.
- How to turn off variable FOV without a snap: blend `origin(a, true)` into `origin(a, false)` with ease progress. The anti-goals already set the rule.
- The unit test framework and the test target layout under xmake.
- The reload method: section 8 accepts `/clill reactivate`.
- Spyglass, third-person, and controller turn scaling: edge cases to log during in-game tests. They do not change the plan structure.

Task Completed
