package canon

import (
	"context"
	"io"
	"strings"

	"github.com/a-h/templ"
)

// Components are written with this small builder rather than .templ files so that the
// output is byte-for-byte what the Rust crate's builder (rust/src/tag.rs) writes:
// attributes in a fixed order, double-quoted, escaped like maud (& < > "), boolean
// attributes as bare names, no whitespace between tags. They are ordinary
// templ.Components, so they compose with templ templates (@canon.Button(...)).

// attrs is an ordered attribute list being built.
type attrs struct{ b strings.Builder }

func newAttrs() *attrs { return &attrs{} }

// set writes name="value".
func (a *attrs) set(name, value string) *attrs {
	checkAttrName(name)
	a.b.WriteByte(' ')
	a.b.WriteString(name)
	a.b.WriteString(`="`)
	escapeTo(&a.b, value)
	a.b.WriteByte('"')
	return a
}

// opt writes name="value" when value is not empty.
func (a *attrs) opt(name, value string) *attrs {
	if value != "" {
		a.set(name, value)
	}
	return a
}

// when writes name="value" when cond holds.
func (a *attrs) when(cond bool, name, value string) *attrs {
	if cond {
		a.set(name, value)
	}
	return a
}

// flag writes a bare boolean attribute when cond holds.
func (a *attrs) flag(name string, cond bool) *attrs {
	if cond {
		checkAttrName(name)
		a.b.WriteByte(' ')
		a.b.WriteString(name)
	}
	return a
}

// extra appends caller-supplied attributes in order.
func (a *attrs) extra(list []Attr) *attrs {
	for _, at := range list {
		a.set(at.Name, at.Value)
	}
	return a
}

func checkAttrName(name string) {
	if name == "" || strings.ContainsAny(name, " \t\n\f\r\"'>/=<") {
		panic("canon: invalid HTML attribute name " + name)
	}
}

// escapeTo escapes text and attribute values the way maud does.
func escapeTo(b *strings.Builder, s string) {
	for _, r := range s {
		switch r {
		case '&':
			b.WriteString("&amp;")
		case '<':
			b.WriteString("&lt;")
		case '>':
			b.WriteString("&gt;")
		case '"':
			b.WriteString("&quot;")
		default:
			b.WriteRune(r)
		}
	}
}

func escape(s string) string {
	var b strings.Builder
	escapeTo(&b, s)
	return b.String()
}

// el renders <tag attrs>children</tag>.
func el(tag string, a *attrs, children ...templ.Component) templ.Component {
	return templ.ComponentFunc(func(ctx context.Context, w io.Writer) error {
		if _, err := io.WriteString(w, "<"+tag+a.b.String()+">"); err != nil {
			return err
		}
		for _, c := range children {
			if c == nil {
				continue
			}
			if err := c.Render(ctx, w); err != nil {
				return err
			}
		}
		_, err := io.WriteString(w, "</"+tag+">")
		return err
	})
}

// void renders <tag attrs> with no end tag (input).
func void(tag string, a *attrs) templ.Component {
	return raw("<" + tag + a.b.String() + ">")
}

// text renders escaped text.
func text(s string) templ.Component { return raw(escape(s)) }

// raw renders trusted markup as is.
func raw(s string) templ.Component {
	return templ.ComponentFunc(func(_ context.Context, w io.Writer) error {
		_, err := io.WriteString(w, s)
		return err
	})
}

// group renders components in order; nil entries are skipped.
func group(children ...templ.Component) templ.Component {
	return templ.ComponentFunc(func(ctx context.Context, w io.Writer) error {
		for _, c := range children {
			if c == nil {
				continue
			}
			if err := c.Render(ctx, w); err != nil {
				return err
			}
		}
		return nil
	})
}

// withChildren builds a component that reads its templ children block.
func withChildren(f func(children templ.Component) templ.Component) templ.Component {
	return templ.ComponentFunc(func(ctx context.Context, w io.Writer) error {
		children := templ.GetChildren(ctx)
		return f(children).Render(templ.ClearChildren(ctx), w)
	})
}

// cls joins a base class with an optional extra class.
func cls(base, extra string) string {
	if extra == "" {
		return base
	}
	return base + " " + extra
}

// styles joins "prop: value" declarations into a style attribute value ("" when none).
func styles(decls ...string) string {
	var parts []string
	for _, d := range decls {
		if d != "" {
			parts = append(parts, d)
		}
	}
	return strings.Join(parts, "; ")
}

func decl(prop, value string) string {
	if value == "" {
		return ""
	}
	return prop + ": " + value
}

func or(v, fallback string) string {
	if v == "" {
		return fallback
	}
	return v
}

// joinIDs joins non-empty ids with spaces (for aria-describedby).
func joinIDs(ids ...string) string {
	var parts []string
	for _, id := range ids {
		if id != "" {
			parts = append(parts, id)
		}
	}
	return strings.Join(parts, " ")
}
