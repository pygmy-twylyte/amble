//! Reverse-engineering utility for existing engine data.
//!
//! Reads the engine's TOML content and emits an approximate Amble DSL
//! representation to help bootstrap script authoring. This tool is left in place for
//! legacy purposes (it was used when much of the Amble demo was first written directly
//! in TOML directly and the DSL was being created). It shouldn't be needed at all any
//! more, and it hasn't been maintained with further developments of the DSL and engine,
//! so the amblescript that it generates will likely not be 100% correct in more complex cases.

use std::fs;

use amble_script::MovabilityAst;

fn q(s: &str) -> String {
    if s.contains('\n') {
        format!("\"\"\"{s}\"\"\"")
    } else {
        let esc = s.replace('"', "\\\"");
        format!("\"{esc}\"")
    }
}

fn movability_from_value(val: Option<&toml::Value>, portable: Option<bool>, restricted: Option<bool>) -> MovabilityAst {
    if let Some(v) = val {
        if let Some(s) = v.as_str()
            && s.eq_ignore_ascii_case("free")
        {
            return MovabilityAst::Free;
        }

        if let Some(tab) = v.as_table() {
            if let Some(fixed) = tab.get("fixed").and_then(|v| v.as_table())
                && let Some(reason) = fixed.get("reason").and_then(|v| v.as_str())
            {
                return MovabilityAst::Fixed {
                    reason: reason.to_string(),
                };
            }

            if let Some(rest) = tab.get("restricted").and_then(|v| v.as_table())
                && let Some(reason) = rest.get("reason").and_then(|v| v.as_str())
            {
                return MovabilityAst::Restricted {
                    reason: reason.to_string(),
                };
            }

            if tab.get("free").is_some() {
                return MovabilityAst::Free;
            }
        }
    }
    if let Some(false) = portable {
        return MovabilityAst::Fixed {
            reason: "It won't budge.".into(),
        };
    }
    if let Some(true) = restricted {
        return MovabilityAst::Restricted {
            reason: "You can't take that yet.".into(),
        };
    }
    MovabilityAst::Free
}

