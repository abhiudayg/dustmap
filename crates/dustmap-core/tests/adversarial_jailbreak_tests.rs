use dustmap_core::{
    safety::{check_raw_traversal, is_system_protected, validate_delete_target, SafetyError},
    trash::{delete_permanently, move_to_trash},
};
use std::fs;
use std::path::{Path, PathBuf};

fn create_jailbreak_fixture(name: &str) -> PathBuf {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let tmp = std::env::temp_dir().join(format!(
        "dustmap_jb_{}_{}_{}",
        name,
        std::process::id(),
        now
    ));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).expect("Failed to create jailbreak fixture root");
    tmp.canonicalize().unwrap_or(tmp)
}

#[test]
fn test_jailbreak_path_traversal_dot_dot_injection() {
    let traversal_payloads = [
        "../../etc/passwd",
        "../../../../../../System/Library",
        "downloads/../../../Library/Keychains",
        "safe_folder/..",
        "safe_folder/../..",
        "/tmp/foo/../../../etc/shadow",
    ];

    for payload in traversal_payloads {
        let err = check_raw_traversal(payload)
            .expect_err(&format!("Payload '{}' must be rejected", payload));
        assert!(
            matches!(err, SafetyError::PathTraversal(_)),
            "Payload '{}' should trigger PathTraversal error",
            payload
        );

        let del_err = validate_delete_target(Path::new(payload), None)
            .expect_err("Delete target must be rejected");
        assert!(matches!(del_err, SafetyError::PathTraversal(_)));
    }
}

#[test]
fn test_jailbreak_null_byte_path_injection() {
    let null_payloads = [
        "/tmp/safe_path\0/etc/passwd",
        "folder\0name/child",
        "\0/System",
    ];

    for payload in null_payloads {
        let err = check_raw_traversal(payload).expect_err("Null byte payload must be rejected");
        assert!(matches!(err, SafetyError::InvalidPath(_)));
    }
}

#[test]
fn test_jailbreak_sip_and_system_protection() {
    let system_targets = [
        "/System",
        "/System/Library",
        "/System/Library/Extensions",
        "/usr",
        "/usr/bin",
        "/usr/lib",
        "/bin",
        "/bin/sh",
        "/sbin",
        "/sbin/launchd",
        "/etc",
        "/var",
        "/private",
        "/Library/Apple",
        "/Library/Application Support/com.apple",
        "/",
        "/Applications",
        "/Library",
        "/Users",
        "/Volumes",
    ];

    for target in system_targets {
        let p = Path::new(target);
        assert!(
            is_system_protected(p),
            "System target '{}' must be flagged as protected",
            target
        );

        let err = validate_delete_target(p, None).expect_err(&format!(
            "Target '{}' must be blocked from deletion",
            target
        ));
        assert!(
            matches!(
                err,
                SafetyError::SystemProtected(_) | SafetyError::SymlinkEscape(_)
            ),
            "Target '{}' must return SystemProtected or SymlinkEscape error, got {:?}",
            target,
            err
        );

        // Verify move_to_trash directly rejects it
        let trash_res = move_to_trash(p);
        assert!(trash_res.is_err(), "move_to_trash must reject '{}'", target);

        // Verify delete_permanently directly rejects it
        let perm_res = delete_permanently(p);
        assert!(
            perm_res.is_err(),
            "delete_permanently must reject '{}'",
            target
        );
    }
}

#[test]
fn test_jailbreak_root_escape_boundary() {
    let root = create_jailbreak_fixture("boundary");
    let safe_child = root.join("safe_child");
    fs::create_dir_all(&safe_child).unwrap();

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let outside_target =
        std::env::temp_dir().join(format!("dustmap_outside_{}_{}", std::process::id(), now));
    fs::create_dir_all(&outside_target).unwrap();

    // Inside target should succeed
    let valid = validate_delete_target(&safe_child, Some(&root));
    assert!(
        valid.is_ok(),
        "Target inside root must be allowed: {:?}",
        valid
    );

    // Outside target must fail with OutsideRoot error
    let invalid = validate_delete_target(&outside_target, Some(&root));
    assert!(invalid.is_err());
    assert!(matches!(
        invalid.unwrap_err(),
        SafetyError::OutsideRoot(_, _)
    ));

    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&outside_target);
}

#[test]
fn test_jailbreak_symlink_escape_to_protected_system() {
    let root = create_jailbreak_fixture("symlink");
    let link_path = root.join("malicious_system_link");

    // Create a symlink pointing to /System
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        if symlink("/System", &link_path).is_ok() {
            let res = validate_delete_target(&link_path, Some(&root));
            assert!(
                res.is_err(),
                "Symlink pointing to /System must be rejected from deletion traversal"
            );
            assert!(
                matches!(res.unwrap_err(), SafetyError::SymlinkEscape(_)),
                "Expected SymlinkEscape error"
            );
        }
    }

    let _ = fs::remove_dir_all(&root);
}
