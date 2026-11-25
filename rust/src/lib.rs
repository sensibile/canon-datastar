//! Astryx components as server-rendered HTML ([maud] `Markup`) for Datastar apps.
//!
//! Markup (class names, `data-*` attributes) matches the Go `canon` package and the golden
//! files in `fixtures/`, and is styled by the shared `css/` stylesheets.
//! Ported from Astryx (<https://github.com/meta/astryx>), MIT, Meta Platforms, Inc.
//!
//! Props structs implement `Default`; enum-like props are strings where "" means the default.
//! Every component takes `attrs` (extra attributes, e.g. Datastar bindings), rendered in order.

mod avatar;
mod banner;
mod button;
mod calendar;
mod collapsible;
mod container;
mod content;
mod dialog;
mod field;
mod forms;
mod html;
mod icon;
mod icons_gen;
mod input_group;
mod inputs;
mod item;
mod link;
mod menu;
mod metadata;
mod misc;
mod multiselector;
mod popover;
mod progress;
mod selector;
mod tabs;
mod text;
mod theme;
mod thumbnail;
mod timestamp;
mod toast;
mod token;
mod tooltip;
mod typeahead;
mod types;

pub use avatar::{
    AvatarGroupOverflowProps, AvatarGroupProps, AvatarProps, AvatarStatusDotProps, avatar,
    avatar_group,
};
pub use banner::{BannerProps, banner};
pub use button::{ButtonProps, button, icon_button};
pub use calendar::{CalendarProps, calendar};
pub use collapsible::{CollapsibleGroupProps, CollapsibleProps, collapsible, collapsible_group};
pub use container::{
    AspectRatioProps, CardProps, CenterProps, GridAuto, GridProps, GridSpanProps, SectionProps,
    aspect_ratio, card, center, grid, grid_span, section,
};
pub use content::{
    BlockquoteProps, CodeProps, EmptyStateProps, KbdProps, SkeletonProps, StatusDotProps,
    blockquote, code, empty_state, kbd, skeleton, status_dot,
};
pub use dialog::{DialogHeaderProps, DialogProps, dialog, dialog_header};
pub use field::{FieldLabelProps, FieldProps, field, field_label, field_status};
pub use forms::{
    CheckboxListItem, CheckboxListProps, FileInputProps, NumberInputProps, RadioListItem,
    RadioListProps, SliderMark, SliderProps, TextAreaProps, checkbox_list, file_input,
    number_input, radio_list, slider, text_area,
};
pub use icon::{IconProps, icon, icon_names};
pub use input_group::{
    InputGroupMember, InputGroupProps, input_group, input_group_item, input_group_text,
};
pub use inputs::{
    CheckboxInputProps, SwitchProps, TextInputProps, checkbox_input, switch, text_input,
};
pub use item::{ItemProps, item};
pub use link::{LinkProps, link};
pub use menu::{
    DropdownMenuGroupProps, DropdownMenuItemProps, DropdownMenuProps, dropdown_menu,
    dropdown_menu_divider, dropdown_menu_group, dropdown_menu_item,
};
pub use metadata::{MetadataListItem, MetadataListProps, metadata_list};
pub use misc::{
    BadgeProps, DividerProps, SpinnerProps, StackProps, VisuallyHiddenProps, badge, divider,
    hstack, spinner, stack, visually_hidden, vstack,
};
pub use multiselector::{MultiSelectorProps, multi_selector};
pub use popover::{PopoverProps, popover};
pub use progress::{ProgressBarMark, ProgressBarProps, progress_bar};
pub use selector::{SelectorOption, SelectorProps, selector, selector_divider, selector_section};
pub use tabs::{TabListProps, TabPanelProps, TabProps, tab_list, tab_panel};
pub use text::{HeadingProps, TextProps, heading, text};
pub use theme::{THEMES, ThemeMode, ThemeProps, theme};
pub use thumbnail::{ThumbnailProps, thumbnail};
pub use timestamp::{TimestampProps, TimestampTooltipEntry, timestamp};
pub use toast::{ToastProps, ToastViewportProps, toast, toast_viewport};
pub use token::{TokenProps, token};
pub use tooltip::{InfoTipProps, TooltipProps, info_tip, tooltip};
pub use typeahead::{
    TokenizerProps, TypeaheadItem, TypeaheadProps, TypeaheadResultsProps, tokenizer, typeahead,
    typeahead_results,
};
pub use types::FieldStatus;

/// One HTML attribute, e.g. a Datastar binding `("data-bind:email".into(), "".into())`.
/// A list keeps its order so Go and Rust render identical markup.
pub type Attr = (String, String);

/// Builds an [`Attr`] from string slices.
pub fn attr(name: &str, value: &str) -> Attr {
    (name.to_string(), value.to_string())
}
