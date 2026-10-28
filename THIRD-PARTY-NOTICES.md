# Third-party notices

Notaya includes or uses the open source components listed in this file. This file gives credit to their authors and reproduces the licenses that ask to travel with the program. It is attribution work and not legal advice.

The license for Notaya's own code has not been chosen yet (the owner has not decided). Nothing in this file grants any license to Notaya itself. Each component below keeps its own license and its own copyright holders.

How this file was made: the lists in the Rust crates and npm packages sections were generated from the Rust lockfile (`src-tauri/Cargo.lock`, target `x86_64-pc-windows-msvc`) and from the npm lockfile and installed packages on 2026-10-23 (system clock, UTC). Copyright lines were copied from the license files of the packages. Where a package ships no copyright line, the table says so and gives the source of the name. Anything that could not be checked is marked "unverified".

Contents: At a glance, Open decisions and things to check, Rust crates, npm packages, License texts.

## At a glance

| Component | What it is here | License | Status |
|---|---|---|---|
| Tauri and the Rust crates | The application shell and the Rust core (226 crates linked into the Windows exe) | Mostly MIT or Apache-2.0 (dual); also Unicode-3.0, Zlib, BSD-3-Clause, MPL-2.0, Unlicense, CC0-1.0 | Full list in the Rust crates section. |
| Microsoft Edge WebView2 runtime | Draws the interface | Loader library: Microsoft BSD-style license (text below). Runtime: Microsoft's own terms, not reproduced | Required on the PC. Not bundled. The Rust crate `webview2-com-sys` (MIT) links Microsoft's static WebView2 loader library (`WebView2LoaderStatic.lib`, Microsoft.Web.WebView2 SDK 1.0.3800.47) into the exe. That library's license text, from Microsoft's own package, is reproduced in the License texts section and ships in `Licenses\WebView2-LICENSE.txt`. The Edge WebView2 Runtime on the PC is Microsoft's product under Microsoft's own terms and is not part of the ZIP. |
| React, TipTap, ProseMirror, KaTeX, Radix UI, and other npm packages | The user interface | Mostly MIT; also ISC, BSD-3-Clause, 0BSD, SIL OFL 1.1 | 114 package rows plus Tailwind CSS in the npm packages section. |
| Inter and JetBrains Mono (fonts) | Interface and code fonts, via the Fontsource npm packages | SIL Open Font License 1.1 | Copyright lines and license text below. |
| KaTeX fonts | Math rendering | SIL Open Font License 1.1, with Reserved Font Names (one per family) | The npm package ships no separate license file; the notice is embedded in the font files. |
| Lucide icons | Interface icons (lucide-react) | ISC; Feather-derived icons are MIT, Copyright (c) 2013-present Cole Bemis | Both texts below. |
| highlight.js (via lowlight) | Code block highlighting | BSD-3-Clause, Copyright (c) 2006, Ivan Sagalaev. | Text below. |
| SQLite | Storage engine, compiled in via `libsqlite3-sys` (SQLite 3.53.2) | Public domain | The author disclaims copyright; credited as a courtesy. |
| zxcvbn | Password strength estimate (Rust crate `zxcvbn` 3.1.1, a port of Dropbox's zxcvbn) | MIT, Copyright (c) 2016 Joshua Holmer | Its frequency dictionaries derive from US census data, Wikipedia and TV/movie word lists (per the crate's README); the package carries no separate notice for those. |

Trademarks: Windows, Microsoft Edge, WebView2, Segoe and Mica are names of Microsoft; Tauri, KaTeX and the other project names belong to their owners. They are used here only to describe what is used. Notaya is not affiliated with or endorsed by Microsoft or any other project named here.

## Open decisions and things to check

These come from the inventory work. They are facts for the owner to decide on; nothing was changed in the code or the build for them.

