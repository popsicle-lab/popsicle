import { useCallback, useEffect, useRef, useState } from "react";

import type { AgentRuntimeClient } from "@/api/client";
import type { ChatMessage, ChatSession, RuntimeEvent } from "@/api/types";

const STORAGE_KEY = "popsicle.agent-runtime.mobile.chat.session_id";

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export function useChatSession(client: AgentRuntimeClient, loaded: boolean) {
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [session, setSession] = useState<ChatSession | null>(null);
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const sessionIdRef = useRef<string | null>(null);

  const applySession = useCallback((next: ChatSession) => {
    setSession(next);
    setMessages(next.messages ?? []);
  }, []);

  const refresh = useCallback(async () => {
    const id = sessionIdRef.current;
    if (!id) return;
    const next = await client.getChatSession(id);
    applySession(next);
  }, [applySession, client]);

  const ensureSession = useCallback(async () => {
    if (sessionIdRef.current) {
      await refresh();
      return sessionIdRef.current;
    }
    const created = await client.createChatSession();
    sessionIdRef.current = created.id;
    setSessionId(created.id);
    applySession({ ...created, messages: [] });
    try {
      const { default: AsyncStorage } = await import(
        "@react-native-async-storage/async-storage"
      );
      await AsyncStorage.setItem(STORAGE_KEY, created.id);
    } catch {
      /* optional persistence */
    }
    return created.id;
  }, [applySession, client, refresh]);

  useEffect(() => {
    if (!loaded) return;
    let cancelled = false;
    (async () => {
      try {
        const { default: AsyncStorage } = await import(
          "@react-native-async-storage/async-storage"
        );
        const saved = await AsyncStorage.getItem(STORAGE_KEY);
        if (cancelled) return;
        if (saved) {
          sessionIdRef.current = saved;
          setSessionId(saved);
          const next = await client.getChatSession(saved);
          if (!cancelled) applySession(next);
        }
      } catch {
        /* start fresh on next send */
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [applySession, client, loaded]);

  useEffect(() => {
    const off = client.connectEvents((event: RuntimeEvent) => {
      const id = sessionIdRef.current;
      if (!id) return;
      if (event.session_id && event.session_id !== id) return;
      if (
        event.type === "chat_message" ||
        event.type === "chat_draft_updated" ||
        event.type === "session_bootstrapped"
      ) {
        refresh().catch(() => {});
      }
    });
    return off;
  }, [client, refresh]);

  const waitForAssistant = useCallback(
    async (sinceTs: number) => {
      for (let i = 0; i < 30; i += 1) {
        await sleep(2000);
        const id = sessionIdRef.current;
        if (!id) return;
        const next = await client.getChatSession(id);
        applySession(next);
        const hasReply = (next.messages ?? []).some(
          (m) => m.role === "assistant" && m.ts >= sinceTs
        );
        if (hasReply) return;
      }
    },
    [applySession, client]
  );

  const sendMessage = useCallback(
    async (content: string) => {
      const text = content.trim();
      if (!text) return;
      setBusy(true);
      setError(null);
      try {
        await ensureSession();
        const id = sessionIdRef.current!;
        const sinceTs = Math.floor(Date.now() / 1000);
        const optimistic: ChatMessage = {
          id: `local-${Date.now()}`,
          session_id: id,
          role: "user",
          content: text,
          ts: sinceTs,
        };
        setMessages((prev) => [...prev, optimistic]);
        const result = await client.postChatMessage(id, text);
        if (!result.accepted) {
          setError(result.reason ?? "消息被拒绝");
          setMessages((prev) => prev.filter((m) => m.id !== optimistic.id));
          return;
        }
        if (result.message) {
          setMessages((prev) => [
            ...prev.filter((m) => m.id !== optimistic.id),
            result.message!,
          ]);
        }
        await waitForAssistant(sinceTs);
      } catch (e) {
        setError(String(e));
      } finally {
        setBusy(false);
      }
    },
    [client, ensureSession, waitForAssistant]
  );

  const bootstrap = useCallback(async () => {
    const id = sessionIdRef.current;
    if (!id) {
      setError("请先发送消息");
      return null;
    }
    setBusy(true);
    setError(null);
    try {
      const result = await client.bootstrapChatSession(id);
      if (!result.accepted) {
        setError(result.reason ?? "立项被拒绝");
        return null;
      }
      for (let i = 0; i < 30; i += 1) {
        await sleep(2000);
        const next = await client.getChatSession(id);
        applySession(next);
        if (next.linked_run_id) {
          return next.linked_run_id;
        }
      }
      setError("立项超时，请稍后在进度 Tab 查看");
      return null;
    } catch (e) {
      setError(String(e));
      return null;
    } finally {
      setBusy(false);
    }
  }, [applySession, client]);

  const resetSession = useCallback(async () => {
    sessionIdRef.current = null;
    setSessionId(null);
    setSession(null);
    setMessages([]);
    setError(null);
    try {
      const { default: AsyncStorage } = await import(
        "@react-native-async-storage/async-storage"
      );
      await AsyncStorage.removeItem(STORAGE_KEY);
    } catch {
      /* ignore */
    }
  }, []);

  return {
    sessionId,
    session,
    messages,
    busy,
    error,
    sendMessage,
    bootstrap,
    resetSession,
    refresh,
  };
}
