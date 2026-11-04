use std::path::Path;

use folisift_core::StyleProfile;
use folisift_render::{RenderOptions, render};

fn options<'a>(style: &'a StyleProfile, root: &'a Path) -> RenderOptions<'a> {
    RenderOptions {
        guide_root: root,
        title: "Tall figure",
        subtitle: "",
        course: "",
        language: "en",
        sources: &[],
        style: Some(style),
        styles_root: None,
        cache_dir: None,
    }
}

#[test]
fn a_tall_svg_is_scaled_into_the_page_in_both_styles() {
    let root = tempfile::tempdir().expect("tempdir");
    let assets = root.path().join("assets");
    std::fs::create_dir_all(&assets).expect("assets");
    std::fs::write(
        assets.join("tall.svg"),
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 400" role="img">
  <title>Tall mechanism</title>
  <desc>A five-stage vertical mechanism used to test page fitting.</desc>
  <rect width="100" height="400" fill="#FAF6EC"/>
  <g fill="#211E19" font-family="Inter" font-size="10">
    <rect x="10" y="10" width="80" height="50" fill="none" stroke="#8A3E2F"/>
    <text x="20" y="40">Stage one</text>
    <rect x="10" y="90" width="80" height="50" fill="none" stroke="#8A3E2F"/>
    <text x="20" y="120">Stage two</text>
    <rect x="10" y="170" width="80" height="50" fill="none" stroke="#8A3E2F"/>
    <text x="20" y="200">Stage three</text>
    <rect x="10" y="250" width="80" height="50" fill="none" stroke="#8A3E2F"/>
    <text x="20" y="280">Stage four</text>
    <rect x="10" y="330" width="80" height="50" fill="none" stroke="#8A3E2F"/>
    <text x="20" y="360">Stage five</text>
  </g>
</svg>"##,
    )
    .expect("svg");
    let content = "---\ntitle: \"Tall figure\"\nlanguage: en\nsources: [PD0]\n---\n\n\
        ::: chapter part=\"B\" title=\"Figure\" toc=\"Figure\"\n:::\n\n\
        ## 1.1 Figure\n\n\
        ![A tall five-stage mechanism scaled to fit the page [PD0, s. 4]](assets/tall.svg)\n";
    for aesthetic in ["paper", "clean"] {
        let style = StyleProfile {
            aesthetic: aesthetic.to_string(),
            language: "en".to_string(),
            source_language: "en".to_string(),
            ..StyleProfile::default()
        };
        let rendered = render(content, &options(&style, root.path())).expect("render");
        let pdf = root.path().join(format!("{aesthetic}.pdf"));
        std::fs::write(&pdf, &rendered.pdf).expect("pdf");
        let audit = folisift_parse::audit_pdf(&pdf, None).expect("audit");
        assert!(audit.pages > 1, "{aesthetic}");
        assert!(audit.overflow_pages.is_empty(), "{aesthetic}: {audit:#?}");
        assert!(
            audit.content_box_pages.is_empty(),
            "{aesthetic}: {audit:#?}"
        );
    }
}

