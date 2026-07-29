use std::path::Path;

#[test]
fn test_launcher_scripts_exist() {
    assert!(Path::new("rainbow-contour.sh").exists());
    assert!(Path::new("rainbow-contour.bat").exists());
}
