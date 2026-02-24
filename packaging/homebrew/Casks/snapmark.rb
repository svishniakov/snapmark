cask "snapmark" do
  version "1.0.0"
  sha256 "e423f6b890d93343897a136f12aaa5417c07bbb127ee2e4dbf5a6e20445df9ad"

  url "https://github.com/svishniakov/snapmark/releases/download/v#{version}/SnapMark-#{version}.dmg"
  name "SnapMark"
  desc "Screenshot annotation tool for macOS"
  homepage "https://github.com/svishniakov/snapmark"

  depends_on macos: ">= :ventura"

  app "SnapMark.app"

  zap trash: [
    "~/Library/Application Support/snapmark/settings.json",
    "~/Library/Preferences/com.snapmark.app.plist",
  ]
end
