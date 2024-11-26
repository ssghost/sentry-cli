use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::LazyLock;
use std::{fs, str};

use assert_cmd::Command;
use regex::bytes::Regex;

use crate::integration::{test_utils::env, MockEndpointBuilder, TestManager};

/// This regex is used to extract the boundary from the content-type header.
/// We need to match the boundary, since it changes with each request.
/// The regex matches the format as specified in
/// https://www.w3.org/Protocols/rfc1341/7_2_Multipart.html.
static CONTENT_TYPE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"^multipart\/form-data; boundary=(?<boundary>[\w'\(\)+,\-\.\/:=? ]{0,69}[\w'\(\)+,\-\.\/:=?])$"#
    )
    .expect("Regex is valid")
});

#[test]
fn command_debug_files_upload() {
    TestManager::new()
        .mock_endpoint(
            MockEndpointBuilder::new("GET", "/api/0/organizations/wat-org/chunk-upload/")
                .with_response_file("debug_files/get-chunk-upload.json"),
        )
        .mock_endpoint(
            MockEndpointBuilder::new(
                "POST",
                "/api/0/projects/wat-org/wat-project/files/difs/assemble/",
            )
            .with_response_file("debug_files/post-difs-assemble.json"),
        )
        .register_trycmd_test("debug_files/upload/debug_files-upload.trycmd")
        .with_default_token();
}

#[test]
fn command_debug_files_upload_pdb() {
    TestManager::new()
        .mock_endpoint(
            MockEndpointBuilder::new("GET", "/api/0/organizations/wat-org/chunk-upload/")
                .with_response_file("debug_files/get-chunk-upload.json"),
        )
        .mock_endpoint(
            MockEndpointBuilder::new(
                "POST",
                "/api/0/projects/wat-org/wat-project/files/difs/assemble/",
            )
            .with_response_body(
                r#"{
                "5f81d6becc51980870acc9f6636ab53d26160763": {
                    "state": "ok",
                    "missingChunks": []
                }
            }"#,
            ),
        )
        .register_trycmd_test("debug_files/upload/debug_files-upload-pdb.trycmd")
        .register_trycmd_test("debug_files/upload/debug_files-upload-pdb-include-sources.trycmd")
        .with_default_token();
}

#[test]
fn command_debug_files_upload_pdb_embedded_sources() {
    TestManager::new()
        .mock_endpoint(
            MockEndpointBuilder::new("GET", "/api/0/organizations/wat-org/chunk-upload/")
                .with_response_file("debug_files/get-chunk-upload.json"),
        )
        .mock_endpoint(
            MockEndpointBuilder::new(
                "POST",
                "/api/0/projects/wat-org/wat-project/files/difs/assemble/",
            )
            .with_response_body(
                r#"{
                "50dd9456dc89cdbc767337da512bdb36b15db6b2": {
                    "state": "ok",
                    "missingChunks": []
                }
            }"#,
            ),
        )
        .register_trycmd_test("debug_files/upload/debug_files-upload-pdb-embedded-sources.trycmd")
        .with_default_token();
}

#[test]
fn command_debug_files_upload_dll_embedded_ppdb_with_sources() {
    TestManager::new()
        .mock_endpoint(
            MockEndpointBuilder::new("GET", "/api/0/organizations/wat-org/chunk-upload/")
                .with_response_file("debug_files/get-chunk-upload.json"),
        )
        .mock_endpoint(
            MockEndpointBuilder::new(
                "POST",
                "/api/0/projects/wat-org/wat-project/files/difs/assemble/",
            )
            .with_response_body(
                r#"{
                "fc1c9e58a65bd4eaf973bbb7e7a7cc01bfdaf15e": {
                    "state": "ok",
                    "missingChunks": []
                }
            }"#,
            ),
        )
        .register_trycmd_test(
            "debug_files/upload/debug_files-upload-dll-embedded-ppdb-with-sources.trycmd",
        )
        .with_default_token();
}

#[test]
fn command_debug_files_upload_mixed_embedded_sources() {
    TestManager::new()
        .mock_endpoint(
            MockEndpointBuilder::new("GET", "/api/0/organizations/wat-org/chunk-upload/")
                .with_response_file("debug_files/get-chunk-upload.json"),
        )
        .mock_endpoint(
            MockEndpointBuilder::new(
                "POST",
                "/api/0/projects/wat-org/wat-project/files/difs/assemble/",
            )
            .with_response_body(
                r#"{
                    "21b76b717dbbd8c89e42d92b29667ac87aa3c124": {
                        "state": "ok",
                        "missingChunks": []
                    }
                }"#,
            ),
        )
        // TODO this isn't tested properly at the moment, because `indicatif` ProgressBar (at least at the current version)
        //      swallows debug logs printed while the progress bar is active and the session is not attended.
        //      See how it's supposed to look like `debug_files-bundle_sources-mixed-embedded-sources.trycmd` and try it out
        //      after an update of `indicatif` to the latest version (currently it's blocked by some other issues).
        .register_trycmd_test("debug_files/upload/debug_files-upload-mixed-embedded-sources.trycmd")
        .with_default_token();
}

#[test]
fn command_debug_files_upload_no_upload() {
    TestManager::new()
        .mock_endpoint(
            MockEndpointBuilder::new("GET", "/api/0/organizations/wat-org/chunk-upload/")
                .with_response_file("debug_files/get-chunk-upload.json"),
        )
        .mock_endpoint(
            MockEndpointBuilder::new(
                "POST",
                "/api/0/projects/wat-org/wat-project/files/difs/assemble/",
            )
            .with_response_file("debug_files/post-difs-assemble.json"),
        )
        .register_trycmd_test("debug_files/upload/debug_files-upload-no-upload.trycmd");
}

