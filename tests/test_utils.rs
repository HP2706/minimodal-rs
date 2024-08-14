use std::process::Command;
use std::process::Child;

pub fn start_server(dirname: Option<&str>) -> Result<Child, Box<dyn std::error::Error>> {
    let mut base_path = "src/server".to_string();
    let mut commands = vec!["run", "--bin", "minimodal-server"];
    if let Some(dirname) = dirname {
        base_path.push_str(dirname);
        commands.push("-dirname");
        commands.push(&base_path);
    }
    
    match Command::new("cargo")
        .args(commands)
        .spawn()
    {
        Ok(child) => Ok(child),
        Err(e) => Err(Box::new(e)),
    }
}