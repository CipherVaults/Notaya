# Observed browser route

This is a conditional reference from repeated native uploads and live recovery in a Notion artwork workflow observed on 2026-10-17. Recheck capabilities, labels, paths, and access on the next run. It is not a universal environment layout or permission grant.

Observed evidence covers the filesystem separation, Thunar extraction, native-picker access, final icon Save, cover replacement, changing course navigation, and partial-page recovery described below. The neutral extraction destination is an adaptation, not a second tested path. The connector-native-upload route is a capability-dependent option, not a route verified with this connector. Archive-safety and image-pair validation are reliability requirements; their inclusion does not claim that a separate end-to-end test of this packaged skill was run.

## Connector observation

The connected tools offered a direct file-upload operation. The page-update schema still exposed `icon` and `cover` as strings, with external image URLs documented for custom image headers. Passing a file-upload reference as the icon was rejected with `Invalid page icon URL`. Browser-native uploads then provided the working route. This does not rule out future connector support; inspect the current schema before choosing a fallback.

## Browser filesystem and extraction

The user signed into the cloud browser through the appropriate login handoff. An artwork archive named `The Days.zip`, shown as approximately 163.8 MiB, was downloaded to that browser computer's `/home/agent/downloads/` directory. It was not visible from the separate executor workspace, and executor `/workspace/shared` files did not appear in the browser picker.

The working extraction method used the browser computer's Thunar file manager:

1. Select the downloaded archive and choose **Open With Other Application**.
2. Enter a custom extraction command using the confirmed destination path. The following is an adapted example with a neutral destination name:

   `unzip -n %f -d /home/agent/downloads/notion-artwork`

3. Leave the option to make this the default application **unchecked**. The task does not require changing file associations.
4. Confirm extraction and inspect the resulting directory rather than treating the launch as proof of success. `-n` avoids overwriting existing files; it does not establish that any existing files are the right version.

`%f` is Thunar's selected-file placeholder in this context, not a shell path to reuse elsewhere. Use the literal path actually discovered in the current browser computer. If a destination contains spaces, use the file manager's supported quoting. This route assumes archive inventory has already found no unsafe extraction paths.

The extracted layout was:

```text
<extraction-root>/The Days/
  Day-XX/
    icon.png
    cover.png
    PAGE.txt
```

The native Notion file picker could see these images. Using picker breadcrumbs and the visible `Day-XX` folder labels was more reliable than repeatedly typing paths. The working icon flow required an upload preview followed by a final **Save**, with the workspace emoji library option left unchecked. The cover flow required replacing the default **Add cover** image via **Change cover → Upload → Upload file** and waiting for the replacement. Old or default artwork sometimes remained briefly; a fresh screenshot showing the intended artwork, without an upload-progress state, was the completion evidence.

## Navigation and verification

For that course, the reliable next-lesson link was near the top of each page. Earlier pages placed it after an Arabic summary; from Day 47, it appeared directly below the title. Wrapped titles changed its screen position. The controller located it from the current page rather than reusing fixed coordinates or the sidebar's Recents list. These locations are course-specific; verify the link's actual destination and the next page's identity.

The first page was checked for persistence before the broader batch. Later pages used brief visual checks and checkpoints, respecting the user's instruction not to reload every page. A prior tool call that hung for roughly six minutes was an execution failure, not evidence that the remaining uploads had completed.

## Recovery lessons from the live run

- A controller handoff stalled around Day 39. Recovery began by inspecting the exact partial state: the icon was saved, but the cover still showed the default image. Only the unfinished cover step was resumed. A single direct controller then continued the batch.
- On Day 57, the icon had been saved when the cover action was denied for missing trusted authorization in that execution context. The specific existing user authorization was supplied through the supported process, and the unchanged action was retried once. That retry succeeded; the matching icon and cover were visibly saved at 20:25 UTC. This was an authorization-evidence issue, not a capability failure, and the successful retry did not require a new route or a broader permission grant.

## Completed-run evidence and limits

The continuation through Day 59 completed at 20:30 UTC on 2026-10-17. Days 13–28 were completed earlier, and matching native icon/cover pairs for Days 29–59 were accounted for with no known gaps. Days 35–37 used earlier completion checkpoints; the remaining pages in that final segment were directly checked during the concluding run. On Day 59, the microphone icon and matching cover were fully visible with no upload progress indicator or open dialog.

This supports the browser workflow as tested in that run. It is not an independent replay test of the packaged skill or a fresh reload audit of every page. The user waived per-page reloads; completion evidence was the observed native artwork plus the per-page checkpoints. Future runs must verify their own target pages, asset mapping, environment, and persistence requirements.
