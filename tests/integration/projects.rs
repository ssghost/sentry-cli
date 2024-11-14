use crate::integration::{mock_endpoint, register_test, MockEndpointBuilder};

#[test]
fn command_projects_list() {
    // mock for projects list
    let _server = mock_endpoint(
        MockEndpointBuilder::new("GET", "/api/0/organizations/wat-org/projects/?cursor=", 200)
            .with_response_file("projects/get-projects.json"),
    );
    register_test("projects/*.trycmd");
}