1. **Notaya's own license is not chosen.** There is no `LICENSE` file and no `license` field in `package.json` or `src-tauri/Cargo.toml`. Without a license nobody but the owner has rights to reuse Notaya's code.
2. **(No longer applicable.)** The source offer for the libraries inside KeePassXC. KeePassXC was removed from Notaya on 2026-10-27, so Notaya no longer ships it or its libraries.
3. **(No longer applicable.)** The KeePassXC license files and source archive in the portable folder. Removed together with KeePassXC on 2026-10-27.
4. **(No longer applicable.)** `vc_redist.x64.exe` and the other Microsoft files in the KeePassXC folder. Removed together with KeePassXC on 2026-10-27. `Notaya.exe` itself does not import the Visual C++ runtime DLLs (the C runtime is linked in; read from its import table, not tested on a PC without the runtime).
5. **Microsoft WebView2 loader license text. (Closed on 2026-10-23.)** `webview2-com-sys` 0.39.1 (MIT) ships Microsoft's prebuilt loader libraries (`WebView2LoaderStatic.lib` is linked statically into the exe). The version string inside the library is `1.0.3800.47`. The license text was read from Microsoft's own NuGet package `Microsoft.Web.WebView2` 1.0.3800.47 (file `LICENSE.txt`, downloaded from api.nuget.org on 2026-10-23; the package's `.nuspec` says `<license type="file">LICENSE.txt</license>`) and is reproduced in the License texts section. That package's `NOTICE.txt` lists third-party components of Microsoft's SDK tooling (for example ANTLR and StringTemplate) and says they come from Microsoft's build of the SDK; it was not found to apply to the loader library itself, and this was not checked further. The WebView2 Runtime on the PC is a Microsoft component under Microsoft's license; it is not in the exe or the ZIP.
6. **Rust crates whose packages carry no license file.** `webview2-com` 0.39.1 and `webview2-com-sys` 0.39.1 (both shipped) and `webview2-com-macros` 0.8.1 (build only) are MIT per `Cargo.toml` but ship no license file; the copyright line "Copyright (c) 2021 Bill Avery" comes from the upstream repository https://github.com/wravery/webview2-rs (file `LICENSE`, fetched on 2026-10-23). `alloc-stdlib` 0.3.0 (BSD-3-Clause) ships no license file; the line "Copyright (c) 2016 Dropbox, Inc." comes from the sibling repository https://github.com/dropbox/rust-alloc-no-stdlib. `selectors` 0.38.0 (MPL-2.0) ships no license file; its table row names only the `Cargo.toml` authors. `tauri-plugin` 2.7.2 (build only) ships none. Several crates by David Tolnay and others (anyhow, serde, serde_json, syn, thiserror and similar) have license files without a copyright line; their table rows show the `Cargo.toml` authors instead, labelled as such. `cargo-platform`, `hashlink`, `pin-project-lite`, `serde_json`, `winnow` have neither.
7. **MPL-2.0 crates (shipped unmodified).** MPL-2.0 asks that the source of the covered files stays available and that the license text is included. The source is unmodified and available at the upstream URLs: `cssparser` 0.37.0 https://github.com/servo/rust-cssparser ; `dtoa-short` 0.3.5 https://github.com/upsuper/dtoa-short ; `option-ext` 0.2.0 https://github.com/soc/option-ext ; `selectors` 0.38.0 https://github.com/servo/stylo . (`cssparser-macros` 0.7.1, MPL-2.0, is a compile-time macro crate and is not in the exe.) The MPL-2.0 text is in the License texts section.
8. **`regex-syntax` 0.8.11 Unicode data.** Declared `MIT OR Apache-2.0`, but its package contains a nested `src/unicode_tables/LICENSE-UNICODE` with the older "UNICODE, INC. LICENSE AGREEMENT - DATA FILES AND SOFTWARE" text (Unicode-DFS-2016). The metadata does not mention it. That text is reproduced below.
9. **`react-remove-scroll-bar` 2.3.8 has no license file.** Its `package.json` says MIT and its README ends with "License: MIT", but the package ships no LICENSE file and no copyright line. It is credited here to its author Anton Korzunov (package.json author) without a year; sibling packages by the same author carry "Copyright (c) 2017/2020 Anton Korzunov". Upstream: https://github.com/theKashey/react-remove-scroll-bar.
10. **KaTeX fonts: OFL notice only in the font files.** The `katex` npm package ships no separate license file for its fonts. The OFL grant exists only as a short notice inside each font file's name table (copyright lines Design Science, Inc. and Khan Academy, with a Reserved Font Name per family). The fonts are embedded in the exe unchanged. The in-app text "Equations are drawn by KaTeX 0.19.0 (MIT licence)" covers the code only and does not mention the OFL fonts.
11. **Unverified because the sandbox could not reach it:** Microsoft's WebView2 Runtime terms.
12. **MIT-0.** `dunce` 1.0.5 is `CC0-1.0 OR MIT-0 OR Apache-2.0`; its package contains only the CC0-1.0 text, which is used here. No MIT-0 text is reproduced because no shipped package contains one.
13. **Icon and logo.** The Notaya mark, two sticky notes in the Sunrise colours (`assets/notaya-icon.svg`, `src/components/Logo.tsx`, `src-tauri/icons/*`), was drawn for this project in the development sessions on 2026-10-27; no outside source is recorded. Every file in `src-tauri/icons/` was regenerated from `assets/notaya-icon.svg` with `npx tauri icon`. No license is recorded for the artwork. The emoji used for notebook icons are plain Unicode characters drawn by the system font and are not shipped.
14. **Version numbers differ:** `package.json` and `src-tauri/Cargo.toml` say 0.1.0, `tauri.conf.json` says 1.0.0.

## Rust crates

All 226 crates linked into the Windows release exe (`x86_64-pc-windows-msvc`), from `cargo metadata --locked`. License is the `license` field of each crate's `Cargo.toml`, unchanged (so "MIT OR Apache-2.0" means the recipient may choose either). Copyright text is copied from the crate's license, COPYRIGHT or SPDX files; where none names a holder, the `Cargo.toml` authors are shown and labelled. Crates that appear twice are different versions. Zero NOTICE files exist in any of the crates (checked across all 302 crates, shipped and build-time), so there is no Apache-2.0 NOTICE text to reproduce.

| Crate | Version | License (Cargo.toml) | Copyright / authors | Source URL |
|---|---|---|---|---|
| aead | 0.6.1 | MIT OR Apache-2.0 | Copyright (c) 2019-2026 The RustCrypto Project Developers; Copyright (c) 2019 MobileCoin, LLC | https://github.com/RustCrypto/traits |
| aho-corasick | 1.1.5 | Unlicense OR MIT | Copyright (c) 2015 Andrew Gallant | https://github.com/BurntSushi/aho-corasick |
| alloc-no-stdlib | 3.0.0 | BSD-3-Clause | Copyright (c) 2016 Dropbox, Inc. | https://github.com/dropbox/rust-alloc-no-stdlib |
| alloc-stdlib | 0.3.0 | BSD-3-Clause | Cargo.toml authors: Daniel Reiter Horn (copyright line in the package's LICENSE.spdx: 2021-2023, The Tauri Programme in the Commons Conservancy) ; NOTE: package ships no license file; sibling repo dropbox/rust-alloc-no-stdlib LICENSE (BSD-3-Clause): "Copyright (c) 2016 Dropbox, Inc." [fetched] | https://github.com/dropbox/rust-alloc-no-stdlib |
| anyhow | 1.0.104 | MIT OR Apache-2.0 | Cargo.toml authors: David Tolnay (copyright line in the package's LICENSE.spdx: 2021-2023, The Tauri Programme in the Commons Conservancy) | https://github.com/dtolnay/anyhow |
| argon2 | 0.6.0 | MIT OR Apache-2.0 | Copyright (c) 2021-2026 The RustCrypto Project Developers | https://github.com/RustCrypto/password-hashes |
| base64 | 0.23.1 | MIT OR Apache-2.0 | Copyright (c) 2025 Alice Maz, Marshall Pierce | https://github.com/marshallpierce/rust-base64 |
| base64ct | 1.8.3 | Apache-2.0 OR MIT | Copyright (c) 2014 Steve "Sc00bz" Thomas (steve at tobtu dot com); Copyright (c) 2021-2025 The RustCrypto Project Developers | https://github.com/RustCrypto/formats |
| bit-set | 0.8.0 | Apache-2.0 OR MIT | Copyright (c) 2023 The Rust Project Developers | https://github.com/contain-rs/bit-set |
| bit-vec | 0.8.0 | Apache-2.0 OR MIT | Copyright (c) 2023 The Rust Project Developers | https://github.com/contain-rs/bit-vec |
| bitflags | 1.3.2 | MIT/Apache-2.0 | Copyright (c) 2014 The Rust Project Developers | https://github.com/bitflags/bitflags |
| bitflags | 2.13.2 | MIT OR Apache-2.0 | Copyright (c) 2014 The Rust Project Developers | https://github.com/bitflags/bitflags |
| blake2 | 0.11.0 | MIT OR Apache-2.0 | Copyright (c) 2017-2026 The RustCrypto Project Developers; Copyright (c) 2017 Artyom Pavlov; Copyright (c) 2015-2016 The blake2-rfc Developers, Cesar Barros | https://github.com/RustCrypto/hashes |
| block-buffer | 0.12.1 | MIT OR Apache-2.0 | Copyright (c) 2018-2025 The RustCrypto Project Developers | https://github.com/RustCrypto/utils |
| brotli | 9.0.0 | BSD-3-Clause AND MIT | Copyright (c) 2016 Dropbox, Inc.; Copyright (c) 2009, 2010, 2013-2016 by the Brotli Authors. | https://github.com/dropbox/rust-brotli |
| brotli-decompressor | 6.0.1 | BSD-3-Clause/MIT | Copyright (c) 2016 Dropbox, Inc. | https://github.com/dropbox/rust-brotli-decompressor |
| bs58 | 0.5.1 | MIT/Apache-2.0 | Copyright (c) 2016 The roaring-rs developers. | https://github.com/Nullus157/bs58-rs |
| bytes | 1.12.1 | MIT | Copyright (c) 2018 Carl Lerche | https://github.com/tokio-rs/bytes |
| camino | 1.2.6 | MIT OR Apache-2.0 | Cargo.toml authors: Without Boats, Ashley Williams, Steve Klabnik, Rain (copyright line in the package's LICENSE.spdx: 2021-2023, The Tauri Programme in the Commons Conservancy) | https://github.com/camino-rs/camino |
| cargo-platform | 0.1.9 | MIT OR Apache-2.0 | none stated in package (no copyright line, no Cargo.toml authors) | https://github.com/rust-lang/cargo |
| cargo_metadata | 0.19.2 | MIT | Cargo.toml authors: Oliver Schneider (copyright line in the package's LICENSE.spdx: 2021-2023, The Tauri Programme in the Commons Conservancy) | https://github.com/oli-obk/cargo_metadata |
| cfb | 0.14.0 | MIT | Copyright (c) 2017 Matthew D. Steele | https://github.com/mdsteele/rust-cfb |
| cfg-if | 1.0.5 | MIT OR Apache-2.0 | Copyright (c) 2014 Alex Crichton | https://github.com/rust-lang/cfg-if |
| chacha20 | 0.10.2 | MIT OR Apache-2.0 | Copyright (c) 2019-2026 The RustCrypto Project Developers | https://github.com/RustCrypto/stream-ciphers |
| chacha20poly1305 | 0.11.0 | Apache-2.0 OR MIT | Copyright (c) 2019-2026 The RustCrypto Project Developers | https://github.com/RustCrypto/AEADs |
| chrono | 0.4.45 | MIT OR Apache-2.0 | Copyright (c) 2014, Kang Seonghoon. | https://github.com/chronotope/chrono |
| cipher | 0.5.2 | MIT OR Apache-2.0 | Copyright (c) 2016-2025 RustCrypto Developers | https://github.com/RustCrypto/traits |
| cmov | 0.5.4 | Apache-2.0 OR MIT | Copyright (c) 2022-2026 The RustCrypto Project Developers | https://github.com/RustCrypto/utils |
| const-oid | 0.10.2 | Apache-2.0 OR MIT | Copyright (c) 2020-2026 The RustCrypto Project Developers | https://github.com/RustCrypto/formats |
| cookie | 0.18.2 | MIT OR Apache-2.0 | Copyright 2017 Sergio Benitez; Copyright 2014 Alex Chricton; Copyright (c) 2017 Sergio Benitez; Copyright (c) 2014 Alex Crichton | https://github.com/SergioBenitez/cookie-rs |
| cpufeatures | 0.3.1 | MIT OR Apache-2.0 | Copyright (c) 2020-2026 The RustCrypto Project Developers | https://github.com/RustCrypto/utils |
| crossbeam-channel | 0.5.17 | MIT OR Apache-2.0 | Copyright (c) 2019 The Crossbeam Project Developers; Copyright (c) 2009 The Go Authors. All rights reserved. | https://github.com/crossbeam-rs/crossbeam |
| crossbeam-deque | 0.8.8 | MIT OR Apache-2.0 | Copyright (c) 2019 The Crossbeam Project Developers | https://github.com/crossbeam-rs/crossbeam |
| crossbeam-epoch | 0.9.21 | MIT OR Apache-2.0 | Copyright (c) 2019 The Crossbeam Project Developers | https://github.com/crossbeam-rs/crossbeam |
| crossbeam-utils | 0.8.23 | MIT OR Apache-2.0 | Copyright (c) 2019 The Crossbeam Project Developers | https://github.com/crossbeam-rs/crossbeam |
| crypto-common | 0.2.2 | MIT OR Apache-2.0 | Copyright (c) 2021-2026 RustCrypto Developers | https://github.com/RustCrypto/traits |
| cssparser | 0.37.0 | MPL-2.0 | Cargo.toml authors: Simon Sapin (copyright line in the package's LICENSE.spdx: 2021-2023, The Tauri Programme in the Commons Conservancy) | https://github.com/servo/rust-cssparser |
| ctor | 1.0.13 | Apache-2.0 OR MIT | Cargo.toml authors: Matt Mastracci (copyright line in the package's LICENSE.spdx: 2021-2023, The Tauri Programme in the Commons Conservancy) | https://github.com/mmastrac/linktime |
| ctutils | 0.4.3 | Apache-2.0 OR MIT | Copyright (c) 2025-2026 The RustCrypto Project Developers | https://github.com/RustCrypto/utils |
| defmt | 1.2.0 | MIT OR Apache-2.0 | Copyright (c) Ferrous Systems | https://github.com/knurling-rs/defmt |
| deranged | 0.5.9 | MIT OR Apache-2.0 | Copyright 2024 Jacob Pratt et al.; Copyright (c) 2024 Jacob Pratt et al. | https://github.com/jhpratt/deranged |
| derive_builder | 0.20.2 | MIT OR Apache-2.0 | Copyright (c) 2016 rust-derive-builder contributors | https://github.com/colin-kiegel/rust-derive-builder |
| derive_more | 2.1.1 | MIT | Copyright (c) 2016 Jelte Fennema | https://github.com/JelteF/derive_more |
| digest | 0.11.3 | MIT OR Apache-2.0 | Copyright (c) 2017-2025 RustCrypto Developers; Copyright (c) 2017 Artyom Pavlov | https://github.com/RustCrypto/traits |
| dirs | 7.0.0 | MIT OR Apache-2.0 | Copyright (c) 2018-2019 dirs-rs contributors | https://codeberg.org/dirs/dirs-rs |
| dirs-sys | 0.5.0 | MIT OR Apache-2.0 | Copyright (c) 2018-2019 dirs-rs contributors | https://github.com/dirs-dev/dirs-sys-rs |
| dom_query | 0.28.0 | MIT | Copyright (c) 2023 Mykola Humanov | https://github.com/niklak/dom_query |
| dpi | 0.1.2 | Apache-2.0 AND MIT | Copyright (c) 2018 Jorge Aparicio; Copyright © 2005-2020 Rich Felker, et al.; Copyright © 1993,2004 Sun Microsystems or; Copyright © 2003-2011 David Schultz or; Copyright © 2003-2009 Steven G. Kargl or; Copyright © 2003-2009 Bruce D. Evans or; Copyright © 2008 Stephen L. Moshier or; Copyright © 2017-2018 Arm Limited | https://github.com/rust-windowing/winit |
| dtoa | 1.0.11 | MIT OR Apache-2.0 | Cargo.toml authors: David Tolnay (copyright line in the package's LICENSE.spdx: 2021-2023, The Tauri Programme in the Commons Conservancy) | https://github.com/dtolnay/dtoa |
| dtoa-short | 0.3.5 | MPL-2.0 | Cargo.toml authors: Xidorn Quan (copyright line in the package's LICENSE.spdx: 2021-2023, The Tauri Programme in the Commons Conservancy) | https://github.com/upsuper/dtoa-short |
| dunce | 1.0.5 | CC0-1.0 OR MIT-0 OR Apache-2.0 | Cargo.toml authors: Kornel (copyright line in the package's LICENSE.spdx: 2021-2023, The Tauri Programme in the Commons Conservancy) | https://gitlab.com/kornelski/dunce |
| dyn-clone | 1.0.20 | MIT OR Apache-2.0 | Cargo.toml authors: David Tolnay (copyright line in the package's LICENSE.spdx: 2021-2023, The Tauri Programme in the Commons Conservancy) | https://github.com/dtolnay/dyn-clone |
| either | 1.19.0 | MIT OR Apache-2.0 | Copyright (c) 2015 ; (license file names no holder; no Cargo.toml authors) | https://github.com/rayon-rs/either |
| equivalent | 1.0.3 | Apache-2.0 OR MIT | Copyright (c) 2016--2023 ; (license file names no holder; no Cargo.toml authors) | https://github.com/indexmap-rs/equivalent |
| erased-serde | 0.4.10 | MIT OR Apache-2.0 | Cargo.toml authors: David Tolnay (copyright line in the package's LICENSE.spdx: 2021-2023, The Tauri Programme in the Commons Conservancy) | https://github.com/dtolnay/erased-serde |
| fallible-iterator | 0.3.0 | MIT/Apache-2.0 | Copyright (c) 2015 The rust-openssl-verify Developers | https://github.com/sfackler/rust-fallible-iterator |
| fallible-streaming-iterator | 0.1.9 | MIT/Apache-2.0 | Copyright (c) 2016 The fallible-streaming-iterator Developers | https://github.com/sfackler/fallible-streaming-iterator |
| fancy-regex | 0.18.0 | MIT | Copyright 2015 The Fancy Regex Authors. | https://github.com/fancy-regex/fancy-regex |
| fnv | 1.0.7 | Apache-2.0 / MIT | Copyright (c) 2017 Contributors | https://github.com/servo/rust-fnv |
| foldhash | 0.2.0 | Zlib | Copyright (c) 2024 Orson Peters | https://github.com/orlp/foldhash |
| form_urlencoded | 1.2.2 | MIT OR Apache-2.0 | Copyright (c) 2013-2016 The rust-url developers | https://github.com/servo/rust-url |
| getrandom | 0.3.4 | MIT OR Apache-2.0 | Copyright (c) 2018-2025 The rust-random Project Developers; Copyright (c) 2014 The Rust Project Developers | https://github.com/rust-random/getrandom |
| getrandom | 0.4.3 | MIT OR Apache-2.0 | Copyright (c) 2018-2026 The rust-random Project Developers; Copyright (c) 2014 The Rust Project Developers | https://github.com/rust-random/getrandom |
| glob | 0.3.4 | MIT OR Apache-2.0 | Copyright (c) 2014 The Rust Project Developers | https://github.com/rust-lang/glob |
| hashbrown | 0.12.3 | MIT OR Apache-2.0 | Copyright (c) 2016 Amanieu d'Antras | https://github.com/rust-lang/hashbrown |
| hashbrown | 0.17.1 | MIT OR Apache-2.0 | Copyright (c) 2016 Amanieu d'Antras | https://github.com/rust-lang/hashbrown |
| hashlink | 0.12.2 | MIT OR Apache-2.0 | LICENSE-MIT: "This work is derived in part from the `linked-hash-map` crate, Copyright (c) 2015 The Rust Project Developers"; no other holder named, no Cargo.toml authors | https://github.com/djc/hashlink |
| heck | 0.5.0 | MIT OR Apache-2.0 | Copyright (c) 2015 The Rust Project Developers | https://github.com/withoutboats/heck |
| hex | 0.4.3 | MIT OR Apache-2.0 | Copyright (c) 2013-2014 The Rust Project Developers.; Copyright (c) 2015-2020 The rust-hex Developers | https://github.com/KokaKiwi/rust-hex |
| hkdf | 0.13.0 | MIT OR Apache-2.0 | Copyright (c) 2015-2018 Vlad Filippov; Copyright (c) 2018-2021 RustCrypto Developers | https://github.com/RustCrypto/KDFs/ |
| hmac | 0.13.0 | MIT OR Apache-2.0 | Copyright (c) 2017 Artyom Pavlov | https://github.com/RustCrypto/MACs |
| html5ever | 0.39.0 | MIT OR Apache-2.0 | Copyright (c) 2014 The html5ever Project Developers | https://github.com/servo/html5ever |
| http | 1.5.0 | MIT OR Apache-2.0 | Copyright 2017 http-rs authors; Copyright (c) 2017 http-rs authors | https://github.com/hyperium/http |
| hybrid-array | 0.4.15 | MIT OR Apache-2.0 | Copyright (c) 2022-2026 The RustCrypto Project Developers | https://github.com/RustCrypto/hybrid-array |
| icu_collections | 2.4.0 | Unicode-3.0 | Copyright © 2020-2024 Unicode, Inc. | https://github.com/unicode-org/icu4x |
| icu_locale_core | 2.4.0 | Unicode-3.0 | Copyright © 2020-2024 Unicode, Inc. | https://github.com/unicode-org/icu4x |
| icu_normalizer | 2.4.0 | Unicode-3.0 | Copyright © 2020-2024 Unicode, Inc. | https://github.com/unicode-org/icu4x |
| icu_normalizer_data | 2.4.0 | Unicode-3.0 | Copyright © 2020-2024 Unicode, Inc. ; NOTE: contains Unicode data files | https://github.com/unicode-org/icu4x |
| icu_properties | 2.4.0 | Unicode-3.0 | Copyright © 2020-2024 Unicode, Inc. | https://github.com/unicode-org/icu4x |
| icu_properties_data | 2.4.0 | Unicode-3.0 | Copyright © 2020-2024 Unicode, Inc. ; NOTE: contains Unicode data files | https://github.com/unicode-org/icu4x |
| icu_provider | 2.4.0 | Unicode-3.0 | Copyright © 2020-2024 Unicode, Inc. | https://github.com/unicode-org/icu4x |
| idna | 1.1.0 | MIT OR Apache-2.0 | Copyright (c) 2013-2025 The rust-url developers | https://github.com/servo/rust-url/ |
| idna_adapter | 1.2.2 | Apache-2.0 OR MIT | Copyright (c) The rust-url developers | https://github.com/hsivonen/idna_adapter |
| indexmap | 1.9.3 | Apache-2.0 OR MIT | Copyright (c) 2016--2017 ; (license file names no holder; no Cargo.toml authors) | https://github.com/bluss/indexmap |
| indexmap | 2.14.2 | Apache-2.0 OR MIT | Copyright (c) 2016--2017 ; (license file names no holder; no Cargo.toml authors) | https://github.com/indexmap-rs/indexmap |
| infer | 0.22.0 | MIT | Copyright (c) 2019 Bojan | https://github.com/bojand/infer |
| inout | 0.2.5 | MIT OR Apache-2.0 | Copyright (c) 2022-2025 The RustCrypto Project Developers; Copyright (c) 2022 Artyom Pavlov | https://github.com/RustCrypto/utils |
| itertools | 0.14.0 | MIT OR Apache-2.0 | Copyright (c) 2015 ; Cargo.toml authors: bluss (license file names no holder) | https://github.com/rust-itertools/itertools |
| itoa | 1.0.18 | MIT OR Apache-2.0 | Cargo.toml authors: David Tolnay (copyright line in the package's LICENSE.spdx: 2021-2023, The Tauri Programme in the Commons Conservancy) | https://github.com/dtolnay/itoa |
| jiff | 0.2.38 | Unlicense OR MIT | Copyright (c) 2015 Andrew Gallant | https://github.com/BurntSushi/jiff |
| jiff-core | 0.1.1 | Unlicense OR MIT | Copyright (c) 2015 Andrew Gallant | https://github.com/BurntSushi/jiff |
| jiff-tzdb | 0.1.9 | Unlicense OR MIT | Copyright (c) 2015 Andrew Gallant ; NOTE: embeds the IANA Time Zone Database (concatenated-zoneinfo.dat); no separate notice in package | https://github.com/BurntSushi/jiff |
| jiff-tzdb-platform | 0.1.3 | Unlicense OR MIT | Copyright (c) 2015 Andrew Gallant | https://github.com/BurntSushi/jiff |
| json-patch | 4.2.0 | MIT/Apache-2.0 | Copyright (c) 2017 Ivan Dubrov | https://github.com/idubrov/json-patch |
| jsonptr | 0.7.1 | MIT OR Apache-2.0 | Copyright 2024 Chance Dinkins; Copyright (c) 2022 Chance Dinkins | https://github.com/chanced/jsonptr |
| keyboard-types | 0.8.3 | MIT OR Apache-2.0 | Copyright (c) 2017 Pyfisch | https://github.com/rust-windowing/keyboard-types |
| lazy_static | 1.5.1 | MIT OR Apache-2.0 | Cargo.toml authors: Marvin Löbel (copyright line in the package's LICENSE.spdx: 2021-2023, The Tauri Programme in the Commons Conservancy) | https://github.com/rust-lang-nursery/lazy-static.rs |
| libc | 0.2.191 | MIT OR Apache-2.0 | Copyright (c) The Rust Project Developers | https://github.com/rust-lang/libc |
| libsqlite3-sys | 0.38.2 | MIT | Copyright (c) 2014 The rusqlite developers ; NOTE: bundles SQLite 3.53.2 amalgamation (sqlite3/sqlite3.c): "The author disclaims copyright to this source code" (public domain); sqlcipher/ dir (Zetetic BSD) is only used with the sqlcipher features, not enabled here | https://github.com/rusqlite/rusqlite |
| litemap | 0.8.3 | Unicode-3.0 | Copyright © 2020-2024 Unicode, Inc. | https://github.com/unicode-org/icu4x |
| lock_api | 0.4.14 | MIT OR Apache-2.0 | Copyright (c) 2016 The Rust Project Developers | https://github.com/Amanieu/parking_lot |
| log | 0.4.34 | MIT OR Apache-2.0 | Copyright (c) 2014 The Rust Project Developers | https://github.com/rust-lang/log |
| markup5ever | 0.39.0 | MIT OR Apache-2.0 | Copyright (c) 2014 The html5ever Project Developers | https://github.com/servo/html5ever |
| memchr | 2.8.3 | Unlicense OR MIT | Copyright (c) 2015 Andrew Gallant | https://github.com/BurntSushi/memchr |
| mime | 0.3.17 | MIT OR Apache-2.0 | Copyright (c) 2014 Sean McArthur | https://github.com/hyperium/mime |
| mio | 1.2.4 | MIT | Copyright (c) 2014 Carl Lerche and other MIO contributors | https://github.com/tokio-rs/mio |
| muda | 0.20.0 | Apache-2.0 OR MIT | Copyright (c) 2022-2022 Tauri Programme within The Commons Conservancy | https://github.com/tauri-apps/muda |
| new_debug_unreachable | 1.0.6 | MIT | Copyright (c) 2015 Jonathan Reem | https://github.com/mbrubeck/rust-debug-unreachable |
| num-conv | 0.2.2 | MIT OR Apache-2.0 | Copyright (c) Jacob Pratt | https://github.com/jhpratt/num-conv |
| num-traits | 0.2.19 | MIT OR Apache-2.0 | Copyright (c) 2014 The Rust Project Developers | https://github.com/rust-num/num-traits |
| once_cell | 1.21.4 | MIT OR Apache-2.0 | Cargo.toml authors: Aleksey Kladov (copyright line in the package's LICENSE.spdx: 2021-2023, The Tauri Programme in the Commons Conservancy) | https://github.com/matklad/once_cell |
| open | 5.4.5 | MIT | Copyright © `2015` `Sebastian Thiel` | https://github.com/Byron/open-rs |
| option-ext | 0.2.0 | MPL-2.0 | Cargo.toml authors: Simon Ochsenreither (copyright line in the package's LICENSE.spdx: 2021-2023, The Tauri Programme in the Commons Conservancy) | https://github.com/soc/option-ext.git |
| parking_lot | 0.12.5 | MIT OR Apache-2.0 | Copyright (c) 2016 The Rust Project Developers | https://github.com/Amanieu/parking_lot |
| parking_lot_core | 0.9.12 | MIT OR Apache-2.0 | Copyright (c) 2016 The Rust Project Developers | https://github.com/Amanieu/parking_lot |
| password-hash | 0.6.1 | MIT OR Apache-2.0 | Copyright (c) 2020-2026 RustCrypto Developers | https://github.com/RustCrypto/traits |
| percent-encoding | 2.3.2 | MIT OR Apache-2.0 | Copyright (c) 2013-2025 The rust-url developers | https://github.com/servo/rust-url/ |
| phc | 0.6.1 | Apache-2.0 OR MIT | Copyright (c) 2021-2026 The RustCrypto Project Developers | https://github.com/RustCrypto/formats |
| phf | 0.13.1 | MIT | Copyright (c) 2014-2022 Steven Fackler, Yuki Okushi | https://github.com/rust-phf/rust-phf |
| phf_shared | 0.13.1 | MIT | Copyright (c) 2014-2022 Steven Fackler, Yuki Okushi | https://github.com/rust-phf/rust-phf |
| pin-project-lite | 0.2.17 | Apache-2.0 OR MIT | none stated in package (no copyright line, no Cargo.toml authors) | https://github.com/taiki-e/pin-project-lite |
| plist | 1.10.3 | MIT | Copyright (c) 2015 Edward Barnard | https://github.com/ebarnard/rust-plist/ |
| poly1305 | 0.9.1 | Apache-2.0 OR MIT | Copyright (c) 2015-2026 The RustCrypto Project Developers | https://github.com/RustCrypto/universal-hashes |
| potential_utf | 0.1.6 | Unicode-3.0 | Copyright © 2020-2024 Unicode, Inc. | https://github.com/unicode-org/icu4x |
| powerfmt | 0.2.1 | MIT OR Apache-2.0 | Copyright 2023 Jacob Pratt et al.; Copyright (c) 2023 Jacob Pratt et al. | https://github.com/jhpratt/powerfmt |
| precomputed-hash | 0.1.1 | MIT | Copyright (c) 2017 Emilio Cobos Álvarez | https://github.com/emilio/precomputed-hash |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 | Cargo.toml authors: David Tolnay, Alex Crichton (copyright line in the package's LICENSE.spdx: 2021-2023, The Tauri Programme in the Commons Conservancy) | https://github.com/dtolnay/proc-macro2 |
| quick-xml | 0.42.0 | MIT | Copyright (c) 2016 Johann Tuffe | https://github.com/tafia/quick-xml |
| quote | 1.0.47 | MIT OR Apache-2.0 | Cargo.toml authors: David Tolnay (copyright line in the package's LICENSE.spdx: 2021-2023, The Tauri Programme in the Commons Conservancy) | https://github.com/dtolnay/quote |
| rand_core | 0.10.1 | MIT OR Apache-2.0 | Copyright (c) 2018-2026 The Rand Project Developers | https://github.com/rust-random/rand_core |
| raw-window-handle | 0.6.2 | MIT OR Apache-2.0 OR Zlib | Copyright (c) 2019 Osspial; Copyright (c) 2020 Osspial | https://github.com/rust-windowing/raw-window-handle |
| rayon | 1.12.0 | MIT OR Apache-2.0 | Copyright (c) 2010 The Rust Project Developers | https://github.com/rayon-rs/rayon |
| rayon-core | 1.13.0 | MIT OR Apache-2.0 | Copyright (c) 2010 The Rust Project Developers | https://github.com/rayon-rs/rayon |
| ref-cast | 1.0.27 | MIT OR Apache-2.0 | Cargo.toml authors: David Tolnay (copyright line in the package's LICENSE.spdx: 2021-2023, The Tauri Programme in the Commons Conservancy) | https://github.com/dtolnay/ref-cast |
| regex | 1.13.1 | MIT OR Apache-2.0 | Copyright (c) 2014 The Rust Project Developers | https://github.com/rust-lang/regex |
| regex-automata | 0.4.18 | MIT OR Apache-2.0 | Copyright (c) 2014 The Rust Project Developers | https://github.com/rust-lang/regex |
| regex-syntax | 0.8.11 | MIT OR Apache-2.0 | Copyright (c) 2014 The Rust Project Developers ; NOTE: also bundles Unicode data tables under "UNICODE, INC. LICENSE AGREEMENT - DATA FILES AND SOFTWARE" (src/unicode_tables/LICENSE-UNICODE) | https://github.com/rust-lang/regex |
| rfd | 0.16.0 | MIT | Copyright (c) 2022 Bartłomiej Maryńczak | https://github.com/PolyMeilex/rfd |
| rusqlite | 0.40.2 | MIT | Copyright (c) 2014 The rusqlite developers | https://github.com/rusqlite/rusqlite |
| rustc-hash | 2.1.3 | Apache-2.0 OR MIT | Cargo.toml authors: The Rust Project Developers (copyright line in the package's LICENSE.spdx: 2021-2023, The Tauri Programme in the Commons Conservancy) | https://github.com/rust-lang/rustc-hash |
| same-file | 1.0.6 | Unlicense/MIT | Copyright (c) 2017 Andrew Gallant | https://github.com/BurntSushi/same-file |
| schemars | 0.8.22 | MIT | Copyright (c) 2019 Graham Esau | https://github.com/GREsau/schemars |
| schemars | 0.9.0 | MIT | Copyright (c) 2019 Graham Esau | https://github.com/GREsau/schemars |
| schemars | 1.2.3 | MIT | Copyright (c) 2019 Graham Esau | https://github.com/GREsau/schemars |
| scopeguard | 1.2.0 | MIT OR Apache-2.0 | Copyright (c) 2016-2019 Ulrik Sverdrup "bluss" and scopeguard developers | https://github.com/bluss/scopeguard |
| selectors | 0.38.0 | MPL-2.0 | Cargo.toml authors: The Servo Project Developers (copyright line in the package's LICENSE.spdx: 2021-2023, The Tauri Programme in the Commons Conservancy) | https://github.com/servo/stylo |
| semver | 1.0.28 | MIT OR Apache-2.0 | Cargo.toml authors: David Tolnay (copyright line in the package's LICENSE.spdx: 2021-2023, The Tauri Programme in the Commons Conservancy) | https://github.com/dtolnay/semver |
| serde | 1.0.229 | MIT OR Apache-2.0 | Cargo.toml authors: Erick Tryzelaar, David Tolnay (copyright line in the package's LICENSE.spdx: 2021-2023, The Tauri Programme in the Commons Conservancy) | https://github.com/serde-rs/serde |
| serde-untagged | 0.1.9 | MIT OR Apache-2.0 | Cargo.toml authors: David Tolnay (copyright line in the package's LICENSE.spdx: 2021-2023, The Tauri Programme in the Commons Conservancy) | https://github.com/dtolnay/serde-untagged |
| serde_core | 1.0.229 | MIT OR Apache-2.0 | Cargo.toml authors: Erick Tryzelaar, David Tolnay (copyright line in the package's LICENSE.spdx: 2021-2023, The Tauri Programme in the Commons Conservancy) | https://github.com/serde-rs/serde |
| serde_json | 1.0.154 | MIT OR Apache-2.0 | none stated in package (no copyright line, no Cargo.toml authors) | https://github.com/serde-rs/json |
| serde_spanned | 1.1.2 | MIT OR Apache-2.0 | Copyright Individual contributors; Copyright (c) Individual contributors | https://github.com/toml-rs/toml |
| serde_with | 3.24.0 | MIT OR Apache-2.0 | Copyright (c) 2015 ; Cargo.toml authors: Jonas Bushart, Marcin Kaźmierczak (license file names no holder) | https://github.com/jonasbb/serde_with/ |
| serialize-to-javascript | 0.1.2 | MIT OR Apache-2.0 | Copyright (c) 2021 Chip Reed | https://github.com/chippers/serialize-to-javascript |
| servo_arc | 0.4.3 | MIT OR Apache-2.0 | Cargo.toml authors: The Servo Project Developers (copyright line in the package's LICENSE.spdx: 2021-2023, The Tauri Programme in the Commons Conservancy) | https://github.com/servo/stylo |
| sha2 | 0.11.1 | MIT OR Apache-2.0 | Copyright (c) 2016-2026 The RustCrypto Project Developers; Copyright (c) 2016 Artyom Pavlov; Copyright (c) 2009-2013 Mozilla Foundation; Copyright (c) 2006-2009 Graydon Hoare | https://github.com/RustCrypto/hashes |
| siphasher | 1.0.4 | MIT OR Apache-2.0 | Copyright 2012-2016 The Rust Project Developers.; Copyright 2016-2026 Frank Denis. | https://github.com/jedisct1/rust-siphash |
| smallvec | 1.16.3 | MIT OR Apache-2.0 | Copyright (c) 2018 The Servo Project Developers | https://github.com/servo/rust-smallvec |
| socket2 | 0.6.5 | MIT OR Apache-2.0 | Copyright (c) 2014 Alex Crichton | https://github.com/rust-lang/socket2 |
| softbuffer | 0.4.8 | MIT OR Apache-2.0 | Copyright 2022 Kirill Chibisov | https://github.com/rust-windowing/softbuffer |
| stable_deref_trait | 1.2.1 | MIT OR Apache-2.0 | Copyright (c) 2017 Robert Grosse | https://github.com/storyyeller/stable_deref_trait |
| string_cache | 0.9.0 | MIT OR Apache-2.0 | Copyright (c) 2012-2013 Mozilla Foundation | https://github.com/servo/string-cache |
| tao | 0.37.1 | Apache-2.0 | Cargo.toml authors: Tauri Programme within The Commons Conservancy, The winit contributors (copyright line in the package's LICENSE.spdx: 2021-2023, The Tauri Programme in the Commons Conservancy) | https://github.com/tauri-apps/tao |
| tauri | 2.12.5 | Apache-2.0 OR MIT | Copyright (c) 2017 - Present Tauri Apps Contributors | https://github.com/tauri-apps/tauri |
| tauri-plugin-dialog | 2.8.3 | Apache-2.0 OR MIT | Copyright (c) 2017 - Present Tauri Apps Contributors | https://github.com/tauri-apps/plugins-workspace |
| tauri-plugin-fs | 2.7.1 | Apache-2.0 OR MIT | Copyright (c) 2017 - Present Tauri Apps Contributors | https://github.com/tauri-apps/plugins-workspace |
| tauri-plugin-opener | 2.7.2 | Apache-2.0 OR MIT | Copyright (c) 2017 - Present Tauri Apps Contributors | https://github.com/tauri-apps/plugins-workspace |
| tauri-plugin-single-instance | 2.5.4 | Apache-2.0 OR MIT | Copyright (c) 2017 - Present The Tauri Programme in the Commons Conservancy | https://github.com/tauri-apps/plugins-workspace |
| tauri-runtime | 2.12.2 | Apache-2.0 OR MIT | Copyright (c) 2017 - Present Tauri Apps Contributors | https://github.com/tauri-apps/tauri |
| tauri-runtime-wry | 2.12.3 | Apache-2.0 OR MIT | Copyright (c) 2017 - Present Tauri Apps Contributors | https://github.com/tauri-apps/tauri |
| tauri-utils | 2.10.2 | Apache-2.0 OR MIT | Copyright (c) 2017 - Present Tauri Apps Contributors | https://github.com/tauri-apps/tauri |
| tendril | 0.5.1 | MIT OR Apache-2.0 | Copyright (c) 2015 Keegan McAllister | https://github.com/servo/html5ever |
| thiserror | 2.0.21 | MIT OR Apache-2.0 | Cargo.toml authors: David Tolnay (copyright line in the package's LICENSE.spdx: 2021-2023, The Tauri Programme in the Commons Conservancy) | https://github.com/dtolnay/thiserror |
| time | 0.3.55 | MIT OR Apache-2.0 | Copyright (c) Jacob Pratt et al. | https://github.com/time-rs/time |
| time-core | 0.1.9 | MIT OR Apache-2.0 | Copyright (c) Jacob Pratt et al. | https://github.com/time-rs/time |
| tinystr | 0.8.5 | Unicode-3.0 | Copyright © 2020-2024 Unicode, Inc. | https://github.com/unicode-org/icu4x |
| tinyvec | 1.13.3 | Zlib OR Apache-2.0 OR MIT | Copyright (c) 2019 Daniel "Lokathor" Gee. | https://github.com/Lokathor/tinyvec |
| tokio | 1.54.1 | MIT | Copyright (c) Tokio Contributors | https://github.com/tokio-rs/tokio |
| toml | 1.1.8+spec-1.1.0 | MIT OR Apache-2.0 | Copyright Individual contributors; Copyright (c) Individual contributors | https://github.com/toml-rs/toml |
| toml_datetime | 1.1.2+spec-1.1.0 | MIT OR Apache-2.0 | Copyright Individual contributors; Copyright (c) Individual contributors | https://github.com/toml-rs/toml |
| toml_parser | 1.1.5+spec-1.1.0 | MIT OR Apache-2.0 | Copyright Individual contributors; Copyright (c) Individual contributors | https://github.com/toml-rs/toml |
| toml_writer | 1.1.3+spec-1.1.0 | MIT OR Apache-2.0 | Copyright Individual contributors; Copyright (c) Individual contributors | https://github.com/toml-rs/toml |
| tracing | 0.1.44 | MIT | Copyright (c) 2019 Tokio Contributors | https://github.com/tokio-rs/tracing |
| tracing-core | 0.1.36 | MIT | Copyright (c) 2019 Tokio Contributors ; NOTE: also embeds MIT-licensed code in src/spin/: "Copyright (c) 2014 Mathijs van de Nes" | https://github.com/tokio-rs/tracing |
| tray-icon | 0.25.1 | MIT OR Apache-2.0 | Copyright (c) 2022-2022 Tauri Programme within The Commons Conservancy | https://github.com/tauri-apps/tray-icon |
| typeid | 1.0.3 | MIT OR Apache-2.0 | Cargo.toml authors: David Tolnay (copyright line in the package's LICENSE.spdx: 2021-2023, The Tauri Programme in the Commons Conservancy) | https://github.com/dtolnay/typeid |
| typenum | 1.20.1 | MIT OR Apache-2.0 | Copyright 2014 Paho Lurie-Gregg; Copyright (c) 2014 Paho Lurie-Gregg | https://github.com/paholg/typenum |
| unicode-ident | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 | Copyright © 1991-2023 Unicode, Inc. | https://github.com/dtolnay/unicode-ident |
| unicode-normalization | 0.1.26 | (MIT OR Apache-2.0) AND Unicode-3.0 | Copyright (c) 2015 The Rust Project Developers; Copyright © 1991-2023 Unicode, Inc. | https://github.com/unicode-rs/unicode-normalization |
| unicode-segmentation | 1.13.3 | MIT OR Apache-2.0 | Copyright (c) 2015 The Rust Project Developers | https://github.com/unicode-rs/unicode-segmentation |
| universal-hash | 0.6.1 | MIT OR Apache-2.0 | Copyright (c) 2019-2025 RustCrypto Developers | https://github.com/RustCrypto/traits |
| url | 2.5.8 | MIT OR Apache-2.0 | Copyright (c) 2013-2025 The rust-url developers | https://github.com/servo/rust-url |
| urlpattern | 0.6.0 | MIT | Copyright (c) 2021 the Deno authors | https://github.com/denoland/rust-urlpattern |
| utf8_iter | 1.0.4 | Apache-2.0 OR MIT | Copyright Mozilla Foundation | https://github.com/hsivonen/utf8_iter |
| uuid | 1.28.0 | Apache-2.0 OR MIT | Copyright (c) 2014 The Rust Project Developers; Copyright (c) 2018 Ashley Mannix, Christopher Armstrong, Dylan DPC, Hunar Roop Kahlon | https://github.com/uuid-rs/uuid |
| walkdir | 2.5.0 | Unlicense/MIT | Copyright (c) 2015 Andrew Gallant | https://github.com/BurntSushi/walkdir |
| web-time | 1.1.0 | MIT OR Apache-2.0 | Copyright 2023 dAxpeDDa; Copyright (c) 2023 dAxpeDDa | https://github.com/daxpedda/web-time |
| web_atoms | 0.2.6 | MIT OR Apache-2.0 | Copyright (c) 2014 The html5ever Project Developers | https://github.com/servo/html5ever |
| webview2-com | 0.39.1 | MIT | NOTE: package ships no license file; upstream github.com/wravery/webview2-rs LICENSE (MIT): "Copyright (c) 2021 Bill Avery" [fetched] | https://github.com/wravery/webview2-rs |
| webview2-com-sys | 0.39.1 | MIT | NOTE: package ships no license file; upstream github.com/wravery/webview2-rs LICENSE (MIT): "Copyright (c) 2021 Bill Avery" [fetched] | https://github.com/wravery/webview2-rs |
| winapi-util | 0.1.11 | Unlicense OR MIT | Copyright (c) 2017 Andrew Gallant | https://github.com/BurntSushi/winapi-util |
| window-vibrancy | 0.8.1 | Apache-2.0 OR MIT | Copyright (c) 2020-2022 Tauri Programme within The Commons Conservancy | https://github.com/tauri-apps/tauri-plugin-vibrancy |
| windows | 0.62.2 | MIT OR Apache-2.0 | Copyright (c) Microsoft Corporation. | https://github.com/microsoft/windows-rs |
| windows-collections | 0.3.2 | MIT OR Apache-2.0 | Copyright (c) Microsoft Corporation. | https://github.com/microsoft/windows-rs |
| windows-core | 0.62.2 | MIT OR Apache-2.0 | Copyright (c) Microsoft Corporation. | https://github.com/microsoft/windows-rs |
| windows-future | 0.3.2 | MIT OR Apache-2.0 | Copyright (c) Microsoft Corporation. | https://github.com/microsoft/windows-rs |
| windows-link | 0.2.1 | MIT OR Apache-2.0 | Copyright (c) Microsoft Corporation. | https://github.com/microsoft/windows-rs |
| windows-numerics | 0.3.1 | MIT OR Apache-2.0 | Copyright (c) Microsoft Corporation. | https://github.com/microsoft/windows-rs |
| windows-result | 0.4.1 | MIT OR Apache-2.0 | Copyright (c) Microsoft Corporation. | https://github.com/microsoft/windows-rs |
| windows-strings | 0.5.1 | MIT OR Apache-2.0 | Copyright (c) Microsoft Corporation. | https://github.com/microsoft/windows-rs |
| windows-sys | 0.60.2 | MIT OR Apache-2.0 | Copyright (c) Microsoft Corporation. | https://github.com/microsoft/windows-rs |
| windows-sys | 0.61.2 | MIT OR Apache-2.0 | Copyright (c) Microsoft Corporation. | https://github.com/microsoft/windows-rs |
| windows-targets | 0.53.5 | MIT OR Apache-2.0 | Copyright (c) Microsoft Corporation. | https://github.com/microsoft/windows-rs |
| windows-threading | 0.2.1 | MIT OR Apache-2.0 | Copyright (c) Microsoft Corporation. | https://github.com/microsoft/windows-rs |
| windows-version | 0.1.7 | MIT OR Apache-2.0 | Copyright (c) Microsoft Corporation. | https://github.com/microsoft/windows-rs |
| windows_x86_64_msvc | 0.53.1 | MIT OR Apache-2.0 | Copyright (c) Microsoft Corporation. | https://github.com/microsoft/windows-rs |
| winnow | 1.0.4 | MIT | none stated in package (no copyright line, no Cargo.toml authors) | https://github.com/winnow-rs/winnow |
| writeable | 0.6.4 | Unicode-3.0 | Copyright © 2020-2024 Unicode, Inc. | https://github.com/unicode-org/icu4x |
| wry | 0.57.0 | Apache-2.0 OR MIT | Copyright (c) 2020-2023 Ngo Iok Ui & Tauri Programme within The Commons Conservancy | https://github.com/tauri-apps/wry |
| yoke | 0.8.4 | Unicode-3.0 | Copyright © 2020-2024 Unicode, Inc. | https://github.com/unicode-org/icu4x |
| zerofrom | 0.1.8 | Unicode-3.0 | Copyright © 2020-2024 Unicode, Inc. | https://github.com/unicode-org/icu4x |
| zeroize | 1.9.1 | Apache-2.0 OR MIT | Copyright (c) 2018-2026 The RustCrypto Project Developers | https://github.com/RustCrypto/utils |
| zerotrie | 0.2.5 | Unicode-3.0 | Copyright © 2020-2024 Unicode, Inc. | https://github.com/unicode-org/icu4x |
| zerovec | 0.11.9 | Unicode-3.0 | Copyright © 2020-2024 Unicode, Inc. | https://github.com/unicode-org/icu4x |
| zmij | 1.0.23 | MIT | Cargo.toml authors: David Tolnay (copyright line in the package's LICENSE.spdx: 2021-2023, The Tauri Programme in the Commons Conservancy) | https://github.com/dtolnay/zmij |
| zxcvbn | 3.1.1 | MIT | Copyright (c) 2016 Joshua Holmer ; NOTE: frequency lists derived from US census data, Wikipedia, TV/movie subtitles (per README); no separate notice in package | https://github.com/shssoichiro/zxcvbn-rs |

### Rust crates used only at build time (not shipped)

76 crates are used only to build Notaya (build scripts, build dependencies and procedural macros). They are not linked into the exe, but code they generate is compiled into it, so they are listed with their licenses. Their copyright lines are the authors named in each crate's Cargo.toml; the license texts are the standard texts in the License texts section (MIT, Apache-2.0, Unicode-3.0, Zlib and so on).

| Crate | Version | License | Authors (Cargo.toml) | Source |
|---|---|---|---|---|
| adler2 | 2.0.1 | 0BSD OR MIT OR Apache-2.0 | Jonas Schievink, oyvindln | https://github.com/oyvindln/adler2 |
| autocfg | 1.5.1 | Apache-2.0 OR MIT | Josh Stone | https://github.com/cuviper/autocfg |
| block-buffer | 0.10.4 | MIT OR Apache-2.0 | RustCrypto Developers | https://github.com/RustCrypto/utils |
| byteorder | 1.5.0 | Unlicense OR MIT | Andrew Gallant | https://github.com/BurntSushi/byteorder |
| cargo_toml | 1.0.1 | Apache-2.0 OR MIT | Kornel | https://gitlab.com/lib.rs/cargo_toml |
| cc | 1.9.0 | MIT OR Apache-2.0 | - | https://github.com/rust-lang/cc-rs |
| cpufeatures | 0.2.17 | MIT OR Apache-2.0 | RustCrypto Developers | https://github.com/RustCrypto/utils |
| crc32fast | 1.5.3 | MIT OR Apache-2.0 | Sam Rijs, Alex Crichton | https://github.com/srijs/rust-crc32fast |
| crypto-common | 0.1.7 | MIT OR Apache-2.0 | RustCrypto Developers | https://github.com/RustCrypto/traits |
| cssparser-macros | 0.7.1 | MPL-2.0 | Simon Sapin | https://github.com/servo/rust-cssparser |
| darling | 0.20.11 | MIT | Ted Driggs | https://github.com/TedDriggs/darling |
| darling | 0.24.1 | MIT | Ted Driggs | https://github.com/TedDriggs/darling |
| darling_core | 0.20.11 | MIT | Ted Driggs | https://github.com/TedDriggs/darling |
| darling_core | 0.24.1 | MIT | Ted Driggs | https://github.com/TedDriggs/darling |
| darling_macro | 0.20.11 | MIT | Ted Driggs | https://github.com/TedDriggs/darling |
| darling_macro | 0.24.1 | MIT | Ted Driggs | https://github.com/TedDriggs/darling |
| defmt-macros | 1.2.0 | MIT OR Apache-2.0 | The Knurling-rs developers | https://github.com/knurling-rs/defmt |
| defmt-parser | 1.1.0 | MIT OR Apache-2.0 | The Knurling-rs developers | https://github.com/knurling-rs/defmt |
| derive_builder_core | 0.20.2 | MIT OR Apache-2.0 | Colin Kiegel, Pascal Hertleif, Jan-Erik Rediger, Ted Driggs | https://github.com/colin-kiegel/rust-derive-builder |
| derive_builder_macro | 0.20.2 | MIT OR Apache-2.0 | Colin Kiegel, Pascal Hertleif, Jan-Erik Rediger, Ted Driggs | https://github.com/colin-kiegel/rust-derive-builder |
| derive_more-impl | 2.1.1 | MIT | Jelte Fennema | https://github.com/JelteF/derive_more |
| digest | 0.10.7 | MIT OR Apache-2.0 | RustCrypto Developers | https://github.com/RustCrypto/traits |
| displaydoc | 0.2.7 | MIT OR Apache-2.0 | Jane Lusby | https://github.com/yaahc/displaydoc |
| embed-resource | 3.0.12 | MIT | наб, Cat Plus Plus, Liigo, azyobuzin, Peter Atashian, pravic, Gabriel Majeri, SonnyX, Johan Andersson, Jordan Poles, MSx | https://github.com/nabijaczleweli/rust-embed-resource |
| fastrand | 2.5.0 | Apache-2.0 OR MIT | Stjepan Glavina | https://github.com/smol-rs/fastrand |
| fdeflate | 0.3.7 | MIT OR Apache-2.0 | The image-rs Developers | https://github.com/image-rs/fdeflate |
| find-msvc-tools | 0.1.15 | MIT OR Apache-2.0 | - | https://github.com/rust-lang/cc-rs |
| flate2 | 1.1.10 | MIT OR Apache-2.0 | Alex Crichton, Josh Triplett | https://github.com/rust-lang/flate2-rs |
| generic-array | 0.14.7 | MIT | Bartłomiej Kamiński, Aaron Trent | https://github.com/fizyk20/generic-array.git |
| ico | 0.6.0 | MIT | Matthew D. Steele | https://github.com/mdsteele/rust-ico |
| ident_case | 1.0.1 | MIT/Apache-2.0 | Ted Driggs | https://github.com/TedDriggs/ident_case |
| miniz_oxide | 0.8.9 | MIT OR Zlib OR Apache-2.0 | Frommi, oyvindln, Rich Geldreich richgel99@gmail.com | https://github.com/Frommi/miniz_oxide/tree/master/miniz_oxide |
| miniz_oxide | 0.9.1 | MIT OR Zlib OR Apache-2.0 | Frommi, oyvindln, Rich Geldreich richgel99@gmail.com | https://github.com/Frommi/miniz_oxide/tree/master/miniz_oxide |
| phf_codegen | 0.13.1 | MIT | Steven Fackler | https://github.com/rust-phf/rust-phf |
| phf_generator | 0.13.1 | MIT | Steven Fackler | https://github.com/rust-phf/rust-phf |
| phf_macros | 0.13.1 | MIT | Steven Fackler | https://github.com/rust-phf/rust-phf |
| pkg-config | 0.3.34 | MIT OR Apache-2.0 | Alex Crichton | https://github.com/rust-lang/pkg-config-rs |
| png | 0.18.1 | MIT OR Apache-2.0 | The image-rs Developers | https://github.com/image-rs/image-png |
| ref-cast-impl | 1.0.27 | MIT OR Apache-2.0 | David Tolnay | https://github.com/dtolnay/ref-cast |
| rustc_version | 0.4.1 | MIT OR Apache-2.0 | - | https://github.com/djc/rustc-version-rs |
| schemars_derive | 0.8.22 | MIT | Graham Esau | https://github.com/GREsau/schemars |
| serde_derive | 1.0.229 | MIT OR Apache-2.0 | Erick Tryzelaar, David Tolnay | https://github.com/serde-rs/serde |
| serde_derive_internals | 0.29.1 | MIT OR Apache-2.0 | Erick Tryzelaar, David Tolnay | https://github.com/serde-rs/serde |
| serde_repr | 0.1.21 | MIT OR Apache-2.0 | David Tolnay | https://github.com/dtolnay/serde-repr |
| serde_with_macros | 3.24.0 | MIT OR Apache-2.0 | Jonas Bushart | https://github.com/jonasbb/serde_with/ |
| serialize-to-javascript-impl | 0.1.2 | MIT OR Apache-2.0 | Chip Reed | https://github.com/chippers/serialize-to-javascript |
| sha2 | 0.10.9 | MIT OR Apache-2.0 | RustCrypto Developers | https://github.com/RustCrypto/hashes |
| shlex | 2.0.1 | MIT OR Apache-2.0 | comex, Fenhl, Adrian Taylor, Alex Touchet, Daniel Parks, Garrett Berg | https://github.com/comex/rust-shlex |
| simd-adler32 | 0.3.10 | MIT | Marvin Countryman | https://github.com/mcountryman/simd-adler32 |
| static_vcruntime | 3.0.0 | MIT OR Apache-2.0 OR Zlib | - | https://github.com/ChrisDenton/static_vcruntime |
| string_cache_codegen | 0.6.1 | MIT OR Apache-2.0 | The Servo Project Developers | https://github.com/servo/string-cache |
| strsim | 0.11.1 | MIT | Danny Guo, maxbachmann | https://github.com/rapidfuzz/strsim-rs |
| syn | 2.0.119 | MIT OR Apache-2.0 | David Tolnay | https://github.com/dtolnay/syn |
| syn | 3.0.9 | MIT OR Apache-2.0 | David Tolnay | https://github.com/dtolnay/syn |
| synstructure | 0.14.0 | MIT | Nika Layzell | https://github.com/mystor/synstructure |
| tauri-build | 2.7.4 | Apache-2.0 OR MIT | Tauri Programme within The Commons Conservancy | https://github.com/tauri-apps/tauri |
| tauri-codegen | 2.7.4 | Apache-2.0 OR MIT | Tauri Programme within The Commons Conservancy | https://github.com/tauri-apps/tauri |
| tauri-macros | 2.7.4 | Apache-2.0 OR MIT | Tauri Programme within The Commons Conservancy | https://github.com/tauri-apps/tauri |
| tauri-plugin | 2.7.2 | Apache-2.0 OR MIT | Tauri Programme within The Commons Conservancy | https://github.com/tauri-apps/tauri |
| tauri-winres | 0.3.7 | MIT | Tauri Programme within The Commons Conservancy, Max Resch | https://github.com/tauri-apps/winres |
| thiserror-impl | 2.0.21 | MIT OR Apache-2.0 | David Tolnay | https://github.com/dtolnay/thiserror |
| time-macros | 0.2.32 | MIT OR Apache-2.0 | Jacob Pratt, Time contributors | https://github.com/time-rs/time |
| tracing-attributes | 0.1.31 | MIT | Tokio Contributors, Eliza Weisman, David Barsky | https://github.com/tokio-rs/tracing |
| vcpkg | 0.2.15 | MIT/Apache-2.0 | Jim McGrath | https://github.com/mcgoo/vcpkg-rs |
| version_check | 0.9.5 | MIT/Apache-2.0 | Sergio Benitez | https://github.com/SergioBenitez/version_check |
| vswhom | 0.1.0 | MIT | nabijaczleweli | https://github.com/nabijaczleweli/vswhom.rs |
| vswhom-sys | 0.1.3 | MIT | наб, forrestsmithfb | https://github.com/nabijaczleweli/vswhom-sys.rs |
| webview2-com-macros | 0.8.1 | MIT | - | https://github.com/wravery/webview2-rs |
| windows-implement | 0.60.2 | MIT OR Apache-2.0 | - | https://github.com/microsoft/windows-rs |
| windows-interface | 0.59.3 | MIT OR Apache-2.0 | - | https://github.com/microsoft/windows-rs |
| winreg | 0.56.0 | MIT | Igor Shaula | https://github.com/gentoo90/winreg-rs |
| yoke-derive | 0.8.4 | Unicode-3.0 | Manish Goregaokar | https://github.com/unicode-org/icu4x |
| zerofrom-derive | 0.1.8 | Unicode-3.0 | Manish Goregaokar | https://github.com/unicode-org/icu4x |
| zeroize_derive | 1.5.0 | Apache-2.0 OR MIT | The RustCrypto Project Developers | https://github.com/RustCrypto/utils |
| zerovec-derive | 0.11.7 | Unicode-3.0 | Manish Goregaokar | https://github.com/unicode-org/icu4x |
| zlib-rs | 0.6.8 | Zlib | - | https://github.com/trifectatechfoundation/zlib-rs |

## npm packages

The user interface is bundled into the exe. These 115 rows are the packages whose code (or fonts and CSS) are in the shipped bundle: 110 JavaScript packages (highlight.js is counted twice because two versions are bundled), 2 font packages, and 2 direct dependencies (`radix-ui`, `@tiptap/pm`) that only re-export other packages. Tailwind CSS has its own row because its generated CSS is in the shipped stylesheet. The bundler removes license comments, so these notices are supplied here.

| Package | Version | License | Copyright | Source URL |
|---|---|---|---|---|
| @dnd-kit/accessibility | 3.1.1 | MIT | Copyright (c) 2021, Claudéric Demers | https://github.com/clauderic/dnd-kit |
| @dnd-kit/core | 6.3.1 | MIT | Copyright (c) 2021, Claudéric Demers | https://github.com/clauderic/dnd-kit |
| @dnd-kit/sortable | 10.0.0 | MIT | Copyright (c) 2021, Claudéric Demers | https://github.com/clauderic/dnd-kit |
| @dnd-kit/utilities | 3.2.2 | MIT | Copyright (c) 2021, Claudéric Demers | https://github.com/clauderic/dnd-kit |
| @floating-ui/core | 1.8.0 | MIT | Copyright (c) 2021-present Floating UI contributors | https://github.com/floating-ui/floating-ui |
| @floating-ui/dom | 1.8.0 | MIT | Copyright (c) 2021-present Floating UI contributors | https://github.com/floating-ui/floating-ui |
| @floating-ui/react-dom | 2.1.9 | MIT | Copyright (c) 2021-present Floating UI contributors | https://github.com/floating-ui/floating-ui |
| @floating-ui/utils | 0.2.12 | MIT | Copyright (c) 2021-present Floating UI contributors | https://github.com/floating-ui/floating-ui |
| @fontsource-variable/inter | 5.3.0 | OFL-1.1 | Copyright 2016 The Inter Project Authors (https://github.com/rsms/inter) | https://github.com/fontsource/font-files |
| @fontsource-variable/jetbrains-mono | 5.3.0 | OFL-1.1 | Copyright 2020 The JetBrains Mono Project Authors (https://github.com/JetBrains/JetBrainsMono) | https://github.com/fontsource/font-files |
| @radix-ui/number | 1.1.3 | MIT | Copyright (c) 2022 WorkOS | https://github.com/radix-ui/primitives |
| @radix-ui/primitive | 1.2.0 | MIT | Copyright (c) 2022 WorkOS | https://github.com/radix-ui/primitives |
| @radix-ui/react-alert-dialog | 1.2.1 | MIT | Copyright (c) 2022 WorkOS | https://github.com/radix-ui/primitives |
| @radix-ui/react-collection | 1.2.0 | MIT | Copyright (c) 2022 WorkOS | https://github.com/radix-ui/primitives |
| @radix-ui/react-compose-refs | 1.1.5 | MIT | Copyright (c) 2022 WorkOS | https://github.com/radix-ui/primitives |
| @radix-ui/react-context | 1.3.0 | MIT | Copyright (c) 2022 WorkOS | https://github.com/radix-ui/primitives |
| @radix-ui/react-dialog | 1.3.1 | MIT | Copyright (c) 2022 WorkOS | https://github.com/radix-ui/primitives |
| @radix-ui/react-direction | 1.2.0 | MIT | Copyright (c) 2022 WorkOS | https://github.com/radix-ui/primitives |
| @radix-ui/react-dismissable-layer | 1.2.1 | MIT | Copyright (c) 2022 WorkOS | https://github.com/radix-ui/primitives |
| @radix-ui/react-dropdown-menu | 2.2.1 | MIT | Copyright (c) 2022 WorkOS | https://github.com/radix-ui/primitives |
| @radix-ui/react-focus-guards | 1.2.0 | MIT | Copyright (c) 2022 WorkOS | https://github.com/radix-ui/primitives |
| @radix-ui/react-focus-scope | 1.3.1 | MIT | Copyright (c) 2022 WorkOS | https://github.com/radix-ui/primitives |
| @radix-ui/react-id | 1.1.4 | MIT | Copyright (c) 2022 WorkOS | https://github.com/radix-ui/primitives |
| @radix-ui/react-menu | 2.2.1 | MIT | Copyright (c) 2022 WorkOS | https://github.com/radix-ui/primitives |
| @radix-ui/react-popover | 1.3.1 | MIT | Copyright (c) 2022 WorkOS | https://github.com/radix-ui/primitives |
| @radix-ui/react-popper | 1.4.0 | MIT | Copyright (c) 2022 WorkOS | https://github.com/radix-ui/primitives |
| @radix-ui/react-portal | 1.2.0 | MIT | Copyright (c) 2022 WorkOS | https://github.com/radix-ui/primitives |
| @radix-ui/react-presence | 1.1.11 | MIT | Copyright (c) 2022 WorkOS | https://github.com/radix-ui/primitives |
| @radix-ui/react-primitive | 2.2.0 | MIT | Copyright (c) 2022 WorkOS | https://github.com/radix-ui/primitives |
| @radix-ui/react-roving-focus | 1.2.1 | MIT | Copyright (c) 2022 WorkOS | https://github.com/radix-ui/primitives |
| @radix-ui/react-slider | 1.6.1 | MIT | Copyright (c) 2022 WorkOS | https://github.com/radix-ui/primitives |
| @radix-ui/react-slot | 1.5.0 | MIT | Copyright (c) 2022 WorkOS | https://github.com/radix-ui/primitives |
| @radix-ui/react-switch | 1.4.1 | MIT | Copyright (c) 2022 WorkOS | https://github.com/radix-ui/primitives |
| @radix-ui/react-tooltip | 1.4.1 | MIT | Copyright (c) 2022 WorkOS | https://github.com/radix-ui/primitives |
| @radix-ui/react-use-callback-ref | 1.1.5 | MIT | Copyright (c) 2022 WorkOS | https://github.com/radix-ui/primitives |
| @radix-ui/react-use-controllable-state | 1.3.0 | MIT | Copyright (c) 2022 WorkOS | https://github.com/radix-ui/primitives |
| @radix-ui/react-use-effect-event | 0.0.5 | MIT | Copyright (c) 2022 WorkOS | https://github.com/radix-ui/primitives |
| @radix-ui/react-use-is-hydrated | 0.1.3 | MIT | Copyright (c) 2022 WorkOS | https://github.com/radix-ui/primitives |
| @radix-ui/react-use-layout-effect | 1.1.4 | MIT | Copyright (c) 2022 WorkOS | https://github.com/radix-ui/primitives |
| @radix-ui/react-use-size | 1.1.5 | MIT | Copyright (c) 2022 WorkOS | https://github.com/radix-ui/primitives |
| @radix-ui/react-visually-hidden | 1.2.13 | MIT | Copyright (c) 2022 WorkOS | https://github.com/radix-ui/primitives |
| @tauri-apps/api | 2.12.4 | MIT OR Apache-2.0 (listed as MIT) | Copyright (c) 2017 - Present Tauri Apps Contributors | https://github.com/tauri-apps/tauri |
| @tiptap/core | 3.32.0 | MIT | Copyright (c) 2025, Tiptap GmbH | https://github.com/ueberdosis/tiptap |
| @tiptap/extension-blockquote | 3.32.0 | MIT | Copyright (c) 2025, Tiptap GmbH | https://github.com/ueberdosis/tiptap |
| @tiptap/extension-bold | 3.32.0 | MIT | Copyright (c) 2025, Tiptap GmbH | https://github.com/ueberdosis/tiptap |
| @tiptap/extension-bubble-menu | 3.32.0 | MIT | Copyright (c) 2025, Tiptap GmbH | https://github.com/ueberdosis/tiptap |
| @tiptap/extension-code | 3.32.0 | MIT | Copyright (c) 2025, Tiptap GmbH | https://github.com/ueberdosis/tiptap |
| @tiptap/extension-code-block | 3.32.0 | MIT | Copyright (c) 2025, Tiptap GmbH | https://github.com/ueberdosis/tiptap |
| @tiptap/extension-code-block-lowlight | 3.32.0 | MIT | Copyright (c) 2025, Tiptap GmbH | https://github.com/ueberdosis/tiptap |
| @tiptap/extension-document | 3.32.0 | MIT | Copyright (c) 2025, Tiptap GmbH | https://github.com/ueberdosis/tiptap |
| @tiptap/extension-floating-menu | 3.32.0 | MIT | Copyright (c) 2025, Tiptap GmbH | https://github.com/ueberdosis/tiptap |
| @tiptap/extension-hard-break | 3.32.0 | MIT | Copyright (c) 2025, Tiptap GmbH | https://github.com/ueberdosis/tiptap |
| @tiptap/extension-heading | 3.32.0 | MIT | Copyright (c) 2025, Tiptap GmbH | https://github.com/ueberdosis/tiptap |
| @tiptap/extension-highlight | 3.32.0 | MIT | Copyright (c) 2025, Tiptap GmbH | https://github.com/ueberdosis/tiptap |
| @tiptap/extension-horizontal-rule | 3.32.0 | MIT | Copyright (c) 2025, Tiptap GmbH | https://github.com/ueberdosis/tiptap |
| @tiptap/extension-image | 3.32.0 | MIT | Copyright (c) 2025, Tiptap GmbH | https://github.com/ueberdosis/tiptap |
| @tiptap/extension-italic | 3.32.0 | MIT | Copyright (c) 2025, Tiptap GmbH | https://github.com/ueberdosis/tiptap |
| @tiptap/extension-link | 3.32.0 | MIT | Copyright (c) 2025, Tiptap GmbH | https://github.com/ueberdosis/tiptap |
| @tiptap/extension-list | 3.32.0 | MIT | Copyright (c) 2025, Tiptap GmbH | https://github.com/ueberdosis/tiptap |
| @tiptap/extension-paragraph | 3.32.0 | MIT | Copyright (c) 2025, Tiptap GmbH | https://github.com/ueberdosis/tiptap |
| @tiptap/extension-strike | 3.32.0 | MIT | Copyright (c) 2025, Tiptap GmbH | https://github.com/ueberdosis/tiptap |
| @tiptap/extension-table | 3.32.0 | MIT | Copyright (c) 2025, Tiptap GmbH | https://github.com/ueberdosis/tiptap |
| @tiptap/extension-text | 3.32.0 | MIT | Copyright (c) 2025, Tiptap GmbH | https://github.com/ueberdosis/tiptap |
| @tiptap/extension-text-align | 3.32.0 | MIT | Copyright (c) 2025, Tiptap GmbH | https://github.com/ueberdosis/tiptap |
| @tiptap/extension-text-style | 3.32.0 | MIT | Copyright (c) 2025, Tiptap GmbH | https://github.com/ueberdosis/tiptap |
| @tiptap/extension-underline | 3.32.0 | MIT | Copyright (c) 2025, Tiptap GmbH | https://github.com/ueberdosis/tiptap |
| @tiptap/extensions | 3.32.0 | MIT | Copyright (c) 2025, Tiptap GmbH | https://github.com/ueberdosis/tiptap |
| @tiptap/pm (direct dep; re-export wrapper) | 3.32.0 | MIT | Copyright (c) 2025, Tiptap GmbH | https://github.com/ueberdosis/tiptap |
| @tiptap/react | 3.32.0 | MIT | Copyright (c) 2025, Tiptap GmbH | https://github.com/ueberdosis/tiptap |
| @tiptap/starter-kit | 3.32.0 | MIT | Copyright (c) 2025, Tiptap GmbH | https://github.com/ueberdosis/tiptap |
| aria-hidden | 1.2.6 | MIT | Copyright (c) 2017 Anton Korzunov | https://github.com/theKashey/aria-hidden |
| clsx | 2.1.1 | MIT | Copyright (c) Luke Edwards <luke.edwards05@gmail.com> (lukeed.com) | https://github.com/lukeed/clsx |
| cmdk | 1.1.1 | MIT | Copyright (c) 2022 Paco Coursey | https://github.com/pacocoursey/cmdk |
| devlop | 1.1.0 | MIT | Copyright (c) 2023 Titus Wormer <tituswormer@gmail.com> | https://github.com/wooorm/devlop |
| fast-equals | 5.4.5 | MIT | Copyright (c) 2025 Tony Quetano | https://github.com/planttheidea/fast-equals |
| framer-motion | 14.6.0 | MIT | Copyright (c) 2018 Framer B.V. | https://github.com/motiondivision/motion/ |
| get-nonce | 1.0.1 | MIT | Copyright (c) 2020 Anton Korzunov | git@github.com:theKashey/get-nonce |
| highlight.js | 11.12.0 | BSD-3-Clause | Copyright (c) 2006, Ivan Sagalaev. | https://github.com/highlightjs/highlight.js |
| highlight.js (nested under lowlight) | 11.11.2 | BSD-3-Clause | Copyright (c) 2006, Ivan Sagalaev. | https://github.com/highlightjs/highlight.js |
| katex | 0.19.0 | MIT (code); OFL-1.1 (bundled KaTeX_* fonts) | Copyright (c) 2013-2020 Khan Academy and other contributors; fonts: Copyright (c) 2009-2010 Design Science, Inc.; Copyright (c) 2014-2018 Khan Academy | https://github.com/KaTeX/KaTeX |
| linkifyjs | 4.3.3 | MIT | Copyright (c) 2024 Nick Frasser | https://github.com/nfrasser/linkifyjs |
| lowlight | 3.3.0 | MIT | Copyright (c) Titus Wormer <tituswormer@gmail.com> | https://github.com/wooorm/lowlight |
| lucide-react | 1.61.0 | ISC (+ MIT for Feather-derived icons) | Copyright (c) 2026 Lucide Icons and Contributors; Copyright (c) 2013-present Cole Bemis (Feather-derived icons) | https://github.com/lucide-icons/lucide |
| motion | 14.6.0 | MIT | Copyright (c) 2024 [Motion](https://motion.dev) B.V. | https://github.com/motiondivision/motion |
| motion-dom | 14.6.0 | MIT | Copyright (c) 2024 [Motion](https://motion.dev) B.V. | https://github.com/motiondivision/motion |
| motion-utils | 14.6.0 | MIT | Copyright (c) 2024 [Motion](https://motion.dev) B.V. | https://github.com/motiondivision/motion |
| orderedmap | 2.1.1 | MIT | Copyright (C) 2016 by Marijn Haverbeke <marijn@haverbeke.berlin> and others | https://github.com/marijnh/orderedmap |
| prosemirror-commands | 1.7.2 | MIT | Copyright (C) 2015-2017 by Marijn Haverbeke <marijn@haverbeke.berlin> and others | https://code.haverbeke.berlin/prosemirror/prosemirror-commands |
| prosemirror-dropcursor | 1.8.4 | MIT | Copyright (C) 2015-2017 by Marijn Haverbeke <marijn@haverbeke.berlin> and others | https://code.haverbeke.berlin/prosemirror/prosemirror-dropcursor |
| prosemirror-gapcursor | 1.4.1 | MIT | Copyright (C) 2015-2017 by Marijn Haverbeke <marijn@haverbeke.berlin> and others | https://github.com/prosemirror/prosemirror-gapcursor |
| prosemirror-history | 1.5.1 | MIT | Copyright (C) 2015-2017 by Marijn Haverbeke <marijn@haverbeke.berlin> and others | https://code.haverbeke.berlin/prosemirror/prosemirror-history |
| prosemirror-keymap | 1.2.3 | MIT | Copyright (C) 2015-2017 by Marijn Haverbeke <marijn@haverbeke.berlin> and others | https://github.com/prosemirror/prosemirror-keymap |
| prosemirror-model | 1.25.12 | MIT | Copyright (C) 2015-2017 by Marijn Haverbeke <marijn@haverbeke.berlin> and others | https://code.haverbeke.berlin/prosemirror/prosemirror-model |
| prosemirror-schema-list | 1.5.1 | MIT | Copyright (C) 2015-2017 by Marijn Haverbeke <marijn@haverbeke.berlin> and others | https://github.com/prosemirror/prosemirror-schema-list |
| prosemirror-state | 1.4.4 | MIT | Copyright (C) 2015-2017 by Marijn Haverbeke <marijn@haverbeke.berlin> and others | https://github.com/prosemirror/prosemirror-state |
| prosemirror-tables | 1.8.5 | MIT | Copyright (C) 2015-2016 by Marijn Haverbeke <marijnh@gmail.com> and others | https://github.com/ProseMirror/prosemirror-tables |
| prosemirror-transform | 1.12.3 | MIT | Copyright (C) 2015-2017 by Marijn Haverbeke <marijn@haverbeke.berlin> and others | https://code.haverbeke.berlin/prosemirror/prosemirror-transform |
| prosemirror-view | 1.42.7 | MIT | Copyright (C) 2015-2017 by Marijn Haverbeke <marijn@haverbeke.berlin> and others | https://code.haverbeke.berlin/prosemirror/prosemirror-view |
| radix-ui (direct dep; re-export wrapper) | 1.8.1 | MIT | Copyright (c) 2022 WorkOS | https://github.com/radix-ui/primitives |
| react | 19.3.1 | MIT | Copyright (c) Meta Platforms, Inc. and affiliates. | https://github.com/react/react |
| react-dom | 19.3.1 | MIT | Copyright (c) Meta Platforms, Inc. and affiliates. | https://github.com/react/react |
| react-remove-scroll | 2.7.2 | MIT | Copyright (c) 2017 Anton Korzunov | https://github.com/theKashey/react-remove-scroll |
| react-remove-scroll-bar | 2.3.8 | MIT | (no license file in package; package.json author: Anton Korzunov) | https://github.com/theKashey/react-remove-scroll-bar |
| react-style-singleton | 2.2.3 | MIT | Copyright (c) 2017 Anton Korzunov | https://github.com/theKashey/react-style-singleton |
| rope-sequence | 1.3.4 | MIT | Copyright (C) 2016 by Marijn Haverbeke <marijn@haverbeke.berlin> | https://github.com/marijnh/rope-sequence |
| scheduler | 0.28.0 | MIT | Copyright (c) Meta Platforms, Inc. and affiliates. | https://github.com/react/react |
| sonner | 2.0.8 | MIT | Copyright (c) 2023 Emil Kowalski | https://github.com/emilkowalski/sonner |
| tailwind-merge | 3.7.0 | MIT | Copyright (c) 2021 Dany Castillo | https://github.com/dcastil/tailwind-merge |
| tslib | 2.8.1 | 0BSD | Copyright (c) Microsoft Corporation. | https://github.com/Microsoft/tslib |
| use-callback-ref | 1.3.3 | MIT | Copyright (c) 2017 Anton Korzunov | https://github.com/theKashey/use-callback-ref/ |
| use-sidecar | 1.1.3 | MIT | Copyright (c) 2017 Anton Korzunov | https://github.com/theKashey/use-sidecar |
| use-sync-external-store | 1.7.0 | MIT | Copyright (c) Meta Platforms, Inc. and affiliates. | https://github.com/react/react |
| w3c-keyname | 2.2.8 | MIT | Copyright (C) 2016 by Marijn Haverbeke <marijn@haverbeke.berlin> and others | https://github.com/marijnh/w3c-keyname |
| zustand | 5.0.16 | MIT | Copyright (c) 2019 Paul Henschel | https://github.com/pmndrs/zustand |
| tailwindcss | 4.3.3 | MIT | Copyright (c) Tailwind Labs, Inc. | https://github.com/tailwindlabs/tailwindcss |

Notes: `@tauri-apps/api` is offered under `MIT OR Apache-2.0` and is listed as MIT. `tslib` is 0BSD (no attribution required; listed anyway). `lucide-react` is ISC plus MIT for icons derived from Feather (both notices in the License texts section). `katex` is MIT for the code; its fonts are SIL OFL 1.1 (see below). Two versions of highlight.js are bundled (11.12.0 and 11.11.2, under lowlight); both license files are identical. Another 46 packages appear in the production dependency closure but are not inside the bundle (unused parts of the `radix-ui` umbrella, type-only packages and similar); they are MIT and are not shipped.

### Build tools (not shipped)

These development tools are not in the exe and are listed for credit: @tailwindcss/vite 4.3.3 (MIT); @tauri-apps/cli 2.12.4 (Apache-2.0 OR MIT); @types/node 26.6.5 (MIT); @types/react 19.3.0 (MIT); @types/react-dom 19.3.0 (MIT); @vitejs/plugin-react 6.1.2 (MIT); tailwindcss 4.3.3 (MIT); typescript 6.0.3 (Apache-2.0); vite 8.3.5 (MIT). Their license fields are as in `package.json`. `vite` may inject a few lines of its own MIT-licensed helper code into the bundle; that was not itemised.

## License texts

Each distinct license text appears once, copied verbatim from a real file in the package named. Many packages share a license with different copyright lines; those lines are in the tables above and apply in place of the line in the text shown. Texts for components whose license could not be fetched are not included (see open decisions).

### MIT License (standard text)

Used by most Rust crates and npm packages listed above. The copyright lines are in the tables above, one per package. The text shown is copied from `@dnd-kit/accessibility` 3.1.1 (`LICENSE`); other packages use the same grant with a different copyright line and slightly different title wording ("The MIT License (MIT)", "(The MIT License)", or no title line).

~~~~text
MIT License

Copyright (c) 2021, Claudéric Demers

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
~~~~

### MIT License, Microsoft windows-rs family

Used by the `windows`, `windows-*` and `windows_x86_64_msvc` crates (`MIT OR Apache-2.0`). Source: `windows` 0.62.2, `license-mit`.

~~~~text
    MIT License

    Copyright (c) Microsoft Corporation.

    Permission is hereby granted, free of charge, to any person obtaining a copy
    of this software and associated documentation files (the "Software"), to deal
    in the Software without restriction, including without limitation the rights
    to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
    copies of the Software, and to permit persons to whom the Software is
    furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in all
    copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
    OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
    SOFTWARE
~~~~

### Microsoft WebView2 loader library (Microsoft.Web.WebView2 SDK)

Used by the statically linked `WebView2LoaderStatic.lib` inside `webview2-com-sys` 0.39.1 (version string `1.0.3800.47` in the library). This is the exact text of `LICENSE.txt` in Microsoft's NuGet package `Microsoft.Web.WebView2` 1.0.3800.47 (downloaded from https://api.nuget.org on 2026-10-23). It is a three-clause BSD-style text. The same file is `Licenses\WebView2-LICENSE.txt` in the portable ZIP.

~~~~text
Copyright (C) Microsoft Corporation. All rights reserved.

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are
met:

   * Redistributions of source code must retain the above copyright
notice, this list of conditions and the following disclaimer.
   * Redistributions in binary form must reproduce the above
copyright notice, this list of conditions and the following disclaimer
in the documentation and/or other materials provided with the
distribution.
   * The name of Microsoft Corporation, or the names of its contributors 
may not be used to endorse or promote products derived from this
software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
"AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
~~~~

### MIT License, webview2-rs

Used by `webview2-com`, `webview2-com-sys` and `webview2-com-macros`. These packages ship no license file; this text was fetched from the upstream repository https://github.com/wravery/webview2-rs (file `LICENSE`) on 2026-10-23.

~~~~text
MIT License

Copyright (c) 2021 Bill Avery

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
~~~~

### MIT License, zxcvbn crate

Used by the `zxcvbn` crate 3.1.1. Source: `zxcvbn-3.1.1/LICENSE`.

~~~~text
The MIT License (MIT)
Copyright (c) 2016 Joshua Holmer

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the "Software"), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
of the Software, and to permit persons to whom the Software is furnished to do
so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
~~~~

### MIT License, embedded spin code in tracing-core

Used by `tracing-core` 0.1.36 for the embedded spin lock code. Source: `src/spin/LICENSE`.

~~~~text
The MIT License (MIT)

Copyright (c) 2014 Mathijs van de Nes

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
~~~~

### MIT License with libm third-party notices

Used by `dpi` 0.1.2 (`Apache-2.0 AND MIT`); the notices of the libm sources it includes must be reproduced. Source: `LICENSE-LIBM-MIT`.

~~~~text
rust-lang/libm as a whole is available for use under the MIT license:

------------------------------------------------------------------------------
Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
------------------------------------------------------------------------------

This Rust library contains the following copyrights:

    Copyright (c) 2018 Jorge Aparicio

Portions of this software are derived from third-party works licensed under
terms compatible with the above MIT license:

* musl libc https://www.musl-libc.org/. This library contains the following
  copyright:

      Copyright © 2005-2020 Rich Felker, et al.

* The CORE-MATH project https://core-math.gitlabpages.inria.fr/. CORE-MATH
  routines are available under the MIT license on a per-file basis.

The musl libc COPYRIGHT file also includes the following notice relevant to
math portions of the library:

------------------------------------------------------------------------------
Much of the math library code (src/math/* and src/complex/*) is
Copyright © 1993,2004 Sun Microsystems or
Copyright © 2003-2011 David Schultz or
Copyright © 2003-2009 Steven G. Kargl or
Copyright © 2003-2009 Bruce D. Evans or
Copyright © 2008 Stephen L. Moshier or
Copyright © 2017-2018 Arm Limited
and labelled as such in comments in the individual source files. All
have been licensed under extremely permissive terms.
------------------------------------------------------------------------------
~~~~

### Apache License 2.0

Used by every Rust crate and npm package that is offered under `Apache-2.0` (alone or as an alternative to MIT, for example Tauri, serde, `tao` (Apache-2.0 only) and the RustCrypto crates). Where MIT is an alternative, this file lists MIT as the license taken. The text is copied from `serde` 1.0.229 (`LICENSE-APACHE`).

~~~~text
                              Apache License
                        Version 2.0, January 2004
                     http://www.apache.org/licenses/

TERMS AND CONDITIONS FOR USE, REPRODUCTION, AND DISTRIBUTION

1. Definitions.

   "License" shall mean the terms and conditions for use, reproduction,
   and distribution as defined by Sections 1 through 9 of this document.

   "Licensor" shall mean the copyright owner or entity authorized by
   the copyright owner that is granting the License.

   "Legal Entity" shall mean the union of the acting entity and all
   other entities that control, are controlled by, or are under common
   control with that entity. For the purposes of this definition,
   "control" means (i) the power, direct or indirect, to cause the
   direction or management of such entity, whether by contract or
   otherwise, or (ii) ownership of fifty percent (50%) or more of the
   outstanding shares, or (iii) beneficial ownership of such entity.

   "You" (or "Your") shall mean an individual or Legal Entity
   exercising permissions granted by this License.

   "Source" form shall mean the preferred form for making modifications,
   including but not limited to software source code, documentation
   source, and configuration files.

   "Object" form shall mean any form resulting from mechanical
   transformation or translation of a Source form, including but
   not limited to compiled object code, generated documentation,
   and conversions to other media types.

   "Work" shall mean the work of authorship, whether in Source or
   Object form, made available under the License, as indicated by a
   copyright notice that is included in or attached to the work
   (an example is provided in the Appendix below).

   "Derivative Works" shall mean any work, whether in Source or Object
   form, that is based on (or derived from) the Work and for which the
   editorial revisions, annotations, elaborations, or other modifications
   represent, as a whole, an original work of authorship. For the purposes
   of this License, Derivative Works shall not include works that remain
   separable from, or merely link (or bind by name) to the interfaces of,
   the Work and Derivative Works thereof.

   "Contribution" shall mean any work of authorship, including
   the original version of the Work and any modifications or additions
   to that Work or Derivative Works thereof, that is intentionally
   submitted to Licensor for inclusion in the Work by the copyright owner
   or by an individual or Legal Entity authorized to submit on behalf of
   the copyright owner. For the purposes of this definition, "submitted"
   means any form of electronic, verbal, or written communication sent
   to the Licensor or its representatives, including but not limited to
   communication on electronic mailing lists, source code control systems,
   and issue tracking systems that are managed by, or on behalf of, the
   Licensor for the purpose of discussing and improving the Work, but
   excluding communication that is conspicuously marked or otherwise
   designated in writing by the copyright owner as "Not a Contribution."

   "Contributor" shall mean Licensor and any individual or Legal Entity
   on behalf of whom a Contribution has been received by Licensor and
   subsequently incorporated within the Work.

2. Grant of Copyright License. Subject to the terms and conditions of
   this License, each Contributor hereby grants to You a perpetual,
   worldwide, non-exclusive, no-charge, royalty-free, irrevocable
   copyright license to reproduce, prepare Derivative Works of,
   publicly display, publicly perform, sublicense, and distribute the
   Work and such Derivative Works in Source or Object form.

3. Grant of Patent License. Subject to the terms and conditions of
   this License, each Contributor hereby grants to You a perpetual,
   worldwide, non-exclusive, no-charge, royalty-free, irrevocable
   (except as stated in this section) patent license to make, have made,
   use, offer to sell, sell, import, and otherwise transfer the Work,
   where such license applies only to those patent claims licensable
   by such Contributor that are necessarily infringed by their
   Contribution(s) alone or by combination of their Contribution(s)
   with the Work to which such Contribution(s) was submitted. If You
   institute patent litigation against any entity (including a
   cross-claim or counterclaim in a lawsuit) alleging that the Work
   or a Contribution incorporated within the Work constitutes direct
   or contributory patent infringement, then any patent licenses
   granted to You under this License for that Work shall terminate
   as of the date such litigation is filed.

4. Redistribution. You may reproduce and distribute copies of the
   Work or Derivative Works thereof in any medium, with or without
   modifications, and in Source or Object form, provided that You
   meet the following conditions:

   (a) You must give any other recipients of the Work or
       Derivative Works a copy of this License; and

   (b) You must cause any modified files to carry prominent notices
       stating that You changed the files; and

   (c) You must retain, in the Source form of any Derivative Works
       that You distribute, all copyright, patent, trademark, and
       attribution notices from the Source form of the Work,
       excluding those notices that do not pertain to any part of
       the Derivative Works; and

   (d) If the Work includes a "NOTICE" text file as part of its
       distribution, then any Derivative Works that You distribute must
       include a readable copy of the attribution notices contained
       within such NOTICE file, excluding those notices that do not
       pertain to any part of the Derivative Works, in at least one
       of the following places: within a NOTICE text file distributed
       as part of the Derivative Works; within the Source form or
       documentation, if provided along with the Derivative Works; or,
       within a display generated by the Derivative Works, if and
       wherever such third-party notices normally appear. The contents
       of the NOTICE file are for informational purposes only and
       do not modify the License. You may add Your own attribution
       notices within Derivative Works that You distribute, alongside
       or as an addendum to the NOTICE text from the Work, provided
       that such additional attribution notices cannot be construed
       as modifying the License.

   You may add Your own copyright statement to Your modifications and
   may provide additional or different license terms and conditions
   for use, reproduction, or distribution of Your modifications, or
   for any such Derivative Works as a whole, provided Your use,
   reproduction, and distribution of the Work otherwise complies with
   the conditions stated in this License.

5. Submission of Contributions. Unless You explicitly state otherwise,
   any Contribution intentionally submitted for inclusion in the Work
   by You to the Licensor shall be under the terms and conditions of
   this License, without any additional terms or conditions.
   Notwithstanding the above, nothing herein shall supersede or modify
   the terms of any separate license agreement you may have executed
   with Licensor regarding such Contributions.

6. Trademarks. This License does not grant permission to use the trade
   names, trademarks, service marks, or product names of the Licensor,
   except as required for reasonable and customary use in describing the
   origin of the Work and reproducing the content of the NOTICE file.

7. Disclaimer of Warranty. Unless required by applicable law or
   agreed to in writing, Licensor provides the Work (and each
   Contributor provides its Contributions) on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
   implied, including, without limitation, any warranties or conditions
   of TITLE, NON-INFRINGEMENT, MERCHANTABILITY, or FITNESS FOR A
   PARTICULAR PURPOSE. You are solely responsible for determining the
   appropriateness of using or redistributing the Work and assume any
   risks associated with Your exercise of permissions under this License.

8. Limitation of Liability. In no event and under no legal theory,
   whether in tort (including negligence), contract, or otherwise,
   unless required by applicable law (such as deliberate and grossly
   negligent acts) or agreed to in writing, shall any Contributor be
   liable to You for damages, including any direct, indirect, special,
   incidental, or consequential damages of any character arising as a
   result of this License or out of the use or inability to use the
   Work (including but not limited to damages for loss of goodwill,
   work stoppage, computer failure or malfunction, or any and all
   other commercial damages or losses), even if such Contributor
   has been advised of the possibility of such damages.

9. Accepting Warranty or Additional Liability. While redistributing
   the Work or Derivative Works thereof, You may choose to offer,
   and charge a fee for, acceptance of support, warranty, indemnity,
   or other liability obligations and/or rights consistent with this
   License. However, in accepting such obligations, You may act only
   on Your own behalf and on Your sole responsibility, not on behalf
   of any other Contributor, and only if You agree to indemnify,
   defend, and hold each Contributor harmless for any liability
   incurred by, or claims asserted against, such Contributor by reason
   of your accepting any such warranty or additional liability.

END OF TERMS AND CONDITIONS
~~~~

### BSD 3-Clause License, Dropbox

Used by `alloc-no-stdlib` 3.0.0, `alloc-stdlib` 0.3.0 (the package has no license file; the upstream text applies), `brotli` 9.0.0 (BSD-3-Clause AND MIT) and `brotli-decompressor` 6.0.1 (`BSD-3-Clause/MIT`, only the BSD-3 text is in the package). Source: `alloc-no-stdlib-3.0.0/LICENSE`.

~~~~text
Copyright (c) 2016 Dropbox, Inc.
All rights reserved.

Redistribution and use in source and binary forms, with or without modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the following disclaimer in the documentation and/or other materials provided with the distribution.

3. Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote products derived from this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
~~~~

### BSD 3-Clause License, highlight.js

Used by highlight.js 11.12.0 and 11.11.2 (identical files). Source: `highlight.js/LICENSE`.

~~~~text
BSD 3-Clause License

Copyright (c) 2006, Ivan Sagalaev.
All rights reserved.

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:

* Redistributions of source code must retain the above copyright notice, this
  list of conditions and the following disclaimer.

* Redistributions in binary form must reproduce the above copyright notice,
  this list of conditions and the following disclaimer in the documentation
  and/or other materials provided with the distribution.

* Neither the name of the copyright holder nor the names of its
  contributors may be used to endorse or promote products derived from
  this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
~~~~

### ISC License and Feather MIT License, lucide-react

Used by `lucide-react` 1.61.0 (ISC), and for the Feather-derived icons listed in it (MIT, Copyright (c) 2013-present Cole Bemis). Source: `lucide-react/LICENSE`.

~~~~text
ISC License

Copyright (c) 2026 Lucide Icons and Contributors

Permission to use, copy, modify, and/or distribute this software for any
purpose with or without fee is hereby granted, provided that the above
copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

---

The following Lucide icons are derived from the Feather project:

airplay, alert-circle, alert-octagon, alert-triangle, aperture, arrow-down-circle, arrow-down-left, arrow-down-right, arrow-down, arrow-left-circle, arrow-left, arrow-right-circle, arrow-right, arrow-up-circle, arrow-up-left, arrow-up-right, arrow-up, at-sign, calendar, cast, check, chevron-down, chevron-left, chevron-right, chevron-up, chevrons-down, chevrons-left, chevrons-right, chevrons-up, circle, clipboard, clock, code, columns, command, compass, corner-down-left, corner-down-right, corner-left-down, corner-left-up, corner-right-down, corner-right-up, corner-up-left, corner-up-right, crosshair, database, divide-circle, divide-square, dollar-sign, download, external-link, feather, frown, hash, headphones, help-circle, info, italic, key, layout, life-buoy, link-2, link, loader, lock, log-in, log-out, maximize, meh, minimize, minimize-2, minus-circle, minus-square, minus, monitor, moon, more-horizontal, more-vertical, move, music, navigation-2, navigation, octagon, pause-circle, percent, plus-circle, plus-square, plus, power, radio, rss, search, server, share, shopping-bag, sidebar, smartphone, smile, square, table-2, tablet, target, terminal, trash-2, trash, triangle, tv, type, upload, x-circle, x-octagon, x-square, x, zoom-in, zoom-out

The MIT License (MIT) (for the icons listed above)

Copyright (c) 2013-present Cole Bemis

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
~~~~

### 0BSD (Zero-clause BSD), tslib

Used by `tslib` 2.8.1. No attribution is required; included for completeness. Source: `tslib/LICENSE.txt`.

~~~~text
Copyright (c) Microsoft Corporation.

Permission to use, copy, modify, and/or distribute this software for any
purpose with or without fee is hereby granted.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY
AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR
OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
PERFORMANCE OF THIS SOFTWARE.
~~~~

### Zlib License, foldhash

Used by `foldhash` 0.2.0 and available as an option in `tinyvec`, `raw-window-handle`, `miniz_oxide` and `static_vcruntime`. Other Zlib components carry their own copyright lines (`zlib-rs`: (C) 2024 Trifecta Tech Foundation). Source: `foldhash-0.2.0/LICENSE`.

~~~~text
Copyright (c) 2024 Orson Peters

This software is provided 'as-is', without any express or implied warranty. In
no event will the authors be held liable for any damages arising from the use of
this software.

Permission is granted to anyone to use this software for any purpose, including
commercial applications, and to alter it and redistribute it freely, subject to
the following restrictions:

1. The origin of this software must not be misrepresented; you must not claim
    that you wrote the original software. If you use this software in a product,
    an acknowledgment in the product documentation would be appreciated but is
    not required.

2. Altered source versions must be plainly marked as such, and must not be
    misrepresented as being the original software.

3. This notice may not be removed or altered from any source distribution.
~~~~

### The Unlicense

Used by `memchr` and, as an alternative to MIT, by `aho-corasick`, `byteorder`, `jiff` and related crates (`Unlicense OR MIT`). Source: `memchr-2.8.3/UNLICENSE`.

~~~~text
This is free and unencumbered software released into the public domain.

Anyone is free to copy, modify, publish, use, compile, sell, or
distribute this software, either in source code form or as a compiled
binary, for any purpose, commercial or non-commercial, and by any
means.

In jurisdictions that recognize copyright laws, the author or authors
of this software dedicate any and all copyright interest in the
software to the public domain. We make this dedication for the benefit
of the public at large and to the detriment of our heirs and
successors. We intend this dedication to be an overt act of
relinquishment in perpetuity of all present and future rights to this
software under copyright law.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR
OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
OTHER DEALINGS IN THE SOFTWARE.

For more information, please refer to <http://unlicense.org/>
~~~~

### CC0 1.0 Universal

Used by `dunce` 1.0.5 (`CC0-1.0 OR MIT-0 OR Apache-2.0`; only this text is in the package). Source: `dunce-1.0.5/LICENSE`.

~~~~text
Creative Commons Legal Code

CC0 1.0 Universal

    CREATIVE COMMONS CORPORATION IS NOT A LAW FIRM AND DOES NOT PROVIDE
    LEGAL SERVICES. DISTRIBUTION OF THIS DOCUMENT DOES NOT CREATE AN
    ATTORNEY-CLIENT RELATIONSHIP. CREATIVE COMMONS PROVIDES THIS
    INFORMATION ON AN "AS-IS" BASIS. CREATIVE COMMONS MAKES NO WARRANTIES
    REGARDING THE USE OF THIS DOCUMENT OR THE INFORMATION OR WORKS
    PROVIDED HEREUNDER, AND DISCLAIMS LIABILITY FOR DAMAGES RESULTING FROM
    THE USE OF THIS DOCUMENT OR THE INFORMATION OR WORKS PROVIDED
    HEREUNDER.

Statement of Purpose

The laws of most jurisdictions throughout the world automatically confer
exclusive Copyright and Related Rights (defined below) upon the creator
and subsequent owner(s) (each and all, an "owner") of an original work of
authorship and/or a database (each, a "Work").

Certain owners wish to permanently relinquish those rights to a Work for
the purpose of contributing to a commons of creative, cultural and
scientific works ("Commons") that the public can reliably and without fear
of later claims of infringement build upon, modify, incorporate in other
works, reuse and redistribute as freely as possible in any form whatsoever
and for any purposes, including without limitation commercial purposes.
These owners may contribute to the Commons to promote the ideal of a free
culture and the further production of creative, cultural and scientific
works, or to gain reputation or greater distribution for their Work in
part through the use and efforts of others.

For these and/or other purposes and motivations, and without any
expectation of additional consideration or compensation, the person
associating CC0 with a Work (the "Affirmer"), to the extent that he or she
is an owner of Copyright and Related Rights in the Work, voluntarily
elects to apply CC0 to the Work and publicly distribute the Work under its
terms, with knowledge of his or her Copyright and Related Rights in the
Work and the meaning and intended legal effect of CC0 on those rights.

1. Copyright and Related Rights. A Work made available under CC0 may be
protected by copyright and related or neighboring rights ("Copyright and
Related Rights"). Copyright and Related Rights include, but are not
limited to, the following:

  i. the right to reproduce, adapt, distribute, perform, display,
     communicate, and translate a Work;
 ii. moral rights retained by the original author(s) and/or performer(s);
iii. publicity and privacy rights pertaining to a person's image or
     likeness depicted in a Work;
 iv. rights protecting against unfair competition in regards to a Work,
     subject to the limitations in paragraph 4(a), below;
  v. rights protecting the extraction, dissemination, use and reuse of data
     in a Work;
 vi. database rights (such as those arising under Directive 96/9/EC of the
     European Parliament and of the Council of 11 March 1996 on the legal
     protection of databases, and under any national implementation
     thereof, including any amended or successor version of such
     directive); and
vii. other similar, equivalent or corresponding rights throughout the
     world based on applicable law or treaty, and any national
     implementations thereof.

2. Waiver. To the greatest extent permitted by, but not in contravention
of, applicable law, Affirmer hereby overtly, fully, permanently,
irrevocably and unconditionally waives, abandons, and surrenders all of
Affirmer's Copyright and Related Rights and associated claims and causes
of action, whether now known or unknown (including existing as well as
future claims and causes of action), in the Work (i) in all territories
worldwide, (ii) for the maximum duration provided by applicable law or
treaty (including future time extensions), (iii) in any current or future
medium and for any number of copies, and (iv) for any purpose whatsoever,
including without limitation commercial, advertising or promotional
purposes (the "Waiver"). Affirmer makes the Waiver for the benefit of each
member of the public at large and to the detriment of Affirmer's heirs and
successors, fully intending that such Waiver shall not be subject to
revocation, rescission, cancellation, termination, or any other legal or
equitable action to disrupt the quiet enjoyment of the Work by the public
as contemplated by Affirmer's express Statement of Purpose.

3. Public License Fallback. Should any part of the Waiver for any reason
be judged legally invalid or ineffective under applicable law, then the
Waiver shall be preserved to the maximum extent permitted taking into
account Affirmer's express Statement of Purpose. In addition, to the
extent the Waiver is so judged Affirmer hereby grants to each affected
person a royalty-free, non transferable, non sublicensable, non exclusive,
irrevocable and unconditional license to exercise Affirmer's Copyright and
Related Rights in the Work (i) in all territories worldwide, (ii) for the
maximum duration provided by applicable law or treaty (including future
time extensions), (iii) in any current or future medium and for any number
of copies, and (iv) for any purpose whatsoever, including without
limitation commercial, advertising or promotional purposes (the
"License"). The License shall be deemed effective as of the date CC0 was
applied by Affirmer to the Work. Should any part of the License for any
reason be judged legally invalid or ineffective under applicable law, such
partial invalidity or ineffectiveness shall not invalidate the remainder
of the License, and in such case Affirmer hereby affirms that he or she
will not (i) exercise any of his or her remaining Copyright and Related
Rights in the Work or (ii) assert any associated claims and causes of
action with respect to the Work, in either case contrary to Affirmer's
express Statement of Purpose.

4. Limitations and Disclaimers.

 a. No trademark or patent rights held by Affirmer are waived, abandoned,
    surrendered, licensed or otherwise affected by this document.
 b. Affirmer offers the Work as-is and makes no representations or
    warranties of any kind concerning the Work, express, implied,
    statutory or otherwise, including without limitation warranties of
    title, merchantability, fitness for a particular purpose, non
    infringement, or the absence of latent or other defects, accuracy, or
    the present or absence of errors, whether or not discoverable, all to
    the greatest extent permissible under applicable law.
 c. Affirmer disclaims responsibility for clearing rights of other persons
    that may apply to the Work or any use thereof, including without
    limitation any person's Copyright and Related Rights in the Work.
    Further, Affirmer disclaims responsibility for obtaining any necessary
    consents, permissions or other rights required for any use of the
    Work.
 d. Affirmer understands and acknowledges that Creative Commons is not a
    party to this document and has no duty or obligation with respect to
    this CC0 or use of the Work.
~~~~

### Unicode License v3, ICU4X crates

Used by the ICU4X family: icu_collections, icu_locale_core, icu_normalizer, icu_normalizer_data, icu_properties, icu_properties_data, icu_provider, litemap, potential_utf, tinystr, writeable, yoke, zerofrom, zerotrie, zerovec (and their derive crates at build time). The data crates embed Unicode data files. Source: `icu_provider-2.4.0/LICENSE`.

~~~~text
UNICODE LICENSE V3

COPYRIGHT AND PERMISSION NOTICE

Copyright © 2020-2024 Unicode, Inc.

NOTICE TO USER: Carefully read the following legal agreement. BY
DOWNLOADING, INSTALLING, COPYING OR OTHERWISE USING DATA FILES, AND/OR
SOFTWARE, YOU UNEQUIVOCALLY ACCEPT, AND AGREE TO BE BOUND BY, ALL OF THE
TERMS AND CONDITIONS OF THIS AGREEMENT. IF YOU DO NOT AGREE, DO NOT
DOWNLOAD, INSTALL, COPY, DISTRIBUTE OR USE THE DATA FILES OR SOFTWARE.

Permission is hereby granted, free of charge, to any person obtaining a
copy of data files and any associated documentation (the "Data Files") or
software and any associated documentation (the "Software") to deal in the
Data Files or Software without restriction, including without limitation
the rights to use, copy, modify, merge, publish, distribute, and/or sell
copies of the Data Files or Software, and to permit persons to whom the
Data Files or Software are furnished to do so, provided that either (a)
this copyright and permission notice appear with all copies of the Data
Files or Software, or (b) this copyright and permission notice appear in
associated Documentation.

THE DATA FILES AND SOFTWARE ARE PROVIDED "AS IS", WITHOUT WARRANTY OF ANY
KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT OF
THIRD PARTY RIGHTS.

IN NO EVENT SHALL THE COPYRIGHT HOLDER OR HOLDERS INCLUDED IN THIS NOTICE
BE LIABLE FOR ANY CLAIM, OR ANY SPECIAL INDIRECT OR CONSEQUENTIAL DAMAGES,
OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS,
WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION,
ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THE DATA
FILES OR SOFTWARE.

Except as contained in this notice, the name of a copyright holder shall
not be used in advertising or otherwise to promote the sale, use or other
dealings in these Data Files or Software without prior written
authorization of the copyright holder.

SPDX-License-Identifier: Unicode-3.0

—

Portions of ICU4X may have been adapted from ICU4C and/or ICU4J.
ICU 1.8.1 to ICU 57.1 © 1995-2016 International Business Machines Corporation and others.
~~~~

### Unicode License v3, unicode-ident

Used by `unicode-ident` 1.0.27 and `unicode-normalization` 0.1.26 (alongside MIT or Apache-2.0). Source: `unicode-ident-1.0.27/LICENSE-UNICODE`.

~~~~text
UNICODE LICENSE V3

COPYRIGHT AND PERMISSION NOTICE

Copyright © 1991-2023 Unicode, Inc.

NOTICE TO USER: Carefully read the following legal agreement. BY
DOWNLOADING, INSTALLING, COPYING OR OTHERWISE USING DATA FILES, AND/OR
SOFTWARE, YOU UNEQUIVOCALLY ACCEPT, AND AGREE TO BE BOUND BY, ALL OF THE
TERMS AND CONDITIONS OF THIS AGREEMENT. IF YOU DO NOT AGREE, DO NOT
DOWNLOAD, INSTALL, COPY, DISTRIBUTE OR USE THE DATA FILES OR SOFTWARE.

Permission is hereby granted, free of charge, to any person obtaining a
copy of data files and any associated documentation (the "Data Files") or
software and any associated documentation (the "Software") to deal in the
Data Files or Software without restriction, including without limitation
the rights to use, copy, modify, merge, publish, distribute, and/or sell
copies of the Data Files or Software, and to permit persons to whom the
Data Files or Software are furnished to do so, provided that either (a)
this copyright and permission notice appear with all copies of the Data
Files or Software, or (b) this copyright and permission notice appear in
associated Documentation.

THE DATA FILES AND SOFTWARE ARE PROVIDED "AS IS", WITHOUT WARRANTY OF ANY
KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT OF
THIRD PARTY RIGHTS.

IN NO EVENT SHALL THE COPYRIGHT HOLDER OR HOLDERS INCLUDED IN THIS NOTICE
BE LIABLE FOR ANY CLAIM, OR ANY SPECIAL INDIRECT OR CONSEQUENTIAL DAMAGES,
OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS,
WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION,
ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THE DATA
FILES OR SOFTWARE.

Except as contained in this notice, the name of a copyright holder shall
not be used in advertising or otherwise to promote the sale, use or other
dealings in these Data Files or Software without prior written
authorization of the copyright holder.
~~~~

### Unicode, Inc. License Agreement for Data Files and Software (Unicode-DFS-2016), regex-syntax

Applies to the Unicode data tables inside `regex-syntax` 0.8.11. Source: `regex-syntax-0.8.11/src/unicode_tables/LICENSE-UNICODE`.

~~~~text
UNICODE, INC. LICENSE AGREEMENT - DATA FILES AND SOFTWARE

Unicode Data Files include all data files under the directories
http://www.unicode.org/Public/, http://www.unicode.org/reports/,
http://www.unicode.org/cldr/data/, http://source.icu-project.org/repos/icu/, and
http://www.unicode.org/utility/trac/browser/.

Unicode Data Files do not include PDF online code charts under the
directory http://www.unicode.org/Public/.

Software includes any source code published in the Unicode Standard
or under the directories
http://www.unicode.org/Public/, http://www.unicode.org/reports/,
http://www.unicode.org/cldr/data/, http://source.icu-project.org/repos/icu/, and
http://www.unicode.org/utility/trac/browser/.

NOTICE TO USER: Carefully read the following legal agreement.
BY DOWNLOADING, INSTALLING, COPYING OR OTHERWISE USING UNICODE INC.'S
DATA FILES ("DATA FILES"), AND/OR SOFTWARE ("SOFTWARE"),
YOU UNEQUIVOCALLY ACCEPT, AND AGREE TO BE BOUND BY, ALL OF THE
TERMS AND CONDITIONS OF THIS AGREEMENT.
IF YOU DO NOT AGREE, DO NOT DOWNLOAD, INSTALL, COPY, DISTRIBUTE OR USE
THE DATA FILES OR SOFTWARE.

COPYRIGHT AND PERMISSION NOTICE

Copyright © 1991-2018 Unicode, Inc. All rights reserved.
Distributed under the Terms of Use in http://www.unicode.org/copyright.html.

Permission is hereby granted, free of charge, to any person obtaining
a copy of the Unicode data files and any associated documentation
(the "Data Files") or Unicode software and any associated documentation
(the "Software") to deal in the Data Files or Software
without restriction, including without limitation the rights to use,
copy, modify, merge, publish, distribute, and/or sell copies of
the Data Files or Software, and to permit persons to whom the Data Files
or Software are furnished to do so, provided that either
(a) this copyright and permission notice appear with all copies
of the Data Files or Software, or
(b) this copyright and permission notice appear in associated
Documentation.

THE DATA FILES AND SOFTWARE ARE PROVIDED "AS IS", WITHOUT WARRANTY OF
ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE
WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT OF THIRD PARTY RIGHTS.
IN NO EVENT SHALL THE COPYRIGHT HOLDER OR HOLDERS INCLUDED IN THIS
NOTICE BE LIABLE FOR ANY CLAIM, OR ANY SPECIAL INDIRECT OR CONSEQUENTIAL
DAMAGES, OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE,
DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER
TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
PERFORMANCE OF THE DATA FILES OR SOFTWARE.

Except as contained in this notice, the name of a copyright holder
shall not be used in advertising or otherwise to promote the sale,
use or other dealings in these Data Files or Software without prior
written authorization of the copyright holder.
~~~~

### Mozilla Public License 2.0

Used by `cssparser`, `dtoa-short`, `option-ext` and `selectors` (and the build-time `cssparser-macros`). Source: `cssparser-0.37.0/LICENSE`. The source of these crates is available at the URLs in the open decisions.

~~~~text
Mozilla Public License Version 2.0
==================================

1. Definitions
--------------

1.1. "Contributor"
    means each individual or legal entity that creates, contributes to
    the creation of, or owns Covered Software.

1.2. "Contributor Version"
    means the combination of the Contributions of others (if any) used
    by a Contributor and that particular Contributor's Contribution.

1.3. "Contribution"
    means Covered Software of a particular Contributor.

1.4. "Covered Software"
    means Source Code Form to which the initial Contributor has attached
    the notice in Exhibit A, the Executable Form of such Source Code
    Form, and Modifications of such Source Code Form, in each case
    including portions thereof.

1.5. "Incompatible With Secondary Licenses"
    means

    (a) that the initial Contributor has attached the notice described
        in Exhibit B to the Covered Software; or

    (b) that the Covered Software was made available under the terms of
        version 1.1 or earlier of the License, but not also under the
        terms of a Secondary License.

1.6. "Executable Form"
    means any form of the work other than Source Code Form.

1.7. "Larger Work"
    means a work that combines Covered Software with other material, in 
    a separate file or files, that is not Covered Software.

1.8. "License"
    means this document.

1.9. "Licensable"
    means having the right to grant, to the maximum extent possible,
    whether at the time of the initial grant or subsequently, any and
    all of the rights conveyed by this License.

1.10. "Modifications"
    means any of the following:

    (a) any file in Source Code Form that results from an addition to,
        deletion from, or modification of the contents of Covered
        Software; or

    (b) any new file in Source Code Form that contains any Covered
        Software.

1.11. "Patent Claims" of a Contributor
    means any patent claim(s), including without limitation, method,
    process, and apparatus claims, in any patent Licensable by such
    Contributor that would be infringed, but for the grant of the
    License, by the making, using, selling, offering for sale, having
    made, import, or transfer of either its Contributions or its
    Contributor Version.

1.12. "Secondary License"
    means either the GNU General Public License, Version 2.0, the GNU
    Lesser General Public License, Version 2.1, the GNU Affero General
    Public License, Version 3.0, or any later versions of those
    licenses.

1.13. "Source Code Form"
    means the form of the work preferred for making modifications.

1.14. "You" (or "Your")
    means an individual or a legal entity exercising rights under this
    License. For legal entities, "You" includes any entity that
    controls, is controlled by, or is under common control with You. For
    purposes of this definition, "control" means (a) the power, direct
    or indirect, to cause the direction or management of such entity,
    whether by contract or otherwise, or (b) ownership of more than
    fifty percent (50%) of the outstanding shares or beneficial
    ownership of such entity.

2. License Grants and Conditions
--------------------------------

2.1. Grants

Each Contributor hereby grants You a world-wide, royalty-free,
non-exclusive license:

(a) under intellectual property rights (other than patent or trademark)
    Licensable by such Contributor to use, reproduce, make available,
    modify, display, perform, distribute, and otherwise exploit its
    Contributions, either on an unmodified basis, with Modifications, or
    as part of a Larger Work; and

(b) under Patent Claims of such Contributor to make, use, sell, offer
    for sale, have made, import, and otherwise transfer either its
    Contributions or its Contributor Version.

2.2. Effective Date

The licenses granted in Section 2.1 with respect to any Contribution
become effective for each Contribution on the date the Contributor first
distributes such Contribution.

2.3. Limitations on Grant Scope

The licenses granted in this Section 2 are the only rights granted under
this License. No additional rights or licenses will be implied from the
distribution or licensing of Covered Software under this License.
Notwithstanding Section 2.1(b) above, no patent license is granted by a
Contributor:

(a) for any code that a Contributor has removed from Covered Software;
    or

(b) for infringements caused by: (i) Your and any other third party's
    modifications of Covered Software, or (ii) the combination of its
    Contributions with other software (except as part of its Contributor
    Version); or

(c) under Patent Claims infringed by Covered Software in the absence of
    its Contributions.

This License does not grant any rights in the trademarks, service marks,
or logos of any Contributor (except as may be necessary to comply with
the notice requirements in Section 3.4).

2.4. Subsequent Licenses

No Contributor makes additional grants as a result of Your choice to
distribute the Covered Software under a subsequent version of this
License (see Section 10.2) or under the terms of a Secondary License (if
permitted under the terms of Section 3.3).

2.5. Representation

Each Contributor represents that the Contributor believes its
Contributions are its original creation(s) or it has sufficient rights
to grant the rights to its Contributions conveyed by this License.

2.6. Fair Use

This License is not intended to limit any rights You have under
applicable copyright doctrines of fair use, fair dealing, or other
equivalents.

2.7. Conditions

Sections 3.1, 3.2, 3.3, and 3.4 are conditions of the licenses granted
in Section 2.1.

3. Responsibilities
-------------------

3.1. Distribution of Source Form

All distribution of Covered Software in Source Code Form, including any
Modifications that You create or to which You contribute, must be under
the terms of this License. You must inform recipients that the Source
Code Form of the Covered Software is governed by the terms of this
License, and how they can obtain a copy of this License. You may not
attempt to alter or restrict the recipients' rights in the Source Code
Form.

3.2. Distribution of Executable Form

If You distribute Covered Software in Executable Form then:

(a) such Covered Software must also be made available in Source Code
    Form, as described in Section 3.1, and You must inform recipients of
    the Executable Form how they can obtain a copy of such Source Code
    Form by reasonable means in a timely manner, at a charge no more
    than the cost of distribution to the recipient; and

(b) You may distribute such Executable Form under the terms of this
    License, or sublicense it under different terms, provided that the
    license for the Executable Form does not attempt to limit or alter
    the recipients' rights in the Source Code Form under this License.

3.3. Distribution of a Larger Work

You may create and distribute a Larger Work under terms of Your choice,
provided that You also comply with the requirements of this License for
the Covered Software. If the Larger Work is a combination of Covered
Software with a work governed by one or more Secondary Licenses, and the
Covered Software is not Incompatible With Secondary Licenses, this
License permits You to additionally distribute such Covered Software
under the terms of such Secondary License(s), so that the recipient of
the Larger Work may, at their option, further distribute the Covered
Software under the terms of either this License or such Secondary
License(s).

3.4. Notices

You may not remove or alter the substance of any license notices
(including copyright notices, patent notices, disclaimers of warranty,
or limitations of liability) contained within the Source Code Form of
the Covered Software, except that You may alter any license notices to
the extent required to remedy known factual inaccuracies.

3.5. Application of Additional Terms

You may choose to offer, and to charge a fee for, warranty, support,
indemnity or liability obligations to one or more recipients of Covered
Software. However, You may do so only on Your own behalf, and not on
behalf of any Contributor. You must make it absolutely clear that any
such warranty, support, indemnity, or liability obligation is offered by
You alone, and You hereby agree to indemnify every Contributor for any
liability incurred by such Contributor as a result of warranty, support,
indemnity or liability terms You offer. You may include additional
disclaimers of warranty and limitations of liability specific to any
jurisdiction.

4. Inability to Comply Due to Statute or Regulation
---------------------------------------------------

If it is impossible for You to comply with any of the terms of this
License with respect to some or all of the Covered Software due to
statute, judicial order, or regulation then You must: (a) comply with
the terms of this License to the maximum extent possible; and (b)
describe the limitations and the code they affect. Such description must
be placed in a text file included with all distributions of the Covered
Software under this License. Except to the extent prohibited by statute
or regulation, such description must be sufficiently detailed for a
recipient of ordinary skill to be able to understand it.

5. Termination
--------------

5.1. The rights granted under this License will terminate automatically
if You fail to comply with any of its terms. However, if You become
compliant, then the rights granted under this License from a particular
Contributor are reinstated (a) provisionally, unless and until such
Contributor explicitly and finally terminates Your grants, and (b) on an
ongoing basis, if such Contributor fails to notify You of the
non-compliance by some reasonable means prior to 60 days after You have
come back into compliance. Moreover, Your grants from a particular
Contributor are reinstated on an ongoing basis if such Contributor
notifies You of the non-compliance by some reasonable means, this is the
first time You have received notice of non-compliance with this License
from such Contributor, and You become compliant prior to 30 days after
Your receipt of the notice.

5.2. If You initiate litigation against any entity by asserting a patent
infringement claim (excluding declaratory judgment actions,
counter-claims, and cross-claims) alleging that a Contributor Version
directly or indirectly infringes any patent, then the rights granted to
You by any and all Contributors for the Covered Software under Section
2.1 of this License shall terminate.

5.3. In the event of termination under Sections 5.1 or 5.2 above, all
end user license agreements (excluding distributors and resellers) which
have been validly granted by You or Your distributors under this License
prior to termination shall survive termination.

************************************************************************
*                                                                      *
*  6. Disclaimer of Warranty                                           *
*  -------------------------                                           *
*                                                                      *
*  Covered Software is provided under this License on an "as is"       *
*  basis, without warranty of any kind, either expressed, implied, or  *
*  statutory, including, without limitation, warranties that the       *
*  Covered Software is free of defects, merchantable, fit for a        *
*  particular purpose or non-infringing. The entire risk as to the     *
*  quality and performance of the Covered Software is with You.        *
*  Should any Covered Software prove defective in any respect, You     *
*  (not any Contributor) assume the cost of any necessary servicing,   *
*  repair, or correction. This disclaimer of warranty constitutes an   *
*  essential part of this License. No use of any Covered Software is   *
*  authorized under this License except under this disclaimer.         *
*                                                                      *
************************************************************************

************************************************************************
*                                                                      *
*  7. Limitation of Liability                                          *
*  --------------------------                                          *
*                                                                      *
*  Under no circumstances and under no legal theory, whether tort      *
*  (including negligence), contract, or otherwise, shall any           *
*  Contributor, or anyone who distributes Covered Software as          *
*  permitted above, be liable to You for any direct, indirect,         *
*  special, incidental, or consequential damages of any character      *
*  including, without limitation, damages for lost profits, loss of    *
*  goodwill, work stoppage, computer failure or malfunction, or any    *
*  and all other commercial damages or losses, even if such party      *
*  shall have been informed of the possibility of such damages. This   *
*  limitation of liability shall not apply to liability for death or   *
*  personal injury resulting from such party's negligence to the       *
*  extent applicable law prohibits such limitation. Some               *
*  jurisdictions do not allow the exclusion or limitation of           *
*  incidental or consequential damages, so this exclusion and          *
*  limitation may not apply to You.                                    *
*                                                                      *
************************************************************************

8. Litigation
-------------

Any litigation relating to this License may be brought only in the
courts of a jurisdiction where the defendant maintains its principal
place of business and such litigation shall be governed by laws of that
jurisdiction, without reference to its conflict-of-law provisions.
Nothing in this Section shall prevent a party's ability to bring
cross-claims or counter-claims.

9. Miscellaneous
----------------

This License represents the complete agreement concerning the subject
matter hereof. If any provision of this License is held to be
unenforceable, such provision shall be reformed only to the extent
necessary to make it enforceable. Any law or regulation which provides
that the language of a contract shall be construed against the drafter
shall not be used to construe this License against a Contributor.

10. Versions of the License
---------------------------

10.1. New Versions

Mozilla Foundation is the license steward. Except as provided in Section
10.3, no one other than the license steward has the right to modify or
publish new versions of this License. Each version will be given a
distinguishing version number.

10.2. Effect of New Versions

You may distribute the Covered Software under the terms of the version
of the License under which You originally received the Covered Software,
or under the terms of any subsequent version published by the license
steward.

10.3. Modified Versions

If you create software not governed by this License, and you want to
create a new license for such software, you may create and use a
modified version of this License if you rename the license and remove
any references to the name of the license steward (except to note that
such modified license differs from this License).

10.4. Distributing Source Code Form that is Incompatible With Secondary
Licenses

If You choose to distribute Source Code Form that is Incompatible With
Secondary Licenses under the terms of this version of the License, the
notice described in Exhibit B of this License must be attached.

Exhibit A - Source Code Form License Notice
-------------------------------------------

  This Source Code Form is subject to the terms of the Mozilla Public
  License, v. 2.0. If a copy of the MPL was not distributed with this
  file, You can obtain one at http://mozilla.org/MPL/2.0/.

If it is not possible or desirable to put the notice in a particular
file, then You may include the notice in a location (such as a LICENSE
file in a relevant directory) where a recipient would be likely to look
for such a notice.

You may add additional accurate notices of copyright ownership.

Exhibit B - "Incompatible With Secondary Licenses" Notice
---------------------------------------------------------

  This Source Code Form is "Incompatible With Secondary Licenses", as
  defined by the Mozilla Public License, v. 2.0.
~~~~

### SQLite public domain dedication

SQLite 3.53.2 is compiled into the exe from the amalgamation inside `libsqlite3-sys` 0.38.2 (the crate itself is MIT, Copyright (c) 2014 The rusqlite developers). Lines 33-41 of `sqlite3/sqlite3.c`. SQLite: https://sqlite.org, public domain; credited as a courtesy.

~~~~text
** 2001 September 15
**
** The author disclaims copyright to this source code.  In place of
** a legal notice, here is a blessing:
**
**    May you do good and not evil.
**    May you find forgiveness for yourself and forgive others.
**    May you share freely, never taking more than you give.
**
~~~~

### COPYRIGHT and SPDX files of some Rust crates

These are the closest equivalents to NOTICE files; no crate ships a NOTICE file. Verbatim from the crate sources named.

**rand project COPYRIGHT text** (source file `rand_core-0.10.1/COPYRIGHT`, from the rand repository)

~~~~text
Copyrights in the Rand project are retained by their contributors. No
copyright assignment is required to contribute to the Rand project.

For full authorship information, see the version control history.

Except as otherwise noted (below and/or in individual files), Rand is
licensed under the Apache License, Version 2.0 <LICENSE-APACHE> or
<http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
<LICENSE-MIT> or <http://opensource.org/licenses/MIT>, at your option.
~~~~

**utf8_iter 1.0.4** (source file `utf8_iter-1.0.4/COPYRIGHT`)

~~~~text
Copyright Mozilla Foundation

Licensed under the Apache License (Version 2.0), or the MIT license,
(the "Licenses") at your option. You may not use this file except in
compliance with one of the Licenses. You may obtain copies of the
Licenses at:

   https://www.apache.org/licenses/LICENSE-2.0
   https://opensource.org/licenses/MIT

Unless required by applicable law or agreed to in writing, software
distributed under the Licenses is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the Licenses for the specific language governing permissions and
limitations under the Licenses.

--

Test code is dedicated to the Public Domain when so designated (see
the individual files for PD/CC0-dedicated sections).

--

The implementation for Utf8CharIndices was adapted from the
CharIndices implementation of the Rust standard library at revision
ab32548539ec38a939c1b58599249f3b54130026
(https://github.com/rust-lang/rust/blob/ab32548539ec38a939c1b58599249f3b54130026/library/core/src/str/iter.rs).

Excerpt from https://github.com/rust-lang/rust/blob/ab32548539ec38a939c1b58599249f3b54130026/COPYRIGHT ,
which refers to
https://github.com/rust-lang/rust/blob/ab32548539ec38a939c1b58599249f3b54130026/LICENSE-APACHE
and
https://github.com/rust-lang/rust/blob/ab32548539ec38a939c1b58599249f3b54130026/LICENSE-MIT
:

For full authorship information, see the version control history or
https://thanks.rust-lang.org

Except as otherwise noted (below and/or in individual files), Rust is
licensed under the Apache License, Version 2.0 <LICENSE-APACHE> or
<http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
<LICENSE-MIT> or <http://opensource.org/licenses/MIT>, at your option.
~~~~

**unicode-normalization 0.1.26 (unicode-segmentation 1.13.3 has the same text)** (source file `unicode-normalization-0.1.26/COPYRIGHT`)

~~~~text
Licensed under the Apache License, Version 2.0
<LICENSE-APACHE or
http://www.apache.org/licenses/LICENSE-2.0> or the MIT
license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
at your option. All files in the project carrying such
notice may not be copied, modified, or distributed except
according to those terms.
~~~~

**tao 0.37.1 (SPDX file; analogous files in muda, tray-icon, wry, window-vibrancy, tauri-plugin-*)** (source file `tao-0.37.1/LICENSE.spdx`)

~~~~text
SPDXVersion: SPDX-2.1
DataLicense: CC0-1.0
PackageName: tao
DataFormat: SPDXRef-1
PackageSupplier: Organization: The Tauri Programme in the Commons Conservancy
PackageHomePage: https://tauri.app
PackageLicenseDeclared: Apache-2.0
PackageCopyrightText: 2021-2023, The Tauri Programme in the Commons Conservancy
PackageSummary: <text>Tao is the official, rust-based window manager for Wry.
                </text>
PackageComment: <text>The package includes the following libraries; see
Relationship information.
                </text>
Created: 2020-05-20T09:00:00Z
PackageDownloadLocation: git://github.com/tauri-apps/tao
PackageDownloadLocation: git+https://github.com/tauri-apps/tao.git
PackageDownloadLocation: git+ssh://github.com/tauri-apps/tao.git
Creator: Person: Daniel Thompson-Yvetot
~~~~

### SIL Open Font License 1.1, Inter

Font family: Inter (variable). Used through `@fontsource-variable/inter` 5.3.0. Copyright 2016 The Inter Project Authors (https://github.com/rsms/inter). No Reserved Font Name is declared in the shipped license file. Source: `@fontsource-variable/inter/LICENSE`. The fonts are included unmodified.

~~~~text
Copyright 2016 The Inter Project Authors (https://github.com/rsms/inter) Inter-Italic[opsz,wght].ttf: Copyright 2016 The Inter Project Authors (https://github.com/rsms/inter)

This Font Software is licensed under the SIL Open Font License, Version 1.1.
This license is copied below, and is also available with a FAQ at:
http://scripts.sil.org/OFL

-----------------------------------------------------------
SIL OPEN FONT LICENSE Version 1.1 - 26 February 2007
-----------------------------------------------------------

PREAMBLE
The goals of the Open Font License (OFL) are to stimulate worldwide
development of collaborative font projects, to support the font creation
efforts of academic and linguistic communities, and to provide a free and
open framework in which fonts may be shared and improved in partnership
with others.

The OFL allows the licensed fonts to be used, studied, modified and
redistributed freely as long as they are not sold by themselves. The
fonts, including any derivative works, can be bundled, embedded,
redistributed and/or sold with any software provided that any reserved
names are not used by derivative works. The fonts and derivatives,
however, cannot be released under any other type of license. The
requirement for fonts to remain under this license does not apply
to any document created using the fonts or their derivatives.

DEFINITIONS
"Font Software" refers to the set of files released by the Copyright
Holder(s) under this license and clearly marked as such. This may
include source files, build scripts and documentation.

"Reserved Font Name" refers to any names specified as such after the
copyright statement(s).

"Original Version" refers to the collection of Font Software components as
distributed by the Copyright Holder(s).

"Modified Version" refers to any derivative made by adding to, deleting,
or substituting -- in part or in whole -- any of the components of the
Original Version, by changing formats or by porting the Font Software to a
new environment.

"Author" refers to any designer, engineer, programmer, technical
writer or other person who contributed to the Font Software.

PERMISSION & CONDITIONS
Permission is hereby granted, free of charge, to any person obtaining
a copy of the Font Software, to use, study, copy, merge, embed, modify,
redistribute, and sell modified and unmodified copies of the Font
Software, subject to the following conditions:

1) Neither the Font Software nor any of its individual components,
in Original or Modified Versions, may be sold by itself.

2) Original or Modified Versions of the Font Software may be bundled,
redistributed and/or sold with any software, provided that each copy
contains the above copyright notice and this license. These can be
included either as stand-alone text files, human-readable headers or
in the appropriate machine-readable metadata fields within text or
binary files as long as those fields can be easily viewed by the user.

3) No Modified Version of the Font Software may use the Reserved Font
Name(s) unless explicit written permission is granted by the corresponding
Copyright Holder. This restriction only applies to the primary font name as
presented to the users.

4) The name(s) of the Copyright Holder(s) or the Author(s) of the Font
Software shall not be used to promote, endorse or advertise any
Modified Version, except to acknowledge the contribution(s) of the
Copyright Holder(s) and the Author(s) or with their explicit written
permission.

5) The Font Software, modified or unmodified, in part or in whole,
must be distributed entirely under this license, and must not be
distributed under any other license. The requirement for fonts to
remain under this license does not apply to any document created
using the Font Software.

TERMINATION
This license becomes null and void if any of the above conditions are
not met.

DISCLAIMER
THE FONT SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO ANY WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT
OF COPYRIGHT, PATENT, TRADEMARK, OR OTHER RIGHT. IN NO EVENT SHALL THE
COPYRIGHT HOLDER BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
INCLUDING ANY GENERAL, SPECIAL, INDIRECT, INCIDENTAL, OR CONSEQUENTIAL
DAMAGES, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
FROM, OUT OF THE USE OR INABILITY TO USE THE FONT SOFTWARE OR FROM
OTHER DEALINGS IN THE FONT SOFTWARE.
~~~~

### SIL Open Font License 1.1, JetBrains Mono

Font family: JetBrains Mono (variable). Used through `@fontsource-variable/jetbrains-mono` 5.3.0. Copyright 2020 The JetBrains Mono Project Authors (https://github.com/JetBrains/JetBrainsMono). No Reserved Font Name is declared in the shipped license file. Source: `@fontsource-variable/jetbrains-mono/LICENSE`. The fonts are included unmodified.

~~~~text
Copyright 2020 The JetBrains Mono Project Authors (https://github.com/JetBrains/JetBrainsMono) JetBrainsMono-Italic[wght].ttf: Copyright 2020 The JetBrains Mono Project Authors (https://github.com/JetBrains/JetBrainsMono)

This Font Software is licensed under the SIL Open Font License, Version 1.1.
This license is copied below, and is also available with a FAQ at:
http://scripts.sil.org/OFL

-----------------------------------------------------------
SIL OPEN FONT LICENSE Version 1.1 - 26 February 2007
-----------------------------------------------------------

PREAMBLE
The goals of the Open Font License (OFL) are to stimulate worldwide
development of collaborative font projects, to support the font creation
efforts of academic and linguistic communities, and to provide a free and
open framework in which fonts may be shared and improved in partnership
with others.

The OFL allows the licensed fonts to be used, studied, modified and
redistributed freely as long as they are not sold by themselves. The
fonts, including any derivative works, can be bundled, embedded,
redistributed and/or sold with any software provided that any reserved
names are not used by derivative works. The fonts and derivatives,
however, cannot be released under any other type of license. The
requirement for fonts to remain under this license does not apply
to any document created using the fonts or their derivatives.

DEFINITIONS
"Font Software" refers to the set of files released by the Copyright
Holder(s) under this license and clearly marked as such. This may
include source files, build scripts and documentation.

"Reserved Font Name" refers to any names specified as such after the
copyright statement(s).

"Original Version" refers to the collection of Font Software components as
distributed by the Copyright Holder(s).

"Modified Version" refers to any derivative made by adding to, deleting,
or substituting -- in part or in whole -- any of the components of the
Original Version, by changing formats or by porting the Font Software to a
new environment.

"Author" refers to any designer, engineer, programmer, technical
writer or other person who contributed to the Font Software.

PERMISSION & CONDITIONS
Permission is hereby granted, free of charge, to any person obtaining
a copy of the Font Software, to use, study, copy, merge, embed, modify,
redistribute, and sell modified and unmodified copies of the Font
Software, subject to the following conditions:

1) Neither the Font Software nor any of its individual components,
in Original or Modified Versions, may be sold by itself.

2) Original or Modified Versions of the Font Software may be bundled,
redistributed and/or sold with any software, provided that each copy
contains the above copyright notice and this license. These can be
included either as stand-alone text files, human-readable headers or
in the appropriate machine-readable metadata fields within text or
binary files as long as those fields can be easily viewed by the user.

3) No Modified Version of the Font Software may use the Reserved Font
Name(s) unless explicit written permission is granted by the corresponding
Copyright Holder. This restriction only applies to the primary font name as
presented to the users.

4) The name(s) of the Copyright Holder(s) or the Author(s) of the Font
Software shall not be used to promote, endorse or advertise any
Modified Version, except to acknowledge the contribution(s) of the
Copyright Holder(s) and the Author(s) or with their explicit written
permission.

5) The Font Software, modified or unmodified, in part or in whole,
must be distributed entirely under this license, and must not be
distributed under any other license. The requirement for fonts to
remain under this license does not apply to any document created
using the Font Software.

TERMINATION
This license becomes null and void if any of the above conditions are
not met.

DISCLAIMER
THE FONT SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO ANY WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT
OF COPYRIGHT, PATENT, TRADEMARK, OR OTHER RIGHT. IN NO EVENT SHALL THE
COPYRIGHT HOLDER BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
INCLUDING ANY GENERAL, SPECIAL, INDIRECT, INCIDENTAL, OR CONSEQUENTIAL
DAMAGES, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
FROM, OUT OF THE USE OR INABILITY TO USE THE FONT SOFTWARE OR FROM
OTHER DEALINGS IN THE FONT SOFTWARE.
~~~~

### SIL Open Font License 1.1, KaTeX fonts

Font families: KaTeX_AMS, KaTeX_Caligraphic, KaTeX_Fraktur, KaTeX_Main, KaTeX_Math, KaTeX_SansSerif, KaTeX_Script, KaTeX_Size1, KaTeX_Size2, KaTeX_Size3, KaTeX_Size4, KaTeX_Typewriter, from `katex` 0.19.0 (`dist/fonts`). Copyright (c) 2009-2010 Design Science, Inc. (www.mathjax.org); Copyright (c) 2014-2018 Khan Academy (www.khanacademy.org). Each family carries its own Reserved Font Name (its family name, for example "KaTeX_Main"). The `katex` package has no separate font license file; the notice below is embedded in the font files (name table IDs 0, 13 and 14, extracted from `KaTeX_Main-Regular.ttf`; the other 19 files carry the same text with their own family name). The fonts are included unmodified. The full OFL 1.1 text that goes with this notice is the same text as in the Inter section above and is repeated here.

~~~~text
Name ID 0 (copyright):
Copyright (c) 2009-2010 Design Science, Inc.
Copyright (c) 2014-2018 Khan Academy

Name ID 13 (license description):
Copyright (c) 2009-2010, Design Science, Inc. (<www.mathjax.org>)
Copyright (c) 2014-2018 Khan Academy (<www.khanacademy.org>),
with Reserved Font Name KaTeX_Main.

This Font Software is licensed under the SIL Open Font License, Version 1.1.
This license available with a FAQ at:
http://scripts.sil.org/OFL

Name ID 14 (license URL):
http://scripts.sil.org/OFL

-----------------------------------------------------------
SIL OPEN FONT LICENSE Version 1.1 - 26 February 2007
-----------------------------------------------------------

PREAMBLE
The goals of the Open Font License (OFL) are to stimulate worldwide
development of collaborative font projects, to support the font creation
efforts of academic and linguistic communities, and to provide a free and
open framework in which fonts may be shared and improved in partnership
with others.

The OFL allows the licensed fonts to be used, studied, modified and
redistributed freely as long as they are not sold by themselves. The
fonts, including any derivative works, can be bundled, embedded,
redistributed and/or sold with any software provided that any reserved
names are not used by derivative works. The fonts and derivatives,
however, cannot be released under any other type of license. The
requirement for fonts to remain under this license does not apply
to any document created using the fonts or their derivatives.

DEFINITIONS
"Font Software" refers to the set of files released by the Copyright
Holder(s) under this license and clearly marked as such. This may
include source files, build scripts and documentation.

"Reserved Font Name" refers to any names specified as such after the
copyright statement(s).

"Original Version" refers to the collection of Font Software components as
distributed by the Copyright Holder(s).

"Modified Version" refers to any derivative made by adding to, deleting,
or substituting -- in part or in whole -- any of the components of the
Original Version, by changing formats or by porting the Font Software to a
new environment.

"Author" refers to any designer, engineer, programmer, technical
writer or other person who contributed to the Font Software.

PERMISSION & CONDITIONS
Permission is hereby granted, free of charge, to any person obtaining
a copy of the Font Software, to use, study, copy, merge, embed, modify,
redistribute, and sell modified and unmodified copies of the Font
Software, subject to the following conditions:

1) Neither the Font Software nor any of its individual components,
in Original or Modified Versions, may be sold by itself.

2) Original or Modified Versions of the Font Software may be bundled,
redistributed and/or sold with any software, provided that each copy
contains the above copyright notice and this license. These can be
included either as stand-alone text files, human-readable headers or
in the appropriate machine-readable metadata fields within text or
binary files as long as those fields can be easily viewed by the user.

3) No Modified Version of the Font Software may use the Reserved Font
Name(s) unless explicit written permission is granted by the corresponding
Copyright Holder. This restriction only applies to the primary font name as
presented to the users.

4) The name(s) of the Copyright Holder(s) or the Author(s) of the Font
Software shall not be used to promote, endorse or advertise any
Modified Version, except to acknowledge the contribution(s) of the
Copyright Holder(s) and the Author(s) or with their explicit written
permission.

5) The Font Software, modified or unmodified, in part or in whole,
must be distributed entirely under this license, and must not be
distributed under any other license. The requirement for fonts to
remain under this license does not apply to any document created
using the Font Software.

TERMINATION
This license becomes null and void if any of the above conditions are
not met.

DISCLAIMER
THE FONT SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO ANY WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT
OF COPYRIGHT, PATENT, TRADEMARK, OR OTHER RIGHT. IN NO EVENT SHALL THE
COPYRIGHT HOLDER BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
INCLUDING ANY GENERAL, SPECIAL, INDIRECT, INCIDENTAL, OR CONSEQUENTIAL
DAMAGES, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
FROM, OUT OF THE USE OR INABILITY TO USE THE FONT SOFTWARE OR FROM
OTHER DEALINGS IN THE FONT SOFTWARE.
~~~~

### Components whose license text is not included

- The Microsoft Edge WebView2 Runtime: Microsoft's own terms, not reproduced (it is installed on the PC, not shipped). The loader library's text is reproduced above (open decision 5).
- Zetetic LLC BSD-3 license in the `sqlcipher/` folder of `libsqlite3-sys`: that code is not compiled in (the sqlcipher features are not enabled).
