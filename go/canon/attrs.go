package canon

import "github.com/a-h/templ"

// attrs converts ordered attributes for templ's spread syntax.
func attrs(list []Attr) templ.OrderedAttributes {
	out := make(templ.OrderedAttributes, 0, len(list))
	for _, a := range list {
		out = append(out, templ.KeyValue[string, any]{Key: a.Name, Value: a.Value})
	}
	return out
}
