use twilight_components::{component, components};
use twilight_model::channel::message::{
    Component,
    component::{ButtonStyle, SelectMenuType, SeparatorSpacingSize, TextInputStyle},
};

// =============================================================================
// TextDisplay Tests
// =============================================================================

#[test]
fn test_text_display_shorthand() {
    let text = component!(Text("Hello, World!"));

    match text {
        Component::TextDisplay(td) => {
            assert_eq!(td.content, "Hello, World!");
            assert!(td.id.is_none());
        }
        _ => panic!("Expected TextDisplay component"),
    }
}

#[test]
fn test_text_display_with_content_property() {
    let text = component!(Text { content: "Hello" });

    match text {
        Component::TextDisplay(td) => {
            assert_eq!(td.content, "Hello");
            assert!(td.id.is_none());
        }
        _ => panic!("Expected TextDisplay component"),
    }
}

#[test]
fn test_text_display_with_id() {
    let text = component!(Text {
        content: "Hello",
        id: Some(42)
    });

    match text {
        Component::TextDisplay(td) => {
            assert_eq!(td.content, "Hello");
            assert_eq!(td.id, Some(42));
        }
        _ => panic!("Expected TextDisplay component"),
    }
}

#[test]
fn test_text_display_alias() {
    let text = component!(TextDisplay("Using full name"));

    match text {
        Component::TextDisplay(td) => {
            assert_eq!(td.content, "Using full name");
        }
        _ => panic!("Expected TextDisplay component"),
    }
}

#[test]
fn test_text_display_with_expression() {
    let name = "Alice";
    let text = component!(Text(format!("Hello, {}!", name)));

    match text {
        Component::TextDisplay(td) => {
            assert_eq!(td.content, "Hello, Alice!");
        }
        _ => panic!("Expected TextDisplay component"),
    }
}

#[test]
fn test_text_display_empty() {
    // Test default when no content provided
    let text = component!(Text {});

    match text {
        Component::TextDisplay(td) => {
            assert_eq!(td.content, "");
        }
        _ => panic!("Expected TextDisplay component"),
    }
}

// =============================================================================
// Container Tests
// =============================================================================

#[test]
fn test_container_empty() {
    let container = component!(Container []);

    match container {
        Component::Container(c) => {
            assert!(c.components.is_empty());
            assert!(c.id.is_none());
            assert!(c.accent_color.is_none());
            assert!(c.spoiler.is_none());
        }
        _ => panic!("Expected Container component"),
    }
}

#[test]
fn test_container_with_color() {
    // User provides raw value, macro wraps in Some(Some(...))
    let container = component!(Container { color: 0xFF0000 } []);

    match container {
        Component::Container(c) => {
            assert_eq!(c.accent_color, Some(Some(0xFF0000)));
        }
        _ => panic!("Expected Container component"),
    }
}

#[test]
fn test_container_with_accent_color_alias() {
    let container = component!(Container { accent_color: 0x00FF00 } []);

    match container {
        Component::Container(c) => {
            assert_eq!(c.accent_color, Some(Some(0x00FF00)));
        }
        _ => panic!("Expected Container component"),
    }
}

#[test]
fn test_container_with_children() {
    let container = component!(Container [
        Text("First"),
        Text("Second"),
        Text("Third"),
    ]);

    match container {
        Component::Container(c) => {
            assert_eq!(c.components.len(), 3);
        }
        _ => panic!("Expected Container component"),
    }
}

#[test]
fn test_container_with_spoiler() {
    let container = component!(Container { spoiler: true } []);

    match container {
        Component::Container(c) => {
            assert_eq!(c.spoiler, Some(true));
        }
        _ => panic!("Expected Container component"),
    }
}

#[test]
fn test_container_with_spoiler_false() {
    let container = component!(Container { spoiler: false } []);

    match container {
        Component::Container(c) => {
            assert_eq!(c.spoiler, Some(false));
        }
        _ => panic!("Expected Container component"),
    }
}

#[test]
fn test_container_all_properties() {
    let container = component!(Container { id: Some(1), color: 0x123456, spoiler: true } [
        Text("Content"),
    ]);

    match container {
        Component::Container(c) => {
            assert_eq!(c.id, Some(1));
            assert_eq!(c.accent_color, Some(Some(0x123456)));
            assert_eq!(c.spoiler, Some(true));
            assert_eq!(c.components.len(), 1);
        }
        _ => panic!("Expected Container component"),
    }
}

