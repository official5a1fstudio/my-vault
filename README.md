# موثوق / Mawthooq

Mawthooq (**موثوق**) is a local password vault and TOTP authenticator built in Rust by **Amin** and **Emad** at 5A1F Studio.

The interface follows the [5A1F Studio website](https://5a1f.vercel.app) visual language: warm paper, ink type, olive structure, coral emphasis, sharp borders and generous editorial spacing.

## What it does

- Keeps account records in a local vault on your device.
- Generates time-based one-time passwords (TOTP).
- Imports accounts from QR images or manual `otpauth://` details.
- On Windows and Linux, can scan a selected window or screen region without a camera.
- On Android, includes a local keyboard-bridge hub.

## Downloads

Each release provides one directly runnable file per platform:

| Platform | File |
|---|---|
| Windows | `my_vault_rs-windows-x86_64.exe` |
| Android | `mawthooq-arm64-v8a.apk` |
| Debian | `my-vault-rs_0.1.1_amd64.deb` |
| Linux | `my_vault_rs-linux-x86_64` |

Release page: https://github.com/official5a1fstudio/my-vault/releases/tag/v0.1.1

## Maturity

Mawthooq is in active development. It has **not** been independently security-audited, has no cloud sync, and does not store real credentials in this repository. Do not rely on it as the only copy of important passwords.

## Build locally

```bash
cargo check --all-targets
cargo test --all-targets
cargo build --release
```

Android builds use `cargo ndk -t arm64-v8a` and are packaged into a signed APK by GitHub Actions.

## Credits

Built by Amin and Emad — 5A1F Studio.
