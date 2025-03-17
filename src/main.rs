use serde::Deserialize;
use std::fs;
use std::io::Write;
use std::rc::Rc;
use std::collections::HashMap;

use std::fmt;

const FILE_PATH: &str = "/home/hector/personal/todoist-to-org/org/";

#[derive(Deserialize, Debug)]
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
    tasks: Option<Vec<DoistTask>>,
}

impl fmt::Display for DoistProject{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
               "* Project {}\n{}",
                self.name.trim(), match &self.tasks {
                   Some(tasks) => {
                       let mut string_builder = String::from("");
                       tasks.iter().for_each(|task|{
                           let d = match task.depth_level {Some(d) => d, None => 0} + 1; // Add one because we are inside the project
                           // let star_str = "*".repeat(d as usize);
                           let star_str = "*".repeat(2);
                           string_builder.push_str(&format!(
                               "{} TODO {}\n {}\n",
                               star_str,
                               task.content,
                               task.description) as &str);
                       });
                       string_builder
                   },
                   None => {
                       "".to_string()
                   }
               }
        )
    }

}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct DoistSection {
    id: String,
    project_id: String,
    order: i64,
    name: String,
}

#[derive(Deserialize, Debug)]
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
    subtasks: Box<Option<Vec<DoistTask>>>,
    
}
impl fmt::Display for DoistTask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let d = match self.depth_level {Some(d) => d, None => 0} + 1; // Add one because we are inside the project
        let star_str = "*".repeat(d as usize);
        write!(f,
               "{} TODO {}\n {}\n",
               star_str,
               self.content,
               self.description) 
    }
}
         

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct DoistTimeDue {
    date: String,
    is_recurring: bool,
    datetime: Option<String>,
    string: String,
    timezone: Option<String>,
    lang: Option<String>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct DoistTimeDeadline {
    date: String,
    lang: Option<String>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct DoistTimeDuration {
    amount: i64,
    unit: String,
}

#[derive(Debug)]
struct DoistTaskFilters {
    filters: Vec<TaskFilterKind>,
 }

#[derive(Debug)]
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
    

fn get_api_token() -> String  {
    fs::read_to_string("/home/hector/personal/todoist-to-org/.env").expect("Should have been able to read this file")
}

fn get_all_projects(base_url: &str, bearer: &str) -> Result<Vec<DoistProject>, String> {
    let projects_url = format!("{}/projects", base_url);
    match ureq::get(projects_url).header("Authorization", bearer).call() {
        Ok(mut response) => {
            Ok(response.body_mut().read_json::<Vec<DoistProject>>().unwrap())
        },
        Err(err) => {
            Err(err.to_string())
        }
    }
}

fn get_project_by_id(base_url: &str, bearer: &str, id: &str) -> Result<DoistProject, String> {
    let projects_by_id = format!("{}/projects/{}", base_url, id);
    match ureq::get(projects_by_id).header("Authorization", bearer).call() {
        Ok(mut response) => {
            Ok(response.body_mut().read_json::<DoistProject>().unwrap())
        },
        Err(err) => {
            Err(err.to_string())
        }
    }
}

fn get_tasks_by_id(base_url: &str, bearer: &str, filters: DoistTaskFilters ) -> Result<Vec<DoistTask>, String> {
    let tasks_url = format!("{base_url}/tasks");
    let query = filters.build_query();
    match ureq::get(tasks_url).header("Authorization", bearer).query_pairs(query).call() {
        Ok(mut response) => {
            let tasks = response.body_mut().read_json::<Vec<DoistTask>>().unwrap();
            let mut hmtasks: HashMap<String, DoistTask> = HashMap::new();

            // We consume tasks here 
            tasks.into_iter().for_each(|mut task| {
                task.depth_level = match task.parent_id {
                    Some(_) => Some(1),
                    None => Some(0),
                };
                
                hmtasks.insert(task.id.clone(), task);
            });

            // New reference counter for HashMap
            let rchm = Rc::new(&hmtasks);

            // We define the recursive depth level function 
            fn get_depth_level(task: &DoistTask, hm: Rc<&HashMap<String, DoistTask>>, depth: u8) -> u8 {
                match task.parent_id.clone() {
                    Some(id) =>{
                        get_depth_level(
                            hm.get(&id as &str).unwrap(),
                            hm,
                            depth + 1
                        )
                    },
                    None => depth,
                }
            }

            let hh = Rc::clone(&rchm);
            let mut hmtmp: HashMap<String, u8> = HashMap::new(); // Hashmap of id and depth
            hh.values().for_each(|t|{
                hmtmp.insert(t.id.clone(), get_depth_level(&t, Rc::clone(&rchm), 0));
            });

            hmtasks.values_mut().for_each(|task|{
                task.depth_level = Some(*hmtmp.get(&task.id.clone()).unwrap());
                task.content = task.content.replacen("*","", 1);
            });

            let mut results: Vec<DoistTask> = Vec::new();
            hmtasks.into_iter().for_each(|(_, task)| results.push(task));

            Ok(results)
        }
        Err(err) => {
            Err(err.to_string())
        }
    }
}

fn process_projects(vec_proj: Vec<DoistProject>) -> HashMap<String, DoistProject> {
    let mut projects_map: HashMap<String, DoistProject> = HashMap::new();
    vec_proj.into_iter().for_each(|project| {
        projects_map.insert(project.id.clone(), project);
    });
    projects_map
}


fn main() {

    // https://api.todoist.com/rest/v2/projects \
    let bearer = format!("Bearer {}", get_api_token());
    let url = "https://api.todoist.com/rest/v2";
    let mut projects: Vec<DoistProject> = Vec::new();

    projects = get_all_projects(&url, &bearer).unwrap();
    let mut projects = process_projects(projects);
    projects.values_mut().for_each(|proj| {
        let filters = DoistTaskFilters {
            filters: vec![TaskFilterKind::ProjectId(proj.id.clone())]
        };
        
        proj.tasks = match get_tasks_by_id(&url, &bearer, filters) {
            Ok(tasks) => Some(tasks),
            Err(err) => None
        };
    });
    
    let mut buffer = fs::File::create(&format!("{}/todoist.org", FILE_PATH)).expect(&format!("Should be able to open or create file at {}", FILE_PATH));
    projects.drain().for_each(|(key, proj)| {
        write!(buffer, "{}\n", proj);
    });

    

}