#[test]
fn test_container_no_properties_with_children() {
    let container = component!(Container [
        Text("Child"),
    ]);

    match container {
        Component::Container(c) => {
            assert!(c.id.is_none());
            assert!(c.accent_color.is_none());
            assert!(c.spoiler.is_none());
            assert_eq!(c.components.len(), 1);
        }
        _ => panic!("Expected Container component"),
    }
}

// =============================================================================
// Separator Tests
// =============================================================================

#[test]
fn test_separator_default() {
    let sep = component!(Separator);

    match sep {
        Component::Separator(s) => {
            // Default: divider = Some(true), spacing = None
            assert_eq!(s.divider, Some(true));
            assert!(s.spacing.is_none());
            assert!(s.id.is_none());
        }
        _ => panic!("Expected Separator component"),
    }
}

#[test]
fn test_separator_alias() {
    let sep = component!(Sep);

    match sep {
        Component::Separator(s) => {
            assert_eq!(s.divider, Some(true));
        }
        _ => panic!("Expected Separator component"),
    }
}

#[test]
fn test_separator_no_divider() {
    let sep = component!(Separator { divider: false });

    match sep {
        Component::Separator(s) => {
            assert_eq!(s.divider, Some(false));
        }
        _ => panic!("Expected Separator component"),
    }
}

#[test]
fn test_separator_with_spacing_large() {
    let sep = component!(Separator {
        spacing: SeparatorSpacingSize::Large
    });

    match sep {
        Component::Separator(s) => {
            assert_eq!(s.spacing, Some(SeparatorSpacingSize::Large));
        }
        _ => panic!("Expected Separator component"),
    }
}

#[test]
fn test_separator_with_spacing_small() {
    let sep = component!(Separator {
        spacing: SeparatorSpacingSize::Small
    });

    match sep {
        Component::Separator(s) => {
            assert_eq!(s.spacing, Some(SeparatorSpacingSize::Small));
        }
        _ => panic!("Expected Separator component"),
    }
}

#[test]
fn test_separator_all_properties() {
    let sep = component!(Separator {
        id: Some(5),
        divider: false,
        spacing: SeparatorSpacingSize::Large
    });

    match sep {
        Component::Separator(s) => {
            assert_eq!(s.id, Some(5));
            assert_eq!(s.divider, Some(false));
            assert_eq!(s.spacing, Some(SeparatorSpacingSize::Large));
        }
        _ => panic!("Expected Separator component"),
    }
}

// =============================================================================
// ActionRow Tests
// =============================================================================

#[test]
fn test_action_row_empty() {
    let row = component!(ActionRow []);

    match row {
        Component::ActionRow(ar) => {
            assert!(ar.components.is_empty());
            assert!(ar.id.is_none());
        }
        _ => panic!("Expected ActionRow component"),
    }
}

#[test]
fn test_action_row_alias() {
    let row = component!(Row []);

    match row {
        Component::ActionRow(ar) => {
            assert!(ar.components.is_empty());
        }
        _ => panic!("Expected ActionRow component"),
    }
}

#[test]
fn test_action_row_with_id() {
    let row = component!(ActionRow { id: Some(10) } []);

    match row {
        Component::ActionRow(ar) => {
            assert_eq!(ar.id, Some(10));
        }
        _ => panic!("Expected ActionRow component"),
    }
}

#[test]
fn test_action_row_with_buttons() {
    let row = component!(Row [
        Button { style: Primary, label: "One", custom_id: "btn1" },
        Button { style: Secondary, label: "Two", custom_id: "btn2" },
    ]);

    match row {
        Component::ActionRow(ar) => {
            assert_eq!(ar.components.len(), 2);
        }
        _ => panic!("Expected ActionRow component"),
    }
}

// =============================================================================
// Button Tests
// =============================================================================

#[test]
fn test_button_minimal() {
    // Minimal button with just custom_id
    let btn = component!(Button { custom_id: "btn" });

    match btn {
        Component::Button(b) => {
            assert_eq!(b.style, ButtonStyle::Primary); // default
            assert_eq!(b.custom_id, Some("btn".into()));
            assert!(b.label.is_none());
            assert!(!b.disabled);
        }
        _ => panic!("Expected Button component"),
    }
}

