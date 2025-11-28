import { useRef, useState } from "react";

import { unjar } from "@/lib/jar";
import type { JarEntry } from "@/lib/jar";

import {
  Empty,
  EmptyContent,
  EmptyHeader,
  EmptyTitle,
} from "@/components/ui/empty";
import { Input } from "./components/ui/input";
import { Table, TableBody, TableCell, TableRow } from "./components/ui/table";
import { Textarea } from "./components/ui/textarea";
import { cn } from "./lib/utils";

type InitialProps = {
  onComplete: (jar: Record<string, JarEntry>) => void;
};

function Initial({ onComplete }: InitialProps): React.ReactNode {
  const [busy, setBusy] = useState(false);

  const handleFileChaned = (event: React.ChangeEvent<HTMLInputElement>) => {
    const [file] = event.target.files ?? [];
    if (typeof file === "undefined") {
      return;
    }

    setBusy(true);
    unjar(file).then(onComplete);
  };

  return (
    <Empty>
      <EmptyHeader>
        <EmptyTitle>Please select Jar file.</EmptyTitle>
      </EmptyHeader>
      <EmptyContent>
        <Input type="file" accept=".jar" onChange={handleFileChaned} disabled={busy} />
      </EmptyContent>
    </Empty>
  );
}

type ViewProps = {
  jar: Record<string, JarEntry>;
};

function View({ jar }: ViewProps): React.ReactNode {
  const textarea = useRef<HTMLTextAreaElement>(null);
  const [text, setText] = useState<string | null>(null);
  const [selected, setSelected] = useState<string | null>(null);
  const handleClicked = (entry: JarEntry) => () => {
    entry.parse().then((data) => {
      textarea.current?.scrollTo({ top: 0, left: 0 });
      setSelected(entry.name);
      setText(JSON.stringify(data, null, 2));
    })
  };

  return (
    <div className="flex h-dvh">
      <nav className="w-md flex-none h-full overflow-y-scroll">
        <Table>
          <TableBody>
          {Object.values(jar).map((entry) => (
            <TableRow key={entry.name} onClick={handleClicked(entry)} className={cn("cursor-pointer", ...selected === entry.name ? ["bg-muted"] : [])}>
              <TableCell title={entry.name}>{entry.name}</TableCell>
            </TableRow>
          ))}
          </TableBody>
        </Table>
      </nav>
      <main className="flex-1 h-full overflow-y-scroll">
        <Textarea ref={textarea} className="size-full font-mono rounded-none resize-none" readOnly value={text ?? ""} />
      </main>
    </div>
  );
}

function App(): React.ReactNode {
  const [jar, setJar] = useState<Record<string, JarEntry> | null>(null);

  if (jar === null) {
    return <Initial onComplete={setJar} />
  }

  return <View jar={jar} />;
}

export default App
