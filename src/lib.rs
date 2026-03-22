//! # Twilight Components Macro
//!
//! A declarative macro for building Discord message components using twilight-model.
//!
//! ## Design Philosophy
//!
//! This macro provides a concise, readable DSL for constructing Discord components
//! that maps naturally to the visual hierarchy of the components themselves.
//!
//! ### Syntax Overview
//!
//! ```text
//! component!(
//!     ComponentType { field: value, ... } [
//!         ChildComponent { ... },
//!         ...
//!     ]
//! )
//! ```
//!
//! - **Braces `{}`**: Optional fields/properties for the component
//! - **Brackets `[]`**: Child components (for container types)
//! - **Parentheses `()`**: Shorthand for the primary/content field
//!
//! ## Examples
//!
//! ### Simple Text Display
//! ```ignore
//! component!(Text("Hello, World!"))
//! // Expands to: Component::TextDisplay(TextDisplay { id: None, content: "Hello, World!".into() })
//! ```
//!
//! ### Button with Properties
//! ```ignore
//! component!(Button { style: Primary, label: "Click Me", custom_id: "btn_1" })
//! ```
//!
//! ### Container with Children
//! ```ignore
//! component!(Container { accent_color: 0xFF0000 } [
//!     Text("Welcome!"),
//!     Separator,
//!     ActionRow [
//!         Button { style: Primary, label: "OK" },
//!         Button { style: Danger, label: "Cancel" },
//!     ],
//! ])
//! ```
//!
//! ### Section with Accessory
//! ```ignore
//! component!(Section {
//!     accessory: Button { style: Link, url: "https://example.com", label: "Visit" }
//! } [
//!     Text("Check out our website!"),
//! ])
//! ```
//!
//! ## Component Reference
//!
//! | Macro Syntax | Twilight Type | Primary Field | Children |
//! |--------------|---------------|---------------|----------|
//! | `Text(content)` | `TextDisplay` | `content` | No |
//! | `Button { ... }` | `Button` | - | No |
//! | `ActionRow [ ... ]` | `ActionRow` | - | Yes |
//! | `Container [ ... ]` | `Container` | - | Yes |
//! | `Section [ ... ]` | `Section` | - | Yes (text components) |
//! | `Separator` | `Separator` | - | No |
//! | `MediaGallery [ ... ]` | `MediaGallery` | - | Yes (items) |
//! | `Thumbnail { url }` | `Thumbnail` | `media.url` | No |
//! | `File { url }` | `FileDisplay` | `file.url` | No |
//! | `FileUpload { ... }` | `FileUpload` | - | No |
//! | `Label { ... }` | `Label` | `label` | Yes (single component) |
//! | `SelectMenu { ... }` | `SelectMenu` | - | Yes (options) |
//! | `TextInput { ... }` | `TextInput` | - | No |

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use syn::{
    Expr, Ident, Result, Token, braced, bracketed, parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    token,
};

/// The main macro for building Discord components.
///
/// See the [crate-level documentation](crate) for detailed usage examples.
#[proc_macro]
pub fn component(input: TokenStream) -> TokenStream {
    let component = parse_macro_input!(input as ComponentNode);
    component.to_token_stream().into()
}

/// Macro for building an array of components `[Component; N]` from multiple top-level components.
///
/// This returns an array rather than a `Vec`, which allows efficient borrowing as `&[Component]`
/// without unnecessary allocations.
///
/// # Example
/// ```ignore
/// components!(
///     Container [
///         Text("Hello"),
///     ],
///     Container [
///         Text("World"),
///     ],
/// )
/// ```
#[proc_macro]
pub fn components(input: TokenStream) -> TokenStream {
    let components = parse_macro_input!(input as ComponentList);
    components.to_token_stream().into()
}

// =============================================================================
// AST Types
// =============================================================================

/// A list of components (for the `components!` macro)
struct ComponentList {
    components: Punctuated<ComponentNode, Token![,]>,
}

impl Parse for ComponentList {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(ComponentList {
            components: Punctuated::parse_terminated(input)?,
        })
    }
}