#[test]
fn test_button_with_label() {
    let btn = component!(Button {
        style: Primary,
        label: "Click Me",
        custom_id: "btn"
    });

    match btn {
        Component::Button(b) => {
            assert_eq!(b.style, ButtonStyle::Primary);
            assert_eq!(b.label, Some("Click Me".into()));
            assert_eq!(b.custom_id, Some("btn".into()));
            assert!(!b.disabled);
        }
        _ => panic!("Expected Button component"),
    }
}

#[test]
fn test_button_all_styles() {
    let primary = component!(Button {
        style: Primary,
        custom_id: "p"
    });
    let secondary = component!(Button {
        style: Secondary,
        custom_id: "s"
    });
    let success = component!(Button {
        style: Success,
        custom_id: "su"
    });
    let danger = component!(Button {
        style: Danger,
        custom_id: "d"
    });
    let link = component!(Button {
        style: Link,
        url: "https://example.com"
    });

    match primary {
        Component::Button(b) => assert_eq!(b.style, ButtonStyle::Primary),
        _ => panic!("Expected Button"),
    }
    match secondary {
        Component::Button(b) => assert_eq!(b.style, ButtonStyle::Secondary),
        _ => panic!("Expected Button"),
    }
    match success {
        Component::Button(b) => assert_eq!(b.style, ButtonStyle::Success),
        _ => panic!("Expected Button"),
    }
    match danger {
        Component::Button(b) => assert_eq!(b.style, ButtonStyle::Danger),
        _ => panic!("Expected Button"),
    }
    match link {
        Component::Button(b) => {
            assert_eq!(b.style, ButtonStyle::Link);
            assert_eq!(b.url, Some("https://example.com".into()));
        }
        _ => panic!("Expected Button"),
    }
}

#[test]
fn test_button_alias() {
    let btn = component!(Btn {
        style: Primary,
        custom_id: "b"
    });

    match btn {
        Component::Button(_) => {}
        _ => panic!("Expected Button component"),
    }
}

#[test]
fn test_button_shorthand_label() {
    let btn = component!(Button("Click Me") { style: Primary, custom_id: "btn" });

    match btn {
        Component::Button(b) => {
            assert_eq!(b.label, Some("Click Me".into()));
        }
        _ => panic!("Expected Button component"),
    }
}

#[test]
fn test_button_disabled() {
    let btn = component!(Button {
        style: Primary,
        custom_id: "btn",
        disabled: true
    });

    match btn {
        Component::Button(b) => {
            assert!(b.disabled);
        }
        _ => panic!("Expected Button component"),
    }
}

#[test]
fn test_button_not_disabled_default() {
    let btn = component!(Button {
        style: Primary,
        custom_id: "btn"
    });

    match btn {
        Component::Button(b) => {
            assert!(!b.disabled);
        }
        _ => panic!("Expected Button component"),
    }
}

#[test]
fn test_button_with_url_no_custom_id() {
    let btn = component!(Button {
        style: Link,
        url: "https://example.com",
        label: "Visit"
    });

    match btn {
        Component::Button(b) => {
            assert_eq!(b.style, ButtonStyle::Link);
            assert_eq!(b.url, Some("https://example.com".into()));
            assert!(b.custom_id.is_none());
        }
        _ => panic!("Expected Button component"),
    }
}

// =============================================================================
// SelectMenu Tests
// =============================================================================

#[test]
fn test_select_menu_minimal() {
    let select = component!(SelectMenu { custom_id: "menu" } []);

    match select {
        Component::SelectMenu(sm) => {
            assert_eq!(sm.custom_id, "menu");
            assert_eq!(sm.kind, SelectMenuType::Text); // default
            assert!(sm.placeholder.is_none());
            assert!(sm.min_values.is_none());
            assert!(sm.max_values.is_none());
            assert!(!sm.disabled);
        }
        _ => panic!("Expected SelectMenu component"),
    }
}

#[test]
fn test_select_menu_alias() {
    let select = component!(Select { custom_id: "sel" } []);

    match select {
        Component::SelectMenu(_) => {}
        _ => panic!("Expected SelectMenu component"),
    }
}

#[test]
fn test_select_menu_with_options() {
    let select = component!(Select { custom_id: "color" } [
        Option { label: "Red", value: "red" },
        Option { label: "Blue", value: "blue" },
    ]);

    match select {
        Component::SelectMenu(sm) => {
            let options = sm.options.unwrap();
            assert_eq!(options.len(), 2);
            assert_eq!(options[0].label, "Red");
            assert_eq!(options[0].value, "red");
        }
        _ => panic!("Expected SelectMenu component"),
    }
}

