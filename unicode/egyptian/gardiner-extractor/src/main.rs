use std::fs::read_to_string;
use serde::Serialize;

#[derive(Serialize)]
struct JseshSign {
    family: String,
    family_name: String,
    jsesh_code: String,
}

fn main() -> anyhow::Result<()> {
    let text = read_to_string("jsesh.txt")?;
    let mut results = Vec::new();

    let mut current_family = None::<String>;
    let mut current_family_name = None::<String>;
    let mut awaiting_family_name = false;

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Match: "A family", "Aa family", etc.
        if let Some(fam) = line.strip_suffix(" family") {
            current_family = Some(fam.to_string());
            current_family_name = None;
            awaiting_family_name = true;
            continue;
        }

        if awaiting_family_name {
            current_family_name = Some(line.to_string());
            awaiting_family_name = false;
            continue;
        }

        // Match sign lines: "12 A14B"
        if let Some((_, code)) = line.split_once(' ') {
            if code.chars().next().unwrap_or('_').is_alphanumeric() {
                if let (Some(f), Some(name)) =
                    (&current_family, &current_family_name)
                {
                    results.push(JseshSign {
                        family: f.clone(),
                        family_name: name.clone(),
                        jsesh_code: code.to_string(),
                    });
                }
            }
        }
    }

    std::fs::write(
        "jsesh_inventory.json",
        serde_json::to_string_pretty(&results)?,
    )?;

    Ok(())
}

