import { wrap, type Remote } from "comlink";
import { useEffect, useState, useRef } from "react";
import type { AiWorkerApi } from "../worker/ai.worker";
import AiWorker from "../worker/ai.worker?worker";

export const useAi = () => {
  const workerRef = useRef<Worker | null>(null);
  const apiRef = useRef<Remote<AiWorkerApi> | null>(null);
  const [isReady, setIsReady] = useState(false);

  useEffect(() => {
    const worker = new AiWorker();
    const api = wrap<AiWorkerApi>(worker);
    workerRef.current = worker;
    apiRef.current = api;

    api.init().then(() => {
      setIsReady(true);
    });

    return () => {
      worker.terminate();
    };
  }, []);

  const getDiscard = async (tehai: number[]) => {
    if (!apiRef.current || !isReady) return null;
    return await apiRef.current.getDiscard(tehai);
  };

  const getShanten = async (tehai: number[]) => {
    if (!apiRef.current || !isReady) return null;
    return await apiRef.current.getShanten(tehai);
  };

  return { isReady, getDiscard, getShanten };
};
