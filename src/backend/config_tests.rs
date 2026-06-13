use super::*;
use std::fs;

#[derive(Default, Clone, Debug)]
struct TestFields {
    foo: String,
    bar: i32,
}

impl ConfigFields for TestFields {
    fn parse_field(&mut self, key: &str, val: &str) {
        match key {
            "foo" => self.foo = val.to_string(),
            "bar" => if let Ok(n) = val.parse::<i32>() { self.bar = n; },
            _ => {}
        }
    }
    fn serialize_fields(&self) -> Vec<(String, String)> {
        vec![
            ("foo".to_string(), self.foo.clone()),
            ("bar".to_string(), self.bar.to_string()),
        ]
    }
}

#[test]
fn test_write_file_atomic() {
    let temp_dir = std::env::temp_dir().join(format!(
        "ignite_backend_cfg_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros()
    ));
    fs::create_dir_all(&temp_dir).unwrap();
    let file_path = temp_dir.join("test.txt");

    write_file_atomic(&file_path, "hello world").unwrap();
    let content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "hello world");

    // Overwrite
    write_file_atomic(&file_path, "hello again").unwrap();
    let content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "hello again");

    fs::remove_dir_all(&temp_dir).unwrap();
}

#[test]
fn test_generic_config_load_save() {
    let temp_dir = std::env::temp_dir().join(format!(
        "ignite_backend_cfg_test_save_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros()
    ));
    fs::create_dir_all(&temp_dir).unwrap();

    // Set APPDATA/XDG variables to point to temp_dir so config_path is in temp_dir
    let original_appdata = std::env::var("APPDATA").ok();
    let original_xdg = std::env::var("XDG_CONFIG_HOME").ok();
    let original_home = std::env::var("HOME").ok();

    unsafe {
        std::env::set_var("APPDATA", &temp_dir);
        std::env::set_var("XDG_CONFIG_HOME", &temp_dir);
        std::env::set_var("HOME", &temp_dir);
    }

    let fields = TestFields {
        foo: "value1".to_string(),
        bar: 42,
    };
    let app_cfg = AppConfig { fields };
    
    // Save
    app_cfg.save("test_app", "cfg.yaml", "Test Header").unwrap();

    // Load
    let loaded = AppConfig::<TestFields>::load("test_app", "cfg.yaml");
    assert_eq!(loaded.fields.foo, "value1");
    assert_eq!(loaded.fields.bar, 42);

    // Reset env vars
    if let Some(val) = original_appdata {
        unsafe { std::env::set_var("APPDATA", val); }
    } else {
        unsafe { std::env::remove_var("APPDATA"); }
    }
    if let Some(val) = original_xdg {
        unsafe { std::env::set_var("XDG_CONFIG_HOME", val); }
    } else {
        unsafe { std::env::remove_var("XDG_CONFIG_HOME"); }
    }
    if let Some(val) = original_home {
        unsafe { std::env::set_var("HOME", val); }
    } else {
        unsafe { std::env::remove_var("HOME"); }
    }

    fs::remove_dir_all(&temp_dir).unwrap();
}
