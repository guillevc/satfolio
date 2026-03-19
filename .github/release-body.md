## Install

| Platform | File |
|----------|------|
| macOS (Apple Silicon) | `Satfolio___VERSION___aarch64.dmg` |
| macOS (Intel) | `Satfolio___VERSION___x64.dmg` |
| Linux (x64 .deb) | `Satfolio___VERSION___amd64.deb` |
| Linux (x64 .rpm) | `Satfolio-__VERSION__-1.x86_64.rpm` |
| Linux (x64 .AppImage) | `Satfolio___VERSION___amd64.AppImage` |

> [!NOTE]
> **macOS:** Satfolio isn't signed through Apple's paid developer program, so macOS will show a warning on first launch. The app is open source and every release is verifiably built from this repo — see [Security & trust](https://github.com/guillevc/satfolio#security--trust).
>
> 1. Open the `.dmg` and drag Satfolio to Applications
> 2. Try to open Satfolio — macOS will show a warning and block it
> 3. Open **System Settings → Privacy & Security**
> 4. Under Security, click **Open Anyway**
> 5. Enter your login password and click OK
>
> This is only needed once — after that, Satfolio opens normally. See [Apple's support page](https://support.apple.com/en-us/102445) for more details.

## Verify this release

Every artifact is built in public CI and signed via [Sigstore](https://www.sigstore.dev) through GitHub Actions, with [build provenance attestations](https://docs.github.com/en/actions/security-for-github-actions/using-artifact-attestations).

```sh
# verify your download matches the checksum
shasum -a 256 <filename>

# verify provenance (requires GitHub CLI)
gh attestation verify <filename> --owner guillevc
```
