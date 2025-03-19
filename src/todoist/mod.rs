
use serde::Deserialize;

const TODOIST_URL: &str = "https://api.todoist.com/rest/v2";

#[derive(Deserialize)]
struct DoistProject {
    id: String,
    name: String,
    color: String,
    parent_id: Option<String>,
    order: i64,
    comment_count: i64,
    is_shared: bool,
    is_favorite: bool,
    is_inbox_project: bool,
    is_team_inbox: bool,
    view_style: String,
    url: String,
    tasks: Vec<DoistTask>
}

#[derive(Deserialize)]
struct DoistSection {
    id: String,
    project_id: String,
    order: i64,
    name: String,
}

#[derive(Deserialize)]
struct DoistTask {
    id: String,
    project_id: String,
    section_id: Option<String>,
    content: String,
    description: String,
    is_completed: bool,
    labels: Vec<String>,
    parent_id: Option<String>,
    order: i64,
    priority: i64,
    due: Option<DoistTimeDue> ,
    deadline: Option<DoistTimeDeadline>,
    url: String,
    comment_count: i64,
    created_at: String,
    creator_id: String,
    assignee_id: Option<String>,
    assigner_id: Option<String>,
    duration: Option<DoistTimeDuration>,
    depth_level: Option<u8>, // How many tasks are in the tree. 0 if none.
}


#[derive(Deserialize)]
struct DoistTimeDue {
    date: String,
    is_recurring: bool,
    datetime: Option<String>,
    string: String,
    timezone: Option<String>,
    lang: Option<String>,
}

#[derive(Deserialize)]
struct DoistTimeDeadline {
    date: String,
    lang: Option<String>,
}

#[derive(Deserialize)]
struct DoistTimeDuration {
    amount: i64,
    unit: String,
}

struct DoistTaskFilters {
    filters: Vec<TaskFilterKind>,
}

enum TaskFilterKind {
    ProjectId(String),
    SectionId(String),
    Label(String),
    Filter(String),
}

impl DoistTaskFilters {
    fn build_query(&self) -> Vec<(String, String)> {
        let mut res: Vec<(String, String)> = Vec::new();
        self.filters.iter().for_each(|filter| {
            res.push(filter.parse_filter());
        });
        res
    }
}

impl TaskFilterKind {
    fn parse_filter(&self) -> (String, String){
        match self {
            TaskFilterKind::ProjectId(f) => (String::from("project_id"), f.to_string()),
            TaskFilterKind::SectionId(f) => (String::from("section_id"), f.to_string()),
            TaskFilterKind::Label(f) =>     (String::from("label"), f.to_string()),
            TaskFilterKind::Filter(f) =>    (String::from("filter"), f.to_string()),
        }
    }
}

pub struct TodoistAccount {
    bearer: String,
    projects: Vec<DoistProject>,
}

impl TodoistAccount {

    pub fn setup(bearer: String) -> Self {
        // fs::read_to_string("/home/hector/personal/todoist-to-org/.env").expect("Should have been able to read this file")
        Self{
            bearer,
            projects: Vec::new(),
        }
    }

    pub fn retrieve_data(&mut self) {
        // 1. Retrive all the projects
        todo!();
    }

    fn get_projects(&self) -> Result<Vec<DoistProject>, String> {
        let projects_url = format!("{}/projects", TODOIST_URL);
        match ureq::get(projects_url).header("Authorization", &self.bearer).call() {
            Ok(mut response) => {
                Ok(response.body_mut().read_json::<Vec<DoistProject>>().unwrap())
            },
            Err(err) => {
                Err(err.to_string())
            }
        }
    }
    
    fn get_project_by_id(&self, id: &str) -> Result<DoistProject, String> {
        let projects_by_id = format!("{}/projects/{}", TODOIST_URL, id);
        match ureq::get(projects_by_id).header("Authorization", &self.bearer).call() {
            Ok(mut response) => {
                Ok(response.body_mut().read_json::<DoistProject>().unwrap())
            },
            Err(err) => {
                Err(err.to_string())
            }
        }
    }
}
x
