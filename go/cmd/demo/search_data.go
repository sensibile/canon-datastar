package main

import (
	"strings"

	"github.com/sensibile/canon-datastar/go/canon"
)

// people is a static Typeahead list with descriptions and keywords.
func people() []canon.TypeaheadItem {
	return []canon.TypeaheadItem{
		{ID: "ada", Label: "Ada Lovelace", Description: "Analytical Engine", Keywords: []string{"math"}},
		{ID: "alan", Label: "Alan Turing", Description: "Computability", Keywords: []string{"enigma"}},
		{ID: "grace", Label: "Grace Hopper", Description: "Compilers", Keywords: []string{"cobol", "navy"}},
		{ID: "edsger", Label: "Edsger Dijkstra", Description: "Shortest paths"},
		{ID: "barbara", Label: "Barbara Liskov", Description: "Substitution"},
		{ID: "donald", Label: "Donald Knuth", Description: "TeX", Keywords: []string{"taocp"}},
		{ID: "margaret", Label: "Margaret Hamilton", Description: "Apollo software", Keywords: []string{"nasa"}},
		{ID: "ken", Label: "Ken Thompson", Description: "Unix, Go"},
	}
}

// languages is a static Tokenizer list.
func languages() []canon.TypeaheadItem {
	var out []canon.TypeaheadItem
	for _, l := range [][2]string{{"go", "Go"}, {"rust", "Rust"}, {"ts", "TypeScript"}, {"py", "Python"},
		{"zig", "Zig"}, {"c", "C"}, {"kt", "Kotlin"}, {"swift", "Swift"}, {"ex", "Elixir"}, {"hs", "Haskell"}} {
		out = append(out, canon.TypeaheadItem{ID: l[0], Label: l[1]})
	}
	return out
}

var cityNames = []string{"Amsterdam", "Athens", "Bangkok", "Barcelona", "Berlin", "Bogotá", "Buenos Aires",
	"Busan", "Cairo", "Cape Town", "Chicago", "Copenhagen", "Daejeon", "Delhi", "Dublin", "Helsinki",
	"Istanbul", "Jakarta", "Lagos", "Lima", "Lisbon", "London", "Los Angeles", "Madrid", "Melbourne",
	"Mexico City", "Montréal", "Mumbai", "Nairobi", "New York", "Osaka", "Oslo", "Paris", "Prague",
	"Rome", "San Francisco", "Santiago", "São Paulo", "Seoul", "Singapore", "Stockholm", "Sydney",
	"Taipei", "Tokyo", "Toronto", "Vienna", "Warsaw", "Zürich"}

// searchCities is the demo's server-side search: a case-insensitive substring match.
func searchCities(query string) []canon.TypeaheadItem {
	q := strings.ToLower(strings.TrimSpace(query))
	var out []canon.TypeaheadItem
	for _, c := range cityNames {
		if q == "" || strings.Contains(strings.ToLower(c), q) {
			out = append(out, canon.TypeaheadItem{ID: strings.ToLower(strings.ReplaceAll(c, " ", "-")), Label: c})
		}
	}
	return out
}

// searchExpr asks /demo/search for the results of the query in an "astryx-search" event.
func searchExpr(id string) string {
	return "@get('/demo/search?id=" + id + "&q=' + encodeURIComponent(evt.detail.query))"
}
