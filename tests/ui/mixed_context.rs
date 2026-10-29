use twilight_components::components;

fn main() {
    let _ = components!(Container [Text("a")], Label("Name") [Input { custom_id: "name" }]);
}
