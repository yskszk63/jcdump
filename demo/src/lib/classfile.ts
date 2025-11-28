import wasm from "./libjcdump.wasm?url";

let mod: Promise<WebAssembly.Module> | undefined;

type Read = {
  (buf: Uint8Array<ArrayBuffer>): number;
};

type Write = {
  (buf: Uint8Array<ArrayBuffer>): number;
};

type Control = {
  initialize(memory: WebAssembly.Memory): void;
  setup(stdin: Read, stdout: Write, stderr: Write): void;
}

function wasip1(): [WebAssembly.ModuleImports, Control] {
  let memory: WebAssembly.Memory | undefined;
  let stdin: Read = () => { throw new Error("Not set: stdin") };
  let stdout: Write = () => { throw new Error("Not set: stdout") };
  let stderr: Write = () => { throw new Error("Not set: stderr") };

  function assertsItIsMemory(val: typeof memory): asserts val is WebAssembly.Memory {
    if (typeof val !== "undefined") {
      return;
    }
    throw new Error("Memory not set.");
  }

  const mod: WebAssembly.ModuleImports = {
    fd_read: (fd: number, iovsptr: number, iovslen: number, readptr: number) => {
      assertsItIsMemory(memory);

      if (fd !== 0) {
        return 8; // badf
      }

      const view = new DataView(memory.buffer);
      let nread = 0;
      for (let i = 0; i < iovslen; i++) {
        const buf = view.getUint32(iovsptr + (i * 8) + 0, true);
        const len = view.getUint32(iovsptr + (i * 8) + 4, true);
        const b = new Uint8Array(memory.buffer, buf, len);
        nread += stdin(b);
        break;
      }

      view.setUint32(readptr, nread, true);

      return 0;
    },
    fd_write: (fd: number, ciovsptr: number, ciovslen: number, writtenptr: number) => {
      assertsItIsMemory(memory);

      let w: Write;
      switch (fd) {
        case 1:
          w = stdout;
          break;

        case 2:
          w = stderr;
          break;

        default:
          return 8; // badf
      }

      const view = new DataView(memory.buffer);
      let written = 0;
      for (let i = 0; i < ciovslen; i++) {
        const buf = view.getUint32(ciovsptr + (i * 8) + 0, true);
        const len = view.getUint32(ciovsptr + (i * 8) + 4, true);
        let b = new Uint8Array(memory.buffer, buf, len);
        while (b.length > 0) {
          const n = w(b);
          written += n;
          b = b.slice(n);
        }
      }

      view.setUint32(writtenptr, written, true);

      return 0;
    },
    environ_get: () => {
      return 52; // nosys
    },
    environ_sizes_get: (environcptr: number, environbufptr: number) => {
      assertsItIsMemory(memory);

      const view = new DataView(memory.buffer);
      view.setUint32(environcptr, 0, true);
      view.setUint32(environbufptr, 0, true);
      return 0;
    },
    proc_exit: (...args: unknown[]) => {
      console.log("proc_exit", args);
      return 0;
    },
  };

  const ctrl: Control = {
    initialize(_memory) {
      memory = _memory;
    },
    setup(_stdin, _stdout, _stderr) {
      stdin = _stdin;
      stdout = _stdout;
      stderr = _stderr;
    },
  };

  return [mod, ctrl];
}

let cache: Promise<[() => number, Pick<Control, "setup">]> | undefined;

async function getInstance(): Promise<[() => number, Pick<Control, "setup">]> {
  return await (cache ??= (async () => {
    const m = await (mod ??= WebAssembly.compileStreaming(fetch(wasm)));
    const [wasi, ctrl] = wasip1();
    const instance = await WebAssembly.instantiate(m, {
      wasi_snapshot_preview1: wasi,
    });

    const {
      parse,
      memory,
    } = instance.exports;

    if (typeof parse !== "function" || !(memory instanceof WebAssembly.Memory)) {
      throw new Error();
    }

    ctrl.initialize(memory);

    return [parse as () => number, ctrl];
  })());
}

export type Classfile = unknown;

export async function parse(data: ReadableStream<Uint8Array<ArrayBuffer>>): Promise<Classfile> {
  let input = await new Response(data).bytes();
  const output: Uint8Array<ArrayBuffer>[] = [];

  const [parse, ctrl] = await getInstance();

  const stdin: Read = (buf) => {
    const n = Math.min(buf.length, input.length);
    buf.set(input.slice(0, n));
    input = input.slice(n);
    return n;
  };

  const stdout: Write = (buf) => {
    output.push(new Uint8Array(buf));
    return buf.length;
  };

  const stderrenc = new TextDecoder();
  const stderr: Write = (buf) => {
    console.error(stderrenc.decode(buf, { stream: true }));
    return buf.length;
  };

  ctrl.setup(stdin, stdout, stderr);

  const n = parse();
  stderrenc.decode(new Uint8Array(0));
  if (n !== 0) {
    throw new Error();
  }

  return JSON.parse(await new Blob(output).text());
}
