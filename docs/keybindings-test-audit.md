# Keybindings test audit

This records the audit before the compact recorder update. The mutation results
below apply to that version, which still had Save and Remove buttons.

The current recorder captures all keys, including Escape, Enter, Tab, and Backspace.
Its inline buttons save, cancel, and remove shortcuts. The interaction test is
`recorder_buttons_save_cancel_and_remove`. The test host renders actual
command rows. `recording_keeps_command_rows_and_shortcut_column_in_place`
clicks a real command row and checks row and shortcut-column bounds while starting,
capturing, displaying a conflict, and cancelling at narrow and wide widths. This
also closes the command-row-click coverage gap noted below.

`special_keys_are_recorded_and_never_dispatch_while_capturing` assigns special keys,
confirms their actions work outside recording, and verifies they cannot dispatch
or end recording while capture is active. `recording_intercepts_commands_and_saves_a_replacement`
also sends a key immediately after activation, before the recorder's first paint.
The mutation results and test counts below describe the earlier audited version.

The settings diff introduced 12 tests. Most exercise interactions between the
keymap, persistence, and recorder. They do not merely compare a helper's output
with a copy of its implementation. However, the original recorder tests used a
substitute focus tree, and cancellation was only checked with an invalid draft.
Those were coverage weaknesses.

The repository's 24-test total includes 12 pre-existing tests. In particular,
the four basic binding-service tests and the sidebar action test predate this
settings work. Moving those tests into another file did not add coverage.

## Trace of the tests added for settings

Service tests are in [keybindings_service/src/tests.rs](../crates/keybindings_service/src/tests.rs).
Recorder and search tests are in [settings/keybindings/tests.rs](../crates/workspace/src/settings/keybindings/tests.rs).

| Test | Behavior exercised and regression it catches | Audit decision |
| --- | --- | --- |
| `overrides_and_removed_shortcuts_survive_restart` | Writes real temporary preferences, creates a fresh GPUI app, reloads them, and checks the GPUI keymap. Detects lost customizations, resurrected removed shortcuts, and old shortcuts remaining active. Also checks resetting persisted preferences. | Keep. Independent file and keymap observations. |
| `rejects_conflicts_including_aliases_and_chord_prefixes` | Submits an alias of an occupied shortcut and both directions of a chord-prefix collision. Checks rejection and preservation of the current binding. | Keep. Protects ambiguous keyboard dispatch. |
| `reset_all_restores_swapped_defaults_and_keeps_unmanaged_bindings` | Swaps two defaults, checks that an individual reset conflicts, then resets both. Checks GPUI bindings and an unrelated binding in the Input context. | Keep. Catches reset ordering and accidental clearing of other owners' bindings. |
| `failed_save_keeps_the_active_binding` | Creates a file where the preferences directory needs to be, forcing a real filesystem error. Verifies that the old shortcut remains active and the attempted replacement is absent. | Keep. Detects applying a change before saving it succeeds. |
| `corrupt_settings_are_not_overwritten` | Loads invalid JSON, attempts an edit, then rereads the original bytes and checks the active shortcut. | Keep. Protects user preferences from being silently overwritten. |
| `reset_restores_an_unassigned_default` | Registers a command with no default, assigns it, then resets it. Verifies removal from the GPUI keymap. | Keep. Renamed from `an_unassigned_command_can_be_bound_removed_and_reset`, which claimed a separate removal step it did not exercise. |
| `fixed_shortcuts_stay_out_of_settings_but_still_prevent_conflicts` | Registers a fixed shortcut outside the editable command list. Checks that it cannot be reassigned and survives Reset all. | Keep. Protects the distinction between editable commands and fixed application shortcuts. |
| `registering_an_existing_binding_keeps_one_active_shortcut` | Switched one action repeatedly between fixed-binding and customizable-command APIs. | Removed. No current application flow does this; most assertions overlapped the replacement and reset tests. |
| `recording_intercepts_commands_and_saves_a_replacement` | Sends actual GPUI keystrokes while recording a harmless fake Quit action. Verifies that it does not dispatch during recording, Enter saves, the old key stops dispatching, and the new key dispatches once. | Keep. Now renders the real recorder instead of a substitute focus tree. |
| `conflict_and_cancel_preserve_existing_shortcuts` | Records a conflicting shortcut, checks the error, then records a valid replacement and presses Escape. Checks that neither command changed. | Strengthened. Cancelling only an invalid draft could pass even if cancellation tried to save it. |
| `inline_recorder_releases_capture_when_focus_leaves` | Moves actual window focus outside the recorder after entering a draft. Verifies that the draft is discarded, the original binding remains, and unrelated shortcuts dispatch again. | Keep. Now uses the real recorder's focus elements. |
| `search_accepts_names_symbols_and_modifier_aliases` | Tests command-name and description matching, modifier aliases, symbols, and a query containing both a matching and a nonmatching word. | Keep as a small unit test. Removed unnecessary window setup; added the mixed query to catch incorrect OR matching. Does not cover connecting filtering to the rendered list. |
| `recorder_buttons_validate_save_cancel_and_remove` | Clicks the real Save, Cancel, and Remove buttons. Rejects a bare-letter shortcut and verifies the effect on subsequent action dispatch. | Added in place of the overlapping service test. Strengthened after mutation testing exposed a false positive in its initial combined action count. |

