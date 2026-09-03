# Gates: Issue #1 Ongrade Tolerance Range, High-Contrast Colors, & Correct Cut/Fill Logic

OWNS: src/config.rs, src/grid_engine.rs, src/html_exporter.rs, src/volume.rs, src/main.rs, config.dat, tests/**

Scope: Fix inverted cut/fill logic (Topo vs Design), configurable ongrade tolerance range in config.dat, customizable high-contrast color palette, and synchronized legend/raster rendering.

- [x] G1: EngineConfig parses ongrade tolerance and custom color palette from config.dat
  CHECK: cargo test --test test_config test_config_ongrade_and_custom_colors -- --exact
  EXPECT: test test_config_ongrade_and_custom_colors ... ok
  EVIDENCE: exit=0; shell=/bin/sh; cwd=/home/cells/Documents/Antigravity Project/Rainbow-Contour; path=44c2620c68ad/42 entries; EXPECT=matched; output-sha256=146f035223eca25d286458239568fe37d51ae2cdd33abdc6b84c285c96fd9a33; output-bytes=319

- [x] G2: Cut and Fill logic is mathematically correct (Topo > Design is Cut, Topo < Design is Fill)
  CHECK: cargo test --test test_volume test_cut_fill_definition_and_ongrade -- --exact
  EXPECT: test test_cut_fill_definition_and_ongrade ... ok
  EVIDENCE: exit=0; shell=/bin/sh; cwd=/home/cells/Documents/Antigravity Project/Rainbow-Contour; path=44c2620c68ad/42 entries; EXPECT=matched; output-sha256=026f3b564d44a33d832044233539b8e2201f53d067db6d84837d05252e5bcc3f; output-bytes=318

- [x] G3: HTML exporter renders custom high-contrast colors and dynamic ongrade range in canvas & legend
  CHECK: cargo test --test test_html_exporter test_html_exporter_custom_colors_and_ongrade -- --exact
  EXPECT: test test_html_exporter_custom_colors_and_ongrade ... ok
  EVIDENCE: exit=0; shell=/bin/sh; cwd=/home/cells/Documents/Antigravity Project/Rainbow-Contour; path=44c2620c68ad/42 entries; EXPECT=matched; output-sha256=4c9c6f145f4bf1de975d91715e3205c6d76cd6bdb67ab11c0d16b97b5cbb1b3f; output-bytes=340

- [x] G4: Full CLI and integration suite pass with backward-compatible defaults
  CHECK: cargo test --test test_cli --test test_e2e_pipeline --test test_wrappers
  EXPECT: test test_launcher_scripts_exist ... ok
  EVIDENCE: exit=0; shell=/bin/sh; cwd=/home/cells/Documents/Antigravity Project/Rainbow-Contour; path=44c2620c68ad/42 entries; EXPECT=matched; output-sha256=5d0c4e88ad302d9d371ef8970b1bc8598eb802c63d9b6cb93de7bb512516fc07; output-bytes=2506
