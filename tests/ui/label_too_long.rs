use twilight_components::components;

fn main() {
    let _ = components!(Label("This label is far too long to fit within the limit") [Input { custom_id: "a" }]);
}
