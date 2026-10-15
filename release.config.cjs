// semantic-release for a desktop app: tags + GitHub Release with the tested Linux
// bundles attached. No npm publish — sh-ui is not an npm package.
module.exports = {
  branches: ["main"],
  tagFormat: "v${version}",
  plugins: [
    "@semantic-release/commit-analyzer",
    "@semantic-release/release-notes-generator",
    [
      "@semantic-release/github",
      {
        assets: [
          { path: "release-artifacts/deb/*.deb", label: "Debian package (amd64)" },
          { path: "release-artifacts/rpm/*.rpm", label: "RPM package (x86_64)" },
          { path: "release-artifacts/appimage/*.AppImage", label: "AppImage" },
        ],
      },
    ],
  ],
};
