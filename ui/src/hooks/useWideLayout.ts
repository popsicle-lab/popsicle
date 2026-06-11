import { useEffect, useState } from "react";

const WIDE_MIN = 1100;

export function useWideLayout(): boolean {
  const [wide, setWide] = useState(
    typeof window !== "undefined" ? window.innerWidth >= WIDE_MIN : false
  );

  useEffect(() => {
    const onResize = () => setWide(window.innerWidth >= WIDE_MIN);
    window.addEventListener("resize", onResize);
    return () => window.removeEventListener("resize", onResize);
  }, []);

  return wide;
}
