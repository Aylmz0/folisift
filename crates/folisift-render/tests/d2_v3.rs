use std::process::Command;

use folisift_render::diagram::{self, DiagramTheme};

#[test]
fn v3_fixtures_compile_and_postprocess() {
    let d2 = std::env::var_os("FOLISIFT_D2").map(std::path::PathBuf::from);
    for name in ["sequence-pilot", "class-pilot", "erd-pilot"] {
        let text = std::fs::read_to_string(format!(
            "{}/../../tests/fixtures/diagrams/{name}.json",
            env!("CARGO_MANIFEST_DIR")
        ))
        .expect("fixture");
        let spec = diagram::spec_from_json(&text).expect("valid spec");
        let source = diagram::d2_source(&spec, &DiagramTheme::default()).expect("source");
        assert!(!source.contains("icon:"), "{name}");
        assert!(!source.contains("import "), "{name}");

        let Some(d2) = &d2 else {
            continue;
        };
        let root = tempfile::tempdir().expect("tempdir");
        let input = root.path().join(format!("{name}.d2"));
        let output = root.path().join(format!("{name}.svg"));
        let second = root.path().join(format!("{name}-second.svg"));
        std::fs::write(&input, &source).expect("d2 source");
        let result = Command::new(d2)
            .env_clear()
            .env("PATH", "")
            .env("HOME", root.path())
            .env("TMPDIR", root.path())
            .args(["--layout=tala", "--theme=0", "--tala-seeds=1,2,3"])
            .arg(&input)
            .arg(&output)
            .output()
            .expect("run d2");
        assert!(
            result.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        let rerun = Command::new(d2)
            .env_clear()
            .env("PATH", "")
            .env("HOME", root.path())
            .env("TMPDIR", root.path())
            .args(["--layout=tala", "--theme=0", "--tala-seeds=1,2,3"])
            .arg(&input)
            .arg(&second)
            .output()
            .expect("run d2 again");
        assert!(
            rerun.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&rerun.stderr)
        );
        let svg = std::fs::read_to_string(&output).expect("svg");
        assert_eq!(svg, std::fs::read_to_string(&second).expect("second svg"));
        assert!(svg.contains("<svg"), "{name}");
        assert!(!svg.contains("<script"), "{name}");
        assert!(!svg.contains("<image"), "{name}");
        assert!(!svg.contains("d2plugin-"), "{name}");
        let processed = diagram::postprocess_svg(
            &svg,
            &spec.title,
            &spec.alt,
            "#FAF6EC",
            &format!("test-{name}"),
        )
        .expect("postprocess");
        assert!(processed.contains("role=\"img\""), "{name}");
        assert!(processed.contains("aria-labelledby="), "{name}");
        assert!(processed.contains("<title"), "{name}");
        assert!(processed.contains("<desc"), "{name}");
        assert!(processed.contains("</svg>"), "{name}");
        let baseline = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../../tests/baselines/d2/{name}.svg"));
        if std::env::var("UPDATE_D2_BASELINES").as_deref() == Ok("1") {
            std::fs::create_dir_all(baseline.parent().expect("baseline parent"))
                .expect("baseline dir");
            std::fs::write(&baseline, &processed).expect("write baseline");
        } else if baseline.is_file() {
            assert_eq!(
                processed,
                std::fs::read_to_string(&baseline).expect("read baseline"),
                "{name} diagram baseline changed"
            );
        }
    }
}
