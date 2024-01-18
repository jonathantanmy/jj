// Copyright 2022 The Jujutsu Authors
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

use std::path::Path;

use crate::common::TestEnvironment;

pub mod common;

/// Test adding a second workspace
#[test]
fn test_workspaces_add_second_workspace() {
    let test_env = TestEnvironment::default();
    test_env.jj_cmd_ok(test_env.env_root(), &["init", "--git", "main"]);
    let main_path = test_env.env_root().join("main");
    let secondary_path = test_env.env_root().join("secondary");

    std::fs::write(main_path.join("file"), "contents").unwrap();
    test_env.jj_cmd_ok(&main_path, &["commit", "-m", "initial"]);

    let stdout = test_env.jj_cmd_success(&main_path, &["workspace", "list"]);
    insta::assert_snapshot!(stdout, @r###"
    default: rlvkpnrz e0e6d567 (empty) (no description set)
    "###);

    let (stdout, stderr) = test_env.jj_cmd_ok(
        &main_path,
        &["workspace", "add", "--name", "second", "../secondary"],
    );
    insta::assert_snapshot!(stdout.replace('\\', "/"), @"");
    insta::assert_snapshot!(stderr.replace('\\', "/"), @r###"
    Created workspace in "../secondary"
    Working copy now at: rzvqmyuk 397eac93 (empty) (no description set)
    Parent commit      : qpvuntsm 7d308bc9 initial
    Added 1 files, modified 0 files, removed 0 files
    "###);

    // Can see the working-copy commit in each workspace in the log output. The "@"
    // node in the graph indicates the current workspace's working-copy commit.
    insta::assert_snapshot!(get_log_output(&test_env, &main_path), @r###"
    ◉  397eac932ad3c349b2659fd2eb035a4dd3da4193 second@
    │ @  e0e6d5672858dc9a57ec5b772b7c4f3270ed0223 default@
    ├─╯
    ◉  7d308bc9d934c53c6cc52935192e2d6ac5d78cfd
    ◉  0000000000000000000000000000000000000000
    "###);
    insta::assert_snapshot!(get_log_output(&test_env, &secondary_path), @r###"
    @  397eac932ad3c349b2659fd2eb035a4dd3da4193 second@
    │ ◉  e0e6d5672858dc9a57ec5b772b7c4f3270ed0223 default@
    ├─╯
    ◉  7d308bc9d934c53c6cc52935192e2d6ac5d78cfd
    ◉  0000000000000000000000000000000000000000
    "###);

    // Both workspaces show up when we list them
    let stdout = test_env.jj_cmd_success(&main_path, &["workspace", "list"]);
    insta::assert_snapshot!(stdout, @r###"
    default: rlvkpnrz e0e6d567 (empty) (no description set)
    second: rzvqmyuk 397eac93 (empty) (no description set)
    "###);
}

/// Test adding a second workspace while the current workspace is editing a
/// merge
#[test]
fn test_workspaces_add_second_workspace_on_merge() {
    let test_env = TestEnvironment::default();
    test_env.jj_cmd_ok(test_env.env_root(), &["init", "--git", "main"]);
    let main_path = test_env.env_root().join("main");

    test_env.jj_cmd_ok(&main_path, &["describe", "-m=left"]);
    test_env.jj_cmd_ok(&main_path, &["new", "@-", "-m=right"]);
    test_env.jj_cmd_ok(&main_path, &["new", "all:@-+", "-m=merge"]);

    let stdout = test_env.jj_cmd_success(&main_path, &["workspace", "list"]);
    insta::assert_snapshot!(stdout, @r###"
    default: zsuskuln 21a0ea6d (empty) merge
    "###);

    test_env.jj_cmd_ok(
        &main_path,
        &["workspace", "add", "--name", "second", "../secondary"],
    );

    // The new workspace's working-copy commit shares all parents with the old one.
    insta::assert_snapshot!(get_log_output(&test_env, &main_path), @r###"
    ◉    6d4c2b8ab610148410b6a346d900cb7ad6298ede second@
    ├─╮
    │ │ @  21a0ea6d1c86c413cb30d7f7d216a74754b148c4 default@
    ╭─┬─╯
    │ ◉  09ba8d9dfa21b8572c2361e6947ca024d8f0a198
    ◉ │  1694f2ddf8ecf9e55ca3cd9554bc0654186b07e0
    ├─╯
    ◉  0000000000000000000000000000000000000000
    "###);
}

/// Test adding a workspace, but at a specific revision using '-r'
#[test]
fn test_workspaces_add_workspace_at_revision() {
    let test_env = TestEnvironment::default();
    test_env.jj_cmd_ok(test_env.env_root(), &["init", "--git", "main"]);
    let main_path = test_env.env_root().join("main");
    let secondary_path = test_env.env_root().join("secondary");

    std::fs::write(main_path.join("file-1"), "contents").unwrap();
    test_env.jj_cmd_ok(&main_path, &["commit", "-m", "first"]);

    std::fs::write(main_path.join("file-2"), "contents").unwrap();
    test_env.jj_cmd_ok(&main_path, &["commit", "-m", "second"]);

    let stdout = test_env.jj_cmd_success(&main_path, &["workspace", "list"]);
    insta::assert_snapshot!(stdout, @r###"
    default: kkmpptxz 2801c219 (empty) (no description set)
    "###);

    let (_, stderr) = test_env.jj_cmd_ok(
        &main_path,
        &[
            "workspace",
            "add",
            "--name",
            "second",
            "../secondary",
            "-r",
            "@--",
        ],
    );
    insta::assert_snapshot!(stderr.replace('\\', "/"), @r###"
    Created workspace in "../secondary"
    Working copy now at: zxsnswpr e6baf9d9 (empty) (no description set)
    Parent commit      : qpvuntsm e7d7dbb9 first
    Added 1 files, modified 0 files, removed 0 files
    "###);

    // Can see the working-copy commit in each workspace in the log output. The "@"
    // node in the graph indicates the current workspace's working-copy commit.
    insta::assert_snapshot!(get_log_output(&test_env, &main_path), @r###"
    ◉  e6baf9d9cfd0b616eac110fc5826b7eca63cffc5 second@
    │ @  2801c219094d5d49a2c5697ba1c34b9593d2c2a9 default@
    │ ◉  4ec5df5a189c813f8cc66aeb35e007929ebb46bc
    ├─╯
    ◉  e7d7dbb91c5a543ea680711093e689916d5f31df
    ◉  0000000000000000000000000000000000000000
    "###);
    insta::assert_snapshot!(get_log_output(&test_env, &secondary_path), @r###"
    @  e6baf9d9cfd0b616eac110fc5826b7eca63cffc5 second@
    │ ◉  2801c219094d5d49a2c5697ba1c34b9593d2c2a9 default@
    │ ◉  4ec5df5a189c813f8cc66aeb35e007929ebb46bc
    ├─╯
    ◉  e7d7dbb91c5a543ea680711093e689916d5f31df
    ◉  0000000000000000000000000000000000000000
    "###);
}

/// Test multiple `-r` flags to `workspace add` to create a workspace
/// working-copy commit with multiple parents.
#[test]
fn test_workspaces_add_workspace_multiple_revisions() {
    let test_env = TestEnvironment::default();
    test_env.jj_cmd_ok(test_env.env_root(), &["init", "--git", "main"]);
    let main_path = test_env.env_root().join("main");

    std::fs::write(main_path.join("file-1"), "contents").unwrap();
    test_env.jj_cmd_ok(&main_path, &["commit", "-m", "first"]);
    test_env.jj_cmd_ok(&main_path, &["new", "-r", "root()"]);

    std::fs::write(main_path.join("file-2"), "contents").unwrap();
    test_env.jj_cmd_ok(&main_path, &["commit", "-m", "second"]);
    test_env.jj_cmd_ok(&main_path, &["new", "-r", "root()"]);

    std::fs::write(main_path.join("file-3"), "contents").unwrap();
    test_env.jj_cmd_ok(&main_path, &["commit", "-m", "third"]);
    test_env.jj_cmd_ok(&main_path, &["new", "-r", "root()"]);

    insta::assert_snapshot!(get_log_output(&test_env, &main_path), @r###"
    @  5b36783cd11c4607a329c5e8c2fd9097c9ce2add
    │ ◉  23881f07b53ce1ea936ca8842e344dea9c3356e5
    ├─╯
    │ ◉  1f6a15f0af2a985703864347f5fdf27a82fc3d73
    ├─╯
    │ ◉  e7d7dbb91c5a543ea680711093e689916d5f31df
    ├─╯
    ◉  0000000000000000000000000000000000000000
    "###);

    let (_, stderr) = test_env.jj_cmd_ok(
        &main_path,
        &[
            "workspace",
            "add",
            "--name=merge",
            "../merged",
            "-r=238",
            "-r=1f6",
            "-r=e7d",
        ],
    );
    insta::assert_snapshot!(stderr.replace('\\', "/"), @r###"
    Created workspace in "../merged"
    Working copy now at: wmwvqwsz fa8fdc28 (empty) (no description set)
    Parent commit      : mzvwutvl 23881f07 third
    Parent commit      : kkmpptxz 1f6a15f0 second
    Parent commit      : qpvuntsm e7d7dbb9 first
    Added 3 files, modified 0 files, removed 0 files
    "###);

    insta::assert_snapshot!(get_log_output(&test_env, &main_path), @r###"
    ◉      fa8fdc28af12d3c96b1e0ed062f5a8f9a99818f0 merge@
    ├─┬─╮
    │ │ ◉  e7d7dbb91c5a543ea680711093e689916d5f31df
    │ ◉ │  1f6a15f0af2a985703864347f5fdf27a82fc3d73
    │ ├─╯
    ◉ │  23881f07b53ce1ea936ca8842e344dea9c3356e5
    ├─╯
    │ @  5b36783cd11c4607a329c5e8c2fd9097c9ce2add default@
    ├─╯
    ◉  0000000000000000000000000000000000000000
    "###);
}

/// Test making changes to the working copy in a workspace as it gets rewritten
/// from another workspace
#[test]
fn test_workspaces_conflicting_edits() {
    let test_env = TestEnvironment::default();
    test_env.jj_cmd_ok(test_env.env_root(), &["init", "--git", "main"]);
    let main_path = test_env.env_root().join("main");
    let secondary_path = test_env.env_root().join("secondary");

    std::fs::write(main_path.join("file"), "contents\n").unwrap();
    test_env.jj_cmd_ok(&main_path, &["new"]);

    test_env.jj_cmd_ok(&main_path, &["workspace", "add", "../secondary"]);

    insta::assert_snapshot!(get_log_output(&test_env, &main_path), @r###"
    ◉  265af0cdbcc7bb33e3734ad72565c943ce3fb0d4 secondary@
    │ @  351099fa72cfbb1b34e410e89821efc623295974 default@
    ├─╯
    ◉  cf911c223d3e24e001fc8264d6dbf0610804fc40
    ◉  0000000000000000000000000000000000000000
    "###);

    // Make changes in both working copies
    std::fs::write(main_path.join("file"), "changed in main\n").unwrap();
    std::fs::write(secondary_path.join("file"), "changed in second\n").unwrap();
    // Squash the changes from the main workspace into the initial commit (before
    // running any command in the secondary workspace
    let (stdout, stderr) = test_env.jj_cmd_ok(&main_path, &["squash"]);
    insta::assert_snapshot!(stdout, @"");
    insta::assert_snapshot!(stderr, @r###"
    Rebased 1 descendant commits
    Working copy now at: mzvwutvl fe8f41ed (empty) (no description set)
    Parent commit      : qpvuntsm c0d4a99e (no description set)
    "###);

    // The secondary workspace's working-copy commit was updated
    insta::assert_snapshot!(get_log_output(&test_env, &main_path), @r###"
    @  fe8f41ed01d693b2d4365cd89e42ad9c531a939b default@
    │ ◉  a1896a17282f19089a5cec44358d6609910e0513 secondary@
    ├─╯
    ◉  c0d4a99ef98ada7da8dc73a778bbb747c4178385
    ◉  0000000000000000000000000000000000000000
    "###);
    let stderr = test_env.jj_cmd_failure(&secondary_path, &["st"]);
    insta::assert_snapshot!(stderr, @r###"
    Error: The working copy is stale (not updated since operation a07b009d6eba).
    Hint: Run `jj workspace update-stale` to update it.
    See https://github.com/martinvonz/jj/blob/main/docs/working-copy.md#stale-working-copy for more information.
    "###);
    // Same error on second run, and from another command
    let stderr = test_env.jj_cmd_failure(&secondary_path, &["log"]);
    insta::assert_snapshot!(stderr, @r###"
    Error: The working copy is stale (not updated since operation a07b009d6eba).
    Hint: Run `jj workspace update-stale` to update it.
    See https://github.com/martinvonz/jj/blob/main/docs/working-copy.md#stale-working-copy for more information.
    "###);
    let (stdout, stderr) = test_env.jj_cmd_ok(&secondary_path, &["workspace", "update-stale"]);
    // It was detected that the working copy is now stale.
    // Since there was an uncommitted change in the working copy, it should
    // have been committed first (causing divergence)
    insta::assert_snapshot!(stdout, @"");
    insta::assert_snapshot!(stderr, @r###"
    Concurrent modification detected, resolving automatically.
    Rebased 1 descendant commits onto commits rewritten by other operation
    Working copy now at: pmmvwywv?? a1896a17 (empty) (no description set)
    Added 0 files, modified 1 files, removed 0 files
    "###);
    insta::assert_snapshot!(get_log_output(&test_env, &secondary_path),
    @r###"
    ◉  a3c96849ef9f124cbfc2416dc13bf17309d5020a (divergent)
    │ ◉  fe8f41ed01d693b2d4365cd89e42ad9c531a939b default@
    ├─╯
    │ @  a1896a17282f19089a5cec44358d6609910e0513 secondary@ (divergent)
    ├─╯
    ◉  c0d4a99ef98ada7da8dc73a778bbb747c4178385
    ◉  0000000000000000000000000000000000000000
    "###);
    // The stale working copy should have been resolved by the previous command
    let stdout = get_log_output(&test_env, &secondary_path);
    assert!(!stdout.starts_with("The working copy is stale"));
    insta::assert_snapshot!(stdout, @r###"
    ◉  a3c96849ef9f124cbfc2416dc13bf17309d5020a (divergent)
    │ ◉  fe8f41ed01d693b2d4365cd89e42ad9c531a939b default@
    ├─╯
    │ @  a1896a17282f19089a5cec44358d6609910e0513 secondary@ (divergent)
    ├─╯
    ◉  c0d4a99ef98ada7da8dc73a778bbb747c4178385
    ◉  0000000000000000000000000000000000000000
    "###);
}

/// Test a clean working copy that gets rewritten from another workspace
#[test]
fn test_workspaces_updated_by_other() {
    let test_env = TestEnvironment::default();
    test_env.jj_cmd_ok(test_env.env_root(), &["init", "--git", "main"]);
    let main_path = test_env.env_root().join("main");
    let secondary_path = test_env.env_root().join("secondary");

    std::fs::write(main_path.join("file"), "contents\n").unwrap();
    test_env.jj_cmd_ok(&main_path, &["new"]);

    test_env.jj_cmd_ok(&main_path, &["workspace", "add", "../secondary"]);

    insta::assert_snapshot!(get_log_output(&test_env, &main_path), @r###"
    ◉  265af0cdbcc7bb33e3734ad72565c943ce3fb0d4 secondary@
    │ @  351099fa72cfbb1b34e410e89821efc623295974 default@
    ├─╯
    ◉  cf911c223d3e24e001fc8264d6dbf0610804fc40
    ◉  0000000000000000000000000000000000000000
    "###);

    // Rewrite the check-out commit in one workspace.
    std::fs::write(main_path.join("file"), "changed in main\n").unwrap();
    let (stdout, stderr) = test_env.jj_cmd_ok(&main_path, &["squash"]);
    insta::assert_snapshot!(stdout, @"");
    insta::assert_snapshot!(stderr, @r###"
    Rebased 1 descendant commits
    Working copy now at: mzvwutvl fe8f41ed (empty) (no description set)
    Parent commit      : qpvuntsm c0d4a99e (no description set)
    "###);

    // The secondary workspace's working-copy commit was updated.
    insta::assert_snapshot!(get_log_output(&test_env, &main_path), @r###"
    @  fe8f41ed01d693b2d4365cd89e42ad9c531a939b default@
    │ ◉  a1896a17282f19089a5cec44358d6609910e0513 secondary@
    ├─╯
    ◉  c0d4a99ef98ada7da8dc73a778bbb747c4178385
    ◉  0000000000000000000000000000000000000000
    "###);
    let stderr = test_env.jj_cmd_failure(&secondary_path, &["st"]);
    insta::assert_snapshot!(stderr, @r###"
    Error: The working copy is stale (not updated since operation a07b009d6eba).
    Hint: Run `jj workspace update-stale` to update it.
    See https://github.com/martinvonz/jj/blob/main/docs/working-copy.md#stale-working-copy for more information.
    "###);
    let (stdout, stderr) = test_env.jj_cmd_ok(&secondary_path, &["workspace", "update-stale"]);
    // It was detected that the working copy is now stale, but clean. So no
    // divergent commit should be created.
    insta::assert_snapshot!(stdout, @"");
    insta::assert_snapshot!(stderr, @r###"
    Working copy now at: pmmvwywv a1896a17 (empty) (no description set)
    Added 0 files, modified 1 files, removed 0 files
    "###);
    insta::assert_snapshot!(get_log_output(&test_env, &secondary_path),
    @r###"
    ◉  fe8f41ed01d693b2d4365cd89e42ad9c531a939b default@
    │ @  a1896a17282f19089a5cec44358d6609910e0513 secondary@
    ├─╯
    ◉  c0d4a99ef98ada7da8dc73a778bbb747c4178385
    ◉  0000000000000000000000000000000000000000
    "###);
}

#[test]
fn test_workspaces_current_op_discarded_by_other() {
    let test_env = TestEnvironment::default();
    test_env.jj_cmd_ok(test_env.env_root(), &["init", "--git", "main"]);
    let main_path = test_env.env_root().join("main");
    let secondary_path = test_env.env_root().join("secondary");

    std::fs::write(main_path.join("file"), "contents\n").unwrap();
    test_env.jj_cmd_ok(&main_path, &["new"]);

    test_env.jj_cmd_ok(&main_path, &["workspace", "add", "../secondary"]);

    // Create an op by creating a commit in one workspace.
    std::fs::write(main_path.join("file"), "changed in main\n").unwrap();
    test_env.jj_cmd_ok(&main_path, &["commit", "-m", "changed in main"]);

    let stdout = test_env.jj_cmd_success(
        &main_path,
        &[
            "operation",
            "log",
            "--template",
            "id.short(10) ++ description",
        ],
    );
    insta::assert_snapshot!(stdout, @r###"
    @  61803bbb6fcommit 337e6027f903f3616d31599d25b6a1ee9b0588f1
    ◉  9296fa56b1snapshot working copy
    ◉  a07b009d6eCreate initial working-copy commit in workspace secondary
    ◉  bb9e857734add workspace 'secondary'
    ◉  35c191ac25new empty commit
    ◉  859801f07bsnapshot working copy
    ◉  27143b59c6add workspace 'default'
    ◉  0e8aee02e2initialize repo
    ◉  0000000000
    "###);
    let output = std::process::Command::new("xxd")
        .arg(main_path.join(".jj/working_copy/checkout"))
        .output()
        .unwrap();
    insta::assert_snapshot!(String::from_utf8(output.stderr).unwrap(), @"");
    insta::assert_snapshot!(String::from_utf8(output.stdout).unwrap(), @r###"
    00000000: 1240 6180 3bbb 6f2e 3c1b d3fe 3f72 ed6e  .@a.;.o.<...?r.n
    00000010: c032 32df 9d1a 53e2 e61a 4582 e388 e13f  .22...S...E....?
    00000020: f7c7 184d 5535 bdbe 4b8d d7e5 a3b6 664c  ...MU5..K.....fL
    00000030: a163 7576 faae 8127 f37e 7de2 460f 6f61  .cuv...'.~}.F.oa
    00000040: 589c 1a07 6465 6661 756c 74              X...default
    "###);
    let output = std::process::Command::new("xxd")
        .arg(secondary_path.join(".jj/working_copy/checkout"))
        .output()
        .unwrap();
    insta::assert_snapshot!(String::from_utf8(output.stderr).unwrap(), @"");
    insta::assert_snapshot!(String::from_utf8(output.stdout).unwrap(), @r###"
    00000000: 1240 a07b 009d 6eba 3e81 7e2e ed69 0e58  .@.{..n.>.~..i.X
    00000010: ba2f 9b3d 925a 582f 6e68 beff f9ee af59  ./.=.ZX/nh.....Y
    00000020: 7d15 1ef4 e2ad 5c7e 4a56 61a9 bfe2 1ba4  }.....\~JVa.....
    00000030: 917b f382 c597 f278 c5a5 812d b235 32f5  .{.....x...-.52.
    00000040: 04dd 1a09 7365 636f 6e64 6172 79         ....secondary
    "###);

    // Abandon ops, including the one the secondary workspace is currently on.
    test_env.jj_cmd_ok(&main_path, &["operation", "abandon", "..@-"]);
    test_env.jj_cmd_ok(&main_path, &["util", "gc", "--expire=now"]);

    let output = std::process::Command::new("xxd")
        .arg(secondary_path.join(".jj/working_copy/checkout"))
        .output()
        .unwrap();
    insta::assert_snapshot!(String::from_utf8(output.stderr).unwrap(), @"");
    insta::assert_snapshot!(String::from_utf8(output.stdout).unwrap(), @r###"
    00000000: 1240 a07b 009d 6eba 3e81 7e2e ed69 0e58  .@.{..n.>.~..i.X
    00000010: ba2f 9b3d 925a 582f 6e68 beff f9ee af59  ./.=.ZX/nh.....Y
    00000020: 7d15 1ef4 e2ad 5c7e 4a56 61a9 bfe2 1ba4  }.....\~JVa.....
    00000030: 917b f382 c597 f278 c5a5 812d b235 32f5  .{.....x...-.52.
    00000040: 04dd 1a09 7365 636f 6e64 6172 79         ....secondary
    "###);
    let output = std::process::Command::new("xxd")
        .arg(main_path.join(".jj/repo/op_store/operations/022863b2b32b51e446f64cd0aa4fd2076909be21db40be6025936de7782d8282849a1bb0f4b2e25505fb9f7de07eca0db9347d7590088e7800501be92b82b2ba"))
        .output()
        .unwrap();
    insta::assert_snapshot!(String::from_utf8(output.stderr).unwrap(), @"");
    insta::assert_snapshot!(String::from_utf8(output.stdout).unwrap(), @r###"
    00000000: 0a40 3bdf 165a e049 8856 098c 6e91 fcb8  .@;..Z.I.V..n...
    00000010: edca a0ec 7618 6b4d 442c dfeb 9d47 172d  ....v.kMD,...G.-
    00000020: 7787 9eaa 782a 8541 e8f4 8cf8 4fd7 f6fa  w...x*.A....O...
    00000030: 7161 80e0 0c77 92f8 02ee 952e 68d9 1e88  qa...w......h...
    00000040: b08c 1240 0000 0000 0000 0000 0000 0000  ...@............
    00000050: 0000 0000 0000 0000 0000 0000 0000 0000  ................
    00000060: 0000 0000 0000 0000 0000 0000 0000 0000  ................
    00000070: 0000 0000 0000 0000 0000 0000 0000 0000  ................
    00000080: 0000 0000 1a92 010a 0a08 f0fe e387 c71c  ................
    00000090: 10a4 0312 0a08 f0fe e387 c71c 10a4 031a  ................
    000000a0: 2f63 6f6d 6d69 7420 3333 3765 3630 3237  /commit 337e6027
    000000b0: 6639 3033 6633 3631 3664 3331 3539 3964  f903f3616d31599d
    000000c0: 3235 6236 6131 6565 3962 3035 3838 6631  25b6a1ee9b0588f1
    000000d0: 2210 686f 7374 2e65 7861 6d70 6c65 2e63  ".host.example.c
    000000e0: 6f6d 2a0d 7465 7374 2d75 7365 726e 616d  om*.test-usernam
    000000f0: 6532 260a 0461 7267 7312 1e6a 6a20 636f  e2&..args..jj co
    00000100: 6d6d 6974 202d 6d20 2763 6861 6e67 6564  mmit -m 'changed
    00000110: 2069 6e20 6d61 696e 27                    in main'
    "###);
    // let output = std::process::Command::new("xxd")
    //     .arg(main_path.join(".jj/repo/op_store/views/3bdf165ae0498856098c6e91fcb8edcaa0ec76186b4d442cdfeb9d47172d77879eaa782a8541e8f48cf84fd7f6fa716180e00c7792f802ee952e68d91e88b08c"))
    //     .output()
    //     .unwrap();
    // insta::assert_snapshot!(String::from_utf8(output.stderr).unwrap(), @"");
    // insta::assert_snapshot!(String::from_utf8(output.stdout).unwrap(), @"");
    let output = std::process::Command::new("tree")
        .arg(main_path.join(".jj"))
        .output()
        .unwrap();
    insta::assert_snapshot!(String::from_utf8(output.stderr).unwrap(), @"");
    insta::assert_snapshot!(String::from_utf8(output.stdout).unwrap().lines().skip(1).collect::<Vec<_>>().join("\n"), @r###"
    ├── repo
    │   ├── index
    │   │   ├── 00328fb5bf6cf8ca55f59b82a788f3b71797b7263dda09475083dd8bae49ebbc48ca81493657cf940b38a589a88221f6f2fb00acefae6de137af51ceced0e7d2
    │   │   ├── 2e77d42f14c86a6fb7906176f02f2197f419dc26da2ac2cb6426ed7aef08ed4f15231d24313d9e598d0df338550b8d7ad4b3a40d90182644000fb2e8e34dab1f
    │   │   ├── 4732ed168e5e3fc031ae77b02b9c483ba6098145545ca0bed67ff8aa0baae3026e6677120098fc193f6eb5c6552349c42346805961bec07fe8191f1909bfb2bf
    │   │   ├── 6f6d01db56ec8ebea5b292bbf5bcbafead24fa2ab91504149135023160fe6393f7b7a0b43d31d5cc6b050cbc38f7b1a17330b6edd27688d1fbd7b4a5592246ee
    │   │   ├── 9d442af5a9f19bb29730262f5e130fe759ce15fcb143e9ad3ac119c50dcd77461ad5df1b6b53d9ba69c35546e943a47639781fa073e9ba731be013ed7324a359
    │   │   ├── e40a190e9011a7c71a08afdfb7809ccc5812ebb438e0778a37fa51f155c445d68785e8d61fdfe1ed18c00a095430d28d3ec7673eb855aa92f0531327c161039e
    │   │   ├── ed509a02a0f14a8cf999c4d4cc23d1bdd4b8fadb3aecefbf4dc676d38aed04ef42300873360e119e8d17f065c6decef2ed3ab707c3ab625bf0ee2ed2a60df463
    │   │   ├── fe02e9777c826ede369d77371908f636350bd6de0e66325bef07e57932ee25c7d580c8d2a09fe56377a339cd1f1cffa2141f3db61ba9fa2841bc15468824abdc
    │   │   ├── operations
    │   │   │   ├── 0e8aee02e24230c99d6d90d469c582a60fdb2ae8329341bbdb09f4a0beceba1ce7c84fc9ba6c7657d6d275b392b89b825502475ad2501be1ddebd4a09b07668c
    │   │   │   ├── 27143b59c6904046f6be83ad6fe145d819944f9abbd7247ea9c57848d1d2c678ea8265598a156fe8aeef31d24d958bf6cfa0c2eb3afef40bdae2c5e98d73d0ee
    │   │   │   ├── 35c191ac252c501abda5c080c55b6751aef5b9f618e81cf7db802d16924c98fe67ce4fd2a41a36afb443f0296b7b8ff16ca6a140e145418a0db68ee293038484
    │   │   │   ├── 61803bbb6f2e3c1bd3fe3f72ed6ec03232df9d1a53e2e61a4582e388e13ff7c7184d5535bdbe4b8dd7e5a3b6664ca1637576faae8127f37e7de2460f6f61589c
    │   │   │   ├── 859801f07b25feb774646b6ee44fd4876227dce47d36eeee08e42af4e3d89b719db10178aad4adef0e974ed84c767a6c83abafcaba15cd9d4c26f91e44f964f6
    │   │   │   ├── 9296fa56b10b861f24906bed073e0f46f3a996cee69990bb95e2299f454bb68f6810348fb672c3d655ee50b360ab491824bcd9a486d581671794f1dfd77ae896
    │   │   │   ├── a07b009d6eba3e817e2eed690e58ba2f9b3d925a582f6e68befff9eeaf597d151ef4e2ad5c7e4a5661a9bfe21ba4917bf382c597f278c5a5812db23532f504dd
    │   │   │   └── bb9e857734b8e0bf0a19cd3a57984159e6fc492de1f4c0b32ef733eb159ff25a58b5dda5372dce0212c0f6829514a0cbdcf8fb5825c20d436b538b432bd77f19
    │   │   └── type
    │   ├── op_heads
    │   │   ├── heads
    │   │   │   └── 022863b2b32b51e446f64cd0aa4fd2076909be21db40be6025936de7782d8282849a1bb0f4b2e25505fb9f7de07eca0db9347d7590088e7800501be92b82b2ba
    │   │   └── type
    │   ├── op_store
    │   │   ├── operations
    │   │   │   └── 022863b2b32b51e446f64cd0aa4fd2076909be21db40be6025936de7782d8282849a1bb0f4b2e25505fb9f7de07eca0db9347d7590088e7800501be92b82b2ba
    │   │   ├── type
    │   │   └── views
    │   │       └── 3bdf165ae0498856098c6e91fcb8edcaa0ec76186b4d442cdfeb9d47172d77879eaa782a8541e8f48cf84fd7f6fa716180e00c7792f802ee952e68d91e88b08c
    │   ├── store
    │   │   ├── extra
    │   │   │   ├── 2ce3d1c11d2c308a1543dd65d0ac99f09f6e687c2f2d3b15d669f7241aeb176661b4fc21628e660026abd934ecdd269835e6b767456f5e19c55b8fd3e2b226b1
    │   │   │   ├── 482ae5a29fbe856c7272f2071b8b0f0359ee2d89ff392b8a900643fbd0836eccd067b8bf41909e206c90d45d6e7d8b6686b93ecaee5fe1a9060d87b672101310
    │   │   │   ├── 59f152d0edc99cb5e580c27275e88a29ee006ed591dcf85b61809568f8a86cfde5519ac074b8cf4624c7f012e2d6fff73c8598784aac3f057ed417ee35843136
    │   │   │   ├── 5c762c3eb51053545c7fc7784d873631ad3a9375c1be219e39bb85a5980685a15cdd6ae9a3889499c167526d2f01f4321f08002bb62dc29a893a29abe69c27d9
    │   │   │   ├── 5f9e96679c17963365a5e4110ba7e30b5cdd6c5d4233fe42336bcffadcb811d8cdc83df12494162ff28e29cd503363d2108de6a9741ed29383fd5c86885c6b60
    │   │   │   ├── 60233991858e9e93b768cfe4907ac8ae4690afe649af3142ae830570a0bb30740714c0f28639f4f0db2b6a6e0c71fd1bcbbff315b6d819bb1438357c39c17a42
    │   │   │   ├── 7f83a13a9458892c71c3337abfadf20000ce7555f66e625a66451071fcc6306d3a7085e5a12f6429e0d0cd8e0ac21ab6a3baeda1f34491b05372f411a4b22401
    │   │   │   ├── 8b0fd35602df0417e49ce5838c1dd3bd33894a39c8a37d63effff6dd24fa647277571ad7870eaca3d649b4a919c69e667d1a775bb919dcf4dfff6d0dc65b329d
    │   │   │   ├── f78c0b77845c79b8c4911d734867bc75ebdb27d6cb490de7e19eacc0676eda13969b424f55cd372958fb320cebfc493bb0127ca9b1963de9117e62c226b57598
    │   │   │   └── heads
    │   │   │       └── 5f9e96679c17963365a5e4110ba7e30b5cdd6c5d4233fe42336bcffadcb811d8cdc83df12494162ff28e29cd503363d2108de6a9741ed29383fd5c86885c6b60
    │   │   ├── git
    │   │   │   ├── config
    │   │   │   ├── description
    │   │   │   ├── HEAD
    │   │   │   ├── hooks
    │   │   │   │   ├── applypatch-msg.sample
    │   │   │   │   ├── commit-msg.sample
    │   │   │   │   ├── docs.url
    │   │   │   │   ├── fsmonitor-watchman.sample
    │   │   │   │   ├── post-update.sample
    │   │   │   │   ├── pre-applypatch.sample
    │   │   │   │   ├── pre-commit.sample
    │   │   │   │   ├── pre-merge-commit.sample
    │   │   │   │   ├── prepare-commit-msg.sample
    │   │   │   │   ├── pre-push.sample
    │   │   │   │   └── pre-rebase.sample
    │   │   │   ├── info
    │   │   │   │   ├── exclude
    │   │   │   │   └── refs
    │   │   │   ├── objects
    │   │   │   │   ├── info
    │   │   │   │   │   └── packs
    │   │   │   │   └── pack
    │   │   │   │       ├── pack-399427e462d2b22ec42349c8f8e5eda7c4307226.bitmap
    │   │   │   │       ├── pack-399427e462d2b22ec42349c8f8e5eda7c4307226.idx
    │   │   │   │       ├── pack-399427e462d2b22ec42349c8f8e5eda7c4307226.pack
    │   │   │   │       └── pack-399427e462d2b22ec42349c8f8e5eda7c4307226.rev
    │   │   │   ├── packed-refs
    │   │   │   └── refs
    │   │   │       ├── heads
    │   │   │       ├── jj
    │   │   │       └── tags
    │   │   ├── git_target
    │   │   └── type
    │   └── submodule_store
    │       └── type
    └── working_copy
        ├── checkout
        ├── tree_state
        └── type

    24 directories, 60 files
    "###);

    // The jj_cmd_ok below is not supposed to work
    let (stdout, stderr) = test_env.jj_cmd_ok(&secondary_path, &["st"]);
    insta::assert_snapshot!(stdout, @r###"
    The working copy is clean
    Working copy : pmmvwywv 265af0cd (empty) (no description set)
    Parent commit: qpvuntsm cf911c22 (no description set)
    "###);
    insta::assert_snapshot!(stderr, @"");

    // let stderr = test_env.jj_cmd_failure(&secondary_path, &["st"]);
    // insta::assert_snapshot!(stderr, @r###"
    // TBD
    // "###);
}

#[test]
fn test_workspaces_update_stale_noop() {
    let test_env = TestEnvironment::default();
    test_env.jj_cmd_ok(test_env.env_root(), &["init", "--git", "main"]);
    let main_path = test_env.env_root().join("main");

    let (stdout, stderr) = test_env.jj_cmd_ok(&main_path, &["workspace", "update-stale"]);
    insta::assert_snapshot!(stdout, @"");
    insta::assert_snapshot!(stderr, @r###"
    Nothing to do (the working copy is not stale).
    "###);

    let stderr = test_env.jj_cmd_failure(
        &main_path,
        &["workspace", "update-stale", "--ignore-working-copy"],
    );
    insta::assert_snapshot!(stderr, @r###"
    Error: This command must be able to update the working copy.
    Hint: Don't use --ignore-working-copy.
    "###);

    let stdout = test_env.jj_cmd_success(&main_path, &["op", "log", "-Tdescription"]);
    insta::assert_snapshot!(stdout, @r###"
    @  add workspace 'default'
    ◉  initialize repo
    ◉
    "###);
}

/// Test "update-stale" in a dirty, but not stale working copy.
#[test]
fn test_workspaces_update_stale_snapshot() {
    let test_env = TestEnvironment::default();
    test_env.jj_cmd_ok(test_env.env_root(), &["init", "--git", "main"]);
    let main_path = test_env.env_root().join("main");
    let secondary_path = test_env.env_root().join("secondary");

    std::fs::write(main_path.join("file"), "changed in main\n").unwrap();
    test_env.jj_cmd_ok(&main_path, &["new"]);
    test_env.jj_cmd_ok(&main_path, &["workspace", "add", "../secondary"]);

    // Record new operation in one workspace.
    test_env.jj_cmd_ok(&main_path, &["new"]);

    // Snapshot the other working copy, which unfortunately results in concurrent
    // operations, but should be resolved cleanly.
    std::fs::write(secondary_path.join("file"), "changed in second\n").unwrap();
    let (stdout, stderr) = test_env.jj_cmd_ok(&secondary_path, &["workspace", "update-stale"]);
    insta::assert_snapshot!(stdout, @"");
    insta::assert_snapshot!(stderr, @r###"
    Concurrent modification detected, resolving automatically.
    Nothing to do (the working copy is not stale).
    "###);

    insta::assert_snapshot!(get_log_output(&test_env, &secondary_path), @r###"
    @  4976dfa88529814c4dd8c06253fbd82d076b79f8 secondary@
    │ ◉  8357b22214ba8adb6d2d378fa5b85274f1c7967c default@
    │ ◉  1a769966ed69fa7abadbd2d899e2be1025cb04fb
    ├─╯
    ◉  b4a6c25e777817db67fdcbd50f1dd3b74b46b5f1
    ◉  0000000000000000000000000000000000000000
    "###);
}

/// Test forgetting workspaces
#[test]
fn test_workspaces_forget() {
    let test_env = TestEnvironment::default();
    test_env.jj_cmd_ok(test_env.env_root(), &["init", "--git", "main"]);
    let main_path = test_env.env_root().join("main");

    std::fs::write(main_path.join("file"), "contents").unwrap();
    test_env.jj_cmd_ok(&main_path, &["new"]);

    test_env.jj_cmd_ok(&main_path, &["workspace", "add", "../secondary"]);
    let (stdout, stderr) = test_env.jj_cmd_ok(&main_path, &["workspace", "forget"]);
    insta::assert_snapshot!(stdout, @"");
    insta::assert_snapshot!(stderr, @"");

    // When listing workspaces, only the secondary workspace shows up
    let stdout = test_env.jj_cmd_success(&main_path, &["workspace", "list"]);
    insta::assert_snapshot!(stdout, @r###"
    secondary: pmmvwywv feda1c4e (empty) (no description set)
    "###);

    // `jj status` tells us that there's no working copy here
    let (stdout, stderr) = test_env.jj_cmd_ok(&main_path, &["st"]);
    insta::assert_snapshot!(stdout, @r###"
    No working copy
    "###);
    insta::assert_snapshot!(stderr, @"");

    // The old working copy doesn't get an "@" in the log output
    // TODO: We should abandon the empty working copy commit
    // TODO: It seems useful to still have the "secondary@" marker here even though
    // there's only one workspace. We should show it when the command is not run
    // from that workspace.
    insta::assert_snapshot!(get_log_output(&test_env, &main_path), @r###"
    ◉  feda1c4e5ffe63fb16818ccdd8c21483537e31f2
    │ ◉  e949be04e93e830fcce23fefac985c1deee52eea
    ├─╯
    ◉  123ed18e4c4c0d77428df41112bc02ffc83fb935
    ◉  0000000000000000000000000000000000000000
    "###);

    // Revision "@" cannot be used
    let stderr = test_env.jj_cmd_failure(&main_path, &["log", "-r", "@"]);
    insta::assert_snapshot!(stderr, @r###"
    Error: Workspace "default" doesn't have a working copy
    "###);

    // Try to add back the workspace
    // TODO: We should make this just add it back instead of failing
    let stderr = test_env.jj_cmd_failure(&main_path, &["workspace", "add", "."]);
    insta::assert_snapshot!(stderr, @r###"
    Error: Workspace already exists
    "###);

    // Add a third workspace...
    test_env.jj_cmd_ok(&main_path, &["workspace", "add", "../third"]);
    // ... and then forget it, and the secondary workspace too
    let (stdout, stderr) =
        test_env.jj_cmd_ok(&main_path, &["workspace", "forget", "secondary", "third"]);
    insta::assert_snapshot!(stdout, @"");
    insta::assert_snapshot!(stderr, @"");
    // No workspaces left
    let stdout = test_env.jj_cmd_success(&main_path, &["workspace", "list"]);
    insta::assert_snapshot!(stdout, @"");
}

#[test]
fn test_workspaces_forget_multi_transaction() {
    let test_env = TestEnvironment::default();
    test_env.jj_cmd_ok(test_env.env_root(), &["init", "--git", "main"]);
    let main_path = test_env.env_root().join("main");

    std::fs::write(main_path.join("file"), "contents").unwrap();
    test_env.jj_cmd_ok(&main_path, &["new"]);

    test_env.jj_cmd_ok(&main_path, &["workspace", "add", "../second"]);
    test_env.jj_cmd_ok(&main_path, &["workspace", "add", "../third"]);

    // there should be three workspaces
    let stdout = test_env.jj_cmd_success(&main_path, &["workspace", "list"]);
    insta::assert_snapshot!(stdout, @r###"
    default: rlvkpnrz e949be04 (empty) (no description set)
    second: pmmvwywv feda1c4e (empty) (no description set)
    third: rzvqmyuk 485853ed (empty) (no description set)
    "###);

    // delete two at once, in a single tx
    test_env.jj_cmd_ok(&main_path, &["workspace", "forget", "second", "third"]);
    let stdout = test_env.jj_cmd_success(&main_path, &["workspace", "list"]);
    insta::assert_snapshot!(stdout, @r###"
    default: rlvkpnrz e949be04 (empty) (no description set)
    "###);

    // the op log should have multiple workspaces forgotten in a single tx
    let stdout = test_env.jj_cmd_success(&main_path, &["op", "log", "--limit", "1"]);
    insta::assert_snapshot!(stdout, @r###"
    @  f96865b00a04 test-username@host.example.com 2001-02-03 04:05:12.000 +07:00 - 2001-02-03 04:05:12.000 +07:00
    │  forget workspaces second, third
    │  args: jj workspace forget second third
    "###);

    // now, undo, and that should restore both workspaces
    test_env.jj_cmd_ok(&main_path, &["op", "undo"]);

    // finally, there should be three workspaces at the end
    let stdout = test_env.jj_cmd_success(&main_path, &["workspace", "list"]);
    insta::assert_snapshot!(stdout, @r###"
    default: rlvkpnrz e949be04 (empty) (no description set)
    second: pmmvwywv feda1c4e (empty) (no description set)
    third: rzvqmyuk 485853ed (empty) (no description set)
    "###);
}

/// Test context of commit summary template
#[test]
fn test_list_workspaces_template() {
    let test_env = TestEnvironment::default();
    test_env.jj_cmd_ok(test_env.env_root(), &["init", "--git", "main"]);
    test_env.add_config(
        r#"
        templates.commit_summary = """commit_id.short() ++ " " ++ description.first_line() ++
                                      if(current_working_copy, " (current)")"""
        "#,
    );
    let main_path = test_env.env_root().join("main");
    let secondary_path = test_env.env_root().join("secondary");

    std::fs::write(main_path.join("file"), "contents").unwrap();
    test_env.jj_cmd_ok(&main_path, &["commit", "-m", "initial"]);
    test_env.jj_cmd_ok(
        &main_path,
        &["workspace", "add", "--name", "second", "../secondary"],
    );

    // "current_working_copy" should point to the workspace we operate on
    let stdout = test_env.jj_cmd_success(&main_path, &["workspace", "list"]);
    insta::assert_snapshot!(stdout, @r###"
    default: e0e6d5672858  (current)
    second: f68da2d114f1 
    "###);

    let stdout = test_env.jj_cmd_success(&secondary_path, &["workspace", "list"]);
    insta::assert_snapshot!(stdout, @r###"
    default: e0e6d5672858 
    second: f68da2d114f1  (current)
    "###);
}

/// Test getting the workspace root from primary and secondary workspaces
#[test]
fn test_workspaces_root() {
    let test_env = TestEnvironment::default();
    test_env.jj_cmd_ok(test_env.env_root(), &["init", "--git", "main"]);
    let main_path = test_env.env_root().join("main");
    let secondary_path = test_env.env_root().join("secondary");

    let stdout = test_env.jj_cmd_success(&main_path, &["workspace", "root"]);
    insta::assert_snapshot!(stdout, @r###"
    $TEST_ENV/main
    "###);
    let main_subdir_path = main_path.join("subdir");
    std::fs::create_dir(&main_subdir_path).unwrap();
    let stdout = test_env.jj_cmd_success(&main_subdir_path, &["workspace", "root"]);
    insta::assert_snapshot!(stdout, @r###"
    $TEST_ENV/main
    "###);

    test_env.jj_cmd_ok(
        &main_path,
        &["workspace", "add", "--name", "secondary", "../secondary"],
    );
    let stdout = test_env.jj_cmd_success(&secondary_path, &["workspace", "root"]);
    insta::assert_snapshot!(stdout, @r###"
    $TEST_ENV/secondary
    "###);
    let secondary_subdir_path = secondary_path.join("subdir");
    std::fs::create_dir(&secondary_subdir_path).unwrap();
    let stdout = test_env.jj_cmd_success(&secondary_subdir_path, &["workspace", "root"]);
    insta::assert_snapshot!(stdout, @r###"
    $TEST_ENV/secondary
    "###);
}

fn get_log_output(test_env: &TestEnvironment, cwd: &Path) -> String {
    let template = r#"
    separate(" ",
      commit_id,
      working_copies,
      if(divergent, "(divergent)"),
    )
    "#;
    test_env.jj_cmd_success(cwd, &["log", "-T", template, "-r", "all()"])
}
