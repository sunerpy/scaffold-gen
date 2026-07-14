use super::*;

#[test]
fn rewrite_requires_python_updates_only_the_anchored_requirement() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let pyproject = tmp.path().join("pyproject.toml");
    std::fs::write(
        &pyproject,
        "[project]\nname = \"demo\"\nrequires-python = \">=3.14\"\ndependencies = []\n",
    )
    .expect("write pyproject");

    rewrite_requires_python(tmp.path()).expect("rewrite requires-python");

    let content = std::fs::read_to_string(pyproject).expect("read pyproject");
    assert_eq!(
        content,
        "[project]\nname = \"demo\"\nrequires-python = \">=3.12\"\ndependencies = []\n"
    );
}
