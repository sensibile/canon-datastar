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
| `static/canon.js` | Overlay behaviour: positioning, menu and tab keyboard, tooltips, toasts, `window.canon` |
| `static/datastar.js` | Datastar client v1.0.5 (MIT, Star Federation), vendored for convenience |
| `go/canon` | Go package: templ components |
| `rust/` | Rust crate `canon-datastar`: maud components |
| `fixtures/` | Golden HTML per component case. Go and Rust tests both render to these exact files |
| `icons/default-icons.json` | Astryx's built-in glyphs; `scripts/gen-icons.mjs` turns it into Go and Rust tables |
| `demo/`, `go/cmd/demo`, `rust/examples/demo` | Gallery servers (same page in each language) |

## Use

Link the base stylesheet, every theme you use, `canon.js` and then Datastar:

```html
<link rel="stylesheet" href="/canon/css/astryx.css">
<link rel="stylesheet" href="/canon/css/themes/neutral.css">
<script type="module" src="/canon/static/canon.js"></script>
<script type="module" src="/canon/static/datastar.js"></script>
```

`canon.js` goes first so Datastar expressions can call `canon.*`. Pages without overlays don't need it.

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

Components are `templ.Component`s, so they work inside `.templ` files or on their own:

```templ
@canon.Theme(canon.ThemeProps{Name: "matcha", Mode: canon.ModeDark}) {
	@canon.Button(canon.ButtonProps{Label: "Save", Variant: "primary", Attrs: []canon.Attr{
		canon.A("data-attr:aria-busy", "$saving && 'true'"),
		canon.A("data-on:click", "$saving = true; @post('/save')"),
	}})
	@canon.TextInput(canon.TextInputProps{ID: "email", Label: "Email", InputAttrs: []canon.Attr{canon.A("data-bind:email", "")}})
}
```

### Rust

```rust
use canon_datastar::*;

let page = theme(
    &ThemeProps { name: Some("matcha".into()), mode: ThemeMode::Dark, ..Default::default() },
    maud::html! {
        (button(&ButtonProps { label: "Save".into(), variant: "primary".into(), ..Default::default() }))
        (text_input(&TextInputProps {
            id: "email".into(),
            label: "Email".into(),
            input_attrs: vec![attr("data-bind:email", "")],
            ..Default::default()
        }))
    },
);
```

Serve `css/` and `static/` from this repo (e.g. `tower_http::services::ServeDir`).

### Props and Datastar

Props mirror Astryx's (`isDisabled` → `IsDisabled` / `is_disabled`, `type` → `Type` / `kind`).
Enum-like props are strings, and `""` means Astryx's default. Every component takes extra
attributes for Datastar bindings (`Attrs` / `attrs`, plus `InputAttrs` / `input_attrs` on form
controls for the `<input>`); they render in the order given.

Component state lives in attributes and native input state, so Datastar can drive it without
re-rendering: the button spinner shows while `aria-busy="true"` (bind it with
`data-attr:aria-busy="$saving && 'true'"`), checkbox and switch visuals follow the input's
`:checked`, so `data-bind` just works.

### Overlays

Overlays are native: `Tooltip`, `Popover` and `DropdownMenu` render a `popover` element (top layer,
light dismiss, Escape and focus return from the browser), `Dialog` renders a modal `<dialog>`.
`static/canon.js` adds the rest through event delegation, so markup patched in over SSE works with
no setup: anchored positioning with flip and shift, the menu keyboard model (arrows, Home/End,
typeahead), tooltip hover and focus timing, InfoTip pinning, Dialog `Purpose`, tab roving focus,
and toast timers. Wire it to Datastar like this:

- Open state as a signal: `Dialog` and `Popover` take `Signal: "confirmOpen"`. Setting the signal
  opens or closes them, and closing them in the browser sets it back, so the server can close a
  dialog by patching `{confirmOpen: false}`.
- Or call the API from any expression: `canon.dialog('settings')`, `canon.popover('filters', false)`,
  `canon.toast({body: 'Copied'})`.
- Menu items and tabs take Datastar actions in `Attrs`, e.g. `data-on:click="@post('/archive')"`.
  `TabList` with `Signal: "tab"` keeps the selected tab in `$tab`, and `TabPanel`s follow it.
- Toasts from the server: render one `ToastViewport` per page and patch a `Toast` into it, e.g.
  `sse.PatchElementTempl(canon.Toast(canon.ToastProps{Body: "Saved"}), datastar.WithSelectorID("astryx-toasts"), datastar.WithModeAppend())`
  in Go, or `PatchElements::new(toast(&p).into_string()).selector("#astryx-toasts").mode(ElementPatchMode::Append)` in Rust.
  While a Dialog is open, canon.js moves the viewport inside it (the rest of the page is inert)
  and puts it back when the Dialog closes, so patch toasts by the viewport's id, not its position.

### Layout and content

Most of these are plain markup. Where Astryx keeps state in the browser, `canon.js` does it and the
server stays in charge of the first render:

