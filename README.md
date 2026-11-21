# canon-datastar

The [Astryx](https://astryx.atmeta.com) design system ([source](https://github.com/meta/astryx), MIT)
as server-rendered HTML for [Datastar](https://data-star.dev) apps, in **Go** ([templ](https://templ.guide))
and **Rust** ([maud](https://maud.lambda.xyz)). Sister project of
[canon-svelte](https://github.com/sensibile/canon-svelte); both use the same Astryx CSS, class
names (`astryx-*`) and `data-*` attributes.

Components render plain HTML on the server. Interactive parts (dialogs, menus, tabs, toasts) use
Datastar attributes and signals in the browser, and SSE where the server drives the UI.

## Layout

| Path | What |
| --- | --- |
| `css/` | Shared stylesheets: `astryx.css` (reset + tokens + theme scope), `themes/<name>.css` |
| `static/datastar.js` | Datastar client v1.0.5 (MIT, Star Federation), vendored for convenience |
| `go/canon` | Go package: templ components |
| `rust/` | Rust crate `canon-datastar`: maud components |
| `fixtures/` | Golden HTML per component case. Go and Rust tests both render to these exact files |
| `demo/`, `go/cmd/demo`, `rust/examples/demo` | Gallery servers (same page in each language) |

## Use

Link the base stylesheet, every theme you use, and Datastar:

```html
<link rel="stylesheet" href="/canon/css/astryx.css">
<link rel="stylesheet" href="/canon/css/themes/neutral.css">
<script type="module" src="/canon/static/datastar.js"></script>
```

Themes: `neutral`, `butter`, `chocolate`, `matcha`, `stone`, `gothic`, `y2k`. Each theme file
brings its fonts. Put `data-astryx-theme="<name>"` (and `data-theme="light|dark"`, or nothing to
follow the OS) on `<html>`, and use `Theme` to scope a different theme to part of the page.

### Go

```go
import (
	canondatastar "github.com/sensibile/canon-datastar"
	"github.com/sensibile/canon-datastar/go/canon"
)

// Serves css/ and static/ from the embedded copy.
mux.Handle("GET /canon/", http.StripPrefix("/canon/", http.FileServerFS(canondatastar.Assets)))
```

```templ
@canon.Theme(canon.ThemeProps{Name: "matcha", Mode: canon.ModeDark}) {
	<p>Hello</p>
}
```

### Rust

```rust
use canon_datastar::{theme, ThemeMode, ThemeProps};

let page = theme(
    &ThemeProps { name: Some("matcha".into()), mode: ThemeMode::Dark, ..Default::default() },
    maud::html! { p { "Hello" } },
);
```

Serve `css/` and `static/` from this repo (e.g. `tower_http::services::ServeDir`).

Every component takes extra attributes (`Attrs` / `attrs`) for Datastar bindings, e.g.
`data-attr:data-astryx-theme="$theme"`. They render in the order given.

## Components

- `Theme`

More follow in stages: basic components, overlays, then the remaining Astryx categories.

## Develop

```sh
go run ./go/cmd/demo                      # http://localhost:8080
(cd rust && cargo run --example demo)     # http://localhost:8081
go test ./...                             # Go vs fixtures/
(cd rust && cargo test)                   # Rust vs fixtures/
templ generate                            # after editing .templ files (commit the generated *_templ.go)
UPDATE_FIXTURES=1 go test ./go/...        # rewrite fixtures/ from the Go output, then check Rust against them
node scripts/sync-astryx.mjs [version]    # re-pull reset, tokens and theme CSS from the Astryx npm packages
```

Markup notes for keeping Go and Rust identical: tests normalise only quote escaping (`&#34;` vs
`&quot;`, `&#39;` vs `'`). templ appends `;` to `style` attribute values and maud does not, so
write inline styles with a trailing `;` in Rust or avoid them in components.

## What comes from Astryx

- `css/reset.css`, `css/themes/<name>/*`: copied verbatim from `@astryxdesign/core` and
  `@astryxdesign/theme-<name>` (currently 0.7.17), fonts and their licenses included. Identical to
  canon-svelte's copies.
- `css/tokens.css`: default token values extracted from `@astryxdesign/core/dist/astryx.css`.
- Components: re-implemented as HTML templates, ported from the React sources.

See [LICENSE](./LICENSE) for attribution.
