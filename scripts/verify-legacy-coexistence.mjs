import fs from "fs-extra";

const resourceChecks =
  process.platform === "win32"
    ? [
        "src-tauri/resources/clash-verge-service-legacy.exe",
        "src-tauri/resources/install-service-legacy.exe",
        "src-tauri/resources/uninstall-service-legacy.exe",
      ]
    : [
        "src-tauri/resources/clash-verge-service-legacy",
        "src-tauri/resources/install-service-legacy",
        "src-tauri/resources/uninstall-service-legacy",
      ];

const checks = [
  {
    file: "src-tauri/src/core/service.rs",
    mustContain: [
      'const SERVICE_URL: &str = "http://127.0.0.1:33210";',
      '"install-service-legacy.exe"',
      '"uninstall-service-legacy.exe"',
      '"install-service-legacy"',
      '"uninstall-service-legacy"',
      "CoreManager::normalize_configured_core()",
    ],
    mustNotContain: [
      '"http://127.0.0.1:33211"',
      'with_file_name("install-service")',
      'with_file_name("uninstall-service")',
    ],
  },
  {
    file: "src-tauri/src/utils/dirs.rs",
    mustContain: [
      '"clash-verge-service-legacy.exe"',
      '"clash-verge-service-legacy"',
    ],
    mustNotContain: ['"clash-verge-service.exe"', '"clash-verge-service"'],
  },
  {
    file: "src-tauri/build.rs",
    mustContain: [
      '"clash-verge-service-legacy.exe"',
      '"install-service-legacy.exe"',
      '"uninstall-service-legacy.exe"',
      '"clash-verge-service-legacy"',
      '"install-service-legacy"',
      '"uninstall-service-legacy"',
    ],
    mustNotContain: [
      '"clash-verge-service.exe",',
      '"install-service.exe",',
      '"uninstall-service.exe",',
      '"clash-verge-service",',
      '"install-service",',
      '"uninstall-service",',
    ],
  },
  {
    file: "src-tauri/template/installer.nsi",
    mustContain: [
      '!define VERGE_SERVICE_NAME "clash_verge_service_legacy"',
      '!define VERGE_SERVICE_PROCESS "clash-verge-service-legacy.exe"',
      '!define LEGACY_PREVIOUS_SERVICE_NAME "clash_verge_service"',
      '!define LEGACY_PREVIOUS_SERVICE_PROCESS "clash-verge-service.exe"',
      'ReadRegStr $R0 HKLM "SYSTEM\\CurrentControlSet\\Services\\${LEGACY_PREVIOUS_SERVICE_NAME}" "ImagePath"',
      '${StrLoc} $R3 $R1 $R2 ">"',
      "!insertmacro RemovePreviousLegacyService",
      "!insertmacro EnsureVergeServiceInstalled",
      'ReadRegStr $R0 HKLM "SYSTEM\\CurrentControlSet\\Services\\${VERGE_SERVICE_NAME}" "ImagePath"',
      "Reinstall ${VERGE_SERVICE_NAME} for current install dir",
      '!insertmacro RemoveServiceByName "${VERGE_SERVICE_NAME}"',
      'Abort "Service Stop Error ($0)"',
      'Abort "Service Remove Error ($0)"',
      'Abort "Check Service Status Error ($0)"',
      'Abort "Install Service Error ($0)"',
      'Abort "Install Service Error (missing install-service-legacy.exe)"',
      'Abort "Start Service Error ($0)"',
      '!insertmacro TryCloseProcess "clash-verge.exe" previous',
    ],
    mustNotContain: [
      'SimpleSC::StartService "clash_verge_service"',
      'FindProcess "clash-verge-service.exe"',
      'KillProcess "clash-verge-service.exe"',
    ],
  },
  {
    file: "src-tauri/src/core/core.rs",
    mustContain: [
      'pub(crate) const DEFAULT_CLASH_CORE: &str = "verge-mihomo";',
      'const ALPHA_CLASH_CORE: &str = "verge-mihomo-alpha";',
      '"verge-mihomo-legacy" => DEFAULT_CLASH_CORE',
      '"verge-mihomo-alpha-legacy" => ALPHA_CLASH_CORE',
      "normalize_configured_core",
    ],
    mustNotContain: [
      'const CLASH_CORES: [&str; 2] = ["verge-mihomo", "verge-mihomo-alpha"];',
    ],
  },
  {
    file: "scripts/check.mjs",
    mustContain: [
      'const LEGACY_SERVICE_NAME = "clash_verge_service_legacy";',
      'const LEGACY_SERVICE_PORT = "33210";',
      'const LEGACY_SERVICE_BIN = "clash-verge-service-legacy";',
      'const LEGACY_INSTALL_BIN = "install-service-legacy";',
      'const LEGACY_UNINSTALL_BIN = "uninstall-service-legacy";',
      'const LEGACY_APP_IDENTIFIER = "io.github.xqd922.clash-verge-rev-legacy";',
      'const LEGACY_IPC_SOCKET = "/tmp/clash-verge-service-legacy.sock";',
      "clash-verge-service-legacy.log",
      "LEGACY_SERVICE_SOURCE_REPO",
      "LEGACY_SERVICE_SOURCE_REF",
      "ensureLegacyServiceResources",
    ],
    mustNotContain: [
      "LEGACY_SERVICE_TAG",
      "DEFAULT_LEGACY_SERVICE_TAG",
      "ensureLegacyWindowsServiceResources",
      "shouldUseLegacyWindowsServiceBundle",
      "resolveLegacyWindowsServiceResources",
    ],
  },
  {
    file: ".github/workflows/release-1x-legacy.yml",
    mustContain: [
      "pnpm legacy:verify-coexistence",
      "uses: softprops/action-gh-release@v3",
    ],
    mustNotContain: [
      "tagName: ${{ needs.prepare-release.outputs.release_tag }}",
      '- "v*-legacy.*"',
    ],
  },
];

let failed = false;

for (const check of checks) {
  const content = await fs.readFile(check.file, "utf8");

  for (const expected of check.mustContain ?? []) {
    if (!content.includes(expected)) {
      console.error(`[legacy-coexistence] ${check.file} missing: ${expected}`);
      failed = true;
    }
  }

  for (const unexpected of check.mustNotContain ?? []) {
    if (content.includes(unexpected)) {
      console.error(
        `[legacy-coexistence] ${check.file} still contains: ${unexpected}`
      );
      failed = true;
    }
  }
}

for (const file of resourceChecks) {
  if (!(await fs.pathExists(file))) {
    console.error(`[legacy-coexistence] missing generated resource: ${file}`);
    failed = true;
  }
}

if (failed) {
  process.exit(1);
}

console.log("[legacy-coexistence] service identifiers are isolated");