use std::collections::VecDeque;

const TAB: &str = "    ";

pub fn wrap(file_content: &str, mod_name: &str) -> String {
    let mut lines = file_content
        .split('\n')
        .map(str::to_owned)
        .collect::<VecDeque<String>>();
    mod_block(&mut lines, mod_name);
    allow_dead_code(&mut lines);
    fold_marker(&mut lines, mod_name);
    let lines = lines.iter().cloned().collect::<Vec<_>>();
    lines.join("\n")
}

fn mod_block(lines: &mut VecDeque<String>, mod_name: &str) {
    lines.push_front(format!("mod {} {{", mod_name));
    lines.iter_mut().for_each(|x| {
        if !x.is_empty() {
            *x = format!("{}{}", &TAB, &x)
        }
    });
    lines.push_back("}".to_owned());
}

fn allow_dead_code(lines: &mut VecDeque<String>) {
    lines.push_front("#[allow(dead_code)]".to_owned());
}

fn fold_marker(lines: &mut VecDeque<String>, mod_name: &str) {
    lines.push_front(format!("// {} {{{{{{", mod_name));
    lines.push_back("// }}}".to_owned());
}