#[test]
fn test_select_menu_types() {
    let text = component!(Select { custom_id: "t", kind: Text } []);
    let user = component!(Select { custom_id: "u", kind: User } []);
    let role = component!(Select { custom_id: "r", kind: Role } []);
    let channel = component!(Select { custom_id: "c", kind: Channel } []);
    let mentionable = component!(Select { custom_id: "m", kind: Mentionable } []);

    match text {
        Component::SelectMenu(sm) => assert_eq!(sm.kind, SelectMenuType::Text),
        _ => panic!("Expected SelectMenu"),
    }
    match user {
        Component::SelectMenu(sm) => assert_eq!(sm.kind, SelectMenuType::User),
        _ => panic!("Expected SelectMenu"),
    }
    match role {
        Component::SelectMenu(sm) => assert_eq!(sm.kind, SelectMenuType::Role),
        _ => panic!("Expected SelectMenu"),
    }
    match channel {
        Component::SelectMenu(sm) => assert_eq!(sm.kind, SelectMenuType::Channel),
        _ => panic!("Expected SelectMenu"),
    }
    match mentionable {
        Component::SelectMenu(sm) => assert_eq!(sm.kind, SelectMenuType::Mentionable),
        _ => panic!("Expected SelectMenu"),
    }
}

#[test]
fn test_select_menu_with_placeholder() {
    let select = component!(Select { custom_id: "s", placeholder: "Choose..." } []);

    match select {
        Component::SelectMenu(sm) => {
            assert_eq!(sm.placeholder, Some("Choose...".into()));
        }
        _ => panic!("Expected SelectMenu component"),
    }
}

#[test]
fn test_select_menu_with_min_max_values() {
    let select = component!(Select { custom_id: "s", min_values: 1, max_values: 5 } []);

    match select {
        Component::SelectMenu(sm) => {
            assert_eq!(sm.min_values, Some(1));
            assert_eq!(sm.max_values, Some(5));
        }
        _ => panic!("Expected SelectMenu component"),
    }
}

#[test]
fn test_select_menu_disabled() {
    let select = component!(Select { custom_id: "s", disabled: true } []);

    match select {
        Component::SelectMenu(sm) => {
            assert!(sm.disabled);
        }
        _ => panic!("Expected SelectMenu component"),
    }
}

#[test]
fn test_select_option_with_description() {
    let select = component!(Select { custom_id: "s" } [
        Option { label: "Item", value: "item", description: "A description" },
    ]);

    match select {
        Component::SelectMenu(sm) => {
            let options = sm.options.unwrap();
            assert_eq!(options[0].description, Some("A description".into()));
        }
        _ => panic!("Expected SelectMenu component"),
    }
}

#[test]
fn test_select_option_default() {
    let select = component!(Select { custom_id: "s" } [
        Option { label: "Default", value: "def", default: true },
        Option { label: "Other", value: "other" },
    ]);

    match select {
        Component::SelectMenu(sm) => {
            let options = sm.options.unwrap();
            assert!(options[0].default);
            assert!(!options[1].default);
        }
        _ => panic!("Expected SelectMenu component"),
    }
}

#[test]
fn test_select_option_shorthand() {
    let select = component!(Select { custom_id: "s" } [
        Option("Quick Label") { value: "val" },
    ]);

    match select {
        Component::SelectMenu(sm) => {
            let options = sm.options.unwrap();
            assert_eq!(options[0].label, "Quick Label");
        }
        _ => panic!("Expected SelectMenu component"),
    }
}

// =============================================================================
// TextInput Tests
// =============================================================================

#[test]
fn test_text_input_minimal() {
    let input = component!(TextInput { custom_id: "input" });

    match input {
        Component::TextInput(ti) => {
            assert_eq!(ti.custom_id, "input");
            assert!(ti.label.is_none());
            assert_eq!(ti.style, TextInputStyle::Short); // default
            assert!(ti.placeholder.is_none());
            assert!(ti.min_length.is_none());
            assert!(ti.max_length.is_none());
            assert!(ti.required.is_none());
        }
        _ => panic!("Expected TextInput component"),
    }
}

#[test]
fn test_text_input_with_label() {
    let input = component!(TextInput {
        custom_id: "input",
        label: "Your Name"
    });

    match input {
        Component::TextInput(ti) => {
            assert_eq!(ti.custom_id, "input");
            assert_eq!(ti.label, Some("Your Name".into()));
        }
        _ => panic!("Expected TextInput component"),
    }
}

