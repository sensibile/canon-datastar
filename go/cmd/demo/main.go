// Command demo serves the canon-datastar gallery: go run ./go/cmd/demo (from the repo root).
package main

import (
	"encoding/json"
	"log"
	"net/http"
	"os"
	"slices"
	"time"

	"github.com/starfederation/datastar-go/datastar"

	canondatastar "github.com/sensibile/canon-datastar"
	"github.com/sensibile/canon-datastar/go/canon"
)

func main() {
	mux := http.NewServeMux()
	mux.Handle("GET /canon/", http.StripPrefix("/canon/", http.FileServerFS(canondatastar.Assets)))
	mux.Handle("GET /demo/", http.StripPrefix("/demo/", http.FileServer(http.Dir("demo"))))
	mux.HandleFunc("GET /{$}", func(w http.ResponseWriter, r *http.Request) {
		theme, mode := themeFromQuery(r)
		page := index(theme, mode)
		if err := page.Render(r.Context(), w); err != nil {
			log.Print(err)
		}
	})
	mux.HandleFunc("GET /components", func(w http.ResponseWriter, r *http.Request) {
		theme, mode := themeFromQuery(r)
		if err := componentsIndex(theme, mode).Render(r.Context(), w); err != nil {
			log.Print(err)
		}
	})
	mux.HandleFunc("GET /content", func(w http.ResponseWriter, r *http.Request) {
		theme, mode := themeFromQuery(r)
		if err := contentIndex(theme, mode).Render(r.Context(), w); err != nil {
			log.Print(err)
		}
	})
	mux.HandleFunc("GET /forms", func(w http.ResponseWriter, r *http.Request) {
		theme, mode := themeFromQuery(r)
		if err := formsIndex(theme, mode).Render(r.Context(), w); err != nil {
			log.Print(err)
		}
	})
	mux.HandleFunc("GET /dates", func(w http.ResponseWriter, r *http.Request) {
		theme, mode := themeFromQuery(r)
		if err := datesIndex(theme, mode).Render(r.Context(), w); err != nil {
			log.Print(err)
		}
	})
	// Forms: echo the signals the page sent, to show what the bound controls hold.
	mux.HandleFunc("POST /demo/forms", func(w http.ResponseWriter, r *http.Request) {
		var signals map[string]any
		if err := datastar.ReadSignals(r, &signals); err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}
		out, _ := json.MarshalIndent(signals, "", "  ")
		if err := datastar.NewSSE(w, r).PatchElementTempl(submitted(string(out))); err != nil {
			log.Print(err)
		}
	})
	// Typeahead/Tokenizer remote search: answer with the results for the query (after a short
	// delay, to show the loading state).
	mux.HandleFunc("GET /demo/search", func(w http.ResponseWriter, r *http.Request) {
		q := r.URL.Query().Get("q")
		time.Sleep(200 * time.Millisecond)
		results := canon.TypeaheadResults(canon.TypeaheadResultsProps{ID: r.URL.Query().Get("id"), Query: q,
			Items: searchCities(q)})
		if err := datastar.NewSSE(w, r).PatchElementTempl(results); err != nil {
			log.Print(err)
		}
	})
	// ProgressBar over SSE: patch the bar as the "upload" advances; Datastar morphs it in place.
	mux.HandleFunc("POST /demo/progress", func(w http.ResponseWriter, r *http.Request) {
		sse := datastar.NewSSE(w, r)
		for v := 0.0; v <= 100; v += 10 {
			if err := sse.PatchElementTempl(progressBar(v)); err != nil {
				log.Print(err)
				return
			}
			time.Sleep(250 * time.Millisecond)
		}
	})
	mux.HandleFunc("GET /overlays", func(w http.ResponseWriter, r *http.Request) {
		theme, mode := themeFromQuery(r)
		if err := overlaysIndex(theme, mode).Render(r.Context(), w); err != nil {
			log.Print(err)
		}
	})
	// Toast over SSE: append a server-rendered Toast to the viewport.
	mux.HandleFunc("POST /demo/toast", func(w http.ResponseWriter, r *http.Request) {
		sse := datastar.NewSSE(w, r)
		t := canon.Toast(canon.ToastProps{Type: r.URL.Query().Get("type"), Body: r.URL.Query().Get("body")})
		if err := sse.PatchElementTempl(t, datastar.WithSelectorID("astryx-toasts"), datastar.WithModeAppend()); err != nil {
			log.Print(err)
		}
	})
	// Dialog over SSE: close the confirm dialog through its signal and show a toast.
	mux.HandleFunc("POST /demo/delete", func(w http.ResponseWriter, r *http.Request) {
		var signals struct {
			Deleted int `json:"deleted"`
		}
		if err := datastar.ReadSignals(r, &signals); err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}
		sse := datastar.NewSSE(w, r)
		time.Sleep(800 * time.Millisecond)
		if err := sse.MarshalAndPatchSignals(map[string]any{"confirmOpen": false, "deleted": signals.Deleted + 1}); err != nil {
			log.Print(err)
		}
		if err := sse.PatchElementTempl(canon.Toast(canon.ToastProps{Body: "Project deleted"}),
			datastar.WithSelectorID("astryx-toasts"), datastar.WithModeAppend()); err != nil {
			log.Print(err)
		}
	})
	// Datastar demo: the button sets $saving, posts its signals here, and this SSE response
	// patches $saving back to false (clearing aria-busy) after a short delay.
	mux.HandleFunc("POST /demo/save", func(w http.ResponseWriter, r *http.Request) {
		var signals struct {
			Saved int `json:"saved"`
		}
		if err := datastar.ReadSignals(r, &signals); err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}
		sse := datastar.NewSSE(w, r)
		time.Sleep(1200 * time.Millisecond)
		if err := sse.MarshalAndPatchSignals(map[string]any{"saving": false, "saved": signals.Saved + 1}); err != nil {
			log.Print(err)
		}
	})
	addr := os.Getenv("ADDR")
	if addr == "" {
		addr = "localhost:8080"
	}
	log.Printf("canon-datastar demo on http://%s", addr)
	log.Fatal(http.ListenAndServe(addr, mux))
}

// themeFromQuery lets ?theme=gothic&mode=dark pick the server-rendered starting point.
func themeFromQuery(r *http.Request) (string, canon.ThemeMode) {
	theme := r.URL.Query().Get("theme")
	if !slices.Contains(canon.Themes, theme) {
		theme = "neutral"
	}
	mode := canon.ThemeMode(r.URL.Query().Get("mode"))
	if mode != canon.ModeLight && mode != canon.ModeDark {
		mode = canon.ModeSystem
	}
	return theme, mode
}
