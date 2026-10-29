use super::*;
use crate::ComponentList;

fn run(src: &str, root: Root) -> Vec<String> {
    let list: ComponentList = syn::parse_str(src).expect("test input parses");
    let nodes: Vec<&ComponentNode> = list.components.iter().collect();
    match validate(&nodes, root) {
        Ok(()) => Vec::new(),
        Err(err) => messages(err),
    }
}

/// Valid as a `component!` fragment.
#[track_caller]
fn ok(src: &str) {
    let errors = run(src, Root::Single);
    assert!(
        errors.is_empty(),
        "expected no errors for `{src}`, got {errors:?}"
    );
}

/// Valid as a full `components!` list.
#[track_caller]
fn ok_list(src: &str) {
    let errors = run(src, Root::List);
    assert!(
        errors.is_empty(),
        "expected no errors for `{src}`, got {errors:?}"
    );
}

#[track_caller]
fn rejects(src: &str, needle: &str) {
    assert_errors(run(src, Root::Single), src, needle);
}

#[track_caller]
fn rejects_list(src: &str, needle: &str) {
    assert_errors(run(src, Root::List), src, needle);
}

#[track_caller]
fn assert_errors(errors: Vec<String>, src: &str, needle: &str) {
    assert!(
        errors.iter().any(|e| e.contains(needle)),
        "expected an error containing `{needle}` for `{src}`, got {errors:?}"
    );
}

fn long(n: usize) -> String {
    "a".repeat(n)
}

// =============================================================================
// Shape
// =============================================================================

#[test]
fn unknown_field() {
    rejects(
        r#"Button { custom_i: "a" }"#,
        "unknown field `custom_i` for `Button`",
    );
}

#[test]
fn duplicate_field_and_alias() {
    rejects(
        r#"Button { custom_id: "a", custom_id: "b" }"#,
        "field `custom_id` is given more than once",
    );
    rejects(
        r#"Container { color: 1, accent_color: 2 } [Text("a")]"#,
        "field `accent_color` is given more than once",
    );
}

#[test]
fn shorthand_and_field_together() {
    rejects(
        r#"Text("a") { content: "b" }"#,
        "already given in parentheses",
    );
}

#[test]
fn shorthand_on_kind_without_one() {
    rejects(
        r#"Separator("a")"#,
        "`Separator` does not take a value in parentheses",
    );
}

#[test]
fn children_on_leaf_kind() {
    rejects(
        r#"Button { custom_id: "a" } [Text("x")]"#,
        "`Button` does not take child components",
    );
}

