//! Compile-time checks for the component rules that Discord documents.
//!
//! The macro sees only tokens, so value checks run on literals. A value that is
//! computed at runtime is skipped. Structure (nesting, child counts, required
//! fields, field combinations) is always known and always checked.
//!
//! The message or modal context is inferred from the components in one
//! invocation: some components are message-only and some are modal-only.

use std::collections::HashSet;
use std::fmt::Display;

use quote::ToTokens;
use syn::{Expr, Lit, Token, punctuated::Punctuated};

use crate::{ComponentKind, ComponentNode, Property, PropertyValue};

const MESSAGE_COMPONENT_MAX: usize = 40;
const MODAL_COMPONENT_MAX: usize = 5;
const CUSTOM_ID_MAX: usize = 100;
const ACTION_ROW_BUTTON_MAX: usize = 5;
const SECTION_COMPONENT_MAX: usize = 3;
const MEDIA_GALLERY_ITEM_MAX: usize = 10;
const MEDIA_DESCRIPTION_MAX: usize = 1024;
const BUTTON_LABEL_MAX: usize = 80;
const BUTTON_URL_MAX: usize = 512;
const SELECT_OPTION_MAX: usize = 25;
const SELECT_VALUES_MAX: u128 = 25;
const SELECT_PLACEHOLDER_MAX: usize = 150;
const SELECT_OPTION_TEXT_MAX: usize = 100;
const TEXT_INPUT_LENGTH_MAX: u128 = 4000;
const TEXT_INPUT_PLACEHOLDER_MAX: usize = 100;
const FILE_UPLOAD_VALUES_MAX: u128 = 10;
const LABEL_LABEL_MAX: usize = 45;
const LABEL_DESCRIPTION_MAX: usize = 100;
const ACCENT_COLOR_MAX: u128 = 0xFF_FFFF;

const BUTTON_STYLES: &[&str] = &[
    "Primary",
    "Secondary",
    "Success",
    "Danger",
    "Link",
    "Premium",
];
const SELECT_KINDS: &[&str] = &["Text", "String", "User", "Role", "Mentionable", "Channel"];
const TEXT_INPUT_STYLES: &[&str] = &["Short", "Paragraph"];
const SEPARATOR_SPACINGS: &[&str] = &["Small", "Large"];

/// Which macro is being validated.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Root {
    /// `component!`: the result may be a fragment that is nested elsewhere.
    Single,
    /// `components!`: the result is the full list of top-level components.
    List,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum Context {
    #[default]
    Unknown,
    Message,
    Modal,
}

/// Validates the nodes of one macro invocation and returns every error found.
pub(crate) fn validate(nodes: &[&ComponentNode], root: Root) -> syn::Result<()> {
    let mut validator = Validator::default();
    validator.context = validator.infer_context(nodes);

    for node in nodes {
        validator.node(node);
    }

    // When the context is mixed, root errors would only repeat that error.
    if root == Root::List && !validator.mixed {
        validator.roots(nodes);
    }

    match validator.error {
        Some(err) => Err(err),
        None => Ok(()),
    }
}

#[derive(Default)]
struct Validator {
    error: Option<syn::Error>,
    context: Context,
    mixed: bool,
    custom_ids: HashSet<String>,
    ids: HashSet<u128>,
}

impl Validator {
    fn error(&mut self, tokens: impl ToTokens, message: impl Display) {
        let err = syn::Error::new_spanned(tokens, message);
        match &mut self.error {
            Some(existing) => existing.combine(err),
            None => self.error = Some(err),
        }
    }

    // -------------------------------------------------------------------------
    // Context
    // -------------------------------------------------------------------------

    fn infer_context(&mut self, nodes: &[&ComponentNode]) -> Context {
        let mut first_message: Option<&ComponentNode> = None;
        let mut first_modal: Option<&ComponentNode> = None;
        let mut conflict: Option<(&ComponentNode, &ComponentNode)> = None;

        for node in nodes {
            walk(node, &mut |n| {
                let (own, other) = match context_of(n.kind) {
                    Context::Message => (&mut first_message, first_modal),
                    Context::Modal => (&mut first_modal, first_message),
                    Context::Unknown => return,
                };
                if own.is_none() {
                    *own = Some(n);
                    if let (Some(other), None) = (other, conflict) {
                        conflict = Some((n, other));
                    }
                }
            });
        }

        if let Some((node, other)) = conflict {
            let (node_ctx, other_ctx) = match context_of(node.kind) {
                Context::Modal => ("modals", "messages"),
                _ => ("messages", "modals"),
            };
            self.error(
                &node.ident,
                format!(
                    "`{}` can only be used in {}, but this invocation also contains `{}`, which can only be used in {}",
                    node.kind.name(),
                    node_ctx,
                    other.kind.name(),
                    other_ctx,
                ),
            );
            self.mixed = true;
            return Context::Unknown;
        }

        match (first_message, first_modal) {
            (Some(_), _) => Context::Message,
            (_, Some(_)) => Context::Modal,
            _ => Context::Unknown,
        }
    }

