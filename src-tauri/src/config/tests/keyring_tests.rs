use super::*;

#[test]
fn secret_ref_reference_formats_sources_without_secret_values() {
    let env_reference = secret_ref_reference(&SecretRef::FromEnv {
        from_env: String::from("OPENAI_API_KEY"),
    });
    let file_reference = secret_ref_reference(&SecretRef::FromFile {
        from_file: String::from("/secure/openai.key"),
    });
    let keyring_reference = secret_ref_reference(&SecretRef::FromKeyring {
        from_keyring: KeyringRef {
            service: String::from("blind-browser"),
            account: String::from("planner/openai-default"),
        },
    });

    assert_eq!(env_reference, "Environment variable: OPENAI_API_KEY");
    assert_eq!(file_reference, "File reference: /secure/openai.key");
    assert_eq!(
        keyring_reference,
        "OS keyring entry: blind-browser / planner/openai-default"
    );
    assert!(!env_reference.contains("super-secret"));
    assert!(!file_reference.contains("super-secret"));
    assert!(!keyring_reference.contains("super-secret"));
}

#[test]
fn resolve_secret_ref_reads_all_supported_reference_types() {
    let env_var_name = format!(
        "BLIND_BROWSER_TEST_SECRET_{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after UNIX_EPOCH")
            .as_nanos()
    );
    let file_path = test_temp_path("resolve_secret_file", "secret.txt");

    std::env::set_var(&env_var_name, "env-secret");
    fs::write(&file_path, "file-secret").expect("secret file should write");
    set_keyring_secret("blind-browser", "tests/keyring-secret", "keyring-secret")
        .expect("keyring secret should store");

    let env_secret = resolve_secret_ref(&SecretRef::FromEnv {
        from_env: env_var_name.clone(),
    })
    .expect("env secret should resolve");
    let file_secret = resolve_secret_ref(&SecretRef::FromFile {
        from_file: file_path.display().to_string(),
    })
    .expect("file secret should resolve");
    let keyring_secret = resolve_secret_ref(&SecretRef::FromKeyring {
        from_keyring: KeyringRef {
            service: String::from("blind-browser"),
            account: String::from("tests/keyring-secret"),
        },
    })
    .expect("keyring secret should resolve");

    assert_eq!(env_secret, "env-secret");
    assert_eq!(file_secret, "file-secret");
    assert_eq!(keyring_secret, "keyring-secret");

    std::env::remove_var(&env_var_name);
    let _ = fs::remove_file(&file_path);
}

#[test]
fn resolve_secret_ref_rejects_missing_or_empty_values() {
    let missing_env_name = format!(
        "BLIND_BROWSER_TEST_MISSING_SECRET_{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after UNIX_EPOCH")
            .as_nanos()
    );
    let empty_file_path = test_temp_path("resolve_empty_secret_file", "empty-secret.txt");

    fs::write(&empty_file_path, "").expect("empty secret file should write");
    set_keyring_secret("blind-browser", "tests/empty-keyring-secret", "")
        .expect("empty keyring secret should store");

    let missing_env_error = resolve_secret_ref(&SecretRef::FromEnv {
        from_env: missing_env_name,
    })
    .expect_err("missing env secret should fail");
    let empty_file_error = resolve_secret_ref(&SecretRef::FromFile {
        from_file: empty_file_path.display().to_string(),
    })
    .expect_err("empty file secret should fail");
    let empty_keyring_error = resolve_secret_ref(&SecretRef::FromKeyring {
        from_keyring: KeyringRef {
            service: String::from("blind-browser"),
            account: String::from("tests/empty-keyring-secret"),
        },
    })
    .expect_err("empty keyring secret should fail");

    assert!(missing_env_error.contains("failed to read environment variable"));
    assert_eq!(empty_file_error, "resolved secret value was empty");
    assert_eq!(empty_keyring_error, "resolved secret value was empty");

    let _ = fs::remove_file(&empty_file_path);
}


#[test]
fn persists_remote_planner_api_key_to_keyring_reference_and_reloads_it() {
    let path = test_config_path("persist_remote_planner_api_key");

    let persisted = AppConfig::persist_remote_api_key_at_path(
        &path,
        "openai-default",
        "super-secret",
        "planner",
    )
    .expect("remote planner API key should persist successfully");
    let reloaded = AppConfig::load_from_path(&path).expect("persisted config should reload");

    let expected_keyring_ref = keyring_ref_for_remote_api_key("planner", "openai-default");

    let expected_secret_ref = SecretRef::FromKeyring {
        from_keyring: KeyringRef {
            service: expected_keyring_ref.service.clone(),
            account: expected_keyring_ref.account.clone(),
        },
    };

    assert_eq!(
        persisted
            .remote_planner_profiles
            .get("openai-default")
            .expect("planner profile should remain present")
            .api_key,
        expected_secret_ref
    );
    assert_eq!(
        reloaded
            .remote_planner_profiles
            .get("openai-default")
            .expect("planner profile should reload")
            .api_key,
        expected_secret_ref
    );
    assert_eq!(
        resolve_secret_ref(&expected_secret_ref).expect("keyring secret should resolve"),
        "super-secret"
    );

    if let Some(parent) = path.parent() {
        let _ = std::fs::remove_dir_all(parent);
    }
}

#[test]
fn rejects_empty_remote_api_key_persistence_input() {
    let path = test_config_path("reject_empty_remote_api_key");

    let error =
        AppConfig::persist_remote_api_key_at_path(&path, "openai-default", "   ", "planner")
            .expect_err("empty API key should be rejected");

    match error {
        ConfigError::Validation(message) => {
            assert!(message.contains("non-empty API key value"));
        }
        other => panic!("expected validation error, got {other}"),
    }
}

