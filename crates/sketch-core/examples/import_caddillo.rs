//! Rebuild the curated catalog through the same DXF importer as the app.
use serde_json::{json, Value};
use std::{collections::BTreeMap, env, fs, path::Path};

// Keep original definitions verbatim, including unknown fields and decimal
// literals. Deserializing and reserializing the old geometry can round floats.
fn original_symbols(body: &str) -> Result<&str, Box<dyn std::error::Error>> {
    let key = body
        .find("\"symbols\"")
        .ok_or("original catalog has no symbols")?;
    let after = body[key + 9..]
        .trim_start()
        .strip_prefix(':')
        .ok_or("invalid symbols property")?
        .trim_start();
    if !after.starts_with('{') {
        return Err("symbols must be an object".into());
    }
    let mut depth = 0;
    let mut quoted = false;
    let mut escaped = false;
    for (i, b) in after.bytes().enumerate() {
        if quoted {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                quoted = false;
            }
        } else {
            match b {
                b'"' => quoted = true,
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Ok(&after[..=i]);
                    }
                }
                _ => {}
            }
        }
    }
    Err("unterminated symbols object".into())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args().collect();
    let root = Path::new(args.get(1).ok_or("expected extracted source directory")?);
    let selection: Value =
        serde_json::from_str(&fs::read_to_string("refs/caddillo/selection.json")?)?;
    let original = fs::read_to_string("assets/symbols.previous.json")?;
    let mut lib: Value = serde_json::from_str(&original)?;
    let mut added = BTreeMap::new();
    let mut categories: Vec<Value> = Vec::new();
    for item in selection["symbols"].as_array().ok_or("missing selection")? {
        let file = item["file"].as_str().unwrap();
        let mut symbol = sketch_core::symbol_import::from_dxf(
            &fs::read_to_string(root.join(file))?,
            item["label"].as_str().unwrap(),
        )?;
        symbol.id = item["id"].as_str().unwrap().into();
        symbol.category = item["category"].as_str().unwrap().into();
        symbol.category_name = item["categoryName"].as_str().unwrap().into();
        symbol.description =
            format!("Residential CAD block; source: {file}; CC0 1.0. Dimensions in inches.");
        let category = if let Some(i) = categories.iter().position(|c| c["key"] == symbol.category)
        {
            i
        } else {
            categories
                .push(json!({"key":symbol.category,"name":symbol.category_name,"symbolIds":[]}));
            categories.len() - 1
        };
        categories[category]["symbolIds"]
            .as_array_mut()
            .unwrap()
            .push(json!(symbol.id));
        lib["symbols"][&symbol.id] = serde_json::to_value(&symbol)?;
        added.insert(symbol.id.clone(), symbol);
    }
    categories.extend(lib["categories"].as_array().unwrap().iter().cloned());
    lib["categories"] = json!(categories);
    lib["defaultFavorites"] = json!([
        "caddillo/door-swing-36",
        "caddillo/door-swing-30",
        "caddillo/door-double-72",
        "caddillo/window-single-36",
        "caddillo/window-double-60",
        "caddillo/toilet",
        "caddillo/tub",
        "caddillo/shower-60x32",
        "caddillo/sink-bath-rect",
        "caddillo/fridge-36",
        "caddillo/range-30",
        "caddillo/dishwasher-24",
        "caddillo/washer-27",
        "caddillo/dryer-27",
        "caddillo/bed-king",
        "caddillo/couch-84",
        "caddillo/stairs",
        "caddillo/ac-condenser"
    ]);
    lib["symbolCount"] = json!(lib["symbols"].as_object().unwrap().len());
    lib["generatedFrom"] =
        json!("CADdillo residential selection (CC0 1.0), plus preserved original library");
    let raw_original = original_symbols(&original)?;
    lib.as_object_mut().unwrap().remove("symbols");
    let mut output = serde_json::to_string(&lib)?;
    output.pop();
    output.push_str(",\"symbols\":");
    output.push_str(&raw_original[..raw_original.len() - 1]);
    output.push(',');
    output.push_str(&serde_json::to_string(&added)?[1..]);
    output.push_str("}\n");
    let _: Value = serde_json::from_str(&output)?;
    fs::write("assets/symbols.json", output)?;
    println!(
        "Imported {} CADdillo symbols; {} total definitions, original IDs retained.",
        selection["symbols"].as_array().unwrap().len(),
        lib["symbolCount"]
    );
    Ok(())
}