    fn roots(&mut self, nodes: &[&ComponentNode]) {
        for node in nodes {
            let allowed = match self.context {
                Context::Message => matches!(
                    node.kind,
                    ComponentKind::ActionRow
                        | ComponentKind::Text
                        | ComponentKind::Section
                        | ComponentKind::MediaGallery
                        | ComponentKind::File
                        | ComponentKind::Separator
                        | ComponentKind::Container
                ),
                Context::Modal => matches!(node.kind, ComponentKind::Label | ComponentKind::Text),
                Context::Unknown => node.kind == ComponentKind::Text,
            };
            if !allowed {
                let hint = match node.kind {
                    ComponentKind::Button => "; put it in an ActionRow or a Section accessory",
                    ComponentKind::SelectMenu => {
                        "; put it in an ActionRow (messages) or a Label (modals)"
                    }
                    ComponentKind::TextInput | ComponentKind::FileUpload => "; put it in a Label",
                    ComponentKind::Thumbnail => "; use it as a Section accessory",
                    ComponentKind::SelectOption => "; put it in a SelectMenu",
                    ComponentKind::MediaItem => "; put it in a MediaGallery",
                    _ => "",
                };
                let place = match self.context {
                    Context::Modal => " in a modal",
                    Context::Message => " in a message",
                    Context::Unknown => "",
                };
                self.error(
                    &node.ident,
                    format!(
                        "`{}` cannot be a top-level component{}{}",
                        node.kind.name(),
                        place,
                        hint
                    ),
                );
            }
        }

        match self.context {
            Context::Modal if nodes.len() > MODAL_COMPONENT_MAX => {
                self.error(
                    &nodes[MODAL_COMPONENT_MAX].ident,
                    format!("a modal can have at most {MODAL_COMPONENT_MAX} top-level components"),
                );
            }
            Context::Modal => {}
            _ => {
                let mut count = 0;
                let mut overflow = None;
                for node in nodes {
                    walk(node, &mut |n| {
                        if !matches!(
                            n.kind,
                            ComponentKind::SelectOption | ComponentKind::MediaItem
                        ) {
                            count += 1;
                        }
                        // Components passed as expressions count too.
                        count += expression_components(n);
                        if count > MESSAGE_COMPONENT_MAX && overflow.is_none() {
                            overflow = Some(n);
                        }
                    });
                }
                if let Some(node) = overflow {
                    self.error(
                        &node.ident,
                        format!(
                            "a message can have at most {MESSAGE_COMPONENT_MAX} components in total, including nested ones; this invocation has {count}"
                        ),
                    );
                }
            }
        }
    }

    // -------------------------------------------------------------------------
    // Nodes
    // -------------------------------------------------------------------------

    fn node(&mut self, node: &ComponentNode) {
        self.shape(node);
        self.unique(node);

        match node.kind {
            ComponentKind::Text => self.text(node),
            ComponentKind::Container => self.container(node),
            ComponentKind::Section => self.section(node),
            ComponentKind::Separator => self.separator(node),
            ComponentKind::MediaGallery => self.media_gallery(node),
            ComponentKind::Thumbnail => self.media(node, MEDIA_DESCRIPTION_MAX),
            ComponentKind::MediaItem => self.media(node, MEDIA_DESCRIPTION_MAX),
            ComponentKind::File => self.file(node),
            ComponentKind::FileUpload => self.file_upload(node),
            ComponentKind::Label => self.label(node),
            ComponentKind::ActionRow => self.action_row(node),
            ComponentKind::Button => self.button(node),
            ComponentKind::SelectMenu => self.select_menu(node),
            ComponentKind::TextInput => self.text_input(node),
            ComponentKind::SelectOption => self.select_option(node),
        }

        for child in children(node) {
            self.node(child);
        }
        for nested in property_components(node) {
            self.node(nested);
        }
    }

    /// Checks property keys, shorthand and children against what the kind accepts.
    fn shape(&mut self, node: &ComponentNode) {
        let kind = node.kind;
        let fields = fields(kind);
        let shorthand = shorthand_field(kind);

        if let Some(props) = &node.properties {
            let mut seen: Vec<&str> = Vec::new();
            for prop in &props.properties {
                let key = prop.key.to_string();
                let Some(field) = fields
                    .iter()
                    .find(|f| f.0 == key || f.1 == Some(key.as_str()))
                else {
                    let names: Vec<String> = fields
                        .iter()
                        .flat_map(|f| std::iter::once(f.0).chain(f.1))
                        .map(|n| format!("`{n}`"))
                        .collect();
                    self.error(
                        &prop.key,
                        format!(
                            "unknown field `{key}` for `{}`; expected one of {}",
                            kind.name(),
                            names.join(", ")
                        ),
                    );
                    continue;
                };
                if seen.contains(&field.0) {
                    self.error(
                        &prop.key,
                        format!("field `{}` is given more than once", field.0),
                    );
                    continue;
                }
                seen.push(field.0);
                if node.shorthand.is_some() && shorthand == Some(field.0) {
                    self.error(
                        &prop.key,
                        format!("field `{}` is already given in parentheses", field.0),
                    );
                }
            }
        }

        if let (Some(content), None) = (&node.shorthand, shorthand) {
            self.error(
                &content.expr,
                format!("`{}` does not take a value in parentheses", kind.name()),
            );
        }

        if let (Some(block), false) = (&node.children, takes_children(kind)) {
            let target = block
                .children
                .first()
                .map(|c| c.ident.to_token_stream())
                .unwrap_or_else(|| node.ident.to_token_stream());
            self.error(
                target,
                format!("`{}` does not take child components", kind.name()),
            );
        }
    }