#[test]
fn test_text_input_alias() {
    let input = component!(Input { custom_id: "i" });

    match input {
        Component::TextInput(_) => {}
        _ => panic!("Expected TextInput component"),
    }
}

#[test]
fn test_text_input_paragraph() {
    let input = component!(Input {
        custom_id: "bio",
        style: Paragraph
    });

    match input {
        Component::TextInput(ti) => {
            assert_eq!(ti.style, TextInputStyle::Paragraph);
        }
        _ => panic!("Expected TextInput component"),
    }
}

#[test]
fn test_text_input_short_style() {
    let input = component!(Input {
        custom_id: "name",
        style: Short
    });

    match input {
        Component::TextInput(ti) => {
            assert_eq!(ti.style, TextInputStyle::Short);
        }
        _ => panic!("Expected TextInput component"),
    }
}

#[test]
fn test_text_input_with_constraints() {
    let input = component!(Input {
        custom_id: "msg",
        label: "Message",
        min_length: 10,
        max_length: 100,
        required: true,
        placeholder: "Enter message..."
    });

    match input {
        Component::TextInput(ti) => {
            assert_eq!(ti.min_length, Some(10));
            assert_eq!(ti.max_length, Some(100));
            assert_eq!(ti.required, Some(true));
            assert_eq!(ti.placeholder, Some("Enter message...".into()));
        }
        _ => panic!("Expected TextInput component"),
    }
}

#[test]
fn test_text_input_with_value() {
    let input = component!(Input {
        custom_id: "prefilled",
        value: "Default text"
    });

    match input {
        Component::TextInput(ti) => {
            assert_eq!(ti.value, Some("Default text".into()));
        }
        _ => panic!("Expected TextInput component"),
    }
}

// =============================================================================
// Section Tests (accessory is REQUIRED)
// =============================================================================

#[test]
fn test_section_with_button_accessory() {
    let section = component!(Section {
        accessory: Button { style: Link, url: "https://example.com", label: "Visit" }
    } [
        Text("Check out our site!"),
    ]);

    match section {
        Component::Section(s) => {
            // accessory is Box<Component>, always present
            assert_eq!(s.components.len(), 1);
        }
        _ => panic!("Expected Section component"),
    }
}

#[test]
fn test_section_with_thumbnail_accessory() {
    let section = component!(Section {
        accessory: Thumbnail("https://example.com/image.png")
    } [
        Text("Image description"),
    ]);

    match section {
        Component::Section(s) => {
            // accessory is present
            assert_eq!(s.components.len(), 1);
        }
        _ => panic!("Expected Section component"),
    }
}

#[test]
fn test_section_with_id_and_accessory() {
    let section = component!(Section {
        id: Some(99),
        accessory: Button { style: Primary, custom_id: "btn", label: "Click" }
    } []);

    match section {
        Component::Section(s) => {
            assert_eq!(s.id, Some(99));
        }
        _ => panic!("Expected Section component"),
    }
}

#[test]
fn test_section_multiple_text_children() {
    let section = component!(Section {
        accessory: Thumbnail("https://example.com/img.png")
    } [
        Text("Line 1"),
        Text("Line 2"),
        Text("Line 3"),
    ]);

    match section {
        Component::Section(s) => {
            assert_eq!(s.components.len(), 3);
        }
        _ => panic!("Expected Section component"),
    }
}

// =============================================================================
// MediaGallery Tests
// =============================================================================

#[test]
fn test_media_gallery_empty() {
    let gallery = component!(MediaGallery []);

    match gallery {
        Component::MediaGallery(mg) => {
            assert!(mg.items.is_empty());
            assert!(mg.id.is_none());
        }
        _ => panic!("Expected MediaGallery component"),
    }
}

#[test]
fn test_media_gallery_with_items() {
    let gallery = component!(MediaGallery [
        Item("https://example.com/image1.png"),
        Item("https://example.com/image2.png"),
    ]);

    match gallery {
        Component::MediaGallery(mg) => {
            assert_eq!(mg.items.len(), 2);
        }
        _ => panic!("Expected MediaGallery component"),
    }
}

#[test]
fn test_media_gallery_alias() {
    let gallery = component!(Gallery []);

    match gallery {
        Component::MediaGallery(_) => {}
        _ => panic!("Expected MediaGallery component"),
    }
}

