// Copyright 2020 The Jujutsu Authors
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

use clap_complete::ArgValueCandidates;
use tracing::instrument;
use pollster::FutureExt as _;

use jj_lib::repo_path::RepoPathBuf;
use jj_lib::repo_path::RepoPath;
use crate::cli_util::CommandHelper;
use crate::command_error::CommandError;
use crate::complete;
use crate::templater::TemplateRenderer;
use crate::ui::Ui;
use crate::commands::status::visit_collapsed_untracked_files;

/// List untracked files relative to a revision
#[derive(clap::Args, Clone, Debug)]
pub(crate) struct FileListUntrackedArgs {
    /// Render each untracked file using the given template
    ///
    /// All 0-argument methods of the [`RepoPath` type] are available as
    /// keywords in the template expression. See [`jj help -k templates`] for
    /// more information.
    ///
    /// [`RepoPath` type]:
    ///     https://docs.jj-vcs.dev/latest/templates/#repopath-type
    ///
    /// [`jj help -k templates`]:
    ///     https://docs.jj-vcs.dev/latest/templates/
    #[arg(long, short = 'T', add = ArgValueCandidates::new(complete::template_aliases))]
    template: Option<String>,
}

#[instrument(skip_all)]
pub(crate) fn cmd_file_list_untracked(
    ui: &mut Ui,
    command: &CommandHelper,
    args: &FileListUntrackedArgs,
) -> Result<(), CommandError> {
    let (workspace_command, snapshot_stats) = command.workspace_helper_with_stats(ui)?;
    let commit = workspace_command.resolve_single_rev(ui, &String::from("@").into())?;
    let tree = commit.tree();
    let template: TemplateRenderer<RepoPathBuf> = {
        let language = workspace_command.commit_template_language();
        let text = match &args.template {
            Some(value) => value.to_owned(),
            None => workspace_command.settings().get("templates.file_list_untracked")?,
        };
        workspace_command
            .parse_template(ui, &language, &text)?
            .labeled(["diff", "untracked"])
    };

    ui.request_pager();
    let mut formatter = ui.stdout_formatter();
    
    visit_collapsed_untracked_files(
        snapshot_stats.untracked_paths.keys(),
        tree.clone(),
        |path, _is_dir| {
            template.format(&path.to_owned(), formatter.as_mut())?;
            Ok(())
        },
    ).block_on()?;
    Ok(())
}
