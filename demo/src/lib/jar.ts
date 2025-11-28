import { unzip } from "./zip";
import { parse  } from "./classfile";
import type { Classfile } from "./classfile";

export type JarEntry = {
  name: string;
  parse: () => Promise<Classfile>;
};

export async function unjar(content: Blob): Promise<Record<string, JarEntry>> {
  const zip = await unzip(content);

  const result: Record<string, JarEntry> = {};
  for (const entry of Object.values(zip)) {
    if (!entry.name.endsWith(".class")) {
      continue;
    }

    const name = entry.name.slice(0, -".class".length);
    result[name] = {
      name,
      parse: () => parse(entry.open()),
    };
  }

  return result;
}
