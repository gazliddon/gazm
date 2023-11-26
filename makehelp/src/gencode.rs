use handlebars::Handlebars;
use serde_json::json;
use crate::helpentry::HelpEntry;
use crate::template::TEMPLATE;

pub fn generate_rust_code(_help: &[HelpEntry]) -> String {
    let reg = Handlebars::new();

    let enums: Vec<_> = _help.iter().map(|h| h.id.clone()).collect();

    let data: Vec<(String, String)> = _help
        .iter()
        .map(|h| (h.id.to_string(), h.text.to_string()))
        .collect();

    let data_str: Vec<_> = data
        .into_iter()
        .map(|(id, text)| format!("({id}, String::from(r#\"{text}\"#))"))
        .collect();

    reg.render_template(
        TEMPLATE,
        &json!({"enums" : enums.join(",\n\t"),"data" : data_str.join(",\n\t\t")}),
    )
    .unwrap()
}