fn movability_to_dsl(m: &MovabilityAst) -> String {
    match m {
        MovabilityAst::Free => "free".to_string(),
        MovabilityAst::Fixed { reason } => format!("fixed {}", q(reason)),
        MovabilityAst::Restricted { reason } => format!("restricted {}", q(reason)),
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
            let portable = it.get("portable").and_then(|v| v.as_bool());
            let restricted = it.get("restricted").and_then(|v| v.as_bool());
            let movability = movability_from_value(it.get("movability"), portable, restricted);
            out_items.push_str(&format!("item {id} {{\n"));
            out_items.push_str(&format!("  name {}\n", q(name)));
            out_items.push_str(&format!("  desc {}\n", q(desc)));
            out_items.push_str(&format!("  movability {}\n", movability_to_dsl(&movability)));
            if let Some(loc) = it.get("location").and_then(|v| v.as_table()) {
                if let Some(s) = loc.get("Inventory").and_then(|v| v.as_str()) {
                    out_items.push_str(&format!("  location inventory {s}\n"));
                } else if let Some(s) = loc.get("Room").and_then(|v| v.as_str()) {
                    out_items.push_str(&format!("  location room {s}\n"));
                } else if let Some(s) = loc.get("Npc").and_then(|v| v.as_str()) {
                    out_items.push_str(&format!("  location npc {s}\n"));
                } else if let Some(s) = loc.get("Chest").and_then(|v| v.as_str()) {
                    out_items.push_str(&format!("  location chest {s}\n"));
                } else if let Some(s) = loc.get("Nowhere").and_then(|v| v.as_str()) {
                    out_items.push_str(&format!("  location nowhere {}\n", q(s)));
                }
            }
            if let Some(cs) = it.get("container_state").and_then(|v| v.as_str()) {
                out_items.push_str(&format!("  container state {cs}\n"));
            }
            if let Some(text) = it.get("text").and_then(|v| v.as_str()) {
                out_items.push_str(&format!("  text {}\n", q(text)));
            }
            if let Some(abs) = it.get("abilities").and_then(|v| v.as_array()) {
                for ab in abs {
                    if let Some(typ) = ab.get("type").and_then(|v| v.as_str()) {
                        out_items.push_str(&format!("  ability {typ}\n"));
                    }
                }
            }
            if let Some(req) = it.get("interaction_requires").and_then(|v| v.as_table()) {
                for (interaction, v) in req.iter() {
                    if let Some(ability) = v.as_str() {
                        // New DSL: requires <ability> to <interaction>
                        out_items.push_str(&format!("  requires {ability} to {interaction}\n"));
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
            if let Some(tab) = t.as_table()
                && let Some(id) = tab.get("id").and_then(|v| v.as_str())
            {
                by_id.entry(id.to_string()).or_default().push(tab);
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
                if let Some(s) = tab.get("name").and_then(|v| v.as_str()) {
                    name = s.to_string();
                }
                if let Some(s) = tab.get("description").and_then(|v| v.as_str()) {
                    desc = s.to_string();
                }
                if let Some(s) = tab.get("state").and_then(|v| v.as_str()) {
                    state_inline = Some(s.to_string());
                }
                if let Some(stab) = tab.get("state").and_then(|v| v.as_table())
                    && let Some(s) = stab.get("custom").and_then(|v| v.as_str())
                {
                    state_custom = Some(s.to_string());
                }

                if let Some(l) = tab.get("location").and_then(|v| v.as_table()) {
                    if let Some(s) = l.get("Room").and_then(|v| v.as_str()) {
                        loc_room = Some(s.to_string());
                    }
                    if let Some(s) = l.get("Nowhere").and_then(|v| v.as_str()) {
                        loc_nowhere = Some(s.to_string());
                    }
                }
                if let Some(m) = tab.get("movement").and_then(|v| v.as_table()) {
                    if let Some(s) = m.get("movement_type").and_then(|v| v.as_str()) {
                        mv_type = Some(s.to_string());
                    }
                    if let Some(rs) = m.get("rooms").and_then(|v| v.as_array()) {
                        mv_rooms = rs.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect();
                    }
                    if let Some(s) = m.get("timing").and_then(|v| v.as_str()) {
                        mv_timing = Some(s.to_string());
                    }
                    if let Some(b) = m.get("active").and_then(|v| v.as_bool()) {
                        mv_active = Some(b);
                    }
                }
                if let Some(d) = tab.get("dialogue").and_then(|v| v.as_table()) {
                    for (k, v) in d.iter() {
                        if let Some(arr) = v.as_array() {
                            let lines: Vec<String> =
                                arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect();
                            dialogue.insert(k.clone(), lines);
                        }
                    }
                }
            }
            out_npcs.push_str(&format!("npc {id} {{\n"));
            out_npcs.push_str(&format!("  name {}\n", q(&name)));
            out_npcs.push_str(&format!("  desc {}\n", q(&desc)));
            if let Some(r) = loc_room {
                out_npcs.push_str(&format!("  location room {r}\n"));
            }
            if let Some(nw) = loc_nowhere {
                out_npcs.push_str(&format!("  location nowhere {}\n", q(&nw)));
            }
            if let Some(c) = state_custom {
                out_npcs.push_str(&format!("  state custom {c}\n"));
            } else if let Some(s) = state_inline {
                out_npcs.push_str(&format!("  state {s}\n"));
            }
            if let Some(mt) = mv_type {
                out_npcs.push_str(&format!("  movement {mt} rooms ({} )", mv_rooms.join(", ")));
                if let Some(ti) = mv_timing {
                    out_npcs.push_str(&format!(" timing {ti}"));
                }
                if let Some(ac) = mv_active
                    && ac
                {
                    out_npcs.push_str(" active true");
                }

                out_npcs.push('\n');
            }
            // dialogue buckets
            for (k, lines) in dialogue {
                if let Some(rest) = k.strip_prefix("custom:") {
                    out_npcs.push_str(&format!("  dialogue custom {rest} {{\n"));
                } else {
                    out_npcs.push_str(&format!("  dialogue {k} {{\n"));
                }
                for ln in lines {
                    out_npcs.push_str(&format!("    {}\n", q(&ln)));
                }
                out_npcs.push_str("  }\n");
            }
            out_npcs.push_str("}\n\n");
        }
    }
    fs::write("amble_script/data/npcs.amble", &out_npcs)
        .or_else(|_| fs::write("data/npcs.amble", &out_npcs))
        .expect("write npcs.amble");
}