    fn unique(&mut self, node: &ComponentNode) {
        if let Some(expr) = prop_expr(node, "id") {
            let expr = some_inner(expr).unwrap_or(expr);
            if let Some(id) = lit_int(expr)
                && !self.ids.insert(id)
            {
                self.error(expr, format!("`id` {id} is used more than once"));
            }
        }

        if let Some(expr) = prop_expr(node, "custom_id")
            && let Some(custom_id) = lit_str(expr)
            && !self.custom_ids.insert(custom_id.clone())
        {
            self.error(
                expr,
                format!("`custom_id` \"{custom_id}\" is used more than once"),
            );
        }
    }

    fn text(&mut self, node: &ComponentNode) {
        self.require(node, "content", true);
    }

    fn container(&mut self, node: &ComponentNode) {
        if let Some(expr) = prop_expr(node, "accent_color")
            && let Some(color) = lit_int(expr)
            && color > ACCENT_COLOR_MAX
        {
            self.error(
                expr,
                "`accent_color` must be an RGB color from 0x000000 to 0xFFFFFF",
            );
        }

        for child in children(node) {
            if !matches!(
                child.kind,
                ComponentKind::ActionRow
                    | ComponentKind::Text
                    | ComponentKind::Section
                    | ComponentKind::MediaGallery
                    | ComponentKind::Separator
                    | ComponentKind::File
            ) {
                self.error(
                    &child.ident,
                    format!(
                        "`Container` cannot contain `{}`; expected ActionRow, Text, Section, MediaGallery, Separator or File",
                        child.kind.name()
                    ),
                );
            }
        }
    }

    fn section(&mut self, node: &ComponentNode) {
        let kids = children(node);
        if kids.is_empty() {
            self.error(&node.ident, "`Section` needs 1 to 3 `Text` children");
        }
        for (i, child) in kids.iter().enumerate() {
            if child.kind != ComponentKind::Text {
                self.error(
                    &child.ident,
                    format!(
                        "`Section` can only contain `Text`, not `{}`",
                        child.kind.name()
                    ),
                );
            } else if i == SECTION_COMPONENT_MAX {
                self.error(
                    &child.ident,
                    format!("`Section` can have at most {SECTION_COMPONENT_MAX} children"),
                );
            }
        }

        match prop(node, "accessory").map(|p| &p.value) {
            None => self.error(
                &node.ident,
                "`Section` requires an `accessory` (a Button or Thumbnail)",
            ),
            Some(PropertyValue::Component(accessory))
                if !matches!(
                    accessory.kind,
                    ComponentKind::Button | ComponentKind::Thumbnail
                ) =>
            {
                self.error(
                    &accessory.ident,
                    format!(
                        "a Section `accessory` must be a Button or Thumbnail, not `{}`",
                        accessory.kind.name()
                    ),
                );
            }
            Some(_) => {}
        }
    }

    fn separator(&mut self, node: &ComponentNode) {
        if let Some(expr) = prop_expr(node, "spacing") {
            self.choice(expr, SEPARATOR_SPACINGS, "separator spacing");
        }
    }

    fn media_gallery(&mut self, node: &ComponentNode) {
        let kids = children(node);
        if kids.is_empty() {
            self.error(
                &node.ident,
                "`MediaGallery` needs 1 to 10 `MediaItem` children",
            );
        }
        for (i, child) in kids.iter().enumerate() {
            if child.kind != ComponentKind::MediaItem {
                self.error(
                    &child.ident,
                    format!(
                        "`MediaGallery` can only contain `MediaItem`, not `{}`",
                        child.kind.name()
                    ),
                );
            } else if i == MEDIA_GALLERY_ITEM_MAX {
                self.error(
                    &child.ident,
                    format!("`MediaGallery` can have at most {MEDIA_GALLERY_ITEM_MAX} items"),
                );
            }
        }
    }

    /// Thumbnail and MediaItem.
    fn media(&mut self, node: &ComponentNode, description_max: usize) {
        if let Some(url) = self.require(node, "url", true)
            && lit_str(url).is_some_and(|u| u.is_empty())
        {
            self.error(url, "`url` cannot be empty");
        }
        self.max_len(node, "description", description_max);
    }

