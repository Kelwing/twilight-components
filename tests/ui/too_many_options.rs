use twilight_components::component;

fn main() {
    let _ = component!(Select { custom_id: "s" } [
        Option { label: "o", value: "0" }, Option { label: "o", value: "1" }, Option { label: "o", value: "2" },
        Option { label: "o", value: "3" }, Option { label: "o", value: "4" }, Option { label: "o", value: "5" },
        Option { label: "o", value: "6" }, Option { label: "o", value: "7" }, Option { label: "o", value: "8" },
        Option { label: "o", value: "9" }, Option { label: "o", value: "10" }, Option { label: "o", value: "11" },
        Option { label: "o", value: "12" }, Option { label: "o", value: "13" }, Option { label: "o", value: "14" },
        Option { label: "o", value: "15" }, Option { label: "o", value: "16" }, Option { label: "o", value: "17" },
        Option { label: "o", value: "18" }, Option { label: "o", value: "19" }, Option { label: "o", value: "20" },
        Option { label: "o", value: "21" }, Option { label: "o", value: "22" }, Option { label: "o", value: "23" },
        Option { label: "o", value: "24" }, Option { label: "o", value: "25" },
    ]);
}
