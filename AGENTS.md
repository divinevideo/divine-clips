# Repository Guidelines

## Project Structure & Module Organization
- `clipcrate/` contains the Rust backend.
- `clips-web/` contains the web frontend.
- `clips-verifier/` contains the verifier service/tests.
- Shared orchestration and local environment setup live at the repo root, including `docker-compose.yml` and supporting docs.

## Build, Test, and Validation Commands
- Backend: `cargo check --all-targets` and `cargo test --workspace --lib` from `clipcrate/`.
- Frontend: `npm ci` and `npm run build` from `clips-web/`.
- Verifier: `npm ci` and `npx vitest run` from `clips-verifier/`.
- Use the relevant local service or Docker workflow when validating cross-service behavior.

## Coding Style & Naming Conventions
- Follow the existing Rust, frontend, and verifier patterns already established in each subproject.
- Keep backend, frontend, and verifier changes scoped. Do not mix unrelated refactors across all three surfaces in one PR unless the feature truly requires it.
- Verify service boundaries, contract assumptions, and deployment expectations against the current docs and code before changing them.

## Security & Operational Notes
- Never commit secrets, tokens, service credentials, or logs containing sensitive values.
- Public issues, PRs, branch names, screenshots, and descriptions must not mention corporate partners, customers, brands, campaign names, or other sensitive external identities unless a maintainer explicitly approves it. Use generic descriptors instead.
- Be explicit about any change that affects payouts, marketplace logic, verification, or creator/admin flows.

## Pull Request Guardrails
- PR titles must use Conventional Commit format: `type(scope): summary` or `type: summary`.
- Set the correct PR title when opening the PR. Do not rely on fixing it later.
- If a PR title is edited after opening, verify that the semantic PR title check reruns successfully.
- Keep PRs tightly scoped. Do not include unrelated formatting churn, dependency noise, or drive-by refactors.
- Temporary or transitional code must include `TODO(#issue):` with a tracking issue.
- Externally visible web, verifier, or marketplace behavior changes should include screenshots, sample payloads, or an explicit note that there is no visual change.
- PR descriptions must include a summary, motivation, linked issue when applicable, and manual validation plan.
- Before requesting review, run the relevant checks for the files you changed, or note what you could not run.
