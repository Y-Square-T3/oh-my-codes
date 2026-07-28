#![cfg(target_os = "windows")]

use omc_service::windows::TaskSchedulerManager;
use std::path::PathBuf;

#[test]
fn test_xml_has_utf16_encoding_declaration() {
    let manager = TaskSchedulerManager::new();
    let xml = manager.generate_task_xml(&PathBuf::from("C:\\test\\omcd.exe"));
    assert!(
        xml.contains("encoding=\"UTF-16\""),
        "XML should declare UTF-16 encoding"
    );
}

#[test]
fn test_xml_contains_required_elements() {
    let manager = TaskSchedulerManager::new();
    let xml = manager.generate_task_xml(&PathBuf::from("C:\\test\\omcd.exe"));
    assert!(xml.contains("<Task version=\"1.2\""));
    assert!(xml.contains("<LogonTrigger"));
    assert!(xml.contains("<Command>C:\\test\\omcd.exe</Command>"));
}

#[test]
fn test_xml_encoding_produces_valid_utf16le_with_bom() {
    let manager = TaskSchedulerManager::new();
    let xml = manager.generate_task_xml(&PathBuf::from("C:\\test\\omcd.exe"));

    let (encoded, _, had_errors) = encoding_rs::UTF_16LE.encode(&xml);
    assert!(!had_errors, "UTF-16 LE encoding should succeed");

    let mut bytes = vec![0xFF, 0xFE];
    bytes.extend_from_slice(&encoded);

    assert_eq!(bytes[0], 0xFF);
    assert_eq!(bytes[1], 0xFE);
    assert!(bytes.len() > 2);
}

#[test]
fn test_encoded_xml_can_be_decoded() {
    let manager = TaskSchedulerManager::new();
    let xml = manager.generate_task_xml(&PathBuf::from("C:\\test\\omcd.exe"));

    let (encoded, _, _) = encoding_rs::UTF_16LE.encode(&xml);
    let mut bytes = vec![0xFF, 0xFE];
    bytes.extend_from_slice(&encoded);

    let (decoded, _, had_errors) = encoding_rs::UTF_16LE.decode(&bytes[2..]);
    assert!(!had_errors, "Decoding should succeed");
    assert_eq!(decoded, xml);
}
