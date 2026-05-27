import fs from "fs-extra";

const REPO_OWNER = "xqd922";
const REPO_NAME = "clash-verge-rev-1x-legacy";
const LEGACY_PRODUCT_NAME = "Clash Verge Rev Legacy";
const LEGACY_PACKAGE_NAME = "clash-verge-legacy";
const LEGACY_IDENTIFIER = "io.github.xqd922.clash-verge-rev-legacy";
const LEGACY_UPDATER_TAG = "updater-legacy";
const LEGACY_WINDOWS_TARGET = "Clash Verge Rev Legacy.exe";
// minisign public key matching the fork's TAURI_PRIVATE_KEY (24BCDCCBFA064E1E).
// Committed tauri.conf.json keeps upstream's pubkey for diff-cleanliness;
// CI overrides it here so released binaries actually verify against the
// signatures produced by our signing key.
const LEGACY_UPDATER_PUBKEY =
  "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDI0QkNEQ0NCRkEwNjRFMUUKUldRZVRnYjZ5OXk4SkR2aEF1emxuaTdRZmM3Q1hYaktEcDFhS3BOZ1dmeDl0VUlDZUY4R3dsSFQK";

const releaseTag = process.argv[2] || process.env.RELEASE_TAG;

if (!releaseTag) {
  throw new Error("release tag is required");
}

const version = releaseTag.replace(/^v/, "");

if (!version.includes("-legacy.")) {
  throw new Error(`legacy release tag expected, got "${releaseTag}"`);
}

async function updateJson(file, transform) {
  const current = await fs.readJson(file);
  const next = transform(current);
  await fs.writeJson(file, next, { spaces: 2 });
  await fs.appendFile(file, "\n");
}

async function updateCargoToml(file) {
  let content = await fs.readFile(file, "utf8");
  content = content.replace(/^version = ".*"$/m, `version = "${version}"`);
  content = content.replace(
    /^repository = ".*"$/m,
    `repository = "https://github.com/${REPO_OWNER}/${REPO_NAME}.git"`
  );
  await fs.writeFile(file, content);
}

async function main() {
  await updateJson("package.json", (current) => ({
    ...current,
    name: LEGACY_PACKAGE_NAME,
    version,
  }));

  await updateCargoToml("src-tauri/Cargo.toml");

  await updateJson("src-tauri/tauri.conf.json", (current) => ({
    ...current,
    package: {
      ...current.package,
      productName: LEGACY_PRODUCT_NAME,
      version,
    },
    build: {
      ...current.build,
      beforeBuildCommand: "pnpm run web:build",
    },
    tauri: {
      ...current.tauri,
      bundle: {
        ...current.tauri.bundle,
        identifier: LEGACY_IDENTIFIER,
        publisher: REPO_OWNER,
      },
      updater: {
        ...current.tauri.updater,
        pubkey: LEGACY_UPDATER_PUBKEY,
        endpoints: [
          `https://mirror.ghproxy.com/https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/download/${LEGACY_UPDATER_TAG}/update-proxy.json`,
          `https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/download/${LEGACY_UPDATER_TAG}/update.json`,
        ],
      },
    },
  }));

  for (const file of [
    "src-tauri/webview2.x64.json",
    "src-tauri/webview2.x86.json",
    "src-tauri/webview2.arm64.json",
  ]) {
    await updateJson(file, (current) => ({
      ...current,
      tauri: {
        ...current.tauri,
        bundle: {
          ...current.tauri.bundle,
          identifier: LEGACY_IDENTIFIER,
        },
        updater: {
          ...current.tauri.updater,
          pubkey: LEGACY_UPDATER_PUBKEY,
          endpoints: [
            `https://mirror.ghproxy.com/https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/download/${LEGACY_UPDATER_TAG}/update-fixed-webview2-proxy.json`,
            `https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/download/${LEGACY_UPDATER_TAG}/update-fixed-webview2.json`,
          ],
        },
      },
    }));
  }

  console.log(
    JSON.stringify(
      {
        releaseTag,
        version,
        productName: LEGACY_PRODUCT_NAME,
        executable: LEGACY_WINDOWS_TARGET,
        updaterTag: LEGACY_UPDATER_TAG,
      },
      null,
      2
    )
  );
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
