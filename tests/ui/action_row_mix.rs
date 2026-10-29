use twilight_components::component;

fn main() {
    let _ = component!(Row [Button { custom_id: "a" }, Select { custom_id: "s", kind: User }]);
}
