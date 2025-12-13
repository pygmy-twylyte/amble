use serde_json::json;
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;

fn extract_enum_variants(source: &str, enum_name: &str) -> Vec<String> {
    let mut lines = source.lines();
    let mut variants = Vec::new();
    let mut inside = false;

    for line in &mut lines {
        if line.trim_start().starts_with(&format!("pub enum {enum_name}")) {
            inside = true;
            continue;
        }

        if inside {
            let line = line.trim();
            if line.starts_with('}') {
                break;
            }
            if let Some(name) = line
                .split('{')
                .next()
                .or_else(|| line.split('(').next())
                .map(|s| s.trim().trim_end_matches(','))
                && !name.is_empty() {
                    variants.push(name.to_string());
                }
        }
    }

    variants
}

fn main() -> io::Result<()> {
    let trigger_condition_src = fs::read_to_string("src/loader/triggers/raw_condition.rs")?;
    let trigger_action_src = fs::read_to_string("src/loader/triggers/raw_action.rs")?;
    let item_src = fs::read_to_string("src/item.rs")?;
    let goal_src = fs::read_to_string("src/goal.rs")?;

    let conditions = extract_enum_variants(&trigger_condition_src, "RawTriggerCondition");
    let actions = extract_enum_variants(&trigger_action_src, "RawTriggerAction");
    let abilities = extract_enum_variants(&item_src, "ItemAbility");
    let goal_groups = extract_enum_variants(&goal_src, "GoalGroup");

    let mut snippets = BTreeMap::new();

    // Top-level blocks
    snippets.insert(
        "triggerNew".into(),
        json!({
            "prefix": "triggerNew",
            "description": "New trigger block",
            "body": [
                "[[triggers]]",
                "name = \"${{1:Trigger name}}\"",
                "only_once = ${{2:true}}",
                "conditions = [",
                "    ${{3}}",
                "]",
                "actions = [",
                "    ${{4}}",
                "]"
            ]
        }),
    );

    snippets.insert(
        "itemNew".into(),
        json!({
            "prefix": "itemNew",
            "description": "New item block",
            "body": [
                "[[items]]",
                "id = \"${{1:item-id}}\"",
                "name = \"${{2:Item name}}\"",
                "description = \"\"\"",
                "${{3:Item description}}",
                "\"\"\"",
                "container = ${{4:false}}"
            ]
        }),
    );

    snippets.insert(
        "roomNew".into(),
        json!({
            "prefix": "roomNew",
            "description": "New room block",
            "body": [
                "[[rooms]]",
                "id = \"${{1:room-id}}\"",
                "name = \"${{2:Room name}}\"",
                "base_description = \"\"\"",
                "${{3:Room description}}",
                "\"\"\"",
                "location = \"${{4:Nowhere}}\"",
                "visited = ${{5:false}}"
            ]
        }),
    );

    snippets.insert(
        "npcNew".into(),
        json!({
            "prefix": "npcNew",
            "description": "New NPC block",
            "body": [
                "[[npcs]]",
                "id = \"${{1:npc-id}}\"",
                "name = \"${{2:Name}}\"",
                "description = \"\"\"",
                "${{3:NPC description}}",
                "\"\"\"",
                "state = \"${{4:normal}}\"",
                "",
                "[npcs.location]",
                "Room = \"${{5:room-id}}\""
            ]
        }),
    );

    snippets.insert(
        "goalNew".into(),
        json!({
            "prefix": "goalNew",
            "description": "New goal block",
            "body": [
                "[[goals]]",
                "id = \"${{1:goal-id}}\"",
                "name = \"${{2:Goal name}}\"",
                "description = \"${{3:Goal description}}\"",
                "group = { type = \"${{4:required}}\" }",
                "activate_when = { type = \"${{5:hasFlag}}\", flag = \"${{6:flag}}\" }",
                "finished_when = { type = \"${{7:hasFlag}}\", flag = \"${{8:flag}}\" }"
            ]
        }),
    );

    for cond in &conditions {
        let key = format!("tc-{}", cond.to_lowercase());
        snippets.insert(
            key.clone(),
            json!({
                "prefix": key,
                "description": format!("TriggerCondition: {}", cond),
                "body": [ format!("{{ type = \"{}\", key = \"${{1:value}}\" }}", cond) ]
            }),
        );
    }

    for action in &actions {
        let key = format!("ta-{}", action.to_lowercase());
        snippets.insert(
            key.clone(),
            json!({
                "prefix": key,
                "description": format!("TriggerAction: {}", action),
                "body": [ format!("{{ type = \"{}\", key = \"${{1:value}}\" }}", action) ]
            }),
        );
    }

    for ability in &abilities {
        let key = format!("item-ability-{}", ability.to_lowercase());
        snippets.insert(
            key.clone(),
            json!({
                "prefix": key,
                "description": format!("Item ability: {}", ability),
                "body": [
                    "[[items.abilities]]",
                    format!("type = \"{}\"", ability)
                ]
            }),
        );
    }

    for group in &goal_groups {
        let key = format!("goal-group-{}", group.to_lowercase());
        snippets.insert(
            key.clone(),
            json!({
                "prefix": key,
                "description": format!("Goal group type: {}", group),
                "body": [ format!("group = {{ type = \"{}\" }}", group) ]
            }),
        );
    }

    let project_snippet_path = PathBuf::from("../.zed/snippets/toml.json");
    let global_snippet_path = dirs::config_dir().unwrap().join("zed/snippets/toml.json");
    let backup_path = global_snippet_path.with_extension("toml.json.bak");

    if let Some(parent) = project_snippet_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = File::create(&project_snippet_path)?;
    file.write_all(serde_json::to_string_pretty(&snippets)?.as_bytes())?;
    println!("✅ Wrote project snippets to {}", project_snippet_path.display());

    if global_snippet_path.exists() {
        fs::copy(&global_snippet_path, &backup_path)?;
        println!("🔒 Backed up global snippet as {}", backup_path.display());
    }

    fs::create_dir_all(global_snippet_path.parent().unwrap())?;
    fs::copy(&project_snippet_path, &global_snippet_path)?;
    println!("📦 Installed snippets to {}", global_snippet_path.display());

    Ok(())
}
