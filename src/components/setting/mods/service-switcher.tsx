import { KeyedMutator } from "swr";
import { useState } from "react";
import { useLockFn } from "ahooks";
import { useTranslation } from "react-i18next";
import { installService, uninstallService } from "@/services/cmds";
import { Notice } from "@/components/base";
import { LoadingButton } from "@mui/lab";
import { PasswordInput } from "./password-input";
import getSystem from "@/utils/get-system";

interface Props {
  status: "active" | "installed" | "unknown" | "uninstall";
  mutate: KeyedMutator<"active" | "installed" | "unknown" | "uninstall">;
  patchVerge: (value: Partial<IVergeConfig>) => Promise<void>;
  onChangeData: (patch: Partial<IVergeConfig>) => void;
}

export const ServiceSwitcher = (props: Props) => {
  const { status, mutate, patchVerge, onChangeData } = props;
  const isWindows = getSystem() === "windows";
  const isActive = status === "active";
  const isInstalled = status === "installed";
  const isUninstall = status === "uninstall" || status === "unknown";

  const { t } = useTranslation();
  const [serviceLoading, setServiceLoading] = useState(false);
  const [uninstallServiceLoaing, setUninstallServiceLoading] = useState(false);
  const [openInstall, setOpenInstall] = useState(false);
  const [openUninstall, setOpenUninstall] = useState(false);

  async function install(passwd: string) {
    try {
      setOpenInstall(false);
      await installService(passwd);
      await mutate();
      setTimeout(() => {
        mutate();
      }, 2000);
      Notice.success(t("Service Installed Successfully"));
      setServiceLoading(false);
    } catch (err: any) {
      await mutate();
      setTimeout(() => {
        mutate();
      }, 2000);
      Notice.error(err.message || err.toString());
      setServiceLoading(false);
    }
  }

  async function uninstall(passwd: string) {
    try {
      setOpenUninstall(false);
      await uninstallService(passwd);
      // 服务卸载后 TUN 已无法工作,同步把 enable_tun_mode 关掉,
      // 避免下次启动 core 时 verge 配置说要 TUN 但服务不在的矛盾状态
      await patchVerge({ enable_tun_mode: false });
      onChangeData({ enable_tun_mode: false });
      await mutate();
      setTimeout(() => {
        mutate();
      }, 2000);
      Notice.success(t("Service Uninstalled Successfully"));
      setUninstallServiceLoading(false);
    } catch (err: any) {
      await mutate();
      setTimeout(() => {
        mutate();
      }, 2000);
      Notice.error(err.message || err.toString());
      setUninstallServiceLoading(false);
    }
  }

  const onInstallOrEnableService = useLockFn(async () => {
    setServiceLoading(true);
    if (isUninstall) {
      // install service
      if (isWindows) {
        await install("");
      } else {
        setOpenInstall(true);
      }
    } else {
      try {
        // enable or disable service
        // 关服务时同步把 TUN 也关掉,避免 enable_service_mode=false +
        // enable_tun_mode=true 的孤立状态(下次启动 core 时配置矛盾)
        const patch: Partial<IVergeConfig> = isActive
          ? { enable_service_mode: false, enable_tun_mode: false }
          : { enable_service_mode: true };
        await patchVerge(patch);
        onChangeData(patch);
        await mutate();
        setTimeout(() => {
          mutate();
        }, 2000);
        setServiceLoading(false);
      } catch (err: any) {
        await mutate();
        Notice.error(err.message || err.toString());
        setServiceLoading(false);
      }
    }
  });

  const onUninstallService = useLockFn(async () => {
    setUninstallServiceLoading(true);
    if (isWindows) {
      await uninstall("");
    } else {
      setOpenUninstall(true);
    }
  });

  return (
    <>
      {openInstall && <PasswordInput onConfirm={install} />}
      {openUninstall && <PasswordInput onConfirm={uninstall} />}

      <LoadingButton
        size="small"
        variant={isUninstall ? "outlined" : "contained"}
        onClick={onInstallOrEnableService}
        loading={serviceLoading}
      >
        {isActive ? t("Disable") : isInstalled ? t("Enable") : t("Install")}
      </LoadingButton>
      {isInstalled && (
        <LoadingButton
          size="small"
          variant="outlined"
          color="error"
          sx={{ ml: 1 }}
          onClick={onUninstallService}
          loading={uninstallServiceLoaing}
        >
          {t("Uninstall")}
        </LoadingButton>
      )}
    </>
  );
};
