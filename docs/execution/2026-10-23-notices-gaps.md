# Execution Summary: EX-2026-10-23-20 — Notices gaps: WebView2 loader licence and KeePassXC library source pointers

(4-section form for a small task.)

## 1. Task Overview

After the V1.0 release the owner (message of 21:37:43 UTC) asked for the two small gaps in `THIRD-PARTY-NOTICES.md` to be done. The released V1.0 ZIP and its asset were not touched.

## 2. What was done

- **WebView2 loader:** the exact `LICENSE.txt` of Microsoft's NuGet package `Microsoft.Web.WebView2` 1.0.3800.47 (downloaded from api.nuget.org; the version string `1.0.3800.47` is inside the `WebView2LoaderStatic.lib` shipped by `webview2-com-sys` 0.39.1) is reproduced in `THIRD-PARTY-NOTICES.md` and saved as `packaging/Licenses/WebView2-LICENSE.txt`. The package's `NOTICE.txt` (SDK tooling components) was read but not found to apply to the loader. The WebView2 Runtime is Microsoft's product and is not reproduced.
- **Libraries inside KeePassXC:** rewritten from the real files: the 156 files of the shipped ZIP's `KeePassXC\` folder (version resources, strings, read on the owner's PC by the PC session), the upstream ZIP and source tarball, and upstream sources read from raw.githubusercontent.com (a research agent, model output). Correction: Qt is Qt's own official 6.11.2 build, not vcpkg-built; Botan, zlib, minizip, libqrencode and readline are vcpkg builds at baseline `1600ce2b…`. The notices now give each library's version, licence, copyright line where read, source location and a verification status, plus a "Source offer" section and `packaging/Licenses/KeePassXC-LIBRARIES-SOURCE.txt` for the ZIP.
- Also recorded: `vc_redist.x64.exe` IS in the V1.0 ZIP (the earlier records said it was not); `Cipherleaf.exe` does not import the Visual C++ runtime DLLs (import table only; not run without the runtime); KeePassXC ships an EFF word list `eff_large.wordlist` (licence unverified); the Authenticode signer of the KeePassXC folder files is "DroidMonkey Apps, LLC" (KeePassXC's release signing).
- `npm run build` ok with the larger notices text (the in-app dialog loads it lazily).

## 3. Not verified

download.qt.io, code.qt.io, botan.randombit.net, zlib.net, curl.se, keepassxc.org and Microsoft's sites were unreachable; no library source archive was downloaded or hash-checked; Microsoft's terms for `vc_redist.x64.exe`, `d3dcompiler_47.dll`, `dxcompiler.dll`, `dxil.dll`, the Mesa/LLVM licence, the licence of the Qt translations and of `eff_large.wordlist`, whether KeePassXC changed the Qt DLLs, and KeePassXC's release build job. No official KeePassXC statement on library source or Qt relinking was found. No Tauri/Rust check was run (only the notices text and new text files changed).

## 4. Open items

- The V1.0 ZIP and release asset do **not** include these updates. A V1.0.1 with the updated `Licenses` folder (copy `packaging/Licenses/*` and the updated notices) or an edit of the V1.0 asset is the owner's decision.
- CipherLeaf's own licence is still not chosen.
