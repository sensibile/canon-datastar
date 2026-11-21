package canon

import (
	"math"
	"strconv"
	"strings"
	"unicode"

	"github.com/a-h/templ"
)

// Port of Astryx `Avatar`, `AvatarStatusDot`, `AvatarGroup` and `AvatarGroupOverflow`
// (packages/core/src/Avatar, AvatarGroup), MIT, Meta Platforms, Inc.
// The initials or person glyph is always rendered (hidden behind an image) and static/canon.js
// swaps to FallbackSrc, then to it, when an image fails to load. AvatarGroup's interactive
// avatars form one tab stop moved with the arrow keys, Home and End (static/canon.js).
// Differences: initials take letters and digits (not emoji), and CSS uppercases them.

// AvatarStatusDotProps configures the status dot in an Avatar's corner.
type AvatarStatusDotProps struct {
	// Variant: "success" (default), "neutral", "error".
	Variant string
	// Label joins the avatar's accessible name.
	Label string
	// Icon is a built-in glyph shown on medium and large avatars instead of the default glyph.
	Icon  string
	Class string
	Attrs []Attr
}

// AvatarProps configures Avatar.
type AvatarProps struct {
	Alt, Name, Src, FallbackSrc string
	// Size: "xsm", "sm", "md" (default), "lg", "xl"; Pixels overrides it with a number.
	Size   string
	Pixels int
	// Shape: "circle" (default), "rounded", "square".
	Shape  string
	Status *AvatarStatusDotProps
	// Tooltip overrides the tooltip text (Name by default); HasNoTooltip turns it off.
	Tooltip      string
	HasNoTooltip bool
	TooltipID    string
	// Href renders a link; Action (any non-nil list, e.g. a data-on:click) a button.
	Href, Target, Rel string
	Action            []Attr
	AriaLabel         string
	Class             string
	Attrs             []Attr
}

var avatarPixels = map[string]int{"xsm": 20, "sm": 24, "md": 36, "lg": 48, "xl": 128}

func avatarSize(size string, pixels int) (string, int) {
	if pixels > 0 {
		return strconv.Itoa(pixels), pixels
	}
	if px, ok := avatarPixels[size]; ok {
		return size, px
	}
	return "md", 36
}

func isInitialChar(r rune) bool {
	return unicode.In(r, unicode.Letter, unicode.Number, unicode.Other_Alphabetic)
}

// avatarInitials takes the first letter or digit of the first and last words.
func avatarInitials(name string) string {
	var initials []string
	for _, word := range strings.Fields(name) {
		for _, r := range word {
			if isInitialChar(r) {
				initials = append(initials, string(r))
				break
			}
		}
	}
	switch len(initials) {
	case 0:
		return ""
	case 1:
		return initials[0]
	}
	return initials[0] + initials[len(initials)-1]
}

// meaningful returns s unless it is blank.
func meaningful(s string) string {
	if strings.TrimSpace(s) == "" {
		return ""
	}
	return s
}