impl ToTokens for ComponentList {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let components = self.components.iter();
        tokens.extend(quote! {
            [#(#components),*]
        });
    }
}

/// A single component node in the DSL
struct ComponentNode {
    kind: ComponentKind,
    properties: Option<PropertyBlock>,
    children: Option<ChildBlock>,
    shorthand: Option<ShorthandContent>,
}

impl Parse for ComponentNode {
    fn parse(input: ParseStream) -> Result<Self> {
        let kind: ComponentKind = input.parse()?;

        // Parse optional shorthand content: ComponentType(content)
        let shorthand = if input.peek(token::Paren) {
            let content;
            parenthesized!(content in input);
            Some(content.parse()?)
        } else {
            None
        };

        // Parse optional property block: { field: value, ... }
        let properties = if input.peek(token::Brace) {
            Some(input.parse()?)
        } else {
            None
        };

        // Parse optional children block: [ child, ... ]
        let children = if input.peek(token::Bracket) {
            Some(input.parse()?)
        } else {
            None
        };

        Ok(ComponentNode {
            kind,
            properties,
            children,
            shorthand,
        })
    }
}

impl ToTokens for ComponentNode {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let generated = self.kind.generate(
            self.properties.as_ref(),
            self.children.as_ref(),
            self.shorthand.as_ref(),
        );
        tokens.extend(generated);
    }
}

/// The type of component being constructed
#[derive(Debug, Clone)]
enum ComponentKind {
    // Display Components
    Text,
    Container,
    Section,
    Separator,
    MediaGallery,
    Thumbnail,
    File,
    FileUpload,

    // Layout Components
    Label,

    // Interactive Components
    ActionRow,
    Button,
    SelectMenu,
    TextInput,

    // Supporting types (used as children)
    MediaItem,
    SelectOption,
}

impl Parse for ComponentKind {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;
        match ident.to_string().as_str() {
            "Text" | "TextDisplay" => Ok(ComponentKind::Text),
            "Container" => Ok(ComponentKind::Container),
            "Section" => Ok(ComponentKind::Section),
            "Separator" | "Sep" => Ok(ComponentKind::Separator),
            "MediaGallery" | "Gallery" => Ok(ComponentKind::MediaGallery),
            "Thumbnail" | "Thumb" => Ok(ComponentKind::Thumbnail),
            "File" | "FileDisplay" => Ok(ComponentKind::File),
            "FileUpload" | "Upload" => Ok(ComponentKind::FileUpload),
            "Label" => Ok(ComponentKind::Label),
            "ActionRow" | "Row" => Ok(ComponentKind::ActionRow),
            "Button" | "Btn" => Ok(ComponentKind::Button),
            "SelectMenu" | "Select" => Ok(ComponentKind::SelectMenu),
            "TextInput" | "Input" => Ok(ComponentKind::TextInput),
            "MediaItem" | "Item" => Ok(ComponentKind::MediaItem),
            "SelectOption" | "Option" => Ok(ComponentKind::SelectOption),
            other => Err(syn::Error::new(
                ident.span(),
                format!(
                    "Unknown component type: `{}`. Valid types are: Text, Container, Section, Separator, MediaGallery, Thumbnail, File, FileUpload, Label, ActionRow, Button, SelectMenu, TextInput, MediaItem, SelectOption",
                    other
                ),
            )),
        }
    }
}

impl ComponentKind {
    fn generate(
        &self,
        properties: Option<&PropertyBlock>,
        children: Option<&ChildBlock>,
        shorthand: Option<&ShorthandContent>,
    ) -> TokenStream2 {
        match self {
            ComponentKind::Text => self.generate_text_display(properties, shorthand),
            ComponentKind::Container => self.generate_container(properties, children),
            ComponentKind::Section => self.generate_section(properties, children),
            ComponentKind::Separator => self.generate_separator(properties),
            ComponentKind::MediaGallery => self.generate_media_gallery(properties, children),
            ComponentKind::Thumbnail => self.generate_thumbnail(properties, shorthand),
            ComponentKind::File => self.generate_file(properties, shorthand),
            ComponentKind::FileUpload => self.generate_file_upload(properties),
            ComponentKind::Label => self.generate_label(properties, children, shorthand),
            ComponentKind::ActionRow => self.generate_action_row(properties, children),
            ComponentKind::Button => self.generate_button(properties, shorthand),
            ComponentKind::SelectMenu => self.generate_select_menu(properties, children),
            ComponentKind::TextInput => self.generate_text_input(properties),
            ComponentKind::MediaItem => self.generate_media_item(properties, shorthand),
            ComponentKind::SelectOption => self.generate_select_option(properties, shorthand),
        }
    }

