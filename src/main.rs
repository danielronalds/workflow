use std::{
    env,
    io::{self, stdout, Write},
};

use colored::Colorize;

mod repo;
use repo::Repo;

mod local_projects;

mod intergrations;

mod config;

pub const PROJECTS_DIR: &str = "Projects/";

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let config = config::get_config().unwrap_or_default();

    let delete_mode = args.contains(&String::from("--delete"));

    let (project, projects) = intergrations::fzf::run_fzf(
        match delete_mode {
            true => "Delete: ",
            false => "Open: ",
        },
        delete_mode,
        &config,
    );

    let selected_projects: Vec<Repo> = projects
        .iter()
        .filter(|x| x.name() == project)
        .map(|x| x.to_owned())
        .collect();

    if let Some(selected_project) = selected_projects.get(0) {
        if delete_mode {
            return delete_project(selected_project);
        }

        if !selected_project.local() {
            if config.github().confirm_cloning()
                && !casual::prompt("Project is not local, clone it to ~/Projects/?")
                    .suffix(" [Y/n] ")
                    .default("y".to_string())
                    .matches(|s| matches!(&*s.trim().to_lowercase(), "n" | "no" | "y" | "yes"))
                    .map(|s| matches!(&*s.to_lowercase(), "y" | "yes"))
            {
                return Ok(());
            }
            intergrations::gh::clone_repo(selected_project)?;
        }

        intergrations::tmuxinator::run_tmuxinator(selected_project, config.tmuxinator())?;
    }

    Ok(())
}

/// Deletes a project from ~/Projects/
///
/// # Parameters
///
/// - `repo` The project to delete
fn delete_project(repo: &Repo) -> io::Result<()> {
    // Checking if the project has a clean work tree
    print!("[{}] clean working tree...", "~".bright_yellow().bold());
    stdout().flush()?;
    println!(
        "\r[{}] clean working tree   \n",
        match intergrations::git::repo_clean_tree(&repo)? {
            false => "⨯".bright_red().bold(),
            true => "✓".bright_green().bold(),
        }
    );

    // Checking if the project has been pushed
    print!("[{}] main pushed...", "~".bright_yellow());
    stdout().flush()?;
    println!(
        "\r[{}] main pushed   \n",
        match intergrations::git::repo_pushed(&repo)? {
            false => "⨯".bright_red().bold(),
            true => "✓".bright_green().bold(),
        }
    );

    println!(
        "{}: These checks are only for the main branch of the repo\n",
        "NOTE".bright_red().bold()
    );

    if !casual::confirm(format!("Delete {}?", repo.name())) {
        return Ok(());
    }
    println!("Deleting tmuxinator config");
    intergrations::tmuxinator::delete_tmuxinator(&repo)?;
    println!("Deleting project from ~/Projects/");
    local_projects::delete_local_project(&repo)?;

    println!("Deleted {}!", repo.name());
    Ok(())
}
