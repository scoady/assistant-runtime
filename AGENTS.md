# Project Instructions

## Repository Workflow

- Do not develop directly on `main`.
- Use `feat/<scope>` for user-visible behavior changes.
- Use `fix/<scope>` for defects and regressions.
- Use `chore/<scope>` for maintenance, tooling, dependency, CI, packaging, and refactor-without-behavior-change work.
- Use `docs/<scope>` for documentation-only changes.
- Open a merge request for completed branch work.
- Keep merge requests focused and include validation notes.

## Release Workflow

- Release from merged `main` only.
- If a release requires prep changes, land those changes through an MR before tagging.
- Confirm version metadata matches the intended tag.
- Produce release notes and verify artifacts before and after publish.

## GitHub Tooling

- Prefer normal `git push` and high-level `gh` commands first.
- If GitHub connectivity, credential-helper, or transport issues block those commands, note that plainly and fall back to `gh api` instead of waffling on repeated retries.

## Documentation

- Update docs when commands, workflows, behavior, or release process changes.
- Prefer concise, task-oriented docs with runnable commands.

## Worktrees

- Use git worktrees when parallel branch work would otherwise dirty a single checkout.
- Name worktree directories clearly after the branch or task they support.

## Code Organization

- Keep the codebase modular; prefer small files with one responsibility.
- Do not keep extending the largest file in the repo when a feature-specific file or package should exist.
- Shared models, layout structs, hitboxes, presenters, and helpers should be declared once and reused.
- When a capability already exists, extend it or extract a reusable abstraction from it instead of creating a second parallel implementation.
- Prefer reusable abstractions over local one-off implementations when the same behavior can serve multiple call sites.
- Favor cohesive object-oriented design with clear ownership, composition, and dependency injection instead of global or singleton-heavy wiring.
- Separate rendering, state mutation, layout math, and backend wiring when the code is more than a few helpers.
- Split files early instead of waiting for a later mega-file refactor.
- Move superseded code into an explicit dump or legacy area until it is safely deleted.

## Testing Expectations

- Nontrivial feature work should add or update at least one focused automated test.
- Bug fixes should add regression coverage when the behavior can be isolated.
- Logic-heavy paths should be validated with direct automated tests, not only manual screenshots or clickthroughs.
- Run the project build and test suite before release or handoff.

## Global Skills

- Use the global skills installed from `ai-coding-skills-and-conventions` when the task matches:
  - `code-project-bootstrap`
  - `repo-conventions`
  - `git-worktree-management`
  - `gitlab-mr-workflow`
  - `release-management`
  - `documentation-style`
