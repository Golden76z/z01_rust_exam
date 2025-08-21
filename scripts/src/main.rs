use anyhow::{Context, Result};
use console::style;
use dialoguer::{Confirm, Select, theme::ColorfulTheme};
use rand::rng;
use rand::seq::SliceRandom;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() -> Result<()> {
    let main_folder = "exercice";

    // Create the main exercice folder
    fs::create_dir_all(main_folder).context("Failed to create main exercice folder")?;
    std::env::set_current_dir(main_folder).context("Failed to change to exercice directory")?;

    // Look into ../../lib to discover available exams
    let lib_path = Path::new("../../lib");
    let exams: Vec<String> = fs::read_dir(lib_path)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    if exams.is_empty() {
        anyhow::bail!("No exams found in lib/");
    }

    // Prompt user to select which exam to create
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Which exam do you want to create/reshuffle?")
        .default(0)
        .items(&exams)
        .interact()?;

    let selected_exam = &exams[selection];
    let exam_path = Path::new(selected_exam);

    // Check if exam already exists and prompt for overwrite
    if exam_path.exists() {
        let overwrite = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "Exam '{}' already exists. Overwrite and reshuffle?",
                selected_exam
            ))
            .default(false)
            .interact()?;

        if !overwrite {
            println!("{}", style("Operation cancelled.").yellow());
            return Ok(());
        }

        fs::remove_dir_all(exam_path).context("Failed to remove existing exam folder")?;
    }

    println!("Creating {} ...", style(selected_exam).green());
    create_exam_with_levels(selected_exam)?;
    println!("{}", style("Exam structure created successfully!").green());
    Ok(())
}

fn create_exam_with_levels(exam_name: &str) -> Result<()> {
    let exam_path = Path::new(exam_name);
    fs::create_dir_all(exam_path).context("Failed to create exam folder")?;

    // Levels are folders inside ../../lib/{exam_name}
    let lib_exam_path = Path::new("../../lib").join(exam_name);
    let mut levels: Vec<PathBuf> = fs::read_dir(&lib_exam_path)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
        .map(|e| e.path())
        .collect();

    // Sort levels numerically (1,2,3...)
    levels.sort_by_key(|p| {
        p.file_name()
            .and_then(|n| n.to_string_lossy().parse::<u32>().ok())
            .unwrap_or(9999)
    });

    let mut rng = rng();

    for (i, level_dir) in levels.iter().enumerate() {
        let level_num = i + 1;

        // Get all exercises in this level
        let mut exercises: Vec<PathBuf> = fs::read_dir(level_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
            .map(|e| e.path())
            .collect();

        if exercises.is_empty() {
            println!(
                "{} No exercises found in {:?}",
                style("⚠").yellow(),
                level_dir
            );
            continue;
        }

        // Pick one exercise at random
        exercises.shuffle(&mut rng);
        let chosen = exercises.pop().unwrap();
        let exercise_name = chosen.file_name().unwrap().to_string_lossy().to_string();

        let level_path = exam_path.join(level_num.to_string());
        fs::create_dir_all(&level_path).context("Failed to create level folder")?;
        let exercise_path = level_path.join(&exercise_name);

        create_cargo_project(&exercise_path, &exercise_name)?;
        copy_template_files(&exercise_path, exam_name, level_num, &exercise_name)?;

        println!(
            "  Level {}: {} {}",
            level_num,
            style("✓").green(),
            exercise_name
        );
    }

    Ok(())
}

fn create_cargo_project(path: &Path, name: &str) -> Result<()> {
    let status = Command::new("cargo")
        .arg("new")
        .arg("--bin")
        .arg(name)
        .current_dir(path.parent().unwrap())
        .status()
        .context("Failed to run cargo new")?;

    if !status.success() {
        anyhow::bail!("Cargo new failed for {}", name);
    }

    let temp_path = path.parent().unwrap().join(name);
    if temp_path.exists() && temp_path != path {
        fs::rename(&temp_path, path).context("Failed to move cargo project")?;
    }
    Ok(())
}

fn copy_template_files(
    exercise_path: &Path,
    exam_name: &str,
    level: usize,
    exercise_name: &str,
) -> Result<()> {
    let lib_template_path = Path::new("../../lib")
        .join(exam_name)
        .join(level.to_string())
        .join(exercise_name);

    if lib_template_path.exists() {
        for file in ["main.rs", "lib.rs", "README.md", "Cargo.toml"] {
            let src = lib_template_path.join(file);
            if src.exists() {
                let dest = if file.ends_with(".rs") {
                    exercise_path.join("src").join(file)
                } else {
                    exercise_path.join(file)
                };
                fs::copy(&src, &dest).with_context(|| format!("Failed to copy {}", file))?;
            }
        }
    }

    Ok(())
}