#[test]
fn errors_are_collected() {
    let errors = run(r#"Button { custom_i: "a", url: "x" }"#, Root::Single);
    assert!(errors.len() >= 2, "{errors:?}");
}

// =============================================================================
// Uniqueness
// =============================================================================

#[test]
fn duplicate_custom_id() {
    ok(r#"Row [Button { custom_id: "a" }, Button { custom_id: "b" }]"#);
    rejects(
        r#"Row [Button { custom_id: "a" }, Button { custom_id: "a" }]"#,
        "`custom_id` \"a\" is used more than once",
    );
}

#[test]
fn duplicate_id() {
    ok(r#"Container { id: 1 } [Text { content: "a", id: 2 }]"#);
    rejects(
        r#"Container { id: 1 } [Text { content: "a", id: Some(1) }]"#,
        "`id` 1 is used more than once",
    );
}

// =============================================================================
// Context
// =============================================================================

#[test]
fn mixed_context() {
    rejects_list(
        r#"Container [Text("a")], Label("x") [Input { custom_id: "i" }]"#,
        "`Label` can only be used in modals, but this invocation also contains `Container`",
    );
}

#[test]
fn mixed_context_has_no_root_noise() {
    let errors = run(r#"Row [Input { custom_id: "i" }]"#, Root::List);
    assert!(
        !errors.iter().any(|e| e.contains("top-level")),
        "{errors:?}"
    );
}

#[test]
fn message_roots() {
    ok_list(r#"Text("a"), Separator, Row [Button { custom_id: "b" }]"#);
    rejects_list(
        r#"Text("a"), Button { custom_id: "b" }"#,
        "`Button` cannot be a top-level component in a message",
    );
}

#[test]
fn modal_roots() {
    ok_list(r#"Text("a"), Label("x") [Input { custom_id: "i" }]"#);
    rejects_list(
        r#"Input { custom_id: "i" }"#,
        "`TextInput` cannot be a top-level component in a modal; put it in a Label",
    );
}

#[test]
fn select_menu_cannot_be_a_root() {
    rejects_list(
        r#"Select { custom_id: "s", kind: User }"#,
        "`SelectMenu` cannot be a top-level component",
    );
    // A fragment is fine.
    ok(r#"Select { custom_id: "s", kind: User }"#);
}

#[test]
fn modal_top_level_limit() {
    let label = |i: usize| format!(r#"Label("x") [Input {{ custom_id: "i{i}" }}]"#);
    let five: Vec<String> = (0..5).map(label).collect();
    ok_list(&five.join(", "));
    let six: Vec<String> = (0..6).map(label).collect();
    rejects_list(
        &six.join(", "),
        "a modal can have at most 5 top-level components",
    );
}

#[test]
fn message_component_limit() {
    // Each row is 2 components (the row and its button).
    let row = |i: usize| format!(r#"Row [Button {{ custom_id: "b{i}" }}]"#);
    let twenty: Vec<String> = (0..20).map(row).collect();
    ok_list(&twenty.join(", "));
    let twenty_one: Vec<String> = (0..21).map(row).collect();
    rejects_list(&twenty_one.join(", "), "at most 40 components in total");
}

#[test]
fn modal_disabled_select() {
    ok(r#"Row [Select { custom_id: "s", kind: User, disabled: true }]"#);
    rejects(
        r#"Label("x") [Select { custom_id: "s", kind: User, disabled: true }]"#,
        "components in a modal cannot be disabled",
    );
}

#[test]
fn modal_min_values_zero() {
    ok(r#"Row [Select { custom_id: "s", kind: User, min_values: 0 }]"#);
    ok(r#"Label("x") [Select { custom_id: "s", kind: User, min_values: 0, required: false }]"#);
    ok(r#"Label("x") [Select { custom_id: "s", kind: User, min_values: 0, required: flag }]"#);
    rejects(
        r#"Label("x") [Select { custom_id: "s", kind: User, min_values: 0 }]"#,
        "`min_values` can only be 0 when `required: false`",
    );
    rejects(
        r#"Upload { custom_id: "u", min_values: 0, required: true }"#,
        "`min_values` can only be 0 when `required: false`",
    );
}

// =============================================================================
// Layout
// =============================================================================

#[test]
fn action_row_children() {
    ok(r#"Row [Select { custom_id: "s", kind: Role }]"#);
    rejects("Row []", "`ActionRow` needs 1 to 5");
    rejects(
        r#"Row [Button { custom_id: "a" }, Select { custom_id: "s", kind: Role }]"#,
        "cannot also contain buttons",
    );
    rejects(r#"Row [Text("a")]"#, "`ActionRow` cannot contain `Text`");
}

#[test]
fn action_row_button_limit() {
    let buttons = |n: usize| {
        (0..n)
            .map(|i| format!(r#"Button {{ custom_id: "b{i}" }}"#))
            .collect::<Vec<_>>()
            .join(", ")
    };
    ok(&format!("Row [{}]", buttons(5)));
    rejects(&format!("Row [{}]", buttons(6)), "at most 5 buttons");
}

#[test]
fn section_children_and_accessory() {
    ok(r#"Section { accessory: Thumbnail("https://x") } [Text("a"), Text("b"), Text("c")]"#);
    ok(r#"Section { accessory: my_accessory } [Text("a")]"#);
    rejects(
        r#"Section [Text("a")]"#,
        "`Section` requires an `accessory`",
    );
    rejects(
        r#"Section { accessory: Thumbnail("https://x") } []"#,
        "`Section` needs 1 to 3 `Text` children",
    );
    rejects(
        r#"Section { accessory: Thumbnail("https://x") } [Text("a"), Text("b"), Text("c"), Text("d")]"#,
        "at most 3 children",
    );
    rejects(
        r#"Section { accessory: Separator {} } [Text("a")]"#,
        "must be a Button or Thumbnail",
    );
}

#[test]
fn container_children() {
    ok(r#"Container [Text("a"), Separator, File("attachment://a.txt")]"#);
    rejects(
        r#"Container [Button { custom_id: "a" }]"#,
        "`Container` cannot contain `Button`",
    );
}

#[test]
fn media_gallery_children() {
    rejects("Gallery []", "`MediaGallery` needs 1 to 10");
    let items = |n: usize| vec![r#"Item("https://x")"#; n].join(", ");
    ok(&format!("Gallery [{}]", items(10)));
    rejects(&format!("Gallery [{}]", items(11)), "at most 10 items");
}

#[test]
fn label_child() {
    ok(r#"Label { label: "x", component: Upload { custom_id: "u" } }"#);
    rejects(r#"Label("x")"#, "`Label` requires one child component");
    rejects(
        r#"Label("x") [Input { custom_id: "a" }, Input { custom_id: "b" }]"#,
        "can only contain one child component",
    );
    rejects(
        r#"Label { label: "x", component: Upload { custom_id: "u" } } [Input { custom_id: "a" }]"#,
        "not both",
    );
    rejects(r#"Label("x") [Text("a")]"#, "`Label` cannot contain `Text`");
}

// =============================================================================
// Fields
// =============================================================================

#[test]
fn text_requires_content() {
    ok(r#"Text(format!("{}", 1))"#);
    rejects("Text { id: 1 }", "`Text` requires a `content`");
}

#[test]
fn button_style_rules() {
    ok(r#"Button { style: Link, url: "https://x", label: "Go" }"#);
    ok("Button { style: Premium, sku_id: 1 }");
    ok("Button { style: ButtonStyle::Premium, sku_id: sku }");
    rejects(
        r#"Button { label: "x" }"#,
        "`Primary` buttons must have a `custom_id`",
    );
    rejects(
        r#"Button { custom_id: "a", url: "https://x" }"#,
        "only `Link` buttons can have a `url`",
    );
    rejects(
        r#"Button { style: Link }"#,
        "`Link` buttons must have a `url`",
    );
    rejects(
        r#"Button { style: Link, url: "https://x", custom_id: "a" }"#,
        "`Link` buttons cannot have a `custom_id`",
    );
    rejects(
        "Button { style: Premium }",
        "`Premium` buttons must have a `sku_id`",
    );
    rejects(
        "Button { style: Premium, sku_id: None }",
        "`Premium` buttons must have a `sku_id`",
    );
    rejects(
        r#"Button("Buy") { style: Premium, sku_id: 1 }"#,
        "`Premium` buttons cannot have a `label`",
    );
    rejects(
        r#"Button { style: Premium, sku_id: 1, emoji: "👍" }"#,
        "`Premium` buttons cannot have an `emoji`",
    );
    rejects(
        r#"Button { custom_id: "a", sku_id: 1 }"#,
        "only `Premium` buttons can have a `sku_id`",
    );
}

#[test]
fn button_runtime_style() {
    ok(r#"Button { style: style, custom_id: "a" }"#);
    rejects(
        r#"Button { style: style, custom_id: "a", url: "https://x" }"#,
        "both a `custom_id` and a `url`",
    );
}

#[test]
fn unknown_style() {
    rejects(
        r#"Button { style: Primay, custom_id: "a" }"#,
        "unknown button style `Primay`",
    );
    rejects(
        r#"Button { style: ButtonStyle::Unknown(9), custom_id: "a" }"#,
        "unknown button style `Unknown`",
    );
}

#[test]
fn button_lengths() {
    ok(&format!(
        r#"Button("{}") {{ custom_id: "{}" }}"#,
        long(80),
        long(100)
    ));
    rejects(
        &format!(r#"Button("{}") {{ custom_id: "a" }}"#, long(81)),
        "`label` can be at most 80 characters",
    );
    rejects(
        &format!(r#"Button {{ custom_id: "{}" }}"#, long(101)),
        "`custom_id` must be 1 to 100 characters",
    );
    rejects(
        r#"Button { custom_id: "" }"#,
        "`custom_id` must be 1 to 100 characters",
    );
    rejects(
        &format!(r#"Button {{ style: Link, url: "{}" }}"#, long(513)),
        "`url` can be at most 512 characters",
    );
}

#[test]
fn lengths_count_characters_not_bytes() {
    ok(&format!(
        r#"Label("{}") [Input {{ custom_id: "a" }}]"#,
        "é".repeat(45)
    ));
}

#[test]
fn select_menu_rules() {
    rejects(
        r#"Select { kind: User }"#,
        "`SelectMenu` requires a `custom_id`",
    );
    rejects(r#"Select { custom_id: "s" }"#, "needs at least one option");
    rejects(
        r#"Select { custom_id: "s", kind: User } [Option { label: "a", value: "a" }]"#,
        "only `Text` select menus have options",
    );
    rejects(
        r#"Select { custom_id: "s" } [Text("a")]"#,
        "can only contain `SelectOption`",
    );
    rejects(
        r#"Select { custom_id: "s", kind: User, channel_types: vec![] }"#,
        "only `Channel` select menus can have `channel_types`",
    );
    ok(r#"Select { custom_id: "s", kind: Channel, channel_types: vec![] }"#);
    rejects(
        r#"Select { custom_id: "s", kind: Rol }"#,
        "unknown select menu kind `Rol`",
    );
}

#[test]
fn select_menu_option_limit() {
    let options = |n: usize| {
        (0..n)
            .map(|i| format!(r#"Option {{ label: "o", value: "v{i}" }}"#))
            .collect::<Vec<_>>()
            .join(", ")
    };
    ok(&format!(r#"Select {{ custom_id: "s" }} [{}]"#, options(25)));
    rejects(
        &format!(r#"Select {{ custom_id: "s" }} [{}]"#, options(26)),
        "at most 25 options",
    );
}

#[test]
fn select_menu_values() {
    ok(r#"Select { custom_id: "s", kind: User, min_values: 0, max_values: 25 }"#);
    rejects(
        r#"Select { custom_id: "s", kind: User, max_values: 26 }"#,
        "`max_values` must be from 0 to 25",
    );
    rejects(
        r#"Select { custom_id: "s", kind: User, min_values: 3, max_values: 2 }"#,
        "`min_values` (3) is greater than `max_values` (2)",
    );
    rejects(
        &format!(
            r#"Select {{ custom_id: "s", kind: User, placeholder: "{}" }}"#,
            long(151)
        ),
        "`placeholder` can be at most 150 characters",
    );
}

#[test]
fn select_menu_default_values() {
    ok(r#"Select { custom_id: "s", kind: User, default_values: vec![a] }"#);
    ok(r#"Select { custom_id: "s", kind: User, max_values: 3, default_values: Some(vec![a, b]) }"#);
    ok(r#"Select { custom_id: "s", kind: User, default_values: values }"#);
    rejects(
        r#"Select { custom_id: "s", kind: User, default_values: vec![a, b] }"#,
        "the number of `default_values` (2) must be between",
    );
    rejects(
        r#"Select { custom_id: "s", default_values: vec![a] } [Option { label: "a", value: "a" }]"#,
        "`Text` select menus cannot have `default_values`",
    );
}

#[test]
fn select_option_rules() {
    rejects(
        r#"Select { custom_id: "s" } [Option("a")]"#,
        "`SelectOption` requires a `value`",
    );
    rejects(
        r#"Select { custom_id: "s" } [Option { value: "a" }]"#,
        "`SelectOption` requires a `label`",
    );
    rejects(
        &format!(
            r#"Select {{ custom_id: "s" }} [Option {{ label: "a", value: "{}" }}]"#,
            long(101)
        ),
        "`value` can be at most 100 characters",
    );
}

#[test]
fn text_input_rules() {
    ok(r#"Input { custom_id: "i", style: Paragraph, min_length: 0, max_length: 4000 }"#);
    rejects(
        "Input { style: Short }",
        "`TextInput` requires a `custom_id`",
    );
    rejects(
        r#"Input { custom_id: "i", max_length: 0 }"#,
        "`max_length` must be from 1 to 4000",
    );
    rejects(
        r#"Input { custom_id: "i", min_length: 4001 }"#,
        "`min_length` must be from 0 to 4000",
    );
    rejects(
        r#"Input { custom_id: "i", min_length: 10, max_length: 5 }"#,
        "`min_length` (10) is greater than `max_length` (5)",
    );
    rejects(
        r#"Input { custom_id: "i", style: Long }"#,
        "unknown text input style `Long`",
    );
    rejects(
        r#"Input { custom_id: "i", label: "x" }"#,
        "unknown field `label`",
    );
    rejects(
        &format!(
            r#"Input {{ custom_id: "i", placeholder: "{}" }}"#,
            long(101)
        ),
        "`placeholder` can be at most 100 characters",
    );
}

#[test]
fn file_upload_rules() {
    ok(r#"Upload { custom_id: "u", min_values: 1, max_values: 10 }"#);
    rejects("Upload {}", "`FileUpload` requires a `custom_id`");
    rejects(
        r#"Upload { custom_id: "u", max_values: 11 }"#,
        "`max_values` must be from 0 to 10",
    );
    rejects(
        r#"Upload { custom_id: "u", min_values: 5, max_values: 2 }"#,
        "is greater than `max_values`",
    );
}

#[test]
fn label_rules() {
    rejects(
        r#"Label { description: "d" } [Input { custom_id: "a" }]"#,
        "`Label` requires a `label`",
    );
    rejects(
        r#"Label("") [Input { custom_id: "a" }]"#,
        "`label` cannot be empty",
    );
    rejects(
        &format!(r#"Label("{}") [Input {{ custom_id: "a" }}]"#, long(46)),
        "`label` can be at most 45 characters",
    );
    rejects(
        &format!(
            r#"Label {{ label: "x", description: "{}" }} [Input {{ custom_id: "a" }}]"#,
            long(101)
        ),
        "`description` can be at most 100 characters",
    );
}

#[test]
fn media_rules() {
    ok(&format!(
        r#"Thumbnail {{ url: "https://x", alt: "{}" }}"#,
        long(1024)
    ));
    rejects(
        "Thumbnail { spoiler: true }",
        "`Thumbnail` requires a `url`",
    );
    rejects(r#"Thumbnail("")"#, "`url` cannot be empty");
    rejects(
        &format!(
            r#"Gallery [Item {{ url: "https://x", description: "{}" }}]"#,
            long(1025)
        ),
        "`description` can be at most 1024 characters",
    );
}

#[test]
fn file_rules() {
    ok(r#"File(url)"#);
    rejects(
        r#"File("https://example.com/a.pdf")"#,
        "must start with `attachment://`",
    );
    rejects("File { spoiler: true }", "`File` requires a `url`");
}

#[test]
fn container_color() {
    ok(r#"Container { color: 0xFFFFFF } [Text("a")]"#);
    rejects(
        r#"Container { color: 0x1000000 } [Text("a")]"#,
        "`accent_color` must be an RGB color",
    );
}

#[test]
fn separator_spacing() {
    ok("Separator { spacing: Large }");
    ok("Separator { spacing: SeparatorSpacingSize::Small }");
    ok("Separator { spacing: spacing }");
    rejects(
        "Separator { spacing: Huge }",
        "unknown separator spacing `Huge`",
    );
    rejects(
        "Separator { spacing: SeparatorSpacingSize::Other(3) }",
        "unknown separator spacing `Other`",
    );
}