    fn file(&mut self, node: &ComponentNode) {
        if let Some(url) = self.require(node, "url", true)
            && lit_str(url).is_some_and(|u| !u.starts_with("attachment://"))
        {
            self.error(
                url,
                "`File` only supports attachment references; the url must start with `attachment://`",
            );
        }
    }

    fn file_upload(&mut self, node: &ComponentNode) {
        self.custom_id(node, true);
        let min = self.int_range(node, "min_values", 0, FILE_UPLOAD_VALUES_MAX);
        let max = self.int_range(node, "max_values", 0, FILE_UPLOAD_VALUES_MAX);
        self.min_le_max(node, "min_values", min, "max_values", max);
        // File Upload is modal-only, so the `required` rule always applies.
        self.modal_min_values(node);
    }

    fn label(&mut self, node: &ComponentNode) {
        if let Some(label) = self.require(node, "label", true)
            && lit_str(label).is_some_and(|l| l.is_empty())
        {
            self.error(label, "`label` cannot be empty");
        }
        self.max_len_with_shorthand(node, "label", LABEL_LABEL_MAX);
        self.max_len(node, "description", LABEL_DESCRIPTION_MAX);

        let kids = children(node);
        let from_prop = prop(node, "component");
        match (kids.len(), from_prop) {
            (0, None) => self.error(
                &node.ident,
                "`Label` requires one child component (TextInput, SelectMenu or FileUpload)",
            ),
            (0, Some(_)) | (1, None) => {}
            (_, Some(p)) if !kids.is_empty() => self.error(
                &p.key,
                "`Label` takes one child component, either in brackets or as `component`, not both",
            ),
            _ => self.error(
                &kids[1].ident,
                "`Label` can only contain one child component",
            ),
        }

        let child = kids.first().copied().or(match from_prop.map(|p| &p.value) {
            Some(PropertyValue::Component(c)) => Some(c),
            _ => None,
        });
        if let Some(child) = child
            && !matches!(
                child.kind,
                ComponentKind::TextInput | ComponentKind::SelectMenu | ComponentKind::FileUpload
            )
        {
            self.error(
                &child.ident,
                format!(
                    "`Label` cannot contain `{}`; expected TextInput, SelectMenu or FileUpload",
                    child.kind.name()
                ),
            );
        }
    }

    fn action_row(&mut self, node: &ComponentNode) {
        let kids = children(node);
        if kids.is_empty() {
            self.error(
                &node.ident,
                "`ActionRow` needs 1 to 5 `Button` children or a single `SelectMenu`",
            );
            return;
        }

        let has_select = kids.iter().any(|c| c.kind == ComponentKind::SelectMenu);
        for (i, child) in kids.iter().enumerate() {
            match child.kind {
                ComponentKind::Button if has_select => self.error(
                    &child.ident,
                    "an `ActionRow` with a `SelectMenu` cannot also contain buttons",
                ),
                ComponentKind::Button if i == ACTION_ROW_BUTTON_MAX => self.error(
                    &child.ident,
                    format!("an `ActionRow` can have at most {ACTION_ROW_BUTTON_MAX} buttons"),
                ),
                ComponentKind::Button => {}
                ComponentKind::SelectMenu if kids.len() > 1 => self.error(
                    &child.ident,
                    "an `ActionRow` with a `SelectMenu` can only contain that one select menu",
                ),
                ComponentKind::SelectMenu => {}
                ComponentKind::TextInput | ComponentKind::FileUpload => self.error(
                    &child.ident,
                    format!(
                        "`{}` cannot be in an `ActionRow`; put it in a `Label`",
                        child.kind.name()
                    ),
                ),
                other => self.error(
                    &child.ident,
                    format!(
                        "`ActionRow` cannot contain `{}`; expected Button or SelectMenu",
                        other.name()
                    ),
                ),
            }
        }
    }

