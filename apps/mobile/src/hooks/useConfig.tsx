import AsyncStorage from "@react-native-async-storage/async-storage";
import { createContext, useCallback, useContext, useEffect, useMemo, useState } from "react";

import { AgentRuntimeClient } from "@/api/client";
import { DEFAULT_CONFIG, type MobileConfig } from "@/api/types";

const STORAGE_KEY = "popsicle.agent-runtime.mobile.config";

type ConfigContextValue = {
  config: MobileConfig;
  loaded: boolean;
  client: AgentRuntimeClient;
  saveConfig: (next: MobileConfig) => Promise<void>;
  refreshConfig: () => Promise<void>;
};

const ConfigContext = createContext<ConfigContextValue | null>(null);

export function ConfigProvider({ children }: { children: React.ReactNode }) {
  const [config, setConfig] = useState<MobileConfig>(DEFAULT_CONFIG);
  const [loaded, setLoaded] = useState(false);

  const refreshConfig = useCallback(async () => {
    const raw = await AsyncStorage.getItem(STORAGE_KEY);
    if (raw) {
      setConfig({ ...DEFAULT_CONFIG, ...JSON.parse(raw) });
    } else {
      setConfig(DEFAULT_CONFIG);
    }
    setLoaded(true);
  }, []);

  useEffect(() => {
    refreshConfig().catch(() => setLoaded(true));
  }, [refreshConfig]);

  const saveConfig = useCallback(async (next: MobileConfig) => {
    await AsyncStorage.setItem(STORAGE_KEY, JSON.stringify(next));
    setConfig(next);
  }, []);

  const client = useMemo(() => new AgentRuntimeClient(config), [config]);

  const value = useMemo(
    () => ({ config, loaded, client, saveConfig, refreshConfig }),
    [config, loaded, client, saveConfig, refreshConfig]
  );

  return <ConfigContext.Provider value={value}>{children}</ConfigContext.Provider>;
}

export function useConfig() {
  const ctx = useContext(ConfigContext);
  if (!ctx) {
    throw new Error("useConfig must be used within ConfigProvider");
  }
  return ctx;
}
