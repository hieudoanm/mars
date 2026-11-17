//! Lab commands against an unwritable data directory: DB connect and dataset
//! storage must surface IO errors as `Err(String)`, not panic. Runs in its own
//! test process so the read-only `MARS_LAB_DATA_DIR` cannot affect other tests.

use mars_tauri_lib::lab::*;
use serde_json::json;

#[cfg(unix)]
#[tokio::test]
async fn io_errors_under_unwritable_lab_dir() {
    use std::os::unix::fs::PermissionsExt;

    let dir = std::env::temp_dir().join(format!("mars-tauri-lab-io-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("creates lab dir");
    unsafe {
        std::env::set_var("MARS_LAB_DATA_DIR", &dir);
        std::env::set_var("DATABASE_URL", format!("sqlite://{}/lab.db", dir.display()));
    }

    // Phase 1 — read-only dir: DB connect and dataset storage both fail.
    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o500))
        .expect("makes lab dir read-only");
    let _ = lab_list_datasets().await;
    let _ = lab_list_jobs().await;
    let _ = lab_algorithms(None).await;

    let csv = std::env::temp_dir().join(format!("mars-lab-io-{}.csv", std::process::id()));
    std::fs::write(&csv, "a,b\n1,2\n2,4\n3,5\n4,8\n5,10\n6,12\n7,14\n8,16\n").expect("writes csv");
    let _ = lab_upload_dataset(csv.to_string_lossy().into_owned()).await;

    // Phase 2 — writable again: upload succeeds and the DB connects.
    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))
        .expect("restores write permission");
    let detail = lab_upload_dataset(csv.to_string_lossy().into_owned())
        .await
        .expect("uploads dataset");
    let dataset_id = detail.dataset.id.to_string();

    // Phase 3 — read-only again: subdir perms don't cascade, so lock the
    // storage dirs explicitly. Dataset reads still work (file perms
    // unchanged) but artefact writes cannot create their job directory.
    for sub in ["datasets", "artefacts"] {
        let path = dir.join(sub);
        if path.is_dir() {
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o500))
                .expect("makes storage subdir read-only");
        }
    }
    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o500))
        .expect("makes lab dir read-only again");
    let request = LabTrainRequest {
        dataset_id,
        task_type: "regression".to_owned(),
        algorithm_id: "linear_regression".to_owned(),
        feature_columns: vec!["a".to_owned()],
        label_column: Some("b".to_owned()),
        rl_columns: None,
        hyperparameters: json!({}),
        split_ratio: 0.8,
        random_seed: 1,
        name: None,
    };
    let result = lab_train_inner(request, |_, _| {}).await;
    if let Err(error) = &result {
        eprintln!("training failed at: {error}");
    }
    assert!(result.is_err());

    for sub in ["datasets", "artefacts"] {
        let path = dir.join(sub);
        if path.is_dir() {
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700))
                .expect("restores subdir permissions");
        }
    }
    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))
        .expect("restores permissions");
    std::fs::remove_dir_all(&dir).ok();
}