    fn button(&mut self, node: &ComponentNode) {
        self.custom_id(node, false);
        self.max_len_with_shorthand(node, "label", BUTTON_LABEL_MAX);
        self.max_len(node, "url", BUTTON_URL_MAX);

        let style = match prop_expr(node, "style") {
            Some(expr) => self.choice(expr, BUTTON_STYLES, "button style"),
            None => Some("Primary"),
        };

        // `custom_id`, `label` and `url` are wrapped in `Some` by codegen, so
        // having the key means the field is set.
        let custom_id = prop(node, "custom_id").map(|p| (&p.key, Setness::Yes));
        let url = prop(node, "url").map(|p| (&p.key, Setness::Yes));
        let label = match (&node.shorthand, prop(node, "label")) {
            (Some(s), _) => Some((s.expr.to_token_stream(), Setness::Yes)),
            (None, Some(p)) => Some((p.key.to_token_stream(), Setness::Yes)),
            _ => None,
        };
        let emoji = prop(node, "emoji").map(|p| (&p.key, setness(&p.value)));
        let sku_id = prop(node, "sku_id").map(|p| (&p.key, setness(&p.value)));

        let is_set = |f: &Option<(_, Setness)>| matches!(f, Some((_, Setness::Yes)));
        let may_be_set = |f: &Option<(_, Setness)>| matches!(f, Some((_, s)) if *s != Setness::No);

        match style {
            Some("Link") => {
                if url.is_none() {
                    self.error(&node.ident, "`Link` buttons must have a `url`");
                }
                self.forbid(&custom_id, "`Link` buttons cannot have a `custom_id`");
                if is_set(&sku_id) {
                    self.forbid(&sku_id, "only `Premium` buttons can have a `sku_id`");
                }
            }
            Some("Premium") => {
                if !may_be_set(&sku_id) {
                    self.error(&node.ident, "`Premium` buttons must have a `sku_id`");
                }
                self.forbid(&custom_id, "`Premium` buttons cannot have a `custom_id`");
                self.forbid(&url, "`Premium` buttons cannot have a `url`");
                if let Some((tokens, _)) = &label {
                    self.error(tokens, "`Premium` buttons cannot have a `label`");
                }
                if is_set(&emoji) {
                    self.forbid(&emoji, "`Premium` buttons cannot have an `emoji`");
                }
            }
            Some(style) => {
                if custom_id.is_none() {
                    self.error(
                        &node.ident,
                        format!("`{style}` buttons must have a `custom_id`"),
                    );
                }
                self.forbid(&url, "only `Link` buttons can have a `url`");
                if is_set(&sku_id) {
                    self.forbid(&sku_id, "only `Premium` buttons can have a `sku_id`");
                }
            }
            // The style is only known at runtime, so check the combinations
            // that no style allows.
            None => {
                if custom_id.is_some() && url.is_some() {
                    self.forbid(&url, "a button cannot have both a `custom_id` and a `url`");
                }
                if is_set(&sku_id) {
                    if custom_id.is_some() {
                        self.forbid(
                            &custom_id,
                            "a button cannot have both a `sku_id` and a `custom_id`",
                        );
                    }
                    if url.is_some() {
                        self.forbid(&url, "a button cannot have both a `sku_id` and a `url`");
                    }
                }
            }
        }
    }

    fn select_menu(&mut self, node: &ComponentNode) {
        self.custom_id(node, true);
        self.max_len(node, "placeholder", SELECT_PLACEHOLDER_MAX);
        let min = self.int_range(node, "min_values", 0, SELECT_VALUES_MAX);
        let max = self.int_range(node, "max_values", 0, SELECT_VALUES_MAX);
        self.min_le_max(node, "min_values", min, "max_values", max);

        let kind = match prop_expr(node, "kind") {
            Some(expr) => self.choice(expr, SELECT_KINDS, "select menu kind"),
            None => Some("Text"),
        }
        .map(|k| if k == "String" { "Text" } else { k });

        let kids = children(node);
        for (i, child) in kids.iter().enumerate() {
            if child.kind != ComponentKind::SelectOption {
                self.error(
                    &child.ident,
                    format!(
                        "`SelectMenu` can only contain `SelectOption`, not `{}`",
                        child.kind.name()
                    ),
                );
            } else if kind == Some("Text") && i == SELECT_OPTION_MAX {
                self.error(
                    &child.ident,
                    format!("a select menu can have at most {SELECT_OPTION_MAX} options"),
                );
            }
        }

        match kind {
            Some("Text") => {
                if kids.is_empty() {
                    self.error(
                        &node.ident,
                        "a `Text` select menu needs at least one option",
                    );
                }
            }
            Some(kind) => {
                if let Some(first) = kids.first() {
                    self.error(
                        &first.ident,
                        format!("only `Text` select menus have options; this one is `{kind}`"),
                    );
                }
                if kind != "Channel"
                    && let Some(p) = prop(node, "channel_types")
                {
                    self.error(
                        &p.key,
                        "only `Channel` select menus can have `channel_types`",
                    );
                }
            }
            None => {}
        }

        if let Some(p) = prop(node, "default_values") {
            if kind == Some("Text") {
                self.error(
                    &p.key,
                    "`Text` select menus cannot have `default_values`; set `default` on an option",
                );
            } else if let PropertyValue::Expr(expr) = &p.value
                && let Some(count) = seq_len(some_inner(expr).unwrap_or(expr))
            {
                // Missing bounds default to 1; a runtime bound is not checked.
                let min_bound = match prop(node, "min_values") {
                    None => Some(1),
                    Some(_) => min,
                };
                let max_bound = match prop(node, "max_values") {
                    None => Some(1),
                    Some(_) => max,
                };
                let count = count as u128;
                if min_bound.is_some_and(|m| count < m) || max_bound.is_some_and(|m| count > m) {
                    self.error(
                        expr,
                        format!(
                            "the number of `default_values` ({count}) must be between `min_values` and `max_values` (both default to 1)"
                        ),
                    );
                }
            }
        }

        if self.context == Context::Modal {
            if let Some(expr) = prop_expr(node, "disabled")
                && lit_bool(expr) == Some(true)
            {
                self.error(expr, "components in a modal cannot be disabled");
            }
            self.modal_min_values(node);
        }
    }