func cssString(s string) string {
	s = strings.NewReplacer(`\`, `\\`, `"`, `\"`, "\n", `\a `).Replace(s)
	return `"` + s + `"`
}

type avatarGroupCtx struct {
	size    string
	shape   string
	overlap int
	px      int
	// tabStop is "0" or "-1" for an interactive avatar in the group.
	tabStop string
}

// Avatar renders a user image, initials or a person glyph.
func Avatar(p AvatarProps) templ.Component { return avatar(p, nil) }

func avatar(p AvatarProps, g *avatarGroupCtx) templ.Component {
	size, px := avatarSize(p.Size, p.Pixels)
	shape := or(p.Shape, "circle")
	if g != nil {
		size, px, shape = g.size, g.px, g.shape
	}
	name := meaningful(p.Name)
	initials := avatarInitials(name)
	nameLabel := or(meaningful(p.Alt), name)
	statusLabel := ""
	if p.Status != nil {
		statusLabel = meaningful(p.Status.Label)
	}
	accessibleName := nameLabel
	if nameLabel != "" && statusLabel != "" {
		accessibleName = nameLabel + ", " + statusLabel
	} else if nameLabel == "" {
		accessibleName = statusLabel
	}
	tip := ""
	if !p.HasNoTooltip {
		tip = strings.TrimSpace(or(p.Tooltip, name))
	}

	fpx := float64(px)
	var image templ.Component
	src, fallbackSrc := p.Src, p.FallbackSrc
	if src == "" {
		src, fallbackSrc = fallbackSrc, ""
	}
	if src != "" {
		image = void("img", newAttrs().set("src", src).set("alt", "").set("class", "astryx-avatar-image").opt("data-fallback-src", fallbackSrc))
	}
	fallback := newAttrs().set("class", "astryx-avatar-fallback").
		flag("data-initials", initials != "").
		flag("data-lone", g == nil).
		set("data-size", size).
		flag("hidden", image != nil)
	var fallbackEl templ.Component
	if initials != "" {
		fallbackEl = el("div", fallback.set("style", "font-size: "+num(fpx*0.4)+"px; --_initials: "+cssString(initials)))
	} else {
		glyph := num(fpx * 0.6)
		fallbackEl = el("div", fallback,
			el("svg", newAttrs().set("width", glyph).set("height", glyph).set("viewBox", "0 0 24 24").set("fill", "currentColor").set("aria-hidden", "true"),
				el("path", newAttrs().set("d", "M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z"))))
	}
	var status templ.Component
	if p.Status != nil {
		sa := newAttrs().set("class", "astryx-avatar-status").flag("data-corner", shape != "circle")
		if shape == "circle" {
			off := num(fpx * ((1 - 1/math.Sqrt(2)) / 2))
			sa.set("style", "bottom: "+off+"px; inset-inline-end: "+off+"px")
		}
		status = el("div", sa, avatarStatusDot(*p.Status, px))
	}
	visual := group(el("div", newAttrs().set("class", "astryx-avatar-content"), image, fallbackEl), status)

	isLink := p.Href != ""
	isButton := !isLink && p.Action != nil
	style := "width: " + strconv.Itoa(px) + "px; height: " + strconv.Itoa(px) + "px"
	if g != nil {
		style += "; --_avatar-group-overlap: " + strconv.Itoa(-g.overlap) + "px"
	}
	a := newAttrs().set("class", cls("astryx-avatar", p.Class)).
		set("data-size", size).
		set("data-shape", shape).
		flag("data-interactive", isLink || isButton).
		flag("data-in-group", g != nil)
	var root templ.Component
	switch {
	case isLink || isButton:
		if isLink {
			a.set("href", p.Href).opt("target", p.Target).opt("rel", p.Rel)
		} else {
			a.set("type", "button")
		}
		a.opt("aria-label", or(p.AriaLabel, accessibleName)).set("data-avatar-item", "")
		if g != nil {
			a.set("tabindex", g.tabStop)
		}
		a.set("style", style).extra(p.Action).extra(p.Attrs)
		root = el(map[bool]string{true: "a", false: "button"}[isLink], a, visual)
	default:
		role := "presentation"
		if accessibleName != "" {
			role = "img"
		}
		label := or(p.AriaLabel, accessibleName)
		a.set("role", role).opt("aria-label", label).when(label == "", "aria-hidden", "true").
			when(tip != "" && g == nil, "tabindex", "0").
			set("style", style).extra(p.Attrs)
		root = el("div", a, visual)
	}
	if tip == "" {
		return root
	}
	return tooltip(TooltipProps{ID: p.TooltipID, Content: tip}, root)
}

func avatarStatusDot(p AvatarStatusDotProps, avatarPx int) templ.Component {
	dot, border, iconPx, stroke := 10, 1, 0, 1.0
	switch {
	case avatarPx <= 36:
	case avatarPx <= 72:
		dot, border, iconPx, stroke = 20, 2, 12, 1.5
	default:
		dot, border, iconPx, stroke = 32, 4, 18, 2
	}
	variant := or(p.Variant, "success")
	showsIcon := p.Icon != "" && iconPx > 0
	label := meaningful(p.Label)
	a := newAttrs().when(label != "", "role", "img").
		opt("aria-label", label).
		set("class", cls("astryx-avatar-status-dot", p.Class)).
		set("data-variant", variant).
		set("style", "width: "+strconv.Itoa(dot)+"px; height: "+strconv.Itoa(dot)+"px; border-width: "+strconv.Itoa(border)+"px").
		extra(p.Attrs)
	if showsIcon {
		ip := strconv.Itoa(iconPx)
		return el("div", a, el("span", newAttrs().set("class", "astryx-avatar-status-dot-icon").set("aria-hidden", "true").
			set("style", "width: "+ip+"px; height: "+ip+"px"), Icon(IconProps{Name: p.Icon, Size: "xsm"})))
	}
	if variant != "neutral" && variant != "error" {
		return el("div", a)
	}
	field := float64(dot - border*2)
	f := num(field)
	svg := newAttrs().set("aria-hidden", "true").set("viewBox", "0 0 "+f+" "+f).set("width", f).set("height", f).
		set("fill", "none").set("class", "astryx-avatar-status-dot-glyph")
	var shape templ.Component
	if variant == "neutral" {
		svg.set("data-shape", "ring")
		shape = el("circle", newAttrs().set("cx", num(field/2)).set("cy", num(field/2)).set("r", num((field-stroke)/2)).
			set("stroke", "currentColor").set("stroke-width", num(stroke)))
	} else {
		const span = 0.75
		svg.set("data-shape", "minus")
		shape = el("line", newAttrs().set("x1", num(field*(1-span)/2+stroke/2)).set("y1", num(field/2)).
			set("x2", num(field*(1+span)/2-stroke/2)).set("y2", num(field/2)).
			set("stroke", "currentColor").set("stroke-width", num(stroke)).set("stroke-linecap", "round"))
	}
	return el("div", a, el("svg", svg, shape))
}

// AvatarGroupOverflowProps configures the "+N" chip at the end of an AvatarGroup.
type AvatarGroupOverflowProps struct {
	Count int
	// Action (any non-nil list) makes it a button, e.g. {"data-on:click", "$showAll = true"}.
	Action []Attr
	// Content replaces the "+N" text.
	Content   templ.Component
	AriaLabel string
	Class     string
	Attrs     []Attr
}

// groupThousands formats n with comma separators, like en-US toLocaleString.
func groupThousands(n int) string {
	s := strconv.Itoa(n)
	for i := len(s) - 3; i > 0; i -= 3 {
		s = s[:i] + "," + s[i:]
	}
	return s
}

// AvatarGroupProps configures AvatarGroup.
type AvatarGroupProps struct {
	// ID prefixes the keyboard hint's id ("avatar-group" by default).
	ID string
	// Size and Shape apply to every avatar.
	Size     string
	Pixels   int
	Shape    string
	Avatars  []AvatarProps
	Overflow *AvatarGroupOverflowProps
	// AriaLabel names the group ("Avatars" by default).
	AriaLabel string
	Class     string
	Attrs     []Attr
}

// AvatarGroup renders an overlapping row of avatars that share one size and shape.
func AvatarGroup(p AvatarGroupProps) templ.Component {
	size, px := avatarSize(p.Size, p.Pixels)
	shape := or(p.Shape, "circle")
	overlap := int(math.Round(float64(px) * 0.25))
	hintID := or(p.ID, "avatar-group") + "-hint"
	ctx := &avatarGroupCtx{size: size, shape: shape, overlap: overlap, px: px}

	var children []templ.Component
	items := 0
	for _, av := range p.Avatars {
		if av.Href != "" || av.Action != nil {
			ctx.tabStop = map[bool]string{true: "0", false: "-1"}[items == 0]
			items++
		}
		c := *ctx
		children = append(children, avatar(av, &c))
	}
	if o := p.Overflow; o != nil {
		count := o.Count
		if count < 0 {
			count = 0
		}
		content := o.Content
		if content == nil {
			content = text("+" + groupThousands(count))
		}
		style := "min-width: " + strconv.Itoa(px+4) + "px; height: " + strconv.Itoa(px+4) +
			"px; font-size: max(var(--text-supporting-size), " + num(float64(px)*0.35) + "px); --_avatar-group-overlap: " +
			strconv.Itoa(-overlap) + "px"
		a := newAttrs()
		tag := "span"
		if o.Action != nil {
			tag = "button"
			a.set("type", "button").set("aria-label", or(o.AriaLabel, groupThousands(count)+" more")).
				set("data-avatar-item", "").set("tabindex", map[bool]string{true: "0", false: "-1"}[items == 0])
			items++
		} else {
			a.set("role", "img").set("aria-label", or(o.AriaLabel, groupThousands(count)+" more"))
		}
		a.set("class", cls("astryx-avatar-group-overflow", o.Class)).set("data-size", size).set("data-shape", shape).
			set("style", style).extra(o.Action).extra(o.Attrs)
		children = append(children, el(tag, a, content))
	}
	if items > 0 {
		children = append(children, visuallyHidden(VisuallyHiddenProps{ID: hintID}, text("Use arrow keys to move between avatars")))
	}
	a := newAttrs().set("role", "group").
		set("aria-label", or(p.AriaLabel, "Avatars")).
		when(items > 0, "aria-describedby", hintID).
		set("class", cls("astryx-avatar-group", p.Class)).
		set("data-size", size).
		set("data-shape", shape).
		set("style", "--_avatar-group-overlap: "+strconv.Itoa(-overlap)+"px").
		extra(p.Attrs)
	return el("div", a, children...)
}
