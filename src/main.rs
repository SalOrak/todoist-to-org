use std::fs;

mod todoist;

fn main() {
    let token = fs::read_to_string("/home/hector/personal/todoist-to-org/.env").expect("Should have been able to read this file");
    let t = todoist::TodoistAccount::setup(token);
}

