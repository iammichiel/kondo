use std::{env, io, path};
use walkdir;

const SYMLINK_FOLLOW: bool = true;

const FILE_CARGO_TOML: &str = "Cargo.toml";
const FILE_PACKAGE_JSON: &str = "package.json";
const FILE_ASSEMBLY_CSHARP: &str = "Assembly-CSharp.csproj";
const FILE_STACK_HASKELL: &str = "stack.yaml";
const FILE_SBT_BUILD: &str = "build.sbt";
const FILE_COMPOSER_JSON: &str = "composer.json";

const PROJECT_CARGO_DIRS: [&str; 1] = ["target"];
const PROJECT_NODE_DIRS: [&str; 1] = ["node_modules"];
const PROJECT_UNITY_DIRS: [&str; 7] = [
    "Library",
    "Temp",
    "Obj",
    "Logs",
    "MemoryCaptures",
    "Build",
    "Builds",
];
const PROJECT_STACK_DIRS: [&str; 1] = [".stack-work"];
const PROJECT_SBT_DIRS: [&str; 2] = ["target", "project/target"];
const PROJECT_COMPOSER_DIRS: [&str; 1] = ["vendor"];

const PROJECT_CARGO_NAME: &str = "Cargo";
const PROJECT_NODE_NAME: &str = "Node";
const PROJECT_UNITY_NAME: &str = "Unity";
const PROJECT_STACK_NAME: &str = "Stack";
const PROJECT_SBT_NAME: &str = "SBT";
const PROJECT_COMPOSER_NAME: &str = "Composer";

fn cargo_project(path: &path::Path) -> Option<Project> {
    let has_cargo_toml = path.read_dir().unwrap().any(|r| match r {
        Ok(de) => de.file_name() == FILE_CARGO_TOML,
        Err(_) => false,
    });
    if has_cargo_toml {
        return Some(Project {
            project_type: ProjectType::Cargo,
            path: path.to_path_buf(),
        });
    }
    None
}

fn node_project(path: &path::Path) -> Option<Project> {
    let has_cargo_toml = path.read_dir().unwrap().any(|r| match r {
        Ok(de) => de.file_name() == FILE_PACKAGE_JSON,
        Err(_) => false,
    });
    if has_cargo_toml {
        return Some(Project {
            project_type: ProjectType::Node,
            path: path.to_path_buf(),
        });
    }
    None
}

fn sbt_project(path: &path::Path) -> Option<Project> {
    let has_cargo_toml = path.read_dir().unwrap().any(|r| match r {
        Ok(de) => de.file_name() == FILE_SBT_BUILD,
        Err(_) => false,
    });
    if has_cargo_toml {
        return Some(Project {
            project_type: ProjectType::SBT,
            path: path.to_path_buf(),
        });
    }
    None
}

fn unity_project(path: &path::Path) -> Option<Project> {
    let has_cargo_toml = path.read_dir().unwrap().any(|r| match r {
        Ok(de) => de.file_name() == FILE_ASSEMBLY_CSHARP,
        Err(_) => false,
    });
    if has_cargo_toml {
        return Some(Project {
            project_type: ProjectType::Unity,
            path: path.to_path_buf(),
        });
    }
    None
}

fn stack_project(path: &path::Path) -> Option<Project> {
    let has_cargo_toml = path.read_dir().unwrap().any(|r| match r {
        Ok(de) => de.file_name() == FILE_STACK_HASKELL,
        Err(_) => false,
    });
    if has_cargo_toml {
        return Some(Project {
            project_type: ProjectType::Stack,
            path: path.to_path_buf(),
        });
    }
    None
}

fn composer_project(path: &path::Path) -> Option<Project> {
    let has_composer_json = path.read_dir().unwrap().any(|r| match r {
        Ok(de) => de.file_name() == FILE_COMPOSER_JSON,
        Err(_) => false,
    });
    if has_composer_json {
        return Some(Project {
            project_type: ProjectType::Composer,
            path: path.to_path_buf(),
        });
    }
    None
}

const PROJECT_TYPES: [fn(path: &path::Path) -> Option<Project>; 6] = [
    cargo_project,
    node_project,
    unity_project,
    stack_project,
    sbt_project,
    composer_project,
];

enum ProjectType {
    Cargo,
    Node,
    Unity,
    Stack,
    SBT,
    Composer,
}