#[test]
fn test_media_item_with_properties() {
    let gallery = component!(MediaGallery [
        Item { url: "https://example.com/img.png", description: "Alt text", spoiler: true },
    ]);

    match gallery {
        Component::MediaGallery(mg) => {
            assert_eq!(mg.items[0].description, Some("Alt text".into()));
            assert_eq!(mg.items[0].spoiler, Some(true));
        }
        _ => panic!("Expected MediaGallery component"),
    }
}

#[test]
fn test_media_item_shorthand() {
    let gallery = component!(MediaGallery [
        MediaItem("https://example.com/img.png"),
    ]);

    match gallery {
        Component::MediaGallery(mg) => {
            assert_eq!(mg.items.len(), 1);
            assert_eq!(mg.items[0].media.url, "https://example.com/img.png");
        }
        _ => panic!("Expected MediaGallery component"),
    }
}

#[test]
fn test_media_item_no_spoiler_default() {
    let gallery = component!(MediaGallery [
        Item("https://example.com/img.png"),
    ]);

    match gallery {
        Component::MediaGallery(mg) => {
            assert!(mg.items[0].spoiler.is_none());
        }
        _ => panic!("Expected MediaGallery component"),
    }
}

#[test]
fn test_media_gallery_with_id() {
    let gallery = component!(MediaGallery { id: Some(7) } []);

    match gallery {
        Component::MediaGallery(mg) => {
            assert_eq!(mg.id, Some(7));
        }
        _ => panic!("Expected MediaGallery component"),
    }
}

// =============================================================================
// Thumbnail Tests (returns Component::Thumbnail)
// =============================================================================

#[test]
fn test_thumbnail_shorthand() {
    let thumb = component!(Thumbnail("https://example.com/thumb.png"));

    match thumb {
        Component::Thumbnail(t) => {
            assert_eq!(t.media.url, "https://example.com/thumb.png");
            assert!(t.description.is_none());
            assert!(t.spoiler.is_none());
        }
        _ => panic!("Expected Thumbnail component"),
    }
}

#[test]
fn test_thumbnail_alias() {
    let thumb = component!(Thumb("https://example.com/t.png"));

    match thumb {
        Component::Thumbnail(t) => {
            assert_eq!(t.media.url, "https://example.com/t.png");
        }
        _ => panic!("Expected Thumbnail component"),
    }
}

#[test]
fn test_thumbnail_with_properties() {
    let thumb = component!(Thumbnail {
        url: "https://example.com/img.png",
        description: "Description text",
        spoiler: true
    });

    match thumb {
        Component::Thumbnail(t) => {
            assert_eq!(t.description, Some(Some("Description text".into())));
            assert_eq!(t.spoiler, Some(true));
        }
        _ => panic!("Expected Thumbnail component"),
    }
}

#[test]
fn test_thumbnail_no_spoiler_default() {
    let thumb = component!(Thumbnail("https://example.com/img.png"));

    match thumb {
        Component::Thumbnail(t) => {
            assert!(t.spoiler.is_none());
        }
        _ => panic!("Expected Thumbnail component"),
    }
}

// =============================================================================
// FileDisplay Tests (returns Component::File)
// =============================================================================

#[test]
fn test_file_display_shorthand() {
    let file = component!(File("https://example.com/doc.pdf"));

    match file {
        Component::File(fd) => {
            assert_eq!(fd.file.url, "https://example.com/doc.pdf");
            assert!(fd.spoiler.is_none());
        }
        _ => panic!("Expected File component"),
    }
}

#[test]
fn test_file_display_alias() {
    let file = component!(FileDisplay("https://example.com/f.txt"));

    match file {
        Component::File(_) => {}
        _ => panic!("Expected File component"),
    }
}

#[test]
fn test_file_display_with_spoiler() {
    let file = component!(File {
        url: "https://example.com/file.zip",
        spoiler: true
    });

    match file {
        Component::File(fd) => {
            assert_eq!(fd.spoiler, Some(true));
        }
        _ => panic!("Expected File component"),
    }
}

#[test]
fn test_file_display_no_spoiler_default() {
    let file = component!(File {
        url: "https://example.com/file.zip"
    });

    match file {
        Component::File(fd) => {
            assert!(fd.spoiler.is_none());
        }
        _ => panic!("Expected File component"),
    }
}

// =============================================================================
// components! Macro Tests
// =============================================================================

