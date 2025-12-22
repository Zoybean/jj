// Copyright 2025 The Jujutsu Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::common::TestEnvironment;

#[test]
fn test_file_list_untracked() {
    let test_env = TestEnvironment::default();
    test_env.add_config(r#"snapshot.auto-track = "none()""#);

    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let work_dir = test_env.work_dir("repo");

    work_dir.write_file("always-untracked-file", "...");
    work_dir.write_file("initially-untracked-file", "...");
    let sub_dir = work_dir.create_dir("sub");
    sub_dir.write_file("always-untracked", "...");
    sub_dir.write_file("initially-untracked", "...");

    let output = work_dir.run_jj(["file", "list-untracked"]);
    insta::assert_snapshot!(output.normalize_backslash(), @r"
    ? always-untracked-file
    ? initially-untracked-file
    ? sub/
    [EOF]
    ");

    work_dir
        .run_jj([
            "file",
            "track",
            "initially-untracked-file",
            "sub/initially-untracked",
        ])
        .success();

    let output = work_dir.run_jj(["file", "list-untracked"]);
    insta::assert_snapshot!(output.normalize_backslash(), @r"
    ? always-untracked-file
    ? sub/always-untracked
    [EOF]
    ");
}