- `Collapsible`, `CollapsibleGroup` (single or multiple open) and `Banner` bodies expand and collapse
  on click and fire a bubbling `canon-toggle` event (`evt.detail.open`). With `Signal: "details"`
  the open state mirrors `$details` both ways; `canon.collapse(el, open)` does it from any expression.
- `Banner` dismiss hides the banner, returns focus to where it came from and fires `canon-dismiss`;
  add `DismissAttrs` (e.g. `data-on:click="@post('/dismiss')"`) to tell the server.
- `Avatar` and `Thumbnail` render their fallback hidden next to the image; a failed image switches
  to `FallbackSrc`, then to the initials, glyph or placeholder.
- `Token` (`Href` or `Action`), `Avatar` and `Thumbnail` take Datastar actions in `Action`, and
  removable ones in `RemoveAttrs`. `AvatarGroup` avatars are one tab stop moved with the arrow keys.
- `MetadataList` renders items past `MaxNumOfItems` hidden behind "Show more".
- `Kbd` renders the Ctrl form; on Apple platforms it switches to ⌘ ⌃ ⌥ ⇧.
- `Timestamp` renders `<time datetime>` with a UTC fallback; the browser formats it with Intl in
  the viewer's zone and locale (relative, absolute or `auto`), keeps `IsLive` times ticking and fills
  the details tooltip. In Rust the value is Unix milliseconds (`value_ms`).
- `ProgressBar`: patch it from the server to move it (the demo streams an upload over SSE).

Give each overlay an `ID` when a page has more than one of a kind; defaults are derived from the
content (`tooltip-<slug>`) or fixed (`popover`, `menu`).

## Components

- Content: `Text`, `Heading`, `Icon`, `Badge`, `Spinner`, `VisuallyHidden`
- Action: `Button`, `IconButton`, `Link`
- Layout: `Stack`, `HStack`, `VStack`, `Divider`, `Card`, `Section`, `Center`, `Grid`, `GridSpan`,
  `AspectRatio`, `Collapsible`, `CollapsibleGroup`
- Display: `Avatar` (with `AvatarStatusDotProps`), `AvatarGroup`, `Token`, `Banner`, `Blockquote`,
  `Code`, `Kbd`, `EmptyState`, `Skeleton`, `StatusDot`, `ProgressBar`, `MetadataList`, `Thumbnail`,
  `Timestamp`
- Form: `Field`, `FieldLabel`, `FieldStatusMessage` / `field_status`, `TextInput`, `CheckboxInput`, `Switch`
- Overlay: `Tooltip`, `InfoTip`, `Popover`, `Dialog`, `DialogHeader`, `DropdownMenu`,
  `DropdownMenuItem`, `DropdownMenuGroup`, `DropdownMenuDivider`, `TabList` (with `TabProps`),
  `TabPanel`, `Toast`, `ToastViewport`
- `Theme`

Differences from Astryx and canon-svelte, because the HTML is rendered on the server:
a disabled `Button`/`CheckboxInput`/`Switch` uses the native `disabled` attribute; async
`clickAction` is replaced by Datastar actions; form controls need an `ID` (otherwise one is derived
from the label). An indeterminate checkbox and click-blocking on read-only or loading controls use
Datastar (`data-init`, `data-on:click`). Overlays: `TabList` takes its tabs as data rather than
children (so do `CollapsibleGroup`, `AvatarGroup` and `MetadataList`); a Tooltip's `aria-describedby` is added by `canon.js`; opening a modal Dialog closes open
popovers (the browser's rule for the top layer); DropdownMenu items are composed as children only.
Not ported yet: `TextInput` clear button, `disabledMessage`, `labelTooltip`, custom (non built-in)
icons, Button tooltips, touch long-press and bottom-sheet presentation, menu submenus and
checkbox/radio items, tab overflow arrows, toast swipe, Collapsible height animation, the
Timestamp hover card with copy buttons, emoji initials in Avatar. The remaining Astryx categories come next.

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

Keeping Go and Rust identical: components are written with a small builder in each language
(`go/canon/html.go`, `rust/src/html.rs`) that writes attributes in a fixed order with the same
escaping, rather than with templ/maud templates, whose whitespace and `style` handling differ. Add a
component by writing it in both, adding the same cases to `go/canon/*_test.go` and
`rust/tests/fixtures.rs`, and generating the fixtures from Go. `node scripts/gen-icons.mjs`
regenerates the icon tables after editing `icons/default-icons.json`.

## What comes from Astryx

- `css/reset.css`, `css/themes/<name>/*`: copied verbatim from `@astryxdesign/core` and
  `@astryxdesign/theme-<name>` (currently 0.7.17), fonts and their licenses included. Identical to
  canon-svelte's copies.
- `css/tokens.css`: default token values extracted from `@astryxdesign/core/dist/astryx.css`.
- `icons/default-icons.json`: Astryx's default glyphs (`packages/core/src/Icon/defaultIcons.tsx`).
- Components: re-implemented as HTML builders and global CSS, ported from the React sources
  (via canon-svelte).

See [LICENSE](./LICENSE) for attribution.