#[test]
/// This test ensures that the correct initial call to the debug files assemble endpoint is made.
/// The mock assemble endpoint returns a 200 response simulating the case where all chunks
/// are already uploaded.
fn ensure_correct_assemble_call() {
    let manager = TestManager::new()
        .mock_endpoint(
            MockEndpointBuilder::new("GET", "/api/0/organizations/wat-org/chunk-upload/")
                .with_response_file("debug_files/get-chunk-upload.json"),
        )
        .mock_endpoint(
            MockEndpointBuilder::new(
                "POST",
                "/api/0/projects/wat-org/wat-project/files/difs/assemble/",
            )
            .with_header_matcher("content-type", "application/json")
            .with_response_body(
                r#"{
                "21b76b717dbbd8c89e42d92b29667ac87aa3c124": {
                    "state": "ok",
                    "missingChunks": []
                }
            }"#,
            ),
        );

    let mut command = Command::cargo_bin("sentry-cli").expect("sentry-cli should be available");

    command.args(
        "debug-files upload --include-sources tests/integration/_fixtures/SrcGenSampleApp.pdb"
            .split(' '),
    );

    env::set_all(manager.server_info(), |k, v| {
        command.env(k, v.as_ref());
    });

    let command_result = command.assert();

    // First assert the mock was called as expected, then that the command was successful.
    // This is because failure with the mock assertion can cause the command to fail, and
    // the mock assertion failure is likely more interesting in this case.
    manager.assert_mock_endpoints();
    command_result.success();
}

#[test]
/// This test simulates a full chunk upload (with only one chunk).
/// It verifies that the Sentry CLI makes the expected API calls to the chunk upload endpoint
/// and that the data sent to the chunk upload endpoint is exactly as expected.
/// It also verifies that the correct calls are made to the assemble endpoint.
fn ensure_correct_chunk_upload() {
    let is_first_assemble_call = AtomicBool::new(true);
    let expected_chunk_body =
        fs::read("tests/integration/_expected_requests/debug_files/upload/chunk_upload.bin")
            .expect("expected chunk body file should be present");

    let manager = TestManager::new()
        .mock_endpoint(
            MockEndpointBuilder::new("GET", "/api/0/organizations/wat-org/chunk-upload/")
                .with_response_file("debug_files/get-chunk-upload.json"),
        )
        .mock_endpoint(
            MockEndpointBuilder::new("POST", "/api/0/organizations/wat-org/chunk-upload/")
                .with_response_fn(move |request| {
                    let content_type_headers = request.header("content-type");
                    assert_eq!(
                        content_type_headers.len(),
                        1,
                        "content-type header should be present exactly once, found {} times",
                        content_type_headers.len()
                    );

                    let content_type = content_type_headers[0].as_bytes();

                    let boundary = CONTENT_TYPE_REGEX
                        .captures(content_type)
                        .expect("content-type should match regex")
                        .name("boundary")
                        .expect("boundary should be present")
                        .as_bytes();

                    let boundary_str = str::from_utf8(boundary).expect("boundary should be valid utf-8");

                    let boundary_escaped = regex::escape(boundary_str);

                    let body_regex = Regex::new(&format!(
                        r#"^--{boundary_escaped}(?<chunk_body>(?s-u:.)*?)--{boundary_escaped}--\s*$"#
                    ))
                    .expect("regex should be valid");

                    let body = request.body().expect("body should be readable");

                    let chunk_body = body_regex
                        .captures(body)
                        .expect("body should match regex")
                        .name("chunk_body")
                        .expect("chunk_body section should be present")
                        .as_bytes();

                    assert_eq!(chunk_body, expected_chunk_body);

                    vec![] // Client does not expect a response body
                }),
        )
        .mock_endpoint(
            MockEndpointBuilder::new(
                "POST",
                "/api/0/projects/wat-org/wat-project/files/difs/assemble/",
            )
            .with_header_matcher("content-type", "application/json")
            .with_matcher(r#"{"21b76b717dbbd8c89e42d92b29667ac87aa3c124":{"name":"SrcGenSampleApp.pdb","debug_id":"c02651ae-cd6f-492d-bc33-0b83111e7106-8d8e7c60","chunks":["21b76b717dbbd8c89e42d92b29667ac87aa3c124"]}}"#)
            .with_response_fn(move |_| {
                if is_first_assemble_call.swap(false, Ordering::Relaxed) {
                    r#"{
                        "21b76b717dbbd8c89e42d92b29667ac87aa3c124": {
                            "state": "not_found",
                            "missingChunks": ["21b76b717dbbd8c89e42d92b29667ac87aa3c124"]
                        }
                    }"#
                } else {
                    r#"{
                        "21b76b717dbbd8c89e42d92b29667ac87aa3c124": {
                            "state": "created",
                            "missingChunks": []
                        }
                    }"#
                }
                .into()
            })
            .expect(2),
        );

    let mut command = Command::cargo_bin("sentry-cli").expect("sentry-cli should be available");

    command.args(
        "debug-files upload --include-sources tests/integration/_fixtures/SrcGenSampleApp.pdb"
            .split(' '),
    );

    env::set_all(manager.server_info(), |k, v| {
        command.env(k, v.as_ref());
    });

    let command_result = command.assert();

    // First assert the mock was called as expected, then that the command was successful.
    // This is because failure with the mock assertion can cause the command to fail, and
    // the mock assertion failure is likely more interesting in this case.
    manager.assert_mock_endpoints();
    command_result.success();
}
