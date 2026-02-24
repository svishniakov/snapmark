cask "snapmark" do
  version "0.1.0"
  sha256 "REPLACE_WITH_SHA256"

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
