extern crate gimli;

use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use gimli::read;
use gimli::write::{self, Address, EndianVec};
use gimli::LittleEndian;

fn read_section(section: &str) -> Vec<u8> {
    let mut path = PathBuf::new();
    if let Ok(dir) = env::var("CARGO_MANIFEST_DIR") {
        path.push(dir);
    }
    path.push("fixtures/self");
    path.push(section);

    println!("Reading section \"{}\" at path {:?}", section, path);
    assert!(path.is_file());
    let mut file = File::open(path).unwrap();

    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    buf
}

#[test]
fn test_convert_debug_info() {
    // Convert existing sections
    let debug_info = read_section("debug_info");
    let debug_info = read::DebugInfo::new(&debug_info, LittleEndian);

    let debug_abbrev = read_section("debug_abbrev");
    let debug_abbrev = read::DebugAbbrev::new(&debug_abbrev, LittleEndian);

    let debug_str = read_section("debug_str");
    let debug_str = read::DebugStr::new(&debug_str, LittleEndian);

    let mut strings = write::StringTable::default();
    let units = write::UnitTable::from(
        &debug_info,
        &debug_abbrev,
        &debug_str,
        &mut strings,
        &|address| Some(Address::Absolute(address)),
    ).expect("Should convert compilation units");
    assert_eq!(units.count(), 23);
    let entries = (0..units.count())
        .map(|id| units.get(write::UnitId(id)).count())
        .fold(0, |x, y| x + y);
    assert_eq!(entries, 29560);
    assert_eq!(strings.count(), 3921);

    // Write to new sections
    let mut write_debug_str = write::DebugStr::from(EndianVec::new(LittleEndian));
    let debug_str_offsets = strings
        .write(&mut write_debug_str)
        .expect("Should write strings");

    let debug_str_data = write_debug_str.slice();
    assert_eq!(debug_str_offsets.count(), 3921);
    assert_eq!(debug_str_data.len(), 144731);

    let mut write_debug_info = write::DebugInfo::from(EndianVec::new(LittleEndian));
    let mut write_debug_abbrev = write::DebugAbbrev::from(EndianVec::new(LittleEndian));
    units
        .write(
            &mut write_debug_info,
            &mut write_debug_abbrev,
            &debug_str_offsets,
        ).expect("Should write units");
    let debug_info_data = write_debug_info.slice();
    let debug_abbrev_data = write_debug_abbrev.slice();
    assert_eq!(debug_info_data.len(), 394930);
    assert_eq!(debug_abbrev_data.len(), 1282);

    // Convert new sections
    let debug_info = read::DebugInfo::new(debug_info_data, LittleEndian);
    let debug_abbrev = read::DebugAbbrev::new(debug_abbrev_data, LittleEndian);
    let debug_str = read::DebugStr::new(debug_str_data, LittleEndian);

    let mut strings = write::StringTable::default();
    let units = write::UnitTable::from(
        &debug_info,
        &debug_abbrev,
        &debug_str,
        &mut strings,
        &|address| Some(Address::Absolute(address)),
    ).expect("Should convert compilation units");
    assert_eq!(units.count(), 23);
    let entries = (0..units.count())
        .map(|id| units.get(write::UnitId(id)).count())
        .fold(0, |x, y| x + y);
    assert_eq!(entries, 29560);
    assert_eq!(strings.count(), 3921);
}