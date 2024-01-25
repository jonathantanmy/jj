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

    // Create an op by abandoning the parent commit. Importantly, that commit also
    // changes the target tree in the secondary workspace.
    test_env.jj_cmd_ok(&main_path, &["abandon", "@-"]);

    let stdout = test_env.jj_cmd_success(
        &main_path,
        &[
            "operation",
            "log",
            "--template",
            r#"id.short(10) ++ " " ++ description"#,
        ],
    );
    insta::assert_snapshot!(stdout, @r###"
    @  8880ddc17d abandon commit cf911c223d3e24e001fc8264d6dbf0610804fc40
    ◉  a07b009d6e Create initial working-copy commit in workspace secondary
    ◉  bb9e857734 add workspace 'secondary'
    ◉  35c191ac25 new empty commit
    ◉  859801f07b snapshot working copy
    ◉  27143b59c6 add workspace 'default'
    ◉  0e8aee02e2 initialize repo
    ◉  0000000000
    "###);

    // Abandon ops, including the one the secondary workspace is currently on.
    test_env.jj_cmd_ok(&main_path, &["operation", "abandon", "..@-"]);
    test_env.jj_cmd_ok(&main_path, &["util", "gc", "--expire=now"]);

    let (stdout, stderr) = test_env.jj_cmd_ok(&secondary_path, &["st"]);
    insta::assert_snapshot!(stdout, @r###"
    Working copy changes:
    A file
    Working copy : pmmvwywv a23980d3 (no description set)
    Parent commit: zzzzzzzz 00000000 (empty) (no description set)
    "###);
    insta::assert_snapshot!(stderr, @r###"
    Missing operation, using operation repo was loaded at
    "###);

    // hmm...not idempotent
    let (stdout, stderr) = test_env.jj_cmd_ok(&secondary_path, &["st"]);
    insta::assert_snapshot!(stdout, @r###"
    Working copy changes:
    A file
    Working copy : pmmvwywv a23980d3 (no description set)
    Parent commit: zzzzzzzz 00000000 (empty) (no description set)
    "###);
    insta::assert_snapshot!(stderr, @"");

    let (stdout, stderr) = test_env.jj_cmd_ok(&secondary_path, &["commit", "-m=message"]);
    insta::assert_snapshot!(stdout, @"");
    insta::assert_snapshot!(stderr, @r###"
    Working copy now at: znkkpsqq af427e3c (empty) (no description set)
    Parent commit      : pmmvwywv 92154c2b message
    "###);

    let (stdout, stderr) = test_env.jj_cmd_ok(&secondary_path, &["st"]);
    insta::assert_snapshot!(stdout, @r###"
    The working copy is clean
    Working copy : znkkpsqq af427e3c (empty) (no description set)
    Parent commit: pmmvwywv 92154c2b message
    "###);
    insta::assert_snapshot!(stderr, @"");
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