struct Project {
    project_type: ProjectType,
    path: path::PathBuf,
}

impl Project {
    fn name(&self) -> String {
        self.path.to_str().unwrap().to_string()
    }

    fn size(&self) -> u64 {
        match self.project_type {
            ProjectType::Cargo => PROJECT_CARGO_DIRS.iter(),
            ProjectType::Node => PROJECT_NODE_DIRS.iter(),
            ProjectType::Unity => PROJECT_UNITY_DIRS.iter(),
            ProjectType::Stack => PROJECT_STACK_DIRS.iter(),
            ProjectType::SBT => PROJECT_SBT_DIRS.iter(),
            ProjectType::Composer => PROJECT_COMPOSER_DIRS.iter(),
        }
        .map(|p| dir_size(&self.path.join(p)))
        .sum()
    }

    fn type_name(&self) -> &str {
        match self.project_type {
            ProjectType::Cargo => PROJECT_CARGO_NAME,
            ProjectType::Node => PROJECT_NODE_NAME,
            ProjectType::Unity => PROJECT_UNITY_NAME,
            ProjectType::Stack => PROJECT_STACK_NAME,
            ProjectType::SBT => PROJECT_SBT_NAME,
            ProjectType::Composer => PROJECT_COMPOSER_NAME,
        }
    }
}

fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

fn scan<P: AsRef<path::Path>>(path: &P) -> Vec<Project> {
    walkdir::WalkDir::new(path)
        .follow_links(SYMLINK_FOLLOW)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
        .filter_map(|dir| {
            let dir = dir.path();
            PROJECT_TYPES.iter().find_map(|p| p(dir))
        })
        .collect()
}

fn dir_size(path: &path::Path) -> u64 {
    walkdir::WalkDir::new(path)
        .follow_links(SYMLINK_FOLLOW)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e: walkdir::DirEntry| e.metadata())
        .filter_map(|md| md.ok())
        .map(|e| e.len())
        .sum()
}

fn pretty_size(size: u64) -> String {
    let size = size;
    const KIBIBYTE: u64 = 1024;
    const MEBIBYTE: u64 = 1_048_576;
    const GIBIBYTE: u64 = 1_073_741_824;
    const TEBIBYTE: u64 = 1_099_511_627_776;
    const PEBIBYTE: u64 = 1_125_899_906_842_624;
    const EXBIBYTE: u64 = 1_152_921_504_606_846_976;

    let (size, symbol) = match size {
        size if size < KIBIBYTE => (size as f64, "B"),
        size if size < MEBIBYTE => (size as f64 / KIBIBYTE as f64, "KiB"),
        size if size < GIBIBYTE => (size as f64 / MEBIBYTE as f64, "MiB"),
        size if size < TEBIBYTE => (size as f64 / GIBIBYTE as f64, "GiB"),
        size if size < PEBIBYTE => (size as f64 / TEBIBYTE as f64, "TiB"),
        size if size < EXBIBYTE => (size as f64 / PEBIBYTE as f64, "PiB"),
        _ => (size as f64 / EXBIBYTE as f64, "EiB"),
    };

    format!("{:.1}{}", size, symbol)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use io::Write;
    let dir = {
        let mut args: Vec<String> = env::args().collect();
        if args.len() == 2 {
            path::PathBuf::from(args.pop().unwrap())
        } else {
            env::current_dir()?
        }
    };

    let stdout = io::stdout();
    let mut write_handle = stdout.lock();

    writeln!(&mut write_handle, "Scanning {:?}", dir)?;

    let project_dirs = scan(&dir);

    writeln!(&mut write_handle, "{} projects found", project_dirs.len())?;

    writeln!(&mut write_handle, "Calculating savings per project")?;

    let mut total = 0;

    let mut project_sizes: Vec<(u64, String, &str)> = project_dirs
        .iter()
        .map(|p| {
            let size = p.size();
            total += size;
            (size, p.name(), p.type_name())
        })
        .filter(|(size, _, _)| *size > 0)
        .collect();

    project_sizes.sort_unstable_by_key(|p| p.0);

    for (size, name, type_name) in project_sizes.iter() {
        writeln!(
            &mut write_handle,
            "{:>10} {} {}",
            pretty_size(*size),
            type_name,
            name
        )?;
    }

    writeln!(&mut write_handle, "{} possible savings", pretty_size(total))?;

    Ok(())
}
