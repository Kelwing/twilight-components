# twilight-components

A declarative, ergonomic macro for building Discord message components with [twilight-model](https://crates.io/crates/twilight-model).

## Motivation

Building Discord components with twilight-model requires verbose struct initialization:

```rust
// Before: 11 lines for a simple container with text
let components = vec![Component::Container(Container {
    id: None,
    accent_color: Some(0xFF0000),
    spoiler: false,
    components: vec![
        Component::TextDisplay(TextDisplay {
            id: None,
            content: "Hello, World!".to_string(),
        }),
    ],
})];
```

With this macro, the same code becomes:

```rust
// After: 4 lines with clear structure
let components = components!(
    Container { color: 0xFF0000 } [
        Text("Hello, World!"),
    ]
);
```

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
twilight-components = "0.1"
twilight-model = "0.17"
```

## Quick Start

```rust
use twilight_components::{component, components};

// Single component
let text = component!(Text("Welcome to my bot!"));

// Multiple components
let ui = components!(
    Container { color: 0x5865F2 } [
        Text("# Dashboard"),
        Separator,
        Section {
            accessory: Button { style: Link, url: "https://example.com", label: "Visit" }
        } [
            Text("Check out our website for more information."),
        ],
    ]
);
```

## Syntax Guide

### Basic Structure

```
ComponentType { field: value, ... } [ children, ... ]
             ↑ Properties (optional)  ↑ Children (optional)
```

- **Braces `{}`** - Optional fields/properties
- **Brackets `[]`** - Child components (for containers)
- **Parentheses `()`** - Shorthand for the primary field

### Component Reference

| Macro Syntax | Type | Shorthand | Properties |
|--------------|------|-----------|------------|
| `Text(content)` | TextDisplay | content | `id` |
| `Button { ... }` | Button | label | `id`, `style`, `label`, `custom_id`, `url`, `disabled`, `emoji`, `sku_id` |
| `ActionRow [ ... ]` | ActionRow | - | `id` |
| `Container [ ... ]` | Container | - | `id`, `color`/`accent_color`, `spoiler` |
| `Section [ ... ]` | Section | - | `id`, `accessory` |
| `Separator` | Separator | - | `id`, `divider`, `spacing` |
| `MediaGallery [ ... ]` | MediaGallery | - | `id` |
| `MediaItem(url)` | MediaGalleryItem | url | `description`, `spoiler` |
| `Thumbnail(url)` | Thumbnail | url | `id`, `description`, `spoiler` |
| `File(url)` | FileDisplay | url (`attachment://...`) | `id`, `spoiler` |
| `FileUpload { ... }` | FileUpload | - | `id`, `custom_id`, `min_values`, `max_values`, `required` |
| `Label("text") [ ... ]` | Label | label | `id`, `description`, single child component |
| `Select { ... }` | SelectMenu | - | `id`, `custom_id`, `kind`, `placeholder`, `min_values`, `max_values`, `disabled`, `required`, `channel_types`, `default_values` |
| `Option(label)` | SelectMenuOption | label | `value`, `description`, `emoji`, `default` |
| `Input { ... }` | TextInput | - | `id`, `custom_id`, `style`, `placeholder`, `required`, `min_length`, `max_length`, `value` |

### Aliases

For convenience, many components have shorter aliases:

- `Text` = `TextDisplay`
- `Row` = `ActionRow`
- `Btn` = `Button`
- `Sep` = `Separator`
- `Gallery` = `MediaGallery`
- `Item` = `MediaItem`
- `Thumb` = `Thumbnail`
- `Select` = `SelectMenu`
- `Option` = `SelectOption`
- `Input` = `TextInput`
- `Upload` = `FileUpload`

## Examples

### Buttons in an Action Row

```rust
let buttons = component!(
    Row [
        Button { style: Primary, label: "Confirm", custom_id: "confirm" },
        Button { style: Danger, label: "Cancel", custom_id: "cancel" },
        Button { style: Link, label: "Help", url: "https://example.com/help" },
    ]
);
```

### Select Menu

```rust
let menu = component!(
    Row [
        Select { custom_id: "color_select", placeholder: "Choose a color" } [
            Option { label: "Red", value: "red", description: "A warm color" },
            Option { label: "Blue", value: "blue", description: "A cool color" },
            Option { label: "Green", value: "green", default: true },
        ]
    ]
);
```

### Complex Layout

```rust
let dashboard = components!(
    Container { color: 0x5865F2 } [
        Text("# Welcome to the Dashboard"),
        Separator { spacing: Large },
        
        Section {
            accessory: Thumbnail("https://example.com/avatar.png")
        } [
            Text("**Your Profile**"),
            Text("Level: 42 | XP: 1,337"),
        ],
        
        Separator,
        
        Row [
            Button { style: Primary, label: "View Stats", custom_id: "stats" },
            Button { style: Secondary, label: "Settings", custom_id: "settings" },
        ],
    ],
    
    Container [
        MediaGallery [
            Item("https://example.com/image1.png"),
            Item { url: "https://example.com/image2.png", description: "Alt text" },
        ],
    ]
);
```

### Using Expressions

The macro supports any Rust expression for values:

```rust
let user_name = "Alice";
let avatar_url = get_avatar_url();
let is_premium = user.premium;

let profile = component!(
    Container { color: if is_premium { 0xFFD700 } else { 0x808080 } } [
        Text(format!("# Welcome, {}!", user_name)),
        Section {
            accessory: Thumbnail(avatar_url)
        } [
            Text(format!("Member since: {}", user.joined_at)),
        ],
    ]
);
```

### Modal Forms (TextInput with Label)

In a modal, `Label` is a top-level component. Do not put it in an action row.

```rust
let modal_components = components!(
    Label("Your Name") [
        Input {
            custom_id: "name_input",
            style: Short,
            placeholder: "Enter your name...",
            required: true,
        }
    ],
    Label { label: "About You", description: "Tell us a bit about yourself" } [
        Input {
            custom_id: "bio_input",
            style: Paragraph,
            placeholder: "Tell us about yourself...",
            min_length: 10,
            max_length: 500,
        }
    ],
    Label("Favorite Role") [
        Select { custom_id: "role_select", kind: Role, required: true }
    ],
    Label("Attachment") [
        Upload { custom_id: "file_upload", required: true }
    ],
);
```

### Optional Fields

Optional fields such as `id`, `emoji`, `sku_id` and `default_values` accept either a
plain value or an `Option`:

```rust
let a = component!(Text { content: "Hi", id: 1 });
let b = component!(Text { content: "Hi", id: Some(1) });
let c = component!(Text { content: "Hi", id: maybe_id }); // Option<i32>
```

### Emoji

A string literal is a Unicode emoji. For a custom emoji, give an `EmojiReactionType`:

```rust
let like = component!(Button { label: "Like", custom_id: "like", emoji: "👍" });

let rogue = component!(Button {
    label: "Rogue",
    custom_id: "rogue",
    emoji: EmojiReactionType::Custom {
        animated: false,
        id: Id::new(625891304148303894),
        name: Some("rogue".into()),
    },
});
```

### Premium Buttons

`sku_id` accepts an integer literal or an `Id<SkuMarker>`:

```rust
let buy = component!(Button { style: Premium, sku_id: 1234567890 });
```

## Compile-time Validation

The macros check your components against the rules in Discord's documentation. Most
payloads that Discord would reject at runtime fail to compile instead, with the error on
the token that is wrong:

```text
error: `Primary` buttons must have a `custom_id`
 --> src/main.rs:4:24
  |
4 |     let _ = component!(Button { label: "Click" });
  |                        ^^^^^^
```

The macros check these rules:

- **Fields:** required fields are present, field names are known, and no field is given
  twice. A typo such as `custom_i` is an error.
- **Nesting:** each component contains only the components that Discord allows. For
  example, `Section` contains 1 to 3 `Text` components, and its `accessory` is a
  `Button` or a `Thumbnail`.
- **Counts:**
  - An `ActionRow` has 1 to 5 buttons or 1 select menu.
  - A `MediaGallery` has 1 to 10 items.
  - A `Text` select menu has 1 to 25 options.
  - A `Label` has exactly 1 component.
- **Buttons:** each style has the fields it needs and none of the fields it cannot have.
  For example, a `Link` button needs a `url` and cannot have a `custom_id`.
- **Select menus:** options are allowed only on `Text` menus, `channel_types` only on
  `Channel` menus, and the number of `default_values` must be in range.
- **Limits:** string lengths (such as 1 to 100 characters for a `custom_id`), number
  ranges (such as `max_values` and `accent_color`), and `min_values` ≤ `max_values`.
- **Uniqueness:** each `custom_id` and each `id` is used only once in one invocation.
- **Files:** a `File` URL starts with `attachment://`.

### Messages and modals

The macro infers the context from the components in one invocation:

- **Message-only:** `ActionRow`, `Button`, `Container`, `Section`, `Separator`,
  `MediaGallery`, `Thumbnail`, `File`.
- **Modal-only:** `Label`, `TextInput`, `FileUpload`.
- **Both:** `Text`, `SelectMenu`.

An invocation that mixes message-only and modal-only components does not compile. In a
modal, components cannot be `disabled`, and `min_values: 0` needs `required: false`.

`components!` also checks the top level:

- **Message:** only `ActionRow`, `Text`, `Section`, `MediaGallery`, `File`, `Separator`
  and `Container` can be at the top level. A message can have at most 40 components,
  including nested ones.
- **Modal:** only `Label` and `Text` can be at the top level. A modal can have at most 5
  top-level components.

`component!` builds one component that you can put in a larger structure later, so it
does not check the top level.

### Limits of the checks

- **Runtime values:** the macro checks only literal values. It cannot check a value that
  is computed at runtime, such as `format!(..)` or a variable. Use
  [twilight-validate](https://crates.io/crates/twilight-validate) for those values.
- **Undocumented rules:** these behaviors are not in Discord's documentation, so the
  macro does not check them:
  - an empty `Container`
  - empty `Text` content
  - a button with no label and no emoji
  - `max_values` greater than the number of options
  - duplicate option values
- **Legacy messages:** messages sent without the `IS_COMPONENTS_V2` flag have a limit of
  5 action rows. The macro cannot see the flag, so it does not check this limit.

## Button Styles

When specifying `style` for buttons, you can use shorthand:

- `Primary` - Blue button (requires `custom_id`)
- `Secondary` - Gray button (requires `custom_id`)
- `Success` - Green button (requires `custom_id`)
- `Danger` - Red button (requires `custom_id`)
- `Link` - Gray button with link icon (requires `url`)
- `Premium` - Special premium button (requires `sku_id`)

## Separator Spacing

The `spacing` property accepts:

- `Small` (default)
- `Large`

## Not Yet Supported

These Discord features are not in twilight-model 0.17, so this macro cannot build them:

- `Radio Group`, `Checkbox Group` and `Checkbox` modal components (types 21, 22 and 23).
  The macro gives a compile error if you use them.
- `file_types` on `FileUpload`.
- An explicit `null` `accent_color` on `Container`. Leave out `color` to send no accent color.

## Integration with twilight-http

```rust
use twilight_http::Client;
use twilight_components::components;

async fn send_welcome(client: &Client, channel_id: Id<ChannelMarker>) {
    let components = components!(
        Container { color: 0x57F287 } [
            Text("# Welcome! 🎉"),
            Text("Thanks for joining our server."),
            Separator,
            Row [
                Button { style: Primary, label: "Get Started", custom_id: "onboard" },
            ],
        ]
    );
    
    client
        .create_message(channel_id)
        .components(&components)?
        .flags(MessageFlags::IS_COMPONENTS_V2)
        .await?;
}
```

## Comparison

| Feature | twilight-model | Builder Pattern | This Macro |
|---------|----------------|-----------------|------------|
| Verbosity | High | Medium | Low |
| Type Safety | Full | Full | Full |
| Discord Rule Checks | Runtime (twilight-validate) | Runtime (twilight-validate) | Compile time (literals) |
| Nested Components | Tedious | Better | Natural |
| Learning Curve | Moderate | Moderate | Low |
| IDE Support | Full | Full | Partial |

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - see LICENSE for details.
