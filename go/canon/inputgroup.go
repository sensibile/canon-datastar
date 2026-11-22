package canon

import "github.com/a-h/templ"

// InputGroupItem marks an element (a Button, say) as a joined member of an InputGroup row:
// ButtonProps{Attrs: []Attr{canon.InputGroupItem}}.
var InputGroupItem = Attr{"data-astryx-input-group-item", ""}

// InputGroupMember tells a control it sits inside an InputGroup: it then renders only its
// control, named by the group label plus its own. Get one from InputGroupProps.Member.
type InputGroupMember struct {
	LabelID     string
	DescribedBy string
	IsDisabled  bool
	Size        string
}

// InputGroupProps configures InputGroup.
type InputGroupProps struct {
	// ID of the group; defaults to "astryx-" + a slug of Label.
	ID            string
	Label         string
	IsLabelHidden bool
	Description   string
	IsDisabled    bool
	IsOptional    bool
	IsRequired    bool
	// Size: "sm", "md" (default), "lg"; members default to it.
	Size   string
	Status *FieldStatus
	Class  string
	Attrs  []Attr
}

func (p InputGroupProps) ids() (id, labelID, descID, statusID string) {
	id = controlID(p.ID, p.Label)
	labelID = id + "-label"
	if p.Description != "" {
		descID = id + "-desc"
	}
	if p.Status != nil && p.Status.Message != "" {
		statusID = id + "-status"
	}
	return
}

// Member returns what to pass as a member control's Group prop.
func (p InputGroupProps) Member() *InputGroupMember {
	_, labelID, descID, statusID := p.ids()
	return &InputGroupMember{LabelID: labelID, DescribedBy: joinIDs(descID, statusID),
		IsDisabled: p.IsDisabled, Size: or(p.Size, "md")}
}

// InputGroup joins inputs, buttons and InputGroupText add-ons (its children) into one row
// under one label. Give member controls Group: p.Member().
// Port of Astryx `InputGroup` (packages/core/src/InputGroup), MIT, Meta Platforms, Inc.
func InputGroup(p InputGroupProps) templ.Component {
	return withChildren(func(children templ.Component) templ.Component {
		return inputGroup(p, children)
	})
}

func inputGroup(p InputGroupProps, children templ.Component) templ.Component {
	id, labelID, descID, statusID := p.ids()
	statusType := ""
	var status *FieldStatus
	if p.Status != nil {
		statusType = p.Status.Type
		status = &FieldStatus{Type: p.Status.Type, Message: p.Status.Message, MessageID: statusID}
	}
	row := el("div", newAttrs().set("id", id).set("role", "group").
		set("class", cls("astryx-input-group", p.Class)).
		set("data-size", or(p.Size, "md")).
		opt("data-status", statusType).
		flag("data-disabled", p.IsDisabled).
		when(p.IsDisabled, "aria-disabled", "true").
		set("aria-labelledby", labelID).
		opt("aria-describedby", joinIDs(descID, statusID)).
		extra(p.Attrs), children)
	return field(FieldProps{Label: p.Label, IsLabelHidden: p.IsLabelHidden, Description: p.Description,
		InputID: id, LabelID: labelID, IsGroupLabel: true, DescriptionID: descID,
		IsOptional: p.IsOptional, IsRequired: p.IsRequired, IsDisabled: p.IsDisabled,
		Status: status, StatusVariant: "detached"}, row)
}

// InputGroupText renders a static add-on inside an InputGroup, such as a unit or URL prefix.
// Port of Astryx `InputGroupText`, MIT, Meta Platforms, Inc.
func InputGroupText(label string) templ.Component {
	return el("div", newAttrs().set("data-astryx-input-group-item", "").set("class", "astryx-input-group-text"),
		el("span", newAttrs().set("dir", "auto"), text(label)))
}
