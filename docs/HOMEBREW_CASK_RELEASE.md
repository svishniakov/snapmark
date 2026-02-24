# Homebrew Cask Release Guide (Custom Tap)

This project is prepared for publishing via your own Homebrew tap (`homebrew-<tapname>`).

## 1) Build release DMG

```bash
./scripts/build_release_dmg.sh
```

Output:
- `build/SnapMark-<version>.dmg`
- SHA256 printed in terminal

## 2) Create/update cask file

```bash
./scripts/generate_cask.sh \
  --repo svishniakov/snapmark \
  --dmg build/SnapMark-<version>.dmg
```

Output:
- `packaging/homebrew/Casks/snapmark.rb`

## 3) Create GitHub release and upload DMG

1. Push a tag:
```bash
git tag v<version>
git push origin v<version>
```
2. GitHub Action `.github/workflows/release.yml` builds and uploads:
   - `build/SnapMark-<version>.dmg`
3. If needed, create/edit release `v<version>` in GitHub UI.

## 4) Publish to your tap

Tap repository naming convention:
- `homebrew-<tapname>` (for example: `homebrew-apps`)
- For this project: `homebrew-snapmark`

In tap repository:
1. Create folder `Casks/` if missing.
2. Copy generated cask:
   - from this repo: `packaging/homebrew/Casks/snapmark.rb`
   - to tap repo: `Casks/snapmark.rb`
3. Commit and push.

## 5) Install check

```bash
brew tap svishniakov/snapmark
brew install --cask snapmark
```

## Notes

- `dock_icon_visible` preference is persisted in:
  - `~/Library/Application Support/snapmark/settings.json`
- If you change app name/bundle identifier/minimum macOS version, update cask metadata accordingly.