    fn text_input(&mut self, node: &ComponentNode) {
        self.custom_id(node, true);
        if let Some(expr) = prop_expr(node, "style") {
            self.choice(expr, TEXT_INPUT_STYLES, "text input style");
        }
        let min = self.int_range(node, "min_length", 0, TEXT_INPUT_LENGTH_MAX);
        let max = self.int_range(node, "max_length", 1, TEXT_INPUT_LENGTH_MAX);
        self.min_le_max(node, "min_length", min, "max_length", max);
        self.max_len(node, "placeholder", TEXT_INPUT_PLACEHOLDER_MAX);
        self.max_len(node, "value", TEXT_INPUT_LENGTH_MAX as usize);
    }

    fn select_option(&mut self, node: &ComponentNode) {
        self.require(node, "label", true);
        self.max_len_with_shorthand(node, "label", SELECT_OPTION_TEXT_MAX);
        self.require(node, "value", false);
        self.max_len(node, "value", SELECT_OPTION_TEXT_MAX);
        self.max_len(node, "description", SELECT_OPTION_TEXT_MAX);
    }

    // -------------------------------------------------------------------------
    // Field helpers
    // -------------------------------------------------------------------------

    /// Reports a missing field and returns the field's expression when it is one.
    fn require<'a>(
        &mut self,
        node: &'a ComponentNode,
        name: &str,
        shorthand: bool,
    ) -> Option<&'a Expr> {
        if shorthand && let Some(content) = &node.shorthand {
            return Some(&content.expr);
        }
        if prop(node, name).is_none() {
            self.error(
                &node.ident,
                format!("`{}` requires a `{name}`", node.kind.name()),
            );
        }
        prop_expr(node, name)
    }

    fn custom_id(&mut self, node: &ComponentNode, required: bool) {
        let expr = if required {
            self.require(node, "custom_id", false)
        } else {
            prop_expr(node, "custom_id")
        };
        if let Some(expr) = expr
            && let Some(value) = lit_str(expr)
        {
            let len = value.chars().count();
            if !(1..=CUSTOM_ID_MAX).contains(&len) {
                self.error(
                    expr,
                    format!("`custom_id` must be 1 to {CUSTOM_ID_MAX} characters; it is {len}"),
                );
            }
        }
    }

    fn max_len(&mut self, node: &ComponentNode, name: &str, max: usize) {
        if let Some(expr) = prop_expr(node, name) {
            self.check_len(expr, name, max);
        }
    }

    fn max_len_with_shorthand(&mut self, node: &ComponentNode, name: &str, max: usize) {
        match &node.shorthand {
            Some(content) => self.check_len(&content.expr, name, max),
            None => self.max_len(node, name, max),
        }
    }

    fn check_len(&mut self, expr: &Expr, name: &str, max: usize) {
        if let Some(value) = lit_str(expr) {
            let len = value.chars().count();
            if len > max {
                self.error(
                    expr,
                    format!("`{name}` can be at most {max} characters; it is {len}"),
                );
            }
        }
    }

    /// Checks an integer literal against a range and returns it.
    fn int_range(
        &mut self,
        node: &ComponentNode,
        name: &str,
        min: u128,
        max: u128,
    ) -> Option<u128> {
        let expr = prop_expr(node, name)?;
        let value = lit_int(expr)?;
        if !(min..=max).contains(&value) {
            self.error(expr, format!("`{name}` must be from {min} to {max}"));
        }
        Some(value)
    }

    fn min_le_max(
        &mut self,
        node: &ComponentNode,
        min_name: &str,
        min: Option<u128>,
        max_name: &str,
        max: Option<u128>,
    ) {
        if let (Some(min), Some(max)) = (min, max)
            && min > max
            && let Some(expr) = prop_expr(node, min_name)
        {
            self.error(
                expr,
                format!("`{min_name}` ({min}) is greater than `{max_name}` ({max})"),
            );
        }
    }

    /// In a modal, `min_values: 0` needs `required: false`.
    fn modal_min_values(&mut self, node: &ComponentNode) {
        let Some(expr) = prop_expr(node, "min_values") else {
            return;
        };
        // `required` defaults to true. A runtime value may be false, so it is not reported.
        let required = match prop_expr(node, "required") {
            None => Some(true),
            Some(required) => lit_bool(required),
        };
        if lit_int(expr) == Some(0) && required == Some(true) {
            self.error(
                expr,
                "in a modal, `min_values` can only be 0 when `required: false` is set",
            );
        }
    }

    fn forbid<T: ToTokens>(&mut self, field: &Option<(T, Setness)>, message: &str) {
        if let Some((tokens, _)) = field {
            self.error(tokens, message);
        }
    }

    /// Resolves an enum-like value. Returns the known variant name, or `None`
    /// when the value is only known at runtime or is invalid (invalid values are reported).
    fn choice(
        &mut self,
        expr: &Expr,
        allowed: &[&'static str],
        what: &str,
    ) -> Option<&'static str> {
        let expr = unwrap_expr(expr);
        let (name, is_call) = match expr {
            Expr::Path(p) => (p.path.segments.last()?.ident.to_string(), false),
            Expr::Call(c) => match unwrap_expr(&c.func) {
                Expr::Path(p) => (p.path.segments.last()?.ident.to_string(), true),
                _ => return None,
            },
            _ => return None,
        };
        if !is_call && let Some(known) = allowed.iter().find(|a| **a == name) {
            return Some(known);
        }
        // CamelCase names are enum variants; anything else is a variable or constant.
        if is_camel_case(&name) {
            self.error(
                expr,
                format!(
                    "unknown {what} `{name}`; expected one of {}",
                    allowed.join(", ")
                ),
            );
        }
        None
    }
}

