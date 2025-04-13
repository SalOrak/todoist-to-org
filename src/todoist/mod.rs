use serde::Deserialize;
use std::fmt;
use std::fs;
use std::io::Write;

const TODOIST_URL: &str = "https://api.todoist.com/rest/v2";

#[derive(Deserialize, Clone)]
#[allow(dead_code)]
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
    #[serde(skip)]
    tasks: Vec<DoistTask>
}

#[derive(Deserialize, Clone)]
#[allow(dead_code)]
struct DoistSection {
    id: String,
    project_id: String,
    order: i64,
    name: String,
}

#[derive(Deserialize, Clone)]
#[allow(dead_code)]
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
    
    #[serde(skip)]
    subtasks: Vec<DoistTask>,
}


#[derive(Deserialize, Clone)]
#[allow(dead_code)]
struct DoistTimeDue {
    date: String,
    is_recurring: bool,
    datetime: Option<String>,
    string: String,
    timezone: Option<String>,
    lang: Option<String>,
}

#[derive(Deserialize, Clone)]
#[allow(dead_code)]
struct DoistTimeDeadline {
    date: String,
    lang: Option<String>,
}

#[derive(Deserialize, Clone)]
#[allow(dead_code)]
struct DoistTimeDuration {
    amount: i64,
    unit: String,
}

struct DoistTaskFilters {
    filters: Vec<TaskFilterKind>,
}

#[allow(dead_code)]
enum TaskFilterKind {
    ProjectId(String),
    SectionId(String),
    Label(String),
    Filter(String),
}

impl DoistTaskFilters {
    fn new() -> Self {
        Self {
            filters: vec![],
        }
    }
    
    fn build_query(&self) -> Vec<(String, String)> {
        let mut res: Vec<(String, String)> = Vec::new();
        self.filters.iter().for_each(|filter| {
            res.push(filter.parse_filter());
        });
        res
    }

    fn push(&mut self, filter: TaskFilterKind) {
        self.filters.push(filter);
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

impl fmt::Display for DoistProject{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let title_task = self.name.trim();
        write!(f,
               "* Project {}\n{}",
            title_task, {
                let mut task_string: String = String::new();
                self.tasks.iter().for_each(|t| {
                    task_string.push_str(
                        &*(t.print_task(2))
                    )
                });
                task_string
            }
        )
    }

}

impl DoistTask {
    fn print_task(self: &Self, depth: usize) -> String {
        // 1. Print the task
        let mut content = self.content.clone();
        if let Some(stripped) = content.strip_prefix("* ") {
            content = stripped.to_string();
        };
        
        let mut task_string: String = format!("{} {}\n{}\n", "*".repeat(depth), content, self.description);
        // 2. Print the subtasks
        self.subtasks.iter().for_each(|task| task_string = format!("{}\n{}", task_string, task.print_task(depth + 1)));
        return task_string
    }
}

impl TodoistAccount {

    pub fn new(bearer: String) -> Self {
        Self{
            bearer: format!("Bearer {}", bearer),
            projects: Vec::new(),
        }
    }

    pub fn download(&mut self) -> Result<String, String> {
        // 1. Retrive all the projects
        let mut tmp_projects : Vec<DoistProject>;
        match self.get_projects() {
            Ok(projects) => tmp_projects = projects,
            Err(error) => return Err(error),
        };

        tmp_projects.iter_mut().for_each(|p| {
            let mut filters: DoistTaskFilters = DoistTaskFilters::new();
            let project_id = TaskFilterKind::ProjectId(p.id.clone());
            filters.push(project_id);
            match TodoistAccount::get_tasks_by_project(&self, &filters) { 
                Ok(tasks) => p.tasks = tasks,
                Err(_) => println!("Error while getting the task for project {}", p.id.clone()),
            }
        });
        
        tmp_projects.iter_mut()
            .for_each(|p| {
                let mut project: DoistProject = p.clone();
                let mut subtasks: Vec<DoistTask> = Vec::new();
                p.tasks.iter()
                    .filter(|t| t.parent_id.is_none())
                    .for_each(|t| {
                        let mut new_task = t.clone();
                        new_task.subtasks = TodoistAccount::build_tree(&t.clone(), &p.tasks);
                        subtasks.push(new_task);
                    });
                project.tasks = subtasks;
                self.projects.push(project);
            });

        return Ok("Hello world!".to_string());
    }

    pub fn dump_to(self: &Self, base_path: &str) -> Result<(), String> {
        match fs::exists(base_path) {
            Ok(_) => (),
            Err(error) => return Err(error.to_string()),
        };

        let base_path = base_path.strip_suffix("/").unwrap_or(base_path);

        self.projects.iter().for_each(|project| {
            // print each project into the corresponding file
            let lower_filename = project.name.to_lowercase();
            let trimmed_filename = lower_filename.trim();
            let filename = trimmed_filename.replace(" ", "_");
            match fs::File::create(format!("{}/{}.org", base_path, filename)) {
                Ok(mut file) => {
                    let _ = file.write(format!("{}", project).as_bytes());
                },
                Err(error) => println!("Error while processing projects:\n\t{}",error.to_string()),
            };
        });

        Ok(())
    }

    fn build_tree(task: &DoistTask, tasks: &Vec<DoistTask>) -> Vec<DoistTask>{
        let mut childrens: Vec<DoistTask> = vec![];
        tasks.iter()
            .filter(|t| {
                match t.parent_id {
                    Some(ref pid) => pid.clone() == task.id,
                    None => false,
                }
            })
            .for_each(|t| {
                let mut new_task = t.clone();
                new_task.subtasks = TodoistAccount::build_tree(&t, tasks);
                childrens.push(new_task);
            });
        childrens
    }

    fn get_projects(&self) -> Result<Vec<DoistProject>, String> {
        let projects_url = format!("{}/projects", TODOIST_URL);
        match ureq::get(projects_url).header("Authorization", &self.bearer).call() {
            Ok(mut response) => {
                Ok(response.body_mut().read_json::<Vec<DoistProject>>().unwrap())
            },
            Err(err) => {
                println!("Error while getting the projects...");
                Err(err.to_string())
            }
        }
    }

    fn get_tasks_by_project(&self, filters: &DoistTaskFilters) -> Result<Vec<DoistTask>, String> {
        let tasks_url = format!("{}/tasks", TODOIST_URL);
        match ureq::get(tasks_url)
            .header("Authorization", &self.bearer)
            .query_pairs(filters.build_query())
            .call() {
            Ok(mut response) => Ok(response.body_mut().read_json::<Vec<DoistTask>>().unwrap()),
            Err(error) => Err(error.to_string()),
        }
    }

    #[allow(dead_code)]
    fn get_project_by_id(&self, id: &str) -> Result<DoistProject, String> {
        let projects_by_id = format!("{TODOIST_URL}/projects/{id}");
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