#[test]
fn test_components_macro_single() {
    let comps = components!(Text("Hello"));

    assert_eq!(comps.len(), 1);
}

#[test]
fn test_components_macro_multiple() {
    let comps = components!(
        Container [
            Text("First container"),
        ],
        Container [
            Text("Second container"),
        ],
    );

    assert_eq!(comps.len(), 2);
}

#[test]
fn test_components_macro_mixed() {
    let comps = components!(
        Container { color: 0xFF0000 } [
            Text("Header"),
            Separator,
            Row [
                Button { style: Primary, label: "OK", custom_id: "ok" },
            ],
        ],
        Row [
            Button { style: Link, label: "Help", url: "https://example.com" },
        ],
    );

    assert_eq!(comps.len(), 2);
}

// =============================================================================
// Complex Integration Tests
// =============================================================================

#[test]
fn test_nested_structure() {
    let ui = component!(Container { color: 0x5865F2 } [
        Text("# Welcome"),
        Separator { spacing: SeparatorSpacingSize::Large },
        Section {
            accessory: Thumbnail("https://example.com/avatar.png")
        } [
            Text("**Profile**"),
            Text("Level 42"),
        ],
        Separator,
        Row [
            Button { style: Primary, label: "Stats", custom_id: "stats" },
            Button { style: Secondary, label: "Settings", custom_id: "settings" },
        ],
    ]);

    match ui {
        Component::Container(c) => {
            assert_eq!(c.components.len(), 5);
            assert_eq!(c.accent_color, Some(Some(0x5865F2)));
        }
        _ => panic!("Expected Container"),
    }
}

#[test]
fn test_expression_in_values() {
    let color = 0xFF0000u32;
    let label = String::from("Dynamic");
    let is_disabled = true;

    let btn = component!(Button {
        style: Primary,
        label: label.clone(),
        custom_id: "btn",
        disabled: is_disabled
    });

    match btn {
        Component::Button(b) => {
            assert_eq!(b.label, Some("Dynamic".into()));
            assert!(b.disabled);
        }
        _ => panic!("Expected Button"),
    }

    let container = component!(Container { color: color } []);

    match container {
        Component::Container(c) => {
            assert_eq!(c.accent_color, Some(Some(0xFF0000)));
        }
        _ => panic!("Expected Container"),
    }
}

#[test]
fn test_conditional_expression() {
    let premium = true;
    let container = component!(Container {
        color: if premium { 0xFFD700 } else { 0x808080 }
    } []);

    match container {
        Component::Container(c) => {
            assert_eq!(c.accent_color, Some(Some(0xFFD700)));
        }
        _ => panic!("Expected Container"),
    }

    let not_premium = false;
    let container2 = component!(Container {
        color: if not_premium { 0xFFD700 } else { 0x808080 }
    } []);

    match container2 {
        Component::Container(c) => {
            assert_eq!(c.accent_color, Some(Some(0x808080)));
        }
        _ => panic!("Expected Container"),
    }
}

#[test]
fn test_format_macro_in_text() {
    let username = "Alice";
    let level = 42;

    let text = component!(Text(format!("Welcome, {}! Level: {}", username, level)));

    match text {
        Component::TextDisplay(td) => {
            assert_eq!(td.content, "Welcome, Alice! Level: 42");
        }
        _ => panic!("Expected TextDisplay"),
    }
}

// =============================================================================
// Modal Form Test
// =============================================================================

#[test]
fn test_modal_form_layout() {
    let form = components!(
        Row[Input {
            custom_id: "name",
            label: "Your Name",
            style: Short,
            placeholder: "Enter name...",
            required: true
        }],
        Row[Input {
            custom_id: "bio",
            label: "About You",
            style: Paragraph,
            min_length: 10,
            max_length: 500
        }]
    );

    assert_eq!(form.len(), 2);

    match &form[0] {
        Component::ActionRow(ar) => {
            assert_eq!(ar.components.len(), 1);
            match &ar.components[0] {
                Component::TextInput(ti) => {
                    assert_eq!(ti.custom_id, "name");
                    assert_eq!(ti.style, TextInputStyle::Short);
                    assert_eq!(ti.required, Some(true));
                }
                _ => panic!("Expected TextInput"),
            }
        }
        _ => panic!("Expected ActionRow"),
    }
}

// =============================================================================
// Select Menu Complete Example
// =============================================================================