// =============================================================================
// Kind tables
// =============================================================================

/// The fields each kind accepts, as `(name, alias)`.
fn fields(kind: ComponentKind) -> &'static [(&'static str, Option<&'static str>)] {
    match kind {
        ComponentKind::Text => &[("id", None), ("content", None)],
        ComponentKind::Container => &[
            ("id", None),
            ("accent_color", Some("color")),
            ("spoiler", None),
        ],
        ComponentKind::Section => &[("id", None), ("accessory", None)],
        ComponentKind::Separator => &[("id", None), ("divider", None), ("spacing", None)],
        ComponentKind::MediaGallery | ComponentKind::ActionRow => &[("id", None)],
        ComponentKind::Thumbnail => &[
            ("id", None),
            ("url", None),
            ("description", Some("alt")),
            ("spoiler", None),
        ],
        ComponentKind::File => &[("id", None), ("url", None), ("spoiler", None)],
        ComponentKind::FileUpload => &[
            ("id", None),
            ("custom_id", None),
            ("min_values", None),
            ("max_values", None),
            ("required", None),
        ],
        ComponentKind::Label => &[
            ("id", None),
            ("label", None),
            ("description", None),
            ("component", None),
        ],
        ComponentKind::Button => &[
            ("id", None),
            ("style", None),
            ("label", None),
            ("custom_id", None),
            ("url", None),
            ("disabled", None),
            ("emoji", None),
            ("sku_id", None),
        ],
        ComponentKind::SelectMenu => &[
            ("id", None),
            ("custom_id", None),
            ("kind", None),
            ("placeholder", None),
            ("min_values", None),
            ("max_values", None),
            ("disabled", None),
            ("required", None),
            ("channel_types", None),
            ("default_values", None),
        ],
        ComponentKind::TextInput => &[
            ("id", None),
            ("custom_id", None),
            ("style", None),
            ("min_length", None),
            ("max_length", None),
            ("placeholder", None),
            ("required", None),
            ("value", None),
        ],
        ComponentKind::MediaItem => &[
            ("url", None),
            ("description", Some("alt")),
            ("spoiler", None),
        ],
        ComponentKind::SelectOption => &[
            ("label", None),
            ("value", None),
            ("description", None),
            ("emoji", None),
            ("default", None),
        ],
    }
}

/// The field that `Kind(value)` sets, if the kind has one.
fn shorthand_field(kind: ComponentKind) -> Option<&'static str> {
    match kind {
        ComponentKind::Text => Some("content"),
        ComponentKind::Thumbnail | ComponentKind::File | ComponentKind::MediaItem => Some("url"),
        ComponentKind::Label | ComponentKind::Button | ComponentKind::SelectOption => Some("label"),
        _ => None,
    }
}

fn takes_children(kind: ComponentKind) -> bool {
    matches!(
        kind,
        ComponentKind::Container
            | ComponentKind::Section
            | ComponentKind::MediaGallery
            | ComponentKind::Label
            | ComponentKind::ActionRow
            | ComponentKind::SelectMenu
    )
}

fn context_of(kind: ComponentKind) -> Context {
    match kind {
        ComponentKind::Label | ComponentKind::TextInput | ComponentKind::FileUpload => {
            Context::Modal
        }
        ComponentKind::Text
        | ComponentKind::SelectMenu
        | ComponentKind::SelectOption
        | ComponentKind::MediaItem => Context::Unknown,
        ComponentKind::ActionRow
        | ComponentKind::Button
        | ComponentKind::Container
        | ComponentKind::Section
        | ComponentKind::Separator
        | ComponentKind::MediaGallery
        | ComponentKind::Thumbnail
        | ComponentKind::File => Context::Message,
    }
}

