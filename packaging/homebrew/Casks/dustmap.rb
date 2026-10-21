cask "dustmap" do
  version "0.1.0"
  sha256 :no_check

  url "https://github.com/abhiudayg/dustmap/releases/download/v#{version}/DustMap_#{version}_aarch64.dmg"
  name "DustMap"
  desc "Storage analyzer and safe, reversible cleaner for macOS"
  homepage "https://github.com/abhiudayg/dustmap"

  depends_on macos: ">= :ventura"

  app "DustMap.app"

  zap trash: [
    "~/Library/Application Support/com.dustmap.desktop/settings.json",
    "~/Library/Application Support/com.dustmap.desktop/audit.jsonl",
    "~/Library/Application Support/com.dustmap.desktop/journal",
    "~/Library/Application Support/com.dustmap.desktop/scan-cache",
    "~/Library/Application Support/com.dustmap.desktop/storage-history",
    "~/Library/Application Support/com.dustmap.desktop/rules-cache",
    "~/Library/Logs/com.dustmap.desktop",
  ]
end