#[test]
fn test_complete_select_menu() {
    let menu = component!(Row [
        Select {
            custom_id: "fruit_select",
            placeholder: "Pick a fruit",
            min_values: 1,
            max_values: 3
        } [
            Option { label: "Apple", value: "apple", description: "A red fruit" },
            Option { label: "Banana", value: "banana", description: "A yellow fruit" },
            Option { label: "Cherry", value: "cherry", default: true },
        ]
    ]);

    match menu {
        Component::ActionRow(ar) => match &ar.components[0] {
            Component::SelectMenu(sm) => {
                assert_eq!(sm.placeholder, Some("Pick a fruit".into()));
                assert_eq!(sm.min_values, Some(1));
                assert_eq!(sm.max_values, Some(3));

                let options = sm.options.as_ref().unwrap();
                assert_eq!(options.len(), 3);
                assert!(options[2].default);
            }
            _ => panic!("Expected SelectMenu"),
        },
        _ => panic!("Expected ActionRow"),
    }
}

// =============================================================================
// Default Value Tests (verifying omitted optional fields)
// =============================================================================

#[test]
fn test_button_defaults() {
    // Only provide required fields, verify all optionals have correct defaults
    let btn = component!(Button { custom_id: "test" });

    match btn {
        Component::Button(b) => {
            assert_eq!(b.style, ButtonStyle::Primary); // default style
            assert!(b.label.is_none());
            assert!(b.url.is_none());
            assert!(b.emoji.is_none());
            assert!(b.sku_id.is_none());
            assert!(!b.disabled); // default false
            assert!(b.id.is_none());
        }
        _ => panic!("Expected Button"),
    }
}

#[test]
fn test_select_menu_defaults() {
    let select = component!(Select { custom_id: "test" } []);

    match select {
        Component::SelectMenu(sm) => {
            assert_eq!(sm.kind, SelectMenuType::Text); // default
            assert!(sm.placeholder.is_none());
            assert!(sm.min_values.is_none());
            assert!(sm.max_values.is_none());
            assert!(!sm.disabled);
            assert!(sm.channel_types.is_none());
            assert!(sm.default_values.is_none());
            assert!(sm.required.is_none());
        }
        _ => panic!("Expected SelectMenu"),
    }
}

#[test]
fn test_text_input_defaults() {
    let input = component!(Input { custom_id: "test" });

    match input {
        Component::TextInput(ti) => {
            assert_eq!(ti.style, TextInputStyle::Short); // default
            assert!(ti.label.is_none());
            assert!(ti.placeholder.is_none());
            assert!(ti.min_length.is_none());
            assert!(ti.max_length.is_none());
            assert!(ti.required.is_none());
            assert!(ti.value.is_none());
        }
        _ => panic!("Expected TextInput"),
    }
}

#[test]
fn test_container_defaults() {
    let container = component!(Container []);

    match container {
        Component::Container(c) => {
            assert!(c.id.is_none());
            assert!(c.accent_color.is_none());
            assert!(c.spoiler.is_none());
        }
        _ => panic!("Expected Container"),
    }
}

#[test]
fn test_separator_defaults() {
    let sep = component!(Separator);

    match sep {
        Component::Separator(s) => {
            assert!(s.id.is_none());
            assert_eq!(s.divider, Some(true)); // default per design
            assert!(s.spacing.is_none());
        }
        _ => panic!("Expected Separator"),
    }
}

#[test]
fn test_select_option_defaults() {
    let select = component!(Select { custom_id: "s" } [
        Option { label: "Test", value: "test" },
    ]);

    match select {
        Component::SelectMenu(sm) => {
            let options = sm.options.unwrap();
            assert!(!options[0].default); // default false
            assert!(options[0].description.is_none());
            assert!(options[0].emoji.is_none());
        }
        _ => panic!("Expected SelectMenu"),
    }
}

#[test]
fn test_thumbnail_defaults() {
    let thumb = component!(Thumbnail("https://example.com/img.png"));

    match thumb {
        Component::Thumbnail(t) => {
            assert!(t.id.is_none());
            assert!(t.description.is_none());
            assert!(t.spoiler.is_none());
        }
        _ => panic!("Expected Thumbnail"),
    }
}

#[test]
fn test_file_defaults() {
    let file = component!(File("https://example.com/file.txt"));

    match file {
        Component::File(f) => {
            assert!(f.id.is_none());
            assert!(f.spoiler.is_none());
        }
        _ => panic!("Expected File"),
    }
}
