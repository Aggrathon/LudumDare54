# Instructions for building WASM

1. Install target: `rustup target install wasm32-unknown-unknown`
2. (Optional) Install runner for local tests (see *.cargo/config.toml*): `cargo install wasm-server-runner`
3. Install bindgen: `cargo install wasm-bindgen-cli`
4. Build for WASM: `cargo build --release --target wasm32-unknown-unknown`
5. Generate bindings: `wasm-bindgen --out-dir ./target/html --target web .\target\wasm32-unknown-unknown\release\ludum_dare_54.wasm`

## Deploy to GitHub Pages

1. Do the above 
2. Switch to the `gh-pages` branch (created with `git switch --orphan gh-pages`)
3. Get the index: `git checkout master -- html/index.html; rm index.html; cp html/index.html .`
3. Get the assets: `git checkout master -- assets`
5. Get the wasm: `wasm-bindgen --out-dir ./out/ --target web .\target\wasm32-unknown-unknown\release\ludum_dare_54.wasm`
5. Test locally: `python -m http.test`
6. Commit and push
