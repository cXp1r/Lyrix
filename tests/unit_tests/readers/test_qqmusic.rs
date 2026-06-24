use lyrix::readers::qqmusic::{
    find_qqmusic_qrc_path_by_metadata_in_dir, is_qqmusic_qrc_filename, parse_qqmusic_qrc_filename,
};
use lyrix::models::TrackMetadata;
use std::fs;
use std::path::PathBuf;

#[test]
fn parses_qqmusic_qrc_filename() {
    let info = parse_qqmusic_qrc_filename(
        "安月名莉子 (Azuna Riko) - rise - 303 - TVアニメ「やがて君になる」オープニングテーマ「君にふれて」 (触摸你)_qm.qrc",
    )
    .expect("should parse");

    assert_eq!(info.artist, "安月名莉子 (Azuna Riko)");
    assert_eq!(info.title, "rise");
    assert_eq!(info.index, "303");
    assert_eq!(
        info.album,
        "TVアニメ「やがて君になる」オープニングテーマ「君にふれて」 (触摸你)"
    );
    assert!(is_qqmusic_qrc_filename(
        "安月名莉子 (Azuna Riko) - rise - 303 - TVアニメ「やがて君になる」オープニングテーマ「君にふれて」 (触摸你)_qm.qrc"
    ));
}

#[test]
fn rejects_non_matching_qqmusic_qrc_filename() {
    assert!(parse_qqmusic_qrc_filename("bad_name.qrc").is_none());
    assert!(parse_qqmusic_qrc_filename("artist - title - abc - album_qm.qrc").is_none());
    assert!(parse_qqmusic_qrc_filename("artist - title - 303 - album.txt").is_none());
}

#[test]
fn finds_qqmusic_qrc_path_by_metadata_in_fixture_dir() {
    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("unit_tests")
        .join("parsers")
        .join("eg");
    let metadata = TrackMetadata {
        title: Some("rise".to_string()),
        artist: Some("安月名莉子 (Azuna Riko)".to_string()),
        album: Some(
            "TVアニメ「やがて君になる」オープニングテーマ「君にふれて」 (触摸你)".to_string(),
        ),
        ..Default::default()
    };

    let found = find_qqmusic_qrc_path_by_metadata_in_dir(&fixture_dir, &metadata)
        .expect("search should succeed")
        .expect("fixture qrc should exist");

    let expected = fs::read_dir(&fixture_dir)
        .expect("fixture dir readable")
        .find_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("qrc") {
                Some(path)
            } else {
                None
            }
        })
        .expect("fixture qrc should exist");

    assert_eq!(found, expected);
}