#[test]
fn diagram_width_changes_the_rendered_image_bbox() {
    use folisift_render::diagram::{
        D2_VERSION, DiagramManifest, DiagramSpec, DiagramTheme, POSTPROCESS_VERSION,
        SPEC_SCHEMA_VERSION, TALA_SEEDS, salt, spec_hash, theme_digest,
    };

    let root = tempfile::tempdir().expect("tempdir");
    let guide = root.path();
    let png = image::RgbaImage::from_fn(100, 400, |x, y| {
        if x == 0 || y == 0 || x == 99 || y == 399 {
            image::Rgba([33, 30, 25, 255])
        } else {
            image::Rgba([250, 246, 236, 255])
        }
    });
    let spec = DiagramSpec {
        schema_version: SPEC_SCHEMA_VERSION,
        id: "tall".to_string(),
        kind: "flow".to_string(),
        title: "Tall diagram".to_string(),
        caption: "Tall diagram.".to_string(),
        alt: "A tall diagram used to test width scaling.".to_string(),
        source: "PD0, s. 1".to_string(),
        direction: Some("down".to_string()),
        layout: "tala".to_string(),
        aspect_ratio: None,
        allow_page_span: false,
        width: None,
        height: None,
        max_width: None,
        max_height: None,
        fit: None,
        grid: None,
        vars: std::collections::BTreeMap::new(),
        imports: Vec::new(),
        class_styles: std::collections::BTreeMap::new(),
        sketch: false,
        nodes: vec![
            folisift_render::diagram::Node {
                id: "a".to_string(),
                label: "A".to_string(),
                label_lines: Vec::new(),
                group: None,
                shape: None,
            },
            folisift_render::diagram::Node {
                id: "b".to_string(),
                label: "B".to_string(),
                label_lines: Vec::new(),
                group: None,
                shape: None,
            },
        ],
        edges: vec![folisift_render::diagram::Edge {
            from: "a".to_string(),
            to: "b".to_string(),
            label: None,
            arrowhead: None,
            line: None,
        }],
        groups: Vec::new(),
        actors: Vec::new(),
        messages: Vec::new(),
        notes: Vec::new(),
        classes: Vec::new(),
        tables: Vec::new(),
        node_meta: std::collections::BTreeMap::new(),
    };
    std::fs::create_dir_all(guide.join("diagrams")).expect("diagrams");
    std::fs::create_dir_all(guide.join("data/diagrams")).expect("data diagrams");
    std::fs::create_dir_all(guide.join("assets/generated")).expect("assets");
    std::fs::write(
        guide.join("diagrams/tall.json"),
        serde_json::to_string_pretty(&spec).expect("spec json"),
    )
    .expect("spec");
    let mut png_bytes = std::io::Cursor::new(Vec::new());
    png.write_to(&mut png_bytes, image::ImageFormat::Png)
        .expect("png");
    let png_bytes = png_bytes.into_inner();
    let svg_hash = blake3::hash(&png_bytes).to_hex().to_string();
    std::fs::write(
        guide.join(format!("assets/generated/{svg_hash}.png")),
        png_bytes,
    )
    .expect("png");
    let spec_sha = spec_hash(&spec).expect("spec hash");
    let manifest = DiagramManifest {
        schema_version: SPEC_SCHEMA_VERSION,
        id: "tall".to_string(),
        kind: "flow".to_string(),
        title: "Tall diagram".to_string(),
        caption: "Tall diagram.".to_string(),
        alt: "A tall diagram used to test width scaling.".to_string(),
        source: "PD0, s. 1".to_string(),
        spec_sha256: spec_sha.clone(),
        svg_sha256: svg_hash.clone(),
        svg: format!("assets/generated/{svg_hash}.png"),
        d2_version: D2_VERSION.to_string(),
        layout: "tala".to_string(),
        direction: Some("down".to_string()),
        aspect_ratio: None,
        allow_page_span: false,
        sketch: false,
        tala_seeds: TALA_SEEDS.to_string(),
        salt: salt(&spec_sha),
        fonts: vec!["Inter-Regular.ttf".to_string()],
        font_digest: "test".to_string(),
        theme_digest: theme_digest(&DiagramTheme::default()).expect("theme"),
        postprocess_version: POSTPROCESS_VERSION,
    };
    std::fs::write(
        guide.join("data/diagrams/tall.json"),
        serde_json::to_string_pretty(&manifest).expect("manifest json"),
    )
    .expect("manifest");

    let mut widths = Vec::new();
    for width in ["60%", "90%"] {
        let content = format!(
            "---\ntitle: \"Width test\"\nlanguage: en\nsources: [PD0]\n---\n\n\
             ::: chapter part=\"B\" title=\"Figure\" toc=\"Figure\"\n:::\n\n\
             ## 1.1 Figure\n\n::: diagram id=\"tall\" width=\"{width}\"\n:::\n"
        );
        let style = StyleProfile {
            aesthetic: "paper".to_string(),
            language: "en".to_string(),
            source_language: "en".to_string(),
            ..StyleProfile::default()
        };
        let rendered = render(&content, &options(&style, guide)).expect("render");
        let pdf = guide.join(format!("width-{}.pdf", width.trim_end_matches('%')));
        std::fs::write(&pdf, &rendered.pdf).expect("pdf");
        let mut max_width = 0.0_f64;
        for page in 1..=rendered.pages {
            for image in folisift_parse::image_boxes(&pdf, page).expect("image boxes") {
                max_width = max_width.max(image.width);
            }
        }
        assert!(max_width > 0.0, "{width}: no image");
        widths.push(max_width);
    }
    assert!(
        widths[1] > widths[0] * 1.2,
        "width bbox did not change: {widths:?}"
    );
}
