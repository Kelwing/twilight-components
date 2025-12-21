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
twilight-components-macro = "0.1"
twilight-model = "0.17"
```

## Quick Start

```rust
use twilight_components_macro::{component, components};

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
| `Button { ... }` | Button | label | `style`, `label`, `custom_id`, `url`, `disabled`, `emoji` |
| `ActionRow [ ... ]` | ActionRow | - | - |
| `Container [ ... ]` | Container | - | `id`, `color`/`accent_color`, `spoiler` |
| `Section [ ... ]` | Section | - | `id`, `accessory` |
| `Separator` | Separator | - | `id`, `divider`, `spacing` |
| `MediaGallery [ ... ]` | MediaGallery | - | `id` |
| `MediaItem(url)` | MediaGalleryItem | url | `description`, `spoiler` |
| `Thumbnail(url)` | Thumbnail | url | `description`, `spoiler` |
| `File(url)` | FileDisplay | url | `spoiler` |
| `Select { ... }` | SelectMenu | - | `custom_id`, `kind`, `placeholder`, `min_values`, `max_values` |
| `Option(label)` | SelectMenuOption | label | `value`, `description`, `emoji`, `default` |
| `Input { ... }` | TextInput | - | `custom_id`, `label`, `style`, `placeholder`, `required` |

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

### Modal Forms (TextInput)

```rust
let modal_components = components!(
    Row [
        Input {
            custom_id: "name_input",
            label: "Your Name",
            style: Short,
            placeholder: "Enter your name...",
            required: true,
        }
    ],
    Row [
        Input {
            custom_id: "bio_input", 
            label: "About You",
            style: Paragraph,
            placeholder: "Tell us about yourself...",
            min_length: 10,
            max_length: 500,
        }
    ]
);
```

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

## Integration with twilight-http

```rust
use twilight_http::Client;
use twilight_components_macro::components;

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
| Nested Components | Tedious | Better | Natural |
| Learning Curve | Moderate | Moderate | Low |
| IDE Support | Full | Full | Partial |

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - see LICENSE for details.