The fake Quit action increments a counter. These tests do not quit the native
application. The restart test creates a fresh app state and reloads a real file;
it does not relaunch the operating-system process.

## Deliberately broken implementations

Ran 22 selected mutations separately in a temporary copy of the workspace. Each
variant compiled, and each selected test was run against it. The baseline passed.
The user's running application did not receive these mutations.

| Deliberate defect | Result |
| --- | --- |
| Stop unbinding the previous shortcut | Detected |
| Discard preferences instead of persisting them | Detected |
| Ignore saved shortcuts when registering commands | Detected |
| Update the runtime keymap before a failed save | Detected |
| Allow overwriting preferences after a load error | Detected |
| Skip restoring bindings in Reset all | Detected |
| Allow an exact shortcut conflict | Detected |
| Allow a longer chord with an occupied prefix | Detected |
| Allow a shortcut that is a prefix of an occupied chord | Detected |
| Ignore fixed commands when checking conflicts | Detected |
| Skip resets whose default is unassigned | Detected |
| Allow recorded keystrokes to dispatch global actions | Detected |
| Keep the draft after focus leaves | Detected |
| Make Escape save instead of cancel | Detected by the strengthened cancellation test |
| Remove the recorder's rendered focus scope | Detected by using the real recorder |
| Accept bare letters as shortcuts | Detected |
| Wire Save to Cancel | Initially survived; detected after checking the old and new shortcuts separately |
| Wire Cancel to Save | Detected |
| Wire Remove to Cancel | Detected |
| Stop normalizing the Command symbol in search | Detected |
| Match any search word instead of all words | Detected by the mixed-query assertion |
| Disconnect search filtering from the rendered command list | Survived |

After strengthening the button test, 21 of these 22 selected defects are detected.
This is evidence for the listed scenarios, not an exhaustive coverage measurement.

## Remaining gaps

- Search normalization is automated; the input-to-visible-list connection is not.
  A full-page headless test currently fails because the GPUI input accesses a
  native window handle. A substitute list would hide the same wiring defect.
- Opening settings through Command-comma, the native menu, and the status-bar
  button is checked manually. The recorder tests start recording directly, so
  they do not cover clicking a command row to begin recording.
- Native menu key equivalents, focus restoration to the previous workspace
  control, and visual layout remain manual checks.
- The filesystem tests cover successful persistence, failed writes, and corrupt
  files. They do not simulate a crash or power loss during an atomic write.

Manual checks establish that these flows work now. They do not automatically
detect future regressions.