// =============================================================================
// AST helpers
// =============================================================================

fn children(node: &ComponentNode) -> Vec<&ComponentNode> {
    node.children
        .as_ref()
        .map(|c| c.children.iter().collect())
        .unwrap_or_default()
}

/// Components given as property values, such as a Section `accessory`.
fn property_components(node: &ComponentNode) -> impl Iterator<Item = &ComponentNode> {
    node.properties
        .iter()
        .flat_map(|p| p.properties.iter())
        .filter_map(|p| match &p.value {
            PropertyValue::Component(c) => Some(c.as_ref()),
            PropertyValue::Expr(_) => None,
        })
}

/// Number of components given as plain expressions (`accessory` or `component`).
fn expression_components(node: &ComponentNode) -> usize {
    node.properties
        .iter()
        .flat_map(|p| p.properties.iter())
        .filter(|p| {
            (p.key == "accessory" || p.key == "component")
                && matches!(p.value, PropertyValue::Expr(_))
        })
        .count()
}

fn walk<'a>(node: &'a ComponentNode, f: &mut impl FnMut(&'a ComponentNode)) {
    f(node);
    for child in children(node) {
        walk(child, f);
    }
    for nested in property_components(node) {
        walk(nested, f);
    }
}

/// Finds a property by name or by its alias.
fn prop<'a>(node: &'a ComponentNode, name: &str) -> Option<&'a Property> {
    let alias = fields(node.kind)
        .iter()
        .find(|f| f.0 == name)
        .and_then(|f| f.1);
    node.properties
        .as_ref()?
        .properties
        .iter()
        .find(|p| p.key == name || alias.is_some_and(|a| p.key == a))
}

fn prop_expr<'a>(node: &'a ComponentNode, name: &str) -> Option<&'a Expr> {
    match &prop(node, name)?.value {
        PropertyValue::Expr(expr) => Some(expr),
        PropertyValue::Component(_) => None,
    }
}

/// Whether an optional field's value is set.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Setness {
    Yes,
    No,
    /// Only known at runtime.
    Maybe,
}

fn setness(value: &PropertyValue) -> Setness {
    let PropertyValue::Expr(expr) = value else {
        return Setness::Yes;
    };
    let expr = unwrap_expr(expr);
    if is_none(expr) {
        Setness::No
    } else if matches!(expr, Expr::Lit(_) | Expr::Struct(_)) || some_inner(expr).is_some() {
        Setness::Yes
    } else {
        Setness::Maybe
    }
}

fn unwrap_expr(expr: &Expr) -> &Expr {
    match expr {
        Expr::Paren(p) => unwrap_expr(&p.expr),
        Expr::Group(g) => unwrap_expr(&g.expr),
        _ => expr,
    }
}

fn lit_str(expr: &Expr) -> Option<String> {
    match unwrap_expr(expr) {
        Expr::Lit(syn::ExprLit {
            lit: Lit::Str(s), ..
        }) => Some(s.value()),
        _ => None,
    }
}

fn lit_int(expr: &Expr) -> Option<u128> {
    match unwrap_expr(expr) {
        Expr::Lit(syn::ExprLit {
            lit: Lit::Int(i), ..
        }) => i.base10_parse().ok(),
        _ => None,
    }
}

fn lit_bool(expr: &Expr) -> Option<bool> {
    match unwrap_expr(expr) {
        Expr::Lit(syn::ExprLit {
            lit: Lit::Bool(b), ..
        }) => Some(b.value),
        _ => None,
    }
}

fn is_none(expr: &Expr) -> bool {
    matches!(unwrap_expr(expr), Expr::Path(p) if p.path.is_ident("None"))
}

/// The inner expression of `Some(inner)`.
fn some_inner(expr: &Expr) -> Option<&Expr> {
    match unwrap_expr(expr) {
        Expr::Call(call) if call.args.len() == 1 => match unwrap_expr(&call.func) {
            Expr::Path(p) if p.path.is_ident("Some") => call.args.first(),
            _ => None,
        },
        _ => None,
    }
}

/// The length of a literal `[a, b]` or `vec![a, b]`.
fn seq_len(expr: &Expr) -> Option<usize> {
    match unwrap_expr(expr) {
        Expr::Array(array) => Some(array.elems.len()),
        Expr::Macro(m) if m.mac.path.is_ident("vec") => m
            .mac
            .parse_body_with(Punctuated::<Expr, Token![,]>::parse_terminated)
            .ok()
            .map(|elems| elems.len()),
        _ => None,
    }
}

fn is_camel_case(name: &str) -> bool {
    name.chars().next().is_some_and(char::is_uppercase) && name.chars().any(char::is_lowercase)
}

/// Used by tests to render error messages.
#[cfg(test)]
fn messages(err: syn::Error) -> Vec<String> {
    err.into_iter().map(|e| e.to_string()).collect()
}

#[cfg(test)]
mod tests;
