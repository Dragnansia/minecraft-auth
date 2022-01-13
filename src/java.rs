use std::fs;

#[cfg(target_os = "linux")]
const JAVA_PATH: &str = "/usr/lib/jvm/";
#[cfg(target_os = "linux")]
const JAVA_FOLDER_NAME_B8: &str = "java-1.";
#[cfg(target_os = "linux")]
const JAVA_PATH_END_B8: &str = "jre/bin/java";
#[cfg(target_os = "linux")]
const JAVA_FOLDER_NAME_A8: &str = "java-";
#[cfg(target_os = "linux")]
const JAVA_PATH_END_A8: &str = "bin/java";

#[cfg(target_os = "windows")]
const JAVA_PATH: &str = "C:\\Program Files\\Java\\";
#[cfg(target_os = "windows")]
const JAVA_PATH_END_B8: &str = "bin/java.exe";
#[cfg(target_os = "windows")]
const JAVA_FOLDER_NAME_B8: &str = "jre1.";
#[cfg(target_os = "windows")]
const JAVA_PATH_END_A8: &str = "bin/java.exe";
#[cfg(target_os = "windows")]
const JAVA_FOLDER_NAME_A8: &str = "jdk-";

/// Try to find java version and return path
/// if version found
pub fn find_java_version(version: u8) -> Option<String> {
    let (folder_java_begin, end_path) = if version <= 8 {
        (JAVA_FOLDER_NAME_B8, JAVA_PATH_END_B8)
    } else {
        (JAVA_FOLDER_NAME_A8, JAVA_PATH_END_A8)
    };

    let folder_start = format!("{}{}", folder_java_begin, version);
    if let Ok(mut dir_content) = fs::read_dir(JAVA_PATH) {
        if let Some(dir) = dir_content.find(|el| {
            el.as_ref()
                .unwrap()
                .file_name()
                .to_str()
                .unwrap()
                .starts_with(&folder_start)
        }) {
            Some(format!(
                "{}/{}",
                dir.unwrap().path().to_str().unwrap(),
                end_path
            ))
        } else {
            None
        }
    } else {
        None
    }
}
