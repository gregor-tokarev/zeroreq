<p align="center">
  <img src=".github/icon.png" width="128" alt="Zeroreq icon">
</p>

<h1 align="center">Zeroreq</h1>

<p align="center">zero-friction API client</p>

## macOS releases

Push a semantic version tag such as `v0.1.0` to run the macOS release
workflow. The version in `crates/zeroreq/Cargo.toml` must match the tag.

The workflow builds an Apple Silicon (`arm64`) app, signs it with a Developer ID
Application certificate, enables the hardened runtime, notarizes the app and
DMG with Apple, staples the notarization tickets, and publishes the ZIP, DMG,
update manifest, and SHA-256 checksums to a GitHub release.

Zeroreq checks the latest GitHub release when an installed app starts. Users can
also choose **Zeroreq → Check for Updates…**. Before replacing the app, the
updater verifies the archive checksum, Apple Developer ID signature, expected
Team ID, and Gatekeeper assessment.
