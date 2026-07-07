import { useCallback, useEffect, useState } from "react";
import { RefreshCw, LogIn } from "lucide-react";
import {
  agentRuntimeServerStatus,
  cursorAgentLogin,
  cursorAgentStatus,
  daemonStart,
  daemonStatus,
  daemonStop,
  getAgentRuntimeConfig,
  saveAgentRuntimeConfig,
  type AgentRuntimeConfigDto,
  type AgentRuntimeServerStatusDto,
  type CursorAgentStatusDto,
  type DaemonStatusDto,
} from "../hooks/useTauri";
import { useLocale } from "../i18n/LocaleContext";

export function AgentRuntimeSection() {
  const { m } = useLocale();
  const ar = m.agentRuntime;
  const [cfg, setCfg] = useState<AgentRuntimeConfigDto | null>(null);
  const [serverUrl, setServerUrl] = useState("");
  const [runtimeId, setRuntimeId] = useState("default");
  const [cursorStatus, setCursorStatus] = useState<CursorAgentStatusDto | null>(
    null
  );
  const [daemon, setDaemon] = useState<DaemonStatusDto | null>(null);
  const [serverStatus, setServerStatus] =
    useState<AgentRuntimeServerStatusDto | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [loginMsg, setLoginMsg] = useState<string | null>(null);
  const [daemonMsg, setDaemonMsg] = useState<string | null>(null);
  const [daemonBusy, setDaemonBusy] = useState(false);

  const refreshStatus = useCallback(async () => {
    try {
      const [ca, dm, srv] = await Promise.all([
        cursorAgentStatus().catch(() => null),
        daemonStatus().catch(() => null),
        agentRuntimeServerStatus().catch(() => null),
      ]);
      setCursorStatus(ca);
      setDaemon(dm);
      setServerStatus(srv);
    } catch {
      /* optional panels */
    }
  }, []);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const c = await getAgentRuntimeConfig();
      setCfg(c);
      setServerUrl(c.server_url);
      setRuntimeId(c.runtime_id || "default");
      await refreshStatus();
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, [refreshStatus]);

  useEffect(() => {
    load();
  }, [load]);

  const handleSave = async () => {
    setSaving(true);
    setError(null);
    setSaved(false);
    try {
      const next = await saveAgentRuntimeConfig({
        server_url: serverUrl,
        runtime_id: runtimeId,
      });
      setCfg(next);
      setSaved(true);
      await refreshStatus();
    } catch (e) {
      setError(String(e));
    } finally {
      setSaving(false);
    }
  };

  const handleLogin = async () => {
    setLoginMsg(null);
    try {
      const msg = await cursorAgentLogin();
      setLoginMsg(msg);
    } catch (e) {
      setError(String(e));
    }
  };

  const handleDaemonStart = async () => {
    if (!serverUrl.trim()) {
      setError(ar.daemonNeedServerUrl);
      return;
    }
    setDaemonBusy(true);
    setDaemonMsg(null);
    setError(null);
    try {
      await saveAgentRuntimeConfig({
        server_url: serverUrl,
        runtime_id: runtimeId,
      });
      setSaved(true);
      const result = await daemonStart();
      setDaemonMsg(result.message);
      await refreshStatus();
    } catch (e) {
      setError(String(e));
    } finally {
      setDaemonBusy(false);
    }
  };

  const handleDaemonStop = async () => {
    setDaemonBusy(true);
    setDaemonMsg(null);
    setError(null);
    try {
      const result = await daemonStop();
      setDaemonMsg(result.message);
      await refreshStatus();
    } catch (e) {
      setError(String(e));
    } finally {
      setDaemonBusy(false);
    }
  };

  const canStartDaemon =
    !daemonBusy &&
    !daemon?.poll_running &&
    serverUrl.trim().length > 0;
  const canStopDaemon = !daemonBusy && !!daemon?.poll_running;

  if (loading && !cfg) {
    return (
      <section className="card p-5 text-[13px] text-[var(--text-muted)]">
        {ar.title}…
      </section>
    );
  }

  return (
    <section className="card space-y-4 p-5">
      <div>
        <h3 className="text-[14px] font-semibold">{ar.title}</h3>
        <p className="mt-1 text-[12px] text-[var(--text-muted)]">{ar.intro}</p>
      </div>

      <div className="grid gap-3 sm:grid-cols-2">
        <label className="space-y-1 text-[13px]">
          <span className="font-medium">{ar.serverUrl}</span>
          <input
            className="input w-full"
            value={serverUrl}
            onChange={(e) => {
              setServerUrl(e.target.value);
              setSaved(false);
            }}
            placeholder="http://127.0.0.1:8787"
          />
          <span className="text-[11px] text-[var(--text-muted)]">
            {ar.serverUrlHint}
          </span>
        </label>
        <label className="space-y-1 text-[13px]">
          <span className="font-medium">{ar.runtimeId}</span>
          <input
            className="input w-full font-mono text-[12px]"
            value={runtimeId}
            onChange={(e) => {
              setRuntimeId(e.target.value);
              setSaved(false);
            }}
          />
          <span className="text-[11px] text-[var(--text-muted)]">
            {ar.runtimeIdHint}
          </span>
        </label>
      </div>

      <div className="flex flex-wrap items-center gap-3">
        <button
          type="button"
          className="btn btn-primary"
          disabled={saving}
          onClick={handleSave}
        >
          {saving ? ar.saving : ar.save}
        </button>
        {saved && (
          <span className="text-[13px] text-[var(--accent-green)]">
            {ar.saved}
          </span>
        )}
        <button
          type="button"
          className="btn btn-ghost text-[12px]"
          onClick={refreshStatus}
        >
          <RefreshCw size={14} />
          {ar.refreshStatus}
        </button>
      </div>

      <div className="grid gap-3 md:grid-cols-3">
        <div className="rounded-[var(--radius-sm)] border border-[var(--border)] p-3 text-[12px]">
          <p className="font-medium">{ar.cursorAgent}</p>
          <p className="mt-1 text-[var(--text-muted)]">
            {cursorStatus?.installed
              ? ar.cursorAgentInstalled
              : ar.cursorAgentMissing}
          </p>
          {cursorStatus?.installed && (
            <p
              className={`mt-1 ${
                cursorStatus.logged_in
                  ? "text-[var(--accent-green)]"
                  : "text-[var(--accent-amber)]"
              }`}
            >
              {cursorStatus.logged_in
                ? ar.cursorAgentLoggedIn
                : ar.cursorAgentNotLoggedIn}
            </p>
          )}
          {cursorStatus?.error && !cursorStatus.installed && (
            <p className="mt-2 text-[11px] text-[var(--accent-amber)]">
              {cursorStatus.error}
            </p>
          )}
          {cursorStatus?.output && (
            <pre className="mt-2 max-h-20 overflow-auto whitespace-pre-wrap font-mono text-[10px] text-[var(--text-secondary)]">
              {cursorStatus.output.trim()}
            </pre>
          )}
          <div className="mt-2 flex gap-2">
            <button
              type="button"
              className="btn btn-ghost text-[11px]"
              disabled={!cursorStatus?.installed}
              onClick={handleLogin}
            >
              <LogIn size={13} />
              {ar.cursorAgentLogin}
            </button>
          </div>
          {loginMsg && (
            <p className="mt-2 text-[11px] text-[var(--text-secondary)]">
              {loginMsg}
            </p>
          )}
        </div>

        <div className="rounded-[var(--radius-sm)] border border-[var(--border)] p-3 text-[12px]">
          <p className="font-medium">{ar.daemon}</p>
          <p
            className={`mt-1 ${
              daemon?.poll_running
                ? "text-[var(--accent-green)]"
                : "text-[var(--accent-amber)]"
            }`}
          >
            {daemon?.poll_running ? ar.daemonPollRunning : ar.daemonPollStopped}
            {daemon?.pid ? ` (pid ${daemon.pid})` : ""}
          </p>
          <p
            className={`mt-1 ${
              daemon?.online
                ? "text-[var(--text-muted)]"
                : "text-[var(--accent-amber)]"
            }`}
          >
            {daemon?.online ? ar.daemonOnline : ar.daemonOffline}
          </p>
          {daemon?.detected_clis && daemon.detected_clis.length > 0 && (
            <p className="mt-1 text-[var(--text-muted)]">
              CLI: {daemon.detected_clis.join(", ")}
            </p>
          )}
          {daemon?.last_error && (
            <p className="mt-1 text-[var(--accent-red)]">{daemon.last_error}</p>
          )}
          <div className="mt-2 flex flex-wrap gap-2">
            <button
              type="button"
              className="btn btn-primary text-[11px]"
              disabled={!canStartDaemon}
              onClick={handleDaemonStart}
            >
              {daemonBusy && !daemon?.poll_running
                ? ar.daemonStarting
                : ar.daemonStart}
            </button>
            <button
              type="button"
              className="btn btn-ghost text-[11px]"
              disabled={!canStopDaemon}
              onClick={handleDaemonStop}
            >
              {daemonBusy && daemon?.poll_running
                ? ar.daemonStopping
                : ar.daemonStop}
            </button>
          </div>
          {daemonMsg && (
            <p className="mt-2 text-[11px] text-[var(--text-secondary)]">
              {daemonMsg}
            </p>
          )}
          <p className="mt-2 text-[11px] text-[var(--text-muted)]">
            {ar.daemonHint}{" "}
            <code className="font-mono">{daemon?.foreground_hint}</code>
          </p>
        </div>

        <div className="rounded-[var(--radius-sm)] border border-[var(--border)] p-3 text-[12px]">
          <p className="font-medium">{ar.serverStatus}</p>
          {serverStatus ? (
            <>
              <p
                className={`mt-1 ${
                  serverStatus.server_ok
                    ? "text-[var(--accent-green)]"
                    : "text-[var(--accent-red)]"
                }`}
              >
                {serverStatus.server_ok
                  ? ar.serverOk
                  : ar.serverUnreachable}
              </p>
              <p className="mt-1 text-[var(--text-muted)]">
                storage: {serverStatus.storage}
              </p>
              <p
                className={`mt-1 ${
                  serverStatus.runtime_state === "online"
                    ? "text-[var(--accent-green)]"
                    : "text-[var(--accent-amber)]"
                }`}
              >
                {serverStatus.runtime_state === "online"
                  ? ar.runtimeOnline
                  : ar.runtimeOffline}
              </p>
            </>
          ) : (
            <p className="mt-1 text-[var(--text-muted)]">
              {ar.serverUnreachable}
            </p>
          )}
        </div>
      </div>

      {error && (
        <p className="text-[13px] text-[var(--accent-red)]">{error}</p>
      )}
      {cfg && (
        <p className="text-[12px] text-[var(--text-muted)]">
          {ar.configPath}：{cfg.config_path}
        </p>
      )}
    </section>
  );
}
