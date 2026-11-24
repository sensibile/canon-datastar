package main

import "github.com/sensibile/canon-datastar/go/canon"

func opt(value, label string) canon.SelectorOption {
	return canon.SelectorOption{Value: value, Label: label}
}

// fruitOptions is a flat list with a description, an icon and a disabled option.
func fruitOptions() []canon.SelectorOption {
	return []canon.SelectorOption{
		{Value: "apple", Label: "Apple", Description: "Crisp and sweet"},
		opt("banana", "Banana"),
		{Value: "cherry", Label: "Cherry", Icon: "success"},
		{Value: "durian", Label: "Durian", IsDisabled: true},
		opt("elderberry", "Elderberry"),
		opt("fig", "Fig"),
		opt("grape", "Grape"),
	}
}

// regionOptions groups options into sections with a divider.
func regionOptions() []canon.SelectorOption {
	return []canon.SelectorOption{
		canon.SelectorSection("Americas", opt("us-east", "US East"), opt("us-west", "US West"), opt("sa", "São Paulo")),
		canon.SelectorDivider(),
		canon.SelectorSection("Asia Pacific", opt("seoul", "Seoul"), opt("tokyo", "Tokyo"), opt("sydney", "Sydney")),
		canon.SelectorDivider(),
		opt("global", "Global"),
	}
}
