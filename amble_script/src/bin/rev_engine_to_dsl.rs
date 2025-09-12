use std::fs;

fn q(s: &str) -> String {
    if s.contains('\n') {
        format!("\"\"\"{}\"\"\"", s)
    } else {
        let esc = s.replace('"', "\\\"");
        format!("\"{}\"", esc)
    }
}

fn main() {
    // Reverse-engineer items.toml
    let items_toml = fs::read_to_string("amble_engine/data/items.toml")
        .or_else(|_| fs::read_to_string("../amble_engine/data/items.toml"))
        .expect("read items");
    let val: toml::Value = toml::from_str(&items_toml).expect("parse items");
    let mut out_items = String::new();
    if let Some(arr) = val.get("items").and_then(|v| v.as_array()) {
        for it in arr {
            let id = it.get("id").and_then(|v| v.as_str()).unwrap_or("");
            let name = it.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let desc = it.get("description").and_then(|v| v.as_str()).unwrap_or("");
            let portable = it.get("portable").and_then(|v| v.as_bool()).unwrap_or(false);
            out_items.push_str(&format!("item {} {{\n", id));
            out_items.push_str(&format!("  name {}\n", q(name)));
            out_items.push_str(&format!("  desc {}\n", q(desc)));
            out_items.push_str(&format!("  portable {}\n", if portable {"true"} else {"false"}));
            if let Some(loc) = it.get("location").and_then(|v| v.as_table()) {
                if let Some(s) = loc.get("Inventory").and_then(|v| v.as_str()) {
                    out_items.push_str(&format!("  location inventory {}\n", s));
                } else if let Some(s) = loc.get("Room").and_then(|v| v.as_str()) {
                    out_items.push_str(&format!("  location room {}\n", s));
                } else if let Some(s) = loc.get("Npc").and_then(|v| v.as_str()) {
                    out_items.push_str(&format!("  location npc {}\n", s));
                } else if let Some(s) = loc.get("Chest").and_then(|v| v.as_str()) {
                    out_items.push_str(&format!("  location chest {}\n", s));
                } else if let Some(s) = loc.get("Nowhere").and_then(|v| v.as_str()) {
                    out_items.push_str(&format!("  location nowhere {}\n", q(s)));
                }
            }
            if let Some(cs) = it.get("container_state").and_then(|v| v.as_str()) {
                out_items.push_str(&format!("  container state {}\n", cs));
            }
            if let Some(true) = it.get("restricted").and_then(|v| v.as_bool()) {
                out_items.push_str("  restricted true\n");
            }
            if let Some(text) = it.get("text").and_then(|v| v.as_str()) {
                out_items.push_str(&format!("  text {}\n", q(text)));
            }
            if let Some(abs) = it.get("abilities").and_then(|v| v.as_array()) {
                for ab in abs {
                    if let Some(typ) = ab.get("type").and_then(|v| v.as_str()) {
                        out_items.push_str(&format!("  ability {}\n", typ));
                    }
                }
            }
            if let Some(req) = it.get("interaction_requires").and_then(|v| v.as_table()) {
                for (k, v) in req.iter() {
                    if let Some(ability) = v.as_str() {
                        out_items.push_str(&format!("  requires {} {}\n", k, ability));
                    }
                }
            }
            out_items.push_str("}\n\n");
        }
    }
    fs::write("amble_script/data/items.amble", &out_items)
        .or_else(|_| fs::write("data/items.amble", &out_items))
        .expect("write items.amble");

    // Reverse-engineer npcs.toml
    let npcs_toml = fs::read_to_string("amble_engine/data/npcs.toml")
        .or_else(|_| fs::read_to_string("../amble_engine/data/npcs.toml"))
        .expect("read npcs");
    let val: toml::Value = toml::from_str(&npcs_toml).expect("parse npcs");
    let mut out_npcs = String::new();
    if let Some(arr) = val.get("npcs").and_then(|v| v.as_array()) {
        // Group entries that belong to same npc id
        use std::collections::BTreeMap;
        let mut by_id: BTreeMap<String, Vec<&toml::value::Table>> = BTreeMap::new();
        for t in arr {
            if let Some(tab) = t.as_table() {
                if let Some(id) = tab.get("id").and_then(|v| v.as_str()) {
                    by_id.entry(id.to_string()).or_default().push(tab);
                }
            }
        }
        for (id, tabs) in by_id {
            // Merge known fields across possible repeated tables
            let mut name = "".to_string();
            let mut desc = "".to_string();
            let mut state_inline: Option<String> = None;
            let mut state_custom: Option<String> = None;
            let mut loc_room: Option<String> = None;
            let mut loc_nowhere: Option<String> = None;
            let mut mv_type: Option<String> = None;
            let mut mv_rooms: Vec<String> = Vec::new();
            let mut mv_timing: Option<String> = None;
            let mut mv_active: Option<bool> = None;
            let mut dialogue: BTreeMap<String, Vec<String>> = BTreeMap::new();
            for tab in tabs {
                if let Some(s) = tab.get("name").and_then(|v| v.as_str()) { name = s.to_string(); }
                if let Some(s) = tab.get("description").and_then(|v| v.as_str()) { desc = s.to_string(); }
                if let Some(s) = tab.get("state").and_then(|v| v.as_str()) { state_inline = Some(s.to_string()); }
                if let Some(stab) = tab.get("state").and_then(|v| v.as_table()) {
                    if let Some(s) = stab.get("custom").and_then(|v| v.as_str()) { state_custom = Some(s.to_string()); }
                }
                if let Some(l) = tab.get("location").and_then(|v| v.as_table()) {
                    if let Some(s) = l.get("Room").and_then(|v| v.as_str()) { loc_room = Some(s.to_string()); }
                    if let Some(s) = l.get("Nowhere").and_then(|v| v.as_str()) { loc_nowhere = Some(s.to_string()); }
                }
                if let Some(m) = tab.get("movement").and_then(|v| v.as_table()) {
                    if let Some(s) = m.get("movement_type").and_then(|v| v.as_str()) { mv_type = Some(s.to_string()); }
                    if let Some(rs) = m.get("rooms").and_then(|v| v.as_array()) {
                        mv_rooms = rs.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect();
                    }
                    if let Some(s) = m.get("timing").and_then(|v| v.as_str()) { mv_timing = Some(s.to_string()); }
                    if let Some(b) = m.get("active").and_then(|v| v.as_bool()) { mv_active = Some(b); }
                }
                if let Some(d) = tab.get("dialogue").and_then(|v| v.as_table()) {
                    for (k, v) in d.iter() {
                        if let Some(arr) = v.as_array() {
                            let lines: Vec<String> = arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect();
                            dialogue.insert(k.clone(), lines);
                        }
                    }
                }
            }
            out_npcs.push_str(&format!("npc {} {{\n", id));
            out_npcs.push_str(&format!("  name {}\n", q(&name)));
            out_npcs.push_str(&format!("  desc {}\n", q(&desc)));
            if let Some(r) = loc_room { out_npcs.push_str(&format!("  location room {}\n", r)); }
            if let Some(nw) = loc_nowhere { out_npcs.push_str(&format!("  location nowhere {}\n", q(&nw))); }
            if let Some(c) = state_custom { out_npcs.push_str(&format!("  state custom {}\n", c)); }
            else if let Some(s) = state_inline { out_npcs.push_str(&format!("  state {}\n", s)); }
            if let Some(mt) = mv_type {
                out_npcs.push_str(&format!("  movement {} rooms ({} )", mt, mv_rooms.join(", ")));
                if let Some(ti) = mv_timing { out_npcs.push_str(&format!(" timing {}", ti)); }
                if let Some(ac) = mv_active { if ac { out_npcs.push_str(" active true"); } }
                out_npcs.push_str("\n");
            }
            // dialogue buckets
            for (k, lines) in dialogue {
                if let Some(rest) = k.strip_prefix("custom:") {
                    out_npcs.push_str(&format!("  dialogue custom {} {{\n", rest));
                } else {
                    out_npcs.push_str(&format!("  dialogue {} {{\n", k));
                }
                for ln in lines { out_npcs.push_str(&format!("    {}\n", q(&ln))); }
                out_npcs.push_str("  }\n");
            }
            out_npcs.push_str("}\n\n");
        }
    }
    fs::write("amble_script/data/npcs.amble", &out_npcs)
        .or_else(|_| fs::write("data/npcs.amble", &out_npcs))
        .expect("write npcs.amble");
}
