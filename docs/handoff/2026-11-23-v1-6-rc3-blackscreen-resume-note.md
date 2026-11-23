# Notaya 1.6 RC3 startup and safety copy

Ahmed (AE0101) · Gaming-PC · 2026-11-23 UTC

Ahmed's 2026-11-23T17:16:22.243Z instruction authorized diagnosing and fixing the installed RC2 black screen from 0afaacf3072101b9ac46bc810abcd268dbc79e5f on AE0101/v1-6-rc. Checks used a disposable installed-layout home and WebView2 profile in the named scratch directory. Publication stays held.

## Findings

The shipped RC2 installer executable displayed a black content area after the readiness watchdog showed its native window while vault status was pending. The large test vault reproduced the empty body and remaining temporary copy. The dark Mica cache set the appearance of the empty area, and the same empty renderer content reproduced with GPU acceleration disabled. Fresh-profile startup with a small vault reached notes.

## Changes

The main window paints the themed loading screen until vault status resolves. The safety-copy verifier reads paired rows in id order and checks exact headers, table presence, row counts, ids and sealed bytes. Each matching row still goes through authenticated decryption and unpadding. The SQLite copy, disk flush and write-through publication continue to run before the vault upgrade. Startup removes the exact abandoned part and journal names. Windows checks the entry type and uses the same handle for deletion.

## Evidence and next step

The isolated checks passed with 909 Rust library tests, 19 bridge tests, 18 readiness checks, 5 probe checks and 192/192 desktop-dev and production browser checks. The fixture contained 1,202,257,920 bytes. Verification measured 27.066–46.674 s before and 4.098–6.079 s after. The final natural release run displayed notes at 12.261 s and exited normally. The separate forced-delay release run also displayed the loading view and exited normally. The isolated overrides select the portable mode flag while using the installed directory layout.

The local summary and incremental bundle kept with the isolated verification evidence identify the exact revision reviewed in the RC3 audit. [Session 154](../sessions.md#session-154).

## RC3 audit follow-up

The watchdog still reveals the loading screen after five seconds while the vault status request runs. The loader uses the theme's foreground, border and accent colors. A rejected status request that has exhausted its startup retry shows a status error and a **Try again** button. The audit follow-up passed 18 readiness checks, five probe checks and 211 Chromium checks in each mock mode. The local follow-up receipt identifies the commit, checked files and bundle. [Session 155](../sessions.md#session-155).

<a id="stalled-automatic-retry"></a>

## Stalled automatic retry

The second audit found that a rejected first request followed by a stalled automatic retry could leave the loader up indefinitely. The five-second watchdog now shows the status error and **Try again** in that case. A pending first request still shows **Opening your vault…**. A manual retry clears the error while it runs and restores it if the request rejects. A late rejection from the automatic attempt cannot replace the pending manual retry screen.

The focused readiness suite covers the five-second boundary, an early listener failure and a successful automatic retry with listener registration still pending. The browser checks cover the stalled retry, the pending manual retry, its failure and later recovery. The local receipt records the squashed commit, tree, check counts and bundle verification. Native WebView2 and independent review of this follow-up remain pending. [Session 156](../sessions.md#session-156).
