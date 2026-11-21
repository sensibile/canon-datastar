// Package canondatastar embeds the shared stylesheets and the Datastar client so a
// Go server can serve them without copying files:
//
//	http.Handle("/canon/", http.StripPrefix("/canon/", http.FileServerFS(canondatastar.Assets)))
//
// Assets holds css/ (astryx.css, reset.css, tokens.css, theme.css, themes/...) and
// static/datastar.js. Components live in the go/canon package.
package canondatastar

import "embed"

//go:embed css static
var Assets embed.FS
