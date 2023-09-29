use std::collections::HashMap;

pub fn parse_code_blocks(content: &str) -> Vec<(String, HashMap<&str, &str>)> {
    let mut blocks = Vec::new();
    let mut current_meta = HashMap::new();
    let mut current_block = Vec::new();
    let mut in_block = false;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("```") {
            in_block = !in_block;
            if !in_block && !current_block.is_empty() {
                blocks.push((current_block.join("\n"), current_meta));
                current_block.clear();
                current_meta = HashMap::new();
            }
            continue;
        }

        if in_block {
            current_block.push(line);
        } else if !line.is_empty() {
            let split: Vec<_> = line.split('=').collect();
            if split.len() == 2 {
                current_meta.insert(split[0], split[1]);
            }
        }
    }

    blocks
}
