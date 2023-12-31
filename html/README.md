# Instructions for preparing for WASM

1. Change the window to `fit_canvas_to_parent = true` (https://github.com/bevyengine/bevy/blob/main/examples/window/window_settings.rs) and set the body CSS to `margin: 0; height: 100%`.
3. Read: https://github.com/bevyengine/bevy/blob/latest/examples/README.md#wasm

# Instructions for building WASM

1. Install target: `rustup target install wasm32-unknown-unknown`
3. Install bindgen: `cargo install wasm-bindgen-cli`
4. Build for WASM: `cargo build --profile wasm-release --target wasm32-unknown-unknown`
5. Generate bindings: `wasm-bindgen --out-dir ./target/html --target web .\target\wasm32-unknown-unknown\wasm-release\ludum_dare_54.wasm`

# Deploy to GitHub Pages

1. Do the above 
2. Switch to the `gh-pages` branch (created with `git switch --orphan gh-pages`)
3. Get the index: `git checkout master -- html/index.html; rm index.html; cp html/index.html .`
3. Get the assets: `git checkout master -- assets`
5. Get the WASM files: `wasm-bindgen --out-dir . --target web .\target\wasm32-unknown-unknown\wasm-release\ludum_dare_54.wasm`
5. Test locally: `python -m http.server`
6. Commit and push