    fn generate_text_display(
        &self,
        properties: Option<&PropertyBlock>,
        shorthand: Option<&ShorthandContent>,
    ) -> TokenStream2 {
        let content = shorthand
            .map(|s| s.to_token_stream())
            .or_else(|| properties.and_then(|p| p.get("content")))
            .unwrap_or_else(|| quote! { String::new() });

        let id = properties
            .and_then(|p| p.get("id"))
            .unwrap_or_else(|| quote! { None });

        quote! {
            ::twilight_model::channel::message::Component::TextDisplay(
                ::twilight_model::channel::message::component::TextDisplay {
                    id: #id,
                    content: (#content).into(),
                }
            )
        }
    }

    fn generate_container(
        &self,
        properties: Option<&PropertyBlock>,
        children: Option<&ChildBlock>,
    ) -> TokenStream2 {
        let components = children
            .map(|c| {
                let kids = c.children.iter();
                quote! { vec![#(#kids),*] }
            })
            .unwrap_or_else(|| quote! { vec![] });

        let id = properties
            .and_then(|p| p.get("id"))
            .unwrap_or_else(|| quote! { None });

        // accent_color is Option<Option<u32>> - wrap user value in Some(Some(...))
        let accent_color = properties
            .and_then(|p| p.get("accent_color").or_else(|| p.get("color")))
            .map(|c| quote! { Some(Some(#c)) })
            .unwrap_or_else(|| quote! { None });

        // spoiler is Option<bool> - wrap user value in Some(...)
        let spoiler = properties
            .and_then(|p| p.get("spoiler"))
            .map(|s| quote! { Some(#s) })
            .unwrap_or_else(|| quote! { None });

        quote! {
            ::twilight_model::channel::message::Component::Container(
                ::twilight_model::channel::message::component::Container {
                    id: #id,
                    accent_color: #accent_color,
                    components: #components,
                    spoiler: #spoiler,
                }
            )
        }
    }

    fn generate_section(
        &self,
        properties: Option<&PropertyBlock>,
        children: Option<&ChildBlock>,
    ) -> TokenStream2 {
        let components = children
            .map(|c| {
                let kids = c.children.iter();
                quote! { vec![#(#kids),*] }
            })
            .unwrap_or_else(|| quote! { vec![] });

        let id = properties
            .and_then(|p| p.get("id"))
            .unwrap_or_else(|| quote! { None });

        // Accessory is required (Box<Component>) - must be a Button or Thumbnail
        // Both Button and Thumbnail now return Component variants, so just wrap in Box
        let accessory = properties
            .and_then(|p| p.get("accessory"))
            .map(|acc| quote! { Box::new(#acc) })
            .unwrap_or_else(|| {
                // If no accessory provided, generate code that will fail to compile
                // with a clear error about missing accessory
                quote! { compile_error!("Section requires an accessory (Button or Thumbnail)") }
            });

        quote! {
            ::twilight_model::channel::message::Component::Section(
                ::twilight_model::channel::message::component::Section {
                    id: #id,
                    accessory: #accessory,
                    components: #components,
                }
            )
        }
    }

    fn generate_separator(&self, properties: Option<&PropertyBlock>) -> TokenStream2 {
        let id = properties
            .and_then(|p| p.get("id"))
            .unwrap_or_else(|| quote! { None });

        // divider is Option<bool> - default to Some(true) per design
        let divider = properties
            .and_then(|p| p.get("divider"))
            .map(|d| quote! { Some(#d) })
            .unwrap_or_else(|| quote! { Some(true) });

        // spacing is Option<SeparatorSpacingSize>
        let spacing = properties
            .and_then(|p| p.get("spacing"))
            .map(|s| quote! { Some(#s) })
            .unwrap_or_else(|| quote! { None });

        quote! {
            ::twilight_model::channel::message::Component::Separator(
                ::twilight_model::channel::message::component::Separator {
                    id: #id,
                    divider: #divider,
                    spacing: #spacing,
                }
            )
        }
    }

    fn generate_media_gallery(
        &self,
        properties: Option<&PropertyBlock>,
        children: Option<&ChildBlock>,
    ) -> TokenStream2 {
        let items = children
            .map(|c| {
                let kids = c.children.iter();
                quote! { vec![#(#kids),*] }
            })
            .unwrap_or_else(|| quote! { vec![] });

        let id = properties
            .and_then(|p| p.get("id"))
            .unwrap_or_else(|| quote! { None });

        quote! {
            ::twilight_model::channel::message::Component::MediaGallery(
                ::twilight_model::channel::message::component::MediaGallery {
                    id: #id,
                    items: #items,
                }
            )
        }
    }

    fn generate_thumbnail(
        &self,
        properties: Option<&PropertyBlock>,
        shorthand: Option<&ShorthandContent>,
    ) -> TokenStream2 {
        let url = shorthand
            .map(|s| s.to_token_stream())
            .or_else(|| properties.and_then(|p| p.get("url")))
            .unwrap_or_else(|| quote! { "" });

        let id = properties
            .and_then(|p| p.get("id"))
            .unwrap_or_else(|| quote! { None });

        // description is Option<Option<String>> - wrap in Some(Some(...)) when provided
        let description = properties
            .and_then(|p| p.get("description").or_else(|| p.get("alt")))
            .map(|d| quote! { Some(Some((#d).into())) })
            .unwrap_or_else(|| quote! { None });

        // spoiler is Option<bool>
        let spoiler = properties
            .and_then(|p| p.get("spoiler"))
            .map(|s| quote! { Some(#s) })
            .unwrap_or_else(|| quote! { None });

        quote! {
            ::twilight_model::channel::message::Component::Thumbnail(
                ::twilight_model::channel::message::component::Thumbnail {
                    id: #id,
                    description: #description,
                    media: ::twilight_model::channel::message::component::UnfurledMediaItem {
                        url: (#url).into(),
                        proxy_url: None,
                        height: None,
                        width: None,
                        content_type: None,
                    },
                    spoiler: #spoiler,
                }
            )
        }
    }

    fn generate_file(
        &self,
        properties: Option<&PropertyBlock>,
        shorthand: Option<&ShorthandContent>,
    ) -> TokenStream2 {
        let url = shorthand
            .map(|s| s.to_token_stream())
            .or_else(|| properties.and_then(|p| p.get("url")))
            .unwrap_or_else(|| quote! { "" });

        let id = properties
            .and_then(|p| p.get("id"))
            .unwrap_or_else(|| quote! { None });

        // spoiler is Option<bool>
        let spoiler = properties
            .and_then(|p| p.get("spoiler"))
            .map(|s| quote! { Some(#s) })
            .unwrap_or_else(|| quote! { None });

        quote! {
            ::twilight_model::channel::message::Component::File(
                ::twilight_model::channel::message::component::FileDisplay {
                    id: #id,
                    file: ::twilight_model::channel::message::component::UnfurledMediaItem {
                        url: (#url).into(),
                        proxy_url: None,
                        height: None,
                        width: None,
                        content_type: None,
                    },
                    spoiler: #spoiler,
                }
            )
        }
    }

    fn generate_file_upload(&self, properties: Option<&PropertyBlock>) -> TokenStream2 {
        let id = properties
            .and_then(|p| p.get("id"))
            .unwrap_or_else(|| quote! { None });

        let custom_id = properties
            .and_then(|p| p.get("custom_id"))
            .map(|c| quote! { (#c).into() })
            .unwrap_or_else(|| quote! { String::new() });

        let max_values = properties
            .and_then(|p| p.get("max_values"))
            .map(|m| quote! { Some(#m) })
            .unwrap_or_else(|| quote! { None });

        let min_values = properties
            .and_then(|p| p.get("min_values"))
            .map(|m| quote! { Some(#m) })
            .unwrap_or_else(|| quote! { None });

        let required = properties
            .and_then(|p| p.get("required"))
            .map(|r| quote! { Some(#r) })
            .unwrap_or_else(|| quote! { None });

        quote! {
            ::twilight_model::channel::message::Component::FileUpload(
                ::twilight_model::channel::message::component::FileUpload {
                    id: #id,
                    custom_id: #custom_id,
                    max_values: #max_values,
                    min_values: #min_values,
                    required: #required,
                }
            )
        }
    }

    fn generate_label(
        &self,
        properties: Option<&PropertyBlock>,
        children: Option<&ChildBlock>,
        shorthand: Option<&ShorthandContent>,
    ) -> TokenStream2 {
        let label = shorthand
            .map(|s| s.to_token_stream())
            .or_else(|| properties.and_then(|p| p.get("label")))
            .map(|l| quote! { (#l).into() })
            .unwrap_or_else(|| quote! { String::new() });

        let id = properties
            .and_then(|p| p.get("id"))
            .unwrap_or_else(|| quote! { None });

        let description = properties
            .and_then(|p| p.get("description"))
            .map(|d| quote! { Some((#d).into()) })
            .unwrap_or_else(|| quote! { None });

        // The Label wraps a single child component
        let component = children
            .and_then(|c| c.children.first())
            .map(|child| quote! { Box::new(#child) })
            .or_else(|| {
                properties
                    .and_then(|p| p.get("component"))
                    .map(|c| quote! { Box::new(#c) })
            })
            .unwrap_or_else(|| {
                quote! { compile_error!("Label requires a child component") }
            });

        quote! {
            ::twilight_model::channel::message::Component::Label(
                ::twilight_model::channel::message::component::Label {
                    id: #id,
                    label: #label,
                    description: #description,
                    component: #component,
                }
            )
        }
    }

    fn generate_action_row(
        &self,
        properties: Option<&PropertyBlock>,
        children: Option<&ChildBlock>,
    ) -> TokenStream2 {
        let components = children
            .map(|c| {
                let kids = c.children.iter();
                quote! { vec![#(#kids),*] }
            })
            .unwrap_or_else(|| quote! { vec![] });

        let id = properties
            .and_then(|p| p.get("id"))
            .unwrap_or_else(|| quote! { None });

        quote! {
            ::twilight_model::channel::message::Component::ActionRow(
                ::twilight_model::channel::message::component::ActionRow {
                    id: #id,
                    components: #components,
                }
            )
        }
    }

    fn generate_button(
        &self,
        properties: Option<&PropertyBlock>,
        shorthand: Option<&ShorthandContent>,
    ) -> TokenStream2 {
        let label = shorthand
            .map(|s| {
                let content = s.to_token_stream();
                quote! { Some((#content).into()) }
            })
            .or_else(|| {
                properties
                    .and_then(|p| p.get("label"))
                    .map(|l| quote! { Some((#l).into()) })
            })
            .unwrap_or_else(|| quote! { None });

        let id = properties
            .and_then(|p| p.get("id"))
            .unwrap_or_else(|| quote! { None });

        let style = properties
            .and_then(|p| p.get("style"))
            .map(resolve_button_style)
            .unwrap_or_else(
                || quote! { ::twilight_model::channel::message::component::ButtonStyle::Primary },
            );

        let custom_id = properties
            .and_then(|p| p.get("custom_id"))
            .map(|c| quote! { Some((#c).into()) })
            .unwrap_or_else(|| quote! { None });

        let url = properties
            .and_then(|p| p.get("url"))
            .map(|u| quote! { Some((#u).into()) })
            .unwrap_or_else(|| quote! { None });

        let disabled = properties
            .and_then(|p| p.get("disabled"))
            .unwrap_or_else(|| quote! { false });

        let emoji = properties
            .and_then(|p| p.get("emoji"))
            .unwrap_or_else(|| quote! { None });

        let sku_id = properties
            .and_then(|p| p.get("sku_id"))
            .unwrap_or_else(|| quote! { None });

        quote! {
            ::twilight_model::channel::message::Component::Button(
                ::twilight_model::channel::message::component::Button {
                    id: #id,
                    custom_id: #custom_id,
                    disabled: #disabled,
                    emoji: #emoji,
                    label: #label,
                    style: #style,
                    url: #url,
                    sku_id: #sku_id,
                }
            )
        }
    }

    fn generate_select_menu(
        &self,
        properties: Option<&PropertyBlock>,
        children: Option<&ChildBlock>,
    ) -> TokenStream2 {
        let options = children
            .map(|c| {
                let kids = c.children.iter();
                quote! { Some(vec![#(#kids),*]) }
            })
            .unwrap_or_else(|| quote! { None });

        let id = properties
            .and_then(|p| p.get("id"))
            .unwrap_or_else(|| quote! { None });

        let custom_id = properties
            .and_then(|p| p.get("custom_id"))
            .map(|c| quote! { (#c).into() })
            .unwrap_or_else(|| quote! { String::new() });

        let kind = properties
            .and_then(|p| p.get("kind").or_else(|| p.get("type")))
            .map(resolve_select_menu_type)
            .unwrap_or_else(
                || quote! { ::twilight_model::channel::message::component::SelectMenuType::Text },
            );

        let placeholder = properties
            .and_then(|p| p.get("placeholder"))
            .map(|ph| quote! { Some((#ph).into()) })
            .unwrap_or_else(|| quote! { None });

        let min_values = properties
            .and_then(|p| p.get("min_values"))
            .map(|m| quote! { Some(#m) })
            .unwrap_or_else(|| quote! { None });

        let max_values = properties
            .and_then(|p| p.get("max_values"))
            .map(|m| quote! { Some(#m) })
            .unwrap_or_else(|| quote! { None });

        let disabled = properties
            .and_then(|p| p.get("disabled"))
            .unwrap_or_else(|| quote! { false });

        let channel_types = properties
            .and_then(|p| p.get("channel_types"))
            .map(|m| quote! { Some(#m) })
            .unwrap_or_else(|| quote! { None });

        let default_values = properties
            .and_then(|p| p.get("default_values"))
            .unwrap_or_else(|| quote! { None });

        let required = properties
            .and_then(|p| p.get("required"))
            .map(|r| quote! { Some(#r) })
            .unwrap_or_else(|| quote! { None });

        quote! {
            ::twilight_model::channel::message::Component::SelectMenu(
                ::twilight_model::channel::message::component::SelectMenu {
                    id: #id,
                    channel_types: #channel_types,
                    custom_id: #custom_id,
                    default_values: #default_values,
                    disabled: #disabled,
                    kind: #kind,
                    max_values: #max_values,
                    min_values: #min_values,
                    options: #options,
                    placeholder: #placeholder,
                    required: #required,
                }
            )
        }
    }

    fn generate_text_input(&self, properties: Option<&PropertyBlock>) -> TokenStream2 {
        let id = properties
            .and_then(|p| p.get("id"))
            .unwrap_or_else(|| quote! { None });

        let custom_id = properties
            .and_then(|p| p.get("custom_id"))
            .map(|c| quote! { (#c).into() })
            .unwrap_or_else(|| quote! { String::new() });

        let style = properties
            .and_then(|p| p.get("style"))
            .map(resolve_text_input_style)
            .unwrap_or_else(
                || quote! { ::twilight_model::channel::message::component::TextInputStyle::Short },
            );

        let max_length = properties
            .and_then(|p| p.get("max_length"))
            .map(|m| quote! { Some(#m) })
            .unwrap_or_else(|| quote! { None });

        let min_length = properties
            .and_then(|p| p.get("min_length"))
            .map(|m| quote! { Some(#m) })
            .unwrap_or_else(|| quote! { None });

        let placeholder = properties
            .and_then(|p| p.get("placeholder"))
            .map(|ph| quote! { Some((#ph).into()) })
            .unwrap_or_else(|| quote! { None });

        let required = properties
            .and_then(|p| p.get("required"))
            .map(|r| quote! { Some(#r) })
            .unwrap_or_else(|| quote! { None });

        let value = properties
            .and_then(|p| p.get("value"))
            .map(|v| quote! { Some((#v).into()) })
            .unwrap_or_else(|| quote! { None });

        quote! {
            ::twilight_model::channel::message::Component::TextInput(
                ::twilight_model::channel::message::component::TextInput {
                    id: #id,
                    custom_id: #custom_id,
                    #[allow(deprecated)]
                    label: None,
                    max_length: #max_length,
                    min_length: #min_length,
                    placeholder: #placeholder,
                    required: #required,
                    style: #style,
                    value: #value,
                }
            )
        }
    }

    fn generate_media_item(
        &self,
        properties: Option<&PropertyBlock>,
        shorthand: Option<&ShorthandContent>,
    ) -> TokenStream2 {
        let url = shorthand
            .map(|s| s.to_token_stream())
            .or_else(|| properties.and_then(|p| p.get("url")))
            .unwrap_or_else(|| quote! { "" });

        let description = properties
            .and_then(|p| p.get("description").or_else(|| p.get("alt")))
            .map(|d| quote! { Some((#d).into()) })
            .unwrap_or_else(|| quote! { None });

        // spoiler is Option<bool>
        let spoiler = properties
            .and_then(|p| p.get("spoiler"))
            .map(|s| quote! { Some(#s) })
            .unwrap_or_else(|| quote! { None });

        quote! {
            ::twilight_model::channel::message::component::MediaGalleryItem {
                description: #description,
                media: ::twilight_model::channel::message::component::UnfurledMediaItem {
                    url: (#url).into(),
                    proxy_url: None,
                    height: None,
                    width: None,
                    content_type: None,
                },
                spoiler: #spoiler,
            }
        }
    }

    fn generate_select_option(
        &self,
        properties: Option<&PropertyBlock>,
        shorthand: Option<&ShorthandContent>,
    ) -> TokenStream2 {
        let label = shorthand
            .map(|s| s.to_token_stream())
            .or_else(|| properties.and_then(|p| p.get("label")))
            .map(|l| quote! { (#l).into() })
            .unwrap_or_else(|| quote! { String::new() });

        let value = properties
            .and_then(|p| p.get("value"))
            .map(|v| quote! { (#v).into() })
            .unwrap_or_else(|| quote! { String::new() });

        let description = properties
            .and_then(|p| p.get("description"))
            .map(|d| quote! { Some((#d).into()) })
            .unwrap_or_else(|| quote! { None });

        let emoji = properties
            .and_then(|p| p.get("emoji"))
            .unwrap_or_else(|| quote! { None });

        let default = properties
            .and_then(|p| p.get("default"))
            .unwrap_or_else(|| quote! { false });

        quote! {
            ::twilight_model::channel::message::component::SelectMenuOption {
                default: #default,
                description: #description,
                emoji: #emoji,
                label: #label,
                value: #value,
            }
        }
    }
}

// =============================================================================
// Property Parsing
// =============================================================================

/// A block of key-value properties: `{ key: value, ... }`
struct PropertyBlock {
    properties: Punctuated<Property, Token![,]>,
}

impl Parse for PropertyBlock {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        braced!(content in input);
        Ok(PropertyBlock {
            properties: Punctuated::parse_terminated(&content)?,
        })
    }
}

impl PropertyBlock {
    fn get(&self, name: &str) -> Option<TokenStream2> {
        self.properties
            .iter()
            .find(|p| p.key == name)
            .map(|p| p.value.to_token_stream())
    }
}

/// A single property: `key: value`
struct Property {
    key: Ident,
    _colon: Token![:],
    value: PropertyValue,
}

impl Parse for Property {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Property {
            key: input.parse()?,
            _colon: input.parse()?,
            value: input.parse()?,
        })
    }
}

/// The value side of a property - can be an expression or nested component
enum PropertyValue {
    Expr(Expr),
    Component(Box<ComponentNode>),
}

impl Parse for PropertyValue {
    fn parse(input: ParseStream) -> Result<Self> {
        // Check if this looks like a component (capitalized identifier followed by (, {, or [)
        if input.peek(Ident) {
            let fork = input.fork();
            let ident: Ident = fork.parse()?;
            let ident_str = ident.to_string();
            let first_char = ident_str.chars().next().unwrap_or('a');

            // Exclude common Rust types/constructors that start with uppercase
            const RUST_BUILTINS: &[&str] = &[
                "Some", "None", "Ok", "Err", "Box", "Vec", "String", "Arc", "Rc", "Cell",
                "RefCell", "Mutex", "RwLock", "Option", "Result",
            ];

            // Component types start with uppercase, are followed by parens/braces/brackets,
            // and are not common Rust builtins
            if first_char.is_uppercase()
                && !RUST_BUILTINS.contains(&ident_str.as_str())
                && (fork.peek(token::Paren) || fork.peek(token::Brace) || fork.peek(token::Bracket))
            {
                return Ok(PropertyValue::Component(Box::new(input.parse()?)));
            }
        }

        // Otherwise parse as a regular expression
        Ok(PropertyValue::Expr(input.parse()?))
    }
}

impl ToTokens for PropertyValue {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            PropertyValue::Expr(e) => e.to_tokens(tokens),
            PropertyValue::Component(c) => c.to_tokens(tokens),
        }
    }
}

// =============================================================================
// Child Block Parsing
// =============================================================================

/// A block of child components: `[ child, ... ]`
struct ChildBlock {
    children: Punctuated<ComponentNode, Token![,]>,
}

impl Parse for ChildBlock {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        bracketed!(content in input);
        Ok(ChildBlock {
            children: Punctuated::parse_terminated(&content)?,
        })
    }
}

// =============================================================================
// Shorthand Content Parsing
// =============================================================================

/// Shorthand content in parentheses: `(content)`
struct ShorthandContent {
    expr: Expr,
}

impl Parse for ShorthandContent {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(ShorthandContent {
            expr: input.parse()?,
        })
    }
}

impl ToTokens for ShorthandContent {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.expr.to_tokens(tokens);
    }
}

// =============================================================================
// Helper Functions for Style Resolution
// =============================================================================

fn resolve_button_style(style: TokenStream2) -> TokenStream2 {
    // If it's a simple identifier like `Primary`, expand to full path
    let style_str = style.to_string();
    match style_str.as_str() {
        "Primary" => quote! { ::twilight_model::channel::message::component::ButtonStyle::Primary },
        "Secondary" => {
            quote! { ::twilight_model::channel::message::component::ButtonStyle::Secondary }
        }
        "Success" => quote! { ::twilight_model::channel::message::component::ButtonStyle::Success },
        "Danger" => quote! { ::twilight_model::channel::message::component::ButtonStyle::Danger },
        "Link" => quote! { ::twilight_model::channel::message::component::ButtonStyle::Link },
        "Premium" => quote! { ::twilight_model::channel::message::component::ButtonStyle::Premium },
        _ => style, // Pass through as-is if it's a more complex expression
    }
}

fn resolve_select_menu_type(kind: TokenStream2) -> TokenStream2 {
    let kind_str = kind.to_string();
    match kind_str.as_str() {
        "Text" | "String" => {
            quote! { ::twilight_model::channel::message::component::SelectMenuType::Text }
        }
        "User" => quote! { ::twilight_model::channel::message::component::SelectMenuType::User },
        "Role" => quote! { ::twilight_model::channel::message::component::SelectMenuType::Role },
        "Mentionable" => {
            quote! { ::twilight_model::channel::message::component::SelectMenuType::Mentionable }
        }
        "Channel" => {
            quote! { ::twilight_model::channel::message::component::SelectMenuType::Channel }
        }
        _ => kind,
    }
}

fn resolve_text_input_style(style: TokenStream2) -> TokenStream2 {
    let style_str = style.to_string();
    match style_str.as_str() {
        "Short" => quote! { ::twilight_model::channel::message::component::TextInputStyle::Short },
        "Paragraph" => {
            quote! { ::twilight_model::channel::message::component::TextInputStyle::Paragraph }
        }
        _ => style,
    }
}
