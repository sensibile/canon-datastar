package canon

import "strings"

// Shared prop vocab, mirroring Astryx (packages/core/src/utils/types.ts). Enum-like props are
// plain strings; the empty string means "use the default".

// Space is a spacing step: "0", "0.5", "1", "1.5", "2", "3", "4", "5", "6", "8", "10".
// It maps to var(--spacing-N). Empty means unset.
type Space string

func (s Space) css() string {
	if s == "" {
		return ""
	}
	return "var(--spacing-" + strings.ReplaceAll(string(s), ".", "-") + ")"
}

// FieldStatus is a validation message for a form control.
type FieldStatus struct {
	// Type is "warning", "error" or "success".
	Type    string
	Message string
	// MessageID overrides the message element's id.
	MessageID string
}

// slug derives a default element id from a label: ASCII letters and digits, others become "-".
func slug(s string) string {
	var b strings.Builder
	dash := false
	for _, r := range strings.ToLower(s) {
		if (r >= 'a' && r <= 'z') || (r >= '0' && r <= '9') {
			b.WriteRune(r)
			dash = false
		} else if !dash && b.Len() > 0 {
			b.WriteByte('-')
			dash = true
		}
	}
	return strings.TrimSuffix(b.String(), "-")
}

// controlID picks a form control's id: the explicit one, else "astryx-" + slug(label).
func controlID(id, label string) string {
	if id != "" {
		return id
	}
	if s := slug(label); s != "" {
		return "astryx-" + s
	}
	return "astryx-input"
}
