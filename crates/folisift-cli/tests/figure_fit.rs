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
