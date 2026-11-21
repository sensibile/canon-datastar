// Command demo serves the canon-datastar gallery: go run ./go/cmd/demo (from the repo root).
package main

import (
	"log"
	"net/http"
	"os"
	"slices"

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
