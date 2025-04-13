use std::fs;

mod todoist;

const PROJECTS_PATH: &str = "/home/hector/personal/todoist-to-org/org";

fn main() {
    let token = fs::read_to_string("/home/hector/personal/todoist-to-org/.env").expect("Should have been able to read this file");
    let mut my_todoist = todoist::TodoistAccount::new(token);

    match my_todoist.download() {
        Ok(_) => (),
        Err(error) => panic!("{error}"),
    }

    let _ = my_todoist.dump_to(PROJECTS_PATH);
}

